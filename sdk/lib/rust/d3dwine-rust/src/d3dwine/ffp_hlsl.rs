use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn get_material_colour_source(
    mut mcs: wined3d_material_color_source,
    mut material: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    match mcs as ::core::ffi::c_uint {
        WINED3D_MCS_MATERIAL => {
            return material;
        }
        WINED3D_MCS_COLOR1 => {
            return b"i.diffuse\0".as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_MCS_COLOR2 => {
            return b"i.specular\0".as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            ERR!(
                "Invalid material color source %#x.\n",
                mcs as ::core::ffi::c_uint
            );
            return b"<invalid>\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
}

unsafe extern "C" fn generate_lighting_footer(
    mut buffer: *mut wined3d_string_buffer,
    mut settings: *const wined3d_ffp_vs_settings,
    mut idx: ::core::ffi::c_uint,
    mut legacy_lighting: bool,
) {
    shader_addline(
        buffer,
        b"        diffuse += saturate(dot(dir, normal)) * c.lights[%u].diffuse.xyz * att;\n\0"
            as *const u8 as *const ::core::ffi::c_char,
        idx,
    );
    if (*settings).localviewer() != 0 {
        shader_addline(
            buffer,
            b"        t = dot(normal, ffp_normalize(dir - ffp_normalize(ec_pos.xyz)));\n\0"
                as *const u8 as *const ::core::ffi::c_char,
        );
    } else {
        shader_addline(
            buffer,
            b"        t = dot(normal, ffp_normalize(dir + float3(0.0, 0.0, -1.0)));\n\0"
                as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if (*settings).specular_enable() != 0 {
        shader_addline(
            buffer,
            b"        if (dot(dir, normal) > 0.0 && t > 0.0\0" as *const u8
                as *const ::core::ffi::c_char,
        );
        if legacy_lighting {
            shader_addline(
                buffer,
                b" && c.material.power > 0.0\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        shader_addline(buffer, b")\n\0" as *const u8 as *const ::core::ffi::c_char);
        shader_addline(
            buffer,
            b"            specular += pow(t, c.material.power) * c.lights[%u].specular * att;\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            idx,
        );
    }
}

unsafe extern "C" fn generate_lighting(
    buffer: *mut wined3d_string_buffer,
    settings: *const wined3d_ffp_vs_settings,
    legacy_lighting: bool,
) {
    let mut ambient: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut diffuse: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut specular: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut emissive: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut idx: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

    if (*settings).lighting() == 0 {
        if (*settings).diffuse() != 0 {
            shader_addline(
                buffer,
                b"    o.diffuse = i.diffuse;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            shader_addline(
                buffer,
                b"    o.diffuse = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        shader_addline(
            buffer,
            b"    o.specular = i.specular;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }

    shader_addline(
        buffer,
        b"    float3 ambient = c.ambient_colour.xyz;\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    shader_addline(
        buffer,
        b"    float3 diffuse = 0.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    shader_addline(
        buffer,
        b"    float3 dir, dst;\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    shader_addline(
        buffer,
        b"    float att, t, range, falloff, cos_htheta, cos_hphi;\n\0".as_ptr()
            as *const ::core::ffi::c_char,
    );

    if (*settings).specular_enable() != 0 {
        shader_addline(
            buffer,
            b"    float4 specular = 0.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }

    ambient = get_material_colour_source(
        (*settings).ambient_source() as wined3d_material_color_source,
        b"c.material.ambient\0".as_ptr() as *const ::core::ffi::c_char,
    );
    diffuse = get_material_colour_source(
        (*settings).diffuse_source() as wined3d_material_color_source,
        b"c.material.diffuse\0".as_ptr() as *const ::core::ffi::c_char,
    );
    specular = get_material_colour_source(
        (*settings).specular_source() as wined3d_material_color_source,
        b"c.material.specular\0".as_ptr() as *const ::core::ffi::c_char,
    );
    emissive = get_material_colour_source(
        (*settings).emissive_source() as wined3d_material_color_source,
        b"c.material.emissive\0".as_ptr() as *const ::core::ffi::c_char,
    );

    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < (*settings).point_light_count() {
        shader_addline(
            buffer,
            b"    dir = c.lights[%u].position.xyz - ec_pos.xyz;\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"    dst.z = dot(dir, dir);\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"    dst.y = sqrt(dst.z);\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"    dst.x = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );

        shader_addline(
            buffer,
            b"    range = c.lights[%u].packed_params.x;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );

        if legacy_lighting {
            shader_addline(
                buffer,
                b"    dst.y = (range - dst.y) / range;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    dst.z = dst.y * dst.y;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    if (dst.y > 0.0)\n{\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"        att = dot(dst.xyz, c.lights[%u].attenuation.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
        } else {
            shader_addline(
                buffer,
                b"    if (dst.y <= range)\n{\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"        att = 1.0 / dot(dst.xyz, c.lights[%u].attenuation.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
        }
        shader_addline(
            buffer,
            b"        ambient += c.lights[%u].ambient.xyz * att;\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        if (*settings).normal() != 0 {
            shader_addline(
                buffer,
                b"        dir = ffp_normalize(dir);\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            generate_lighting_footer(buffer, settings, idx, legacy_lighting);
        }
        shader_addline(
            buffer,
            b"    }\n\0".as_ptr() as *const ::core::ffi::c_char,
        );

        idx = idx.wrapping_add(1);
        i = i.wrapping_add(1);
    }

    let mut i_0: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i_0 < (*settings).spot_light_count() {
        shader_addline(
            buffer,
            b"    dir = c.lights[%u].position.xyz - ec_pos.xyz;\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"    dst.z = dot(dir, dir);\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"    dst.y = sqrt(dst.z);\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"    dst.x = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );

        shader_addline(
            buffer,
            b"    range = c.lights[%u].packed_params.x;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"    falloff = c.lights[%u].packed_params.y;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"    cos_htheta = c.lights[%u].packed_params.z;\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"    cos_hphi = c.lights[%u].packed_params.w;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );

        if legacy_lighting {
            shader_addline(
                buffer,
                b"    dst.y = (range - dst.y) / range;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    dst.z = dst.y * dst.y;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    if (dst.y > 0.0)\n{\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            shader_addline(
                buffer,
                b"    if (dst.y <= range)\n{\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        shader_addline(
            buffer,
            b"        dir = ffp_normalize(dir);\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"        t = dot(-dir, ffp_normalize(c.lights[%u].direction.xyz));\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        shader_addline(
            buffer,
            b"        if (t > cos_htheta) att = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"        else if (t <= cos_hphi) att = 0.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(
            buffer,
            b"        else att = pow((t - cos_hphi) / (cos_htheta - cos_hphi), falloff);\n\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        if legacy_lighting {
            shader_addline(
                buffer,
                b"        att *= dot(dst.xyz, c.lights[%u].attenuation.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
        } else {
            shader_addline(
                buffer,
                b"        att /= dot(dst.xyz, c.lights[%u].attenuation.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
        }
        shader_addline(
            buffer,
            b"        ambient += c.lights[%u].ambient.xyz * att;\n\0".as_ptr()
                as *const ::core::ffi::c_char,
            idx,
        );
        if (*settings).normal() != 0 {
            generate_lighting_footer(buffer, settings, idx, legacy_lighting);
        }
        shader_addline(
            buffer,
            b"    }\n\0".as_ptr() as *const ::core::ffi::c_char,
        );

        idx = idx.wrapping_add(1);
        i_0 = i_0.wrapping_add(1);
    }

    let mut i_1: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i_1 < (*settings).directional_light_count() {
        shader_addline(
            buffer,
            b"    ambient += c.lights[%u].ambient.xyz;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );
        if (*settings).normal() != 0 {
            shader_addline(
                buffer,
                b"    att = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    dir = ffp_normalize(c.lights[%u].direction.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
            generate_lighting_footer(buffer, settings, idx, legacy_lighting);
        }

        idx = idx.wrapping_add(1);
        i_1 = i_1.wrapping_add(1);
    }

    let mut i_2: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i_2 < (*settings).parallel_point_light_count() {
        shader_addline(
            buffer,
            b"    ambient += c.lights[%u].ambient.xyz;\n\0".as_ptr() as *const ::core::ffi::c_char,
            idx,
        );
        if (*settings).normal() != 0 {
            shader_addline(
                buffer,
                b"    att = 1.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
            shader_addline(
                buffer,
                b"    dir = ffp_normalize(c.lights[%u].position.xyz);\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
                idx,
            );
            generate_lighting_footer(buffer, settings, idx, legacy_lighting);
        }

        idx = idx.wrapping_add(1);
        i_2 = i_2.wrapping_add(1);
    }

    shader_addline(
        buffer,
        b"    o.diffuse.xyz = %s.xyz * ambient + %s.xyz * diffuse + %s.xyz;\n\0".as_ptr()
            as *const ::core::ffi::c_char,
        ambient,
        diffuse,
        emissive,
    );
    shader_addline(
        buffer,
        b"    o.diffuse.w = %s.w;\n\0".as_ptr() as *const ::core::ffi::c_char,
        diffuse,
    );
    if (*settings).specular_enable() != 0 {
        shader_addline(
            buffer,
            b"    o.specular = %s * specular;\n\0".as_ptr() as *const ::core::ffi::c_char,
            specular,
        );
    } else {
        shader_addline(
            buffer,
            b"    o.specular = i.specular;\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}

unsafe extern "C" fn ffp_hlsl_generate_vertex_shader(settings: *const wined3d_ffp_vs_settings, buffer: *mut wined3d_string_buffer, legacy_lighting: bool) -> bool {
    let mut texcoord: wined3d_string_buffer = core::mem::zeroed();

    /* This must be kept in sync with struct wined3d_ffp_vs_constants. */
    shader_addline(buffer, b"uniform struct\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4x4 modelview_matrices[%u];\n\0" as *const u8 as *const ::core::ffi::c_char, MAX_VERTEX_BLENDS);
    shader_addline(buffer, b"    float4x4 projection_matrix;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4x4 texture_matrices[%u];\n\0" as *const u8 as *const ::core::ffi::c_char, WINED3D_MAX_FFP_TEXTURES);
    shader_addline(buffer, b"    float4 point_params;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    struct\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    {\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 diffuse;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 ambient;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 specular;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 emissive;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float power;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    } material;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 ambient_colour;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    struct\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    {\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 diffuse, specular, ambient;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 position, direction;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 packed_params;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"        float4 attenuation;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    } lights[%u];\n\0" as *const u8 as *const ::core::ffi::c_char, WINED3D_MAX_ACTIVE_LIGHTS);
    shader_addline(buffer, b"} c;\n\0" as *const u8 as *const ::core::ffi::c_char);

    shader_addline(buffer, b"struct input\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 pos : POSITION;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 blend_weight : BLENDWEIGHT;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    uint blend_indices : BLENDINDICES;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float3 normal : NORMAL;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float point_size : PSIZE;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 diffuse : COLOR0;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 specular : COLOR1;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 texcoord[%u] : TEXCOORD;\n\0" as *const u8 as *const ::core::ffi::c_char, WINED3D_MAX_FFP_TEXTURES);
    shader_addline(buffer, b"};\n\n\0" as *const u8 as *const ::core::ffi::c_char);

    shader_addline(buffer, b"struct output\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 pos : POSITION;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 diffuse : COLOR0;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 specular : COLOR1;\n\0" as *const u8 as *const ::core::ffi::c_char);
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        if ((*settings).texgen[i as usize] & 0xffff0000 as ::core::ffi::c_uint) != WINED3DTSS_TCI_PASSTHRU as ::core::ffi::c_uint
            || ((*settings).texcoords() & (1 as ::core::ffi::c_uint) << i) != 0
        {
            shader_addline(buffer, b"    float4 texcoord%u : TEXCOORD%u;\n\0" as *const u8 as *const ::core::ffi::c_char, i, i);
        }
        i = i.wrapping_add(1);
    }
    if (*settings).fog_mode() as ::core::ffi::c_int == WINED3D_FFP_VS_FOG_DEPTH as ::core::ffi::c_int
        || (*settings).fog_mode() as ::core::ffi::c_int == WINED3D_FFP_VS_FOG_RANGE as ::core::ffi::c_int
    {
        shader_addline(buffer, b"    float fogcoord : FOG;\n\0" as *const u8 as *const ::core::ffi::c_char);
    }
    shader_addline(buffer, b"    float point_size : PSIZE;\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"};\n\n\0" as *const u8 as *const ::core::ffi::c_char);

    shader_addline(buffer, b"float3 ffp_normalize(float3 n)\n{\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float lensq = dot(n, n);\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    return lensq == 0.0 ? n : (n * rsqrt(lensq));\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"}\n\n\0" as *const u8 as *const ::core::ffi::c_char);

    shader_addline(buffer, b"void main(in struct input i, out struct output o)\n\0" as *const u8 as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0" as *const u8 as *const ::core::ffi::c_char);

    shader_addline(buffer, b"    i.blend_weight[%u] = 1.0;\n\0" as *const u8 as *const ::core::ffi::c_char, (*settings).vertexblends());

    if (*settings).transformed() != 0 {
        shader_addline(buffer, b"    float4 ec_pos = float4(i.pos.xyz, 1.0);\n\0" as *const u8 as *const ::core::ffi::c_char);
        /* We reuse the projection matrix to undo the transformation from clip
         * coordinates to pixel coordinates. */
        shader_addline(buffer, b"    float4 out_pos = mul(c.projection_matrix, ec_pos);\n\n\0" as *const u8 as *const ::core::ffi::c_char);

        /* Use a ternary; this is not the most natural way to write it but is
         * nicer to the compiler. */
        shader_addline(buffer, b"    o.pos = (i.pos.w == 0.0 ? out_pos : out_pos / i.pos.w);\n\0" as *const u8 as *const ::core::ffi::c_char);
    } else {
        let mut i_0: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        while i_0 < (*settings).vertexblends() {
            shader_addline(buffer, b"    i.blend_weight[%u] -= i.blend_weight[%u];\n\0" as *const u8 as *const ::core::ffi::c_char, (*settings).vertexblends(), i_0);
            i_0 = i_0.wrapping_add(1);
        }

        shader_addline(buffer, b"    float4 ec_pos = 0.0;\n\n\0" as *const u8 as *const ::core::ffi::c_char);
        let mut i_1: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        while i_1 < (*settings).vertexblends() + 1 as ::core::ffi::c_uint {
            shader_addline(buffer, b"    ec_pos += i.blend_weight[%u] * mul(c.modelview_matrices[%u], float4(i.pos.xyz, 1.0));\n\0" as *const u8 as *const ::core::ffi::c_char, i_1, i_1);
            i_1 = i_1.wrapping_add(1);
        }

        shader_addline(buffer, b"    o.pos = mul(c.projection_matrix, ec_pos);\n\0" as *const u8 as *const ::core::ffi::c_char);
        shader_addline(buffer, b"    ec_pos /= ec_pos.w;\n\n\0" as *const u8 as *const ::core::ffi::c_char);
    }

    shader_addline(buffer, b"    float3 normal = 0.0;\n\0" as *const u8 as *const ::core::ffi::c_char);
    if (*settings).normal() != 0 {
        if (*settings).vertexblends() == 0 {
            if (*settings).transformed() != 0 {
                shader_addline(buffer, b"    normal = i.normal;\n\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                shader_addline(buffer, b"    normal = mul((float3x3)c.modelview_matrices[1], i.normal);\n\0" as *const u8 as *const ::core::ffi::c_char);
            }
        } else {
            let mut i_2: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
            while i_2 < (*settings).vertexblends() + 1 as ::core::ffi::c_uint {
                if (*settings).transformed() != 0 {
                    shader_addline(buffer, b"    normal += i.blend_weight[%u] * i.normal;\n\0" as *const u8 as *const ::core::ffi::c_char, i_2);
                } else {
                    shader_addline(buffer, b"    normal += i.blend_weight[%u] * mul((float3x3)c.modelview_matrices[%u], i.normal);\n\0" as *const u8 as *const ::core::ffi::c_char, i_2, i_2);
                }
                i_2 = i_2.wrapping_add(1);
            }
        }

        if (*settings).normalize() != 0 {
            shader_addline(buffer, b"    normal = ffp_normalize(normal);\n\0" as *const u8 as *const ::core::ffi::c_char);
        }
    }

    generate_lighting(buffer, settings, legacy_lighting);

    string_buffer_init(core::ptr::addr_of_mut!(texcoord));
    let mut current_block_94: u64;
    let mut i_3: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i_3 < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        string_buffer_clear(core::ptr::addr_of_mut!(texcoord));

        match (*settings).texgen[i_3 as usize] & 0xffff0000 as ::core::ffi::c_uint {
            WINED3DTSS_TCI_PASSTHRU => {
                if (*settings).texcoords() & (1 as ::core::ffi::c_uint) << i_3 != 0 {
                    shader_addline(
                        core::ptr::addr_of_mut!(texcoord),
                        b"i.texcoord[%u]\0" as *const u8 as *const ::core::ffi::c_char,
                        (*settings).texgen[i_3 as usize] & 0x0000ffff as ::core::ffi::c_uint,
                    );
                    current_block_94 = 3229571381435211107;
                } else {
                    current_block_94 = 15594603006322722090;
                }
            }
            _ => {
                FIXME!("Unhandled texgen %#x.\n", (*settings).texgen[i_3 as usize]);
                current_block_94 = 3229571381435211107;
            }
        }
        match current_block_94 {
            3229571381435211107 => {
                if (*settings).transformed() != 0 {
                    shader_addline(buffer, b"    o.texcoord%u = %s;\n\0" as *const u8 as *const ::core::ffi::c_char, i_3, texcoord.buffer);
                } else {
                    shader_addline(buffer, b"    o.texcoord%u = mul(c.texture_matrices[%u], %s);\n\0" as *const u8 as *const ::core::ffi::c_char, i_3, i_3, texcoord.buffer);
                }
            }
            _ => {}
        }
        i_3 = i_3.wrapping_add(1);
    }
    string_buffer_free(core::ptr::addr_of_mut!(texcoord));

    match ((*settings).fog_mode() as ::core::ffi::c_int) as u32 {
        WINED3D_FFP_VS_FOG_OFF => {
            /* nothing */
        }

        WINED3D_FFP_VS_FOG_FOGCOORD => {
            /* This is FOGTABLEMODE == NONE, FOGVERTEXMODE == NONE.
             * The specular W is used in that case, even if we output fogcoord,
             * so there's no point in outputting fogcoord. */
        }

        WINED3D_FFP_VS_FOG_RANGE => {
            shader_addline(buffer, b"    o.fogcoord = length(ec_pos.xyz);\n\0" as *const u8 as *const ::core::ffi::c_char);
        }

        WINED3D_FFP_VS_FOG_DEPTH => {
            if (*settings).transformed() != 0 {
                shader_addline(buffer, b"    o.fogcoord = ec_pos.z;\n\0" as *const u8 as *const ::core::ffi::c_char);
            } else {
                shader_addline(buffer, b"    o.fogcoord = abs(o.pos.z);\n\0" as *const u8 as *const ::core::ffi::c_char);
            }
        }
        _ => {}
    }

    shader_addline(
        buffer,
        b"    o.point_size = %s / sqrt(c.point_params.x + c.point_params.y * length(ec_pos.xyz) + c.point_params.z * dot(ec_pos.xyz, ec_pos.xyz));\n\0" as *const u8 as *const ::core::ffi::c_char,
        if (*settings).per_vertex_point_size() as ::core::ffi::c_int != 0 {
            b"i.point_size\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"c.point_params.w\0" as *const u8 as *const ::core::ffi::c_char
        },
    );

    shader_addline(buffer, b"}\n\0" as *const u8 as *const ::core::ffi::c_char);

    return true;
}

unsafe extern "C" fn get_fragment_op_arg(
    mut buffer: *mut wined3d_string_buffer,
    mut argnum: ::core::ffi::c_uint,
    mut stage: ::core::ffi::c_uint,
    mut arg: ::core::ffi::c_uint,
) -> *const ::core::ffi::c_char {
    let mut ret: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if arg == ARG_UNUSED as ::core::ffi::c_uint {
        return b"<unused arg>\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let sel: ::core::ffi::c_uint = arg & WINED3DTA_SELECTMASK as ::core::ffi::c_uint;
    if sel == WINED3DTA_DIFFUSE as ::core::ffi::c_uint {
        ret = b"i.diffuse\0" as *const u8 as *const ::core::ffi::c_char;
    } else if sel == WINED3DTA_CURRENT as ::core::ffi::c_uint {
        ret = b"ret\0" as *const u8 as *const ::core::ffi::c_char;
    } else if sel == WINED3DTA_TEXTURE as ::core::ffi::c_uint {
        if stage == 0 as ::core::ffi::c_uint {
            ret = b"tex0\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 1 as ::core::ffi::c_uint {
            ret = b"tex1\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 2 as ::core::ffi::c_uint {
            ret = b"tex2\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 3 as ::core::ffi::c_uint {
            ret = b"tex3\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 4 as ::core::ffi::c_uint {
            ret = b"tex4\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 5 as ::core::ffi::c_uint {
            ret = b"tex5\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 6 as ::core::ffi::c_uint {
            ret = b"tex6\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 7 as ::core::ffi::c_uint {
            ret = b"tex7\0" as *const u8 as *const ::core::ffi::c_char;
        } else {
            ret = b"<invalid texture>\0" as *const u8 as *const ::core::ffi::c_char;
        }
    } else if sel == WINED3DTA_TFACTOR as ::core::ffi::c_uint {
        ret = b"c.texture_factor\0" as *const u8 as *const ::core::ffi::c_char;
    } else if sel == WINED3DTA_SPECULAR as ::core::ffi::c_uint {
        ret = b"i.specular\0" as *const u8 as *const ::core::ffi::c_char;
    } else if sel == WINED3DTA_TEMP as ::core::ffi::c_uint {
        ret = b"temp_reg\0" as *const u8 as *const ::core::ffi::c_char;
    } else if sel == WINED3DTA_CONSTANT as ::core::ffi::c_uint {
        if stage == 0 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[0]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 1 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[1]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 2 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[2]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 3 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[3]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 4 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[4]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 5 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[5]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 6 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[6]\0" as *const u8 as *const ::core::ffi::c_char;
        } else if stage == 7 as ::core::ffi::c_uint {
            ret = b"c.texture_constants[7]\0" as *const u8 as *const ::core::ffi::c_char;
        } else {
            ret = b"<invalid constant>\0" as *const u8 as *const ::core::ffi::c_char;
        }
    } else {
        return b"<unhandled arg>\0" as *const u8 as *const ::core::ffi::c_char;
    }

    if arg & WINED3DTA_COMPLEMENT as ::core::ffi::c_uint != 0 {
        shader_addline(
            buffer,
            b"    arg%u = 1.0 - %s;\n\0" as *const u8 as *const ::core::ffi::c_char,
            argnum,
            ret,
        );
        if argnum == 0 as ::core::ffi::c_uint {
            ret = b"arg0\0" as *const u8 as *const ::core::ffi::c_char;
        } else if argnum == 1 as ::core::ffi::c_uint {
            ret = b"arg1\0" as *const u8 as *const ::core::ffi::c_char;
        } else if argnum == 2 as ::core::ffi::c_uint {
            ret = b"arg2\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }

    if arg & WINED3DTA_ALPHAREPLICATE as ::core::ffi::c_uint != 0 {
        shader_addline(
            buffer,
            b"    arg%u = %s.w;\n\0" as *const u8 as *const ::core::ffi::c_char,
            argnum,
            ret,
        );
        if argnum == 0 as ::core::ffi::c_uint {
            ret = b"arg0\0" as *const u8 as *const ::core::ffi::c_char;
        } else if argnum == 1 as ::core::ffi::c_uint {
            ret = b"arg1\0" as *const u8 as *const ::core::ffi::c_char;
        } else if argnum == 2 as ::core::ffi::c_uint {
            ret = b"arg2\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }

    return ret;
}

unsafe extern "C" fn generate_fragment_op(
    buffer: *mut wined3d_string_buffer,
    stage: ::core::ffi::c_uint,
    color: bool,
    alpha: bool,
    tmp_dst: bool,
    op: ::core::ffi::c_uint,
    dw_arg0: ::core::ffi::c_uint,
    dw_arg1: ::core::ffi::c_uint,
    dw_arg2: ::core::ffi::c_uint,
) {
    let mut dstmask: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut dstreg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut arg0: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut arg1: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut arg2: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();

    if color && alpha {
        dstmask = b"\0".as_ptr() as *const ::core::ffi::c_char;
    } else if color {
        dstmask = b".xyz\0".as_ptr() as *const ::core::ffi::c_char;
    } else {
        dstmask = b".w\0".as_ptr() as *const ::core::ffi::c_char;
    }

    dstreg = if tmp_dst {
        b"temp_reg\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"ret\0".as_ptr() as *const ::core::ffi::c_char
    };

    arg0 = get_fragment_op_arg(buffer, 0, stage, dw_arg0);
    arg1 = get_fragment_op_arg(buffer, 1, stage, dw_arg1);
    arg2 = get_fragment_op_arg(buffer, 2, stage, dw_arg2);

    match op {
        WINED3D_TOP_DISABLE => {
        }

        WINED3D_TOP_SELECT_ARG1 => {
            shader_addline(
                buffer,
                b"    %s%s = %s%s;\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
            );
        }

        WINED3D_TOP_SELECT_ARG2 => {
            shader_addline(
                buffer,
                b"    %s%s = %s%s;\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_MODULATE => {
            shader_addline(
                buffer,
                b"    %s%s = %s%s * %s%s;\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_MODULATE_4X => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * %s%s * 4.0);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_MODULATE_2X => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * %s%s * 2.0);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_ADD => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_ADD_SIGNED => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s + (%s - 0.5)%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_ADD_SIGNED_2X => {
            shader_addline(
                buffer,
                b"    %s%s = saturate((%s%s + (%s - 0.5)%s) * 2.0);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_SUBTRACT => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s - %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
            );
        }

        WINED3D_TOP_ADD_SMOOTH => {
            shader_addline(
                buffer,
                b"    %s%s = saturate((1.0 - %s)%s * %s%s + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
            );
        }

        WINED3D_TOP_BLEND_DIFFUSE_ALPHA => {
            arg0 = get_fragment_op_arg(buffer, 0, stage, WINED3DTA_DIFFUSE);
            shader_addline(
                buffer,
                b"    %s%s = lerp(%s%s, %s%s, %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
                arg0,
            );
        }

        WINED3D_TOP_BLEND_TEXTURE_ALPHA => {
            arg0 = get_fragment_op_arg(buffer, 0, stage, WINED3DTA_TEXTURE);
            shader_addline(
                buffer,
                b"    %s%s = lerp(%s%s, %s%s, %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
                arg0,
            );
        }

        WINED3D_TOP_BLEND_FACTOR_ALPHA => {
            arg0 = get_fragment_op_arg(buffer, 0, stage, WINED3DTA_TFACTOR);
            shader_addline(
                buffer,
                b"    %s%s = lerp(%s%s, %s%s, %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
                arg0,
            );
        }

        WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM => {
            arg0 = get_fragment_op_arg(buffer, 0, stage, WINED3DTA_TEXTURE);
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * (1.0 - %s.w) + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg0,
                arg1,
                dstmask,
            );
        }

        WINED3D_TOP_BLEND_CURRENT_ALPHA => {
            arg0 = get_fragment_op_arg(buffer, 0, stage, WINED3DTA_CURRENT);
            shader_addline(
                buffer,
                b"    %s%s = lerp(%s%s, %s%s, %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
                arg0,
            );
        }

        WINED3D_TOP_MODULATE_ALPHA_ADD_COLOR => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * %s.w + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                arg1,
                dstmask,
            );
        }

        WINED3D_TOP_MODULATE_COLOR_ADD_ALPHA => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * %s%s + %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
                arg1,
            );
        }

        WINED3D_TOP_MODULATE_INVALPHA_ADD_COLOR => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * (1.0 - %s.w) + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                arg1,
                dstmask,
            );
        }

        WINED3D_TOP_MODULATE_INVCOLOR_ADD_ALPHA => {
            shader_addline(
                buffer,
                b"    %s%s = saturate((1.0 - %s)%s * %s%s + %s.w);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
                arg1,
            );
        }

        WINED3D_TOP_BUMPENVMAP | WINED3D_TOP_BUMPENVMAP_LUMINANCE => {
            // These are handled in the first pass, nothing to do.
        }

        WINED3D_TOP_DOTPRODUCT3 => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(dot(%s.xyz - 0.5, %s.xyz - 0.5) * 4.0);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                arg2,
            );
        }

        WINED3D_TOP_MULTIPLY_ADD => {
            shader_addline(
                buffer,
                b"    %s%s = saturate(%s%s * %s%s + %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg1,
                dstmask,
                arg2,
                dstmask,
                arg0,
                dstmask,
            );
        }

        WINED3D_TOP_LERP => {
            // MSDN isn't quite right here.
            shader_addline(
                buffer,
                b"    %s%s = lerp(%s%s, %s%s, %s%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                dstreg,
                dstmask,
                arg2,
                dstmask,
                arg1,
                dstmask,
                arg0,
                dstmask,
            );
        }

        _ => {
            FIXME!("Unhandled operation %#x.\n", op);
        }
    }
}

unsafe extern "C" fn ffp_hlsl_generate_pixel_shader(
    settings: *const ffp_frag_settings,
    buffer: *mut wined3d_string_buffer,
) -> bool {
    let mut tex_map: uint8_t = 0;
    let mut i: ::core::ffi::c_uint;

    /* Find out which textures are read. */
    i = 0;
    'lp1: while i < WINED3D_MAX_FFP_TEXTURES {
        let mut arg0: ::core::ffi::c_uint;
        let mut arg1: ::core::ffi::c_uint;
        let mut arg2: ::core::ffi::c_uint;
        'body1: {
            if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_DISABLE {
                break 'lp1;
            }

            arg0 = (*settings).op[i as usize].carg0() as u32 & WINED3DTA_SELECTMASK;
            arg1 = (*settings).op[i as usize].carg1() as u32 & WINED3DTA_SELECTMASK;
            arg2 = (*settings).op[i as usize].carg2() as u32 & WINED3DTA_SELECTMASK;

            if arg0 == WINED3DTA_TEXTURE
                || arg1 == WINED3DTA_TEXTURE
                || arg2 == WINED3DTA_TEXTURE
                || (i == 0 && (*settings).color_key_enabled() as u32 != 0)
            {
                tex_map = (tex_map as u32 | (1 << i)) as uint8_t;
            }

            match (*settings).op[i as usize].cop() as u32 {
                WINED3D_TOP_BUMPENVMAP_LUMINANCE
                | WINED3D_TOP_BUMPENVMAP
                | WINED3D_TOP_BLEND_TEXTURE_ALPHA
                | WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM => {
                    tex_map = (tex_map as u32 | (1 << i)) as uint8_t;
                }
                _ => {}
            }

            if (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_DISABLE {
                break 'body1;
            }

            arg0 = (*settings).op[i as usize].aarg0() as u32 & WINED3DTA_SELECTMASK;
            arg1 = (*settings).op[i as usize].aarg1() as u32 & WINED3DTA_SELECTMASK;
            arg2 = (*settings).op[i as usize].aarg2() as u32 & WINED3DTA_SELECTMASK;

            if arg0 == WINED3DTA_TEXTURE
                || arg1 == WINED3DTA_TEXTURE
                || arg2 == WINED3DTA_TEXTURE
            {
                tex_map = (tex_map as u32 | (1 << i)) as uint8_t;
            }
        }
        i += 1;
    }

    i = 0;
    'lp2: while i < WINED3D_MAX_FFP_TEXTURES {
        let mut sampler_type: *const ::core::ffi::c_char =
            core::ptr::null::<::core::ffi::c_char>();
        'body2: {
            if (tex_map as u32 & (1 << i)) == 0 {
                break 'body2;
            }

            match (*settings).op[i as usize].tex_type() as u32 {
                WINED3D_GL_RES_TYPE_TEX_1D => {
                    sampler_type = b"1D\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_2D => {
                    sampler_type = b"2D\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_3D => {
                    sampler_type = b"3D\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_CUBE => {
                    sampler_type = b"CUBE\0".as_ptr() as *const ::core::ffi::c_char;
                }
                _ => {
                    FIXME!(
                        "Unhandled sampler type %#x.\n",
                        (*settings).op[i as usize].tex_type() as u32
                    );
                    sampler_type = core::ptr::null::<::core::ffi::c_char>();
                }
            }
            if !sampler_type.is_null() {
                shader_addline(
                    buffer,
                    b"sampler%s ps_sampler%u : register(s%u);\n\0".as_ptr() as *const ::core::ffi::c_char,
                    sampler_type,
                    i,
                    i,
                );
            }

            shader_addline(
                buffer,
                b"static float4 tex%u;\n\0".as_ptr() as *const ::core::ffi::c_char,
                i,
            );
        }
        i += 1;
    }

    /* This must be kept in sync with struct wined3d_ffp_vs_constants. */
    shader_addline(buffer, b"uniform struct\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(
        buffer,
        b"    float4 texture_constants[%u];\n\0".as_ptr() as *const ::core::ffi::c_char,
        WINED3D_MAX_FFP_TEXTURES,
    );
    shader_addline(buffer, b"    float4 texture_factor;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 specular_enable;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 color_key[2];\n\0".as_ptr() as *const ::core::ffi::c_char);
    /* Bumpenv constants are manually packed. */
    shader_addline(
        buffer,
        b"    float4 bumpenv_matrices[%u];\n\0".as_ptr() as *const ::core::ffi::c_char,
        WINED3D_MAX_FFP_TEXTURES,
    );
    shader_addline(buffer, b"    float4 bumpenv_lscale[2];\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 bumpenv_loffset[2];\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"} c;\n\0".as_ptr() as *const ::core::ffi::c_char);

    shader_addline(buffer, b"struct input\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 pos : POSITION;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 diffuse : COLOR0;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 specular : COLOR1;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(
        buffer,
        b"    float4 texcoord[%u] : TEXCOORD;\n\0".as_ptr() as *const ::core::ffi::c_char,
        WINED3D_MAX_FFP_TEXTURES,
    );
    shader_addline(buffer, b"};\n\n\0".as_ptr() as *const ::core::ffi::c_char);

    shader_addline(buffer, b"struct output\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"{\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 colour : COLOR;\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"};\n\n\0".as_ptr() as *const ::core::ffi::c_char);

    shader_addline(
        buffer,
        b"void main(in struct input i, out struct output o)\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    shader_addline(buffer, b"{\n\0".as_ptr() as *const ::core::ffi::c_char);

    shader_addline(
        buffer,
        b"    float4 texcoord[%u];\n\0".as_ptr() as *const ::core::ffi::c_char,
        WINED3D_MAX_FFP_TEXTURES,
    );
    shader_addline(buffer, b"    float4 temp_reg = 0.0;\n\n\0".as_ptr() as *const ::core::ffi::c_char);
    shader_addline(buffer, b"    float4 ret, arg0, arg1, arg2;\n\n\0".as_ptr() as *const ::core::ffi::c_char);

    i = 0;
    while i < WINED3D_MAX_FFP_TEXTURES {
        if (tex_map as u32 & (1 << i)) != 0 {
            if (*settings).pointsprite() as u32 != 0
                || ((*settings).texcoords_initialized as u32 & (1 << i)) != 0
            {
                shader_addline(
                    buffer,
                    b"    texcoord[%u] = i.texcoord[%u];\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                    i,
                );
            } else {
                shader_addline(
                    buffer,
                    b"    texcoord[%u] = 0.0;\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                );
            }
        }
        i += 1;
    }

    /* Texture sampling. */

    i = 0;
    'lp3: while i < WINED3D_MAX_FFP_TEXTURES && (*settings).op[i as usize].cop() as u32 != WINED3D_TOP_DISABLE {
        let mut texture_function: *const ::core::ffi::c_char =
            core::ptr::null::<::core::ffi::c_char>();
        let mut coord_mask: *const ::core::ffi::c_char = core::ptr::null::<::core::ffi::c_char>();
        let mut proj: bool = false;
        'body3: {
            if (tex_map as u32 & (1 << i)) == 0 {
                break 'body3;
            }

            if (*settings).op[i as usize].projected() as u32 == WINED3D_PROJECTION_NONE {
                proj = false;
            } else if (*settings).op[i as usize].projected() as u32 == WINED3D_PROJECTION_COUNT4
                || (*settings).op[i as usize].projected() as u32 == WINED3D_PROJECTION_COUNT3
            {
                proj = true;
            } else {
                proj = true;
            }

            match (*settings).op[i as usize].tex_type() as u32 {
                WINED3D_GL_RES_TYPE_TEX_1D => {
                    texture_function = b"tex1D\0".as_ptr() as *const ::core::ffi::c_char;
                    coord_mask = b"x\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_2D => {
                    texture_function = b"tex2D\0".as_ptr() as *const ::core::ffi::c_char;
                    coord_mask = b"xy\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_3D => {
                    texture_function = b"tex3D\0".as_ptr() as *const ::core::ffi::c_char;
                    coord_mask = b"xyz\0".as_ptr() as *const ::core::ffi::c_char;
                }
                WINED3D_GL_RES_TYPE_TEX_CUBE => {
                    texture_function = b"texCUBE\0".as_ptr() as *const ::core::ffi::c_char;
                    coord_mask = b"xyz\0".as_ptr() as *const ::core::ffi::c_char;
                }
                _ => {
                    FIXME!(
                        "Unhandled texture type %#x.\n",
                        (*settings).op[i as usize].tex_type() as u32
                    );
                    texture_function = b"\0".as_ptr() as *const ::core::ffi::c_char;
                    coord_mask = b"xyzw\0".as_ptr() as *const ::core::ffi::c_char;
                    proj = false;
                }
            }

            if proj {
                coord_mask = b"xyzw\0".as_ptr() as *const ::core::ffi::c_char;
            }

            if i > 0
                && ((*settings).op[(i - 1) as usize].cop() as u32 == WINED3D_TOP_BUMPENVMAP
                    || (*settings).op[(i - 1) as usize].cop() as u32 == WINED3D_TOP_BUMPENVMAP_LUMINANCE)
            {
                shader_addline(
                    buffer,
                    b"ret.xy = mul(transpose((float2x2)c.bumpenv_matrices[%u]), tex%u.xy);\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i - 1,
                    i - 1,
                );

                /* With projective textures, texbem only divides the static texture
                 * coordinate, not the displacement, so multiply the displacement
                 * with the dividing parameter before sampling. */
                if (*settings).op[i as usize].projected() as u32 != WINED3D_PROJECTION_NONE {
                    if (*settings).op[i as usize].projected() as u32 == WINED3D_PROJECTION_COUNT4 {
                        shader_addline(
                            buffer,
                            b"ret.xy = (ret.xy * texcoord[%u].w) + texcoord[%u].xy;\n\0".as_ptr() as *const ::core::ffi::c_char,
                            i,
                            i,
                        );
                        shader_addline(
                            buffer,
                            b"ret.zw = texcoord[%u].ww;\n\0".as_ptr() as *const ::core::ffi::c_char,
                            i,
                        );
                    } else {
                        shader_addline(
                            buffer,
                            b"ret.xy = (ret.xy * texcoord[%u].z) + texcoord[%u].xy;\n\0".as_ptr() as *const ::core::ffi::c_char,
                            i,
                            i,
                        );
                        shader_addline(
                            buffer,
                            b"ret.zw = texcoord[%u].zz;\n\0".as_ptr() as *const ::core::ffi::c_char,
                            i,
                        );
                    }
                } else {
                    shader_addline(
                        buffer,
                        b"ret = texcoord[%u] + ret.xyxy;\n\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                    );
                }

                shader_addline(
                    buffer,
                    b"tex%u = %s%s(ps_sampler%u, ret.%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                    texture_function,
                    (if proj { b"proj\0".as_ptr() } else { b"\0".as_ptr() }) as *const ::core::ffi::c_char,
                    i,
                    coord_mask,
                );

                if (*settings).op[(i - 1) as usize].cop() as u32 == WINED3D_TOP_BUMPENVMAP_LUMINANCE {
                    shader_addline(
                        buffer,
                        b"tex%u *= saturate(tex%u.z * c.bumpenv_lscale[%u][%u] + c.bumpenv_loffset[%u][%u]);\n\0".as_ptr() as *const ::core::ffi::c_char,
                        i,
                        i - 1,
                        (i - 1) / 4,
                        (i - 1) % 4,
                        (i - 1) / 4,
                        (i - 1) % 4,
                    );
                }
            } else if (*settings).op[i as usize].projected() as u32 == WINED3D_PROJECTION_COUNT3 {
                shader_addline(
                    buffer,
                    b"    tex%u = %s%s(ps_sampler%u, texcoord[%u].xyzz);\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                    texture_function,
                    (if proj { b"proj\0".as_ptr() } else { b"\0".as_ptr() }) as *const ::core::ffi::c_char,
                    i,
                    i,
                );
            } else {
                shader_addline(
                    buffer,
                    b"    tex%u = %s%s(ps_sampler%u, texcoord[%u].%s);\n\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                    texture_function,
                    (if proj { b"proj\0".as_ptr() } else { b"\0".as_ptr() }) as *const ::core::ffi::c_char,
                    i,
                    i,
                    coord_mask,
                );
            }
        }
        i += 1;
    }

    if (*settings).color_key_enabled() as u32 != 0 {
        shader_addline(
            buffer,
            b"    if (all(tex0 >= c.color_key[0]) && all(tex0 < c.color_key[1]))\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
        shader_addline(buffer, b"        discard;\n\0".as_ptr() as *const ::core::ffi::c_char);
    }

    shader_addline(buffer, b"    ret = i.diffuse;\n\0".as_ptr() as *const ::core::ffi::c_char);

    i = 0;
    'lp4: while i < WINED3D_MAX_FFP_TEXTURES {
        let mut op_equal: bool = false;
        'body4: {
            if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_DISABLE {
                break 'lp4;
            }

            if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_SELECT_ARG1
                && (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_SELECT_ARG1
            {
                op_equal = (*settings).op[i as usize].carg1() as u32 == (*settings).op[i as usize].aarg1() as u32;
            } else if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_SELECT_ARG1
                && (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_SELECT_ARG2
            {
                op_equal = (*settings).op[i as usize].carg1() as u32 == (*settings).op[i as usize].aarg2() as u32;
            } else if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_SELECT_ARG2
                && (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_SELECT_ARG1
            {
                op_equal = (*settings).op[i as usize].carg2() as u32 == (*settings).op[i as usize].aarg1() as u32;
            } else if (*settings).op[i as usize].cop() as u32 == WINED3D_TOP_SELECT_ARG2
                && (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_SELECT_ARG2
            {
                op_equal = (*settings).op[i as usize].carg2() as u32 == (*settings).op[i as usize].aarg2() as u32;
            } else {
                op_equal = (*settings).op[i as usize].aop() as u32 == (*settings).op[i as usize].cop() as u32
                    && (*settings).op[i as usize].carg0() as u32 == (*settings).op[i as usize].aarg0() as u32
                    && (*settings).op[i as usize].carg1() as u32 == (*settings).op[i as usize].aarg1() as u32
                    && (*settings).op[i as usize].carg2() as u32 == (*settings).op[i as usize].aarg2() as u32;
            }

            if (*settings).op[i as usize].aop() as u32 == WINED3D_TOP_DISABLE {
                generate_fragment_op(
                    buffer,
                    i,
                    true,
                    false,
                    (*settings).op[i as usize].tmp_dst() as u32 != 0,
                    (*settings).op[i as usize].cop() as u32,
                    (*settings).op[i as usize].carg0() as u32,
                    (*settings).op[i as usize].carg1() as u32,
                    (*settings).op[i as usize].carg2() as u32,
                );
            } else if op_equal {
                generate_fragment_op(
                    buffer,
                    i,
                    true,
                    true,
                    (*settings).op[i as usize].tmp_dst() as u32 != 0,
                    (*settings).op[i as usize].cop() as u32,
                    (*settings).op[i as usize].carg0() as u32,
                    (*settings).op[i as usize].carg1() as u32,
                    (*settings).op[i as usize].carg2() as u32,
                );
            } else if (*settings).op[i as usize].cop() as u32 != WINED3D_TOP_BUMPENVMAP
                && (*settings).op[i as usize].cop() as u32 != WINED3D_TOP_BUMPENVMAP_LUMINANCE
            {
                generate_fragment_op(
                    buffer,
                    i,
                    true,
                    false,
                    (*settings).op[i as usize].tmp_dst() as u32 != 0,
                    (*settings).op[i as usize].cop() as u32,
                    (*settings).op[i as usize].carg0() as u32,
                    (*settings).op[i as usize].carg1() as u32,
                    (*settings).op[i as usize].carg2() as u32,
                );
                generate_fragment_op(
                    buffer,
                    i,
                    false,
                    true,
                    (*settings).op[i as usize].tmp_dst() as u32 != 0,
                    (*settings).op[i as usize].aop() as u32,
                    (*settings).op[i as usize].aarg0() as u32,
                    (*settings).op[i as usize].aarg1() as u32,
                    (*settings).op[i as usize].aarg2() as u32,
                );
            }
        }
        i += 1;
    }

    shader_addline(
        buffer,
        b"    o.colour = i.specular * c.specular_enable + ret;\n\0".as_ptr() as *const ::core::ffi::c_char,
    );

    shader_addline(buffer, b"}\n\0".as_ptr() as *const ::core::ffi::c_char);

    return true;
}

unsafe extern "C" fn compile_hlsl_shader(
    mut hlsl: *const wined3d_string_buffer,
    mut sm1: *mut vkd3d_shader_code,
    mut profile: *const ::core::ffi::c_char,
) -> bool {
    let mut hlsl_source_info: vkd3d_shader_hlsl_source_info = vkd3d_shader_hlsl_source_info {
        type_: VKD3D_SHADER_STRUCTURE_TYPE_HLSL_SOURCE_INFO,
        next: ::core::ptr::null::<::core::ffi::c_void>(),
        entry_point: ::core::ptr::null::<::core::ffi::c_char>(),
        secondary_code: vkd3d_shader_code {
            code: ::core::ptr::null::<::core::ffi::c_void>(),
            size: 0,
        },
        profile: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut compile_info: vkd3d_shader_compile_info = vkd3d_shader_compile_info {
        type_: VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO,
        next: ::core::ptr::null::<::core::ffi::c_void>(),
        source: vkd3d_shader_code {
            code: ::core::ptr::null::<::core::ffi::c_void>(),
            size: 0,
        },
        source_type: VKD3D_SHADER_SOURCE_NONE,
        target_type: VKD3D_SHADER_TARGET_NONE,
        options: ::core::ptr::null::<vkd3d_shader_compile_option>(),
        option_count: 0,
        log_level: VKD3D_SHADER_LOG_NONE,
        source_name: ::core::ptr::null::<::core::ffi::c_char>(),
    };
    let mut messages: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = 0;

    compile_info.source.code = (*hlsl).buffer as *const ::core::ffi::c_void;
    compile_info.source.size = (*hlsl).content_size as usize;
    compile_info.source_type = VKD3D_SHADER_SOURCE_HLSL;
    compile_info.target_type = VKD3D_SHADER_TARGET_D3D_BYTECODE;
    compile_info.log_level = VKD3D_SHADER_LOG_WARNING;

    compile_info.next = core::ptr::addr_of_mut!(hlsl_source_info) as *const ::core::ffi::c_void;
    hlsl_source_info.profile = profile;

    ret = vkd3d_shader_compile(
        core::ptr::addr_of_mut!(compile_info),
        sm1,
        core::ptr::addr_of_mut!(messages),
    );
    if !messages.is_null() && *messages != 0 && ((FIXME_ON!(__wine_dbch_d3d_shader)) != 0) {
        let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();

        ptr = messages;
        end = ptr.add(strlen(ptr) as usize);
        loop {
            line = wined3d_get_line(core::ptr::addr_of_mut!(ptr), end);
            if line.is_null() {
                break;
            }
            FIXME!(
                "    %.*s",
                ptr.offset_from(line) as ::core::ffi::c_int,
                line
            );
        }
        FIXME!("\n");
    }
    vkd3d_shader_free_messages(messages);

    if ret < 0 {
        ERR!("Failed to compile HLSL, ret %d.\n", ret);
        return false;
    }

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn ffp_hlsl_compile_vs(settings: *const wined3d_ffp_vs_settings, shader_desc: *mut wined3d_shader_desc, device: *mut wined3d_device) -> bool {
    let mut string: wined3d_string_buffer = core::mem::zeroed();
    let mut sm1: vkd3d_shader_code = core::mem::zeroed();

    if ((string_buffer_init(&mut string as *mut wined3d_string_buffer)) == 0) {
        return false;
    }

    if !ffp_hlsl_generate_vertex_shader(
        settings,
        &mut string as *mut wined3d_string_buffer,
        (((*(*device).wined3d).flags & WINED3D_LEGACY_FFP_LIGHTING) != 0),
    ) {
        string_buffer_free(&mut string as *mut wined3d_string_buffer);
        return false;
    }

    if !compile_hlsl_shader(
        &mut string as *mut wined3d_string_buffer,
        &mut sm1 as *mut vkd3d_shader_code,
        b"vs_2_a\0".as_ptr() as *const core::ffi::c_char,
    ) {
        string_buffer_free(&mut string as *mut wined3d_string_buffer);
        return false;
    }
    string_buffer_free(&mut string as *mut wined3d_string_buffer);

    (*shader_desc).byte_code = sm1.code as *const DWORD;
    (*shader_desc).byte_code_size = !(0 as usize);
    return true;
}
#[no_mangle]
pub unsafe extern "C" fn ffp_hlsl_compile_ps(settings: *const ffp_frag_settings, shader_desc: *mut wined3d_shader_desc) -> bool {
    let mut string: wined3d_string_buffer = core::mem::zeroed();
    let mut sm1: vkd3d_shader_code = core::mem::zeroed();

    if ((string_buffer_init(core::ptr::addr_of_mut!(string))) == 0) {
        return false;
    }

    if !ffp_hlsl_generate_pixel_shader(settings, core::ptr::addr_of_mut!(string)) {
        string_buffer_free(core::ptr::addr_of_mut!(string));
        return false;
    }

    if !compile_hlsl_shader(core::ptr::addr_of_mut!(string), core::ptr::addr_of_mut!(sm1), b"ps_2_a\0".as_ptr() as *const i8) {
        string_buffer_free(core::ptr::addr_of_mut!(string));
        return false;
    }
    string_buffer_free(core::ptr::addr_of_mut!(string));

    (*shader_desc).byte_code = sm1.code as *const DWORD;
    (*shader_desc).byte_code_size = !(0 as usize);
    return true;
}
