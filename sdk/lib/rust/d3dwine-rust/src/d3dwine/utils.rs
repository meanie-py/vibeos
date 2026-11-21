use crate::d3dwine::glsl_shader::print_glsl_info_log;
use crate::d3dwine::glsl_shader::shader_glsl_validate_link;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_51 {
    pub src_format: wined3d_format_id,
    pub conversion: wined3d_color_key_conversion,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_59 {
    pub id: wined3d_format_id,
    pub fixup: *const ::core::ffi::c_char,
    pub legacy: BOOL,
    pub extension: wined3d_gl_extension,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_45 {
    pub id: wined3d_format_id,
    pub idx: ::core::ffi::c_uint,
}


static mut format_index_remap: [C2RustUnnamed_45; 24] = [
    C2RustUnnamed_45 {
        id: WINED3DFMT_UYVY,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_YUY2,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_YV12,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 2 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_DXT1,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 3 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_DXT2,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 4 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_DXT3,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 5 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_DXT4,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 6 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_DXT5,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 7 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_MULTI2_ARGB8,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_G8R8_G8B8,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 9 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_R8G8_B8G8,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_ATI1N,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 11 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_ATI2N,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 12 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_INST,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 13 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_NVDB,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 14 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_NVHU,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 15 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_NVHS,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 16 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_INTZ,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 17 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_RESZ,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 18 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_NULL,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 19 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_R16,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 20 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_AL16,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 21 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_NV12,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 22 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
    C2RustUnnamed_45 {
        id: WINED3DFMT_ATOC,
        idx: (WINED3DFMT_BC7_UNORM_SRGB as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 23 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    },
];

static mut formats: [wined3d_format_channels; 78] = [
    wined3d_format_channels {
        id: WINED3DFMT_UNKNOWN,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 0 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_UYVY,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_YUY2,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_YV12,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_NV12,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_DXT1,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_DXT2,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_DXT3,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_DXT4,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_DXT5,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_MULTI2_ARGB8,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_G8R8_G8B8,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8_B8G8,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8_SNORM_Cx,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R11G11B10_FLOAT,
        red_size: 11 as DWORD,
        green_size: 11 as DWORD,
        blue_size: 10 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 11 as DWORD,
        blue_offset: 22 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"FFF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_P8_UINT_A8_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 8 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_P8_UINT,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_B8G8R8_UNORM,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 16 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 3 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B5G6R5_UNORM,
        red_size: 5 as DWORD,
        green_size: 6 as DWORD,
        blue_size: 5 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 11 as DWORD,
        green_offset: 5 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B5G5R5X1_UNORM,
        red_size: 5 as DWORD,
        green_size: 5 as DWORD,
        blue_size: 5 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 10 as DWORD,
        green_offset: 5 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B5G5R5A1_UNORM,
        red_size: 5 as DWORD,
        green_size: 5 as DWORD,
        blue_size: 5 as DWORD,
        alpha_size: 1 as DWORD,
        red_offset: 10 as DWORD,
        green_offset: 5 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 15 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B4G4R4A4_UNORM,
        red_size: 4 as DWORD,
        green_size: 4 as DWORD,
        blue_size: 4 as DWORD,
        alpha_size: 4 as DWORD,
        red_offset: 8 as DWORD,
        green_offset: 4 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 12 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B2G3R3_UNORM,
        red_size: 3 as DWORD,
        green_size: 3 as DWORD,
        blue_size: 2 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 5 as DWORD,
        green_offset: 2 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_A8_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"u\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B2G3R3A8_UNORM,
        red_size: 3 as DWORD,
        green_size: 3 as DWORD,
        blue_size: 2 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 5 as DWORD,
        green_offset: 2 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 8 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B4G4R4X4_UNORM,
        red_size: 4 as DWORD,
        green_size: 4 as DWORD,
        blue_size: 4 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 8 as DWORD,
        green_offset: 4 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8B8X8_UNORM,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 16 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_B10G10R10A2_UNORM,
        red_size: 10 as DWORD,
        green_size: 10 as DWORD,
        blue_size: 10 as DWORD,
        alpha_size: 2 as DWORD,
        red_offset: 20 as DWORD,
        green_offset: 10 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 30 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_L8_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_L8A8_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 8 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_L4A4_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 4 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 4 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_L16_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
        red_size: 5 as DWORD,
        green_size: 5 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 5 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R10G11B11_SNORM,
        red_size: 10 as DWORD,
        green_size: 11 as DWORD,
        blue_size: 11 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 10 as DWORD,
        blue_offset: 21 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"iii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_R10G10B10_SNORM_A2_UNORM,
        red_size: 10 as DWORD,
        green_size: 10 as DWORD,
        blue_size: 10 as DWORD,
        alpha_size: 2 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 10 as DWORD,
        blue_offset: 20 as DWORD,
        alpha_offset: 30 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"iiiu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_D16_LOCKABLE,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 16 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_D32_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 32 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_S1_UINT_D15_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 15 as BYTE,
        stencil_size: 1 as BYTE,
        channels: b"SD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_X8D24_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 24 as BYTE,
        stencil_size: 0 as BYTE,
        channels: b"XD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_S4X4_UINT_D24_UNORM,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 24 as BYTE,
        stencil_size: 4 as BYTE,
        channels: b"SXD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_S8_UINT_D24_FLOAT,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 24 as BYTE,
        stencil_size: 8 as BYTE,
        channels: b"SD\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_format_channels {
        id: WINED3DFMT_ATI1N,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_ATI2N,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_NVDB,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 0 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_ATOC,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 0 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_INST,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 0 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_INTZ,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 24 as BYTE,
        stencil_size: 8 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_RESZ,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 0 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_NVHU,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_NVHS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_NULL,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 16 as DWORD,
        alpha_offset: 24 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R16,
        red_size: 16 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_AL16,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 16 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 16 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R9G9B9E5_SHAREDEXP,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R32G32B32A32_TYPELESS,
        red_size: 32 as DWORD,
        green_size: 32 as DWORD,
        blue_size: 32 as DWORD,
        alpha_size: 32 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 32 as DWORD,
        blue_offset: 64 as DWORD,
        alpha_offset: 96 as DWORD,
        bpp: 16 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R32G32B32_TYPELESS,
        red_size: 32 as DWORD,
        green_size: 32 as DWORD,
        blue_size: 32 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 32 as DWORD,
        blue_offset: 64 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 12 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R16G16B16A16_TYPELESS,
        red_size: 16 as DWORD,
        green_size: 16 as DWORD,
        blue_size: 16 as DWORD,
        alpha_size: 16 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 16 as DWORD,
        blue_offset: 32 as DWORD,
        alpha_offset: 48 as DWORD,
        bpp: 8 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R32G32_TYPELESS,
        red_size: 32 as DWORD,
        green_size: 32 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 32 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 8 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R32G8X24_TYPELESS,
        red_size: 32 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 8 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R10G10B10A2_TYPELESS,
        red_size: 10 as DWORD,
        green_size: 10 as DWORD,
        blue_size: 10 as DWORD,
        alpha_size: 2 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 10 as DWORD,
        blue_offset: 20 as DWORD,
        alpha_offset: 30 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R10G10B10X2_TYPELESS,
        red_size: 10 as DWORD,
        green_size: 10 as DWORD,
        blue_size: 10 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 10 as DWORD,
        blue_offset: 20 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8B8A8_TYPELESS,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 16 as DWORD,
        alpha_offset: 24 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R16G16_TYPELESS,
        red_size: 16 as DWORD,
        green_size: 16 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 16 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R32_TYPELESS,
        red_size: 32 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R24G8_TYPELESS,
        red_size: 24 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8G8_TYPELESS,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R16_TYPELESS,
        red_size: 16 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 2 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_R8_TYPELESS,
        red_size: 8 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC1_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC2_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC3_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC4_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC5_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC6H_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_BC7_TYPELESS,
        red_size: 0 as DWORD,
        green_size: 0 as DWORD,
        blue_size: 0 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 0 as DWORD,
        green_offset: 0 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 1 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_B8G8R8A8_TYPELESS,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 8 as DWORD,
        red_offset: 16 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 24 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
    wined3d_format_channels {
        id: WINED3DFMT_B8G8R8X8_TYPELESS,
        red_size: 8 as DWORD,
        green_size: 8 as DWORD,
        blue_size: 8 as DWORD,
        alpha_size: 0 as DWORD,
        red_offset: 16 as DWORD,
        green_offset: 8 as DWORD,
        blue_offset: 0 as DWORD,
        alpha_offset: 0 as DWORD,
        bpp: 4 as UINT,
        depth_size: 0 as BYTE,
        stencil_size: 0 as BYTE,
        channels: ::core::ptr::null::<::core::ffi::c_char>(),
    },
];

static mut typed_formats: [wined3d_typed_format_info; 71] = [
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32A32_UINT,
        typeless_id: WINED3DFMT_R32G32B32A32_TYPELESS,
        channels: b"UUUU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32A32_SINT,
        typeless_id: WINED3DFMT_R32G32B32A32_TYPELESS,
        channels: b"IIII\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32A32_FLOAT,
        typeless_id: WINED3DFMT_R32G32B32A32_TYPELESS,
        channels: b"FFFF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32_UINT,
        typeless_id: WINED3DFMT_R32G32B32_TYPELESS,
        channels: b"UUU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32_SINT,
        typeless_id: WINED3DFMT_R32G32B32_TYPELESS,
        channels: b"III\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32B32_FLOAT,
        typeless_id: WINED3DFMT_R32G32B32_TYPELESS,
        channels: b"FFF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16B16A16_UNORM,
        typeless_id: WINED3DFMT_R16G16B16A16_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16B16A16_SNORM,
        typeless_id: WINED3DFMT_R16G16B16A16_TYPELESS,
        channels: b"iiii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16B16A16_UINT,
        typeless_id: WINED3DFMT_R16G16B16A16_TYPELESS,
        channels: b"UUUU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16B16A16_SINT,
        typeless_id: WINED3DFMT_R16G16B16A16_TYPELESS,
        channels: b"IIII\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16B16A16_FLOAT,
        typeless_id: WINED3DFMT_R16G16B16A16_TYPELESS,
        channels: b"FFFF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32_UINT,
        typeless_id: WINED3DFMT_R32G32_TYPELESS,
        channels: b"UU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32_SINT,
        typeless_id: WINED3DFMT_R32G32_TYPELESS,
        channels: b"II\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32G32_FLOAT,
        typeless_id: WINED3DFMT_R32G32_TYPELESS,
        channels: b"FF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32_FLOAT_X8X24_TYPELESS,
        typeless_id: WINED3DFMT_R32G8X24_TYPELESS,
        channels: b"FXX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_X32_TYPELESS_G8X24_UINT,
        typeless_id: WINED3DFMT_R32G8X24_TYPELESS,
        channels: b"XUX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_D32_FLOAT_S8X24_UINT,
        typeless_id: WINED3DFMT_R32G8X24_TYPELESS,
        channels: b"DSX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R10G10B10A2_SNORM,
        typeless_id: WINED3DFMT_R10G10B10A2_TYPELESS,
        channels: b"iiii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R10G10B10A2_UINT,
        typeless_id: WINED3DFMT_R10G10B10A2_TYPELESS,
        channels: b"UUUU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R10G10B10A2_UNORM,
        typeless_id: WINED3DFMT_R10G10B10A2_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R10G10B10X2_SNORM,
        typeless_id: WINED3DFMT_R10G10B10X2_TYPELESS,
        channels: b"iiiX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R10G10B10X2_UINT,
        typeless_id: WINED3DFMT_R10G10B10X2_TYPELESS,
        channels: b"UUUX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8B8A8_UINT,
        typeless_id: WINED3DFMT_R8G8B8A8_TYPELESS,
        channels: b"UUUU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8B8A8_SINT,
        typeless_id: WINED3DFMT_R8G8B8A8_TYPELESS,
        channels: b"IIII\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8B8A8_SNORM,
        typeless_id: WINED3DFMT_R8G8B8A8_TYPELESS,
        channels: b"iiii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8B8A8_UNORM_SRGB,
        typeless_id: WINED3DFMT_R8G8B8A8_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8B8A8_UNORM,
        typeless_id: WINED3DFMT_R8G8B8A8_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16_UNORM,
        typeless_id: WINED3DFMT_R16G16_TYPELESS,
        channels: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16_SNORM,
        typeless_id: WINED3DFMT_R16G16_TYPELESS,
        channels: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16_UINT,
        typeless_id: WINED3DFMT_R16G16_TYPELESS,
        channels: b"UU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16_SINT,
        typeless_id: WINED3DFMT_R16G16_TYPELESS,
        channels: b"II\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16G16_FLOAT,
        typeless_id: WINED3DFMT_R16G16_TYPELESS,
        channels: b"FF\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_D32_FLOAT,
        typeless_id: WINED3DFMT_R32_TYPELESS,
        channels: b"D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32_FLOAT,
        typeless_id: WINED3DFMT_R32_TYPELESS,
        channels: b"F\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32_UINT,
        typeless_id: WINED3DFMT_R32_TYPELESS,
        channels: b"U\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R32_SINT,
        typeless_id: WINED3DFMT_R32_TYPELESS,
        channels: b"I\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R24_UNORM_X8_TYPELESS,
        typeless_id: WINED3DFMT_R24G8_TYPELESS,
        channels: b"uX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_X24_TYPELESS_G8_UINT,
        typeless_id: WINED3DFMT_R24G8_TYPELESS,
        channels: b"XU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_D24_UNORM_S8_UINT,
        typeless_id: WINED3DFMT_R24G8_TYPELESS,
        channels: b"DS\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8_SNORM,
        typeless_id: WINED3DFMT_R8G8_TYPELESS,
        channels: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8_UNORM,
        typeless_id: WINED3DFMT_R8G8_TYPELESS,
        channels: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8_UINT,
        typeless_id: WINED3DFMT_R8G8_TYPELESS,
        channels: b"UU\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8G8_SINT,
        typeless_id: WINED3DFMT_R8G8_TYPELESS,
        channels: b"II\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_D16_UNORM,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"D\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16_UNORM,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"u\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16_SNORM,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"i\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16_UINT,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"U\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16_SINT,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"I\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R16_FLOAT,
        typeless_id: WINED3DFMT_R16_TYPELESS,
        channels: b"F\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8_UNORM,
        typeless_id: WINED3DFMT_R8_TYPELESS,
        channels: b"u\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8_SNORM,
        typeless_id: WINED3DFMT_R8_TYPELESS,
        channels: b"i\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8_UINT,
        typeless_id: WINED3DFMT_R8_TYPELESS,
        channels: b"U\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_R8_SINT,
        typeless_id: WINED3DFMT_R8_TYPELESS,
        channels: b"I\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC1_UNORM_SRGB,
        typeless_id: WINED3DFMT_BC1_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC1_UNORM,
        typeless_id: WINED3DFMT_BC1_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC2_UNORM_SRGB,
        typeless_id: WINED3DFMT_BC2_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC2_UNORM,
        typeless_id: WINED3DFMT_BC2_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC3_UNORM_SRGB,
        typeless_id: WINED3DFMT_BC3_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC3_UNORM,
        typeless_id: WINED3DFMT_BC3_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC4_UNORM,
        typeless_id: WINED3DFMT_BC4_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC4_SNORM,
        typeless_id: WINED3DFMT_BC4_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC5_UNORM,
        typeless_id: WINED3DFMT_BC5_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC5_SNORM,
        typeless_id: WINED3DFMT_BC5_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC6H_UF16,
        typeless_id: WINED3DFMT_BC6H_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC6H_SF16,
        typeless_id: WINED3DFMT_BC6H_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC7_UNORM_SRGB,
        typeless_id: WINED3DFMT_BC7_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_BC7_UNORM,
        typeless_id: WINED3DFMT_BC7_TYPELESS,
        channels: b"\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_B8G8R8A8_UNORM_SRGB,
        typeless_id: WINED3DFMT_B8G8R8A8_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_B8G8R8A8_UNORM,
        typeless_id: WINED3DFMT_B8G8R8A8_TYPELESS,
        channels: b"uuuu\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_B8G8R8X8_UNORM_SRGB,
        typeless_id: WINED3DFMT_B8G8R8X8_TYPELESS,
        channels: b"uuuX\0" as *const u8 as *const ::core::ffi::c_char,
    },
    wined3d_typed_format_info {
        id: WINED3DFMT_B8G8R8X8_UNORM,
        typeless_id: WINED3DFMT_B8G8R8X8_TYPELESS,
        channels: b"uuuX\0" as *const u8 as *const ::core::ffi::c_char,
    },
];

static mut typeless_depth_stencil_formats: [wined3d_typeless_format_depth_stencil_info; 4] = [
    wined3d_typeless_format_depth_stencil_info {
        typeless_id: WINED3DFMT_R32G8X24_TYPELESS,
        depth_stencil_id: WINED3DFMT_D32_FLOAT_S8X24_UINT,
        depth_view_id: WINED3DFMT_R32_FLOAT_X8X24_TYPELESS,
        stencil_view_id: WINED3DFMT_X32_TYPELESS_G8X24_UINT,
        separate_depth_view_format: (TRUE as i32),
    },
    wined3d_typeless_format_depth_stencil_info {
        typeless_id: WINED3DFMT_R24G8_TYPELESS,
        depth_stencil_id: WINED3DFMT_D24_UNORM_S8_UINT,
        depth_view_id: WINED3DFMT_R24_UNORM_X8_TYPELESS,
        stencil_view_id: WINED3DFMT_X24_TYPELESS_G8_UINT,
        separate_depth_view_format: (TRUE as i32),
    },
    wined3d_typeless_format_depth_stencil_info {
        typeless_id: WINED3DFMT_R32_TYPELESS,
        depth_stencil_id: WINED3DFMT_D32_FLOAT,
        depth_view_id: WINED3DFMT_R32_FLOAT,
        stencil_view_id: WINED3DFMT_UNKNOWN,
        separate_depth_view_format: 0,
    },
    wined3d_typeless_format_depth_stencil_info {
        typeless_id: WINED3DFMT_R16_TYPELESS,
        depth_stencil_id: WINED3DFMT_D16_UNORM,
        depth_view_id: WINED3DFMT_R16_UNORM,
        stencil_view_id: WINED3DFMT_UNKNOWN,
        separate_depth_view_format: 0,
    },
];

static mut ddi_formats: [wined3d_format_ddi_info; 9] = [
    wined3d_format_ddi_info {
        id: WINED3DFMT_B8G8R8_UNORM,
        ddi_format: D3DDDIFMT_R8G8B8,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B8G8R8A8_UNORM,
        ddi_format: D3DDDIFMT_A8R8G8B8,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B8G8R8X8_UNORM,
        ddi_format: D3DDDIFMT_X8R8G8B8,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B5G6R5_UNORM,
        ddi_format: D3DDDIFMT_R5G6B5,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B5G5R5X1_UNORM,
        ddi_format: D3DDDIFMT_X1R5G5B5,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B5G5R5A1_UNORM,
        ddi_format: D3DDDIFMT_A1R5G5B5,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B4G4R4A4_UNORM,
        ddi_format: D3DDDIFMT_A4R4G4B4,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_B4G4R4X4_UNORM,
        ddi_format: D3DDDIFMT_X4R4G4B4,
    },
    wined3d_format_ddi_info {
        id: WINED3DFMT_P8_UINT,
        ddi_format: D3DDDIFMT_P8,
    },
];

static mut format_base_flags: [wined3d_format_base_flags; 36] = [
    wined3d_format_base_flags {
        id: WINED3DFMT_ATI1N,
        attrs: (WINED3D_FORMAT_ATTR_MAPPABLE | WINED3D_FORMAT_ATTR_BROKEN_PITCH)
            as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_ATI2N,
        attrs: (WINED3D_FORMAT_ATTR_MAPPABLE | WINED3D_FORMAT_ATTR_BROKEN_PITCH)
            as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_D16_LOCKABLE,
        attrs: WINED3D_FORMAT_ATTR_MAPPABLE as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_INTZ,
        attrs: WINED3D_FORMAT_ATTR_MAPPABLE as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_D32_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_FLOAT as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_S8_UINT_D24_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_FLOAT as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_D32_FLOAT_S8X24_UINT,
        attrs: WINED3D_FORMAT_ATTR_FLOAT as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_INST,
        attrs: WINED3D_FORMAT_ATTR_EXTENSION as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_NULL,
        attrs: WINED3D_FORMAT_ATTR_EXTENSION as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_NVDB,
        attrs: WINED3D_FORMAT_ATTR_EXTENSION as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_ATOC,
        attrs: WINED3D_FORMAT_ATTR_EXTENSION as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_RESZ,
        attrs: WINED3D_FORMAT_ATTR_EXTENSION as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32B32A32_TYPELESS,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32B32A32_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32B32A32_UINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32B32A32_SINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_TYPELESS,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_UNORM,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_UINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_SNORM,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16B16A16_SINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32_TYPELESS,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32_UINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32G32_SINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32_TYPELESS,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32_FLOAT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32_UINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: WINED3D_FORMAT_CAP_INDEX_BUFFER as ::core::ffi::c_uint,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R32_SINT,
        attrs: WINED3D_FORMAT_ATTR_CAST_TO_BLOCK as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16_UINT,
        attrs: 0 as ::core::ffi::c_uint,
        caps: WINED3D_FORMAT_CAP_INDEX_BUFFER as ::core::ffi::c_uint,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R8G8_SNORM,
        attrs: WINED3D_FORMAT_ATTR_BUMPMAP as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
        attrs: WINED3D_FORMAT_ATTR_BUMPMAP as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
        attrs: WINED3D_FORMAT_ATTR_BUMPMAP as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R8G8B8A8_SNORM,
        attrs: WINED3D_FORMAT_ATTR_BUMPMAP as ::core::ffi::c_uint,
        caps: 0,
    },
    wined3d_format_base_flags {
        id: WINED3DFMT_R16G16_SNORM,
        attrs: WINED3D_FORMAT_ATTR_BUMPMAP as ::core::ffi::c_uint,
        caps: 0,
    },
];

static mut format_decompress_info: [wined3d_format_decompress_info; 9] = unsafe {
    [
        wined3d_format_decompress_info {
            id: WINED3DFMT_DXT1,
            decompress: Some(
                decompress_bc1
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_DXT2,
            decompress: Some(
                decompress_bc2
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_DXT3,
            decompress: Some(
                decompress_bc2
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_DXT4,
            decompress: Some(
                decompress_bc3
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_DXT5,
            decompress: Some(
                decompress_bc3
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_BC1_UNORM,
            decompress: Some(
                decompress_bc1
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_BC2_UNORM,
            decompress: Some(
                decompress_bc2
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_BC3_UNORM,
            decompress: Some(
                decompress_bc3
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        wined3d_format_decompress_info {
            id: WINED3DFMT_BC4_UNORM,
            decompress: Some(
                decompress_bc4
                    as unsafe extern "C" fn(
                        *const uint8_t,
                        *mut uint8_t,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
    ]
};

static mut format_block_info: [wined3d_format_block_info; 20] = [
    wined3d_format_block_info {
        id: WINED3DFMT_DXT1,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 8 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_DXT2,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_DXT3,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_DXT4,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_DXT5,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC1_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 8 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC2_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC3_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC4_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 8 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC4_SNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 8 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC5_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC5_SNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC6H_UF16,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC6H_SF16,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_BC7_UNORM,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_ATI1N,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 8 as UINT,
        attrs: (WINED3D_FORMAT_ATTR_COMPRESSED | WINED3D_FORMAT_ATTR_BLOCKS_NO_VERIFY)
            as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_ATI2N,
        block_width: 4 as UINT,
        block_height: 4 as UINT,
        block_byte_count: 16 as UINT,
        attrs: (WINED3D_FORMAT_ATTR_COMPRESSED | WINED3D_FORMAT_ATTR_BLOCKS_NO_VERIFY)
            as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_YUY2,
        block_width: 2 as UINT,
        block_height: 1 as UINT,
        block_byte_count: 4 as UINT,
        attrs: WINED3D_FORMAT_ATTR_BLOCKS_NO_VERIFY as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_UYVY,
        block_width: 2 as UINT,
        block_height: 1 as UINT,
        block_byte_count: 4 as UINT,
        attrs: WINED3D_FORMAT_ATTR_BLOCKS_NO_VERIFY as ::core::ffi::c_uint,
    },
    wined3d_format_block_info {
        id: WINED3DFMT_R9G9B9E5_SHAREDEXP,
        block_width: 1 as UINT,
        block_height: 1 as UINT,
        block_byte_count: 4 as UINT,
        attrs: 0,
    },
];

static mut format_vertex_info: [wined3d_format_vertex_info; 40] = [
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT1,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT2,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32B32_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT3,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32B32A32_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT4,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_B8G8R8A8_UNORM,
        emit_idx: WINED3D_FFP_EMIT_D3DCOLOR,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8B8A8_UINT,
        emit_idx: WINED3D_FFP_EMIT_UBYTE4,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_SINT,
        emit_idx: WINED3D_FFP_EMIT_SHORT2,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_SINT,
        emit_idx: WINED3D_FFP_EMIT_SHORT4,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8B8A8_UNORM,
        emit_idx: WINED3D_FFP_EMIT_UBYTE4N,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_SNORM,
        emit_idx: WINED3D_FFP_EMIT_SHORT2N,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_SNORM,
        emit_idx: WINED3D_FFP_EMIT_SHORT4N,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_UNORM,
        emit_idx: WINED3D_FFP_EMIT_USHORT2N,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_UNORM,
        emit_idx: WINED3D_FFP_EMIT_USHORT4N,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R11G11B10_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT_10F_11F_11F_REV as GLenum,
        extension: ARB_VERTEX_TYPE_10F_11F_11F_REV,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R10G10B10X2_UINT,
        emit_idx: WINED3D_FFP_EMIT_UDEC3,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R10G10B10X2_SNORM,
        emit_idx: WINED3D_FFP_EMIT_DEC3N,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R10G10B10A2_UNORM,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT_2_10_10_10_REV as GLenum,
        extension: ARB_VERTEX_TYPE_2_10_10_10_REV,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT16_2,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT16_2,
        gl_vtx_type: GL_HALF_FLOAT as GLenum,
        extension: ARB_HALF_FLOAT_VERTEX,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT16_4,
        gl_vtx_type: GL_FLOAT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_FLOAT16_4,
        gl_vtx_type: GL_HALF_FLOAT as GLenum,
        extension: ARB_HALF_FLOAT_VERTEX,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8B8A8_SNORM,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8B8A8_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8_UNORM,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16G16B16A16_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8_UNORM,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R8G8_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_BYTE as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16_FLOAT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_HALF_FLOAT as GLenum,
        extension: ARB_HALF_FLOAT_VERTEX,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R16_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_SHORT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32B32_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32B32A32_UINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_UNSIGNED_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
    wined3d_format_vertex_info {
        id: WINED3DFMT_R32G32B32A32_SINT,
        emit_idx: WINED3D_FFP_EMIT_INVALID,
        gl_vtx_type: GL_INT as GLenum,
        extension: WINED3D_GL_EXT_NONE,
    },
];

static mut format_texture_info: [wined3d_format_texture_info; 125] = unsafe {
    [
        wined3d_format_texture_info {
            id: WINED3DFMT_UYVY,
            gl_internal: (GL_RG8 as i32),
            gl_srgb_internal: (GL_RG8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_UYVY,
            gl_internal: (GL_LUMINANCE8_ALPHA8 as i32),
            gl_srgb_internal: (GL_LUMINANCE8_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_UYVY,
            gl_internal: (GL_RGB_RAW_422_APPLE as i32),
            gl_srgb_internal: (GL_RGB_RAW_422_APPLE as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB_422_APPLE as i32),
            gl_type: (GL_UNSIGNED_SHORT_8_8_APPLE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: APPLE_RGB_422,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_UYVY,
            gl_internal: (GL_RGB as i32),
            gl_srgb_internal: (GL_RGB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_YCBCR_422_APPLE as i32),
            gl_type: (GL_UNSIGNED_SHORT_8_8_APPLE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_FILTERING)
                as ::core::ffi::c_uint,
            extension: APPLE_YCBCR_422,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YUY2,
            gl_internal: (GL_RG8 as i32),
            gl_srgb_internal: (GL_RG8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YUY2,
            gl_internal: (GL_LUMINANCE8_ALPHA8 as i32),
            gl_srgb_internal: (GL_LUMINANCE8_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YUY2,
            gl_internal: (GL_RGB_RAW_422_APPLE as i32),
            gl_srgb_internal: (GL_RGB_RAW_422_APPLE as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB_422_APPLE as i32),
            gl_type: (GL_UNSIGNED_SHORT_8_8_REV_APPLE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: APPLE_RGB_422,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YUY2,
            gl_internal: (GL_RGB as i32),
            gl_srgb_internal: (GL_RGB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_YCBCR_422_APPLE as i32),
            gl_type: (GL_UNSIGNED_SHORT_8_8_REV_APPLE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_FILTERING)
                as ::core::ffi::c_uint,
            extension: APPLE_YCBCR_422,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YV12,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_YV12,
            gl_internal: (GL_ALPHA8 as i32),
            gl_srgb_internal: (GL_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_NV12,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_NV12,
            gl_internal: (GL_ALPHA8 as i32),
            gl_srgb_internal: (GL_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_DXT1,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT1_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_DXT2,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT3_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_DXT3,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT3_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_DXT4,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT5_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_DXT5,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT5_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC1_UNORM,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT1_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC2_UNORM,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT3_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC3_UNORM,
            gl_internal: (GL_COMPRESSED_RGBA_S3TC_DXT5_EXT as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_S3TC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC4_UNORM,
            gl_internal: (GL_COMPRESSED_RED_RGTC1 as i32),
            gl_srgb_internal: (GL_COMPRESSED_RED_RGTC1 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC4_SNORM,
            gl_internal: (GL_COMPRESSED_SIGNED_RED_RGTC1 as i32),
            gl_srgb_internal: (GL_COMPRESSED_SIGNED_RED_RGTC1 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC5_UNORM,
            gl_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_srgb_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC5_SNORM,
            gl_internal: (GL_COMPRESSED_SIGNED_RG_RGTC2 as i32),
            gl_srgb_internal: (GL_COMPRESSED_SIGNED_RG_RGTC2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC6H_UF16,
            gl_internal: (GL_COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_ARB as i32),
            gl_srgb_internal: (GL_COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_BPTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC6H_SF16,
            gl_internal: (GL_COMPRESSED_RGB_BPTC_SIGNED_FLOAT_ARB as i32),
            gl_srgb_internal: (GL_COMPRESSED_RGB_BPTC_SIGNED_FLOAT_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_BPTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_BC7_UNORM,
            gl_internal: (GL_COMPRESSED_RGBA_BPTC_UNORM_ARB as i32),
            gl_srgb_internal: (GL_COMPRESSED_SRGB_ALPHA_BPTC_UNORM_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_BPTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32_FLOAT,
            gl_internal: (GL_RGB32F_ARB as i32),
            gl_srgb_internal: (GL_RGB32F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32_FLOAT,
            gl_internal: (GL_R32F as i32),
            gl_srgb_internal: (GL_R32F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32_FLOAT,
            gl_internal: (GL_RGB32F_ARB as i32),
            gl_srgb_internal: (GL_RGB32F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 12 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: Some(
                convert_r32g32_float
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32_FLOAT,
            gl_internal: (GL_RG32F as i32),
            gl_srgb_internal: (GL_RG32F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32B32_FLOAT,
            gl_internal: (GL_RGB32F as i32),
            gl_srgb_internal: (GL_RGB32F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32B32A32_FLOAT,
            gl_internal: (GL_RGBA32F_ARB as i32),
            gl_srgb_internal: (GL_RGBA32F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_FLOAT,
            gl_internal: (GL_RGB16F_ARB as i32),
            gl_srgb_internal: (GL_RGB16F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_HALF_FLOAT_ARB as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_FLOAT,
            gl_internal: (GL_R16F as i32),
            gl_srgb_internal: (GL_R16F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_HALF_FLOAT_ARB as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_FLOAT,
            gl_internal: (GL_RGB16F_ARB as i32),
            gl_srgb_internal: (GL_RGB16F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_HALF_FLOAT_ARB as i32),
            conv_byte_count: 6 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: Some(
                convert_r16g16
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_FLOAT,
            gl_internal: (GL_RG16F as i32),
            gl_srgb_internal: (GL_RG16F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_HALF_FLOAT_ARB as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16B16A16_FLOAT,
            gl_internal: (GL_RGBA16F_ARB as i32),
            gl_srgb_internal: (GL_RGBA16F_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_HALF_FLOAT_ARB as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R11G11B10_FLOAT,
            gl_internal: (GL_R11F_G11F_B10F as i32),
            gl_srgb_internal: (GL_R11F_G11F_B10F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_INT_10F_11F_11F_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_PACKED_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_P8_UINT,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: 0 as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_P8_UINT,
            gl_internal: (GL_ALPHA8 as i32),
            gl_srgb_internal: (GL_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: 0 as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B8G8R8_UNORM,
            gl_internal: (GL_RGB8 as i32),
            gl_srgb_internal: (GL_RGB8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGR as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B8G8R8A8_UNORM,
            gl_internal: (GL_RGBA8 as i32),
            gl_srgb_internal: (GL_SRGB8_ALPHA8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_SRGB_READ
                | WINED3D_FORMAT_CAP_SRGB_WRITE
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B8G8R8X8_UNORM,
            gl_internal: (GL_RGB8 as i32),
            gl_srgb_internal: (GL_SRGB8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_SRGB_READ
                | WINED3D_FORMAT_CAP_SRGB_WRITE) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B5G6R5_UNORM,
            gl_internal: (GL_RGB5 as i32),
            gl_srgb_internal: (GL_SRGB8_EXT as i32),
            gl_rt_internal: (GL_RGB8 as i32),
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_SHORT_5_6_5 as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B5G6R5_UNORM,
            gl_internal: (GL_RGB565 as i32),
            gl_srgb_internal: (GL_SRGB8_EXT as i32),
            gl_rt_internal: (GL_RGB8 as i32),
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_SHORT_5_6_5 as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: ARB_ES2_COMPATIBILITY,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B5G5R5X1_UNORM,
            gl_internal: (GL_RGB5 as i32),
            gl_srgb_internal: (GL_RGB5 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_SHORT_1_5_5_5_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B5G5R5A1_UNORM,
            gl_internal: (GL_RGB5_A1 as i32),
            gl_srgb_internal: (GL_RGB5_A1 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_SHORT_1_5_5_5_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B4G4R4A4_UNORM,
            gl_internal: (GL_RGBA4 as i32),
            gl_srgb_internal: (GL_SRGB8_ALPHA8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_SHORT_4_4_4_4_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B2G3R3_UNORM,
            gl_internal: (GL_R3_G3_B2 as i32),
            gl_srgb_internal: (GL_R3_G3_B2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_BYTE_3_3_2 as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING | WINED3D_FORMAT_CAP_FILTERING)
                as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8_UNORM,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_A8_UNORM,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_A8_UNORM,
            gl_internal: (GL_ALPHA8 as i32),
            gl_srgb_internal: (GL_ALPHA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B4G4R4X4_UNORM,
            gl_internal: (GL_RGB4 as i32),
            gl_srgb_internal: (GL_RGB4 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_SHORT_4_4_4_4_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R10G10B10A2_UINT,
            gl_internal: (GL_RGB10_A2UI as i32),
            gl_srgb_internal: (GL_RGB10_A2UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_UNSIGNED_INT_2_10_10_10_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RGB10_A2UI,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R10G10B10A2_UNORM,
            gl_internal: (GL_RGB10_A2 as i32),
            gl_srgb_internal: (GL_RGB10_A2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_INT_2_10_10_10_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_UNORM,
            gl_internal: (GL_RGBA8 as i32),
            gl_srgb_internal: (GL_SRGB8_ALPHA8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_SRGB_READ
                | WINED3D_FORMAT_CAP_SRGB_WRITE
                | WINED3D_FORMAT_CAP_VTF) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_UINT,
            gl_internal: (GL_RGBA8UI as i32),
            gl_srgb_internal: (GL_RGBA8UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RGB10_A2UI,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_SINT,
            gl_internal: (GL_RGBA8I as i32),
            gl_srgb_internal: (GL_RGBA8I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_INTEGER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8X8_UNORM,
            gl_internal: (GL_RGB8 as i32),
            gl_srgb_internal: (GL_RGB8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_UNORM,
            gl_internal: (GL_RGB16 as i32),
            gl_srgb_internal: (GL_RGB16 as i32),
            gl_rt_internal: (GL_RGBA16 as i32),
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 6 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r16g16
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_UNORM,
            gl_internal: (GL_RG16 as i32),
            gl_srgb_internal: (GL_RG16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_B10G10R10A2_UNORM,
            gl_internal: (GL_RGB10_A2 as i32),
            gl_srgb_internal: (GL_RGB10_A2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_INT_2_10_10_10_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16B16A16_UNORM,
            gl_internal: (GL_RGBA16 as i32),
            gl_srgb_internal: (GL_RGBA16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_UNORM,
            gl_internal: (GL_RG8 as i32),
            gl_srgb_internal: (GL_RG8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_UINT,
            gl_internal: (GL_RG8UI as i32),
            gl_srgb_internal: (GL_RG8UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SINT,
            gl_internal: (GL_RG8I as i32),
            gl_srgb_internal: (GL_RG8I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16B16A16_UINT,
            gl_internal: (GL_RGBA16UI as i32),
            gl_srgb_internal: (GL_RGBA16UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_INTEGER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16B16A16_SINT,
            gl_internal: (GL_RGBA16I as i32),
            gl_srgb_internal: (GL_RGBA16I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_INTEGER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32_UINT,
            gl_internal: (GL_RG32UI as i32),
            gl_srgb_internal: (GL_RG32UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32_SINT,
            gl_internal: (GL_RG32I as i32),
            gl_srgb_internal: (GL_RG32I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_UINT,
            gl_internal: (GL_RG16UI as i32),
            gl_srgb_internal: (GL_RG16UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_SINT,
            gl_internal: (GL_RG16I as i32),
            gl_srgb_internal: (GL_RG16I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG_INTEGER as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32_UINT,
            gl_internal: (GL_R32UI as i32),
            gl_srgb_internal: (GL_R32UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32_SINT,
            gl_internal: (GL_R32I as i32),
            gl_srgb_internal: (GL_R32I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_UNORM,
            gl_internal: (GL_R16 as i32),
            gl_srgb_internal: (GL_R16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_UINT,
            gl_internal: (GL_R16UI as i32),
            gl_srgb_internal: (GL_R16UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_SINT,
            gl_internal: (GL_R16I as i32),
            gl_srgb_internal: (GL_R16I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8_UINT,
            gl_internal: (GL_R8UI as i32),
            gl_srgb_internal: (GL_R8UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8_SINT,
            gl_internal: (GL_R8I as i32),
            gl_srgb_internal: (GL_R8I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED_INTEGER as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L8_UNORM,
            gl_internal: (GL_LUMINANCE8 as i32),
            gl_srgb_internal: (GL_SLUMINANCE8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L8_UNORM,
            gl_internal: (GL_R8 as i32),
            gl_srgb_internal: (GL_R8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L8A8_UNORM,
            gl_internal: (GL_RG8 as i32),
            gl_srgb_internal: (GL_RG8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L8A8_UNORM,
            gl_internal: (GL_LUMINANCE8_ALPHA8 as i32),
            gl_srgb_internal: (GL_SLUMINANCE8_ALPHA8_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_SRGB_READ) as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L4A4_UNORM,
            gl_internal: (GL_RG8 as i32),
            gl_srgb_internal: (GL_RG8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 2 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: Some(
                convert_l4a4_unorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L4A4_UNORM,
            gl_internal: (GL_LUMINANCE4_ALPHA4 as i32),
            gl_srgb_internal: (GL_LUMINANCE4_ALPHA4 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 2 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_FILTERING as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: Some(
                convert_l4a4_unorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L16_UNORM,
            gl_internal: (GL_R16 as i32),
            gl_srgb_internal: (GL_R16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_RG,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_L16_UNORM,
            gl_internal: (GL_LUMINANCE16 as i32),
            gl_srgb_internal: (GL_LUMINANCE16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_LEGACY_CONTEXT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SNORM,
            gl_internal: (GL_RGB8 as i32),
            gl_srgb_internal: (GL_RGB8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGR as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 3 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r8g8_snorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SNORM,
            gl_internal: (GL_DSDT8_NV as i32),
            gl_srgb_internal: (GL_DSDT8_NV as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DSDT_NV as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: NV_TEXTURE_SHADER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SNORM,
            gl_internal: (GL_RG8_SNORM as i32),
            gl_srgb_internal: (GL_RG8_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            gl_internal: (GL_RGB5 as i32),
            gl_srgb_internal: (GL_RGB5 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_SHORT_5_6_5 as i32),
            conv_byte_count: 2 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r5g5_snorm_l6_unorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            gl_internal: (GL_DSDT8_MAG8_NV as i32),
            gl_srgb_internal: (GL_DSDT8_MAG8_NV as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DSDT_MAG_NV as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 3 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: NV_TEXTURE_SHADER,
            upload: Some(
                convert_r5g5_snorm_l6_unorm_nv
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            gl_internal: (GL_RGB8_SNORM as i32),
            gl_srgb_internal: (GL_RGB8_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: Some(
                convert_r5g5_snorm_l6_unorm_ext
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
            gl_internal: (GL_RGB8 as i32),
            gl_srgb_internal: (GL_RGB8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r8g8_snorm_l8x8_unorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
            gl_internal: (GL_DSDT8_MAG8_INTENSITY8_NV as i32),
            gl_srgb_internal: (GL_DSDT8_MAG8_INTENSITY8_NV as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DSDT_MAG_VIB_NV as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_S8_S8_REV_NV as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: NV_TEXTURE_SHADER,
            upload: Some(
                convert_r8g8_snorm_l8x8_unorm_nv
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            gl_internal: (GL_RGBA8 as i32),
            gl_srgb_internal: (GL_RGBA8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGRA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r8g8b8a8_snorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            gl_internal: (GL_SIGNED_RGBA8_NV as i32),
            gl_srgb_internal: (GL_SIGNED_RGBA8_NV as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: NV_TEXTURE_SHADER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            gl_internal: (GL_RGBA8_SNORM as i32),
            gl_srgb_internal: (GL_RGBA8_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_SNORM,
            gl_internal: (GL_RGB16 as i32),
            gl_srgb_internal: (GL_RGB16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_BGR as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 6 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                convert_r16g16_snorm
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_SNORM,
            gl_internal: (GL_SIGNED_HILO16_NV as i32),
            gl_srgb_internal: (GL_SIGNED_HILO16_NV as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_HILO_NV as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: NV_TEXTURE_SHADER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16_SNORM,
            gl_internal: (GL_RG16_SNORM as i32),
            gl_srgb_internal: (GL_RG16_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16G16B16A16_SNORM,
            gl_internal: (GL_RGBA16_SNORM as i32),
            gl_srgb_internal: (GL_RGBA16_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R16_SNORM,
            gl_internal: (GL_R16_SNORM as i32),
            gl_srgb_internal: (GL_R16_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R8_SNORM,
            gl_internal: (GL_R8_SNORM as i32),
            gl_srgb_internal: (GL_R8_SNORM as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SNORM,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D16_LOCKABLE,
            gl_internal: (GL_DEPTH_COMPONENT as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D16_LOCKABLE,
            gl_internal: (GL_DEPTH_COMPONENT16 as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_TEXTURE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D24_UNORM_S8_UINT,
            gl_internal: (GL_DEPTH_COMPONENT24_ARB as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT24_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_TEXTURE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D24_UNORM_S8_UINT,
            gl_internal: (GL_DEPTH24_STENCIL8 as i32),
            gl_srgb_internal: (GL_DEPTH24_STENCIL8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_STENCIL as i32),
            gl_type: (GL_UNSIGNED_INT_24_8 as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: EXT_PACKED_DEPTH_STENCIL,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_X8D24_UNORM,
            gl_internal: (GL_DEPTH_COMPONENT as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: Some(
                x8_d24_unorm_upload
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
            download: Some(
                x8_d24_unorm_download
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_X8D24_UNORM,
            gl_internal: (GL_DEPTH_COMPONENT24_ARB as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT24_ARB as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 4 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_TEXTURE,
            upload: Some(
                x8_d24_unorm_upload
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
            download: Some(
                x8_d24_unorm_download
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D16_UNORM,
            gl_internal: (GL_DEPTH_COMPONENT as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint,
            extension: WINED3D_GL_EXT_NONE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D16_UNORM,
            gl_internal: (GL_DEPTH_COMPONENT16 as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT16 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_UNSIGNED_SHORT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_TEXTURE,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D32_FLOAT,
            gl_internal: (GL_DEPTH_COMPONENT32F as i32),
            gl_srgb_internal: (GL_DEPTH_COMPONENT32F as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_COMPONENT as i32),
            gl_type: (GL_FLOAT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_BUFFER_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_D32_FLOAT_S8X24_UINT,
            gl_internal: (GL_DEPTH32F_STENCIL8 as i32),
            gl_srgb_internal: (GL_DEPTH32F_STENCIL8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_STENCIL as i32),
            gl_type: (GL_FLOAT_32_UNSIGNED_INT_24_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_BUFFER_FLOAT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_S8_UINT_D24_FLOAT,
            gl_internal: (GL_DEPTH32F_STENCIL8 as i32),
            gl_srgb_internal: (GL_DEPTH32F_STENCIL8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_STENCIL as i32),
            gl_type: (GL_FLOAT_32_UNSIGNED_INT_24_8_REV as i32),
            conv_byte_count: 8 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_SHADOW) as ::core::ffi::c_uint,
            extension: ARB_DEPTH_BUFFER_FLOAT,
            upload: Some(
                convert_s8_uint_d24_float
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32B32A32_UINT,
            gl_internal: (GL_RGBA32UI as i32),
            gl_srgb_internal: (GL_RGBA32UI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_UNSIGNED_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_INTEGER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R32G32B32A32_SINT,
            gl_internal: (GL_RGBA32I as i32),
            gl_srgb_internal: (GL_RGBA32I as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA_INTEGER as i32),
            gl_type: (GL_INT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET)
                as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_INTEGER,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_ATI1N,
            gl_internal: (GL_COMPRESSED_RED_RGTC1 as i32),
            gl_srgb_internal: (GL_COMPRESSED_RED_RGTC1 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RED as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_ATI2N,
            gl_internal: (GL_COMPRESSED_LUMINANCE_ALPHA_3DC_ATI as i32),
            gl_srgb_internal: (GL_COMPRESSED_LUMINANCE_ALPHA_3DC_ATI as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ATI_TEXTURE_COMPRESSION_3DC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_ATI2N,
            gl_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_srgb_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_LUMINANCE_ALPHA as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_ATI2N,
            gl_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_srgb_internal: (GL_COMPRESSED_RG_RGTC2 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RG as i32),
            gl_type: (GL_UNSIGNED_BYTE as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_INTZ,
            gl_internal: (GL_DEPTH24_STENCIL8 as i32),
            gl_srgb_internal: (GL_DEPTH24_STENCIL8 as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_DEPTH_STENCIL as i32),
            gl_type: (GL_UNSIGNED_INT_24_8 as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL) as ::core::ffi::c_uint,
            extension: EXT_PACKED_DEPTH_STENCIL,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_NULL,
            gl_internal: 0 as GLint,
            gl_srgb_internal: 0 as GLint,
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGBA as i32),
            gl_type: (GL_UNSIGNED_INT_8_8_8_8_REV as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_FBO_ATTACHABLE) as ::core::ffi::c_uint,
            extension: ARB_FRAMEBUFFER_OBJECT,
            upload: None,
            download: None,
            decompress: None,
        },
        wined3d_format_texture_info {
            id: WINED3DFMT_R9G9B9E5_SHAREDEXP,
            gl_internal: (GL_RGB9_E5_EXT as i32),
            gl_srgb_internal: (GL_RGB9_E5_EXT as i32),
            gl_rt_internal: 0 as GLint,
            gl_format: (GL_RGB as i32),
            gl_type: (GL_UNSIGNED_INT_5_9_9_9_REV_EXT as i32),
            conv_byte_count: 0 as ::core::ffi::c_uint,
            caps: (WINED3D_FORMAT_CAP_TEXTURE
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING
                | WINED3D_FORMAT_CAP_FILTERING) as ::core::ffi::c_uint,
            extension: EXT_TEXTURE_SHARED_EXPONENT,
            upload: None,
            download: None,
            decompress: None,
        },
    ]
};

static mut format_srgb_info: [wined3d_format_srgb_info; 7] = [
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_R8G8B8A8_UNORM_SRGB,
        base_format_id: WINED3DFMT_R8G8B8A8_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_BC1_UNORM_SRGB,
        base_format_id: WINED3DFMT_BC1_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_BC2_UNORM_SRGB,
        base_format_id: WINED3DFMT_BC2_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_BC3_UNORM_SRGB,
        base_format_id: WINED3DFMT_BC3_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_B8G8R8A8_UNORM_SRGB,
        base_format_id: WINED3DFMT_B8G8R8A8_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_B8G8R8X8_UNORM_SRGB,
        base_format_id: WINED3DFMT_B8G8R8X8_UNORM,
    },
    wined3d_format_srgb_info {
        srgb_format_id: WINED3DFMT_BC7_UNORM_SRGB,
        base_format_id: WINED3DFMT_BC7_UNORM,
    },
];

// WINE_DEFAULT_DEBUG_CHANNEL(d3d)

pub const WINED3D_FORMAT_FOURCC_BASE: u32 = WINED3DFMT_BC7_UNORM_SRGB as u32 + 1;

// WINED3D_FORMAT_COUNT depends on ARRAY_SIZE(format_index_remap); that array is
// defined by the c2rust file-scope data, so its length is taken from there.
pub const WINED3D_FORMAT_COUNT: u32 = WINED3D_FORMAT_FOURCC_BASE + 24; // ARRAY_SIZE(format_index_remap): its type says [_; 24]

#[repr(C)]
pub struct wined3d_format_channels {
    pub id: u32,
    pub red_size: u32,
    pub green_size: u32,
    pub blue_size: u32,
    pub alpha_size: u32,
    pub red_offset: u32,
    pub green_offset: u32,
    pub blue_offset: u32,
    pub alpha_offset: u32,
    pub bpp: u32,
    pub depth_size: u8,
    pub stencil_size: u8,
    pub channels: *const i8,
}

#[repr(C)]
pub struct wined3d_typed_format_info {
    pub id: u32,
    pub typeless_id: u32,
    pub channels: *const i8,
}

#[repr(C)]
pub struct wined3d_typeless_format_depth_stencil_info {
    pub typeless_id: u32,
    pub depth_stencil_id: u32,
    pub depth_view_id: u32,
    pub stencil_view_id: u32,
    pub separate_depth_view_format: i32,
}

#[repr(C)]
pub struct wined3d_format_ddi_info {
    pub id: u32,
    pub ddi_format: u32,
}

#[repr(C)]
pub struct wined3d_format_base_flags {
    pub id: u32,
    pub attrs: u32,
    pub caps: u32,
}

#[repr(C)]
pub struct wined3d_format_decompress_info {
    pub id: u32,
    pub decompress: Option<
        unsafe extern "C" fn(
            src: *const u8,
            dst: *mut u8,
            src_row_pitch: u32,
            src_slice_pitch: u32,
            dst_row_pitch: u32,
            dst_slice_pitch: u32,
            width: u32,
            height: u32,
            depth: u32,
        ),
    >,
}

#[repr(C)]
pub struct wined3d_format_block_info {
    pub id: u32,
    pub block_width: u32,
    pub block_height: u32,
    pub block_byte_count: u32,
    pub attrs: u32,
}

#[repr(C)]
pub struct wined3d_format_vertex_info {
    pub id: u32,
    pub emit_idx: u32,
    pub gl_vtx_type: u32,
    pub extension: u32,
}

#[repr(C)]
pub struct wined3d_format_texture_info {
    pub id: u32,
    pub gl_internal: i32,
    pub gl_srgb_internal: i32,
    pub gl_rt_internal: i32,
    pub gl_format: i32,
    pub gl_type: i32,
    pub conv_byte_count: u32,
    pub caps: u32,
    pub extension: u32,
    pub upload: Option<
        unsafe extern "C" fn(
            src: *const u8,
            dst: *mut u8,
            src_row_pitch: u32,
            src_slice_pitch: u32,
            dst_row_pitch: u32,
            dst_slice_pitch: u32,
            width: u32,
            height: u32,
            depth: u32,
        ),
    >,
    pub download: Option<
        unsafe extern "C" fn(
            src: *const u8,
            dst: *mut u8,
            src_row_pitch: u32,
            src_slice_pitch: u32,
            dst_row_pitch: u32,
            dst_slice_pitch: u32,
            width: u32,
            height: u32,
            depth: u32,
        ),
    >,
    pub decompress: Option<
        unsafe extern "C" fn(
            src: *const u8,
            dst: *mut u8,
            src_row_pitch: u32,
            src_slice_pitch: u32,
            dst_row_pitch: u32,
            dst_slice_pitch: u32,
            width: u32,
            height: u32,
            depth: u32,
        ),
    >,
}

#[repr(C)]
pub struct wined3d_format_srgb_info {
    pub srgb_format_id: u32,
    pub base_format_id: u32,
}

#[repr(C)]
pub struct debug_buffer {
    pub str: [i8; 200],
    pub ptr: *mut i8,
    pub size: i32,
}

unsafe extern "C" fn rgb888_from_rgb565(rgb565: WORD, r: *mut BYTE, g: *mut BYTE, b: *mut BYTE) {
    let mut c: BYTE = 0;

    /* (2⁸ - 1) / (2⁵ - 1) ≈ 2⁸ / 2⁵ + 2⁸ / 2¹⁰
     * (2⁸ - 1) / (2⁶ - 1) ≈ 2⁸ / 2⁶ + 2⁸ / 2¹² */
    c = ((rgb565 as i32) >> 11) as BYTE;
    *r = ((((c as i32) << 3) + ((c as i32) >> 2)) as BYTE);
    c = ((((rgb565 as i32) >> 5) & 0x3f) as BYTE);
    *g = ((((c as i32) << 2) + ((c as i32) >> 4)) as BYTE);
    c = (((rgb565 as i32) & 0x1f) as BYTE);
    *b = ((((c as i32) << 3) + ((c as i32) >> 2)) as BYTE);
}

unsafe extern "C" fn build_dxtn_colour_table(
    colour0: WORD,
    colour1: WORD,
    colour_table: *mut DWORD,
    format_id: wined3d_format_id,
) {
    #[derive(Clone, Copy)]
    struct C2RustUnnamed {
        r: u8,
        g: u8,
        b: u8,
    }

    let mut i: u32;
    let mut c: [C2RustUnnamed; 4] =
        [C2RustUnnamed { r: 0, g: 0, b: 0 }; 4];

    rgb888_from_rgb565(
        colour0,
        core::ptr::addr_of_mut!(c[0 as usize].r),
        core::ptr::addr_of_mut!(c[0 as usize].g),
        core::ptr::addr_of_mut!(c[0 as usize].b),
    );
    rgb888_from_rgb565(
        colour1,
        core::ptr::addr_of_mut!(c[1 as usize].r),
        core::ptr::addr_of_mut!(c[1 as usize].g),
        core::ptr::addr_of_mut!(c[1 as usize].b),
    );

    if format_id as u32 == WINED3DFMT_BC1_UNORM as u32 && (colour0 as i32) <= (colour1 as i32) {
        c[2 as usize].r = ((c[0 as usize].r as i32
            + c[1 as usize].r as i32)
            / 2) as u8;
        c[2 as usize].g = ((c[0 as usize].g as i32
            + c[1 as usize].g as i32)
            / 2) as u8;
        c[2 as usize].b = ((c[0 as usize].b as i32
            + c[1 as usize].b as i32)
            / 2) as u8;

        c[3 as usize].r = 0 as u8;
        c[3 as usize].g = 0 as u8;
        c[3 as usize].b = 0 as u8;
    } else {
        i = 0;
        while i < 2 {
            c[(i + 2) as usize].r = ((2 * c[i as usize].r as i32
                + c[(1 - i) as usize].r as i32)
                / 3) as u8;
            c[(i + 2) as usize].g = ((2 * c[i as usize].g as i32
                + c[(1 - i) as usize].g as i32)
                / 3) as u8;
            c[(i + 2) as usize].b = ((2 * c[i as usize].b as i32
                + c[(1 - i) as usize].b as i32)
                / 3) as u8;
            i += 1;
        }
    }

    i = 0;
    while i < 4 {
        *colour_table.add(i as usize) = ((c[i as usize].r as u32) << 16)
            | ((c[i as usize].g as u32) << 8)
            | (c[i as usize].b as u32);
        i += 1;
    }
}

unsafe extern "C" fn build_bc3_alpha_table(alpha0: BYTE, alpha1: BYTE, alpha_table: *mut BYTE) {
    let mut i: u32;

    *alpha_table.offset(0 as isize) = alpha0;
    *alpha_table.offset(1 as isize) = alpha1;

    if alpha0 as u32 > alpha1 as u32 {
        i = 0;
        while i < 6 {
            *alpha_table.offset((2 + i) as isize) =
                ((((6 - i) * (alpha0 as u32)
                    + (i + 1) * (alpha1 as u32))
                    / 7) as BYTE);
            i += 1;
        }
        return;
    } else {
        i = 0;
        while i < 4 {
            *alpha_table.offset((2 + i) as isize) =
                ((((4 - i) * (alpha0 as u32)
                    + (i + 1) * (alpha1 as u32))
                    / 5) as BYTE);
            i += 1;
        }
        *alpha_table.offset(6 as isize) = 0x00 as BYTE;
        *alpha_table.offset(7 as isize) = 0xff as BYTE;
    }
}

unsafe extern "C" fn decompress_dxtn_block(
    src: *const BYTE,
    dst: *mut BYTE,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    dst_row_pitch: ::core::ffi::c_uint,
    format_id: wined3d_format_id,
) {
    let s: *const UINT64 = src as *const UINT64;
    let mut bc1_alpha: BOOL = (FALSE as i32);
    let mut colour_table: [DWORD; 4] = [0; 4];
    let mut alpha_table: [BYTE; 8] = [0; 8];
    let mut alpha_bits: UINT64;
    let mut colour_bits: DWORD;
    let mut x: ::core::ffi::c_uint;
    let mut y: ::core::ffi::c_uint;
    let mut colour_idx: BYTE;
    let mut dst_row: *mut DWORD;
    let mut alpha: BYTE;

    if format_id as ::core::ffi::c_uint == WINED3DFMT_BC1_UNORM as ::core::ffi::c_uint {
        let mut colour0: WORD;
        let mut colour1: WORD;

        alpha_bits = 0 as UINT64;

        colour0 = (*s.offset(0 as isize) & 0xffff as UINT64) as WORD;
        colour1 = ((*s.offset(0 as isize) >> 16 as UINT64)
            & 0xffff as UINT64) as WORD;
        colour_bits = ((*s.offset(0 as isize) >> 32 as UINT64)
            & 0xffffffff as UINT64) as DWORD;
        build_dxtn_colour_table(
            colour0,
            colour1,
            &raw mut colour_table as *mut DWORD,
            format_id,
        );
        if (colour0 as ::core::ffi::c_int) <= (colour1 as ::core::ffi::c_int) {
            bc1_alpha = (TRUE as i32);
        }
    } else {
        alpha_bits = *s.offset(0 as isize);
        if format_id as ::core::ffi::c_uint == WINED3DFMT_BC3_UNORM as ::core::ffi::c_uint {
            build_bc3_alpha_table(
                (alpha_bits & 0xff as UINT64) as BYTE,
                ((alpha_bits >> 8 as UINT64) & 0xff as UINT64) as BYTE,
                &raw mut alpha_table as *mut BYTE,
            );
            alpha_bits >>= 16 as UINT64;
        }

        colour_bits = ((*s.offset(1 as isize) >> 32 as UINT64)
            & 0xffffffff as UINT64) as DWORD;
        build_dxtn_colour_table(
            (*s.offset(1 as isize) & 0xffff as UINT64) as WORD,
            ((*s.offset(1 as isize) >> 16 as UINT64)
                & 0xffff as UINT64) as WORD,
            &raw mut colour_table as *mut DWORD,
            format_id,
        );
    }

    y = 0 as ::core::ffi::c_uint;
    while y < height {
        dst_row = dst.offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut BYTE as *mut DWORD;
        x = 0 as ::core::ffi::c_uint;
        while x < width {
            colour_idx = ((colour_bits
                >> y.wrapping_mul(8 as ::core::ffi::c_uint)
                    .wrapping_add(x.wrapping_mul(2 as ::core::ffi::c_uint)))
                & 0x3 as DWORD) as BYTE;
            match format_id as ::core::ffi::c_uint {
                WINED3DFMT_BC1_UNORM => {
                    alpha = (if bc1_alpha != 0
                        && colour_idx as ::core::ffi::c_int == 3 as ::core::ffi::c_int
                    {
                        0x00 as ::core::ffi::c_int
                    } else {
                        0xff as ::core::ffi::c_int
                    }) as BYTE;
                }
                WINED3DFMT_BC2_UNORM => {
                    alpha = ((alpha_bits
                        >> y.wrapping_mul(16 as ::core::ffi::c_uint)
                            .wrapping_add(x.wrapping_mul(4 as ::core::ffi::c_uint)))
                        & 0xf as UINT64) as BYTE;
                    alpha = (alpha as ::core::ffi::c_int
                        | ((alpha as ::core::ffi::c_int) << 4 as ::core::ffi::c_int))
                        as BYTE;
                }
                WINED3DFMT_BC3_UNORM => {
                    alpha = alpha_table[((alpha_bits
                        >> y.wrapping_mul(12 as ::core::ffi::c_uint)
                            .wrapping_add(x.wrapping_mul(3 as ::core::ffi::c_uint)))
                        & 0x7 as UINT64) as usize];
                }
                _ => {
                    alpha = 0xff as BYTE;
                }
            }
            *dst_row.offset(x as isize) = (((alpha as ::core::ffi::c_int)
                << 24 as ::core::ffi::c_int) as DWORD)
                | colour_table[colour_idx as usize];
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn decompress_dxtn(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: ::core::ffi::c_uint,
    src_slice_pitch: ::core::ffi::c_uint,
    dst_row_pitch: ::core::ffi::c_uint,
    dst_slice_pitch: ::core::ffi::c_uint,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    depth: ::core::ffi::c_uint,
    format_id: wined3d_format_id,
) {
    let mut block_byte_count: ::core::ffi::c_uint = 0;
    let mut block_w: ::core::ffi::c_uint = 0;
    let mut block_h: ::core::ffi::c_uint = 0;
    let mut src_row: *const BYTE = core::ptr::null();
    let mut src_slice: *const BYTE = src;
    let mut dst_row: *mut BYTE = core::ptr::null_mut();
    let mut dst_slice: *mut BYTE = dst;
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut z: ::core::ffi::c_uint = 0;

    block_byte_count = if format_id as ::core::ffi::c_uint == WINED3DFMT_BC1_UNORM {
        8
    } else {
        16
    };

    z = 0;
    while z < depth {
        src_row = src_slice;
        dst_row = dst_slice;
        y = 0;
        while y < height {
            x = 0;
            while x < width {
                block_w = min!(width.wrapping_sub(x), 4);
                block_h = min!(height.wrapping_sub(y), 4);
                decompress_dxtn_block(
                    src_row.offset(
                        x.wrapping_mul(block_byte_count.wrapping_div(4))
                            as isize,
                    ) as *const BYTE,
                    dst_row.offset(x.wrapping_mul(4) as isize) as *mut BYTE,
                    block_w,
                    block_h,
                    dst_row_pitch,
                    format_id,
                );
                x = x.wrapping_add(4);
            }
            src_row = src_row.offset(src_row_pitch as isize);
            dst_row = dst_row.offset(dst_row_pitch.wrapping_mul(4) as isize);
            y = y.wrapping_add(4);
        }
        src_slice = src_slice.offset(src_slice_pitch as isize);
        dst_slice = dst_slice.offset(dst_slice_pitch as isize);
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn decompress_bc3(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: ::core::ffi::c_uint,
    src_slice_pitch: ::core::ffi::c_uint,
    dst_row_pitch: ::core::ffi::c_uint,
    dst_slice_pitch: ::core::ffi::c_uint,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    depth: ::core::ffi::c_uint,
) {
    decompress_dxtn(src, dst, src_row_pitch, src_slice_pitch, dst_row_pitch,
            dst_slice_pitch, width, height, depth, WINED3DFMT_BC3_UNORM);
}

unsafe extern "C" fn decompress_bc2(src: *const BYTE, dst: *mut BYTE, src_row_pitch: ::core::ffi::c_uint, src_slice_pitch: ::core::ffi::c_uint, dst_row_pitch: ::core::ffi::c_uint, dst_slice_pitch: ::core::ffi::c_uint, width: ::core::ffi::c_uint, height: ::core::ffi::c_uint, depth: ::core::ffi::c_uint) {
    decompress_dxtn(src, dst, src_row_pitch, src_slice_pitch, dst_row_pitch, dst_slice_pitch, width, height, depth, WINED3DFMT_BC2_UNORM);
}

unsafe extern "C" fn decompress_bc1(src: *const BYTE, dst: *mut BYTE, src_row_pitch: core::ffi::c_uint, src_slice_pitch: core::ffi::c_uint, dst_row_pitch: core::ffi::c_uint, dst_slice_pitch: core::ffi::c_uint, width: core::ffi::c_uint, height: core::ffi::c_uint, depth: core::ffi::c_uint) {
    decompress_dxtn(src, dst, src_row_pitch, src_slice_pitch, dst_row_pitch, dst_slice_pitch, width, height, depth, WINED3DFMT_BC1_UNORM);
}

unsafe extern "C" fn build_rgtc_colour_table(
    mut red0: uint8_t,
    mut red1: uint8_t,
    mut colour_table: *mut uint8_t,
) {
    let mut i: ::core::ffi::c_uint = 0;
    *colour_table.offset(0 as ::core::ffi::c_int as isize) = red0;
    *colour_table.offset(1 as ::core::ffi::c_int as isize) = red1;
    if red0 as ::core::ffi::c_int <= red1 as ::core::ffi::c_int {
        i = 0 as ::core::ffi::c_uint;
        while i < 4 as ::core::ffi::c_uint {
            *colour_table.offset(
                i.wrapping_add(2 as ::core::ffi::c_uint) as isize,
            ) = (8 as ::core::ffi::c_uint)
                .wrapping_sub((2 as ::core::ffi::c_uint).wrapping_mul(i))
                .wrapping_mul(red0 as ::core::ffi::c_uint)
                .wrapping_add(
                    (2 as ::core::ffi::c_uint)
                        .wrapping_add(
                            (2 as ::core::ffi::c_uint).wrapping_mul(i),
                        )
                        .wrapping_mul(red1 as ::core::ffi::c_uint),
                )
                .wrapping_add(5 as ::core::ffi::c_uint)
                .wrapping_div(10 as ::core::ffi::c_uint) as uint8_t;
            i = i.wrapping_add(1);
        }
        *colour_table.offset(6 as ::core::ffi::c_int as isize) =
            0x00 as uint8_t;
        *colour_table.offset(7 as ::core::ffi::c_int as isize) =
            0xff as uint8_t;
    } else {
        i = 0 as ::core::ffi::c_uint;
        while i < 6 as ::core::ffi::c_uint {
            *colour_table.offset(
                i.wrapping_add(2 as ::core::ffi::c_uint) as isize,
            ) = (12 as ::core::ffi::c_uint)
                .wrapping_sub((2 as ::core::ffi::c_uint).wrapping_mul(i))
                .wrapping_mul(red0 as ::core::ffi::c_uint)
                .wrapping_add(
                    (2 as ::core::ffi::c_uint)
                        .wrapping_add(
                            (2 as ::core::ffi::c_uint).wrapping_mul(i),
                        )
                        .wrapping_mul(red1 as ::core::ffi::c_uint),
                )
                .wrapping_add(7 as ::core::ffi::c_uint)
                .wrapping_div(14 as ::core::ffi::c_uint) as uint8_t;
            i = i.wrapping_add(1);
        }
    };
}

unsafe extern "C" fn decompress_rgtc_block(src: *const u8, dst: *mut u8, width: u32, height: u32, dst_row_pitch: u32) {
    let s: *const u64 = src as *const u64;
    let mut red0: u8 = 0;
    let mut red1: u8 = 0;
    let mut red_idx: u8 = 0;
    let mut colour_table: [u8; 8] = [0; 8];
    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut dst_row: *mut u32 = core::ptr::null_mut();
    let mut bits: u64 = 0;

    red0 = (*s.offset(0 as isize) & 0xff) as u8;
    red1 = ((*s.offset(0 as isize) >> 8) & 0xff) as u8;
    bits = *s.offset(0 as isize) >> 16;
    build_rgtc_colour_table(red0, red1, colour_table.as_mut_ptr());

    y = 0;
    while y < height {
        dst_row = dst.offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut u8 as *mut u32;
        x = 0;
        while x < width {
            red_idx = ((bits
                >> y.wrapping_mul(12)
                    .wrapping_add(x.wrapping_mul(3)))
                & 0x7) as u8;
            *dst_row.offset(x as isize) = (0xff000000
                | ((colour_table[red_idx as usize] as u32) << 16)) as u32;
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
unsafe extern "C" fn decompress_bc4(src: *const uint8_t, dst: *mut uint8_t, src_row_pitch: ::core::ffi::c_uint, src_slice_pitch: ::core::ffi::c_uint, dst_row_pitch: ::core::ffi::c_uint, dst_slice_pitch: ::core::ffi::c_uint, width: ::core::ffi::c_uint, height: ::core::ffi::c_uint, depth: ::core::ffi::c_uint) {
    let mut block_w: ::core::ffi::c_uint;
    let mut block_h: ::core::ffi::c_uint;
    let mut x: ::core::ffi::c_uint;
    let mut y: ::core::ffi::c_uint;
    let mut z: ::core::ffi::c_uint;
    let mut src_row: *const uint8_t;
    let mut src_slice: *const uint8_t;
    let mut dst_row: *mut uint8_t;
    let mut dst_slice: *mut uint8_t;

    z = 0;
    while z < depth {
        src_slice = src.offset(z.wrapping_mul(src_slice_pitch) as isize) as *const uint8_t;
        dst_slice = dst.offset(z.wrapping_mul(dst_slice_pitch) as isize) as *mut uint8_t;
        y = 0;
        while y < height {
            src_row = src_slice.offset(
                (y / 4).wrapping_mul(src_row_pitch) as isize,
            ) as *const uint8_t;
            dst_row = dst_slice.offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut uint8_t;
            x = 0;
            while x < width {
                block_w = min!(width.wrapping_sub(x), 4);
                block_h = min!(height.wrapping_sub(y), 4);
                decompress_rgtc_block(
                    src_row.offset(
                        (x / 4).wrapping_mul(8) as isize,
                    ) as *const uint8_t,
                    dst_row.offset(x.wrapping_mul(4) as isize) as *mut uint8_t,
                    block_w,
                    block_h,
                    dst_row_pitch,
                );
                x = x.wrapping_add(4);
            }
            y = y.wrapping_add(4);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_l4a4_unorm(
    mut src: *const BYTE,
    mut dst: *mut BYTE,
    mut src_row_pitch: UINT,
    mut src_slice_pitch: UINT,
    mut dst_row_pitch: UINT,
    mut dst_slice_pitch: UINT,
    mut width: UINT,
    mut height: UINT,
    mut depth: UINT,
) {
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut z: ::core::ffi::c_uint = 0;
    let mut Source: *const ::core::ffi::c_uchar = ::core::ptr::null::<::core::ffi::c_uchar>();
    let mut Dest: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    z = 0 as ::core::ffi::c_uint;
    while z < depth {
        y = 0 as ::core::ffi::c_uint;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const ::core::ffi::c_uchar;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut ::core::ffi::c_uchar;
            x = 0 as ::core::ffi::c_uint;
            while x < width {
                let fresh33 = Source;
                Source = Source.offset(1);
                let mut color: ::core::ffi::c_uchar = *fresh33;
                /* A */
                *Dest.offset(1 as isize) =
                    ((color as ::core::ffi::c_uint & 0xf0 as ::core::ffi::c_uint)
                        << 0 as u32) as ::core::ffi::c_uchar;
                /* L */
                *Dest.offset(0 as isize) =
                    ((color as ::core::ffi::c_uint & 0x0f as ::core::ffi::c_uint)
                        << 4 as u32) as ::core::ffi::c_uchar;
                Dest = Dest.offset(2 as isize);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r5g5_snorm_l6_unorm(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: UINT,
    src_slice_pitch: UINT,
    dst_row_pitch: UINT,
    dst_slice_pitch: UINT,
    width: UINT,
    height: UINT,
    depth: UINT,
) {
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut z: ::core::ffi::c_uint = 0;
    let mut r_in: ::core::ffi::c_uchar = 0;
    let mut g_in: ::core::ffi::c_uchar = 0;
    let mut l_in: ::core::ffi::c_uchar = 0;
    let mut texel_in: *const ::core::ffi::c_ushort = ::core::ptr::null::<::core::ffi::c_ushort>();
    let mut texel_out: *mut ::core::ffi::c_ushort =
        ::core::ptr::null_mut::<::core::ffi::c_ushort>();
    z = 0 as ::core::ffi::c_uint;
    while z < depth {
        y = 0 as ::core::ffi::c_uint;
        while y < height {
            texel_out = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut ::core::ffi::c_ushort;
            texel_in = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const ::core::ffi::c_ushort;
            x = 0 as ::core::ffi::c_uint;
            while x < width {
                l_in = ((*texel_in as ::core::ffi::c_uint
                    & 0xfc00 as ::core::ffi::c_uint)
                    >> 10 as ::core::ffi::c_int)
                    as ::core::ffi::c_uchar;
                g_in = ((*texel_in as ::core::ffi::c_uint
                    & 0x03e0 as ::core::ffi::c_uint)
                    >> 5 as ::core::ffi::c_int)
                    as ::core::ffi::c_uchar;
                r_in = (*texel_in as ::core::ffi::c_uint
                    & 0x001f as ::core::ffi::c_uint)
                    as ::core::ffi::c_uchar;
                *texel_out = (((r_in as ::core::ffi::c_int + 16 as ::core::ffi::c_int)
                    << 11 as ::core::ffi::c_int)
                    | ((l_in as ::core::ffi::c_int) << 5 as ::core::ffi::c_int)
                    | (g_in as ::core::ffi::c_int + 16 as ::core::ffi::c_int))
                    as ::core::ffi::c_ushort;
                texel_out = texel_out.offset(1);
                texel_in = texel_in.offset(1);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r5g5_snorm_l6_unorm_ext(
    mut src: *const BYTE,
    mut dst: *mut BYTE,
    mut src_row_pitch: UINT,
    mut src_slice_pitch: UINT,
    mut dst_row_pitch: UINT,
    mut dst_slice_pitch: UINT,
    mut width: UINT,
    mut height: UINT,
    mut depth: UINT,
) {
    let mut x: u32;
    let mut y: u32;
    let mut z: u32;
    let mut texel_out: *mut u8;
    let mut r_out: u8;
    let mut g_out: u8;
    let mut r_in: u8;
    let mut g_in: u8;
    let mut l_in: u8;
    let mut texel_in: *const u16;

    z = 0 as u32;
    while z < depth {
        y = 0 as u32;
        while y < height {
            texel_in = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const u16;
            texel_out = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut u8;
            x = 0 as u32;
            while x < width {
                l_in = ((*texel_in as u32 & (0xfc00 as u32))
                    >> (10 as u32)) as u8;
                g_in = ((*texel_in as u32 & (0x03e0 as u32))
                    >> (5 as u32)) as u8;
                r_in = (*texel_in as u32 & (0x001f as u32)) as u8;

                r_out = ((r_in as u32) << (3 as u32)) as u8;
                if (r_in as u32 & (0x10 as u32)) == 0 {
                    r_out |= r_in >> (1 as u32);
                }

                g_out = ((g_in as u32) << (3 as u32)) as u8;
                if (g_in as u32 & (0x10 as u32)) == 0 {
                    g_out |= g_in >> (1 as u32);
                }

                *texel_out.offset((0 as isize)) = r_out;
                *texel_out.offset((1 as isize)) = g_out;
                *texel_out.offset((2 as isize)) =
                    (l_in << (1 as u32)) | (l_in >> (5 as u32));
                *texel_out.offset((3 as isize)) = (0) as u8;

                texel_out = texel_out.offset((4 as isize));
                texel_in = texel_in.offset(1);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r5g5_snorm_l6_unorm_nv(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: UINT,
    src_slice_pitch: UINT,
    dst_row_pitch: UINT,
    dst_slice_pitch: UINT,
    width: UINT,
    height: UINT,
    depth: UINT,
) {
    let mut x: UINT = 0;
    let mut y: UINT = 0;
    let mut z: UINT = 0;
    let mut texel_out: *mut BYTE = core::ptr::null_mut();
    let mut ds_out: BYTE = 0;
    let mut dt_out: BYTE = 0;
    let mut r_in: BYTE = 0;
    let mut g_in: BYTE = 0;
    let mut l_in: BYTE = 0;
    let mut texel_in: *const u16 = core::ptr::null();

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            texel_in = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const u16;
            texel_out = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut BYTE;
            x = 0;
            while x < width {
                l_in = (((*texel_in as UINT) & 0xfc00) >> 10) as BYTE;
                g_in = (((*texel_in as UINT) & 0x03e0) >> 5) as BYTE;
                r_in = ((*texel_in as UINT) & 0x001f) as BYTE;

                ds_out = (((r_in as UINT) << 3) & 0xff) as BYTE;
                if ((r_in as UINT) & 0x10) == 0 {
                    ds_out = ((ds_out as UINT) | ((r_in as UINT) >> 1)) as BYTE;
                }

                dt_out = (((g_in as UINT) << 3) & 0xff) as BYTE;
                if ((g_in as UINT) & 0x10) == 0 {
                    dt_out = ((dt_out as UINT) | ((g_in as UINT) >> 1)) as BYTE;
                }

                *texel_out.offset(0 as isize) = ds_out;
                *texel_out.offset(1 as isize) = dt_out;
                *texel_out.offset(2 as isize) = ((((l_in as UINT) << 1)
                    | ((l_in as UINT) >> 5)) & 0xff) as BYTE;

                texel_out = texel_out.offset(3 as isize);
                texel_in = texel_in.offset(1);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r8g8_snorm(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: UINT,
    src_slice_pitch: UINT,
    dst_row_pitch: UINT,
    dst_slice_pitch: UINT,
    width: UINT,
    height: UINT,
    depth: UINT,
) {
    let mut x: u32;
    let mut y: u32;
    let mut z: u32;
    let mut Source: *const i16 = core::ptr::null::<i16>();
    let mut Dest: *mut u8 = core::ptr::null_mut::<u8>();

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const i16;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut u8;
            x = 0;
            while x < width {
                let fresh32 = Source;
                Source = Source.offset(1);
                let color: i16 = *fresh32;
                /* B */
                *Dest.add(0) = 0xff;
                /* G */
                *Dest.add(1) =
                    (((color as i32) >> 8) + 128) as u8; /* V */
                /* R */
                *Dest.add(2) =
                    ((((color as i32) & 0xff) + 128) as u8); /* U */
                Dest = Dest.add(3);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r8g8_snorm_l8x8_unorm(src: *const BYTE, dst: *mut BYTE, src_row_pitch: UINT, src_slice_pitch: UINT, dst_row_pitch: UINT, dst_slice_pitch: UINT, width: UINT, height: UINT, depth: UINT) {
    let mut x: UINT;
    let mut y: UINT;
    let mut z: UINT;
    let mut Source: *const DWORD = core::ptr::null::<DWORD>();
    let mut Dest: *mut BYTE = core::ptr::null_mut::<BYTE>();

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = (src as *const u8)
                .offset(z.wrapping_mul(src_slice_pitch) as isize)
                .offset(y.wrapping_mul(src_row_pitch) as isize)
                as *const DWORD;
            Dest = dst
                .offset(z.wrapping_mul(dst_slice_pitch) as isize)
                .offset(y.wrapping_mul(dst_row_pitch) as isize)
                as *mut BYTE;
            x = 0;
            while x < width {
                let fresh31 = Source;
                Source = Source.offset(1);
                let color: LONG = *fresh31 as LONG;
                /* B */
                *Dest.offset(0 as isize) =
                    ((color >> 16) & 0xff) as BYTE;
                /* G */
                *Dest.offset(1 as isize) =
                    (((color >> 8) & 0xff) + 128) as BYTE;
                /* R */
                *Dest.offset(2 as isize) =
                    ((color & 0xff) + 128) as BYTE;
                Dest = Dest.offset(4 as isize);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r8g8_snorm_l8x8_unorm_nv(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: UINT,
    src_slice_pitch: UINT,
    dst_row_pitch: UINT,
    dst_slice_pitch: UINT,
    width: UINT,
    height: UINT,
    depth: UINT,
) {
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut z: ::core::ffi::c_uint = 0;
    let mut Source: *const DWORD = ::core::ptr::null::<DWORD>();
    let mut Dest: *mut ::core::ffi::c_uchar = ::core::ptr::null_mut::<::core::ffi::c_uchar>();

    /* This implementation works with the fixed function pipeline and shaders
     * without further modification after converting the surface.
     */
    z = 0 as ::core::ffi::c_uint;
    while z < depth {
        y = 0 as ::core::ffi::c_uint;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const DWORD;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut ::core::ffi::c_uchar;
            x = 0 as ::core::ffi::c_uint;
            while x < width {
                let fresh30 = Source;
                Source = Source.offset(1);
                let mut color: LONG = *fresh30 as LONG;
                /* L */
                *Dest.offset(2 as ::core::ffi::c_int as isize) =
                    ((color as ::core::ffi::c_int >> 16 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as ::core::ffi::c_uchar);
                /* V */
                *Dest.offset(1 as ::core::ffi::c_int as isize) =
                    ((color as ::core::ffi::c_int >> 8 as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as ::core::ffi::c_uchar);
                /* U */
                *Dest.offset(0 as ::core::ffi::c_int as isize) =
                    ((color as ::core::ffi::c_int
                        & 0xff as ::core::ffi::c_int) as ::core::ffi::c_uchar);
                /* X */
                *Dest.offset(3 as ::core::ffi::c_int as isize) =
                    255 as ::core::ffi::c_uchar;
                Dest = Dest.offset(4 as ::core::ffi::c_int as isize);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r8g8b8a8_snorm(src: *const BYTE, dst: *mut BYTE, src_row_pitch: UINT, src_slice_pitch: UINT, dst_row_pitch: UINT, dst_slice_pitch: UINT, width: UINT, height: UINT, depth: UINT) {
    let mut x: ::core::ffi::c_uint;
    let mut y: ::core::ffi::c_uint;
    let mut z: ::core::ffi::c_uint;
    let mut Source: *const DWORD;
    let mut Dest: *mut ::core::ffi::c_uchar;

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const DWORD;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut ::core::ffi::c_uchar;
            x = 0;
            while x < width {
                let fresh = Source;
                Source = Source.offset(1);
                let color: LONG = *fresh as LONG;
                *Dest.offset(0 as isize) =
                    ((((color >> 16) & 0xff) + 128) as ::core::ffi::c_int) as ::core::ffi::c_uchar;
                *Dest.offset(1 as isize) =
                    ((((color >> 8) & 0xff) + 128) as ::core::ffi::c_int) as ::core::ffi::c_uchar;
                *Dest.offset(2 as isize) =
                    (((color & 0xff) + 128) as ::core::ffi::c_int) as ::core::ffi::c_uchar;
                *Dest.offset(3 as isize) =
                    ((((color >> 24) & 0xff) + 128) as ::core::ffi::c_int) as ::core::ffi::c_uchar;
                Dest = Dest.offset(4 as isize);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r16g16_snorm(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: UINT,
    src_slice_pitch: UINT,
    dst_row_pitch: UINT,
    dst_slice_pitch: UINT,
    width: UINT,
    height: UINT,
    depth: UINT,
) {
    let mut x: UINT;
    let mut y: UINT;
    let mut z: UINT;
    let mut Source: *const DWORD;
    let mut Dest: *mut u16;

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = src.offset((z as isize).wrapping_mul(src_slice_pitch as isize))
                .offset((y as isize).wrapping_mul(src_row_pitch as isize))
                as *const DWORD;
            Dest = dst.offset((z as isize).wrapping_mul(dst_slice_pitch as isize))
                .offset((y as isize).wrapping_mul(dst_row_pitch as isize))
                as *mut u16;
            x = 0;
            while x < width {
                let color: DWORD = *Source;
                Source = Source.add(1);
                /* B */
                *Dest.add(0) = 0xffff;
                /* G */
                *Dest.add(1) = (color >> 16).wrapping_add(32768) as u16; /* V */
                /* R */
                *Dest.add(2) = (color & 0xffff).wrapping_add(32768) as u16; /* U */
                Dest = Dest.add(3);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r16g16(src: *const BYTE, dst: *mut BYTE, src_row_pitch: UINT, src_slice_pitch: UINT, dst_row_pitch: UINT, dst_slice_pitch: UINT, width: UINT, height: UINT, depth: UINT) {
    let mut x: UINT = 0;
    let mut y: UINT = 0;
    let mut z: UINT = 0;
    let mut Source: *const WORD = core::ptr::null::<WORD>();
    let mut Dest: *mut WORD = core::ptr::null_mut::<WORD>();
    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const WORD;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut WORD;
            x = 0;
            while x < width {
                let green: WORD = *Source;
                Source = Source.offset(1);
                let red: WORD = *Source;
                Source = Source.offset(1);
                *Dest.offset(0) = green;
                *Dest.offset(1) = red;
                /* Strictly speaking not correct for R16G16F, but it doesn't matter because the
                 * shader overwrites it anyway */
                *Dest.offset(2) = 0xffff;
                Dest = Dest.offset(3);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r32g32_float(src: *const BYTE, dst: *mut BYTE, src_row_pitch: UINT, src_slice_pitch: UINT, dst_row_pitch: UINT, dst_slice_pitch: UINT, width: UINT, height: UINT, depth: UINT) {
    let mut x: UINT;
    let mut y: UINT;
    let mut z: UINT;
    let mut Source: *const f32 = core::ptr::null();
    let mut Dest: *mut f32 = core::ptr::null_mut();
    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            Source = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const f32;
            Dest = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut f32;
            x = 0;
            while x < width {
                let fresh0 = Source;
                Source = Source.offset(1);
                let green: f32 = *fresh0;
                let fresh1 = Source;
                Source = Source.offset(1);
                let red: f32 = *fresh1;
                *Dest.offset(0 as isize) = green;
                *Dest.offset(1 as isize) = red;
                *Dest.offset(2 as isize) = 1.0f32;
                Dest = Dest.offset(3 as isize);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_s8_uint_d24_float(src: *const BYTE, dst: *mut BYTE, src_row_pitch: UINT, src_slice_pitch: UINT, dst_row_pitch: UINT, dst_slice_pitch: UINT, width: UINT, height: UINT, depth: UINT) {
    let mut x: UINT;
    let mut y: UINT;
    let mut z: UINT;

    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            let source: *const DWORD = src
                .offset((z as UINT).wrapping_mul(src_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(src_row_pitch) as isize)
                as *const DWORD;
            let dest_f: *mut f32 = dst
                .offset((z as UINT).wrapping_mul(dst_slice_pitch) as isize)
                .offset((y as UINT).wrapping_mul(dst_row_pitch) as isize)
                as *mut f32;
            let dest_s: *mut DWORD = dest_f as *mut DWORD;

            x = 0;
            while x < width {
                *dest_f.offset(x.wrapping_mul(2) as isize) = float_24_to_32(
                    (*source.offset(x as isize) & 0xffffff00) >> 8,
                );
                *dest_s.offset(
                    x.wrapping_mul(2)
                        .wrapping_add(1) as isize,
                ) = *source.offset(x as isize) & 0xff;
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn x8_d24_unorm_upload(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: u32,
    src_slice_pitch: u32,
    dst_row_pitch: u32,
    dst_slice_pitch: u32,
    width: u32,
    height: u32,
    depth: u32,
) {
    let mut x: u32;
    let mut y: u32;
    let mut z: u32;
    z = 0;
    while z < depth {
        y = 0;
        while y < height {
            let source: *const DWORD =
                src.offset(z.wrapping_mul(src_slice_pitch) as isize)
                    .offset(y.wrapping_mul(src_row_pitch) as isize) as *const DWORD;
            let dest: *mut DWORD =
                dst.offset(z.wrapping_mul(dst_slice_pitch) as isize)
                    .offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut DWORD;
            x = 0;
            while x < width {
                *dest.offset(x as isize) = (*source.offset(x as isize) << 8)
                    | ((*source.offset(x as isize) >> 16) & 0xff);
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn x8_d24_unorm_download(
    src: *const BYTE,
    dst: *mut BYTE,
    src_row_pitch: ::core::ffi::c_uint,
    src_slice_pitch: ::core::ffi::c_uint,
    dst_row_pitch: ::core::ffi::c_uint,
    dst_slice_pitch: ::core::ffi::c_uint,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    depth: ::core::ffi::c_uint,
) {
    let mut x: ::core::ffi::c_uint;
    let mut y: ::core::ffi::c_uint;
    let mut z: ::core::ffi::c_uint;
    z = 0 as ::core::ffi::c_uint;
    while z < depth {
        y = 0 as ::core::ffi::c_uint;
        while y < height {
            let source: *const DWORD =
                src.offset(z.wrapping_mul(src_slice_pitch) as isize)
                    .offset(y.wrapping_mul(src_row_pitch) as isize) as *const DWORD;
            let dest: *mut DWORD =
                dst.offset(z.wrapping_mul(dst_slice_pitch) as isize)
                    .offset(y.wrapping_mul(dst_row_pitch) as isize) as *mut DWORD;
            x = 0 as ::core::ffi::c_uint;
            while x < width {
                *dest.offset(x as isize) = *source.offset(x as isize) >> 8;
                x = x.wrapping_add(1);
            }
            y = y.wrapping_add(1);
        }
        z = z.wrapping_add(1);
    }
}

unsafe extern "C" fn color_in_range(color_key: *const wined3d_color_key, color: DWORD) -> BOOL {
    return (color >= (*color_key).color_space_low_value
        && color <= (*color_key).color_space_high_value) as BOOL;
}

unsafe extern "C" fn convert_b5g6r5_unorm_b5g5r5a1_unorm_color_key(src: *const BYTE, src_pitch: u32, dst: *mut BYTE, dst_pitch: u32, width: u32, height: u32, color_key: *const wined3d_color_key) {
    let mut src_row: *const WORD;
    let mut x: u32;
    let mut y: u32;
    let mut dst_row: *mut WORD;

    y = 0;
    while y < height {
        src_row = src.add((src_pitch.wrapping_mul(y)) as usize) as *const WORD;
        dst_row = dst.add((dst_pitch.wrapping_mul(y)) as usize) as *mut WORD;
        x = 0;
        while x < width {
            let src_color: WORD = *src_row.add(x as usize);
            if color_in_range(color_key, src_color as DWORD) == 0 {
                *dst_row.add(x as usize) = (0x8000
                    | ((src_color & 0xffc0) >> 1)
                    | (src_color & 0x1f)) as WORD;
            } else {
                *dst_row.add(x as usize) = (((src_color & 0xffc0) >> 1)
                    | (src_color & 0x1f)) as WORD;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_b5g5r5x1_unorm_b5g5r5a1_unorm_color_key(
    mut src: *const BYTE,
    mut src_pitch: ::core::ffi::c_uint,
    mut dst: *mut BYTE,
    mut dst_pitch: ::core::ffi::c_uint,
    mut width: ::core::ffi::c_uint,
    mut height: ::core::ffi::c_uint,
    mut color_key: *const wined3d_color_key,
) {
    let mut src_row: *const WORD = ::core::ptr::null::<WORD>();
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut dst_row: *mut WORD = ::core::ptr::null_mut::<WORD>();
    y = 0 as ::core::ffi::c_uint;
    while y < height {
        src_row = src.offset(src_pitch.wrapping_mul(y) as isize) as *const BYTE as *mut WORD;
        dst_row = dst.offset(dst_pitch.wrapping_mul(y) as isize) as *mut BYTE as *mut WORD;
        x = 0 as ::core::ffi::c_uint;
        while x < width {
            let mut src_color: WORD = *src_row.offset(x as isize);
            if color_in_range(color_key, src_color as DWORD) != 0 {
                *dst_row.offset(x as isize) =
                    (src_color as ::core::ffi::c_int
                        & !(0x8000 as ::core::ffi::c_int)) as WORD;
            } else {
                *dst_row.offset(x as isize) =
                    (src_color as ::core::ffi::c_int
                        | 0x8000 as ::core::ffi::c_int) as WORD;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_b8g8r8_unorm_b8g8r8a8_unorm_color_key(src: *const BYTE, src_pitch: u32, dst: *mut BYTE, dst_pitch: u32, width: u32, height: u32, color_key: *const wined3d_color_key) {
    let mut src_row: *const BYTE;
    let mut x: u32;
    let mut y: u32;
    let mut dst_row: *mut DWORD;
    y = 0;
    while y < height {
        src_row = src.offset(src_pitch.wrapping_mul(y) as isize) as *const BYTE;
        dst_row = dst.offset(dst_pitch.wrapping_mul(y) as isize) as *mut BYTE as *mut DWORD;
        x = 0;
        while x < width {
            let mut src_color: DWORD = (((*src_row.offset(
                x.wrapping_mul(3).wrapping_add(2) as isize,
            ) as DWORD) << 16)
                | ((*src_row.offset(
                    x.wrapping_mul(3).wrapping_add(1) as isize,
                ) as DWORD) << 8)
                | (*src_row.offset(
                    x.wrapping_mul(3) as isize,
                ) as DWORD)) as DWORD;
            if color_in_range(color_key, src_color) == 0 {
                *dst_row.offset(x as isize) = src_color | 0xff000000;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_b8g8r8x8_unorm_b8g8r8a8_unorm_color_key(src: *const BYTE, src_pitch: core::ffi::c_uint, dst: *mut BYTE, dst_pitch: core::ffi::c_uint, width: core::ffi::c_uint, height: core::ffi::c_uint, color_key: *const wined3d_color_key) {
    let mut src_row: *const DWORD;
    let mut x: core::ffi::c_uint;
    let mut y: core::ffi::c_uint;
    let mut dst_row: *mut DWORD;

    y = 0;
    while y < height {
        src_row = src.offset(src_pitch.wrapping_mul(y) as isize) as *const BYTE as *mut DWORD;
        dst_row = dst.offset(dst_pitch.wrapping_mul(y) as isize) as *mut BYTE as *mut DWORD;
        x = 0;
        while x < width {
            let src_color: DWORD = *src_row.offset(x as isize);
            if color_in_range(color_key, src_color) != 0 {
                *dst_row.offset(x as isize) = (src_color as core::ffi::c_uint & !(0xff000000 as core::ffi::c_uint)) as DWORD;
            } else {
                *dst_row.offset(x as isize) = (src_color as core::ffi::c_uint | 0xff000000 as core::ffi::c_uint) as DWORD;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_b8g8r8a8_unorm_b8g8r8a8_unorm_color_key(
    src: *const BYTE,
    src_pitch: ::core::ffi::c_uint,
    dst: *mut BYTE,
    dst_pitch: ::core::ffi::c_uint,
    width: ::core::ffi::c_uint,
    height: ::core::ffi::c_uint,
    color_key: *const wined3d_color_key,
) {
    let mut src_row: *const DWORD;
    let mut x: ::core::ffi::c_uint;
    let mut y: ::core::ffi::c_uint;
    let mut dst_row: *mut DWORD;
    y = 0;
    while y < height {
        src_row = src.add(src_pitch.wrapping_mul(y) as usize) as *const DWORD;
        dst_row = dst.add(dst_pitch.wrapping_mul(y) as usize) as *mut DWORD;
        x = 0;
        while x < width {
            let mut src_color: DWORD = *src_row.add(x as usize);
            if color_in_range(color_key, src_color) != 0 {
                src_color &= !(0xff000000);
            }
            *dst_row.add(x as usize) = src_color;
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_get_color_key_conversion(
    mut texture: *const wined3d_texture,
    mut need_alpha_ck: BOOL,
) -> *const wined3d_color_key_conversion {
    let mut format: *const wined3d_format = (*texture).resource.format;
    let mut i: ::core::ffi::c_uint = 0;
    static mut color_key_info: [C2RustUnnamed_51; 5] = unsafe {
        [
            C2RustUnnamed_51 {
                src_format: WINED3DFMT_B5G6R5_UNORM,
                conversion: wined3d_color_key_conversion {
                    dst_format: WINED3DFMT_B5G5R5A1_UNORM,
                    convert: Some(
                        convert_b5g6r5_unorm_b5g5r5a1_unorm_color_key
                            as unsafe extern "C" fn(
                                *const BYTE,
                                ::core::ffi::c_uint,
                                *mut BYTE,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                *const wined3d_color_key,
                            ) -> (),
                    ),
                },
            },
            C2RustUnnamed_51 {
                src_format: WINED3DFMT_B5G5R5X1_UNORM,
                conversion: wined3d_color_key_conversion {
                    dst_format: WINED3DFMT_B5G5R5A1_UNORM,
                    convert: Some(
                        convert_b5g5r5x1_unorm_b5g5r5a1_unorm_color_key
                            as unsafe extern "C" fn(
                                *const BYTE,
                                ::core::ffi::c_uint,
                                *mut BYTE,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                *const wined3d_color_key,
                            ) -> (),
                    ),
                },
            },
            C2RustUnnamed_51 {
                src_format: WINED3DFMT_B8G8R8_UNORM,
                conversion: wined3d_color_key_conversion {
                    dst_format: WINED3DFMT_B8G8R8A8_UNORM,
                    convert: Some(
                        convert_b8g8r8_unorm_b8g8r8a8_unorm_color_key
                            as unsafe extern "C" fn(
                                *const BYTE,
                                ::core::ffi::c_uint,
                                *mut BYTE,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                *const wined3d_color_key,
                            ) -> (),
                    ),
                },
            },
            C2RustUnnamed_51 {
                src_format: WINED3DFMT_B8G8R8X8_UNORM,
                conversion: wined3d_color_key_conversion {
                    dst_format: WINED3DFMT_B8G8R8A8_UNORM,
                    convert: Some(
                        convert_b8g8r8x8_unorm_b8g8r8a8_unorm_color_key
                            as unsafe extern "C" fn(
                                *const BYTE,
                                ::core::ffi::c_uint,
                                *mut BYTE,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                *const wined3d_color_key,
                            ) -> (),
                    ),
                },
            },
            C2RustUnnamed_51 {
                src_format: WINED3DFMT_B8G8R8A8_UNORM,
                conversion: wined3d_color_key_conversion {
                    dst_format: WINED3DFMT_B8G8R8A8_UNORM,
                    convert: Some(
                        convert_b8g8r8a8_unorm_b8g8r8a8_unorm_color_key
                            as unsafe extern "C" fn(
                                *const BYTE,
                                ::core::ffi::c_uint,
                                *mut BYTE,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                ::core::ffi::c_uint,
                                *const wined3d_color_key,
                            ) -> (),
                    ),
                },
            },
        ]
    };
    if need_alpha_ck != 0
        && ((*texture).async_.flags & (WINED3D_TEXTURE_ASYNC_COLOR_KEY as uint32_t)) != 0
    {
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(color_key_info) as usize {
            if color_key_info[i as usize].src_format as ::core::ffi::c_uint
                == (*format).id as ::core::ffi::c_uint
            {
                return &raw const (*(&raw const color_key_info as *const C2RustUnnamed_51)
                    .offset(i as isize))
                .conversion;
            }
            i = i.wrapping_add(1);
        }
        WINE_FIXME!("Color-keying not supported with format %s.\n", debug_d3dformat((*format).id));
    }
    return ::core::ptr::null::<wined3d_color_key_conversion>();
}

unsafe extern "C" fn get_format_idx(format_id: wined3d_format_id) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_uint;
    if (format_id as ::core::ffi::c_uint) < WINED3D_FORMAT_FOURCC_BASE {
        return format_id as ::core::ffi::c_int;
    }
    i = 0;
    while (i as usize) < (ARRAY_SIZE!(format_index_remap) as usize) {
        if format_index_remap[i as usize].id as ::core::ffi::c_uint
            == format_id as ::core::ffi::c_uint
        {
            return format_index_remap[i as usize].idx as ::core::ffi::c_int;
        }
        i += 1;
    }
    return -(1 as ::core::ffi::c_int);
}

unsafe extern "C" fn wined3d_format_gl_mutable(
    mut format: *mut wined3d_format,
) -> *mut wined3d_format_gl {
    return (format as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(wined3d_format_gl, f) as ULONG_PTR)
        as *mut wined3d_format_gl;
}

unsafe extern "C" fn wined3d_format_vk_mutable(format: *mut wined3d_format) -> *mut wined3d_format_vk {
    return CONTAINING_RECORD!(format, wined3d_format_vk, f);
}

unsafe extern "C" fn get_format_by_idx(
    adapter: *const wined3d_adapter,
    fmt_idx: core::ffi::c_int,
) -> *mut wined3d_format {
    return ((*adapter).formats as *mut BYTE)
        .offset((fmt_idx as usize).wrapping_mul((*adapter).format_size) as isize)
        as *mut wined3d_format;
}

unsafe extern "C" fn get_format_gl_by_idx(adapter: *const wined3d_adapter, fmt_idx: i32) -> *mut wined3d_format_gl {
    return wined3d_format_gl_mutable(get_format_by_idx(adapter, fmt_idx));
}

unsafe extern "C" fn get_format_internal(
    adapter: *const wined3d_adapter,
    format_id: wined3d_format_id,
) -> *mut wined3d_format {
    let mut fmt_idx: i32;

    fmt_idx = get_format_idx(format_id);
    if fmt_idx == -1 {
        ERR!(
            "Format %s (%#x) not found.\n",
            debug_d3dformat(format_id),
            format_id
        );
        return core::ptr::null_mut();
    }

    return get_format_by_idx(adapter, fmt_idx);
}

unsafe extern "C" fn get_format_gl_internal(
    adapter: *const wined3d_adapter,
    format_id: wined3d_format_id,
) -> *mut wined3d_format_gl {
    let mut format: *mut wined3d_format = core::ptr::null_mut();

    format = get_format_internal(adapter, format_id);
    if !format.is_null() {
        return wined3d_format_gl_mutable(format);
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn copy_format(
    adapter: *const wined3d_adapter,
    dst_format: *mut wined3d_format,
    src_format: *const wined3d_format,
) {
    let id: wined3d_format_id = (*dst_format).id;
    memcpy(
        dst_format as *mut core::ffi::c_void,
        src_format as *const core::ffi::c_void,
        (*adapter).format_size as usize,
    );
    (*dst_format).id = id;
}

unsafe extern "C" fn format_set_caps(format: *mut wined3d_format, caps: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize) < ARRAY_SIZE!((*format).caps) as usize {
        (*format).caps[i as usize] |= caps;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn format_clear_caps(format: *mut wined3d_format, caps: ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

    while (i as usize) < ARRAY_SIZE!((*format).caps) as usize {
        (*format).caps[i as usize] &= !caps;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn map_channel_type(t: ::core::ffi::c_char) -> wined3d_channel_type {
    match t as u8 {
        b'u' => {
            return WINED3D_CHANNEL_TYPE_UNORM;
        }
        b'i' => {
            return WINED3D_CHANNEL_TYPE_SNORM;
        }
        b'U' => {
            return WINED3D_CHANNEL_TYPE_UINT;
        }
        b'I' => {
            return WINED3D_CHANNEL_TYPE_SINT;
        }
        b'F' => {
            return WINED3D_CHANNEL_TYPE_FLOAT;
        }
        b'D' => {
            return WINED3D_CHANNEL_TYPE_DEPTH;
        }
        b'S' => {
            return WINED3D_CHANNEL_TYPE_STENCIL;
        }
        b'X' => {
            return WINED3D_CHANNEL_TYPE_UNUSED;
        }
        _ => {
            ERR!("Invalid channel type '%c'.\n", t as core::ffi::c_int);
            return WINED3D_CHANNEL_TYPE_NONE;
        }
    }
}

unsafe extern "C" fn parse_channel_desc(
    mut format: *mut wined3d_format,
    mut channel_desc: *const ::core::ffi::c_char,
) {
    let mut component_count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut attrs: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint = 0;
    j = 0 as ::core::ffi::c_uint;
    while (j as usize) < strlen(channel_desc) {
        let mut channel_type: wined3d_channel_type =
            map_channel_type(*channel_desc.offset(j as isize));
        if channel_type as ::core::ffi::c_uint
            == WINED3D_CHANNEL_TYPE_UNORM as ::core::ffi::c_uint
            || channel_type as ::core::ffi::c_uint
                == WINED3D_CHANNEL_TYPE_SNORM as ::core::ffi::c_uint
        {
            attrs |= WINED3D_FORMAT_ATTR_NORMALISED as ::core::ffi::c_uint;
        }
        if channel_type as ::core::ffi::c_uint
            == WINED3D_CHANNEL_TYPE_UINT as ::core::ffi::c_uint
            || channel_type as ::core::ffi::c_uint
                == WINED3D_CHANNEL_TYPE_SINT as ::core::ffi::c_uint
        {
            attrs |= WINED3D_FORMAT_ATTR_INTEGER as ::core::ffi::c_uint;
        }
        if channel_type as ::core::ffi::c_uint
            == WINED3D_CHANNEL_TYPE_FLOAT as ::core::ffi::c_uint
        {
            attrs |= WINED3D_FORMAT_ATTR_FLOAT as ::core::ffi::c_uint;
        }
        if channel_type as ::core::ffi::c_uint
            != WINED3D_CHANNEL_TYPE_UNUSED as ::core::ffi::c_uint
        {
            component_count = component_count.wrapping_add(1);
        }
        if channel_type as ::core::ffi::c_uint
            == WINED3D_CHANNEL_TYPE_DEPTH as ::core::ffi::c_uint
            && (*format).depth_size == 0
        {
            (*format).depth_size = (*format).red_size as BYTE;
            (*format).red_offset = 0 as DWORD;
            (*format).red_size = (*format).red_offset;
        }
        if channel_type as ::core::ffi::c_uint
            == WINED3D_CHANNEL_TYPE_STENCIL as ::core::ffi::c_uint
            && (*format).stencil_size == 0
        {
            (*format).stencil_size = (*format).green_size as BYTE;
            (*format).green_offset = 0 as DWORD;
            (*format).green_size = (*format).green_offset;
        }
        j = j.wrapping_add(1);
    }
    (*format).attrs |= attrs;
    (*format).component_count = component_count;
}

unsafe extern "C" fn init_format_base_info(adapter: *mut wined3d_adapter) -> BOOL {
    let mut format: *mut wined3d_format = core::ptr::null_mut::<wined3d_format>();
    let mut i: core::ffi::c_uint = 0;
    i = 0 as core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(formats) as usize) {
        format = get_format_internal(adapter, formats[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).id = formats[i as usize].id;
        (*format).red_size = formats[i as usize].red_size;
        (*format).green_size = formats[i as usize].green_size;
        (*format).blue_size = formats[i as usize].blue_size;
        (*format).alpha_size = formats[i as usize].alpha_size;
        (*format).red_offset = formats[i as usize].red_offset;
        (*format).green_offset = formats[i as usize].green_offset;
        (*format).blue_offset = formats[i as usize].blue_offset;
        (*format).alpha_offset = formats[i as usize].alpha_offset;
        (*format).byte_count = formats[i as usize].bpp;
        (*format).depth_size = formats[i as usize].depth_size;
        (*format).stencil_size = formats[i as usize].stencil_size;
        (*format).block_width = 1 as UINT;
        (*format).block_height = 1 as UINT;
        (*format).block_byte_count = formats[i as usize].bpp;
        if !formats[i as usize].channels.is_null() {
            parse_channel_desc(format, formats[i as usize].channels);
        }
        i = i.wrapping_add(1);
    }
    i = 0 as core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(typed_formats) as usize) {
        let mut typeless_format: *mut wined3d_format = core::ptr::null_mut::<wined3d_format>();
        format = get_format_internal(adapter, typed_formats[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        typeless_format = get_format_internal(adapter, typed_formats[i as usize].typeless_id);
        if typeless_format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).id = typed_formats[i as usize].id;
        (*format).red_size = (*typeless_format).red_size;
        (*format).green_size = (*typeless_format).green_size;
        (*format).blue_size = (*typeless_format).blue_size;
        (*format).alpha_size = (*typeless_format).alpha_size;
        (*format).red_offset = (*typeless_format).red_offset;
        (*format).green_offset = (*typeless_format).green_offset;
        (*format).blue_offset = (*typeless_format).blue_offset;
        (*format).alpha_offset = (*typeless_format).alpha_offset;
        (*format).byte_count = (*typeless_format).byte_count;
        (*format).depth_size = (*typeless_format).depth_size;
        (*format).stencil_size = (*typeless_format).stencil_size;
        (*format).block_width = (*typeless_format).block_width;
        (*format).block_height = (*typeless_format).block_height;
        (*format).block_byte_count = (*typeless_format).block_byte_count;
        (*format).typeless_id = (*typeless_format).id;
        (*typeless_format).typeless_id = (*typeless_format).id;
        parse_channel_desc(format, typed_formats[i as usize].channels);
        i = i.wrapping_add(1);
    }
    i = 0 as core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(ddi_formats) as usize) {
        format = get_format_internal(adapter, ddi_formats[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).ddi_format = ddi_formats[i as usize].ddi_format;
        i = i.wrapping_add(1);
    }
    i = 0 as core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(format_base_flags) as usize) {
        format = get_format_internal(adapter, format_base_flags[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).attrs |= format_base_flags[i as usize].attrs;
        format_set_caps(format, format_base_flags[i as usize].caps);
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn init_format_block_info(mut adapter: *mut wined3d_adapter) -> BOOL {
    let mut format: *mut wined3d_format = core::ptr::null_mut::<wined3d_format>();
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize) < ARRAY_SIZE!(format_block_info) as usize {
        format = get_format_internal(adapter, format_block_info[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).block_width = format_block_info[i as usize].block_width;
        (*format).block_height = format_block_info[i as usize].block_height;
        (*format).block_byte_count = format_block_info[i as usize].block_byte_count;
        (*format).attrs |= WINED3D_FORMAT_ATTR_BLOCKS | format_block_info[i as usize].attrs;
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn init_format_decompress_info(adapter: *mut wined3d_adapter) -> BOOL {
    let mut format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize) < (ARRAY_SIZE!(format_decompress_info) as usize) {
        format = get_format_internal(adapter, format_decompress_info[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        (*format).caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] |= WINED3D_FORMAT_CAP_DECOMPRESS as ::core::ffi::c_uint;
        (*format).decompress = format_decompress_info[i as usize].decompress;
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn init_srgb_formats(adapter: *mut wined3d_adapter) -> BOOL {
    let mut format: *mut wined3d_format = core::ptr::null_mut();
    let mut srgb_format: *mut wined3d_format = core::ptr::null_mut();
    let mut i: u32 = 0;
    while i < ARRAY_SIZE!(format_srgb_info) as u32 {
        srgb_format = get_format_internal(adapter, format_srgb_info[i as usize].srgb_format_id);
        if srgb_format.is_null() {
            return (FALSE) as BOOL;
        }
        format = get_format_internal(adapter, format_srgb_info[i as usize].base_format_id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }

        copy_format(adapter, srgb_format, format);
        i = i.wrapping_add(1);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn wined3d_gl_type_to_enum(type_0: wined3d_gl_resource_type) -> GLenum {
    match type_0 as ::core::ffi::c_uint {
        WINED3D_GL_RES_TYPE_TEX_1D => return GL_TEXTURE_1D as GLenum,
        WINED3D_GL_RES_TYPE_TEX_2D => return GL_TEXTURE_2D as GLenum,
        WINED3D_GL_RES_TYPE_TEX_3D => return GL_TEXTURE_3D as GLenum,
        WINED3D_GL_RES_TYPE_TEX_CUBE => return GL_TEXTURE_CUBE_MAP_ARB as GLenum,
        WINED3D_GL_RES_TYPE_BUFFER => return GL_TEXTURE_2D as GLenum, /* TODO: GL_TEXTURE_BUFFER. */
        WINED3D_GL_RES_TYPE_RB => return GL_RENDERBUFFER as GLenum,
        WINED3D_GL_RES_TYPE_COUNT | _ => {}
    }
    ERR!("Unexpected GL resource type %u.\n", type_0 as ::core::ffi::c_uint);
    return 0 as GLenum;
}

unsafe extern "C" fn delete_fbo_attachment(
    gl_info: *const wined3d_gl_info,
    d3d_type: wined3d_gl_resource_type,
    object: GLuint,
) {
    let mut object = object;
    match d3d_type as ::core::ffi::c_uint {
        WINED3D_GL_RES_TYPE_TEX_1D | WINED3D_GL_RES_TYPE_TEX_2D | WINED3D_GL_RES_TYPE_TEX_3D | WINED3D_GL_RES_TYPE_TEX_CUBE => {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDeleteTextures
                .unwrap_unchecked()(
                1 as GLsizei,
                core::ptr::addr_of_mut!(object),
            );
        }
        WINED3D_GL_RES_TYPE_RB => {
            (*gl_info)
                .fbo_ops
                .glDeleteRenderbuffers
                .unwrap_unchecked()(
                1 as GLsizei,
                core::ptr::addr_of_mut!(object),
            );
        }
        WINED3D_GL_RES_TYPE_BUFFER | WINED3D_GL_RES_TYPE_COUNT | _ => {}
    };
}

unsafe extern "C" fn create_and_bind_fbo_attachment(
    gl_info: *const wined3d_gl_info,
    format_gl: *const wined3d_format_gl,
    d3d_type: wined3d_gl_resource_type,
    object: *mut GLuint,
    internal: GLenum,
) {
    let format: GLenum = (*format_gl).format as GLenum;
    let type_0: GLenum = (*format_gl).type_ as GLenum;
    let mut attach_type: GLenum;

    attach_type = if (*format_gl).f.depth_size != 0 { GL_DEPTH_ATTACHMENT } else { GL_COLOR_ATTACHMENT0 };
    match d3d_type as ::core::ffi::c_uint {
        WINED3D_GL_RES_TYPE_TEX_1D => {
            (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(1, object);
            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(GL_TEXTURE_1D, *object);
            (*gl_info).gl_ops.gl.p_glTexImage1D.unwrap_unchecked()(
                GL_TEXTURE_1D, 0, internal as GLint, 16,
                0, format, type_0, ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_1D, GL_TEXTURE_MIN_FILTER, (GL_NEAREST as i32),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_1D, GL_TEXTURE_MAG_FILTER, (GL_NEAREST as i32),
            );

            (*gl_info).fbo_ops.glFramebufferTexture1D.unwrap_unchecked()(
                GL_FRAMEBUFFER, attach_type, GL_TEXTURE_1D, *object, 0,
            );
            if (*format_gl).f.stencil_size != 0 {
                (*gl_info).fbo_ops.glFramebufferTexture1D.unwrap_unchecked()(
                    GL_FRAMEBUFFER, GL_STENCIL_ATTACHMENT, GL_TEXTURE_1D, *object, 0,
                );
            }
        }

        WINED3D_GL_RES_TYPE_TEX_2D => {
            (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(1, object);
            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                wined3d_gl_type_to_enum(d3d_type), *object,
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                wined3d_gl_type_to_enum(d3d_type), 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                wined3d_gl_type_to_enum(d3d_type), GL_TEXTURE_MIN_FILTER, (GL_NEAREST as i32),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                wined3d_gl_type_to_enum(d3d_type), GL_TEXTURE_MAG_FILTER, (GL_NEAREST as i32),
            );

            (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
                GL_FRAMEBUFFER, attach_type, wined3d_gl_type_to_enum(d3d_type), *object, 0,
            );
            if (*format_gl).f.stencil_size != 0 {
                (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
                    GL_FRAMEBUFFER, GL_STENCIL_ATTACHMENT, wined3d_gl_type_to_enum(d3d_type), *object,
                    0,
                );
            }
        }

        WINED3D_GL_RES_TYPE_TEX_3D => {
            (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(1, object);
            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(GL_TEXTURE_3D, *object);
            ((*gl_info).gl_ops.ext.p_glTexImage3D.unwrap_unchecked())(
                GL_TEXTURE_3D, 0, internal as GLint, 16,
                16, 16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_3D, GL_TEXTURE_MIN_FILTER, (GL_NEAREST as i32),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_3D, GL_TEXTURE_MAG_FILTER, (GL_NEAREST as i32),
            );

            (*gl_info).fbo_ops.glFramebufferTexture3D.unwrap_unchecked()(
                GL_FRAMEBUFFER, attach_type, GL_TEXTURE_3D, *object, 0,
                0,
            );
            if (*format_gl).f.stencil_size != 0 {
                (*gl_info).fbo_ops.glFramebufferTexture3D.unwrap_unchecked()(
                    GL_FRAMEBUFFER, GL_STENCIL_ATTACHMENT, GL_TEXTURE_3D, *object,
                    0, 0,
                );
            }
        }

        WINED3D_GL_RES_TYPE_TEX_CUBE => {
            (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(1, object);
            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(GL_TEXTURE_CUBE_MAP_ARB, *object);
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_POSITIVE_X_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_NEGATIVE_X_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_POSITIVE_Y_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_NEGATIVE_Y_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_POSITIVE_Z_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_NEGATIVE_Z_ARB, 0, internal as GLint, 16,
                16, 0, format, type_0,
                ::core::ptr::null::<::core::ffi::c_void>(),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_ARB, GL_TEXTURE_MIN_FILTER, (GL_NEAREST as i32),
            );
            (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
                GL_TEXTURE_CUBE_MAP_ARB, GL_TEXTURE_MAG_FILTER, (GL_NEAREST as i32),
            );

            (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
                GL_FRAMEBUFFER, attach_type, GL_TEXTURE_CUBE_MAP_POSITIVE_X_ARB, *object, 0,
            );
            if (*format_gl).f.stencil_size != 0 {
                (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
                    GL_FRAMEBUFFER, GL_STENCIL_ATTACHMENT, GL_TEXTURE_CUBE_MAP_POSITIVE_X_ARB, *object,
                    0,
                );
            }
        }

        WINED3D_GL_RES_TYPE_RB => {
            (*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked()(1, object);
            (*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked()(GL_RENDERBUFFER, *object);
            (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
                GL_RENDERBUFFER, internal, 16, 16,
            );
            (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                GL_FRAMEBUFFER, attach_type, GL_RENDERBUFFER, *object,
            );
            if (*format_gl).f.stencil_size != 0 {
                (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                    GL_FRAMEBUFFER, GL_STENCIL_ATTACHMENT, GL_RENDERBUFFER, *object,
                );
            }
        }

        WINED3D_GL_RES_TYPE_BUFFER | WINED3D_GL_RES_TYPE_COUNT => {}
        _ => {}
    }

    /* Ideally we'd skip all formats already known not to work on textures
     * by checking for WINED3D_FORMAT_CAP_TEXTURE here. However, we want to
     * know if we can attach WINED3DFMT_P8_UINT textures to FBOs, and this
     * format never has WINED3D_FORMAT_CAP_TEXTURE set. Instead, swallow GL
     * errors generated by invalid formats. */
    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}
}

unsafe extern "C" fn draw_test_quad(
    mut ctx: *mut wined3d_caps_gl_ctx,
    mut geometry: *const wined3d_vec3,
    mut color: *const wined3d_color,
) {
    let mut gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    static mut default_geometry: [wined3d_vec3; 4] = [
        wined3d_vec3 { x: -1.0f32, y: -1.0f32, z: 0.0f32 },
        wined3d_vec3 { x: 1.0f32, y: -1.0f32, z: 0.0f32 },
        wined3d_vec3 { x: -1.0f32, y: 1.0f32, z: 0.0f32 },
        wined3d_vec3 { x: 1.0f32, y: 1.0f32, z: 0.0f32 },
    ];
    static mut vs_core_header: [u8; 63] =
        *b"#version 150\nin vec4 pos;\nin vec4 color;\nout vec4 out_color;\n\n\0";
    static mut vs_legacy_header: [u8; 81] =
        *b"#version 120\nattribute vec4 pos;\nattribute vec4 color;\nvarying vec4 out_color;\n\n\0";
    static mut vs_body: [u8; 63] =
        *b"void main()\n{\n    gl_Position = pos;\n    out_color = color;\n}\n\0";
    static mut fs_core: [u8; 107] =
        *b"#version 150\nin vec4 out_color;\nout vec4 fragment_color;\n\nvoid main()\n{\n    fragment_color = out_color;\n}\n\0";
    static mut fs_legacy: [u8; 87] =
        *b"#version 120\nvarying vec4 out_color;\n\nvoid main()\n{\n    gl_FragData[0] = out_color;\n}\n\0";
    let mut source: [*const ::core::ffi::c_char; 2] =
        [::core::ptr::null::<::core::ffi::c_char>(); 2];
    let mut vs_id: GLuint = 0;
    let mut fs_id: GLuint = 0;
    let mut i: ::core::ffi::c_uint = 0;

    if geometry.is_null() {
        geometry = &raw const default_geometry as *const wined3d_vec3;
    }

    if (*gl_info).supported[ARB_VERTEX_BUFFER_OBJECT as usize] == 0
        || (*gl_info).supported[ARB_VERTEX_SHADER as usize] == 0
        || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] == 0
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_LIGHTING as GLenum);
        (*gl_info)
            .gl_ops
            .gl
            .p_glMatrixMode
            .unwrap_unchecked()(GL_MODELVIEW as GLenum);
        (*gl_info)
            .gl_ops
            .gl
            .p_glLoadIdentity
            .unwrap_unchecked()();
        (*gl_info)
            .gl_ops
            .gl
            .p_glMatrixMode
            .unwrap_unchecked()(GL_PROJECTION as GLenum);
        (*gl_info)
            .gl_ops
            .gl
            .p_glLoadIdentity
            .unwrap_unchecked()();

        (*gl_info)
            .gl_ops
            .gl
            .p_glBegin
            .unwrap_unchecked()(GL_TRIANGLE_STRIP as GLenum);
        (*gl_info)
            .gl_ops
            .gl
            .p_glColor4f
            .unwrap_unchecked()(
            (*color).r as GLfloat,
            (*color).g as GLfloat,
            (*color).b as GLfloat,
            (*color).a as GLfloat,
        );
        i = 0 as ::core::ffi::c_uint;
        while i < 4 as ::core::ffi::c_uint {
            (*gl_info)
                .gl_ops
                .gl
                .p_glVertex3fv
                .unwrap_unchecked()(
                &raw const (*geometry.offset(i as isize)).x
            );
            i = i.wrapping_add(1);
        }
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnd
            .unwrap_unchecked()();
        checkGLcall!(gl_info, b"draw quad\0".as_ptr() as *const ::core::ffi::c_char);
        return;
    }

    if (*ctx).test_vbo == 0 {
        ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(1, &raw mut (*ctx).test_vbo);
    }
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ARRAY_BUFFER, (*ctx).test_vbo);
    ((*gl_info).gl_ops.ext.p_glBufferData.unwrap_unchecked())(
        GL_ARRAY_BUFFER,
        (core::mem::size_of::<wined3d_vec3>() * 4) as GLsizeiptr,
        geometry as *const ::core::ffi::c_void,
        GL_STREAM_DRAW,
    );
    ((*gl_info).gl_ops.ext.p_glVertexAttribPointer.unwrap_unchecked())(
        0 as GLuint,
        3 as GLint,
        GL_FLOAT as GLenum,
        FALSE as GLboolean,
        0 as GLsizei,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    ((*gl_info).gl_ops.ext.p_glVertexAttrib4f.unwrap_unchecked())(
        1 as GLuint,
        (*color).r as GLfloat,
        (*color).g as GLfloat,
        (*color).b as GLfloat,
        (*color).a as GLfloat,
    );
    ((*gl_info).gl_ops.ext.p_glEnableVertexAttribArray.unwrap_unchecked())(0 as GLuint);
    ((*gl_info).gl_ops.ext.p_glDisableVertexAttribArray.unwrap_unchecked())(1 as GLuint);

    if (*ctx).test_program_id == 0 {
        let mut use_glsl_150: BOOL = ((*gl_info).glsl_version
            >= MAKEDWORD_VERSION!(1, 50))
            as BOOL;

        (*ctx).test_program_id = ((*gl_info).gl_ops.ext.p_glCreateProgram.unwrap_unchecked())();

        vs_id = ((*gl_info).gl_ops.ext.p_glCreateShader.unwrap_unchecked())(GL_VERTEX_SHADER as GLenum);
        source[0 as usize] = if use_glsl_150 != 0 {
            &raw const vs_core_header as *const ::core::ffi::c_char
        } else {
            &raw const vs_legacy_header as *const ::core::ffi::c_char
        };
        source[1 as usize] =
            &raw const vs_body as *const ::core::ffi::c_char;
        ((*gl_info).gl_ops.ext.p_glShaderSource.unwrap_unchecked())(
            vs_id,
            2 as GLsizei,
            &raw mut source as *mut *const ::core::ffi::c_char as *const *const GLchar,
            ::core::ptr::null::<GLint>(),
        );
        ((*gl_info).gl_ops.ext.p_glAttachShader.unwrap_unchecked())((*ctx).test_program_id, vs_id);
        ((*gl_info).gl_ops.ext.p_glDeleteShader.unwrap_unchecked())(vs_id);

        fs_id = ((*gl_info).gl_ops.ext.p_glCreateShader.unwrap_unchecked())(GL_FRAGMENT_SHADER as GLenum);
        source[0 as usize] = if use_glsl_150 != 0 {
            &raw const fs_core as *const ::core::ffi::c_char
        } else {
            &raw const fs_legacy as *const ::core::ffi::c_char
        };
        ((*gl_info).gl_ops.ext.p_glShaderSource.unwrap_unchecked())(
            fs_id,
            1 as GLsizei,
            &raw mut source as *mut *const ::core::ffi::c_char as *const *const GLchar,
            ::core::ptr::null::<GLint>(),
        );
        ((*gl_info).gl_ops.ext.p_glAttachShader.unwrap_unchecked())((*ctx).test_program_id, fs_id);
        ((*gl_info).gl_ops.ext.p_glDeleteShader.unwrap_unchecked())(fs_id);

        ((*gl_info).gl_ops.ext.p_glBindAttribLocation.unwrap_unchecked())(
            (*ctx).test_program_id,
            0 as GLuint,
            b"pos\0".as_ptr() as *const GLchar,
        );
        ((*gl_info).gl_ops.ext.p_glBindAttribLocation.unwrap_unchecked())(
            (*ctx).test_program_id,
            1 as GLuint,
            b"color\0".as_ptr() as *const GLchar,
        );

        if use_glsl_150 != 0 {
            ((*gl_info).gl_ops.ext.p_glBindFragDataLocation.unwrap_unchecked())(
                (*ctx).test_program_id,
                0 as GLuint,
                b"fragment_color\0".as_ptr() as *const GLchar,
            );
        }

        ((*gl_info).gl_ops.ext.p_glCompileShader.unwrap_unchecked())(vs_id);
        print_glsl_info_log(gl_info, vs_id, (FALSE) as BOOL);
        ((*gl_info).gl_ops.ext.p_glCompileShader.unwrap_unchecked())(fs_id);
        print_glsl_info_log(gl_info, fs_id, (FALSE) as BOOL);
        ((*gl_info).gl_ops.ext.p_glLinkProgram.unwrap_unchecked())((*ctx).test_program_id);
        shader_glsl_validate_link(gl_info, (*ctx).test_program_id);
    }
    ((*gl_info).gl_ops.ext.p_glUseProgram.unwrap_unchecked())((*ctx).test_program_id);

    (*gl_info)
        .gl_ops
        .gl
        .p_glDrawArrays
        .unwrap_unchecked()(
        GL_TRIANGLE_STRIP as GLenum,
        0 as GLint,
        4 as GLsizei,
    );

    ((*gl_info).gl_ops.ext.p_glUseProgram.unwrap_unchecked())(0 as GLuint);
    ((*gl_info).gl_ops.ext.p_glDisableVertexAttribArray.unwrap_unchecked())(0 as GLuint);
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
        GL_ARRAY_BUFFER as GLenum,
        0 as GLuint,
    );
    checkGLcall!(gl_info, b"draw quad\0".as_ptr() as *const ::core::ffi::c_char);
}

unsafe extern "C" fn check_fbo_compat(
    ctx: *mut wined3d_caps_gl_ctx,
    format: *mut wined3d_format_gl,
) {
    static black: wined3d_color = wined3d_color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    static half_transparent_red: wined3d_color = wined3d_color { r: 1.0, g: 0.0, b: 0.0, a: 0.5 };
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    let mut status: GLenum = 0;
    let rt_internal: GLenum = (*format).rt_internal as GLenum;
    let mut object: GLuint = 0;
    let mut color_rb: GLuint = 0;
    let mut type_: wined3d_gl_resource_type = 0;
    let mut fallback_fmt_used: BOOL = (FALSE as i32);
    let mut regular_fmt_used: BOOL = (FALSE as i32);

    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_BLEND);

    while (type_ as usize) < ARRAY_SIZE!((*format).f.caps) as usize {
        'body: {
            let mut type_string: *const core::ffi::c_char =
                b"color\0".as_ptr() as *const core::ffi::c_char;

            if type_ == WINED3D_GL_RES_TYPE_BUFFER {
                break 'body;
            }

            create_and_bind_fbo_attachment(
                gl_info,
                format,
                type_,
                &raw mut object,
                (*format).internal as GLenum,
            );

            if ((*format).f.caps[type_ as usize] & WINED3D_FORMAT_CAP_DEPTH_STENCIL) != 0 {
                (*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked()(
                    1,
                    &raw mut color_rb,
                );
                (*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked()(
                    GL_RENDERBUFFER,
                    color_rb,
                );
                if type_ == WINED3D_GL_RES_TYPE_TEX_1D {
                    (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
                        GL_RENDERBUFFER,
                        GL_RGBA8,
                        16,
                        1,
                    );
                } else {
                    (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
                        GL_RENDERBUFFER,
                        GL_RGBA8,
                        16,
                        16,
                    );
                }
                (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                    GL_FRAMEBUFFER,
                    GL_COLOR_ATTACHMENT0,
                    GL_RENDERBUFFER,
                    color_rb,
                );
                checkGLcall!(gl_info, b"Create and attach color rb attachment\0".as_ptr() as *const core::ffi::c_char);
                type_string = b"depth / stencil\0".as_ptr() as *const core::ffi::c_char;
            }

            status = (*gl_info).fbo_ops.glCheckFramebufferStatus.unwrap_unchecked()(
                GL_FRAMEBUFFER,
            );
            checkGLcall!(gl_info, b"Framebuffer format check\0".as_ptr() as *const core::ffi::c_char);

            if status == GL_FRAMEBUFFER_COMPLETE {
                (*format).f.caps[type_ as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE;
                TRACE!(
                    "Format %s is supported as FBO %s attachment, type %u.\n",
                    debug_d3dformat((*format).f.id),
                    type_string,
                    type_
                );
                (*format).rt_internal = (*format).internal;
                regular_fmt_used = (TRUE as i32);
            } else {
                if rt_internal == 0 {
                    if ((*format).f.caps[type_ as usize] & (WINED3D_FORMAT_CAP_RENDERTARGET | WINED3D_FORMAT_CAP_DEPTH_STENCIL)) != 0 {
                        WARN!(
                            "Format %s with rendertarget flag is not supported as FBO color attachment (type %u), and no fallback specified.\n",
                            debug_d3dformat((*format).f.id),
                            type_
                        );
                        (*format).f.caps[type_ as usize] &= !(WINED3D_FORMAT_CAP_RENDERTARGET | WINED3D_FORMAT_CAP_DEPTH_STENCIL);
                    } else {
                        TRACE!(
                            "Format %s is not supported as FBO %s attachment, type %u.\n",
                            debug_d3dformat((*format).f.id),
                            type_string,
                            type_
                        );
                    }
                    (*format).rt_internal = (*format).internal;
                } else {
                    TRACE!(
                        "Format %s is not supported as FBO %s attachment (type %u), trying rtInternal format as fallback.\n",
                        debug_d3dformat((*format).f.id),
                        type_string,
                        type_
                    );

                    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}

                    delete_fbo_attachment(gl_info, type_, object);
                    create_and_bind_fbo_attachment(
                        gl_info,
                        format,
                        type_,
                        &raw mut object,
                        (*format).rt_internal as GLenum,
                    );

                    status = (*gl_info).fbo_ops.glCheckFramebufferStatus.unwrap_unchecked()(
                        GL_FRAMEBUFFER,
                    );
                    checkGLcall!(gl_info, b"Framebuffer format check\0".as_ptr() as *const core::ffi::c_char);

                    if status == GL_FRAMEBUFFER_COMPLETE {
                        TRACE!(
                            "Format %s rtInternal format is supported as FBO %s attachment, type %u.\n",
                            debug_d3dformat((*format).f.id),
                            type_string,
                            type_
                        );
                        fallback_fmt_used = (TRUE as i32);
                    } else {
                        WARN!(
                            "Format %s rtInternal format is not supported as FBO %s attachment, type %u.\n",
                            debug_d3dformat((*format).f.id),
                            type_string,
                            type_
                        );
                        (*format).f.caps[type_ as usize] &= !(WINED3D_FORMAT_CAP_RENDERTARGET | WINED3D_FORMAT_CAP_DEPTH_STENCIL);
                    }
                }
            }

            if status == GL_FRAMEBUFFER_COMPLETE
                && (((*format).f.caps[type_ as usize] & WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING) != 0
                    || ((*gl_info).quirks & WINED3D_QUIRK_LIMITED_TEX_FILTERING) == 0)
                && ((*format).f.attrs & WINED3D_FORMAT_ATTR_INTEGER) == 0
                && (*format).f.id != WINED3DFMT_NULL
                && (*format).f.id != WINED3DFMT_P8_UINT
                && ((*format).format) as u32 != (GL_LUMINANCE as GLenum)
                && ((*format).format) as u32 != (GL_LUMINANCE_ALPHA as GLenum)
                && ((*format).f.red_size != 0 || (*format).f.alpha_size != 0)
            {
                let mut readback: [uint32_t; 16 * 16 * 16] =
                    [0; 16 * 16 * 16];
                let mut color: uint32_t = 0;
                let mut r_range: u32 = 0;
                let mut a_range: u32 = 0;
                let mut r: BYTE = 0;
                let mut a: BYTE = 0;
                let mut match_: BOOL = (TRUE as i32);
                let mut rb: GLuint = 0;

                if (*gl_info).supported[EXT_PACKED_DEPTH_STENCIL as usize] != 0 {
                    (*gl_info).fbo_ops.glGenRenderbuffers.unwrap_unchecked()(
                        1,
                        &raw mut rb,
                    );
                    (*gl_info).fbo_ops.glBindRenderbuffer.unwrap_unchecked()(
                        GL_RENDERBUFFER,
                        rb,
                    );
                    if type_ == WINED3D_GL_RES_TYPE_TEX_1D {
                        (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
                            GL_RENDERBUFFER,
                            GL_DEPTH24_STENCIL8,
                            16,
                            1,
                        );
                    } else {
                        (*gl_info).fbo_ops.glRenderbufferStorage.unwrap_unchecked()(
                            GL_RENDERBUFFER,
                            GL_DEPTH24_STENCIL8,
                            16,
                            16,
                        );
                    }
                    (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                        GL_FRAMEBUFFER,
                        GL_DEPTH_ATTACHMENT,
                        GL_RENDERBUFFER,
                        rb,
                    );
                    (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                        GL_FRAMEBUFFER,
                        GL_STENCIL_ATTACHMENT,
                        GL_RENDERBUFFER,
                        rb,
                    );
                    checkGLcall!(gl_info, b"RB attachment\0".as_ptr() as *const core::ffi::c_char);
                }

                (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_BLEND);
                (*gl_info).gl_ops.gl.p_glClearColor.unwrap_unchecked()(0.0, 0.0, 0.0, 1.0);
                (*gl_info).gl_ops.gl.p_glClear.unwrap_unchecked()(GL_COLOR_BUFFER_BIT);
                if (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() == GL_INVALID_FRAMEBUFFER_OPERATION {
                    while (*gl_info).gl_ops.gl.p_glGetError.unwrap_unchecked()() != 0 {}
                    TRACE!(
                        "Format %s doesn't support post-pixelshader blending, type %u.\n",
                        debug_d3dformat((*format).f.id),
                        type_
                    );
                    (*format).f.caps[type_ as usize] &= !WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING;
                } else {
                    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_BLEND);
                    if type_ == WINED3D_GL_RES_TYPE_TEX_1D {
                        (*gl_info).gl_ops.gl.p_glViewport.unwrap_unchecked()(
                            0,
                            0,
                            16,
                            1,
                        );
                    } else {
                        (*gl_info).gl_ops.gl.p_glViewport.unwrap_unchecked()(
                            0,
                            0,
                            16,
                            16,
                        );
                    }
                    (*gl_info).gl_ops.gl.p_glBlendFunc.unwrap_unchecked()(
                        GL_SRC_ALPHA,
                        GL_ONE_MINUS_SRC_ALPHA,
                    );

                    draw_test_quad(ctx, core::ptr::null(), core::ptr::addr_of!(black));

                    (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_BLEND);

                    draw_test_quad(ctx, core::ptr::null(), core::ptr::addr_of!(half_transparent_red));

                    (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_BLEND);

                    match type_ {
                        WINED3D_GL_RES_TYPE_TEX_1D => {
                            /* Rebinding texture to workaround a fglrx bug. */
                            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                                GL_TEXTURE_1D,
                                object,
                            );
                            (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
                                GL_TEXTURE_1D,
                                0,
                                GL_BGRA,
                                GL_UNSIGNED_INT_8_8_8_8_REV,
                                &raw mut readback as *mut uint32_t as *mut core::ffi::c_void,
                            );
                            color = readback[7 as usize];
                        }
                        WINED3D_GL_RES_TYPE_TEX_2D | WINED3D_GL_RES_TYPE_TEX_3D => {
                            /* Rebinding texture to workaround a fglrx bug. */
                            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                                wined3d_gl_type_to_enum(type_),
                                object,
                            );
                            (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
                                wined3d_gl_type_to_enum(type_),
                                0,
                                GL_BGRA,
                                GL_UNSIGNED_INT_8_8_8_8_REV,
                                &raw mut readback as *mut uint32_t as *mut core::ffi::c_void,
                            );
                            color = readback[(7 * 16 + 7) as usize];
                        }
                        WINED3D_GL_RES_TYPE_TEX_CUBE => {
                            /* Rebinding texture to workaround a fglrx bug. */
                            (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
                                GL_TEXTURE_CUBE_MAP_ARB,
                                object,
                            );
                            (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
                                GL_TEXTURE_CUBE_MAP_POSITIVE_X_ARB,
                                0,
                                GL_BGRA,
                                GL_UNSIGNED_INT_8_8_8_8_REV,
                                &raw mut readback as *mut uint32_t as *mut core::ffi::c_void,
                            );
                            color = readback[(7 * 16 + 7) as usize];
                        }
                        WINED3D_GL_RES_TYPE_RB => {
                            (*gl_info).gl_ops.gl.p_glReadPixels.unwrap_unchecked()(
                                0,
                                0,
                                16,
                                16,
                                GL_BGRA,
                                GL_UNSIGNED_INT_8_8_8_8_REV,
                                &raw mut readback as *mut uint32_t as *mut core::ffi::c_void,
                            );
                            color = readback[(7 * 16 + 7) as usize];
                        }
                        WINED3D_GL_RES_TYPE_BUFFER | WINED3D_GL_RES_TYPE_COUNT => {
                            color = 0;
                        }
                        _ => {}
                    }
                    checkGLcall!(gl_info, b"Post-pixelshader blending check\0".as_ptr() as *const core::ffi::c_char);

                    a = (color >> 24) as BYTE;
                    r = ((color & 0x00ff0000) >> 16) as BYTE;

                    r_range = if (*format).f.red_size < 8 {
                        1 << (8 - (*format).f.red_size)
                    } else {
                        1
                    };
                    a_range = if (*format).f.alpha_size < 8 {
                        1 << (8 - (*format).f.alpha_size)
                    } else {
                        1
                    };

                    if (*format).f.red_size != 0
                        && ((r as u32) < (0x7f as u32).wrapping_sub(r_range)
                            || (r as u32) > (0x7f as u32).wrapping_add(r_range))
                    {
                        match_ = (FALSE as i32);
                    } else if (*format).f.alpha_size > 1
                        && ((a as u32) < (0xbf as u32).wrapping_sub(a_range)
                            || (a as u32) > (0xbf as u32).wrapping_add(a_range))
                    {
                        match_ = (FALSE as i32);
                    }
                    if match_ == 0 {
                        TRACE!(
                            "Format %s doesn't support post-pixelshader blending, type %u.\n",
                            debug_d3dformat((*format).f.id),
                            type_
                        );
                        TRACE!("Color output: %#x\n", color);
                        (*format).f.caps[type_ as usize] &= !WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING;
                    } else {
                        TRACE!(
                            "Format %s supports post-pixelshader blending, type %u.\n",
                            debug_d3dformat((*format).f.id),
                            type_
                        );
                        TRACE!("Color output: %#x\n", color);
                        (*format).f.caps[type_ as usize] |= WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING;
                    }
                }

                if (*gl_info).supported[EXT_PACKED_DEPTH_STENCIL as usize] != 0 {
                    (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                        GL_FRAMEBUFFER,
                        GL_DEPTH_ATTACHMENT,
                        GL_RENDERBUFFER,
                        0,
                    );
                    (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                        GL_FRAMEBUFFER,
                        GL_STENCIL_ATTACHMENT,
                        GL_RENDERBUFFER,
                        0,
                    );
                    (*gl_info).fbo_ops.glDeleteRenderbuffers.unwrap_unchecked()(
                        1,
                        &raw mut rb,
                    );
                    checkGLcall!(gl_info, b"RB cleanup\0".as_ptr() as *const core::ffi::c_char);
                }
            }

            if (*format).internal != (*format).srgb_internal {
                delete_fbo_attachment(gl_info, type_, object);
                create_and_bind_fbo_attachment(
                    gl_info,
                    format,
                    type_,
                    &raw mut object,
                    (*format).srgb_internal as GLenum,
                );

                status = (*gl_info).fbo_ops.glCheckFramebufferStatus.unwrap_unchecked()(
                    GL_FRAMEBUFFER,
                );
                checkGLcall!(gl_info, b"Framebuffer format check\0".as_ptr() as *const core::ffi::c_char);

                if status == GL_FRAMEBUFFER_COMPLETE {
                    TRACE!(
                        "Format %s's sRGB format is FBO attachable, type %u.\n",
                        debug_d3dformat((*format).f.id),
                        type_
                    );
                    (*format).f.caps[type_ as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB;
                    if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0 {
                        (*format).internal = (*format).srgb_internal;
                    }
                } else {
                    WARN!(
                        "Format %s's sRGB format is not FBO attachable, type %u.\n",
                        debug_d3dformat((*format).f.id),
                        type_
                    );
                    format_clear_caps(core::ptr::addr_of_mut!((*format).f), WINED3D_FORMAT_CAP_SRGB_WRITE);
                }
            } else if status == GL_FRAMEBUFFER_COMPLETE {
                (*format).f.caps[type_ as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB;
            }

            if (*format).f.caps[type_ as usize] & WINED3D_FORMAT_CAP_DEPTH_STENCIL != 0 {
                (*gl_info).fbo_ops.glFramebufferRenderbuffer.unwrap_unchecked()(
                    GL_FRAMEBUFFER,
                    GL_COLOR_ATTACHMENT0,
                    GL_RENDERBUFFER,
                    0,
                );
                (*gl_info).fbo_ops.glDeleteRenderbuffers.unwrap_unchecked()(
                    1,
                    &raw mut color_rb,
                );
            }

            delete_fbo_attachment(gl_info, type_, object);
            checkGLcall!(gl_info, b"Framebuffer format check cleanup\0".as_ptr() as *const core::ffi::c_char);
        }
        type_ += 1;
    }

    if fallback_fmt_used != 0 && regular_fmt_used != 0 {
        FIXME!(
            "Format %s needs different render target formats for different resource types.\n",
            debug_d3dformat((*format).f.id)
        );
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_RENDERTARGET
                | WINED3D_FORMAT_CAP_DEPTH_STENCIL
                | WINED3D_FORMAT_CAP_FBO_ATTACHABLE
                | WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB
                | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING,
        );
    }
}

unsafe extern "C" fn query_format_cap(
    mut gl_info: *mut wined3d_gl_info,
    mut format: *mut wined3d_format_gl,
    mut internal: GLint,
    mut pname: GLenum,
    mut cap: ::core::ffi::c_uint,
    mut string: *const ::core::ffi::c_char,
) {
    let mut value: GLint = 0;
    let mut type_0: wined3d_gl_resource_type = 0;
    type_0 = 0;
    while (type_0 as usize) < (ARRAY_SIZE!((*format).f.caps) as usize) {
        (*gl_info)
            .gl_ops
            .ext
            .p_glGetInternalformativ
            .unwrap_unchecked()(
            wined3d_gl_type_to_enum(type_0),
            internal as GLenum,
            pname,
            1,
            core::ptr::addr_of_mut!(value),
        );
        if value == (GL_FULL_SUPPORT as GLint) {
            WINE_TRACE!("Format %s supports %s, resource type %u.\n", debug_d3dformat((*format).f.id), string, type_0 as ::core::ffi::c_uint);
            (*format).f.caps[type_0 as usize] |= cap;
        } else {
            WINE_TRACE!("Format %s doesn't support %s, resource type %u.\n", debug_d3dformat((*format).f.id), string, type_0 as ::core::ffi::c_uint);
            (*format).f.caps[type_0 as usize] &= !cap;
        }
        type_0 += 1;
    }
}

unsafe extern "C" fn init_format_fbo_compat_info(
    adapter: *const wined3d_adapter,
    ctx: *mut wined3d_caps_gl_ctx,
) {
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    let mut i: ::core::ffi::c_uint = 0;
    let mut type_0: ::core::ffi::c_uint = 0;
    let mut fbo: GLuint = 0;

    if (*gl_info).supported[ARB_INTERNALFORMAT_QUERY2 as usize] != 0 {
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < (WINED3D_FORMAT_COUNT as usize) {
            'body: {
                let format: *mut wined3d_format_gl = get_format_gl_by_idx(adapter, i as i32);
                let mut fallback_fmt_used: BOOL = (FALSE as i32);
                let mut regular_fmt_used: BOOL = (FALSE as i32);
                let mut rt_internal: GLenum = (*format).rt_internal as GLenum;
                let mut value: GLint = 0;

                if (*format).internal == 0 {
                    break 'body;
                }

                type_0 = 0 as ::core::ffi::c_uint;
                while (type_0 as usize) < ARRAY_SIZE!((*format).f.caps) as usize {
                    (*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked()(
                        wined3d_gl_type_to_enum(type_0 as wined3d_gl_resource_type),
                        (*format).internal as GLenum,
                        GL_FRAMEBUFFER_RENDERABLE as GLenum,
                        1 as GLsizei,
                        &raw mut value,
                    );
                    if value == (GL_FULL_SUPPORT as GLint) {
                        (*format).f.caps[type_0 as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE as ::core::ffi::c_uint;
                        (*format).rt_internal = (*format).internal;
                        regular_fmt_used = (TRUE as i32);

                        (*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked()(
                            wined3d_gl_type_to_enum(type_0 as wined3d_gl_resource_type),
                            (*format).internal as GLenum,
                            GL_FRAMEBUFFER_BLEND as GLenum,
                            1 as GLsizei,
                            &raw mut value,
                        );
                        if value == (GL_FULL_SUPPORT as GLint) {
                            (*format).f.caps[type_0 as usize] |= WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING as ::core::ffi::c_uint;
                        } else {
                            (*format).f.caps[type_0 as usize] &= !(WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING as ::core::ffi::c_uint);
                        }
                    } else {
                        if rt_internal == 0 {
                            if (*format).f.caps[type_0 as usize]
                                & WINED3D_FORMAT_CAP_RENDERTARGET as ::core::ffi::c_uint
                                != 0
                            {
                                WARN!(
                                    "Format %s with rendertarget flag is not supported as FBO color attachment and no fallback specified, resource type %u.\n",
                                    debug_d3dformat((*format).f.id),
                                    type_0,
                                );
                                (*format).f.caps[type_0 as usize] &= !(WINED3D_FORMAT_CAP_RENDERTARGET as ::core::ffi::c_uint);
                            } else {
                                TRACE!(
                                    "Format %s is not supported as FBO color attachment, resource type %u.\n",
                                    debug_d3dformat((*format).f.id),
                                    type_0,
                                );
                            }
                            (*format).rt_internal = (*format).internal;
                        } else {
                            (*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked()(
                                wined3d_gl_type_to_enum(type_0 as wined3d_gl_resource_type),
                                rt_internal,
                                GL_FRAMEBUFFER_RENDERABLE as GLenum,
                                1 as GLsizei,
                                &raw mut value,
                            );
                            if value == (GL_FULL_SUPPORT as GLint) {
                                fallback_fmt_used = (TRUE as i32);
                            } else {
                                WARN!(
                                    "Format %s rtInternal format is not supported as FBO color attachment, resource type %u.\n",
                                    debug_d3dformat((*format).f.id),
                                    type_0,
                                );
                                (*format).f.caps[type_0 as usize] &= !(WINED3D_FORMAT_CAP_RENDERTARGET as ::core::ffi::c_uint);
                            }
                        }
                    }

                    if (*format).internal != (*format).srgb_internal {
                        (*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked()(
                            wined3d_gl_type_to_enum(type_0 as wined3d_gl_resource_type),
                            (*format).srgb_internal as GLenum,
                            GL_FRAMEBUFFER_RENDERABLE as GLenum,
                            1 as GLsizei,
                            &raw mut value,
                        );
                        if value == (GL_FULL_SUPPORT as GLint) {
                            (*format).f.caps[type_0 as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB as ::core::ffi::c_uint;
                            if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0 {
                                (*format).internal = (*format).srgb_internal;
                            }
                        } else {
                            WARN!(
                                "Format %s's sRGB format is not FBO attachable, resource type %u.\n",
                                debug_d3dformat((*format).f.id),
                                type_0,
                            );
                            format_clear_caps(&raw mut (*format).f, WINED3D_FORMAT_CAP_SRGB_WRITE as ::core::ffi::c_uint);
                        }
                    } else if (*format).f.caps[type_0 as usize] & WINED3D_FORMAT_CAP_FBO_ATTACHABLE as ::core::ffi::c_uint != 0 {
                        (*format).f.caps[type_0 as usize] |= WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB as ::core::ffi::c_uint;
                    }
                    type_0 = type_0.wrapping_add(1);
                }

                if fallback_fmt_used != 0 && regular_fmt_used != 0 {
                    format_clear_caps(
                        &raw mut (*format).f,
                        (WINED3D_FORMAT_CAP_RENDERTARGET | WINED3D_FORMAT_CAP_FBO_ATTACHABLE
                            | WINED3D_FORMAT_CAP_FBO_ATTACHABLE_SRGB | WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING) as ::core::ffi::c_uint,
                    );
                }
            }
            i = i.wrapping_add(1);
        }
        return;
    }

    (*gl_info).fbo_ops.glGenFramebuffers.unwrap_unchecked()(
        1 as GLsizei,
        &raw mut fbo,
    );
    (*gl_info).fbo_ops.glBindFramebuffer.unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        fbo,
    );
    (*gl_info).gl_ops.gl.p_glDrawBuffer.unwrap_unchecked()(
        GL_COLOR_ATTACHMENT0 as GLenum,
    );
    (*gl_info).gl_ops.gl.p_glReadBuffer.unwrap_unchecked()(
        GL_COLOR_ATTACHMENT0 as GLenum,
    );

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < (WINED3D_FORMAT_COUNT as usize) {
        'body2: {
            let format: *mut wined3d_format_gl = get_format_gl_by_idx(adapter, i as i32);

            if (*format).internal == 0 {
                break 'body2;
            }

            if (*format).f.attrs & WINED3D_FORMAT_ATTR_COMPRESSED as ::core::ffi::c_uint != 0 {
                break 'body2;
            }

            check_fbo_compat(ctx, format);
        }
        i = i.wrapping_add(1);
    }

    (*gl_info).fbo_ops.glDeleteFramebuffers.unwrap_unchecked()(
        1 as GLsizei,
        &raw mut fbo,
    );
}

unsafe extern "C" fn lookup_gl_view_class(mut internal_format: GLenum) -> GLenum {
    #[repr(C)]
    struct ViewClass {
        internal_format: GLenum,
        view_class: GLenum,
    }
    static view_classes: [ViewClass; 70] = [
        /* 128-bit */
        ViewClass { internal_format: GL_RGBA32F as GLenum, view_class: GL_VIEW_CLASS_128_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA32UI as GLenum, view_class: GL_VIEW_CLASS_128_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA32I as GLenum, view_class: GL_VIEW_CLASS_128_BITS as GLenum },
        /* 96-bit */
        ViewClass { internal_format: GL_RGB32F as GLenum, view_class: GL_VIEW_CLASS_96_BITS as GLenum },
        ViewClass { internal_format: GL_RGB32UI as GLenum, view_class: GL_VIEW_CLASS_96_BITS as GLenum },
        ViewClass { internal_format: GL_RGB32I as GLenum, view_class: GL_VIEW_CLASS_96_BITS as GLenum },
        /* 64-bit */
        ViewClass { internal_format: GL_RGBA16F as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RG32F as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA16UI as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RG32UI as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA16I as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RG32I as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA16 as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA16_SNORM as GLenum, view_class: GL_VIEW_CLASS_64_BITS as GLenum },
        /* 48-bit */
        ViewClass { internal_format: GL_RGB16 as GLenum, view_class: GL_VIEW_CLASS_48_BITS as GLenum },
        ViewClass { internal_format: GL_RGB16_SNORM as GLenum, view_class: GL_VIEW_CLASS_48_BITS as GLenum },
        ViewClass { internal_format: GL_RGB16F as GLenum, view_class: GL_VIEW_CLASS_48_BITS as GLenum },
        ViewClass { internal_format: GL_RGB16UI as GLenum, view_class: GL_VIEW_CLASS_48_BITS as GLenum },
        ViewClass { internal_format: GL_RGB16I as GLenum, view_class: GL_VIEW_CLASS_48_BITS as GLenum },
        /* 32-bit */
        ViewClass { internal_format: GL_RG16F as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_R11F_G11F_B10F as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_R32F as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGB10_A2UI as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA8UI as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RG16UI as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_R32UI as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA8I as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RG16I as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_R32I as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGB10_A2 as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA8 as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RG16 as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGBA8_SNORM as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RG16_SNORM as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_SRGB8_ALPHA8 as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        ViewClass { internal_format: GL_RGB9_E5 as GLenum, view_class: GL_VIEW_CLASS_32_BITS as GLenum },
        /* 24-bit */
        ViewClass { internal_format: GL_RGB8 as GLenum, view_class: GL_VIEW_CLASS_24_BITS as GLenum },
        ViewClass { internal_format: GL_RGB8_SNORM as GLenum, view_class: GL_VIEW_CLASS_24_BITS as GLenum },
        ViewClass { internal_format: GL_SRGB8 as GLenum, view_class: GL_VIEW_CLASS_24_BITS as GLenum },
        ViewClass { internal_format: GL_RGB8UI as GLenum, view_class: GL_VIEW_CLASS_24_BITS as GLenum },
        ViewClass { internal_format: GL_RGB8I as GLenum, view_class: GL_VIEW_CLASS_24_BITS as GLenum },
        /* 16-bit */
        ViewClass { internal_format: GL_R16F as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_RG8UI as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_R16UI as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_RG8I as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_R16I as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_RG8 as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_R16 as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_RG8_SNORM as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        ViewClass { internal_format: GL_R16_SNORM as GLenum, view_class: GL_VIEW_CLASS_16_BITS as GLenum },
        /* 8-bit */
        ViewClass { internal_format: GL_R8UI as GLenum, view_class: GL_VIEW_CLASS_8_BITS as GLenum },
        ViewClass { internal_format: GL_R8I as GLenum, view_class: GL_VIEW_CLASS_8_BITS as GLenum },
        ViewClass { internal_format: GL_R8 as GLenum, view_class: GL_VIEW_CLASS_8_BITS as GLenum },
        ViewClass { internal_format: GL_R8_SNORM as GLenum, view_class: GL_VIEW_CLASS_8_BITS as GLenum },
        /* RGTC1 */
        ViewClass { internal_format: GL_COMPRESSED_RED_RGTC1 as GLenum, view_class: GL_VIEW_CLASS_RGTC1_RED as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SIGNED_RED_RGTC1 as GLenum, view_class: GL_VIEW_CLASS_RGTC1_RED as GLenum },
        /* RGTC2 */
        ViewClass { internal_format: GL_COMPRESSED_RG_RGTC2 as GLenum, view_class: GL_VIEW_CLASS_RGTC2_RG as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SIGNED_RG_RGTC2 as GLenum, view_class: GL_VIEW_CLASS_RGTC2_RG as GLenum },
        /* BPTC unorm */
        ViewClass { internal_format: GL_COMPRESSED_RGBA_BPTC_UNORM as GLenum, view_class: GL_VIEW_CLASS_BPTC_UNORM as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SRGB_ALPHA_BPTC_UNORM as GLenum, view_class: GL_VIEW_CLASS_BPTC_UNORM as GLenum },
        /* BPTC float */
        ViewClass { internal_format: GL_COMPRESSED_RGB_BPTC_SIGNED_FLOAT as GLenum, view_class: GL_VIEW_CLASS_BPTC_FLOAT as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT as GLenum, view_class: GL_VIEW_CLASS_BPTC_FLOAT as GLenum },
        /* DXT1 RGB */
        ViewClass { internal_format: GL_COMPRESSED_RGB_S3TC_DXT1_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT1_RGB as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SRGB_S3TC_DXT1_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT1_RGB as GLenum },
        /* DXT1 RGBA */
        ViewClass { internal_format: GL_COMPRESSED_RGBA_S3TC_DXT1_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT1_RGBA as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT1_RGBA as GLenum },
        /* DXT3 */
        ViewClass { internal_format: GL_COMPRESSED_RGBA_S3TC_DXT3_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT3_RGBA as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT3_RGBA as GLenum },
        /* DXT5 */
        ViewClass { internal_format: GL_COMPRESSED_RGBA_S3TC_DXT5_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT5_RGBA as GLenum },
        ViewClass { internal_format: GL_COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT as GLenum, view_class: GL_VIEW_CLASS_S3TC_DXT5_RGBA as GLenum },
    ];

    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(view_classes) as usize {
        if view_classes[i as usize].internal_format == internal_format {
            return view_classes[i as usize].view_class;
        }
        i = i.wrapping_add(1);
    }

    return GL_NONE as GLenum;
}

unsafe extern "C" fn query_view_class(mut format: *mut wined3d_format_gl) {
    let mut internal_view_class: GLenum = 0;
    let mut gamma_view_class: GLenum = 0;
    let mut rt_view_class: GLenum = 0;
    internal_view_class = lookup_gl_view_class((*format).internal as GLenum);
    gamma_view_class = lookup_gl_view_class((*format).srgb_internal as GLenum);
    rt_view_class = lookup_gl_view_class((*format).rt_internal as GLenum);
    if internal_view_class == gamma_view_class || gamma_view_class == rt_view_class {
        (*format).view_class = internal_view_class;
        WINE_TRACE!("Format %s is member of GL view class %#x.\n", debug_d3dformat((*format).f.id), (*format).view_class);
    } else {
        (*format).view_class = GL_NONE as GLenum;
    }
}

unsafe extern "C" fn query_internal_format(
    adapter: *mut wined3d_adapter,
    format: *mut wined3d_format_gl,
    texture_info: *const wined3d_format_texture_info,
    gl_info: *mut wined3d_gl_info,
    srgb_format: BOOL,
) {
    let mut count: GLint = 0;
    let mut multisample_types: [GLint; 8] = [0; 8];
    let mut i: u32 = 0;
    let mut max_log2: u32 = 0;
    let mut target: GLenum = 0;

    if (*gl_info).supported[ARB_INTERNALFORMAT_QUERY2 as usize] != 0 {
        query_format_cap(gl_info, format, (*format).internal, GL_VERTEX_TEXTURE as GLenum,
                WINED3D_FORMAT_CAP_VTF as u32, b"vertex texture usage\0".as_ptr() as *const i8);
        query_format_cap(gl_info, format, (*format).internal, GL_FILTER as GLenum,
                WINED3D_FORMAT_CAP_FILTERING as u32, b"filtering\0".as_ptr() as *const i8);
        query_format_cap(gl_info, format, (*format).internal, GL_SHADER_IMAGE_STORE as GLenum,
                WINED3D_FORMAT_CAP_UNORDERED_ACCESS as u32, b"unordered access\0".as_ptr() as *const i8);

        if srgb_format != 0 || (*format).srgb_internal != (*format).internal {
            query_format_cap(gl_info, format, (*format).srgb_internal, GL_SRGB_READ as GLenum,
                    WINED3D_FORMAT_CAP_SRGB_READ as u32, b"sRGB read\0".as_ptr() as *const i8);
            query_format_cap(gl_info, format, (*format).srgb_internal, GL_SRGB_WRITE as GLenum,
                    WINED3D_FORMAT_CAP_SRGB_WRITE as u32, b"sRGB write\0".as_ptr() as *const i8);

            if ((*format).f.caps[WINED3D_GL_RES_TYPE_TEX_2D as usize]
                    & (WINED3D_FORMAT_CAP_SRGB_READ as u32 | WINED3D_FORMAT_CAP_SRGB_WRITE as u32)) == 0
            {
                (*format).srgb_internal = (*format).internal;
            } else if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0 {
                (*format).internal = (*format).srgb_internal;
            }
        }
    } else {
        if (*gl_info).limits.samplers[WINED3D_SHADER_TYPE_VERTEX as usize] == 0 {
            format_clear_caps(core::ptr::addr_of_mut!((*format).f), WINED3D_FORMAT_CAP_VTF as u32);
        }

        if ((*gl_info).quirks & WINED3D_QUIRK_LIMITED_TEX_FILTERING as u32) == 0 {
            format_set_caps(core::ptr::addr_of_mut!((*format).f), WINED3D_FORMAT_CAP_FILTERING as u32);
        } else if (*format).f.id as u32 != WINED3DFMT_R32G32B32A32_FLOAT as u32
            && (*format).f.id as u32 != WINED3DFMT_R32_FLOAT as u32
        {
            format_clear_caps(core::ptr::addr_of_mut!((*format).f), WINED3D_FORMAT_CAP_VTF as u32);
        }

        if srgb_format != 0 || (*format).srgb_internal != (*format).internal {
            /* Filter sRGB capabilities if EXT_texture_sRGB is not supported. */
            if (*gl_info).supported[EXT_TEXTURE_SRGB as usize] == 0 {
                (*format).srgb_internal = (*format).internal;
                format_clear_caps(core::ptr::addr_of_mut!((*format).f),
                        WINED3D_FORMAT_CAP_SRGB_READ as u32 | WINED3D_FORMAT_CAP_SRGB_WRITE as u32);
            } else if (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0 {
                (*format).internal = (*format).srgb_internal;
            }
        }
    }

    if (*gl_info).supported[ARB_DEPTH_TEXTURE as usize] == 0
        && ((*format).f.depth_size != 0 || (*format).f.stencil_size != 0)
    {
        TRACE!("Disabling texturing support for depth / stencil format %s.\n",
                debug_d3dformat((*format).f.id));
        (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_1D as usize] &= !(WINED3D_FORMAT_CAP_TEXTURE as u32);
        (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] &= !(WINED3D_FORMAT_CAP_TEXTURE as u32);
        (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &= !(WINED3D_FORMAT_CAP_TEXTURE as u32);
        (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_CUBE as usize] &= !(WINED3D_FORMAT_CAP_TEXTURE as u32);
    }

    query_view_class(format);

    if (*format).internal != 0
        && ((*format).f.caps[WINED3D_GL_RES_TYPE_RB as usize]
            & (WINED3D_FORMAT_CAP_RENDERTARGET as u32 | WINED3D_FORMAT_CAP_DEPTH_STENCIL as u32)) != 0
        && ((*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0
            || (*gl_info).supported[EXT_FRAMEBUFFER_MULTISAMPLE as usize] != 0)
    {
        if (*gl_info).supported[ARB_INTERNALFORMAT_QUERY as usize] != 0 {
            target = if (*gl_info).supported[ARB_TEXTURE_MULTISAMPLE as usize] != 0 {
                GL_TEXTURE_2D_MULTISAMPLE as GLenum
            } else {
                GL_RENDERBUFFER as GLenum
            };
            count = 0 as GLint;
            ((*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked())(target, (*format).internal as GLenum,
                    GL_NUM_SAMPLE_COUNTS as GLenum, 1 as GLsizei,
                    core::ptr::addr_of_mut!(count));
            if count > ARRAY_SIZE!(multisample_types) as GLint {
                FIXME!("Unexpectedly high number of multisample types %d.\n", count);
            }
            count = min!(count, ARRAY_SIZE!(multisample_types) as GLint);
            ((*gl_info).gl_ops.ext.p_glGetInternalformativ.unwrap_unchecked())(target, (*format).internal as GLenum,
                    GL_SAMPLES as GLenum, count as GLsizei,
                    multisample_types.as_mut_ptr());
            checkGLcall!(gl_info, "query sample counts");
            i = 0 as u32;
            while i < count as u32 {
                if !((multisample_types[i as usize] as usize)
                    > (core::mem::size_of::<DWORD>()).wrapping_mul(CHAR_BIT as usize))
                {
                    (*format).f.multisample_types |= (1 as u32)
                        << (multisample_types[i as usize] - 1 as GLint);
                }
                i = i.wrapping_add(1);
            }
        } else {
            max_log2 = wined3d_log2i(min!(
                (*gl_info).limits.samples as usize,
                (core::mem::size_of::<DWORD>()).wrapping_mul(CHAR_BIT as usize),
            ) as u32);
            i = 1 as u32;
            while i <= max_log2 {
                (*format).f.multisample_types |= (1 as u32)
                    << (((1 as u32) << i).wrapping_sub(1 as u32));
                i = i.wrapping_add(1);
            }
        }
    }
}

unsafe extern "C" fn init_format_texture_info(
    adapter: *mut wined3d_adapter,
    gl_info: *mut wined3d_gl_info,
) -> BOOL {
    let mut format: *mut wined3d_format_gl = core::ptr::null_mut();
    let mut srgb_format: *mut wined3d_format_gl = core::ptr::null_mut();
    let mut fragment_caps: fragment_caps = core::mem::zeroed();
    let mut shader_caps: shader_caps = core::mem::zeroed();
    let mut i: core::ffi::c_uint = 0;
    let mut j: core::ffi::c_uint = 0;

    ((*(*adapter).fragment_pipe).get_caps.unwrap_unchecked())(
        adapter,
        core::ptr::addr_of_mut!(fragment_caps),
    );
    ((*(*adapter).shader_backend).shader_get_caps.unwrap_unchecked())(
        adapter,
        core::ptr::addr_of_mut!(shader_caps),
    );

    i = 0 as core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(format_texture_info) as usize {
        'body: {
            format = get_format_gl_internal(adapter, format_texture_info[i as usize].id);
            if format.is_null() {
                return (FALSE) as BOOL;
            }

            if (*gl_info).supported[format_texture_info[i as usize].extension as usize] == 0 {
                break 'body;
            }

            /* ARB_texture_rg defines floating point formats, but only if
             * ARB_texture_float is also supported. */
            if (*gl_info).supported[ARB_TEXTURE_FLOAT as usize] == 0
                && ((*format).f.attrs & (WINED3D_FORMAT_ATTR_FLOAT as u32)) != 0
            {
                break 'body;
            }

            /* ARB_texture_rg defines integer formats if EXT_texture_integer is also supported. */
            if (*gl_info).supported[EXT_TEXTURE_INTEGER as usize] == 0
                && ((*format).f.attrs & (WINED3D_FORMAT_ATTR_INTEGER as u32)) != 0
            {
                break 'body;
            }

            (*format).internal = format_texture_info[i as usize].gl_internal;
            (*format).srgb_internal = format_texture_info[i as usize].gl_srgb_internal;
            (*format).rt_internal = format_texture_info[i as usize].gl_rt_internal;
            (*format).format = format_texture_info[i as usize].gl_format;
            (*format).type_ = format_texture_info[i as usize].gl_type;
            (*format).f.color_fixup = COLOR_FIXUP_IDENTITY;
            (*format).f.height_scale.numerator = 1 as UINT;
            (*format).f.height_scale.denominator = 1 as UINT;

            (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_1D as usize] |=
                format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);
            (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] |=
                format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);
            (*format).f.caps[WINED3D_GL_RES_TYPE_BUFFER as usize] |=
                format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);

            /* GL_ARB_depth_texture does not support 3D textures. It also says "cube textures are
             * problematic", but doesn't explicitly mandate that an error is generated. */
            if (*gl_info).supported[EXT_TEXTURE3D as usize] != 0
                && (format_texture_info[i as usize].caps & (WINED3D_FORMAT_CAP_DEPTH_STENCIL as u32)) == 0
            {
                (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] |=
                    format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);
            }

            if (*gl_info).supported[ARB_TEXTURE_CUBE_MAP as usize] != 0 {
                (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_CUBE as usize] |=
                    format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);
            }

            (*format).f.caps[WINED3D_GL_RES_TYPE_RB as usize] |=
                format_texture_info[i as usize].caps | (WINED3D_FORMAT_CAP_BLIT as u32);
            (*format).f.caps[WINED3D_GL_RES_TYPE_RB as usize] &= !(WINED3D_FORMAT_CAP_TEXTURE as u32);

            if (*format).srgb_internal != (*format).internal
                && ((*adapter).d3d_info.wined3d_creation_flags
                    & (WINED3D_SRGB_READ_WRITE_CONTROL as u32))
                    == 0
            {
                (*format).srgb_internal = (*format).internal;
                format_clear_caps(
                    core::ptr::addr_of_mut!((*format).f),
                    (WINED3D_FORMAT_CAP_SRGB_READ as core::ffi::c_uint)
                        | (WINED3D_FORMAT_CAP_SRGB_WRITE as core::ffi::c_uint),
                );
            }

            if (*gl_info).supported[ARB_SHADOW as usize] == 0
                && ((*format).f.caps[WINED3D_GL_RES_TYPE_TEX_2D as usize]
                    & (WINED3D_FORMAT_CAP_SHADOW as u32))
                    != 0
            {
                format_clear_caps(
                    core::ptr::addr_of_mut!((*format).f),
                    WINED3D_FORMAT_CAP_TEXTURE as core::ffi::c_uint,
                );
            }

            query_internal_format(
                adapter,
                format,
                core::ptr::addr_of!(format_texture_info[i as usize]),
                gl_info,
                (FALSE) as BOOL,
            );

            /* Texture conversion stuff */
            (*format).f.conv_byte_count = format_texture_info[i as usize].conv_byte_count as UINT;
            (*format).f.upload = format_texture_info[i as usize].upload;
            (*format).f.download = format_texture_info[i as usize].download;

            srgb_format = core::ptr::null_mut();
            j = 0 as core::ffi::c_uint;
            while (j as usize) < ARRAY_SIZE!(format_srgb_info) as usize {
                if format_srgb_info[j as usize].base_format_id as core::ffi::c_uint
                    == (*format).f.id as core::ffi::c_uint
                {
                    srgb_format = get_format_gl_internal(
                        adapter,
                        format_srgb_info[j as usize].srgb_format_id,
                    );
                    if srgb_format.is_null() {
                        return (FALSE) as BOOL;
                    }
                    break;
                }
                j = j.wrapping_add(1);
            }
            if srgb_format.is_null() {
                break 'body;
            }

            copy_format(
                adapter,
                core::ptr::addr_of_mut!((*srgb_format).f),
                core::ptr::addr_of!((*format).f),
            );

            if (*gl_info).supported[EXT_TEXTURE_SRGB as usize] != 0
                && ((*adapter).d3d_info.wined3d_creation_flags
                    & (WINED3D_SRGB_READ_WRITE_CONTROL as u32))
                    == 0
            {
                (*srgb_format).internal = format_texture_info[i as usize].gl_srgb_internal;
                (*srgb_format).srgb_internal = format_texture_info[i as usize].gl_srgb_internal;
                format_set_caps(
                    core::ptr::addr_of_mut!((*srgb_format).f),
                    (WINED3D_FORMAT_CAP_SRGB_READ as core::ffi::c_uint)
                        | (WINED3D_FORMAT_CAP_SRGB_WRITE as core::ffi::c_uint),
                );
                query_internal_format(
                    adapter,
                    srgb_format,
                    core::ptr::addr_of!(format_texture_info[i as usize]),
                    gl_info,
                    (TRUE) as BOOL,
                );
            }
        }
        i = i.wrapping_add(1);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn compare_uint(x: u32, y: u32, max_diff: u32) -> i32 {
    let diff: u32 = if x > y { x.wrapping_sub(y) } else { y.wrapping_sub(x) };

    return (diff <= max_diff) as i32;
}

unsafe extern "C" fn compare_colour(c1: DWORD, c2: DWORD, max_diff: BYTE) -> BOOL {
    ((compare_uint(c1 & 0xff, c2 & 0xff, max_diff as UINT) != 0)
        && (compare_uint((c1 >> 8) & 0xff, (c2 >> 8) & 0xff, max_diff as UINT) != 0)
        && (compare_uint((c1 >> 16) & 0xff, (c2 >> 16) & 0xff, max_diff as UINT) != 0)
        && (compare_uint((c1 >> 24) & 0xff, (c2 >> 24) & 0xff, max_diff as UINT) != 0)) as BOOL
}

unsafe extern "C" fn check_filter(
    mut gl_info: *const wined3d_gl_info,
    mut internal: GLenum,
) -> BOOL {
    static mut data: [DWORD; 2] = [
        0x00000000 as DWORD,
        0xffffffff as DWORD,
    ];
    let mut tex: GLuint = 0;
    let mut fbo: GLuint = 0;
    let mut buffer: GLuint = 0;
    let mut readback: [uint32_t; (16 * 1) as usize] =
        [0; (16 * 1) as usize];
    let mut ret: BOOL = FALSE as BOOL;

    while (*gl_info)
        .gl_ops
        .gl
        .p_glGetError
        .unwrap_unchecked()()
        != 0
    {}

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut buffer);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, buffer);
    memset(
        &raw mut readback as *mut uint32_t as *mut ::core::ffi::c_void,
        0x7e as ::core::ffi::c_int,
        ((16 * 1) as usize
            * ::core::mem::size_of::<uint32_t>()) as size_t,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGBA8 as GLint,
        16 as GLsizei,
        1 as GLsizei,
        0 as GLint,
        GL_RGBA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8 as GLenum,
        &raw mut readback as *mut uint32_t as *const ::core::ffi::c_void,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MIN_FILTER as GLenum,
        GL_NEAREST as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAG_FILTER as GLenum,
        GL_NEAREST as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_S as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_T as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_R as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        internal as GLint,
        2 as GLsizei,
        1 as GLsizei,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        &raw const data as *const DWORD as *const ::core::ffi::c_void,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MIN_FILTER as GLenum,
        GL_LINEAR as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAG_FILTER as GLenum,
        GL_LINEAR as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_S as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_T as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_WRAP_R as GLenum,
        GL_CLAMP_TO_EDGE as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum);

    (*gl_info)
        .fbo_ops
        .glGenFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER as GLenum, fbo);
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        GL_COLOR_ATTACHMENT0 as GLenum,
        GL_TEXTURE_2D as GLenum,
        buffer,
        0 as GLint,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glDrawBuffer
        .unwrap_unchecked()(GL_COLOR_ATTACHMENT0 as GLenum);

    (*gl_info)
        .gl_ops
        .gl
        .p_glViewport
        .unwrap_unchecked()(
        0 as GLint,
        0 as GLint,
        16 as GLsizei,
        1 as GLsizei,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_LIGHTING as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glMatrixMode
        .unwrap_unchecked()(GL_MODELVIEW as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glLoadIdentity
        .unwrap_unchecked()();
    (*gl_info)
        .gl_ops
        .gl
        .p_glMatrixMode
        .unwrap_unchecked()(GL_PROJECTION as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glLoadIdentity
        .unwrap_unchecked()();

    (*gl_info)
        .gl_ops
        .gl
        .p_glClearColor
        .unwrap_unchecked()(
        0 as GLfloat,
        1 as GLfloat,
        0 as GLfloat,
        0 as GLfloat,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glClear
        .unwrap_unchecked()(GL_COLOR_BUFFER_BIT as GLbitfield);

    (*gl_info)
        .gl_ops
        .gl
        .p_glBegin
        .unwrap_unchecked()(GL_TRIANGLE_STRIP as GLenum);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexCoord2f
        .unwrap_unchecked()(0.0f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex2f
        .unwrap_unchecked()(-1.0f32, -1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexCoord2f
        .unwrap_unchecked()(1.0f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex2f
        .unwrap_unchecked()(1.0f32, -1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexCoord2f
        .unwrap_unchecked()(0.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex2f
        .unwrap_unchecked()(-1.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexCoord2f
        .unwrap_unchecked()(1.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glVertex2f
        .unwrap_unchecked()(1.0f32, 1.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnd
        .unwrap_unchecked()();

    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum, buffer);
    memset(
        &raw mut readback as *mut uint32_t as *mut ::core::ffi::c_void,
        0x7f as ::core::ffi::c_int,
        ((16 * 1) as usize
            * ::core::mem::size_of::<uint32_t>()) as size_t,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glGetTexImage
        .unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        &raw mut readback as *mut uint32_t as *mut ::core::ffi::c_void,
    );
    if compare_colour(
        readback[6 as usize],
        0xffffffff as DWORD,
        5 as BYTE,
    ) != 0
        || compare_colour(
            readback[6 as usize],
            0x00000000 as DWORD,
            5 as BYTE,
        ) != 0
        || compare_colour(
            readback[9 as usize],
            0xffffffff as DWORD,
            5 as BYTE,
        ) != 0
        || compare_colour(
            readback[9 as usize],
            0x00000000 as DWORD,
            5 as BYTE,
        ) != 0
    {
        WINE_TRACE!("Read back colors 0x%08x and 0x%08x close to unfiltered color, assuming no filtering\n", readback[6 as usize], readback[9 as usize]);
        ret = FALSE as BOOL;
    } else {
        WINE_TRACE!("Read back colors are 0x%08x and 0x%08x, assuming texture is filtered\n", readback[6 as usize], readback[9 as usize]);
        ret = TRUE as BOOL;
    }

    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        0 as GLuint,
    );
    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(1 as GLsizei, &raw mut fbo);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut tex);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1 as GLsizei, &raw mut buffer);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_TEXTURE_2D as GLenum);

    if (*gl_info)
        .gl_ops
        .gl
        .p_glGetError
        .unwrap_unchecked()()
        != 0
    {
        WINE_FIXME!("Error during filtering test for format %x, returning no filtering\n", internal);
        ret = FALSE as BOOL;
    }

    return ret;
}

unsafe extern "C" fn init_format_filter_info(
    adapter: *mut wined3d_adapter,
    gl_info: *mut wined3d_gl_info,
) {
    let vendor: wined3d_pci_vendor = (*adapter).driver_info.vendor;
    let mut format: *mut wined3d_format_gl = core::ptr::null_mut::<wined3d_format_gl>();
    let mut i: core::ffi::c_uint;
    static fmts16: [wined3d_format_id; 3] = [
        WINED3DFMT_R16_FLOAT,
        WINED3DFMT_R16G16_FLOAT,
        WINED3DFMT_R16G16B16A16_FLOAT,
    ];
    let mut filtered: BOOL;

    /* This was already handled by init_format_texture_info(). */
    if (*gl_info).supported[ARB_INTERNALFORMAT_QUERY2 as usize] != 0 {
        return;
    }

    if (*gl_info).supported[WINED3D_GL_LEGACY_CONTEXT as usize] == 0 {
        if vendor == HW_VENDOR_NVIDIA && (*gl_info).supported[ARB_TEXTURE_FLOAT as usize] != 0 {
            WINE_TRACE!("Nvidia card with texture_float support: Assuming float16 blending\n");
            filtered = (TRUE as i32);
        } else if (*gl_info).limits.glsl_varyings > 44 {
            WINE_TRACE!("More than 44 GLSL varyings - assuming d3d10 card with float16 blending\n");
            filtered = (TRUE as i32);
        } else {
            WINE_TRACE!("Assuming no float16 blending\n");
            filtered = (FALSE as i32);
        }

        if filtered != 0 {
            i = 0;
            while (i as usize) < ARRAY_SIZE!(fmts16) as usize {
                format = get_format_gl_internal(adapter, fmts16[i as usize]);
                format_set_caps(
                    core::ptr::addr_of_mut!((*format).f),
                    WINED3D_FORMAT_CAP_FILTERING as core::ffi::c_uint,
                );
                i = i.wrapping_add(1);
            }
        }
        return;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(fmts16) as usize {
        format = get_format_gl_internal(adapter, fmts16[i as usize]);
        if (*format).internal == 0 {
            i = i.wrapping_add(1);
            continue; /* Not supported by GL */
        }

        filtered = check_filter(gl_info, (*format).internal as GLenum);
        if filtered != 0 {
            WINE_TRACE!("Format %s supports filtering.\n", debug_d3dformat((*format).f.id));
            format_set_caps(
                core::ptr::addr_of_mut!((*format).f),
                WINED3D_FORMAT_CAP_FILTERING as core::ffi::c_uint,
            );
        } else { WINE_TRACE!("Format %s does not support filtering.\n", debug_d3dformat((*format).f.id)); }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn fixup_source_from_char(c: ::core::ffi::c_char) -> fixup_channel_source {
    match c as ::core::ffi::c_int {
        49 => return CHANNEL_SOURCE_ONE,
        120 | 88 => return CHANNEL_SOURCE_X,
        121 | 89 => return CHANNEL_SOURCE_Y,
        122 | 90 => return CHANNEL_SOURCE_Z,
        119 | 87 => return CHANNEL_SOURCE_W,
        48 | _ => return CHANNEL_SOURCE_ZERO,
    }
}

unsafe extern "C" fn fixup_sign_from_char(c: ::core::ffi::c_char) -> ::core::ffi::c_uint {
    if c as ::core::ffi::c_int == 'x' as ::core::ffi::c_int
        || c as ::core::ffi::c_int == 'y' as ::core::ffi::c_int
        || c as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
        || c as ::core::ffi::c_int == 'w' as ::core::ffi::c_int
    {
        return 1;
    }
    return 0;
}

unsafe extern "C" fn create_color_fixup_desc_from_string(s: *const c_char) -> color_fixup_desc {
    let mut fixup: color_fixup_desc = core::mem::zeroed();

    if strlen(s) != 4 {
        ERR!("Invalid fixup string %s.\n", wine_dbgstr_a(s));
        return COLOR_FIXUP_IDENTITY;
    }

    fixup.set_x_sign_fixup(fixup_sign_from_char(*s.offset(0 as isize)) as u16);
    fixup.set_x_source(fixup_source_from_char(*s.offset(0 as isize)) as u16);
    fixup.set_y_sign_fixup(fixup_sign_from_char(*s.offset(1 as isize)) as u16);
    fixup.set_y_source(fixup_source_from_char(*s.offset(1 as isize)) as u16);
    fixup.set_z_sign_fixup(fixup_sign_from_char(*s.offset(2 as isize)) as u16);
    fixup.set_z_source(fixup_source_from_char(*s.offset(2 as isize)) as u16);
    fixup.set_w_sign_fixup(fixup_sign_from_char(*s.offset(3 as isize)) as u16);
    fixup.set_w_source(fixup_source_from_char(*s.offset(3 as isize)) as u16);

    return fixup;
}

unsafe extern "C" fn apply_format_fixups(
    mut adapter: *mut wined3d_adapter,
    mut gl_info: *mut wined3d_gl_info,
) {
    let mut d3d_info: *const wined3d_d3d_info = core::ptr::addr_of_mut!((*adapter).d3d_info);
    let mut format: *mut wined3d_format_gl = core::ptr::null_mut::<wined3d_format_gl>();
    let mut use_legacy_fixups: BOOL = 0;
    let mut i: ::core::ffi::c_uint = 0;
    static mut fixups: [C2RustUnnamed_59; 35] = [
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16_FLOAT,
            fixup: b"X11W\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R32_FLOAT,
            fixup: b"X11W\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16G16_UNORM,
            fixup: b"XY1W\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16G16_FLOAT,
            fixup: b"XY1W\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R32G32_FLOAT,
            fixup: b"XY1W\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8_SNORM,
            fixup: b"xy11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8_SNORM,
            fixup: b"XY11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: NV_TEXTURE_SHADER,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8_SNORM,
            fixup: b"XY11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: EXT_TEXTURE_SNORM,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16G16_SNORM,
            fixup: b"xy11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16G16_SNORM,
            fixup: b"XY11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: NV_TEXTURE_SHADER,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R16G16_SNORM,
            fixup: b"XY11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (TRUE as i32),
            extension: EXT_TEXTURE_SNORM,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            fixup: b"xyzw\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: NV_TEXTURE_SHADER,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8B8A8_SNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: EXT_TEXTURE_SNORM,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            fixup: b"xzY1\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: NV_TEXTURE_SHADER,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R5G5_SNORM_L6_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: EXT_TEXTURE_SNORM,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
            fixup: b"xyZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: NV_TEXTURE_SHADER,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_ATI1N,
            fixup: b"XXXX\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: EXT_TEXTURE_COMPRESSION_RGTC,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_ATI1N,
            fixup: b"XXXX\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_ATI2N,
            fixup: b"XW11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: ATI_TEXTURE_COMPRESSION_3DC,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_ATI2N,
            fixup: b"YX11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: EXT_TEXTURE_COMPRESSION_RGTC,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_ATI2N,
            fixup: b"YX11\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: ARB_TEXTURE_COMPRESSION_RGTC,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_A8_UNORM,
            fixup: b"000X\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_A8_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_LEGACY_CONTEXT,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L8A8_UNORM,
            fixup: b"XXXY\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L8A8_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_LEGACY_CONTEXT,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L4A4_UNORM,
            fixup: b"XXXY\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L4A4_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_LEGACY_CONTEXT,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L16_UNORM,
            fixup: b"XXX1\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L16_UNORM,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_LEGACY_CONTEXT,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_INTZ,
            fixup: b"XXXX\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_EXT_NONE,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_INTZ,
            fixup: b"XYZW\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: WINED3D_GL_LEGACY_CONTEXT,
        },
        C2RustUnnamed_59 {
            id: WINED3DFMT_L8_UNORM,
            fixup: b"XXX1\0" as *const u8 as *const ::core::ffi::c_char,
            legacy: (FALSE as i32),
            extension: ARB_TEXTURE_RG,
        },
    ];
    use_legacy_fixups = ((*d3d_info).wined3d_creation_flags
        & (WINED3D_LEGACY_UNBOUND_RESOURCE_COLOR as uint32_t)) as BOOL;
    i = (0) as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(fixups) as usize {
        if !(fixups[i as usize].legacy != 0 && use_legacy_fixups == 0) {
            if !((*gl_info).supported[fixups[i as usize].extension as usize] == 0) {
                format = get_format_gl_internal(adapter, fixups[i as usize].id);
                (*format).f.color_fixup =
                    create_color_fixup_desc_from_string(fixups[i as usize].fixup);
            }
        }
        i = i.wrapping_add(1);
    }
    if (*gl_info).supported[APPLE_YCBCR_422 as usize] == 0
        && (*gl_info).supported[APPLE_RGB_422 as usize] == 0
        && ((*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0
            || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0
                && (*gl_info).supported[ARB_VERTEX_SHADER as usize] != 0)
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_YUY2);
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_YUY2);
        format = get_format_gl_internal(adapter, WINED3DFMT_UYVY);
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_UYVY);
    } else if (*gl_info).supported[APPLE_YCBCR_422 as usize] == 0
        && (*gl_info).supported[APPLE_RGB_422 as usize] != 0
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_YUY2);
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_YUV);
        format = get_format_gl_internal(adapter, WINED3DFMT_UYVY);
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_YUV);
    } else if (*gl_info).supported[APPLE_YCBCR_422 as usize] == 0
        && ((*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] == 0
            && ((*gl_info).supported[ARB_FRAGMENT_SHADER as usize] == 0
                || (*gl_info).supported[ARB_VERTEX_SHADER as usize] == 0))
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_YUY2);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_BLIT as ::core::ffi::c_uint,
        );
        (*format).internal = (0) as GLint;
        format = get_format_gl_internal(adapter, WINED3DFMT_UYVY);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_BLIT as ::core::ffi::c_uint,
        );
        (*format).internal = (0) as GLint;
    }
    if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0
        || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0
            && (*gl_info).supported[ARB_VERTEX_SHADER as usize] != 0
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_YV12);
        (*format).f.attrs |= WINED3D_FORMAT_ATTR_HEIGHT_SCALE as ::core::ffi::c_uint;
        (*format).f.height_scale.numerator = (3) as UINT;
        (*format).f.height_scale.denominator = (2) as UINT;
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_YV12);
        format = get_format_gl_internal(adapter, WINED3DFMT_NV12);
        (*format).f.attrs |= WINED3D_FORMAT_ATTR_HEIGHT_SCALE as ::core::ffi::c_uint;
        (*format).f.height_scale.numerator = (3) as UINT;
        (*format).f.height_scale.denominator = (2) as UINT;
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_NV12);
    } else {
        format = get_format_gl_internal(adapter, WINED3DFMT_YV12);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_BLIT as ::core::ffi::c_uint,
        );
        (*format).internal = (0) as GLint;
        format = get_format_gl_internal(adapter, WINED3DFMT_NV12);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_BLIT as ::core::ffi::c_uint,
        );
        (*format).internal = (0) as GLint;
    }
    if (*gl_info).supported[ARB_FRAGMENT_PROGRAM as usize] != 0
        || (*gl_info).supported[ARB_FRAGMENT_SHADER as usize] != 0
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_P8_UINT);
        (*format).f.color_fixup = create_complex_fixup_desc(COMPLEX_FIXUP_P8);
    }
    if (*gl_info).supported[ARB_HALF_FLOAT_PIXEL as usize] == 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_R16_FLOAT);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
        format = get_format_gl_internal(adapter, WINED3DFMT_R16G16_FLOAT);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
        format = get_format_gl_internal(adapter, WINED3DFMT_R16G16B16A16_FLOAT);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
    }
    if (*gl_info).quirks & WINED3D_QUIRK_BROKEN_RGBA16 as uint32_t != 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_R16G16B16A16_UNORM);
        format_clear_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
    }
    /* ATI instancing hack: Although ATI cards do not support Shader Model
     * 3.0, they support instancing. To query if the card supports instancing
     * CheckDeviceFormat() with the special format MAKEFOURCC('I','N','S','T')
     * is used. Should an application check for this, provide a proper return
     * value. We can do instancing with all shader versions, but we need
     * vertex shaders.
     *
     * Additionally applications have to set the D3DRS_POINTSIZE render state
     * to MAKEFOURCC('I','N','S','T') once to enable instancing. Wined3d
     * doesn't need that and just ignores it.
     *
     * With Shader Model 3.0 capable cards Instancing 'just works' in Windows. */
    /* FIXME: This should just check the shader backend caps. */
    if (*gl_info).supported[ARB_VERTEX_PROGRAM as usize] != 0
        || (*gl_info).supported[ARB_VERTEX_SHADER as usize] != 0
    {
        format = get_format_gl_internal(adapter, WINED3DFMT_INST);
        format_set_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
    }
    /* Depth bound test. To query if the card supports it CheckDeviceFormat()
     * with the special format MAKEFOURCC('N','V','D','B') is used. It is
     * enabled by setting D3DRS_ADAPTIVETESS_X render state to
     * MAKEFOURCC('N','V','D','B') and then controlled by setting
     * D3DRS_ADAPTIVETESS_Z (zMin) and D3DRS_ADAPTIVETESS_W (zMax) to test
     * value. */
    if (*gl_info).supported[EXT_DEPTH_BOUNDS_TEST as usize] != 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_NVDB);
        format_set_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
    }
    if (*gl_info).supported[ARB_MULTISAMPLE as usize] != 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_ATOC);
        format_set_caps(
            core::ptr::addr_of_mut!((*format).f),
            WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
        );
    }
    /* RESZ aka AMD DX9-level hack for multisampled depth buffer resolve. You query for RESZ
     * support by checking for availability of MAKEFOURCC('R','E','S','Z') surfaces with
     * RENDERTARGET usage. */
    if (*gl_info).supported[ARB_FRAMEBUFFER_OBJECT as usize] != 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_RESZ);
        format_set_caps(
            core::ptr::addr_of_mut!((*format).f),
            (WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_RENDERTARGET) as ::core::ffi::c_uint,
        );
    }
    i = (0) as ::core::ffi::c_uint;
    while (i as usize) < (WINED3D_FORMAT_COUNT as usize) {
        format = get_format_gl_by_idx(adapter, i as ::core::ffi::c_int);
        if !((*format).f.caps[WINED3D_GL_RES_TYPE_TEX_2D as usize]
            & WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint
            == 0)
        {
            if !(is_identity_fixup((*format).f.color_fixup) != 0) {
                WINE_TRACE!("Checking support for fixup:\n");
                dump_color_fixup_desc((*format).f.color_fixup);
                if (*(*adapter).shader_backend)
                    .shader_color_fixup_supported
                    .unwrap_unchecked()((*format).f.color_fixup)
                    == 0
                    || (*(*adapter).fragment_pipe)
                        .color_fixup_supported
                        .unwrap_unchecked()(
                        (*format).f.color_fixup
                    ) == 0
                {
                    WINE_TRACE!("[FAILED]\n");
                    format_clear_caps(
                        core::ptr::addr_of_mut!((*format).f),
                        WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint,
                    );
                } else { WINE_TRACE!("[OK]\n"); }
            }
        }
        i = i.wrapping_add(1);
    }
    /* These formats are not supported for 3D textures. See also
     * WINED3D_FORMAT_CAP_DECOMPRESS. */
    format = get_format_gl_internal(adapter, WINED3DFMT_ATI1N);
    (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &=
        !(WINED3D_FORMAT_CAP_TEXTURE) as ::core::ffi::c_uint;
    format = get_format_gl_internal(adapter, WINED3DFMT_ATI2N);
    (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &=
        !(WINED3D_FORMAT_CAP_TEXTURE) as ::core::ffi::c_uint;
    format = get_format_gl_internal(adapter, WINED3DFMT_BC4_SNORM);
    (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &=
        !(WINED3D_FORMAT_CAP_TEXTURE) as ::core::ffi::c_uint;
    format = get_format_gl_internal(adapter, WINED3DFMT_BC5_UNORM);
    (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &=
        !(WINED3D_FORMAT_CAP_TEXTURE) as ::core::ffi::c_uint;
    format = get_format_gl_internal(adapter, WINED3DFMT_BC5_SNORM);
    (*format).f.caps[WINED3D_GL_RES_TYPE_TEX_3D as usize] &=
        !(WINED3D_FORMAT_CAP_TEXTURE) as ::core::ffi::c_uint;
}

unsafe extern "C" fn init_format_vertex_info(
    adapter: *const wined3d_adapter,
    gl_info: *mut wined3d_gl_info,
) -> BOOL {
    let mut format: *mut wined3d_format_gl = core::ptr::null_mut::<wined3d_format_gl>();
    let mut i: ::core::ffi::c_uint = 0;

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(format_vertex_info) as usize {
        format = get_format_gl_internal(adapter, format_vertex_info[i as usize].id);
        if format.is_null() {
            return (FALSE) as BOOL;
        }
        if !((*gl_info).supported[format_vertex_info[i as usize].extension as usize] == 0) {
            (*format).f.emit_idx = format_vertex_info[i as usize].emit_idx;
            (*format).vtx_type = format_vertex_info[i as usize].gl_vtx_type;
            (*format).vtx_format = (*format).f.component_count as GLint;
            (*format).f.caps[WINED3D_GL_RES_TYPE_BUFFER as usize] |=
                WINED3D_FORMAT_CAP_VERTEX_ATTRIBUTE as ::core::ffi::c_uint;
        }
        i = i.wrapping_add(1);
    }

    if (*gl_info).supported[ARB_VERTEX_ARRAY_BGRA as usize] != 0 {
        format = get_format_gl_internal(adapter, WINED3DFMT_B8G8R8A8_UNORM);
        (*format).vtx_format = (GL_BGRA as i32);
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn init_typeless_formats(adapter: *const wined3d_adapter) -> BOOL {
    let mut caps: [::core::ffi::c_uint; (WINED3D_GL_RES_TYPE_COUNT) as _] = [0; (WINED3D_GL_RES_TYPE_COUNT) as _];
    let mut attrs: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (::core::mem::size_of::<[wined3d_typed_format_info; 71]>() as usize)
            .wrapping_div(::core::mem::size_of::<wined3d_typed_format_info>() as usize)
    {
        let mut format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        let mut typeless_format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        format = get_format_internal(adapter, typed_formats[i as usize].id);
        if format.is_null() {
            return FALSE as BOOL;
        }
        typeless_format = get_format_internal(adapter, typed_formats[i as usize].typeless_id);
        if typeless_format.is_null() {
            return FALSE as BOOL;
        }

        attrs = (*typeless_format).attrs;
        memcpy(
            core::ptr::addr_of_mut!(caps) as *mut ::core::ffi::c_void,
            core::ptr::addr_of!((*typeless_format).caps) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_uint; (WINED3D_GL_RES_TYPE_COUNT) as _]>() as size_t,
        );
        copy_format(adapter, typeless_format, format);
        (*typeless_format).attrs |= attrs;
        j = 0 as ::core::ffi::c_uint;
        while (j as usize)
            < (::core::mem::size_of::<[::core::ffi::c_uint; (WINED3D_GL_RES_TYPE_COUNT) as _]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_uint>() as usize)
        {
            (*typeless_format).caps[j as usize] |= caps[j as usize];
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize)
        < (::core::mem::size_of::<[wined3d_typeless_format_depth_stencil_info; 4]>() as usize)
            .wrapping_div(
                ::core::mem::size_of::<wined3d_typeless_format_depth_stencil_info>() as usize,
            )
    {
        let mut typeless_format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        let mut typeless_ds_format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        let mut ds_format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        let mut depth_view_format: *mut wined3d_format = ::core::ptr::null_mut::<wined3d_format>();
        let mut stencil_view_format: *mut wined3d_format =
            ::core::ptr::null_mut::<wined3d_format>();
        let mut format_id: wined3d_format_id = WINED3DFMT_UNKNOWN;

        typeless_format = get_format_internal(
            adapter,
            typeless_depth_stencil_formats[i as usize].typeless_id,
        );
        if typeless_format.is_null() {
            return FALSE as BOOL;
        }
        ds_format = get_format_internal(
            adapter,
            typeless_depth_stencil_formats[i as usize].depth_stencil_id,
        );
        if ds_format.is_null() {
            return FALSE as BOOL;
        }

        typeless_ds_format = get_format_by_idx(
            adapter,
            (WINED3D_FORMAT_COUNT as ::core::ffi::c_int + i as ::core::ffi::c_int),
        );
        (*typeless_ds_format).id = typeless_depth_stencil_formats[i as usize].typeless_id;
        copy_format(adapter, typeless_ds_format, ds_format);
        (*typeless_ds_format).attrs = (*typeless_format).attrs;
        j = 0 as ::core::ffi::c_uint;
        while (j as usize)
            < (::core::mem::size_of::<[::core::ffi::c_uint; (WINED3D_GL_RES_TYPE_COUNT) as _]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_uint>() as usize)
        {
            (*typeless_ds_format).caps[j as usize] = (*typeless_format).caps[j as usize];
            (*typeless_format).caps[j as usize] &=
                !(WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint);
            j = j.wrapping_add(1);
        }

        format_id = typeless_depth_stencil_formats[i as usize].depth_view_id;
        if (format_id as ::core::ffi::c_uint != 0)
            && (typeless_depth_stencil_formats[i as usize].separate_depth_view_format != 0)
        {
            depth_view_format = get_format_internal(adapter, format_id);
            if depth_view_format.is_null() {
                return FALSE as BOOL;
            }
            copy_format(adapter, depth_view_format, ds_format);
            (*depth_view_format).red_size = (*depth_view_format).depth_size as DWORD;
            (*depth_view_format).depth_size = 0 as BYTE;
            (*depth_view_format).stencil_size = 0 as BYTE;
        }

        format_id = typeless_depth_stencil_formats[i as usize].stencil_view_id;
        if (format_id as u64 != 0) {
            stencil_view_format = get_format_internal(adapter, format_id);
            if stencil_view_format.is_null() {
                return FALSE as BOOL;
            }
            copy_format(adapter, stencil_view_format, ds_format);
            (*stencil_view_format).green_size = (*stencil_view_format).stencil_size as DWORD;
            (*stencil_view_format).depth_size = 0 as BYTE;
            (*stencil_view_format).stencil_size = 0 as BYTE;
        }
        i = i.wrapping_add(1);
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn init_format_gen_mipmap_info(
    mut adapter: *const wined3d_adapter,
    mut gl_info: *mut wined3d_gl_info,
) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    if (*gl_info).fbo_ops.glGenerateMipmap.is_none() {
        return;
    }
    i = (0) as ::core::ffi::c_uint;
    while (i as usize) < ((WINED3D_FORMAT_COUNT) as usize) {
        let mut format: *mut wined3d_format = get_format_by_idx(adapter, i as ::core::ffi::c_int);
        j = (0) as ::core::ffi::c_uint;
        while (j as usize) < ARRAY_SIZE!((*format).caps) as usize {
            if (!(*format).caps[j as usize]
                & ((WINED3D_FORMAT_CAP_RENDERTARGET) as u32 | (WINED3D_FORMAT_CAP_FILTERING) as u32))
                == 0
            {
                (*format).caps[j as usize] |= (WINED3D_FORMAT_CAP_GEN_MIPMAP) as u32;
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_caps_gl_ctx_test_viewport_subpixel_bits(ctx: *mut wined3d_caps_gl_ctx) -> BOOL {
    static mut red: wined3d_color = wined3d_color {
        r: 1.0f32,
        g: 0.0f32,
        b: 0.0f32,
        a: 1.0f32,
    };
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    static mut offset: f32 = -63.0f32 / 128.0f32;
    let mut texture: GLuint = 0;
    let mut fbo: GLuint = 0;
    let mut readback: [uint32_t; 4] = [0; 4];
    let mut i: ::core::ffi::c_uint = 0;

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(texture),
    );
    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        texture,
    );
    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        GL_TEXTURE_MAX_LEVEL as GLenum,
        0 as GLint,
    );
    (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_RGBA8 as GLint,
        ARRAY_SIZE!(readback) as GLsizei,
        1 as GLsizei,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        core::ptr::null::<::core::ffi::c_void>(),
    );
    (*gl_info).fbo_ops.glGenFramebuffers.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(fbo),
    );
    (*gl_info).fbo_ops.glBindFramebuffer.unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        fbo,
    );
    (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        GL_COLOR_ATTACHMENT0 as GLenum,
        GL_TEXTURE_2D as GLenum,
        texture,
        0 as GLint,
    );
    checkGLcall!(gl_info, b"create resources\0" as *const u8 as *const ::core::ffi::c_char);

    (*gl_info).gl_ops.gl.p_glClearColor.unwrap_unchecked()(
        1.0f32, 1.0f32, 1.0f32, 1.0f32,
    );
    (*gl_info).gl_ops.gl.p_glClear.unwrap_unchecked()(
        GL_COLOR_BUFFER_BIT as GLbitfield,
    );
    ((*gl_info).gl_ops.ext.p_glViewportIndexedf.unwrap_unchecked())(
        0 as GLuint,
        offset as GLfloat,
        offset as GLfloat,
        4.0f32,
        1.0f32,
    );
    draw_test_quad(ctx, core::ptr::null::<wined3d_vec3>(), core::ptr::addr_of!(red));
    checkGLcall!(gl_info, b"draw\0" as *const u8 as *const ::core::ffi::c_char);

    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        texture,
    );
    (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
        GL_TEXTURE_2D as GLenum,
        0 as GLint,
        GL_BGRA as GLenum,
        GL_UNSIGNED_INT_8_8_8_8_REV as GLenum,
        core::ptr::addr_of_mut!(readback) as *mut uint32_t as *mut ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, b"readback\0" as *const u8 as *const ::core::ffi::c_char);

    WINE_TRACE!("Readback colors are 0x%08x, 0x%08x, 0x%08x, 0x%08x.\n", readback[0 as ::core::ffi::c_int as usize], readback[1 as ::core::ffi::c_int as usize], readback[2 as ::core::ffi::c_int as usize], readback[3 as ::core::ffi::c_int as usize]);

    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(texture),
    );
    (*gl_info).fbo_ops.glDeleteFramebuffers.unwrap_unchecked()(
        1 as GLsizei,
        core::ptr::addr_of_mut!(fbo),
    );
    (*gl_info).fbo_ops.glBindFramebuffer.unwrap_unchecked()(
        GL_FRAMEBUFFER as GLenum,
        0 as GLuint,
    );
    checkGLcall!(gl_info, b"delete resources\0" as *const u8 as *const ::core::ffi::c_char);

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < (ARRAY_SIZE!(readback) as usize) {
        if readback[i as usize] != 0xffff0000 as uint32_t {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_caps_gl_ctx_test_filling_convention(ctx: *mut wined3d_caps_gl_ctx, offset: ::core::ffi::c_float) -> bool {
    static RED: wined3d_color = wined3d_color { r: 1.0f32, g: 0.0f32, b: 0.0f32, a: 1.0f32 };
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut clear: ::core::ffi::c_uint = 0;
    let mut draw: ::core::ffi::c_uint = 0;
    let mut texture: GLuint = 0;
    let mut fbo: GLuint = 0;
    let mut readback: [[uint32_t; 8]; 8] = [[0; 8]; 8];
    let edge_geometry: [wined3d_vec3; 4] = [
        wined3d_vec3 {
            x: (-1.0f32 + offset) / 8.0f32,
            y: (-1.0f32 + offset) / 8.0f32,
            z: 0.0f32,
        },
        wined3d_vec3 {
            x: (1.0f32 + offset) / 8.0f32,
            y: (-1.0f32 + offset) / 8.0f32,
            z: 0.0f32,
        },
        wined3d_vec3 {
            x: (-1.0f32 + offset) / 8.0f32,
            y: (1.0f32 + offset) / 8.0f32,
            z: 0.0f32,
        },
        wined3d_vec3 {
            x: (1.0f32 + offset) / 8.0f32,
            y: (1.0f32 + offset) / 8.0f32,
            z: 0.0f32,
        },
    ];

    (*gl_info).gl_ops.gl.p_glGenTextures.unwrap_unchecked()(1, core::ptr::addr_of_mut!(texture));
    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(GL_TEXTURE_2D, texture);
    (*gl_info).gl_ops.gl.p_glTexParameteri.unwrap_unchecked()(GL_TEXTURE_2D, GL_TEXTURE_MAX_LEVEL, 0);
    (*gl_info).gl_ops.gl.p_glTexImage2D.unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        (GL_RGBA8 as i32),
        8,
        8,
        0,
        GL_BGRA,
        GL_UNSIGNED_INT_8_8_8_8_REV,
        ::core::ptr::null::<::core::ffi::c_void>(),
    );
    (*gl_info).fbo_ops.glGenFramebuffers.unwrap_unchecked()(1, core::ptr::addr_of_mut!(fbo));
    (*gl_info).fbo_ops.glBindFramebuffer.unwrap_unchecked()(GL_FRAMEBUFFER, fbo);
    (*gl_info).fbo_ops.glFramebufferTexture2D.unwrap_unchecked()(
        GL_FRAMEBUFFER,
        GL_COLOR_ATTACHMENT0,
        GL_TEXTURE_2D,
        texture,
        0,
    );
    checkGLcall!(gl_info, "create resources");

    (*gl_info).gl_ops.gl.p_glViewport.unwrap_unchecked()(0, 0, 8, 8);
    (*gl_info).gl_ops.gl.p_glClearColor.unwrap_unchecked()(0.0f32, 0.0f32, 1.0f32, 1.0f32);
    (*gl_info).gl_ops.gl.p_glClear.unwrap_unchecked()(GL_COLOR_BUFFER_BIT);

    draw_test_quad(ctx, &raw const edge_geometry as *const wined3d_vec3, core::ptr::addr_of!(RED));
    checkGLcall!(gl_info, "draw");

    (*gl_info).gl_ops.gl.p_glBindTexture.unwrap_unchecked()(GL_TEXTURE_2D, texture);
    (*gl_info).gl_ops.gl.p_glGetTexImage.unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        GL_BGRA,
        GL_UNSIGNED_INT_8_8_8_8_REV,
        core::ptr::addr_of_mut!(readback) as *mut ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, "readback");

    (*gl_info).gl_ops.gl.p_glDeleteTextures.unwrap_unchecked()(1, core::ptr::addr_of_mut!(texture));
    (*gl_info).fbo_ops.glDeleteFramebuffers.unwrap_unchecked()(1, core::ptr::addr_of_mut!(fbo));
    (*gl_info).fbo_ops.glBindFramebuffer.unwrap_unchecked()(GL_FRAMEBUFFER, 0);
    checkGLcall!(gl_info, "delete resources");

    /* We expect that exactly one fragment is generated. */
    y = 0;
    while (y as usize) < ARRAY_SIZE!(readback) as usize {
        x = 0;
        while (x as usize) < ARRAY_SIZE!(readback[0]) as usize {
            if readback[y as usize][x as usize] == 0xff0000ff {
                clear = clear.wrapping_add(1);
            } else if readback[y as usize][x as usize] == 0xffff0000 {
                draw = draw.wrapping_add(1);
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }

    if clear != 63 || draw != 1 {
        FIXME!("Unexpected filling convention test result.\n");
        return FALSE != 0;
    }

    /* One pixel was drawn, check if it is the expected one */
    return readback[3 as usize][3 as usize] == 0xffff0000;
}

unsafe extern "C" fn wined3d_adapter_find_polyoffset_scale(
    ctx: *mut wined3d_caps_gl_ctx,
    format: GLenum,
) -> ::core::ffi::c_float {
    let gl_info: *const wined3d_gl_info = (*ctx).gl_info;
    static blue: wined3d_color = wined3d_color {
        r: 0.0f32,
        g: 0.0f32,
        b: 1.0f32,
        a: 1.0f32,
    };
    let mut fbo: GLuint = 0;
    let mut color: GLuint = 0;
    let mut depth: GLuint = 0;
    let mut low: ::core::ffi::c_uint = 0;
    let mut high: ::core::ffi::c_uint = 32;
    let mut cur: ::core::ffi::c_uint;
    let mut readback: [uint32_t; 256] = [0; 256];
    static geometry: [wined3d_vec3; 4] = [
        wined3d_vec3 {
            x: -1.0f32,
            y: -1.0f32,
            z: -1.0f32,
        },
        wined3d_vec3 {
            x: 1.0f32,
            y: -1.0f32,
            z: 0.0f32,
        },
        wined3d_vec3 {
            x: -1.0f32,
            y: 1.0f32,
            z: -1.0f32,
        },
        wined3d_vec3 {
            x: 1.0f32,
            y: 1.0f32,
            z: 0.0f32,
        },
    ];

    /* Most drivers want 2^23 for fixed point depth buffers, including r300g, r600g,
     * Nvidia. Use this as a fallback if the detection fails. */
    let fallback: ::core::ffi::c_uint = 23;

    (*gl_info)
        .gl_ops
        .gl
        .p_glGenTextures
        .unwrap_unchecked()(1, &raw mut color);
    (*gl_info)
        .gl_ops
        .gl
        .p_glBindTexture
        .unwrap_unchecked()(GL_TEXTURE_2D, color);
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexParameteri
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        GL_TEXTURE_MAX_LEVEL,
        0,
    );
    (*gl_info)
        .gl_ops
        .gl
        .p_glTexImage2D
        .unwrap_unchecked()(
        GL_TEXTURE_2D,
        0,
        (GL_RGBA8 as i32),
        256,
        1,
        0,
        GL_BGRA,
        GL_UNSIGNED_INT_8_8_8_8_REV,
        core::ptr::null(),
    );

    (*gl_info)
        .fbo_ops
        .glGenRenderbuffers
        .unwrap_unchecked()(1, &raw mut depth);
    (*gl_info)
        .fbo_ops
        .glBindRenderbuffer
        .unwrap_unchecked()(GL_RENDERBUFFER, depth);
    (*gl_info)
        .fbo_ops
        .glRenderbufferStorage
        .unwrap_unchecked()(
        GL_RENDERBUFFER,
        format,
        256,
        1,
    );

    (*gl_info)
        .fbo_ops
        .glGenFramebuffers
        .unwrap_unchecked()(1, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER, fbo);
    (*gl_info)
        .fbo_ops
        .glFramebufferTexture2D
        .unwrap_unchecked()(
        GL_FRAMEBUFFER,
        GL_COLOR_ATTACHMENT0,
        GL_TEXTURE_2D,
        color,
        0,
    );
    (*gl_info)
        .fbo_ops
        .glFramebufferRenderbuffer
        .unwrap_unchecked()(
        GL_FRAMEBUFFER,
        GL_DEPTH_ATTACHMENT,
        GL_RENDERBUFFER,
        depth,
    );
    checkGLcall!(gl_info, "Setup framebuffer");

    (*gl_info)
        .gl_ops
        .gl
        .p_glClearColor
        .unwrap_unchecked()(0.0f32, 0.0f32, 0.5f32, 0.0f32);
    (*gl_info)
        .gl_ops
        .gl
        .p_glClearDepth
        .unwrap_unchecked()(0.5f64);
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_DEPTH_TEST);
    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_POLYGON_OFFSET_FILL);
    (*gl_info)
        .gl_ops
        .gl
        .p_glViewport
        .unwrap_unchecked()(
        0,
        0,
        256,
        1,
    );
    checkGLcall!(gl_info, "Misc parameters");

    loop {
        if high.wrapping_sub(low) <= 1 {
            ERR!(
                "PolygonOffset scale factor detection failed, using fallback value 2^%u.\n",
                fallback
            );
            cur = fallback;
            break;
        } else {
            cur = low
                .wrapping_add(high)
                .wrapping_div(2);

            (*gl_info)
                .gl_ops
                .gl
                .p_glClear
                .unwrap_unchecked()(
                (GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT) as GLbitfield,
            );
            /* The post viewport transform Z of the geometry runs from 0.0 to 0.5. We want to push it another
             * 0.25 so that the Z buffer content (0.5) cuts the quad off at half the screen. */
            (*gl_info)
                .gl_ops
                .gl
                .p_glPolygonOffset
                .unwrap_unchecked()(
                0.0f32,
                ((1 as ::core::ffi::c_uint) << cur) as GLfloat * 0.25f32,
            );
            draw_test_quad(
                ctx,
                &raw const geometry as *const wined3d_vec3,
                &raw const blue,
            );
            checkGLcall!(gl_info, "Test draw");

            /* Rebinding texture to workaround a fglrx bug. */
            (*gl_info)
                .gl_ops
                .gl
                .p_glBindTexture
                .unwrap_unchecked()(GL_TEXTURE_2D, color);
            (*gl_info)
                .gl_ops
                .gl
                .p_glGetTexImage
                .unwrap_unchecked()(
                GL_TEXTURE_2D,
                0,
                GL_BGRA,
                GL_UNSIGNED_INT_8_8_8_8_REV,
                &raw mut readback as *mut uint32_t as *mut ::core::ffi::c_void,
            );
            checkGLcall!(gl_info, "readback");

            WINE_TRACE!("low %02u, high %02u, cur %2u, 0=0x%08x, 125=0x%08x, 131=0x%08x, 255=0x%08x\n", low, high, cur, readback[0 as ::core::ffi::c_int as usize], readback[125 as ::core::ffi::c_int as usize], readback[131 as ::core::ffi::c_int as usize], readback[255 as ::core::ffi::c_int as usize]);

            if (readback[125] & 0xff) < 0xa0
            {
                high = cur;
            } else if (readback[131] & 0xff)
                > 0xa0
            {
                low = cur;
            } else {
                WINE_TRACE!("Found scale factor 2^%u for format %x.\n", cur, format);
                break;
            }
        }
    }

    (*gl_info)
        .gl_ops
        .gl
        .p_glDeleteTextures
        .unwrap_unchecked()(1, &raw mut color);
    (*gl_info)
        .fbo_ops
        .glDeleteRenderbuffers
        .unwrap_unchecked()(1, &raw mut depth);
    (*gl_info)
        .fbo_ops
        .glDeleteFramebuffers
        .unwrap_unchecked()(1, &raw mut fbo);
    (*gl_info)
        .fbo_ops
        .glBindFramebuffer
        .unwrap_unchecked()(GL_FRAMEBUFFER, 0);
    checkGLcall!(gl_info, "Delete framebuffer");

    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_DEPTH_TEST);
    (*gl_info)
        .gl_ops
        .gl
        .p_glDisable
        .unwrap_unchecked()(GL_POLYGON_OFFSET_FILL);
    return ((1 as ::core::ffi::c_uint) << cur) as ::core::ffi::c_float;
}

unsafe extern "C" fn init_format_depth_bias_scale(
    adapter: *mut wined3d_adapter,
    ctx: *mut wined3d_caps_gl_ctx,
) {
    let d3d_info: *mut wined3d_d3d_info = core::ptr::addr_of_mut!((*adapter).d3d_info);
    let mut i: ::core::ffi::c_uint = 0;

    i = 0 as ::core::ffi::c_uint;
    while i < (WINED3D_FORMAT_COUNT as ::core::ffi::c_uint) {
        let format: *mut wined3d_format_gl =
            get_format_gl_by_idx(adapter, i as ::core::ffi::c_int);

        if (*format).f.depth_size as ::core::ffi::c_int != 0
            && (*format).f.caps[WINED3D_GL_RES_TYPE_RB as usize] as ::core::ffi::c_uint
                & (WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint)
                != 0
        {
            (*format).f.depth_bias_scale =
                wined3d_adapter_find_polyoffset_scale(ctx, (*format).internal as GLenum);

            if (*d3d_info).wined3d_creation_flags & (WINED3D_NORMALIZED_DEPTH_BIAS as u32) == 0 {
                /* The single-precision binary floating-point format has
                 * a significand precision of 24 bits.
                 */
                if (*format).f.attrs & (WINED3D_FORMAT_ATTR_FLOAT as ::core::ffi::c_uint) != 0 {
                    (*format).f.depth_bias_scale /= ((1
                        as ::core::ffi::c_uint)
                        << (24 as ::core::ffi::c_uint))
                        as ::core::ffi::c_float;
                } else {
                    (*format).f.depth_bias_scale /= ((1
                        as ::core::ffi::c_uint)
                        << ((*format).f.depth_size as ::core::ffi::c_uint))
                        as ::core::ffi::c_float;
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn wined3d_adapter_init_format_info(adapter: *mut wined3d_adapter, format_size: size_t) -> BOOL {
    let mut count: u32 = (WINED3D_FORMAT_COUNT as u32) + (ARRAY_SIZE!(typeless_depth_stencil_formats) as u32);

    (*adapter).formats = calloc((count as size_t) as _, (format_size) as _);
    if (*adapter).formats.is_null() {
        ERR!("Failed to allocate memory.\n");
        return (FALSE) as BOOL;
    }
    (*adapter).format_size = format_size;

    'fail: {
        if init_format_base_info(adapter) == 0 {
            break 'fail;
        }
        if init_format_block_info(adapter) == 0 {
            break 'fail;
        }
        if init_format_decompress_info(adapter) == 0 {
            break 'fail;
        }
        if init_srgb_formats(adapter) == 0 {
            break 'fail;
        }

        return (TRUE) as BOOL;
    }

    free((*adapter).formats);
    (*adapter).formats = core::ptr::null_mut();
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_no3d_init_format_info(adapter: *mut wined3d_adapter) -> BOOL {
    let mut format: *mut wined3d_format = core::ptr::null_mut();
    let mut i: core::ffi::c_uint;
    static blit_formats: [wined3d_format_id; 17] = [
        WINED3DFMT_B8G8R8A8_UNORM,
        WINED3DFMT_B8G8R8X8_UNORM,
        WINED3DFMT_B5G6R5_UNORM,
        WINED3DFMT_B5G5R5X1_UNORM,
        WINED3DFMT_B5G5R5A1_UNORM,
        WINED3DFMT_B4G4R4A4_UNORM,
        WINED3DFMT_B2G3R3_UNORM,
        WINED3DFMT_A8_UNORM,
        WINED3DFMT_B2G3R3A8_UNORM,
        WINED3DFMT_B4G4R4X4_UNORM,
        WINED3DFMT_R10G10B10A2_UNORM,
        WINED3DFMT_R8G8B8A8_UNORM,
        WINED3DFMT_R8G8B8X8_UNORM,
        WINED3DFMT_R16G16_UNORM,
        WINED3DFMT_B10G10R10A2_UNORM,
        WINED3DFMT_R16G16B16A16_UNORM,
        WINED3DFMT_P8_UINT,
    ];

    if wined3d_adapter_init_format_info(adapter, core::mem::size_of::<wined3d_format>() as size_t) == 0 {
        return (FALSE) as BOOL;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(blit_formats) as usize {
        format = get_format_internal(adapter, blit_formats[i as usize]);
        if format.is_null() {
            return (FALSE) as BOOL;
        }

        (*format).caps[WINED3D_GL_RES_TYPE_TEX_2D as usize] |= WINED3D_FORMAT_CAP_BLIT;
        (*format).caps[WINED3D_GL_RES_TYPE_RB as usize] |= WINED3D_FORMAT_CAP_BLIT;

        i = i.wrapping_add(1);
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_gl_init_format_info(adapter: *mut wined3d_adapter, ctx: *mut wined3d_caps_gl_ctx) -> BOOL {
    let gl_info: *mut wined3d_gl_info = core::ptr::addr_of_mut!((*wined3d_adapter_gl(adapter)).gl_info);

    'fail: {
        if wined3d_adapter_init_format_info(adapter, core::mem::size_of::<wined3d_format_gl>() as usize) == 0 {
            return (FALSE) as BOOL;
        }

        if init_format_texture_info(adapter, gl_info) == 0 {
            break 'fail;
        }
        if init_format_vertex_info(adapter, gl_info) == 0 {
            break 'fail;
        }

        apply_format_fixups(adapter, gl_info);
        init_format_fbo_compat_info(adapter, ctx);
        init_format_filter_info(adapter, gl_info);
        init_format_gen_mipmap_info(adapter, gl_info);
        init_format_depth_bias_scale(adapter, ctx);

        if init_typeless_formats(adapter) == 0 {
            break 'fail;
        }

        return (TRUE) as BOOL;
    }

    free((*adapter).formats as *mut core::ffi::c_void);
    (*adapter).formats = core::ptr::null_mut();
    return (FALSE) as BOOL;
}

unsafe extern "C" fn init_vulkan_format_info(
    format: *mut wined3d_format_vk,
    vk_info: *const wined3d_vk_info,
    vk_physical_device: VkPhysicalDevice,
) {
    struct VulkanFormatEntry {
        id: ::core::ffi::c_uint,
        vk_format: VkFormat,
        fixup: *const ::core::ffi::c_char,
    }
    let vulkan_formats: [VulkanFormatEntry; 71] = [
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32A32_FLOAT, vk_format: VK_FORMAT_R32G32B32A32_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32A32_UINT, vk_format: VK_FORMAT_R32G32B32A32_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32A32_SINT, vk_format: VK_FORMAT_R32G32B32A32_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32_FLOAT, vk_format: VK_FORMAT_R32G32B32_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32_UINT, vk_format: VK_FORMAT_R32G32B32_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32B32_SINT, vk_format: VK_FORMAT_R32G32B32_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16B16A16_FLOAT, vk_format: VK_FORMAT_R16G16B16A16_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16B16A16_UNORM, vk_format: VK_FORMAT_R16G16B16A16_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16B16A16_UINT, vk_format: VK_FORMAT_R16G16B16A16_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16B16A16_SNORM, vk_format: VK_FORMAT_R16G16B16A16_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16B16A16_SINT, vk_format: VK_FORMAT_R16G16B16A16_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32_FLOAT, vk_format: VK_FORMAT_R32G32_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32_UINT, vk_format: VK_FORMAT_R32G32_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32G32_SINT, vk_format: VK_FORMAT_R32G32_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R10G10B10A2_UNORM, vk_format: VK_FORMAT_A2B10G10R10_UNORM_PACK32, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R11G11B10_FLOAT, vk_format: VK_FORMAT_B10G11R11_UFLOAT_PACK32, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8_UNORM, vk_format: VK_FORMAT_R8G8_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8_UINT, vk_format: VK_FORMAT_R8G8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8_SNORM, vk_format: VK_FORMAT_R8G8_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8_SINT, vk_format: VK_FORMAT_R8G8_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8B8A8_UNORM, vk_format: VK_FORMAT_R8G8B8A8_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8B8A8_UNORM_SRGB, vk_format: VK_FORMAT_R8G8B8A8_SRGB, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8B8A8_UINT, vk_format: VK_FORMAT_R8G8B8A8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8B8A8_SNORM, vk_format: VK_FORMAT_R8G8B8A8_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8G8B8A8_SINT, vk_format: VK_FORMAT_R8G8B8A8_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16_FLOAT, vk_format: VK_FORMAT_R16G16_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16_UNORM, vk_format: VK_FORMAT_R16G16_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16_UINT, vk_format: VK_FORMAT_R16G16_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16_SNORM, vk_format: VK_FORMAT_R16G16_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16G16_SINT, vk_format: VK_FORMAT_R16G16_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_D32_FLOAT, vk_format: VK_FORMAT_D32_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32_FLOAT, vk_format: VK_FORMAT_R32_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32_UINT, vk_format: VK_FORMAT_R32_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32_SINT, vk_format: VK_FORMAT_R32_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16_FLOAT, vk_format: VK_FORMAT_R16_SFLOAT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_D16_UNORM, vk_format: VK_FORMAT_D16_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16_UNORM, vk_format: VK_FORMAT_R16_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16_UINT, vk_format: VK_FORMAT_R16_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16_SNORM, vk_format: VK_FORMAT_R16_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R16_SINT, vk_format: VK_FORMAT_R16_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8_UNORM, vk_format: VK_FORMAT_R8_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8_UINT, vk_format: VK_FORMAT_R8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8_SNORM, vk_format: VK_FORMAT_R8_SNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R8_SINT, vk_format: VK_FORMAT_R8_SINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_A8_UNORM, vk_format: VK_FORMAT_R8_UNORM, fixup: b"000X\0".as_ptr() as *const ::core::ffi::c_char },
        VulkanFormatEntry { id: WINED3DFMT_B8G8R8A8_UNORM, vk_format: VK_FORMAT_B8G8R8A8_UNORM, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_B8G8R8A8_UNORM_SRGB, vk_format: VK_FORMAT_B8G8R8A8_SRGB, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_B8G8R8X8_UNORM, vk_format: VK_FORMAT_B8G8R8A8_UNORM, fixup: b"XYZ1\0".as_ptr() as *const ::core::ffi::c_char },
        VulkanFormatEntry { id: WINED3DFMT_B8G8R8X8_UNORM_SRGB, vk_format: VK_FORMAT_B8G8R8A8_SRGB, fixup: b"XYZ1\0".as_ptr() as *const ::core::ffi::c_char },
        VulkanFormatEntry { id: WINED3DFMT_B5G6R5_UNORM, vk_format: VK_FORMAT_R5G6B5_UNORM_PACK16, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_B5G5R5A1_UNORM, vk_format: VK_FORMAT_A1R5G5B5_UNORM_PACK16, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_B4G4R4A4_UNORM, vk_format: VK_FORMAT_R4G4B4A4_UNORM_PACK16, fixup: b"YZWX\0".as_ptr() as *const ::core::ffi::c_char },
        VulkanFormatEntry { id: WINED3DFMT_BC1_UNORM, vk_format: VK_FORMAT_BC1_RGBA_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC1_UNORM_SRGB, vk_format: VK_FORMAT_BC1_RGBA_SRGB_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC2_UNORM, vk_format: VK_FORMAT_BC2_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC2_UNORM_SRGB, vk_format: VK_FORMAT_BC2_SRGB_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC3_UNORM, vk_format: VK_FORMAT_BC3_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC3_UNORM_SRGB, vk_format: VK_FORMAT_BC3_SRGB_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC4_UNORM, vk_format: VK_FORMAT_BC4_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC4_SNORM, vk_format: VK_FORMAT_BC4_SNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC5_UNORM, vk_format: VK_FORMAT_BC5_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC5_SNORM, vk_format: VK_FORMAT_BC5_SNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC6H_UF16, vk_format: VK_FORMAT_BC6H_UFLOAT_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC6H_SF16, vk_format: VK_FORMAT_BC6H_SFLOAT_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC7_UNORM, vk_format: VK_FORMAT_BC7_UNORM_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_BC7_UNORM_SRGB, vk_format: VK_FORMAT_BC7_SRGB_BLOCK, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R9G9B9E5_SHAREDEXP, vk_format: VK_FORMAT_E5B9G9R9_UFLOAT_PACK32, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_D32_FLOAT_S8X24_UINT, vk_format: VK_FORMAT_D32_SFLOAT_S8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_R32_FLOAT_X8X24_TYPELESS, vk_format: VK_FORMAT_D32_SFLOAT_S8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_X32_TYPELESS_G8X24_UINT, vk_format: VK_FORMAT_D32_SFLOAT_S8_UINT, fixup: ::core::ptr::null() },
        VulkanFormatEntry { id: WINED3DFMT_D24_UNORM_S8_UINT, vk_format: VK_FORMAT_D24_UNORM_S8_UINT, fixup: ::core::ptr::null() },
    ];
    let mut vk_format: VkFormat = VK_FORMAT_UNDEFINED;
    let mut image_properties: VkImageFormatProperties = core::mem::zeroed();
    let mut texture_flags: VkFormatFeatureFlags = 0;
    let mut properties: VkFormatProperties = core::mem::zeroed();
    let mut vk_usage: VkImageUsageFlags = 0;
    let mut caps: ::core::ffi::c_uint = 0;
    let mut fixup: *const ::core::ffi::c_char = ::core::ptr::null();
    let mut i: ::core::ffi::c_uint = 0;
    let mut mask: uint32_t = 0;
    let mut vr: VkResult = 0;

    i = 0;
    while (i as usize) < ARRAY_SIZE!(vulkan_formats) as usize {
        if vulkan_formats[i as usize].id == (*format).f.id as ::core::ffi::c_uint {
            vk_format = vulkan_formats[i as usize].vk_format;
            fixup = vulkan_formats[i as usize].fixup;
            break;
        }
        i = i.wrapping_add(1);
    }
    if vk_format == 0 {
        WARN!("Unsupported format %s.\n", debug_d3dformat((*format).f.id));
        return;
    }

    (*format).vk_format = vk_format;
    if !fixup.is_null() {
        (*format).f.color_fixup = create_color_fixup_desc_from_string(fixup);
    } else {
        (*format).f.color_fixup = COLOR_FIXUP_IDENTITY;
    }

    ((*vk_info).vk_ops.vkGetPhysicalDeviceFormatProperties.unwrap_unchecked())(vk_physical_device, vk_format, core::ptr::addr_of_mut!(properties));
    if vk_format == VK_FORMAT_D24_UNORM_S8_UINT {
        if (!properties.optimalTilingFeatures & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT) != 0 {
            /* AMD doesn't support VK_FORMAT_D24_UNORM_S8_UINT. */
            WARN!("Mapping VK_FORMAT_D24_UNORM_S8_UINT to VK_FORMAT_D32_SFLOAT_S8_UINT.\n");

            vk_format = VK_FORMAT_D32_SFLOAT_S8_UINT;
            (*format).vk_format = vk_format;
            ((*vk_info).vk_ops.vkGetPhysicalDeviceFormatProperties.unwrap_unchecked())(vk_physical_device, vk_format, core::ptr::addr_of_mut!(properties));
        }
    }

    if properties.bufferFeatures & VK_FORMAT_FEATURE_VERTEX_BUFFER_BIT != 0 {
        (*format).f.caps[(WINED3D_GL_RES_TYPE_BUFFER) as usize] |= WINED3D_FORMAT_CAP_VERTEX_ATTRIBUTE;
    }
    if properties.bufferFeatures & VK_FORMAT_FEATURE_UNIFORM_TEXEL_BUFFER_BIT != 0 {
        (*format).f.caps[(WINED3D_GL_RES_TYPE_BUFFER) as usize] |= WINED3D_FORMAT_CAP_TEXTURE;
    }

    caps = 0;
    texture_flags = properties.linearTilingFeatures | properties.optimalTilingFeatures;
    if texture_flags & VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_TEXTURE | WINED3D_FORMAT_CAP_VTF;
    }
    if texture_flags & VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_RENDERTARGET;
    }
    if texture_flags & VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BLEND_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING;
    }
    if texture_flags & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_DEPTH_STENCIL;
    }
    if texture_flags & VK_FORMAT_FEATURE_SAMPLED_IMAGE_FILTER_LINEAR_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_FILTERING;
    }
    if texture_flags & VK_FORMAT_FEATURE_STORAGE_IMAGE_BIT != 0 {
        caps |= WINED3D_FORMAT_CAP_UNORDERED_ACCESS;
    }
    if (texture_flags & VK_FORMAT_FEATURE_TRANSFER_SRC_BIT != 0)
        && (texture_flags & VK_FORMAT_FEATURE_TRANSFER_DST_BIT != 0)
    {
        caps |= WINED3D_FORMAT_CAP_BLIT;
    }

    if (!caps & (WINED3D_FORMAT_CAP_RENDERTARGET | WINED3D_FORMAT_CAP_FILTERING)) == 0 {
        caps |= WINED3D_FORMAT_CAP_GEN_MIPMAP;
    }

    (*format).f.caps[(WINED3D_GL_RES_TYPE_TEX_1D) as usize] |= caps;
    (*format).f.caps[(WINED3D_GL_RES_TYPE_TEX_2D) as usize] |= caps;
    (*format).f.caps[(WINED3D_GL_RES_TYPE_TEX_3D) as usize] |= caps;
    (*format).f.caps[(WINED3D_GL_RES_TYPE_TEX_CUBE) as usize] |= caps;

    vk_usage = 0;
    if texture_flags & VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT != 0 {
        vk_usage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;
    } else if texture_flags & VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT != 0 {
        vk_usage = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;
    }
    if vk_usage != 0 {
        vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceImageFormatProperties.unwrap_unchecked())(
            vk_physical_device,
            vk_format,
            VK_IMAGE_TYPE_2D,
            VK_IMAGE_TILING_OPTIMAL,
            vk_usage,
            0,
            core::ptr::addr_of_mut!(image_properties),
        );
        if (vr as ::core::ffi::c_int) < 0 {
            ERR!("Failed to get image format properties, vr %s.\n", wined3d_debug_vkresult(vr));
            return;
        }

        mask = (image_properties.sampleCounts & 0x3f) as uint32_t;
        while mask != 0 {
            i = ((1 as ::core::ffi::c_uint)
                << wined3d_bit_scan(core::ptr::addr_of_mut!(mask)))
                .wrapping_sub(1);
            (*format).f.multisample_types |= (1 as ::core::ffi::c_uint) << i;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_adapter_vk_init_format_info(adapter_vk: *mut wined3d_adapter_vk, vk_info: *const wined3d_vk_info) -> BOOL {
    let vk_physical_device: VkPhysicalDevice = (*adapter_vk).physical_device;
    let adapter: *mut wined3d_adapter = &raw mut (*adapter_vk).a;
    let mut format: *mut wined3d_format_vk = core::ptr::null_mut::<wined3d_format_vk>();
    let mut i: ::core::ffi::c_uint;

    if wined3d_adapter_init_format_info(adapter, core::mem::size_of::<wined3d_format_vk>() as size_t) == 0 {
        return (FALSE) as BOOL;
    }

    i = 0;
    while i < WINED3D_FORMAT_COUNT {
        format = wined3d_format_vk_mutable(get_format_by_idx(adapter, i as ::core::ffi::c_int));

        if (*format).f.id as u64 != 0 {
            init_vulkan_format_info(format, vk_info, vk_physical_device);
        }
        i += 1;
    }

    'fail: {
        if init_typeless_formats(adapter) == 0 {
            break 'fail;
        }
        return (TRUE) as BOOL;
    }

    free((*adapter).formats);
    (*adapter).formats = core::ptr::null_mut();
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_format(adapter: *const wined3d_adapter, format_id: wined3d_format_id, bind_flags: ::core::ffi::c_uint) -> *const wined3d_format {
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut idx: ::core::ffi::c_int = get_format_idx(format_id);
    let mut i: ::core::ffi::c_uint = 0;

    if idx == -1 {
        WINE_FIXME!("Can't find format %s (%#x) in the format lookup table.\n", debug_d3dformat(format_id), format_id as ::core::ffi::c_uint);
        return get_format_internal(adapter, WINED3DFMT_UNKNOWN);
    }

    format = get_format_by_idx(adapter, idx);

    if (bind_flags & (WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint)) != 0
        && wined3d_format_is_typeless(format) != 0
    {
        i = 0;
        while (i as usize) < ARRAY_SIZE!(typeless_depth_stencil_formats) as usize {
            if typeless_depth_stencil_formats[i as usize].typeless_id == format_id {
                return get_format_by_idx(
                    adapter,
                    (WINED3D_FORMAT_COUNT as ::core::ffi::c_int)
                        + (i as ::core::ffi::c_int),
                );
            }
            i = i.wrapping_add(1);
        }

        WINE_FIXME!("Cannot find depth/stencil typeless format %s (%#x).\n", debug_d3dformat(format_id), format_id as ::core::ffi::c_uint);
        return get_format_internal(adapter, WINED3DFMT_UNKNOWN);
    }

    return format;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_typed_format_id(
    adapter: *const wined3d_adapter,
    format: *const wined3d_format,
    channel_type: wined3d_channel_type,
) -> wined3d_format_id {
    let mut info: *const wined3d_typed_format_info =
        ::core::ptr::null::<wined3d_typed_format_info>();
    let mut i: uint32_t = 0;
    i = 0 as uint32_t;
    while (i as usize) < ARRAY_SIZE!(typed_formats) as usize {
        info = (&raw const typed_formats as *const wined3d_typed_format_info).offset(i as isize)
            as *const wined3d_typed_format_info;
        if (*info).typeless_id as ::core::ffi::c_uint
            == (*format).typeless_id as ::core::ffi::c_uint
            && map_channel_type(
                *(*info).channels.offset(0 as isize),
            ) as ::core::ffi::c_uint
                == channel_type as ::core::ffi::c_uint
        {
            return (*info).id;
        }
        i = i.wrapping_add(1);
    }
    return WINED3DFMT_UNKNOWN;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_is_depth_view(
    resource_format_id: wined3d_format_id,
    view_format_id: wined3d_format_id,
) -> BOOL {
    let mut i: u32;
    i = 0;
    while (i as usize) < ARRAY_SIZE!(typeless_depth_stencil_formats) as usize {
        if (typeless_depth_stencil_formats[i as usize].typeless_id as u32) == (resource_format_id as u32) {
            return ((typeless_depth_stencil_formats[i as usize].depth_view_id as u32) == (view_format_id as u32)) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_calculate_pitch(format: *const wined3d_format, alignment: ::core::ffi::c_uint, width: ::core::ffi::c_uint, height: ::core::ffi::c_uint, row_pitch: *mut ::core::ffi::c_uint, slice_pitch: *mut ::core::ffi::c_uint) {
    /* For block based formats, pitch means the amount of bytes to the next
     * row of blocks rather than the next row of pixels. */
    if ((*format).attrs & (WINED3D_FORMAT_ATTR_BLOCKS) as ::core::ffi::c_uint) != 0 {
        let row_block_count: ::core::ffi::c_uint = width
            .wrapping_add((*format).block_width as ::core::ffi::c_uint)
            .wrapping_sub((1) as ::core::ffi::c_uint)
            .wrapping_div((*format).block_width as ::core::ffi::c_uint);
        let slice_block_count: ::core::ffi::c_uint = height
            .wrapping_add((*format).block_height as ::core::ffi::c_uint)
            .wrapping_sub((1) as ::core::ffi::c_uint)
            .wrapping_div((*format).block_height as ::core::ffi::c_uint);
        *row_pitch = (row_block_count).wrapping_mul((*format).block_byte_count as ::core::ffi::c_uint)
            as ::core::ffi::c_uint;
        *row_pitch = (*row_pitch)
            .wrapping_add(alignment)
            .wrapping_sub((1) as ::core::ffi::c_uint)
            & !alignment.wrapping_sub((1) as ::core::ffi::c_uint);
        *slice_pitch = (*row_pitch).wrapping_mul(slice_block_count);
    } else {
        /* Bytes / row */
        *row_pitch = ((*format).byte_count as ::core::ffi::c_uint).wrapping_mul(width);
        *row_pitch = (*row_pitch)
            .wrapping_add(alignment)
            .wrapping_sub((1) as ::core::ffi::c_uint)
            & !alignment.wrapping_sub((1) as ::core::ffi::c_uint);
        *slice_pitch = (*row_pitch).wrapping_mul(height);
    }

    if ((*format).attrs & (WINED3D_FORMAT_ATTR_HEIGHT_SCALE) as ::core::ffi::c_uint) != 0 {
        /* The D3D format requirements make sure that the resulting format is an integer again */
        *slice_pitch =
            (*slice_pitch).wrapping_mul((*format).height_scale.numerator as ::core::ffi::c_uint);
        *slice_pitch =
            (*slice_pitch).wrapping_div((*format).height_scale.denominator as ::core::ffi::c_uint);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_calculate_size(format: *const wined3d_format, alignment: UINT, width: UINT, height: UINT, depth: UINT) -> UINT {
    let mut row_pitch: UINT = 0;
    let mut slice_pitch: UINT = 0;

    if (*format).id as u32 == WINED3DFMT_UNKNOWN {
        return 0;
    }

    if ((*format).attrs & WINED3D_FORMAT_ATTR_BROKEN_PITCH) != 0 {
        return width.wrapping_mul(height).wrapping_mul(depth).wrapping_mul((*format).byte_count);
    }

    wined3d_format_calculate_pitch(format, alignment, width, height, core::ptr::addr_of_mut!(row_pitch), core::ptr::addr_of_mut!(slice_pitch));

    return slice_pitch.wrapping_mul(depth);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_formats_are_srgb_variants(mut format1: wined3d_format_id, mut format2: wined3d_format_id) -> BOOL {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize) < ARRAY_SIZE!(format_srgb_info) as usize {
        if format1 as ::core::ffi::c_uint == format_srgb_info[i as usize].srgb_format_id as ::core::ffi::c_uint {
            return (format2 as ::core::ffi::c_uint == format_srgb_info[i as usize].base_format_id as ::core::ffi::c_uint) as BOOL;
        }
        if format1 as ::core::ffi::c_uint == format_srgb_info[i as usize].base_format_id as ::core::ffi::c_uint {
            return (format2 as ::core::ffi::c_uint == format_srgb_info[i as usize].srgb_format_id as ::core::ffi::c_uint) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn debug_box(box_0: *const wined3d_box) -> *const ::core::ffi::c_char {
    if box_0.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"(%u, %u, %u)-(%u, %u, %u)\0".as_ptr() as *const ::core::ffi::c_char,
        (*box_0).left,
        (*box_0).top,
        (*box_0).front,
        (*box_0).right,
        (*box_0).bottom,
        (*box_0).back,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_color(color: *const wined3d_color) -> *const ::core::ffi::c_char {
    if color.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{%.8e, %.8e, %.8e, %.8e}\0" as *const u8 as *const ::core::ffi::c_char,
        (*color).r as ::core::ffi::c_double,
        (*color).g as ::core::ffi::c_double,
        (*color).b as ::core::ffi::c_double,
        (*color).a as ::core::ffi::c_double,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_ivec4(v: *const wined3d_ivec4) -> *const ::core::ffi::c_char {
    if v.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{%d, %d, %d, %d}\0" as *const u8 as *const ::core::ffi::c_char,
        (*v).x,
        (*v).y,
        (*v).z,
        (*v).w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_uvec4(v: *const wined3d_uvec4) -> *const ::core::ffi::c_char {
    if v.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{%u, %u, %u, %u}\0".as_ptr() as *const ::core::ffi::c_char,
        (*v).x,
        (*v).y,
        (*v).z,
        (*v).w,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_vec4(v: *const wined3d_vec4) -> *const ::core::ffi::c_char {
    if v.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{%.8e, %.8e, %.8e, %.8e}\0" as *const u8 as *const ::core::ffi::c_char,
        (*v).x as ::core::ffi::c_double,
        (*v).y as ::core::ffi::c_double,
        (*v).z as ::core::ffi::c_double,
        (*v).w as ::core::ffi::c_double,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_const_bo_address(
    address: *const wined3d_const_bo_address,
) -> *const core::ffi::c_char {
    if address.is_null() {
        return b"(null)\0".as_ptr() as *const core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{%p:%p}\0".as_ptr() as *const core::ffi::c_char,
        (*address).buffer_object,
        (*address).addr,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_bo_address(
    mut address: *const wined3d_bo_address,
) -> *const ::core::ffi::c_char {
    return debug_const_bo_address(address as *const wined3d_const_bo_address);
}

#[no_mangle]
pub unsafe extern "C" fn debug_d3dformat(format_id: wined3d_format_id) -> *const ::core::ffi::c_char {
    match format_id as ::core::ffi::c_uint {
        WINED3DFMT_UNKNOWN => return concat!(stringify!(WINED3DFMT_UNKNOWN), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8_UNORM => return concat!(stringify!(WINED3DFMT_B8G8R8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B5G5R5X1_UNORM => return concat!(stringify!(WINED3DFMT_B5G5R5X1_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B4G4R4A4_UNORM => return concat!(stringify!(WINED3DFMT_B4G4R4A4_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B2G3R3_UNORM => return concat!(stringify!(WINED3DFMT_B2G3R3_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B2G3R3A8_UNORM => return concat!(stringify!(WINED3DFMT_B2G3R3A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B4G4R4X4_UNORM => return concat!(stringify!(WINED3DFMT_B4G4R4X4_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8X8_UNORM => return concat!(stringify!(WINED3DFMT_R8G8B8X8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B10G10R10A2_UNORM => return concat!(stringify!(WINED3DFMT_B10G10R10A2_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_P8_UINT_A8_UNORM => return concat!(stringify!(WINED3DFMT_P8_UINT_A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_P8_UINT => return concat!(stringify!(WINED3DFMT_P8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_L8_UNORM => return concat!(stringify!(WINED3DFMT_L8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_L8A8_UNORM => return concat!(stringify!(WINED3DFMT_L8A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_L4A4_UNORM => return concat!(stringify!(WINED3DFMT_L4A4_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R5G5_SNORM_L6_UNORM => return concat!(stringify!(WINED3DFMT_R5G5_SNORM_L6_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_SNORM_L8X8_UNORM => return concat!(stringify!(WINED3DFMT_R8G8_SNORM_L8X8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G11B11_SNORM => return concat!(stringify!(WINED3DFMT_R10G11B11_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10X2_TYPELESS => return concat!(stringify!(WINED3DFMT_R10G10B10X2_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10X2_UINT => return concat!(stringify!(WINED3DFMT_R10G10B10X2_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10X2_SNORM => return concat!(stringify!(WINED3DFMT_R10G10B10X2_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10_SNORM_A2_UNORM => return concat!(stringify!(WINED3DFMT_R10G10B10_SNORM_A2_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D16_LOCKABLE => return concat!(stringify!(WINED3DFMT_D16_LOCKABLE), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D32_UNORM => return concat!(stringify!(WINED3DFMT_D32_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_S1_UINT_D15_UNORM => return concat!(stringify!(WINED3DFMT_S1_UINT_D15_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_X8D24_UNORM => return concat!(stringify!(WINED3DFMT_X8D24_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_S4X4_UINT_D24_UNORM => return concat!(stringify!(WINED3DFMT_S4X4_UINT_D24_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_L16_UNORM => return concat!(stringify!(WINED3DFMT_L16_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_S8_UINT_D24_FLOAT => return concat!(stringify!(WINED3DFMT_S8_UINT_D24_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_SNORM_Cx => return concat!(stringify!(WINED3DFMT_R8G8_SNORM_Cx), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32A32_TYPELESS => return concat!(stringify!(WINED3DFMT_R32G32B32A32_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32A32_FLOAT => return concat!(stringify!(WINED3DFMT_R32G32B32A32_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32A32_UINT => return concat!(stringify!(WINED3DFMT_R32G32B32A32_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32A32_SINT => return concat!(stringify!(WINED3DFMT_R32G32B32A32_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32_TYPELESS => return concat!(stringify!(WINED3DFMT_R32G32B32_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32_FLOAT => return concat!(stringify!(WINED3DFMT_R32G32B32_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32_UINT => return concat!(stringify!(WINED3DFMT_R32G32B32_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32B32_SINT => return concat!(stringify!(WINED3DFMT_R32G32B32_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_TYPELESS => return concat!(stringify!(WINED3DFMT_R16G16B16A16_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_FLOAT => return concat!(stringify!(WINED3DFMT_R16G16B16A16_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_UNORM => return concat!(stringify!(WINED3DFMT_R16G16B16A16_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_UINT => return concat!(stringify!(WINED3DFMT_R16G16B16A16_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_SNORM => return concat!(stringify!(WINED3DFMT_R16G16B16A16_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16B16A16_SINT => return concat!(stringify!(WINED3DFMT_R16G16B16A16_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32_TYPELESS => return concat!(stringify!(WINED3DFMT_R32G32_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32_FLOAT => return concat!(stringify!(WINED3DFMT_R32G32_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32_UINT => return concat!(stringify!(WINED3DFMT_R32G32_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G32_SINT => return concat!(stringify!(WINED3DFMT_R32G32_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32G8X24_TYPELESS => return concat!(stringify!(WINED3DFMT_R32G8X24_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D32_FLOAT_S8X24_UINT => return concat!(stringify!(WINED3DFMT_D32_FLOAT_S8X24_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32_FLOAT_X8X24_TYPELESS => return concat!(stringify!(WINED3DFMT_R32_FLOAT_X8X24_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_X32_TYPELESS_G8X24_UINT => return concat!(stringify!(WINED3DFMT_X32_TYPELESS_G8X24_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10A2_TYPELESS => return concat!(stringify!(WINED3DFMT_R10G10B10A2_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10A2_UNORM => return concat!(stringify!(WINED3DFMT_R10G10B10A2_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10A2_UINT => return concat!(stringify!(WINED3DFMT_R10G10B10A2_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10A2_SNORM => return concat!(stringify!(WINED3DFMT_R10G10B10A2_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R10G10B10_XR_BIAS_A2_UNORM => return concat!(stringify!(WINED3DFMT_R10G10B10_XR_BIAS_A2_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R11G11B10_FLOAT => return concat!(stringify!(WINED3DFMT_R11G11B10_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_TYPELESS => return concat!(stringify!(WINED3DFMT_R8G8B8A8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_UNORM => return concat!(stringify!(WINED3DFMT_R8G8B8A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_R8G8B8A8_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_UINT => return concat!(stringify!(WINED3DFMT_R8G8B8A8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_SNORM => return concat!(stringify!(WINED3DFMT_R8G8B8A8_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8B8A8_SINT => return concat!(stringify!(WINED3DFMT_R8G8B8A8_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_TYPELESS => return concat!(stringify!(WINED3DFMT_R16G16_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_FLOAT => return concat!(stringify!(WINED3DFMT_R16G16_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_UNORM => return concat!(stringify!(WINED3DFMT_R16G16_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_UINT => return concat!(stringify!(WINED3DFMT_R16G16_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_SNORM => return concat!(stringify!(WINED3DFMT_R16G16_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16G16_SINT => return concat!(stringify!(WINED3DFMT_R16G16_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32_TYPELESS => return concat!(stringify!(WINED3DFMT_R32_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D32_FLOAT => return concat!(stringify!(WINED3DFMT_D32_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32_FLOAT => return concat!(stringify!(WINED3DFMT_R32_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32_UINT => return concat!(stringify!(WINED3DFMT_R32_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R32_SINT => return concat!(stringify!(WINED3DFMT_R32_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R24G8_TYPELESS => return concat!(stringify!(WINED3DFMT_R24G8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D24_UNORM_S8_UINT => return concat!(stringify!(WINED3DFMT_D24_UNORM_S8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R24_UNORM_X8_TYPELESS => return concat!(stringify!(WINED3DFMT_R24_UNORM_X8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_X24_TYPELESS_G8_UINT => return concat!(stringify!(WINED3DFMT_X24_TYPELESS_G8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_TYPELESS => return concat!(stringify!(WINED3DFMT_R8G8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_UNORM => return concat!(stringify!(WINED3DFMT_R8G8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_UINT => return concat!(stringify!(WINED3DFMT_R8G8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_SNORM => return concat!(stringify!(WINED3DFMT_R8G8_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_SINT => return concat!(stringify!(WINED3DFMT_R8G8_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_TYPELESS => return concat!(stringify!(WINED3DFMT_R16_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_FLOAT => return concat!(stringify!(WINED3DFMT_R16_FLOAT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_D16_UNORM => return concat!(stringify!(WINED3DFMT_D16_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_UNORM => return concat!(stringify!(WINED3DFMT_R16_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_UINT => return concat!(stringify!(WINED3DFMT_R16_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_SNORM => return concat!(stringify!(WINED3DFMT_R16_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16_SINT => return concat!(stringify!(WINED3DFMT_R16_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8_TYPELESS => return concat!(stringify!(WINED3DFMT_R8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8_UNORM => return concat!(stringify!(WINED3DFMT_R8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8_UINT => return concat!(stringify!(WINED3DFMT_R8_UINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8_SNORM => return concat!(stringify!(WINED3DFMT_R8_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8_SINT => return concat!(stringify!(WINED3DFMT_R8_SINT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_A8_UNORM => return concat!(stringify!(WINED3DFMT_A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R1_UNORM => return concat!(stringify!(WINED3DFMT_R1_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R9G9B9E5_SHAREDEXP => return concat!(stringify!(WINED3DFMT_R9G9B9E5_SHAREDEXP), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_B8G8_UNORM => return concat!(stringify!(WINED3DFMT_R8G8_B8G8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_G8R8_G8B8_UNORM => return concat!(stringify!(WINED3DFMT_G8R8_G8B8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC1_TYPELESS => return concat!(stringify!(WINED3DFMT_BC1_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC1_UNORM => return concat!(stringify!(WINED3DFMT_BC1_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC1_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_BC1_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC2_TYPELESS => return concat!(stringify!(WINED3DFMT_BC2_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC2_UNORM => return concat!(stringify!(WINED3DFMT_BC2_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC2_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_BC2_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC3_TYPELESS => return concat!(stringify!(WINED3DFMT_BC3_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC3_UNORM => return concat!(stringify!(WINED3DFMT_BC3_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC3_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_BC3_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC4_TYPELESS => return concat!(stringify!(WINED3DFMT_BC4_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC4_UNORM => return concat!(stringify!(WINED3DFMT_BC4_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC4_SNORM => return concat!(stringify!(WINED3DFMT_BC4_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC5_TYPELESS => return concat!(stringify!(WINED3DFMT_BC5_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC5_UNORM => return concat!(stringify!(WINED3DFMT_BC5_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC5_SNORM => return concat!(stringify!(WINED3DFMT_BC5_SNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B5G6R5_UNORM => return concat!(stringify!(WINED3DFMT_B5G6R5_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B5G5R5A1_UNORM => return concat!(stringify!(WINED3DFMT_B5G5R5A1_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8A8_UNORM => return concat!(stringify!(WINED3DFMT_B8G8R8A8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8X8_UNORM => return concat!(stringify!(WINED3DFMT_B8G8R8X8_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8A8_TYPELESS => return concat!(stringify!(WINED3DFMT_B8G8R8A8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8A8_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_B8G8R8A8_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8X8_TYPELESS => return concat!(stringify!(WINED3DFMT_B8G8R8X8_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_B8G8R8X8_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_B8G8R8X8_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC6H_TYPELESS => return concat!(stringify!(WINED3DFMT_BC6H_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC6H_UF16 => return concat!(stringify!(WINED3DFMT_BC6H_UF16), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC6H_SF16 => return concat!(stringify!(WINED3DFMT_BC6H_SF16), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC7_TYPELESS => return concat!(stringify!(WINED3DFMT_BC7_TYPELESS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC7_UNORM => return concat!(stringify!(WINED3DFMT_BC7_UNORM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_BC7_UNORM_SRGB => return concat!(stringify!(WINED3DFMT_BC7_UNORM_SRGB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_UYVY => return concat!(stringify!(WINED3DFMT_UYVY), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_YUY2 => return concat!(stringify!(WINED3DFMT_YUY2), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_YV12 => return concat!(stringify!(WINED3DFMT_YV12), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_DXT1 => return concat!(stringify!(WINED3DFMT_DXT1), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_DXT2 => return concat!(stringify!(WINED3DFMT_DXT2), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_DXT3 => return concat!(stringify!(WINED3DFMT_DXT3), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_DXT4 => return concat!(stringify!(WINED3DFMT_DXT4), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_DXT5 => return concat!(stringify!(WINED3DFMT_DXT5), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_MULTI2_ARGB8 => return concat!(stringify!(WINED3DFMT_MULTI2_ARGB8), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_G8R8_G8B8 => return concat!(stringify!(WINED3DFMT_G8R8_G8B8), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R8G8_B8G8 => return concat!(stringify!(WINED3DFMT_R8G8_B8G8), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_ATI1N => return concat!(stringify!(WINED3DFMT_ATI1N), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_ATI2N => return concat!(stringify!(WINED3DFMT_ATI2N), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_INST => return concat!(stringify!(WINED3DFMT_INST), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_NVDB => return concat!(stringify!(WINED3DFMT_NVDB), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_NVHU => return concat!(stringify!(WINED3DFMT_NVHU), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_NVHS => return concat!(stringify!(WINED3DFMT_NVHS), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_INTZ => return concat!(stringify!(WINED3DFMT_INTZ), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_RESZ => return concat!(stringify!(WINED3DFMT_RESZ), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_NULL => return concat!(stringify!(WINED3DFMT_NULL), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_R16 => return concat!(stringify!(WINED3DFMT_R16), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_AL16 => return concat!(stringify!(WINED3DFMT_AL16), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_NV12 => return concat!(stringify!(WINED3DFMT_NV12), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3DFMT_ATOC => return concat!(stringify!(WINED3DFMT_ATOC), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {
            let mut fourcc: [::core::ffi::c_char; 5] = [0; 5];
            fourcc[0 as usize] = format_id as ::core::ffi::c_char;
            fourcc[1 as usize] = ((format_id as ::core::ffi::c_uint) >> 8) as ::core::ffi::c_char;
            fourcc[2 as usize] = ((format_id as ::core::ffi::c_uint) >> 16) as ::core::ffi::c_char;
            fourcc[3 as usize] = ((format_id as ::core::ffi::c_uint) >> 24) as ::core::ffi::c_char;
            fourcc[4 as usize] = 0 as ::core::ffi::c_char;
            if isprint(fourcc[0 as usize] as ::core::ffi::c_int) != 0
                && isprint(fourcc[1 as usize] as ::core::ffi::c_int) != 0
                && isprint(fourcc[2 as usize] as ::core::ffi::c_int) != 0
                && isprint(fourcc[3 as usize] as ::core::ffi::c_int) != 0
            {
                WINE_FIXME!("Unrecognized %#x (as fourcc: %s) WINED3DFORMAT!\n", format_id as ::core::ffi::c_uint, &raw mut fourcc as *mut ::core::ffi::c_char);
            } else { WINE_FIXME!("Unrecognized %#x WINED3DFORMAT!\n", format_id as ::core::ffi::c_uint); }
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3ddevicetype(mut device_type: wined3d_device_type) -> *const ::core::ffi::c_char {
    match device_type as ::core::ffi::c_uint {
        WINED3D_DEVICE_TYPE_HAL => {
            return concat!(stringify!(WINED3D_DEVICE_TYPE_HAL), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_DEVICE_TYPE_REF => {
            return concat!(stringify!(WINED3D_DEVICE_TYPE_REF), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_DEVICE_TYPE_SW => {
            return concat!(stringify!(WINED3D_DEVICE_TYPE_SW), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized device type %#x.\n", device_type as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}

unsafe extern "C" fn init_debug_buffer(buffer: *mut debug_buffer, default_string: *const i8) {
    snprintf(
        core::ptr::addr_of_mut!((*buffer).str) as *mut i8,
        core::mem::size_of::<[i8; 200]>(),
        b"%s\0".as_ptr() as *const i8,
        default_string,
    );
    (*buffer).ptr = core::ptr::addr_of_mut!((*buffer).str) as *mut i8;
    (*buffer).size = core::mem::size_of::<[i8; 200]>() as i32;
}

unsafe extern "C" fn debug_append(mut buffer: *mut debug_buffer, mut str: *const i8, mut separator: *const i8) {
    let mut size: i32 = 0;
    if separator.is_null() || (*buffer).ptr == core::ptr::addr_of_mut!((*buffer).str) as *mut i8 {
        separator = b"\0".as_ptr() as *const i8;
    }
    size = snprintf(
        (*buffer).ptr,
        (*buffer).size as usize,
        b"%s%s\0".as_ptr() as *const i8,
        separator,
        str,
    );
    if size == -1 || size >= (*buffer).size {
        (*buffer).size = 0;
        strcpy(
            (core::ptr::addr_of_mut!((*buffer).str) as *mut i8).offset(
                (core::mem::size_of::<[i8; 200]>()).wrapping_sub(4) as isize,
            ) as *mut i8,
            b"...\0".as_ptr() as *const i8,
        );
        return;
    }
    (*buffer).ptr = (*buffer).ptr.offset(size as isize);
    (*buffer).size -= size;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_resource_access(mut access: uint32_t) -> *const ::core::ffi::c_char {
    let mut buffer: debug_buffer = debug_buffer {
        str: [0; 200],
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    init_debug_buffer(
        &raw mut buffer,
        b"0\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (access & WINED3D_RESOURCE_ACCESS_GPU) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_RESOURCE_ACCESS_GPU\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        access &= !WINED3D_RESOURCE_ACCESS_GPU;
    }
    if (access & WINED3D_RESOURCE_ACCESS_CPU) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_RESOURCE_ACCESS_CPU\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        access &= !WINED3D_RESOURCE_ACCESS_CPU;
    }
    if (access & WINED3D_RESOURCE_ACCESS_MAP_R) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_RESOURCE_ACCESS_MAP_R\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        access &= !WINED3D_RESOURCE_ACCESS_MAP_R;
    }
    if (access & WINED3D_RESOURCE_ACCESS_MAP_W) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_RESOURCE_ACCESS_MAP_W\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        access &= !WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    if access != 0 {
        FIXME!("Unrecognised access flag(s) %#x.\n", access);
    }
    return wine_dbg_sprintf(
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut buffer.str as *mut ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_bind_flags(mut bind_flags: uint32_t) -> *const ::core::ffi::c_char {
    let mut buffer: debug_buffer = core::mem::zeroed();

    init_debug_buffer(
        core::ptr::addr_of_mut!(buffer),
        b"0\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if bind_flags & WINED3D_BIND_VERTEX_BUFFER != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_VERTEX_BUFFER\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_VERTEX_BUFFER;
    }
    if bind_flags & WINED3D_BIND_INDEX_BUFFER != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_INDEX_BUFFER\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_INDEX_BUFFER;
    }
    if bind_flags & WINED3D_BIND_CONSTANT_BUFFER != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_CONSTANT_BUFFER\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_CONSTANT_BUFFER;
    }
    if bind_flags & WINED3D_BIND_SHADER_RESOURCE != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_SHADER_RESOURCE\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_SHADER_RESOURCE;
    }
    if bind_flags & WINED3D_BIND_STREAM_OUTPUT != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_STREAM_OUTPUT\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_STREAM_OUTPUT;
    }
    if bind_flags & WINED3D_BIND_RENDER_TARGET != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_RENDER_TARGET\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_RENDER_TARGET;
    }
    if bind_flags & WINED3D_BIND_DEPTH_STENCIL != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_DEPTH_STENCIL\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_DEPTH_STENCIL;
    }
    if bind_flags & WINED3D_BIND_UNORDERED_ACCESS != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_UNORDERED_ACCESS\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_UNORDERED_ACCESS;
    }
    if bind_flags & WINED3D_BIND_INDIRECT_BUFFER != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_BIND_INDIRECT_BUFFER\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        bind_flags &= !WINED3D_BIND_INDIRECT_BUFFER;
    }
    if bind_flags != 0 {
        FIXME!("Unrecognised bind flag(s) %#x.\n", bind_flags);
    }

    return wine_dbg_sprintf(
        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
        core::ptr::addr_of_mut!(buffer.str) as *mut ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_view_desc(d: *const wined3d_view_desc, resource: *const wined3d_resource) -> *const ::core::ffi::c_char {
    let mut buffer: debug_buffer = debug_buffer {
        str: [0; 200],
        ptr: core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut flags: ::core::ffi::c_uint = (*d).flags;

    init_debug_buffer(
        core::ptr::addr_of_mut!(buffer),
        b"0\0".as_ptr() as *const ::core::ffi::c_char,
    );

    if (flags & (WINED3D_VIEW_BUFFER_RAW as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_BUFFER_RAW\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_BUFFER_RAW as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_BUFFER_APPEND as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_BUFFER_APPEND\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_BUFFER_APPEND as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_BUFFER_COUNTER as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_BUFFER_COUNTER\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_BUFFER_COUNTER as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_TEXTURE_CUBE as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_TEXTURE_CUBE\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_TEXTURE_CUBE as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_TEXTURE_ARRAY\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_TEXTURE_ARRAY as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_READ_ONLY_DEPTH as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_READ_ONLY_DEPTH\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_READ_ONLY_DEPTH as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_READ_ONLY_STENCIL as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_READ_ONLY_STENCIL\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_READ_ONLY_STENCIL as ::core::ffi::c_uint);
    }
    if (flags & (WINED3D_VIEW_FORWARD_REFERENCE as ::core::ffi::c_uint)) != 0 {
        debug_append(
            core::ptr::addr_of_mut!(buffer),
            b"WINED3D_VIEW_FORWARD_REFERENCE\0".as_ptr() as *const ::core::ffi::c_char,
            b" | \0".as_ptr() as *const ::core::ffi::c_char,
        );
        flags &= !(WINED3D_VIEW_FORWARD_REFERENCE as ::core::ffi::c_uint);
    }

    if flags != 0 {
        FIXME!("Unrecognised view flag(s) %#x.\n", flags);
    }

    if (*resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        return wine_dbg_sprintf(
            b"format %s, flags %s, start_idx %u, count %u\0".as_ptr() as *const ::core::ffi::c_char,
            debug_d3dformat((*d).format_id),
            core::ptr::addr_of_mut!(buffer.str) as *mut ::core::ffi::c_char,
            (*d).u.buffer.start_idx,
            (*d).u.buffer.count,
        );
    } else {
        return wine_dbg_sprintf(
            b"format %s, flags %s, level_idx %u, level_count %u, layer_idx %u, layer_count %u\0"
                .as_ptr() as *const ::core::ffi::c_char,
            debug_d3dformat((*d).format_id),
            core::ptr::addr_of_mut!(buffer.str) as *mut ::core::ffi::c_char,
            (*d).u.texture.level_idx,
            (*d).u.texture.level_count,
            (*d).u.texture.layer_idx,
            (*d).u.texture.layer_count,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dusage(mut usage: uint32_t) -> *const ::core::ffi::c_char {
    let mut buffer: debug_buffer = debug_buffer {
        str: [0; 200],
        ptr: core::ptr::null_mut::<core::ffi::c_char>(),
        size: 0,
    };
    init_debug_buffer(
        &raw mut buffer,
        b"0\0" as *const u8 as *const core::ffi::c_char,
    );
    if usage & WINED3DUSAGE_SOFTWAREPROCESSING as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_SOFTWAREPROCESSING\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_SOFTWAREPROCESSING as u32);
    }
    if usage & WINED3DUSAGE_DONOTCLIP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_DONOTCLIP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_DONOTCLIP as u32);
    }
    if usage & WINED3DUSAGE_POINTS as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_POINTS\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_POINTS as u32);
    }
    if usage & WINED3DUSAGE_RTPATCHES as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_RTPATCHES\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_RTPATCHES as u32);
    }
    if usage & WINED3DUSAGE_NPATCHES as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_NPATCHES\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_NPATCHES as u32);
    }
    if usage & WINED3DUSAGE_DYNAMIC as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_DYNAMIC\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_DYNAMIC as u32);
    }
    if usage & WINED3DUSAGE_RESTRICTED_CONTENT as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_RESTRICTED_CONTENT\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_RESTRICTED_CONTENT as u32);
    }
    if usage & WINED3DUSAGE_RESTRICT_SHARED_RESOURCE_DRIVER as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_RESTRICT_SHARED_RESOURCE_DRIVER\0" as *const u8
                as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_RESTRICT_SHARED_RESOURCE_DRIVER as u32);
    }
    if usage & WINED3DUSAGE_RESTRICT_SHARED_RESOURCE as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_RESTRICT_SHARED_RESOURCE\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_RESTRICT_SHARED_RESOURCE as u32);
    }
    if usage & WINED3DUSAGE_DMAP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_DMAP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_DMAP as u32);
    }
    if usage & WINED3DUSAGE_TEXTAPI as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_TEXTAPI\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_TEXTAPI as u32);
    }
    if usage & WINED3DUSAGE_LEGACY_CUBEMAP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_LEGACY_CUBEMAP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_LEGACY_CUBEMAP as u32);
    }
    if usage & WINED3DUSAGE_OWNDC as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_OWNDC\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_OWNDC as u32);
    }
    if usage & WINED3DUSAGE_STATICDECL as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_STATICDECL\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_STATICDECL as u32);
    }
    if usage & WINED3DUSAGE_OVERLAY as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_OVERLAY\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_OVERLAY as u32);
    }
    if usage & WINED3DUSAGE_MANAGED as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_MANAGED\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_MANAGED as u32);
    }
    if usage & WINED3DUSAGE_VIDMEM_ACCOUNTING as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_VIDMEM_ACCOUNTING\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_VIDMEM_ACCOUNTING as u32);
    }
    if usage & WINED3DUSAGE_QUERY_FILTER as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_FILTER\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_FILTER as u32);
    }
    if usage & WINED3DUSAGE_QUERY_GENMIPMAP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_GENMIPMAP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_GENMIPMAP as u32);
    }
    if usage & WINED3DUSAGE_QUERY_LEGACYBUMPMAP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_LEGACYBUMPMAP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_LEGACYBUMPMAP as u32);
    }
    if usage & WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING\0" as *const u8
                as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_POSTPIXELSHADER_BLENDING as u32);
    }
    if usage & WINED3DUSAGE_QUERY_SRGBREAD as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_SRGBREAD\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_SRGBREAD as u32);
    }
    if usage & WINED3DUSAGE_QUERY_SRGBWRITE as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_SRGBWRITE\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_SRGBWRITE as u32);
    }
    if usage & WINED3DUSAGE_QUERY_VERTEXTEXTURE as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_VERTEXTEXTURE\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_VERTEXTEXTURE as u32);
    }
    if usage & WINED3DUSAGE_QUERY_WRAPANDMIP as u32 != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3DUSAGE_QUERY_WRAPANDMIP\0" as *const u8 as *const core::ffi::c_char,
            b" | \0" as *const u8 as *const core::ffi::c_char,
        );
        usage &= !(WINED3DUSAGE_QUERY_WRAPANDMIP as u32);
    }
    if usage != 0 {
        WINE_FIXME!("Unrecognized usage flag(s) %#x.\n", usage);
    }
    return wine_dbg_sprintf(
        b"%s\0" as *const u8 as *const core::ffi::c_char,
        &raw mut buffer.str as *mut core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3ddeclmethod(
    mut method: wined3d_decl_method,
) -> *const ::core::ffi::c_char {
    match method as ::core::ffi::c_uint {
        WINED3D_DECL_METHOD_DEFAULT => {
            return b"WINED3D_DECL_METHOD_DEFAULT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_PARTIAL_U => {
            return b"WINED3D_DECL_METHOD_PARTIAL_U\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_PARTIAL_V => {
            return b"WINED3D_DECL_METHOD_PARTIAL_V\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_CROSS_UV => {
            return b"WINED3D_DECL_METHOD_CROSS_UV\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_UV => {
            return b"WINED3D_DECL_METHOD_UV\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_LOOKUP => {
            return b"WINED3D_DECL_METHOD_LOOKUP\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_METHOD_LOOKUP_PRESAMPLED => {
            return b"WINED3D_DECL_METHOD_LOOKUP_PRESAMPLED\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized declaration method %#x.\n", method as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3ddeclusage(
    mut usage: wined3d_decl_usage,
) -> *const ::core::ffi::c_char {
    match usage {
        WINED3D_DECL_USAGE_POSITION => {
            return b"WINED3D_DECL_USAGE_POSITION\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_BLEND_WEIGHT => {
            return b"WINED3D_DECL_USAGE_BLEND_WEIGHT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_BLEND_INDICES => {
            return b"WINED3D_DECL_USAGE_BLEND_INDICES\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_NORMAL => {
            return b"WINED3D_DECL_USAGE_NORMAL\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_PSIZE => {
            return b"WINED3D_DECL_USAGE_PSIZE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_TEXCOORD => {
            return b"WINED3D_DECL_USAGE_TEXCOORD\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_TANGENT => {
            return b"WINED3D_DECL_USAGE_TANGENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_BINORMAL => {
            return b"WINED3D_DECL_USAGE_BINORMAL\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_TESS_FACTOR => {
            return b"WINED3D_DECL_USAGE_TESS_FACTOR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_POSITIONT => {
            return b"WINED3D_DECL_USAGE_POSITIONT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_COLOR => {
            return b"WINED3D_DECL_USAGE_COLOR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_FOG => {
            return b"WINED3D_DECL_USAGE_FOG\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_DEPTH => {
            return b"WINED3D_DECL_USAGE_DEPTH\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_DECL_USAGE_SAMPLE => {
            return b"WINED3D_DECL_USAGE_SAMPLE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized %u declaration usage!\n", usage as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dinput_classification(
    mut classification: wined3d_input_classification,
) -> *const ::core::ffi::c_char {
    match classification {
        WINED3D_INPUT_PER_VERTEX_DATA => {
            return b"WINED3D_INPUT_PER_VERTEX_DATA\0".as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_INPUT_PER_INSTANCE_DATA => {
            return b"WINED3D_INPUT_PER_INSTANCE_DATA\0".as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized input classification %#x.\n", classification as ::core::ffi::c_uint);
            return b"unrecognized\0".as_ptr() as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dresourcetype(resource_type: wined3d_resource_type) -> *const ::core::ffi::c_char {
    match resource_type {
        WINED3D_RTYPE_NONE => {
            let _ = WINED3D_RTYPE_NONE;
            return b"WINED3D_RTYPE_NONE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_RTYPE_BUFFER => {
            let _ = WINED3D_RTYPE_BUFFER;
            return b"WINED3D_RTYPE_BUFFER\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_RTYPE_TEXTURE_1D => {
            let _ = WINED3D_RTYPE_TEXTURE_1D;
            return b"WINED3D_RTYPE_TEXTURE_1D\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_RTYPE_TEXTURE_2D => {
            let _ = WINED3D_RTYPE_TEXTURE_2D;
            return b"WINED3D_RTYPE_TEXTURE_2D\0" as *const u8 as *const ::core::ffi::c_char;
        }
        WINED3D_RTYPE_TEXTURE_3D => {
            let _ = WINED3D_RTYPE_TEXTURE_3D;
            return b"WINED3D_RTYPE_TEXTURE_3D\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized resource type %#x.\n", resource_type as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dprimitivetype(mut primitive_type: wined3d_primitive_type) -> *const ::core::ffi::c_char {
    match primitive_type as ::core::ffi::c_uint {
        WINED3D_PT_UNDEFINED => return b"WINED3D_PT_UNDEFINED\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_POINTLIST => return b"WINED3D_PT_POINTLIST\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_LINELIST => return b"WINED3D_PT_LINELIST\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_LINESTRIP => return b"WINED3D_PT_LINESTRIP\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_TRIANGLELIST => return b"WINED3D_PT_TRIANGLELIST\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_TRIANGLESTRIP => return b"WINED3D_PT_TRIANGLESTRIP\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_TRIANGLEFAN => return b"WINED3D_PT_TRIANGLEFAN\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_LINELIST_ADJ => return b"WINED3D_PT_LINELIST_ADJ\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_LINESTRIP_ADJ => return b"WINED3D_PT_LINESTRIP_ADJ\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_TRIANGLELIST_ADJ => return b"WINED3D_PT_TRIANGLELIST_ADJ\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_TRIANGLESTRIP_ADJ => return b"WINED3D_PT_TRIANGLESTRIP_ADJ\0" as *const u8 as *const ::core::ffi::c_char,
        WINED3D_PT_PATCH => return b"WINED3D_PT_PATCH\0" as *const u8 as *const ::core::ffi::c_char,
        _ => {
            FIXME!("Unrecognized primitive type %#x.\n", primitive_type as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3drenderstate(state: wined3d_render_state) -> *const ::core::ffi::c_char {
    match state {
        WINED3D_RS_ANTIALIAS => return concat!(stringify!(WINED3D_RS_ANTIALIAS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_TEXTUREPERSPECTIVE => return concat!(stringify!(WINED3D_RS_TEXTUREPERSPECTIVE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAPU => return concat!(stringify!(WINED3D_RS_WRAPU), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAPV => return concat!(stringify!(WINED3D_RS_WRAPV), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ZENABLE => return concat!(stringify!(WINED3D_RS_ZENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FILLMODE => return concat!(stringify!(WINED3D_RS_FILLMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SHADEMODE => return concat!(stringify!(WINED3D_RS_SHADEMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_LINEPATTERN => return concat!(stringify!(WINED3D_RS_LINEPATTERN), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MONOENABLE => return concat!(stringify!(WINED3D_RS_MONOENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ROP2 => return concat!(stringify!(WINED3D_RS_ROP2), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_PLANEMASK => return concat!(stringify!(WINED3D_RS_PLANEMASK), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ZWRITEENABLE => return concat!(stringify!(WINED3D_RS_ZWRITEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ALPHATESTENABLE => return concat!(stringify!(WINED3D_RS_ALPHATESTENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_LASTPIXEL => return concat!(stringify!(WINED3D_RS_LASTPIXEL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SRCBLEND => return concat!(stringify!(WINED3D_RS_SRCBLEND), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DESTBLEND => return concat!(stringify!(WINED3D_RS_DESTBLEND), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_CULLMODE => return concat!(stringify!(WINED3D_RS_CULLMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ZFUNC => return concat!(stringify!(WINED3D_RS_ZFUNC), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ALPHAREF => return concat!(stringify!(WINED3D_RS_ALPHAREF), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ALPHAFUNC => return concat!(stringify!(WINED3D_RS_ALPHAFUNC), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DITHERENABLE => return concat!(stringify!(WINED3D_RS_DITHERENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ALPHABLENDENABLE => return concat!(stringify!(WINED3D_RS_ALPHABLENDENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGENABLE => return concat!(stringify!(WINED3D_RS_FOGENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SPECULARENABLE => return concat!(stringify!(WINED3D_RS_SPECULARENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ZVISIBLE => return concat!(stringify!(WINED3D_RS_ZVISIBLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SUBPIXEL => return concat!(stringify!(WINED3D_RS_SUBPIXEL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SUBPIXELX => return concat!(stringify!(WINED3D_RS_SUBPIXELX), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STIPPLEDALPHA => return concat!(stringify!(WINED3D_RS_STIPPLEDALPHA), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGCOLOR => return concat!(stringify!(WINED3D_RS_FOGCOLOR), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGTABLEMODE => return concat!(stringify!(WINED3D_RS_FOGTABLEMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGSTART => return concat!(stringify!(WINED3D_RS_FOGSTART), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGEND => return concat!(stringify!(WINED3D_RS_FOGEND), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGDENSITY => return concat!(stringify!(WINED3D_RS_FOGDENSITY), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STIPPLEENABLE => return concat!(stringify!(WINED3D_RS_STIPPLEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORKEYENABLE => return concat!(stringify!(WINED3D_RS_COLORKEYENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MIPMAPLODBIAS => return concat!(stringify!(WINED3D_RS_MIPMAPLODBIAS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_RANGEFOGENABLE => return concat!(stringify!(WINED3D_RS_RANGEFOGENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ANISOTROPY => return concat!(stringify!(WINED3D_RS_ANISOTROPY), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FLUSHBATCH => return concat!(stringify!(WINED3D_RS_FLUSHBATCH), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_TRANSLUCENTSORTINDEPENDENT => return concat!(stringify!(WINED3D_RS_TRANSLUCENTSORTINDEPENDENT), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILENABLE => return concat!(stringify!(WINED3D_RS_STENCILENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILFAIL => return concat!(stringify!(WINED3D_RS_STENCILFAIL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILZFAIL => return concat!(stringify!(WINED3D_RS_STENCILZFAIL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILPASS => return concat!(stringify!(WINED3D_RS_STENCILPASS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILFUNC => return concat!(stringify!(WINED3D_RS_STENCILFUNC), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILREF => return concat!(stringify!(WINED3D_RS_STENCILREF), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILMASK => return concat!(stringify!(WINED3D_RS_STENCILMASK), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_STENCILWRITEMASK => return concat!(stringify!(WINED3D_RS_STENCILWRITEMASK), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_TEXTUREFACTOR => return concat!(stringify!(WINED3D_RS_TEXTUREFACTOR), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP0 => return concat!(stringify!(WINED3D_RS_WRAP0), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP1 => return concat!(stringify!(WINED3D_RS_WRAP1), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP2 => return concat!(stringify!(WINED3D_RS_WRAP2), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP3 => return concat!(stringify!(WINED3D_RS_WRAP3), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP4 => return concat!(stringify!(WINED3D_RS_WRAP4), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP5 => return concat!(stringify!(WINED3D_RS_WRAP5), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP6 => return concat!(stringify!(WINED3D_RS_WRAP6), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP7 => return concat!(stringify!(WINED3D_RS_WRAP7), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_CLIPPING => return concat!(stringify!(WINED3D_RS_CLIPPING), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_LIGHTING => return concat!(stringify!(WINED3D_RS_LIGHTING), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_EXTENTS => return concat!(stringify!(WINED3D_RS_EXTENTS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_AMBIENT => return concat!(stringify!(WINED3D_RS_AMBIENT), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_FOGVERTEXMODE => return concat!(stringify!(WINED3D_RS_FOGVERTEXMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORVERTEX => return concat!(stringify!(WINED3D_RS_COLORVERTEX), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_LOCALVIEWER => return concat!(stringify!(WINED3D_RS_LOCALVIEWER), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_NORMALIZENORMALS => return concat!(stringify!(WINED3D_RS_NORMALIZENORMALS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORKEYBLENDENABLE => return concat!(stringify!(WINED3D_RS_COLORKEYBLENDENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DIFFUSEMATERIALSOURCE => return concat!(stringify!(WINED3D_RS_DIFFUSEMATERIALSOURCE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SPECULARMATERIALSOURCE => return concat!(stringify!(WINED3D_RS_SPECULARMATERIALSOURCE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_AMBIENTMATERIALSOURCE => return concat!(stringify!(WINED3D_RS_AMBIENTMATERIALSOURCE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_EMISSIVEMATERIALSOURCE => return concat!(stringify!(WINED3D_RS_EMISSIVEMATERIALSOURCE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_VERTEXBLEND => return concat!(stringify!(WINED3D_RS_VERTEXBLEND), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_CLIPPLANEENABLE => return concat!(stringify!(WINED3D_RS_CLIPPLANEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SOFTWAREVERTEXPROCESSING => return concat!(stringify!(WINED3D_RS_SOFTWAREVERTEXPROCESSING), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSIZE => return concat!(stringify!(WINED3D_RS_POINTSIZE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSIZE_MIN => return concat!(stringify!(WINED3D_RS_POINTSIZE_MIN), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSPRITEENABLE => return concat!(stringify!(WINED3D_RS_POINTSPRITEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSCALEENABLE => return concat!(stringify!(WINED3D_RS_POINTSCALEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSCALE_A => return concat!(stringify!(WINED3D_RS_POINTSCALE_A), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSCALE_B => return concat!(stringify!(WINED3D_RS_POINTSCALE_B), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSCALE_C => return concat!(stringify!(WINED3D_RS_POINTSCALE_C), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MULTISAMPLEANTIALIAS => return concat!(stringify!(WINED3D_RS_MULTISAMPLEANTIALIAS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MULTISAMPLEMASK => return concat!(stringify!(WINED3D_RS_MULTISAMPLEMASK), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_PATCHEDGESTYLE => return concat!(stringify!(WINED3D_RS_PATCHEDGESTYLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_PATCHSEGMENTS => return concat!(stringify!(WINED3D_RS_PATCHSEGMENTS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DEBUGMONITORTOKEN => return concat!(stringify!(WINED3D_RS_DEBUGMONITORTOKEN), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POINTSIZE_MAX => return concat!(stringify!(WINED3D_RS_POINTSIZE_MAX), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_INDEXEDVERTEXBLENDENABLE => return concat!(stringify!(WINED3D_RS_INDEXEDVERTEXBLENDENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORWRITEENABLE => return concat!(stringify!(WINED3D_RS_COLORWRITEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_TWEENFACTOR => return concat!(stringify!(WINED3D_RS_TWEENFACTOR), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BLENDOP => return concat!(stringify!(WINED3D_RS_BLENDOP), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_POSITIONDEGREE => return concat!(stringify!(WINED3D_RS_POSITIONDEGREE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_NORMALDEGREE => return concat!(stringify!(WINED3D_RS_NORMALDEGREE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SCISSORTESTENABLE => return concat!(stringify!(WINED3D_RS_SCISSORTESTENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SLOPESCALEDEPTHBIAS => return concat!(stringify!(WINED3D_RS_SLOPESCALEDEPTHBIAS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ANTIALIASEDLINEENABLE => return concat!(stringify!(WINED3D_RS_ANTIALIASEDLINEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MINTESSELLATIONLEVEL => return concat!(stringify!(WINED3D_RS_MINTESSELLATIONLEVEL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_MAXTESSELLATIONLEVEL => return concat!(stringify!(WINED3D_RS_MAXTESSELLATIONLEVEL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ADAPTIVETESS_X => return concat!(stringify!(WINED3D_RS_ADAPTIVETESS_X), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ADAPTIVETESS_Y => return concat!(stringify!(WINED3D_RS_ADAPTIVETESS_Y), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ADAPTIVETESS_Z => return concat!(stringify!(WINED3D_RS_ADAPTIVETESS_Z), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ADAPTIVETESS_W => return concat!(stringify!(WINED3D_RS_ADAPTIVETESS_W), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_ENABLEADAPTIVETESSELLATION => return concat!(stringify!(WINED3D_RS_ENABLEADAPTIVETESSELLATION), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_TWOSIDEDSTENCILMODE => return concat!(stringify!(WINED3D_RS_TWOSIDEDSTENCILMODE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BACK_STENCILFAIL => return concat!(stringify!(WINED3D_RS_BACK_STENCILFAIL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BACK_STENCILZFAIL => return concat!(stringify!(WINED3D_RS_BACK_STENCILZFAIL), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BACK_STENCILPASS => return concat!(stringify!(WINED3D_RS_BACK_STENCILPASS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BACK_STENCILFUNC => return concat!(stringify!(WINED3D_RS_BACK_STENCILFUNC), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORWRITEENABLE1 => return concat!(stringify!(WINED3D_RS_COLORWRITEENABLE1), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORWRITEENABLE2 => return concat!(stringify!(WINED3D_RS_COLORWRITEENABLE2), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_COLORWRITEENABLE3 => return concat!(stringify!(WINED3D_RS_COLORWRITEENABLE3), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SRGBWRITEENABLE => return concat!(stringify!(WINED3D_RS_SRGBWRITEENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DEPTHBIAS => return concat!(stringify!(WINED3D_RS_DEPTHBIAS), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP8 => return concat!(stringify!(WINED3D_RS_WRAP8), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP9 => return concat!(stringify!(WINED3D_RS_WRAP9), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP10 => return concat!(stringify!(WINED3D_RS_WRAP10), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP11 => return concat!(stringify!(WINED3D_RS_WRAP11), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP12 => return concat!(stringify!(WINED3D_RS_WRAP12), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP13 => return concat!(stringify!(WINED3D_RS_WRAP13), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP14 => return concat!(stringify!(WINED3D_RS_WRAP14), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_WRAP15 => return concat!(stringify!(WINED3D_RS_WRAP15), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SEPARATEALPHABLENDENABLE => return concat!(stringify!(WINED3D_RS_SEPARATEALPHABLENDENABLE), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_SRCBLENDALPHA => return concat!(stringify!(WINED3D_RS_SRCBLENDALPHA), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_DESTBLENDALPHA => return concat!(stringify!(WINED3D_RS_DESTBLENDALPHA), "\0").as_ptr() as *const core::ffi::c_char,
        WINED3D_RS_BLENDOPALPHA => return concat!(stringify!(WINED3D_RS_BLENDOPALPHA), "\0").as_ptr() as *const core::ffi::c_char,
        _ => {
            FIXME!("Unrecognized %u render state!\n", state as core::ffi::c_uint);
            return b"unrecognized\0".as_ptr() as *const core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dsamplerstate(
    state: wined3d_sampler_state,
) -> *const ::core::ffi::c_char {
    match state {
        WINED3D_SAMP_BORDER_COLOR => {
            return concat!(stringify!(WINED3D_SAMP_BORDER_COLOR), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_ADDRESS_U => {
            return concat!(stringify!(WINED3D_SAMP_ADDRESS_U), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_ADDRESS_V => {
            return concat!(stringify!(WINED3D_SAMP_ADDRESS_V), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_ADDRESS_W => {
            return concat!(stringify!(WINED3D_SAMP_ADDRESS_W), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MAG_FILTER => {
            return concat!(stringify!(WINED3D_SAMP_MAG_FILTER), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MIN_FILTER => {
            return concat!(stringify!(WINED3D_SAMP_MIN_FILTER), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MIP_FILTER => {
            return concat!(stringify!(WINED3D_SAMP_MIP_FILTER), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MIPMAP_LOD_BIAS => {
            return concat!(stringify!(WINED3D_SAMP_MIPMAP_LOD_BIAS), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MAX_MIP_LEVEL => {
            return concat!(stringify!(WINED3D_SAMP_MAX_MIP_LEVEL), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_MAX_ANISOTROPY => {
            return concat!(stringify!(WINED3D_SAMP_MAX_ANISOTROPY), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_SRGB_TEXTURE => {
            return concat!(stringify!(WINED3D_SAMP_SRGB_TEXTURE), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_ELEMENT_INDEX => {
            return concat!(stringify!(WINED3D_SAMP_ELEMENT_INDEX), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_SAMP_DMAP_OFFSET => {
            return concat!(stringify!(WINED3D_SAMP_DMAP_OFFSET), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized %u sampler state!\n", state as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dtexturefiltertype(filter_type: wined3d_texture_filter_type) -> *const ::core::ffi::c_char {
    match filter_type as ::core::ffi::c_uint {
        WINED3D_TEXF_NONE => {
            return concat!(stringify!(WINED3D_TEXF_NONE), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_POINT => {
            return concat!(stringify!(WINED3D_TEXF_POINT), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_LINEAR => {
            return concat!(stringify!(WINED3D_TEXF_LINEAR), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_ANISOTROPIC => {
            return concat!(stringify!(WINED3D_TEXF_ANISOTROPIC), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_FLAT_CUBIC => {
            return concat!(stringify!(WINED3D_TEXF_FLAT_CUBIC), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_GAUSSIAN_CUBIC => {
            return concat!(stringify!(WINED3D_TEXF_GAUSSIAN_CUBIC), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_PYRAMIDAL_QUAD => {
            return concat!(stringify!(WINED3D_TEXF_PYRAMIDAL_QUAD), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_TEXF_GAUSSIAN_QUAD => {
            return concat!(stringify!(WINED3D_TEXF_GAUSSIAN_QUAD), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                FIXME!(
                    "Unrecognized texture filter type 0x%08x.\n",
                    filter_type as ::core::ffi::c_uint
                );
            }
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dtexturestate(state: wined3d_texture_stage_state) -> *const ::core::ffi::c_char {
    match state {
        WINED3D_TSS_COLOR_OP => return b"WINED3D_TSS_COLOR_OP\0".as_ptr() as *const i8,
        WINED3D_TSS_COLOR_ARG1 => return b"WINED3D_TSS_COLOR_ARG1\0".as_ptr() as *const i8,
        WINED3D_TSS_COLOR_ARG2 => return b"WINED3D_TSS_COLOR_ARG2\0".as_ptr() as *const i8,
        WINED3D_TSS_ALPHA_OP => return b"WINED3D_TSS_ALPHA_OP\0".as_ptr() as *const i8,
        WINED3D_TSS_ALPHA_ARG1 => return b"WINED3D_TSS_ALPHA_ARG1\0".as_ptr() as *const i8,
        WINED3D_TSS_ALPHA_ARG2 => return b"WINED3D_TSS_ALPHA_ARG2\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_MAT00 => return b"WINED3D_TSS_BUMPENV_MAT00\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_MAT01 => return b"WINED3D_TSS_BUMPENV_MAT01\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_MAT10 => return b"WINED3D_TSS_BUMPENV_MAT10\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_MAT11 => return b"WINED3D_TSS_BUMPENV_MAT11\0".as_ptr() as *const i8,
        WINED3D_TSS_TEXCOORD_INDEX => return b"WINED3D_TSS_TEXCOORD_INDEX\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_LSCALE => return b"WINED3D_TSS_BUMPENV_LSCALE\0".as_ptr() as *const i8,
        WINED3D_TSS_BUMPENV_LOFFSET => return b"WINED3D_TSS_BUMPENV_LOFFSET\0".as_ptr() as *const i8,
        WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS => return b"WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS\0".as_ptr() as *const i8,
        WINED3D_TSS_COLOR_ARG0 => return b"WINED3D_TSS_COLOR_ARG0\0".as_ptr() as *const i8,
        WINED3D_TSS_ALPHA_ARG0 => return b"WINED3D_TSS_ALPHA_ARG0\0".as_ptr() as *const i8,
        WINED3D_TSS_RESULT_ARG => return b"WINED3D_TSS_RESULT_ARG\0".as_ptr() as *const i8,
        WINED3D_TSS_CONSTANT => return b"WINED3D_TSS_CONSTANT\0".as_ptr() as *const i8,
        _ => {
            FIXME!("Unrecognized %u texture state!\n", state);
            return b"unrecognized\0".as_ptr() as *const i8;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dtop(mut d3dtop: wined3d_texture_op) -> *const ::core::ffi::c_char {
    match d3dtop {
        WINED3D_TOP_DISABLE => return concat!(stringify!(WINED3D_TOP_DISABLE), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_SELECT_ARG1 => return concat!(stringify!(WINED3D_TOP_SELECT_ARG1), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_SELECT_ARG2 => return concat!(stringify!(WINED3D_TOP_SELECT_ARG2), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE => return concat!(stringify!(WINED3D_TOP_MODULATE), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_2X => return concat!(stringify!(WINED3D_TOP_MODULATE_2X), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_4X => return concat!(stringify!(WINED3D_TOP_MODULATE_4X), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_ADD => return concat!(stringify!(WINED3D_TOP_ADD), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_ADD_SIGNED => return concat!(stringify!(WINED3D_TOP_ADD_SIGNED), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_ADD_SIGNED_2X => return concat!(stringify!(WINED3D_TOP_ADD_SIGNED_2X), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_SUBTRACT => return concat!(stringify!(WINED3D_TOP_SUBTRACT), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_ADD_SMOOTH => return concat!(stringify!(WINED3D_TOP_ADD_SMOOTH), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BLEND_DIFFUSE_ALPHA => return concat!(stringify!(WINED3D_TOP_BLEND_DIFFUSE_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BLEND_TEXTURE_ALPHA => return concat!(stringify!(WINED3D_TOP_BLEND_TEXTURE_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BLEND_FACTOR_ALPHA => return concat!(stringify!(WINED3D_TOP_BLEND_FACTOR_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM => return concat!(stringify!(WINED3D_TOP_BLEND_TEXTURE_ALPHA_PM), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BLEND_CURRENT_ALPHA => return concat!(stringify!(WINED3D_TOP_BLEND_CURRENT_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_PREMODULATE => return concat!(stringify!(WINED3D_TOP_PREMODULATE), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_ALPHA_ADD_COLOR => return concat!(stringify!(WINED3D_TOP_MODULATE_ALPHA_ADD_COLOR), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_COLOR_ADD_ALPHA => return concat!(stringify!(WINED3D_TOP_MODULATE_COLOR_ADD_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_INVALPHA_ADD_COLOR => return concat!(stringify!(WINED3D_TOP_MODULATE_INVALPHA_ADD_COLOR), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MODULATE_INVCOLOR_ADD_ALPHA => return concat!(stringify!(WINED3D_TOP_MODULATE_INVCOLOR_ADD_ALPHA), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BUMPENVMAP => return concat!(stringify!(WINED3D_TOP_BUMPENVMAP), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_BUMPENVMAP_LUMINANCE => return concat!(stringify!(WINED3D_TOP_BUMPENVMAP_LUMINANCE), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_DOTPRODUCT3 => return concat!(stringify!(WINED3D_TOP_DOTPRODUCT3), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_MULTIPLY_ADD => return concat!(stringify!(WINED3D_TOP_MULTIPLY_ADD), "\0").as_ptr() as *const ::core::ffi::c_char,
        WINED3D_TOP_LERP => return concat!(stringify!(WINED3D_TOP_LERP), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {
            WINE_FIXME!("Unrecognized texture op %#x.\n", d3dtop as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dtstype(mut tstype: wined3d_transform_state) -> *const ::core::ffi::c_char {
    match tstype as ::core::ffi::c_uint {
        x if x == WINED3D_TS_VIEW as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_VIEW), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_PROJECTION as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_PROJECTION), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE0 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE0), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE1 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE1), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE2 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE2), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE3 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE3), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE4 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE4), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE5 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE5), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE6 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE6), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_TEXTURE7 as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_TEXTURE7), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_WORLD_MATRIX!(0) as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_WORLD_MATRIX!(0)), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_WORLD_MATRIX!(1) as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_WORLD_MATRIX!(1)), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_WORLD_MATRIX!(2) as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_WORLD_MATRIX!(2)), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        x if x == WINED3D_TS_WORLD_MATRIX!(3) as ::core::ffi::c_uint => {
            return concat!(stringify!(WINED3D_TS_WORLD_MATRIX!(3)), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            if (tstype as ::core::ffi::c_uint) > 256 as ::core::ffi::c_uint
                && (tstype as ::core::ffi::c_uint) < 512 as ::core::ffi::c_uint
            {
                WINE_FIXME!("WINED3D_TS_WORLD_MATRIX(%u). 1..255 not currently supported.\n", tstype as ::core::ffi::c_uint);
                return b"WINED3D_TS_WORLD_MATRIX > 0\0" as *const u8 as *const ::core::ffi::c_char;
            }
            WINE_FIXME!("Unrecognized transform state %#x.\n", tstype as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_shader_type(type_: wined3d_shader_type) -> *const ::core::ffi::c_char {
    match type_ {
        WINED3D_SHADER_TYPE_PIXEL => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_PIXEL), "\0").as_ptr() as *const core::ffi::c_char;
        }
        WINED3D_SHADER_TYPE_VERTEX => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_VERTEX), "\0").as_ptr() as *const core::ffi::c_char;
        }
        WINED3D_SHADER_TYPE_GEOMETRY => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_GEOMETRY), "\0").as_ptr() as *const core::ffi::c_char;
        }
        WINED3D_SHADER_TYPE_HULL => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_HULL), "\0").as_ptr() as *const core::ffi::c_char;
        }
        WINED3D_SHADER_TYPE_DOMAIN => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_DOMAIN), "\0").as_ptr() as *const core::ffi::c_char;
        }
        WINED3D_SHADER_TYPE_COMPUTE => {
            return concat!(stringify!(WINED3D_SHADER_TYPE_COMPUTE), "\0").as_ptr() as *const core::ffi::c_char;
        }
        _ => {
            FIXME!("Unrecognized shader type %#x.\n", type_ as u32);
            return b"unrecognized\0".as_ptr() as *const core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dstate(state: uint32_t) -> *const ::core::ffi::c_char {
    if STATE_IS_RENDER!(state) {
        return wine_dbg_sprintf(
            b"STATE_RENDER(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            debug_d3drenderstate(state.wrapping_sub(STATE_RENDER!(0))),
        );
    }
    if STATE_IS_TEXTURESTAGE!(state) {
        let texture_stage: ::core::ffi::c_uint = state.wrapping_sub(
            (STATE_TEXTURESTAGE!(0, 0)) as u32,
        ) / ((WINED3D_HIGHEST_TEXTURE_STATE + 1)) as u32;
        let texture_state: uint32_t = state.wrapping_sub((STATE_TEXTURESTAGE!(
            texture_stage,
            0,
        )) as u32);
        return wine_dbg_sprintf(
            b"STATE_TEXTURESTAGE(%#x, %s)\0".as_ptr() as *const ::core::ffi::c_char,
            texture_stage,
            debug_d3dtexturestate(texture_state),
        );
    }
    if STATE_IS_COMPUTE_SHADER!(state) {
        return wine_dbg_sprintf(
            b"STATE_SHADER(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            debug_shader_type(WINED3D_SHADER_TYPE_COMPUTE),
        );
    }
    if STATE_IS_GRAPHICS_SHADER!(state) {
        return wine_dbg_sprintf(
            b"STATE_SHADER(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            debug_shader_type(state.wrapping_sub(STATE_SHADER!(0))),
        );
    }
    if STATE_IS_COMPUTE_CONSTANT_BUFFER!(state) {
        return wine_dbg_sprintf(
            b"STATE_CONSTANT_BUFFER(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            debug_shader_type(WINED3D_SHADER_TYPE_COMPUTE),
        );
    }
    if STATE_IS_GRAPHICS_CONSTANT_BUFFER!(state) {
        return wine_dbg_sprintf(
            b"STATE_CONSTANT_BUFFER(%s)\0".as_ptr() as *const ::core::ffi::c_char,
            debug_shader_type(state.wrapping_sub(STATE_CONSTANT_BUFFER!(0))),
        );
    }
    if STATE_IS_COMPUTE_SHADER_RESOURCE_BINDING!(state) {
        return b"STATE_COMPUTE_SHADER_RESOURCE_BINDING\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_GRAPHICS_SHADER_RESOURCE_BINDING!(state) {
        return b"STATE_GRAPHICS_SHADER_RESOURCE_BINDING\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING!(state) {
        return b"STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING!(state) {
        return b"STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_STREAMSRC!(state) {
        return b"STATE_STREAMSRC\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_INDEXBUFFER!(state) {
        return b"STATE_INDEXBUFFER\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_VDECL!(state) {
        return b"STATE_VDECL\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_VIEWPORT!(state) {
        return b"STATE_VIEWPORT\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_SCISSORRECT!(state) {
        return b"STATE_SCISSORRECT\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_CLIPPLANE!(state) {
        return wine_dbg_sprintf(
            b"STATE_CLIPPLANE(%#x)\0".as_ptr() as *const ::core::ffi::c_char,
            state.wrapping_sub((STATE_CLIPPLANE!(0)) as u32),
        );
    }
    if STATE_IS_RASTERIZER!(state) {
        return b"STATE_RASTERIZER\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_DEPTH_BOUNDS!(state) {
        return b"STATE_DEPTH_BOUNDS\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_BASEVERTEXINDEX!(state) {
        return b"STATE_BASEVERTEXINDEX\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_FRAMEBUFFER!(state) {
        return b"STATE_FRAMEBUFFER\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_STREAM_OUTPUT!(state) {
        return b"STATE_STREAM_OUTPUT\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_BLEND!(state) {
        return b"STATE_BLEND\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_BLEND_FACTOR!(state) {
        return b"STATE_BLEND_FACTOR\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_SAMPLE_MASK!(state) {
        return b"STATE_SAMPLE_MASK\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_DEPTH_STENCIL!(state) {
        return b"STATE_DEPTH_STENCIL\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if STATE_IS_STENCIL_REF!(state) {
        return b"STATE_STENCIL_REF\0".as_ptr() as *const ::core::ffi::c_char;
    }

    return wine_dbg_sprintf(
        b"UNKNOWN_STATE(%#x)\0".as_ptr() as *const ::core::ffi::c_char,
        state,
    );
}
#[no_mangle]
pub unsafe extern "C" fn debug_fboattachment(mut attachment: GLenum) -> *const ::core::ffi::c_char {
    match attachment {
        GL_COLOR_ATTACHMENT0 => {
            return b"GL_COLOR_ATTACHMENT0\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT1 => {
            return b"GL_COLOR_ATTACHMENT1\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT2 => {
            return b"GL_COLOR_ATTACHMENT2\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT3 => {
            return b"GL_COLOR_ATTACHMENT3\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT4 => {
            return b"GL_COLOR_ATTACHMENT4\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT5 => {
            return b"GL_COLOR_ATTACHMENT5\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT6 => {
            return b"GL_COLOR_ATTACHMENT6\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT7 => {
            return b"GL_COLOR_ATTACHMENT7\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT8 => {
            return b"GL_COLOR_ATTACHMENT8\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT9 => {
            return b"GL_COLOR_ATTACHMENT9\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT10 => {
            return b"GL_COLOR_ATTACHMENT10\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT11 => {
            return b"GL_COLOR_ATTACHMENT11\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT12 => {
            return b"GL_COLOR_ATTACHMENT12\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT13 => {
            return b"GL_COLOR_ATTACHMENT13\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT14 => {
            return b"GL_COLOR_ATTACHMENT14\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_COLOR_ATTACHMENT15 => {
            return b"GL_COLOR_ATTACHMENT15\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_DEPTH_ATTACHMENT => {
            return b"GL_DEPTH_ATTACHMENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        GL_STENCIL_ATTACHMENT => {
            return b"GL_STENCIL_ATTACHMENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            return wine_dbg_sprintf(
                b"Unknown FBO attachment %#x\0" as *const u8 as *const ::core::ffi::c_char,
                attachment,
            );
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_fbostatus(mut status: GLenum) -> *const ::core::ffi::c_char {
    match status {
        GL_FRAMEBUFFER_COMPLETE => {
            return concat!(stringify!(GL_FRAMEBUFFER_COMPLETE), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS_EXT => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS_EXT), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_FORMATS_EXT => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_FORMATS_EXT), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_READ_BUFFER), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_MULTISAMPLE), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_INCOMPLETE_LAYER_COUNT_ARB => {
            return concat!(stringify!(GL_FRAMEBUFFER_INCOMPLETE_LAYER_COUNT_ARB), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_UNSUPPORTED => {
            return concat!(stringify!(GL_FRAMEBUFFER_UNSUPPORTED), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        GL_FRAMEBUFFER_UNDEFINED => {
            return concat!(stringify!(GL_FRAMEBUFFER_UNDEFINED), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            WINE_FIXME!("Unrecognized FBO status 0x%08x.\n", status);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn debug_glerror(error: GLenum) -> *const ::core::ffi::c_char {
    match error {
        GL_NO_ERROR => return concat!(stringify!(GL_NO_ERROR), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_INVALID_ENUM => return concat!(stringify!(GL_INVALID_ENUM), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_INVALID_VALUE => return concat!(stringify!(GL_INVALID_VALUE), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_INVALID_OPERATION => return concat!(stringify!(GL_INVALID_OPERATION), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_STACK_OVERFLOW => return concat!(stringify!(GL_STACK_OVERFLOW), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_STACK_UNDERFLOW => return concat!(stringify!(GL_STACK_UNDERFLOW), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_OUT_OF_MEMORY => return concat!(stringify!(GL_OUT_OF_MEMORY), "\0").as_ptr() as *const ::core::ffi::c_char,
        GL_INVALID_FRAMEBUFFER_OPERATION => return concat!(stringify!(GL_INVALID_FRAMEBUFFER_OPERATION), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {
            WINE_FIXME!("Unrecognized GL error 0x%08x.\n", error);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_vkresult(vr: VkResult) -> *const ::core::ffi::c_char {
    match vr as ::core::ffi::c_int {
        VK_ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR => {
            return b"VK_ERROR_INVALID_OPAQUE_CAPTURE_ADDRESS_KHR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_NOT_PERMITTED_EXT => {
            return b"VK_ERROR_NOT_PERMITTED_EXT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_FRAGMENTATION_EXT => {
            return b"VK_ERROR_FRAGMENTATION_EXT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_INVALID_EXTERNAL_HANDLE => {
            return b"VK_ERROR_INVALID_EXTERNAL_HANDLE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_OUT_OF_POOL_MEMORY => {
            return b"VK_ERROR_OUT_OF_POOL_MEMORY\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_INVALID_SHADER_NV => {
            return b"VK_ERROR_INVALID_SHADER_NV\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_OUT_OF_DATE_KHR => {
            return b"VK_ERROR_OUT_OF_DATE_KHR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_NATIVE_WINDOW_IN_USE_KHR => {
            return b"VK_ERROR_NATIVE_WINDOW_IN_USE_KHR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_SURFACE_LOST_KHR => {
            return b"VK_ERROR_SURFACE_LOST_KHR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_FRAGMENTED_POOL => {
            return b"VK_ERROR_FRAGMENTED_POOL\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_FORMAT_NOT_SUPPORTED => {
            return b"VK_ERROR_FORMAT_NOT_SUPPORTED\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_TOO_MANY_OBJECTS => {
            return b"VK_ERROR_TOO_MANY_OBJECTS\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_INCOMPATIBLE_DRIVER => {
            return b"VK_ERROR_INCOMPATIBLE_DRIVER\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_FEATURE_NOT_PRESENT => {
            return b"VK_ERROR_FEATURE_NOT_PRESENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_EXTENSION_NOT_PRESENT => {
            return b"VK_ERROR_EXTENSION_NOT_PRESENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_LAYER_NOT_PRESENT => {
            return b"VK_ERROR_LAYER_NOT_PRESENT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_MEMORY_MAP_FAILED => {
            return b"VK_ERROR_MEMORY_MAP_FAILED\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_DEVICE_LOST => {
            return b"VK_ERROR_DEVICE_LOST\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_INITIALIZATION_FAILED => {
            return b"VK_ERROR_INITIALIZATION_FAILED\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_OUT_OF_DEVICE_MEMORY => {
            return b"VK_ERROR_OUT_OF_DEVICE_MEMORY\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_ERROR_OUT_OF_HOST_MEMORY => {
            return b"VK_ERROR_OUT_OF_HOST_MEMORY\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_SUCCESS => {
            return b"VK_SUCCESS\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_NOT_READY => {
            return b"VK_NOT_READY\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_TIMEOUT => {
            return b"VK_TIMEOUT\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_EVENT_SET => {
            return b"VK_EVENT_SET\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_EVENT_RESET => {
            return b"VK_EVENT_RESET\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_INCOMPLETE => {
            return b"VK_INCOMPLETE\0" as *const u8 as *const ::core::ffi::c_char;
        }
        VK_SUBOPTIMAL_KHR => {
            return b"VK_SUBOPTIMAL_KHR\0" as *const u8 as *const ::core::ffi::c_char;
        }
        _ => {
            return wine_dbg_sprintf(
                b"unrecognised(%d)\0" as *const u8 as *const ::core::ffi::c_char,
                vr as ::core::ffi::c_int,
            );
        }
    }
}

unsafe extern "C" fn debug_fixup_channel_source(
    source: fixup_channel_source,
) -> *const core::ffi::c_char {
    match source {
        CHANNEL_SOURCE_ZERO => return concat!(stringify!(CHANNEL_SOURCE_ZERO), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_ONE => return concat!(stringify!(CHANNEL_SOURCE_ONE), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_X => return concat!(stringify!(CHANNEL_SOURCE_X), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_Y => return concat!(stringify!(CHANNEL_SOURCE_Y), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_Z => return concat!(stringify!(CHANNEL_SOURCE_Z), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_W => return concat!(stringify!(CHANNEL_SOURCE_W), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_COMPLEX0 => return concat!(stringify!(CHANNEL_SOURCE_COMPLEX0), "\0").as_ptr() as *const core::ffi::c_char,
        CHANNEL_SOURCE_COMPLEX1 => return concat!(stringify!(CHANNEL_SOURCE_COMPLEX1), "\0").as_ptr() as *const core::ffi::c_char,
        _ => {
            FIXME!(
                "Unrecognized fixup_channel_source %#x\n",
                source as core::ffi::c_uint
            );
            return concat!("unrecognized", "\0").as_ptr() as *const core::ffi::c_char;
        }
    }
}

unsafe extern "C" fn debug_complex_fixup(mut fixup: complex_fixup) -> *const ::core::ffi::c_char {
    match fixup {
        COMPLEX_FIXUP_YUY2 => return concat!(stringify!(COMPLEX_FIXUP_YUY2), "\0").as_ptr() as *const ::core::ffi::c_char,
        COMPLEX_FIXUP_UYVY => return concat!(stringify!(COMPLEX_FIXUP_UYVY), "\0").as_ptr() as *const ::core::ffi::c_char,
        COMPLEX_FIXUP_YV12 => return concat!(stringify!(COMPLEX_FIXUP_YV12), "\0").as_ptr() as *const ::core::ffi::c_char,
        COMPLEX_FIXUP_NV12 => return concat!(stringify!(COMPLEX_FIXUP_NV12), "\0").as_ptr() as *const ::core::ffi::c_char,
        COMPLEX_FIXUP_P8 => return concat!(stringify!(COMPLEX_FIXUP_P8), "\0").as_ptr() as *const ::core::ffi::c_char,
        COMPLEX_FIXUP_YUV => return concat!(stringify!(COMPLEX_FIXUP_YUV), "\0").as_ptr() as *const ::core::ffi::c_char,
        _ => {
            WINE_FIXME!("Unrecognized complex fixup %#x\n", fixup as ::core::ffi::c_uint);
            return b"unrecognized\0" as *const u8 as *const ::core::ffi::c_char;
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn dump_color_fixup_desc(mut fixup: color_fixup_desc) {
    if is_complex_fixup(fixup) != 0 {
        WINE_TRACE!("\tComplex: %s\n", debug_complex_fixup(get_complex_fixup(fixup)));
        return;
    }
    WINE_TRACE!("\tX: %s%s\n", debug_fixup_channel_source(fixup.x_source() as fixup_channel_source), if fixup.x_sign_fixup() as ::core::ffi::c_int != 0 { b", SIGN_FIXUP\0" as *const u8 as *const ::core::ffi::c_char } else { b"\0" as *const u8 as *const ::core::ffi::c_char });
    WINE_TRACE!("\tY: %s%s\n", debug_fixup_channel_source(fixup.y_source() as fixup_channel_source), if fixup.y_sign_fixup() as ::core::ffi::c_int != 0 { b", SIGN_FIXUP\0" as *const u8 as *const ::core::ffi::c_char } else { b"\0" as *const u8 as *const ::core::ffi::c_char });
    WINE_TRACE!("\tZ: %s%s\n", debug_fixup_channel_source(fixup.z_source() as fixup_channel_source), if fixup.z_sign_fixup() as ::core::ffi::c_int != 0 { b", SIGN_FIXUP\0" as *const u8 as *const ::core::ffi::c_char } else { b"\0" as *const u8 as *const ::core::ffi::c_char });
    WINE_TRACE!("\tW: %s%s\n", debug_fixup_channel_source(fixup.w_source() as fixup_channel_source), if fixup.w_sign_fixup() as ::core::ffi::c_int != 0 { b", SIGN_FIXUP\0" as *const u8 as *const ::core::ffi::c_char } else { b"\0" as *const u8 as *const ::core::ffi::c_char });
}
#[no_mangle]
pub unsafe extern "C" fn is_invalid_op(state: *const wined3d_state, stage: ::core::ffi::c_int, op: wined3d_texture_op, arg1: DWORD, arg2: DWORD, arg3: DWORD) -> BOOL {
    if op as ::core::ffi::c_uint == WINED3D_TOP_DISABLE as ::core::ffi::c_uint {
        return (FALSE) as BOOL;
    }
    if !wined3d_state_get_ffp_texture(state, stage as ::core::ffi::c_uint).is_null() {
        return (FALSE) as BOOL;
    }

    if (arg1 & WINED3DTA_SELECTMASK as DWORD) == WINED3DTA_TEXTURE as DWORD
        && op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG2 as ::core::ffi::c_uint
    {
        return (TRUE) as BOOL;
    }
    if (arg2 & WINED3DTA_SELECTMASK as DWORD) == WINED3DTA_TEXTURE as DWORD
        && op as ::core::ffi::c_uint != WINED3D_TOP_SELECT_ARG1 as ::core::ffi::c_uint
    {
        return (TRUE) as BOOL;
    }
    if (arg3 & WINED3DTA_SELECTMASK as DWORD) == WINED3DTA_TEXTURE as DWORD
        && (op as ::core::ffi::c_uint == WINED3D_TOP_MULTIPLY_ADD as ::core::ffi::c_uint
            || op as ::core::ffi::c_uint == WINED3D_TOP_LERP as ::core::ffi::c_uint)
    {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn get_identity_matrix(mat: *mut wined3d_matrix) {
    static identity: wined3d_matrix = wined3d_matrix {
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
    *mat = identity;
}
#[no_mangle]
pub unsafe extern "C" fn get_modelview_matrix(state: *const wined3d_stateblock_state, index: ::core::ffi::c_uint, mat: *mut wined3d_matrix) {
    multiply_matrix(
        mat,
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_VIEW as usize]),
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_WORLD_MATRIX!(index) as usize]),
    );
}

unsafe extern "C" fn compute_texture_matrix(
    matrix: *const wined3d_matrix,
    flags: uint32_t,
    calculated_coords: BOOL,
    format_id: wined3d_format_id,
    out_matrix: *mut wined3d_matrix,
) {
    let mut mat: wined3d_matrix = core::mem::zeroed();

    if flags == (WINED3D_TTFF_DISABLE) as uint32_t || flags == (WINED3D_TTFF_COUNT1) as uint32_t {
        get_identity_matrix(out_matrix);
        return;
    }

    if flags == ((WINED3D_TTFF_COUNT1) as uint32_t | (WINED3D_TTFF_PROJECTED) as uint32_t) {
        ERR!("Invalid texture transform flags: WINED3D_TTFF_COUNT1 | WINED3D_TTFF_PROJECTED.\n");
        return;
    }

    mat = *matrix;

    if (flags & (WINED3D_TTFF_PROJECTED) as uint32_t) == 0 && calculated_coords == 0 {
        match format_id as ::core::ffi::c_uint {
            WINED3DFMT_R32_FLOAT => {
                mat._41 = mat._21;
                mat._42 = mat._22;
                mat._43 = mat._23;
                mat._44 = mat._24;
            }
            WINED3DFMT_R32G32_FLOAT => {
                mat._41 = mat._31;
                mat._42 = mat._32;
                mat._43 = mat._33;
                mat._44 = mat._34;
            }
            WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_UNKNOWN => {}
            _ => {
                FIXME!("Unexpected fixed function texture coord input\n");
            }
        }
    }

    *out_matrix = mat;
}

unsafe extern "C" fn get_texcoord_format(
    decl: *const wined3d_vertex_declaration,
    index: ::core::ffi::c_uint,
) -> wined3d_format_id {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < (*decl).element_count {
        if (*(*decl).elements.offset(i as isize)).usage as ::core::ffi::c_int
            == WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int
            && (*(*decl).elements.offset(i as isize)).usage_idx as ::core::ffi::c_uint == index
        {
            return (*(*(*decl).elements.offset(i as isize)).format).id;
        }
        i = i.wrapping_add(1);
    }

    return WINED3DFMT_UNKNOWN;
}
#[no_mangle]
pub unsafe extern "C" fn get_texture_matrix(state: *const wined3d_stateblock_state, tex: ::core::ffi::c_uint, mat: *mut wined3d_matrix) {
    let generated: BOOL = ((*state).texture_states[tex as usize][WINED3D_TSS_TEXCOORD_INDEX as usize]
        & 0xffff0000 as u32
        != WINED3DTSS_TCI_PASSTHRU as u32) as ::core::ffi::c_int;
    let coord_idx: ::core::ffi::c_uint = min!(
        (*state).texture_states[tex as usize][WINED3D_TSS_TEXCOORD_INDEX as usize]
            & 0x0000ffff as u32,
        WINED3D_MAX_FFP_TEXTURES as u32 - 1 as u32,
    );

    compute_texture_matrix(
        core::ptr::addr_of!(
            (*state).transforms[(WINED3D_TS_TEXTURE0 as usize) + tex as usize]
        ) as *const wined3d_matrix,
        (*state).texture_states[tex as usize][WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS as usize],
        generated,
        get_texcoord_format((*state).vertex_declaration, coord_idx),
        mat,
    );
}
#[no_mangle]
pub unsafe extern "C" fn get_pointsize_minmax(context: *const wined3d_context, state: *const wined3d_state, out_min: *mut ::core::ffi::c_float, out_max: *mut ::core::ffi::c_float) {
    #[repr(C)]
    union MinMax {
        d: DWORD,
        f: f32,
    }
    let mut min: MinMax = MinMax { d: 0 };
    let mut max: MinMax = MinMax { d: 0 };
    min.d = (*state).render_states[WINED3D_RS_POINTSIZE_MIN as usize] as DWORD;
    max.d = (*state).render_states[WINED3D_RS_POINTSIZE_MAX as usize] as DWORD;

    if min.f > max.f {
        min.f = max.f;
    }

    *out_min = min.f;
    *out_max = max.f;
}
#[no_mangle]
pub unsafe extern "C" fn get_fog_start_end(context: *const wined3d_context, state: *const wined3d_state, start: *mut ::core::ffi::c_float, end: *mut ::core::ffi::c_float) {
    #[repr(C)]
    union TmpValue {
        d: DWORD,
        f: f32,
    }
    let mut tmpvalue: TmpValue = TmpValue { d: 0 };
    match (*context).fog_source as ::core::ffi::c_uint {
        FOGSOURCE_VS => {
            *start = 1.0f32;
            *end = 0.0f32;
        }
        FOGSOURCE_FFP => {
            tmpvalue.d = (*state).render_states[WINED3D_RS_FOGSTART as ::core::ffi::c_int as usize] as DWORD;
            *start = tmpvalue.f;
            tmpvalue.d = (*state).render_states[WINED3D_RS_FOGEND as ::core::ffi::c_int as usize] as DWORD;
            *end = tmpvalue.f;
            /* Special handling for fog_start == fog_end. In d3d with vertex
             * fog, everything is fogged. With table fog, everything with
             * fog_coord < fog_start is unfogged, and fog_coord > fog_start
             * is fogged. Windows drivers disagree when fog_coord == fog_start. */
            if (*state).render_states[WINED3D_RS_FOGTABLEMODE as ::core::ffi::c_int as usize]
                == WINED3D_FOG_NONE as ::core::ffi::c_uint
                && *start == *end
            {
                *start = -INFINITY;
                *end = 0.0f32;
            }
        }
        _ => {
            /* This should not happen, context->fog_source is set in wined3d, not the app. */
            ERR!("Unexpected fog coordinate source.\n");
            *start = 0.0f32;
            *end = 0.0f32;
        }
    }
}

unsafe extern "C" fn wined3d_get_primary_display(display: *mut WCHAR) -> BOOL {
    let mut display_device: DISPLAY_DEVICEW = core::mem::zeroed();
    let mut device_idx: DWORD = 0;

    display_device.cb = core::mem::size_of::<DISPLAY_DEVICEW>() as DWORD;
    while (EnumDisplayDevicesW(
        core::ptr::null(),
        {
            let fresh = device_idx;
            device_idx = device_idx.wrapping_add(1);
            fresh
        },
        core::ptr::addr_of_mut!(display_device),
        0,
    ) != 0)
    {
        if (display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0 {
            lstrcpyW(
                display as LPWSTR,
                core::ptr::addr_of!(display_device.DeviceName) as *const WCHAR as LPCWSTR,
            );
            return (TRUE) as BOOL;
        }
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_get_primary_adapter_luid(luid: *mut LUID) -> BOOL {
    let mut open_adapter_param: D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME = core::mem::zeroed();
    let mut close_adapter_param: D3DKMT_CLOSEADAPTER = core::mem::zeroed();

    if wined3d_get_primary_display(core::ptr::addr_of_mut!(open_adapter_param.DeviceName) as *mut WCHAR) == 0 {
        return (FALSE) as BOOL;
    }

    if D3DKMTOpenAdapterFromGdiDisplayName(core::ptr::addr_of_mut!(open_adapter_param)) != 0 {
        return (FALSE) as BOOL;
    }

    *luid = open_adapter_param.AdapterLuid;
    close_adapter_param.hAdapter = open_adapter_param.hAdapter;
    D3DKMTCloseAdapter(core::ptr::addr_of_mut!(close_adapter_param));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_pack(format: *const wined3d_format, value: *const wined3d_uvec4) -> uint32_t {
    let mut p: uint32_t = 0;

    p |= ((*value).x & wined3d_mask_from_size((*format).red_size)) << (*format).red_offset;
    p |= ((*value).y & wined3d_mask_from_size((*format).green_size)) << (*format).green_offset;
    p |= ((*value).z & wined3d_mask_from_size((*format).blue_size)) << (*format).blue_offset;
    p |= ((*value).w & wined3d_mask_from_size((*format).alpha_size)) << (*format).alpha_offset;

    return p;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_colour_to_vk(
    format: *const wined3d_format,
    c: *const wined3d_color,
    retval: *mut VkClearColorValue,
) {
    if (*format).attrs & WINED3D_FORMAT_ATTR_INTEGER != 0 {
        (*retval).int32[0 as usize] = (*c).r as int32_t;
        (*retval).int32[1 as usize] = (*c).g as int32_t;
        (*retval).int32[2 as usize] = (*c).b as int32_t;
        (*retval).int32[3 as usize] = (*c).a as int32_t;
    } else {
        (*retval).float32[0 as usize] = (*c).r;
        (*retval).float32[1 as usize] = (*c).g;
        (*retval).float32[2 as usize] = (*c).b;
        (*retval).float32[3 as usize] = (*c).a;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_convert_from_float(
    format: *const wined3d_format,
    mut color: *const wined3d_color,
    ret: *mut ::core::ffi::c_void,
) {
    // utils.c's function-local anonymous structs of the two conversion tables.
    struct C2RustUnnamed_50 {
        format_id: wined3d_format_id,
        mul: wined3d_vec4,
        shift: wined3d_uvec4,
    }
    struct C2RustUnnamed_49 {
        format_id: wined3d_format_id,
        mul: wined3d_dvec4,
        shift: wined3d_uvec4,
    }
    static float_conv: [C2RustUnnamed_50; 3] = [
        C2RustUnnamed_50 {
            format_id: WINED3DFMT_P8_UINT,
            mul: wined3d_vec4 {
                x: 0.0f32,
                y: 0.0f32,
                z: 0.0f32,
                w: 255.0f32,
            },
            shift: wined3d_uvec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        },
        C2RustUnnamed_50 {
            format_id: WINED3DFMT_S1_UINT_D15_UNORM,
            mul: wined3d_vec4 {
                x: 32767.0f32,
                y: 0.0f32,
                z: 0.0f32,
                w: 0.0f32,
            },
            shift: wined3d_uvec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        },
        C2RustUnnamed_50 {
            format_id: WINED3DFMT_D16_UNORM,
            mul: wined3d_vec4 {
                x: 65535.0f32,
                y: 0.0f32,
                z: 0.0f32,
                w: 0.0f32,
            },
            shift: wined3d_uvec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        },
    ];
    static double_conv: [C2RustUnnamed_49; 3] = [
        C2RustUnnamed_49 {
            format_id: WINED3DFMT_D24_UNORM_S8_UINT,
            mul: wined3d_dvec4 {
                x: 16777215.0f64,
                y: 1.0f64,
                z: 0.0f64,
                w: 0.0f64,
            },
            shift: wined3d_uvec4 {
                x: 8,
                y: 0,
                z: 0,
                w: 0,
            },
        },
        C2RustUnnamed_49 {
            format_id: WINED3DFMT_X8D24_UNORM,
            mul: wined3d_dvec4 {
                x: 16777215.0f64,
                y: 0.0f64,
                z: 0.0f64,
                w: 0.0f64,
            },
            shift: wined3d_uvec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        },
        C2RustUnnamed_49 {
            format_id: WINED3DFMT_D32_UNORM,
            mul: wined3d_dvec4 {
                x: 4294967295.0f64,
                y: 0.0f64,
                z: 0.0f64,
                w: 0.0f64,
            },
            shift: wined3d_uvec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            },
        },
    ];
    let mut format_id: wined3d_format_id = (*format).id;
    let mut colour_srgb: wined3d_color = wined3d_color {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    let mut idx: wined3d_uvec4 = wined3d_uvec4 {
        x: 0,
        y: 0,
        z: 0,
        w: 0,
    };
    let mut shift: wined3d_uvec4 = wined3d_uvec4 {
        x: 0,
        y: 0,
        z: 0,
        w: 0,
    };
    let mut i: ::core::ffi::c_uint = 0;

    memset(
        ret,
        0 as ::core::ffi::c_int,
        max!(
            core::mem::size_of::<uint32_t>() as size_t,
            (*format).byte_count as size_t,
        ),
    );

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(format_srgb_info) as usize {
        if format_id as ::core::ffi::c_uint
            != format_srgb_info[i as usize].srgb_format_id as ::core::ffi::c_uint
        {
            i = i.wrapping_add(1);
        } else {
            wined3d_colour_srgb_from_linear(core::ptr::addr_of_mut!(colour_srgb), color);
            format_id = format_srgb_info[i as usize].base_format_id;
            color = core::ptr::addr_of!(colour_srgb);
            break;
        }
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(float_conv) as usize {
        let mut ret_i: *mut uint32_t = ret as *mut uint32_t;
        if format_id as ::core::ffi::c_uint
            != float_conv[i as usize].format_id as ::core::ffi::c_uint
        {
            i = i.wrapping_add(1);
        } else {
            idx.x = float_conv[i as usize].shift.x / 32;
            idx.y = float_conv[i as usize].shift.y / 32;
            idx.z = float_conv[i as usize].shift.z / 32;
            idx.w = float_conv[i as usize].shift.w / 32;
            shift.x = float_conv[i as usize].shift.x % 32;
            shift.y = float_conv[i as usize].shift.y % 32;
            shift.z = float_conv[i as usize].shift.z % 32;
            shift.w = float_conv[i as usize].shift.w % 32;

            *ret_i.offset(idx.x as isize) =
                (((*color).r * float_conv[i as usize].mul.x + 0.5f32) as uint32_t) << shift.x;
            *ret_i.offset(idx.y as isize) |=
                (((*color).g * float_conv[i as usize].mul.y + 0.5f32) as uint32_t) << shift.y;
            *ret_i.offset(idx.z as isize) |=
                (((*color).b * float_conv[i as usize].mul.z + 0.5f32) as uint32_t) << shift.z;
            *ret_i.offset(idx.w as isize) |=
                (((*color).a * float_conv[i as usize].mul.w + 0.5f32) as uint32_t) << shift.w;
            return;
        }
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(double_conv) as usize {
        let mut ret_i: *mut uint32_t = core::ptr::null_mut();
        if format_id as ::core::ffi::c_uint
            != double_conv[i as usize].format_id as ::core::ffi::c_uint
        {
            i = i.wrapping_add(1);
        } else {
            idx.x = double_conv[i as usize].shift.x / 32;
            idx.y = double_conv[i as usize].shift.y / 32;
            idx.z = double_conv[i as usize].shift.z / 32;
            idx.w = double_conv[i as usize].shift.w / 32;
            shift.x = double_conv[i as usize].shift.x % 32;
            shift.y = double_conv[i as usize].shift.y % 32;
            shift.z = double_conv[i as usize].shift.z % 32;
            shift.w = double_conv[i as usize].shift.w % 32;

            ret_i = ret as *mut uint32_t;
            *ret_i.offset(idx.x as isize) =
                (((*color).r as ::core::ffi::c_double * double_conv[i as usize].mul.x + 0.5f64)
                    as uint32_t)
                    << shift.x;
            *ret_i.offset(idx.y as isize) |=
                (((*color).g as ::core::ffi::c_double * double_conv[i as usize].mul.y + 0.5f64)
                    as uint32_t)
                    << shift.y;
            *ret_i.offset(idx.z as isize) |=
                (((*color).b as ::core::ffi::c_double * double_conv[i as usize].mul.z + 0.5f64)
                    as uint32_t)
                    << shift.z;
            *ret_i.offset(idx.w as isize) |=
                (((*color).a as ::core::ffi::c_double * double_conv[i as usize].mul.w + 0.5f64)
                    as uint32_t)
                    << shift.w;
            return;
        }
    }

    if ((*format).attrs & WINED3D_FORMAT_ATTR_NORMALISED) != 0 {
        let mut ret_i: *mut uint32_t = ret as *mut uint32_t;

        idx.x = ((*format).red_offset / 32) as ::core::ffi::c_uint;
        idx.y = ((*format).green_offset / 32) as ::core::ffi::c_uint;
        idx.z = ((*format).blue_offset / 32) as ::core::ffi::c_uint;
        idx.w = ((*format).alpha_offset / 32) as ::core::ffi::c_uint;
        shift.x = ((*format).red_offset % 32) as ::core::ffi::c_uint;
        shift.y = ((*format).green_offset % 32) as ::core::ffi::c_uint;
        shift.z = ((*format).blue_offset % 32) as ::core::ffi::c_uint;
        shift.w = ((*format).alpha_offset % 32) as ::core::ffi::c_uint;

        *ret_i.offset(idx.x as isize) = (((*color).r
            * wined3d_mask_from_size((*format).red_size as ::core::ffi::c_uint) as ::core::ffi::c_float
            + 0.5f32) as uint32_t)
            << shift.x;
        *ret_i.offset(idx.y as isize) |= (((*color).g
            * wined3d_mask_from_size((*format).green_size as ::core::ffi::c_uint) as ::core::ffi::c_float
            + 0.5f32) as uint32_t)
            << shift.y;
        *ret_i.offset(idx.z as isize) |= (((*color).b
            * wined3d_mask_from_size((*format).blue_size as ::core::ffi::c_uint) as ::core::ffi::c_float
            + 0.5f32) as uint32_t)
            << shift.z;
        *ret_i.offset(idx.w as isize) |= (((*color).a
            * wined3d_mask_from_size((*format).alpha_size as ::core::ffi::c_uint) as ::core::ffi::c_float
            + 0.5f32) as uint32_t)
            << shift.w;

        return;
    }

    /* 32 bit float formats. We don't handle D32_FLOAT_S8X24_UINT for now. */
    if ((*format).attrs & WINED3D_FORMAT_ATTR_FLOAT) != 0
        && ((*format).red_size == 32
            || (*format).depth_size == 32)
        && (*format).stencil_size == 0
    {
        let mut ret_f: *mut ::core::ffi::c_float = ret as *mut ::core::ffi::c_float;
        let mut current_block_65: u64;
        match (*format).byte_count {
            16 => {
                *ret_f.offset(3 as isize) = (*color).a;
                current_block_65 = 9965670297045399468;
            }
            12 => {
                current_block_65 = 9965670297045399468;
            }
            8 => {
                current_block_65 = 6407982369357417134;
            }
            4 => {
                current_block_65 = 10578124522690460110;
            }
            _ => {
                ERR!(
                    "Unexpected byte count %u, format %s.\n",
                    (*format).byte_count,
                    debug_d3dformat(format_id)
                );
                current_block_65 = 2116367355679836638;
            }
        }
        match current_block_65 {
            9965670297045399468 => {
                *ret_f.offset(2 as isize) = (*color).b;
                current_block_65 = 6407982369357417134;
            }
            _ => {}
        }
        match current_block_65 {
            6407982369357417134 => {
                *ret_f.offset(1 as isize) = (*color).g;
                current_block_65 = 10578124522690460110;
            }
            _ => {}
        }
        match current_block_65 {
            10578124522690460110 => {
                *ret_f.offset(0 as isize) = (*color).r;
            }
            _ => {}
        }

        return;
    }

    if ((*format).attrs & WINED3D_FORMAT_ATTR_FLOAT) != 0 && (*format).red_size == 16
    {
        let mut ret_s: *mut uint16_t = ret as *mut uint16_t;
        let mut current_block_78: u64;
        match (*format).byte_count {
            8 => {
                *ret_s.offset(3 as isize) =
                    float_32_to_16(core::ptr::addr_of!((*color).a)) as uint16_t;
                *ret_s.offset(2 as isize) =
                    float_32_to_16(core::ptr::addr_of!((*color).b)) as uint16_t;
                current_block_78 = 4063186388946668995;
            }
            4 => {
                current_block_78 = 4063186388946668995;
            }
            2 => {
                current_block_78 = 5620758419192148984;
            }
            _ => {
                ERR!(
                    "Unexpected byte count %u, format %s.\n",
                    (*format).byte_count,
                    debug_d3dformat(format_id)
                );
                current_block_78 = 16029476503615101993;
            }
        }
        match current_block_78 {
            4063186388946668995 => {
                *ret_s.offset(1 as isize) =
                    float_32_to_16(core::ptr::addr_of!((*color).g)) as uint16_t;
                current_block_78 = 5620758419192148984;
            }
            _ => {}
        }
        match current_block_78 {
            5620758419192148984 => {
                *ret_s.offset(0 as isize) =
                    float_32_to_16(core::ptr::addr_of!((*color).r)) as uint16_t;
            }
            _ => {}
        }

        return;
    }
}

unsafe extern "C" fn color_to_float(mut color: DWORD, mut size: DWORD, mut offset: DWORD) -> f32 {
    let mut mask: uint32_t = wined3d_mask_from_size(size);

    if size == 0 {
        return 1.0f32;
    }

    color >>= offset;
    color &= mask;

    return color as f32 / mask as f32;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_format_get_float_color_key(format: *const wined3d_format, key: *const wined3d_color_key, float_colors: *mut wined3d_color) {
    let mut slop: wined3d_color = wined3d_color {
        r: INFINITY,
        g: INFINITY,
        b: INFINITY,
        a: INFINITY,
    };

    match (*format).id as ::core::ffi::c_uint {
        WINED3DFMT_B8G8R8_UNORM
        | WINED3DFMT_B8G8R8A8_UNORM
        | WINED3DFMT_B8G8R8X8_UNORM
        | WINED3DFMT_B5G6R5_UNORM
        | WINED3DFMT_B5G5R5X1_UNORM
        | WINED3DFMT_B5G5R5A1_UNORM
        | WINED3DFMT_B4G4R4A4_UNORM
        | WINED3DFMT_B2G3R3_UNORM
        | WINED3DFMT_R8_UNORM
        | WINED3DFMT_A8_UNORM
        | WINED3DFMT_B2G3R3A8_UNORM
        | WINED3DFMT_B4G4R4X4_UNORM
        | WINED3DFMT_R10G10B10A2_UNORM
        | WINED3DFMT_R10G10B10A2_SNORM
        | WINED3DFMT_R8G8B8A8_UNORM
        | WINED3DFMT_R8G8B8X8_UNORM
        | WINED3DFMT_R16G16_UNORM
        | WINED3DFMT_B10G10R10A2_UNORM => {
            if (*format).red_size != 0 {
                slop.r = 0.5f32
                    / wined3d_mask_from_size((*format).red_size as ::core::ffi::c_uint)
                        as ::core::ffi::c_float;
            }
            if (*format).green_size != 0 {
                slop.g = 0.5f32
                    / wined3d_mask_from_size((*format).green_size as ::core::ffi::c_uint)
                        as ::core::ffi::c_float;
            }
            if (*format).blue_size != 0 {
                slop.b = 0.5f32
                    / wined3d_mask_from_size((*format).blue_size as ::core::ffi::c_uint)
                        as ::core::ffi::c_float;
            }
            if (*format).alpha_size != 0 {
                slop.a = 0.5f32
                    / wined3d_mask_from_size((*format).alpha_size as ::core::ffi::c_uint)
                        as ::core::ffi::c_float;
            }

            (*float_colors.offset(0 as isize)).r = color_to_float(
                (*key).color_space_low_value as DWORD,
                (*format).red_size,
                (*format).red_offset,
            ) - slop.r;
            (*float_colors.offset(0 as isize)).g = color_to_float(
                (*key).color_space_low_value as DWORD,
                (*format).green_size,
                (*format).green_offset,
            ) - slop.g;
            (*float_colors.offset(0 as isize)).b = color_to_float(
                (*key).color_space_low_value as DWORD,
                (*format).blue_size,
                (*format).blue_offset,
            ) - slop.b;
            (*float_colors.offset(0 as isize)).a = color_to_float(
                (*key).color_space_low_value as DWORD,
                (*format).alpha_size,
                (*format).alpha_offset,
            ) - slop.a;

            (*float_colors.offset(1 as isize)).r = color_to_float(
                (*key).color_space_high_value as DWORD,
                (*format).red_size,
                (*format).red_offset,
            ) + slop.r;
            (*float_colors.offset(1 as isize)).g = color_to_float(
                (*key).color_space_high_value as DWORD,
                (*format).green_size,
                (*format).green_offset,
            ) + slop.g;
            (*float_colors.offset(1 as isize)).b = color_to_float(
                (*key).color_space_high_value as DWORD,
                (*format).blue_size,
                (*format).blue_offset,
            ) + slop.b;
            (*float_colors.offset(1 as isize)).a = color_to_float(
                (*key).color_space_high_value as DWORD,
                (*format).alpha_size,
                (*format).alpha_offset,
            ) + slop.a;
        }
        WINED3DFMT_P8_UINT => {
            (*float_colors.offset(0 as isize)).r = 0.0f32;
            (*float_colors.offset(0 as isize)).g = 0.0f32;
            (*float_colors.offset(0 as isize)).b = 0.0f32;
            (*float_colors.offset(0 as isize)).a =
                ((*key).color_space_low_value as ::core::ffi::c_float - 0.5f32) / 255.0f32;

            (*float_colors.offset(1 as isize)).r = 0.0f32;
            (*float_colors.offset(1 as isize)).g = 0.0f32;
            (*float_colors.offset(1 as isize)).b = 0.0f32;
            (*float_colors.offset(1 as isize)).a =
                ((*key).color_space_high_value as ::core::ffi::c_float + 0.5f32) / 255.0f32;
        }
        _ => {
            ERR!(
                "Unhandled color key to float conversion for format %s.\n",
                debug_d3dformat((*format).id)
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pixelformat_for_depth(depth: DWORD) -> wined3d_format_id {
    match depth {
        8 => return WINED3DFMT_P8_UINT,
        15 => return WINED3DFMT_B5G5R5X1_UNORM,
        16 => return WINED3DFMT_B5G6R5_UNORM,
        24 => return WINED3DFMT_B8G8R8X8_UNORM, /* Robots needs 24bit to be WINED3DFMT_B8G8R8X8_UNORM */
        32 => return WINED3DFMT_B8G8R8X8_UNORM, /* EVE online and the Fur demo need 32bit AdapterDisplayMode to return WINED3DFMT_B8G8R8X8_UNORM */
        _ => return WINED3DFMT_UNKNOWN,
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_format_copy_data(
    format: *const wined3d_format,
    src: *const u8,
    src_row_pitch: ::core::ffi::c_uint,
    src_slice_pitch: ::core::ffi::c_uint,
    dst: *mut u8,
    dst_row_pitch: ::core::ffi::c_uint,
    dst_slice_pitch: ::core::ffi::c_uint,
    w: ::core::ffi::c_uint,
    h: ::core::ffi::c_uint,
    d: ::core::ffi::c_uint,
) {
    let mut row_block_count: ::core::ffi::c_uint;
    let mut row_count: ::core::ffi::c_uint;
    let mut row_size: ::core::ffi::c_uint;
    let mut slice: ::core::ffi::c_uint;
    let mut row: ::core::ffi::c_uint;
    let mut slice_count: ::core::ffi::c_uint = d;
    let mut slice_size: ::core::ffi::c_uint;
    let mut src_row: *const u8;
    let mut dst_row: *mut u8;

    row_block_count = (w as ::core::ffi::c_uint)
        .wrapping_add((*format).block_width)
        .wrapping_sub(1)
        .wrapping_div((*format).block_width) as ::core::ffi::c_uint;
    row_count = (h as ::core::ffi::c_uint)
        .wrapping_add((*format).block_height)
        .wrapping_sub(1)
        .wrapping_div((*format).block_height) as ::core::ffi::c_uint;
    row_size = (row_block_count as ::core::ffi::c_uint)
        .wrapping_mul((*format).block_byte_count) as ::core::ffi::c_uint;
    slice_size = row_size.wrapping_mul(row_count);

    if src_row_pitch == row_size
        && dst_row_pitch == row_size
        && ((src_slice_pitch == slice_size && dst_slice_pitch == slice_size)
            || slice_count == 1)
    {
        memcpy(
            dst as *mut ::core::ffi::c_void,
            src as *const ::core::ffi::c_void,
            slice_count.wrapping_mul(slice_size) as usize,
        );
        return;
    }

    slice = 0;
    while slice < slice_count {
        row = 0;
        while row < row_count {
            src_row = src.offset(
                slice
                    .wrapping_mul(src_slice_pitch)
                    .wrapping_add(row.wrapping_mul(src_row_pitch)) as isize,
            ) as *const u8;
            dst_row = dst.offset(
                slice
                    .wrapping_mul(dst_slice_pitch)
                    .wrapping_add(row.wrapping_mul(dst_row_pitch)) as isize,
            ) as *mut u8;
            memcpy(
                dst_row as *mut ::core::ffi::c_void,
                src_row as *const ::core::ffi::c_void,
                row_size as usize,
            );
            row = row.wrapping_add(1);
        }
        slice = slice.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn multiply_matrix(dst: *mut wined3d_matrix, src1: *const wined3d_matrix, src2: *const wined3d_matrix) {
    let mut tmp: wined3d_matrix = wined3d_matrix {
        _11: 0.0,
        _12: 0.0,
        _13: 0.0,
        _14: 0.0,
        _21: 0.0,
        _22: 0.0,
        _23: 0.0,
        _24: 0.0,
        _31: 0.0,
        _32: 0.0,
        _33: 0.0,
        _34: 0.0,
        _41: 0.0,
        _42: 0.0,
        _43: 0.0,
        _44: 0.0,
    };
    tmp._11 = (*src1)._11 * (*src2)._11 + (*src1)._21 * (*src2)._12 + (*src1)._31 * (*src2)._13 + (*src1)._41 * (*src2)._14;
    tmp._21 = (*src1)._11 * (*src2)._21 + (*src1)._21 * (*src2)._22 + (*src1)._31 * (*src2)._23 + (*src1)._41 * (*src2)._24;
    tmp._31 = (*src1)._11 * (*src2)._31 + (*src1)._21 * (*src2)._32 + (*src1)._31 * (*src2)._33 + (*src1)._41 * (*src2)._34;
    tmp._41 = (*src1)._11 * (*src2)._41 + (*src1)._21 * (*src2)._42 + (*src1)._31 * (*src2)._43 + (*src1)._41 * (*src2)._44;
    tmp._12 = (*src1)._12 * (*src2)._11 + (*src1)._22 * (*src2)._12 + (*src1)._32 * (*src2)._13 + (*src1)._42 * (*src2)._14;
    tmp._22 = (*src1)._12 * (*src2)._21 + (*src1)._22 * (*src2)._22 + (*src1)._32 * (*src2)._23 + (*src1)._42 * (*src2)._24;
    tmp._32 = (*src1)._12 * (*src2)._31 + (*src1)._22 * (*src2)._32 + (*src1)._32 * (*src2)._33 + (*src1)._42 * (*src2)._34;
    tmp._42 = (*src1)._12 * (*src2)._41 + (*src1)._22 * (*src2)._42 + (*src1)._32 * (*src2)._43 + (*src1)._42 * (*src2)._44;
    tmp._13 = (*src1)._13 * (*src2)._11 + (*src1)._23 * (*src2)._12 + (*src1)._33 * (*src2)._13 + (*src1)._43 * (*src2)._14;
    tmp._23 = (*src1)._13 * (*src2)._21 + (*src1)._23 * (*src2)._22 + (*src1)._33 * (*src2)._23 + (*src1)._43 * (*src2)._24;
    tmp._33 = (*src1)._13 * (*src2)._31 + (*src1)._23 * (*src2)._32 + (*src1)._33 * (*src2)._33 + (*src1)._43 * (*src2)._34;
    tmp._43 = (*src1)._13 * (*src2)._41 + (*src1)._23 * (*src2)._42 + (*src1)._33 * (*src2)._43 + (*src1)._43 * (*src2)._44;
    tmp._14 = (*src1)._14 * (*src2)._11 + (*src1)._24 * (*src2)._12 + (*src1)._34 * (*src2)._13 + (*src1)._44 * (*src2)._14;
    tmp._24 = (*src1)._14 * (*src2)._21 + (*src1)._24 * (*src2)._22 + (*src1)._34 * (*src2)._23 + (*src1)._44 * (*src2)._24;
    tmp._34 = (*src1)._14 * (*src2)._31 + (*src1)._24 * (*src2)._32 + (*src1)._34 * (*src2)._33 + (*src1)._44 * (*src2)._34;
    tmp._44 = (*src1)._14 * (*src2)._41 + (*src1)._24 * (*src2)._42 + (*src1)._34 * (*src2)._43 + (*src1)._44 * (*src2)._44;
    *dst = tmp;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_ffp_get_fs_settings(state: *const wined3d_state, d3d_info: *const wined3d_d3d_info, settings: *mut ffp_frag_settings) {
    static args: [u8; ((WINED3D_TOP_LERP) + (1)) as _] = [
        (0) as u8,
        (0) as u8,
        (ARG1) as u8,
        (ARG2) as u8,
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8),
        ((ARG1) as u8) | ((ARG2) as u8) | ((ARG0) as u8),
        ((ARG1) as u8) | ((ARG2) as u8) | ((ARG0) as u8),
    ];
    let mut i: u32 = 0;
    let mut ttff: DWORD = 0;
    let mut cop: DWORD = 0;
    let mut aop: DWORD = 0;
    let mut carg0: DWORD = 0;
    let mut carg1: DWORD = 0;
    let mut carg2: DWORD = 0;
    let mut aarg0: DWORD = 0;
    let mut aarg1: DWORD = 0;
    let mut aarg2: DWORD = 0;
    let mut texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();

    (*settings).set_padding((0) as u8);

    i = (0) as u32;
    while i < (*d3d_info).ffp_fragment_caps.max_blend_stages {
        (*settings).op[i as usize].set_padding((0) as u32);
        if (*state).texture_states[i as usize][(WINED3D_TSS_COLOR_OP) as usize]
            == (WINED3D_TOP_DISABLE) as u32
        {
            (*settings).op[i as usize].set_cop((WINED3D_TOP_DISABLE) as u32);
            (*settings).op[i as usize].set_aop((WINED3D_TOP_DISABLE) as u32);
            (*settings).op[i as usize].set_carg2((ARG_UNUSED) as u32);
            (*settings).op[i as usize].set_carg1((ARG_UNUSED) as u32);
            (*settings).op[i as usize].set_carg0((ARG_UNUSED) as u32);
            (*settings).op[i as usize].set_aarg2((ARG_UNUSED) as u32);
            (*settings).op[i as usize].set_aarg1((ARG_UNUSED) as u32);
            (*settings).op[i as usize].set_aarg0((ARG_UNUSED) as u32);
            (*settings).op[i as usize].color_fixup = (COLOR_FIXUP_IDENTITY);
            (*settings).op[i as usize].set_tmp_dst((0) as u32);
            (*settings).op[i as usize].set_tex_type((WINED3D_GL_RES_TYPE_TEX_1D) as u32);
            (*settings).op[i as usize].set_projected((WINED3D_PROJECTION_NONE) as u32);
            i = i.wrapping_add(1);
            break;
        }

        texture = wined3d_state_get_ffp_texture(state, i);
        if !texture.is_null() {
            if can_use_texture_swizzle(d3d_info, (*texture).resource.format) != 0 {
                (*settings).op[i as usize].color_fixup = (COLOR_FIXUP_IDENTITY);
            } else {
                (*settings).op[i as usize].color_fixup =
                    (*(*texture).resource.format).color_fixup;
            }
            (*settings).op[i as usize].set_tex_type((*texture).resource.gl_type as u32);
        } else {
            (*settings).op[i as usize].color_fixup = (COLOR_FIXUP_IDENTITY);
            (*settings).op[i as usize].set_tex_type((WINED3D_GL_RES_TYPE_TEX_1D) as u32);
        }

        cop = (*state).texture_states[i as usize][(WINED3D_TSS_COLOR_OP) as usize] as DWORD;
        aop = (*state).texture_states[i as usize][(WINED3D_TSS_ALPHA_OP) as usize] as DWORD;

        carg1 = (if ((args[cop as usize] as u32) & ((ARG1) as u32)) != 0 {
            (*state).texture_states[i as usize][(WINED3D_TSS_COLOR_ARG1) as usize] as u32
        } else {
            (ARG_UNUSED) as u32
        }) as DWORD;
        carg2 = (if ((args[cop as usize] as u32) & ((ARG2) as u32)) != 0 {
            (*state).texture_states[i as usize][(WINED3D_TSS_COLOR_ARG2) as usize] as u32
        } else {
            (ARG_UNUSED) as u32
        }) as DWORD;
        carg0 = (if ((args[cop as usize] as u32) & ((ARG0) as u32)) != 0 {
            (*state).texture_states[i as usize][(WINED3D_TSS_COLOR_ARG0) as usize] as u32
        } else {
            (ARG_UNUSED) as u32
        }) as DWORD;

        if is_invalid_op(
            state,
            i as i32,
            cop as wined3d_texture_op,
            carg1,
            carg2,
            carg0,
        ) != 0
        {
            carg0 = (ARG_UNUSED) as DWORD;
            carg2 = (ARG_UNUSED) as DWORD;
            carg1 = (WINED3DTA_CURRENT) as DWORD;
            cop = (WINED3D_TOP_SELECT_ARG1) as DWORD;
        }

        if cop == (WINED3D_TOP_DOTPRODUCT3) as DWORD {
            /* A dotproduct3 on the colorop overwrites the alphaop operation and replicates
             * the color result to the alpha component of the destination
             */
            aop = cop;
            aarg1 = carg1;
            aarg2 = carg2;
            aarg0 = carg0;
        } else {
            aarg1 = (if ((args[aop as usize] as u32) & ((ARG1) as u32)) != 0 {
                (*state).texture_states[i as usize][(WINED3D_TSS_ALPHA_ARG1) as usize] as u32
            } else {
                (ARG_UNUSED) as u32
            }) as DWORD;
            aarg2 = (if ((args[aop as usize] as u32) & ((ARG2) as u32)) != 0 {
                (*state).texture_states[i as usize][(WINED3D_TSS_ALPHA_ARG2) as usize] as u32
            } else {
                (ARG_UNUSED) as u32
            }) as DWORD;
            aarg0 = (if ((args[aop as usize] as u32) & ((ARG0) as u32)) != 0 {
                (*state).texture_states[i as usize][(WINED3D_TSS_ALPHA_ARG0) as usize] as u32
            } else {
                (ARG_UNUSED) as u32
            }) as DWORD;
        }

        if i == 0
            && (*state).render_states[(WINED3D_RS_COLORKEYENABLE) as usize] != 0
            && !texture.is_null()
            && ((*texture).resource.usage & ((WINED3DUSAGE_LEGACY_CUBEMAP) as u32)) == 0
            && ((*texture).async_.color_key_flags & ((WINED3D_CKEY_SRC_BLT) as DWORD)) != 0
            && (*(*texture).resource.format).alpha_size == 0
        {
            if aop == (WINED3D_TOP_DISABLE) as DWORD {
                aarg1 = (WINED3DTA_TEXTURE) as DWORD;
                aop = (WINED3D_TOP_SELECT_ARG1) as DWORD;
            } else if aop == (WINED3D_TOP_SELECT_ARG1) as DWORD
                && aarg1 != (WINED3DTA_TEXTURE) as DWORD
            {
                if !(*state).blend_state.is_null()
                    && (*(*state).blend_state).desc.rt[(0) as usize].enable
                        != 0
                {
                    aarg2 = (WINED3DTA_TEXTURE) as DWORD;
                    aop = (WINED3D_TOP_MODULATE) as DWORD;
                } else {
                    aarg1 = (WINED3DTA_TEXTURE) as DWORD;
                }
            } else if aop == (WINED3D_TOP_SELECT_ARG2) as DWORD
                && aarg2 != (WINED3DTA_TEXTURE) as DWORD
            {
                if !(*state).blend_state.is_null()
                    && (*(*state).blend_state).desc.rt[(0) as usize].enable
                        != 0
                {
                    aarg1 = (WINED3DTA_TEXTURE) as DWORD;
                    aop = (WINED3D_TOP_MODULATE) as DWORD;
                } else {
                    aarg2 = (WINED3DTA_TEXTURE) as DWORD;
                }
            }
        }

        if is_invalid_op(
            state,
            i as i32,
            aop as wined3d_texture_op,
            aarg1,
            aarg2,
            aarg0,
        ) != 0
        {
            aarg0 = (ARG_UNUSED) as DWORD;
            aarg2 = (ARG_UNUSED) as DWORD;
            aarg1 = (WINED3DTA_CURRENT) as DWORD;
            aop = (WINED3D_TOP_SELECT_ARG1) as DWORD;
        }

        if carg1 == (WINED3DTA_TEXTURE) as DWORD
            || carg2 == (WINED3DTA_TEXTURE) as DWORD
            || carg0 == (WINED3DTA_TEXTURE) as DWORD
            || aarg1 == (WINED3DTA_TEXTURE) as DWORD
            || aarg2 == (WINED3DTA_TEXTURE) as DWORD
            || aarg0 == (WINED3DTA_TEXTURE) as DWORD
        {
            ttff = (*state).texture_states[i as usize][(WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS) as usize] as DWORD;
            if ttff
                == (((WINED3D_TTFF_PROJECTED) as DWORD) | ((WINED3D_TTFF_COUNT3) as DWORD))
            {
                (*settings).op[i as usize].set_projected((WINED3D_PROJECTION_COUNT3) as u32);
            } else if (ttff & ((WINED3D_TTFF_PROJECTED) as DWORD)) != 0 {
                (*settings).op[i as usize].set_projected((WINED3D_PROJECTION_COUNT4) as u32);
            } else {
                (*settings).op[i as usize].set_projected((WINED3D_PROJECTION_NONE) as u32);
            }
        } else {
            (*settings).op[i as usize].set_projected((WINED3D_PROJECTION_NONE) as u32);
        }

        (*settings).op[i as usize].set_cop(cop as u32);
        (*settings).op[i as usize].set_aop(aop as u32);
        (*settings).op[i as usize].set_carg0(carg0 as u32);
        (*settings).op[i as usize].set_carg1(carg1 as u32);
        (*settings).op[i as usize].set_carg2(carg2 as u32);
        (*settings).op[i as usize].set_aarg0(aarg0 as u32);
        (*settings).op[i as usize].set_aarg1(aarg1 as u32);
        (*settings).op[i as usize].set_aarg2(aarg2 as u32);
        (*settings).op[i as usize].set_tmp_dst(
            (((*state).texture_states[i as usize][(WINED3D_TSS_RESULT_ARG) as usize]
                == (WINED3DTA_TEMP) as u32) as i32) as u32,
        );
        i = i.wrapping_add(1);
    }

    /* Clear unsupported stages */
    while i < (WINED3D_MAX_FFP_TEXTURES) as u32 {
        memset(
            core::ptr::addr_of_mut!((*settings).op[i as usize]) as *mut core::ffi::c_void,
            (0xff) as core::ffi::c_int,
            core::mem::size_of::<texture_stage_op>() as usize,
        );
        i = i.wrapping_add(1);
    }

    if (*state).render_states[(WINED3D_RS_FOGENABLE) as usize] == 0 {
        (*settings).fog = (WINED3D_FFP_PS_FOG_OFF) as u8;
    } else if (*state).render_states[(WINED3D_RS_FOGTABLEMODE) as usize]
        == (WINED3D_FOG_NONE) as u32
    {
        if use_vs(state) != 0
            || (*(*state).vertex_declaration).position_transformed as i32 != 0
        {
            (*settings).fog = (WINED3D_FFP_PS_FOG_LINEAR) as u8;
        } else {
            match (*state).render_states[(WINED3D_RS_FOGVERTEXMODE) as usize] {
                WINED3D_FOG_NONE | WINED3D_FOG_LINEAR => {
                    (*settings).fog = (WINED3D_FFP_PS_FOG_LINEAR) as u8;
                }
                WINED3D_FOG_EXP => {
                    (*settings).fog = (WINED3D_FFP_PS_FOG_EXP) as u8;
                }
                WINED3D_FOG_EXP2 => {
                    (*settings).fog = (WINED3D_FFP_PS_FOG_EXP2) as u8;
                }
                _ => {}
            }
        }
    } else {
        match (*state).render_states[(WINED3D_RS_FOGTABLEMODE) as usize] {
            WINED3D_FOG_LINEAR => {
                (*settings).fog = (WINED3D_FFP_PS_FOG_LINEAR) as u8;
            }
            WINED3D_FOG_EXP => {
                (*settings).fog = (WINED3D_FFP_PS_FOG_EXP) as u8;
            }
            WINED3D_FOG_EXP2 => {
                (*settings).fog = (WINED3D_FFP_PS_FOG_EXP2) as u8;
            }
            _ => {}
        }
    }
    (*settings).sRGB_write = (((*d3d_info).srgb_write_control() == 0
        && needs_srgb_write(d3d_info, state, core::ptr::addr_of!((*state).fb)) != 0)
        as i32) as u8;

    texture = wined3d_state_get_ffp_texture(state, (0) as u32);
    if (*state).render_states[(WINED3D_RS_COLORKEYENABLE) as usize] != 0
        && !texture.is_null()
        && ((*texture).async_.color_key_flags & ((WINED3D_CKEY_SRC_BLT) as DWORD)) != 0
        && (*settings).op[(0) as usize].cop() as i32
            != (WINED3D_TOP_DISABLE) as i32
    {
        (*settings).set_color_key_enabled((1) as u8);
    } else {
        (*settings).set_color_key_enabled((0) as u8);
    }

    /* texcoords_initialized is set to meaningful values only when GL doesn't
     * support enough varyings to always pass around all the possible texture
     * coordinates.
     * This is used to avoid reading a varying not written by the vertex shader.
     * Reading uninitialized varyings on core profile contexts results in an
     * error while with builtin varyings on legacy contexts you get undefined
     * behavior. */
    if (*d3d_info).limits.varying_count != 0 && (*d3d_info).full_ffp_varyings() == 0 {
        (*settings).texcoords_initialized = (0) as u8;
        i = (0) as u32;
        while i < (WINED3D_MAX_FFP_TEXTURES) as u32 {
            if use_vs(state) != 0 {
                if (((*(*state).shader[(WINED3D_SHADER_TYPE_VERTEX) as usize])
                    .reg_maps
                    .output_registers as u32)
                    & (((1) as u32) << i))
                    != 0
                {
                    (*settings).texcoords_initialized = (((*settings).texcoords_initialized as u32)
                        | (((1) as u32) << i))
                        as u8;
                }
            } else {
                let mut coord_idx: u32 =
                    (*state).texture_states[i as usize][(WINED3D_TSS_TEXCOORD_INDEX) as usize] as u32;
                if ((((*state).texture_states[i as usize][(WINED3D_TSS_TEXCOORD_INDEX) as usize]
                    as u32)
                    >> ((WINED3D_FFP_TCI_SHIFT) as u32))
                    & ((WINED3D_FFP_TCI_MASK) as u32))
                    != 0
                    || (coord_idx < ((WINED3D_MAX_FFP_TEXTURES) as u32)
                        && (((*(*state).vertex_declaration).texcoords as u32)
                            & (((1) as u32) << coord_idx))
                            != 0)
                {
                    (*settings).texcoords_initialized = (((*settings).texcoords_initialized as u32)
                        | (((1) as u32) << i))
                        as u8;
                }
            }
            i = i.wrapping_add(1);
        }
    } else {
        (*settings).texcoords_initialized =
            wined3d_mask_from_size((WINED3D_MAX_FFP_TEXTURES) as u32) as u8;
    }

    (*settings).set_pointsprite(
        ((*state).render_states[(WINED3D_RS_POINTSPRITEENABLE) as usize] != 0
            && (*state).primitive_type as u32 == (WINED3D_PT_POINTLIST) as u32) as u8,
    );

    if (*d3d_info).ffp_alpha_test() != 0 {
        (*settings).set_alpha_test_func(
            (((WINED3D_CMP_ALWAYS) as i32) - ((1) as i32)) as u8,
        );
    } else {
        (*settings).set_alpha_test_func(
            (if (*state).render_states[(WINED3D_RS_ALPHATESTENABLE) as usize] != 0 {
                wined3d_sanitize_cmp_func((*state).render_states[(WINED3D_RS_ALPHAFUNC) as usize]
                    as wined3d_cmp_func) as u32
            } else {
                (WINED3D_CMP_ALWAYS) as u32
            })
            .wrapping_sub((1) as u32) as u8,
        );
    }

    if (*d3d_info).emulated_flatshading() != 0 {
        (*settings).set_flatshading(
            ((*state).render_states[(WINED3D_RS_SHADEMODE) as usize]
                == (WINED3D_SHADE_FLAT) as u32) as u8,
        );
    } else {
        (*settings).set_flatshading((FALSE) as u8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn find_ffp_frag_shader(fragment_shaders: *const rb_tree, settings: *const ffp_frag_settings) -> *const ffp_frag_desc {
    let entry: *mut wine_rb_entry = wine_rb_get(fragment_shaders, settings as *const core::ffi::c_void);
    return if !entry.is_null() {
        WINE_RB_ENTRY_VALUE!(entry, ffp_frag_desc, entry)
    } else {
        core::ptr::null()
    };
}
#[no_mangle]
pub unsafe extern "C" fn add_ffp_frag_shader(shaders: *mut rb_tree, desc: *mut ffp_frag_desc) {
    if wine_rb_put(
        shaders,
        &raw mut (*desc).settings as *const core::ffi::c_void,
        &raw mut (*desc).entry,
    ) == -1
    {
        ERR!("Failed to insert ffp frag shader.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_ffp_frag_program_key_compare(key: *const ::core::ffi::c_void, entry: *const rb_entry) -> ::core::ffi::c_int {
    let ka: *const ffp_frag_settings = key as *const ffp_frag_settings;
    let kb: *const ffp_frag_settings = core::ptr::addr_of!(
        (*((entry as *const u8).sub(core::mem::offset_of!(ffp_frag_desc, entry))
            as *const ffp_frag_desc))
        .settings
    );

    return memcmp(
        ka as *const ::core::ffi::c_void,
        kb as *const ::core::ffi::c_void,
        core::mem::size_of::<ffp_frag_settings>() as usize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_ffp_get_vs_settings(mut state: *const wined3d_state, mut si: *const wined3d_stream_info, mut d3d_info: *const wined3d_d3d_info, mut settings: *mut wined3d_ffp_vs_settings) {
    let mut diffuse_source: wined3d_material_color_source = WINED3D_MCS_MATERIAL;
    let mut emissive_source: wined3d_material_color_source = WINED3D_MCS_MATERIAL;
    let mut ambient_source: wined3d_material_color_source = WINED3D_MCS_MATERIAL;
    let mut specular_source: wined3d_material_color_source = WINED3D_MCS_MATERIAL;
    let mut vdecl: *const wined3d_vertex_declaration = (*state).vertex_declaration;
    let mut coord_idx: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    memset(
        settings as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<wined3d_ffp_vs_settings>() as size_t,
    );
    if (*vdecl).position_transformed {
        (*settings).set_transformed(1 as DWORD as DWORD);
        (*settings).set_point_size(
            ((*state).primitive_type as ::core::ffi::c_uint
                == WINED3D_PT_POINTLIST as ::core::ffi::c_uint)
                as ::core::ffi::c_int as DWORD as DWORD,
        );
        (*settings).set_per_vertex_point_size((*vdecl).point_size as DWORD as DWORD);
        (*settings).set_diffuse((*vdecl).diffuse as DWORD as DWORD);
        if (*state).render_states[WINED3D_RS_FOGENABLE as usize] == 0 {
            (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_OFF as DWORD as DWORD);
        } else if (*state).render_states[WINED3D_RS_FOGTABLEMODE as usize]
            != WINED3D_FOG_NONE as ::core::ffi::c_uint
        {
            (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_DEPTH as DWORD as DWORD);
        } else {
            (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_FOGCOORD as DWORD as DWORD);
        }
        i = 0 as ::core::ffi::c_uint;
        while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
            coord_idx = (*state).texture_states[i as usize]
                [WINED3D_TSS_TEXCOORD_INDEX as usize] as ::core::ffi::c_uint;
            if coord_idx < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint
                && (*vdecl).texcoords as ::core::ffi::c_uint
                    & (1 as ::core::ffi::c_uint) << coord_idx
                    != 0
            {
                (*settings).set_texcoords(
                    (*settings).texcoords()
                        | ((1 as ::core::ffi::c_uint) << i) as DWORD,
                );
            }
            (*settings).texgen[i as usize] = (*state).texture_states[i as usize]
                [WINED3D_TSS_TEXCOORD_INDEX as usize] as ::core::ffi::c_uint;
            i = i.wrapping_add(1);
        }
        if (*d3d_info).full_ffp_varyings() != 0 {
            (*settings).set_texcoords(wined3d_mask_from_size(
                WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint,
            ) as DWORD as DWORD);
        }
        if (*d3d_info).emulated_flatshading() != 0 {
            (*settings).set_flatshading(
                ((*state).render_states[WINED3D_RS_SHADEMODE as usize]
                    == WINED3D_SHADE_FLAT as ::core::ffi::c_uint)
                    as ::core::ffi::c_int as DWORD as DWORD,
            );
        } else {
            (*settings).set_flatshading(FALSE as DWORD as DWORD);
        }
        (*settings).swizzle_map = (*si).swizzle_map;
        return;
    }
    match (*state).render_states[WINED3D_RS_VERTEXBLEND as usize] {
        WINED3D_VBF_DISABLE | WINED3D_VBF_1WEIGHTS | WINED3D_VBF_2WEIGHTS
        | WINED3D_VBF_3WEIGHTS => {
            (*settings).set_vertexblends(
                (*state).render_states[WINED3D_RS_VERTEXBLEND as usize] as DWORD as DWORD,
            );
        }
        _ => {
            WINE_FIXME!("Unsupported vertex blending: %d\n", (*state).render_states[WINED3D_RS_VERTEXBLEND as usize]);
        }
    }
    (*settings).set_clipping(
        ((*state).render_states[WINED3D_RS_CLIPPING as usize] != 0
            && (*state).render_states[WINED3D_RS_CLIPPLANEENABLE as usize] != 0)
            as ::core::ffi::c_int as DWORD as DWORD,
    );
    (*settings).set_diffuse((*vdecl).diffuse as DWORD as DWORD);
    (*settings).set_normal((*vdecl).normal as DWORD as DWORD);
    (*settings).set_normalize(
        ((*settings).normal() as ::core::ffi::c_int != 0
            && (*state).render_states[WINED3D_RS_NORMALIZENORMALS as usize] != 0)
            as ::core::ffi::c_int as DWORD as DWORD,
    );
    (*settings).set_lighting(
        ((*state).render_states[WINED3D_RS_LIGHTING as usize] != 0)
            as ::core::ffi::c_int as DWORD as DWORD,
    );
    (*settings).set_localviewer(
        ((*state).render_states[WINED3D_RS_LOCALVIEWER as usize] != 0)
            as ::core::ffi::c_int as DWORD as DWORD,
    );
    (*settings).set_specular_enable(
        ((*state).render_states[WINED3D_RS_SPECULARENABLE as usize] != 0)
            as ::core::ffi::c_int as DWORD as DWORD,
    );
    (*settings).set_point_size(
        ((*state).primitive_type as ::core::ffi::c_uint
            == WINED3D_PT_POINTLIST as ::core::ffi::c_uint) as ::core::ffi::c_int as DWORD
            as DWORD,
    );
    (*settings).set_per_vertex_point_size((*vdecl).point_size as DWORD as DWORD);

    wined3d_get_material_colour_source(
        &raw mut diffuse_source,
        &raw mut emissive_source,
        &raw mut ambient_source,
        &raw mut specular_source,
        state,
    );
    (*settings).set_diffuse_source(diffuse_source as DWORD as DWORD);
    (*settings).set_emissive_source(emissive_source as DWORD as DWORD);
    (*settings).set_ambient_source(ambient_source as DWORD as DWORD);
    (*settings).set_specular_source(specular_source as DWORD as DWORD);

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
        coord_idx = (*state).texture_states[i as usize]
            [WINED3D_TSS_TEXCOORD_INDEX as usize] as ::core::ffi::c_uint;
        if coord_idx < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint
            && (*vdecl).texcoords as ::core::ffi::c_uint
                & (1 as ::core::ffi::c_uint) << coord_idx
                != 0
        {
            (*settings).set_texcoords(
                (*settings).texcoords()
                    | ((1 as ::core::ffi::c_uint) << i) as DWORD,
            );
        }
        (*settings).texgen[i as usize] = (*state).texture_states[i as usize]
            [WINED3D_TSS_TEXCOORD_INDEX as usize] as ::core::ffi::c_uint;
        i = i.wrapping_add(1);
    }
    if (*d3d_info).full_ffp_varyings() != 0 {
        (*settings).set_texcoords(wined3d_mask_from_size(
            WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint,
        ) as DWORD as DWORD);
    }

    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_MAX_ACTIVE_LIGHTS as ::core::ffi::c_uint {
        if !(*state).light_state.lights[i as usize].is_null() {
            match (*(*state).light_state.lights[i as usize])
                .OriginalParms
                .type_ as ::core::ffi::c_uint
            {
                WINED3D_LIGHT_POINT => {
                    (*settings).set_point_light_count((*settings).point_light_count() + 1 as DWORD);
                }
                WINED3D_LIGHT_SPOT => {
                    (*settings).set_spot_light_count((*settings).spot_light_count() + 1 as DWORD);
                }
                WINED3D_LIGHT_DIRECTIONAL => {
                    (*settings).set_directional_light_count(
                        (*settings).directional_light_count() + 1 as DWORD,
                    );
                }
                WINED3D_LIGHT_PARALLELPOINT => {
                    (*settings).set_parallel_point_light_count(
                        (*settings).parallel_point_light_count() + 1 as DWORD,
                    );
                }
                _ => {
                    WINE_FIXME!("Unhandled light type %#x.\n", (*(*state).light_state.lights[i as usize]) .OriginalParms .type_ as ::core::ffi::c_uint);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if (*state).render_states[WINED3D_RS_FOGENABLE as usize] == 0 {
        (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_OFF as DWORD as DWORD);
    } else if (*state).render_states[WINED3D_RS_FOGTABLEMODE as usize]
        != WINED3D_FOG_NONE as ::core::ffi::c_uint
    {
        (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_DEPTH as DWORD as DWORD);

        if (*state).transforms[WINED3D_TS_PROJECTION as usize]._14 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._24 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._34 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._44 == 1.0f32
        {
            (*settings).set_ortho_fog(1 as DWORD as DWORD);
        }
    } else if (*state).render_states[WINED3D_RS_FOGVERTEXMODE as usize]
        == WINED3D_FOG_NONE as ::core::ffi::c_uint
    {
        (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_FOGCOORD as DWORD as DWORD);
    } else if (*state).render_states[WINED3D_RS_RANGEFOGENABLE as usize] != 0 {
        (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_RANGE as DWORD as DWORD);
    } else {
        (*settings).set_fog_mode(WINED3D_FFP_VS_FOG_DEPTH as DWORD as DWORD);
    }

    if (*d3d_info).emulated_flatshading() != 0 {
        (*settings).set_flatshading(
            ((*state).render_states[WINED3D_RS_SHADEMODE as usize]
                == WINED3D_SHADE_FLAT as ::core::ffi::c_uint)
                as ::core::ffi::c_int as DWORD as DWORD,
        );
    } else {
        (*settings).set_flatshading(FALSE as DWORD as DWORD);
    }

    (*settings).swizzle_map = (*si).swizzle_map;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_ffp_vertex_program_key_compare(key: *const ::core::ffi::c_void, entry: *const rb_entry) -> ::core::ffi::c_int {
    let ka: *const wined3d_ffp_vs_settings = key as *const wined3d_ffp_vs_settings;
    let kb: *const wined3d_ffp_vs_settings = core::ptr::addr_of!(
        (*((entry as *const u8).sub(core::mem::offset_of!(wined3d_ffp_vs_desc, entry))
            as *const wined3d_ffp_vs_desc))
            .settings
    );

    return memcmp(
        ka as *const core::ffi::c_void,
        kb as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_ffp_vs_settings>() as usize,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_location(mut location: uint32_t) -> *const ::core::ffi::c_char {
    let mut buffer: debug_buffer = debug_buffer {
        str: [0; 200],
        ptr: core::ptr::null_mut::<core::ffi::c_char>(),
        size: 0,
    };
    let mut prefix: *const core::ffi::c_char = b"\0".as_ptr() as *const core::ffi::c_char;
    let mut suffix: *const core::ffi::c_char = b"\0".as_ptr() as *const core::ffi::c_char;

    if wined3d_popcount(location as core::ffi::c_uint) > (16 as core::ffi::c_uint) {
        prefix = b"~(\0".as_ptr() as *const core::ffi::c_char;
        location = !location;
        suffix = b")\0".as_ptr() as *const core::ffi::c_char;
    }

    init_debug_buffer(
        &raw mut buffer,
        b"0\0".as_ptr() as *const core::ffi::c_char,
    );

    if (location & (WINED3D_LOCATION_DISCARDED as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_DISCARDED\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_DISCARDED as uint32_t);
    }
    if (location & (WINED3D_LOCATION_SYSMEM as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_SYSMEM\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_SYSMEM as uint32_t);
    }
    if (location & (WINED3D_LOCATION_CLEARED as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_CLEARED\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_CLEARED as uint32_t);
    }
    if (location & (WINED3D_LOCATION_BUFFER as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_BUFFER\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_BUFFER as uint32_t);
    }
    if (location & (WINED3D_LOCATION_TEXTURE_RGB as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_TEXTURE_RGB\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_TEXTURE_RGB as uint32_t);
    }
    if (location & (WINED3D_LOCATION_TEXTURE_SRGB as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_TEXTURE_SRGB\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_TEXTURE_SRGB as uint32_t);
    }
    if (location & (WINED3D_LOCATION_DRAWABLE as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_DRAWABLE\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_DRAWABLE as uint32_t);
    }
    if (location & (WINED3D_LOCATION_RB_MULTISAMPLE as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_RB_MULTISAMPLE\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_RB_MULTISAMPLE as uint32_t);
    }
    if (location & (WINED3D_LOCATION_RB_RESOLVED as uint32_t)) != 0 {
        debug_append(
            &raw mut buffer,
            b"WINED3D_LOCATION_RB_RESOLVED\0".as_ptr() as *const core::ffi::c_char,
            b" | \0".as_ptr() as *const core::ffi::c_char,
        );
        location &= !(WINED3D_LOCATION_RB_RESOLVED as uint32_t);
    }

    if location != 0 {
        WINE_FIXME!("Unrecognized location flag(s) %#x.\n", location);
    }

    return wine_dbg_sprintf(
        b"%s%s%s\0".as_ptr() as *const core::ffi::c_char,
        prefix,
        &raw mut buffer.str as *mut core::ffi::c_char,
        suffix,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_debug_feature_level(level: wined3d_feature_level) -> *const ::core::ffi::c_char {
    match level {
        WINED3D_FEATURE_LEVEL_5 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_5), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_6 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_6), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_7 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_7), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_8 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_8), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_9_1 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_9_1), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_9_2 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_9_2), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_9_3 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_9_3), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_10 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_10), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_10_1 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_10_1), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_11 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_11), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        WINED3D_FEATURE_LEVEL_11_1 => {
            return concat!(stringify!(WINED3D_FEATURE_LEVEL_11_1), "\0").as_ptr() as *const ::core::ffi::c_char;
        }
        _ => {
            return wine_dbg_sprintf(
                b"%#x\0".as_ptr() as *const ::core::ffi::c_char,
                level as ::core::ffi::c_uint,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_ftoa(value: ::core::ffi::c_float, s: *mut ::core::ffi::c_char) {
    let mut idx: i32 = 1;

    if copysignf(1.0f32, value) < 0.0f32 {
        idx += 1;
    }

    /* Be sure to allocate a buffer of at least 17 characters for the result
       as sprintf may return a 3 digit exponent when using the MSVC runtime
       instead of a 2 digit exponent. */
    sprintf(s, b"%.8e\0".as_ptr() as *const i8, value as f64);
    if isfinite!(value) != 0 {
        *s.offset(idx as isize) = b'.' as i8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_release_dc(window: HWND, dc: HDC) {
    if WindowFromDC(dc) != window {
        WARN!("DC %p does not belong to window %p.\n", dc, window);
    } else if ReleaseDC(window, dc) == 0 {
        ERR!("Failed to release device context %p, last error %#lx.\n", dc, GetLastError());
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_clip_blit(
    clip_rect: *const RECT,
    clipped: *mut RECT,
    other: *mut RECT,
) -> BOOL {
    let orig: RECT = *clipped;
    let scale_x: f32 = (orig.right - orig.left) as f32 / ((*other).right - (*other).left) as f32;
    let scale_y: f32 = (orig.bottom - orig.top) as f32 / ((*other).bottom - (*other).top) as f32;

    IntersectRect(clipped as LPRECT, clipped as LPCRECT, clip_rect as LPCRECT);

    if IsRectEmpty(clipped as LPCRECT) != 0 {
        SetRectEmpty(other as LPRECT);
        return (FALSE) as BOOL;
    }

    (*other).left += (((*clipped).left - orig.left) as f32 / scale_x) as LONG;
    (*other).top += (((*clipped).top - orig.top) as f32 / scale_y) as LONG;
    (*other).right -= ((orig.right - (*clipped).right) as f32 / scale_x) as LONG;
    (*other).bottom -= ((orig.bottom - (*clipped).bottom) as f32 / scale_y) as LONG;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_gl_limits_get_uniform_block_range(mut gl_limits: *const wined3d_gl_limits, mut shader_type: wined3d_shader_type, mut base: *mut ::core::ffi::c_uint, mut count: *mut ::core::ffi::c_uint) {
    let mut i: ::core::ffi::c_uint = 0;
    *base = 0 as ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_COUNT as ::core::ffi::c_uint {
        *count = (*gl_limits).uniform_blocks[i as usize];
        if i == shader_type as ::core::ffi::c_uint {
            return;
        }
        *base = (*base).wrapping_add(*count);
        i = i.wrapping_add(1);
    }
    ERR!(
        "Unrecognized shader type %#x.\n",
        shader_type as ::core::ffi::c_uint
    );
    *count = 0 as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_gl_limits_get_texture_unit_range(
    mut gl_limits: *const wined3d_gl_limits,
    mut shader_type: wined3d_shader_type,
    mut base: *mut ::core::ffi::c_uint,
    mut count: *mut ::core::ffi::c_uint,
) {
    let mut i: ::core::ffi::c_uint = 0;
    if shader_type as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint {
        if (*gl_limits).combined_samplers == (*gl_limits).graphics_samplers {
            *base = 0 as ::core::ffi::c_uint;
        } else {
            *base = (*gl_limits).graphics_samplers;
        }
        *count = (*gl_limits).samplers[WINED3D_SHADER_TYPE_COMPUTE as usize];
        return;
    }
    *base = 0 as ::core::ffi::c_uint;
    i = 0 as ::core::ffi::c_uint;
    while i < WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_uint {
        *count = (*gl_limits).samplers[i as usize];
        if i == shader_type as ::core::ffi::c_uint {
            return;
        }
        *base = (*base).wrapping_add(*count);
        i = i.wrapping_add(1);
    }
    ERR!("Unrecognized shader type %#x.\n", shader_type as ::core::ffi::c_uint);
    *count = 0 as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_array_reserve(elements: *mut *mut ::core::ffi::c_void, capacity: *mut SIZE_T, count: SIZE_T, size: SIZE_T) -> BOOL {
    let mut max_capacity: SIZE_T;
    let mut new_capacity: SIZE_T;
    let mut new_elements: *mut core::ffi::c_void;

    if count <= *capacity {
        return (TRUE) as BOOL;
    }

    max_capacity = (!(0 as SIZE_T)) / size;
    if count > max_capacity {
        return (FALSE) as BOOL;
    }

    new_capacity = max!(1, *capacity);
    while new_capacity < count && new_capacity <= max_capacity / 2 {
        new_capacity *= 2;
    }
    if new_capacity < count {
        new_capacity = count;
    }

    new_elements = _recalloc(*elements, (new_capacity) as _, (size) as _);
    if new_elements.is_null() {
        return (FALSE) as BOOL;
    }

    *elements = new_elements;
    *capacity = new_capacity;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn swap_rows(a: *mut *mut f32, b: *mut *mut f32) {
    let mut tmp: *mut f32 = *a;
    *a = *b;
    *b = tmp;
}
#[no_mangle]
pub unsafe extern "C" fn invert_matrix(out: *mut wined3d_matrix, m: *const wined3d_matrix) -> BOOL {
    let mut wtmp: [[f32; 8]; 4] =
        [[0.0f32; 8]; 4];
    let mut m0: f32 = 0.0;
    let mut m1: f32 = 0.0;
    let mut m2: f32 = 0.0;
    let mut m3: f32 = 0.0;
    let mut s: f32 = 0.0;
    let mut r0: *mut f32 = core::ptr::null_mut::<f32>();
    let mut r1: *mut f32 = core::ptr::null_mut::<f32>();
    let mut r2: *mut f32 = core::ptr::null_mut::<f32>();
    let mut r3: *mut f32 = core::ptr::null_mut::<f32>();

    r0 = (&raw mut wtmp as *mut [f32; 8]).offset(0 as isize) as *mut f32;
    r1 = (&raw mut wtmp as *mut [f32; 8]).offset(1 as isize) as *mut f32;
    r2 = (&raw mut wtmp as *mut [f32; 8]).offset(2 as isize) as *mut f32;
    r3 = (&raw mut wtmp as *mut [f32; 8]).offset(3 as isize) as *mut f32;

    *r0.offset(0 as isize) = (*m)._11;
    *r0.offset(1 as isize) = (*m)._12;
    *r0.offset(2 as isize) = (*m)._13;
    *r0.offset(3 as isize) = (*m)._14;
    *r0.offset(4 as isize) = 1.0f32;
    *r0.offset(7 as isize) = 0.0f32;
    *r0.offset(6 as isize) = *r0.offset(7 as isize);
    *r0.offset(5 as isize) = *r0.offset(6 as isize);

    *r1.offset(0 as isize) = (*m)._21;
    *r1.offset(1 as isize) = (*m)._22;
    *r1.offset(2 as isize) = (*m)._23;
    *r1.offset(3 as isize) = (*m)._24;
    *r1.offset(5 as isize) = 1.0f32;
    *r1.offset(7 as isize) = 0.0f32;
    *r1.offset(6 as isize) = *r1.offset(7 as isize);
    *r1.offset(4 as isize) = *r1.offset(6 as isize);

    *r2.offset(0 as isize) = (*m)._31;
    *r2.offset(1 as isize) = (*m)._32;
    *r2.offset(2 as isize) = (*m)._33;
    *r2.offset(3 as isize) = (*m)._34;
    *r2.offset(6 as isize) = 1.0f32;
    *r2.offset(7 as isize) = 0.0f32;
    *r2.offset(5 as isize) = *r2.offset(7 as isize);
    *r2.offset(4 as isize) = *r2.offset(5 as isize);

    *r3.offset(0 as isize) = (*m)._41;
    *r3.offset(1 as isize) = (*m)._42;
    *r3.offset(2 as isize) = (*m)._43;
    *r3.offset(3 as isize) = (*m)._44;
    *r3.offset(7 as isize) = 1.0f32;
    *r3.offset(6 as isize) = 0.0f32;
    *r3.offset(5 as isize) = *r3.offset(6 as isize);
    *r3.offset(4 as isize) = *r3.offset(5 as isize);

    /* Choose pivot - or die. */
    if fabsf(*r3.offset(0 as isize))
        > fabsf(*r2.offset(0 as isize))
    {
        swap_rows(&raw mut r3, &raw mut r2);
    }
    if fabsf(*r2.offset(0 as isize))
        > fabsf(*r1.offset(0 as isize))
    {
        swap_rows(&raw mut r2, &raw mut r1);
    }
    if fabsf(*r1.offset(0 as isize))
        > fabsf(*r0.offset(0 as isize))
    {
        swap_rows(&raw mut r1, &raw mut r0);
    }
    if *r0.offset(0 as isize) == 0.0f32 {
        return (FALSE) as BOOL;
    }

    /* Eliminate first variable. */
    m1 = *r1.offset(0 as isize) / *r0.offset(0 as isize);
    m2 = *r2.offset(0 as isize) / *r0.offset(0 as isize);
    m3 = *r3.offset(0 as isize) / *r0.offset(0 as isize);
    s = *r0.offset(1 as isize);
    *r1.offset(1 as isize) -= m1 * s;
    *r2.offset(1 as isize) -= m2 * s;
    *r3.offset(1 as isize) -= m3 * s;
    s = *r0.offset(2 as isize);
    *r1.offset(2 as isize) -= m1 * s;
    *r2.offset(2 as isize) -= m2 * s;
    *r3.offset(2 as isize) -= m3 * s;
    s = *r0.offset(3 as isize);
    *r1.offset(3 as isize) -= m1 * s;
    *r2.offset(3 as isize) -= m2 * s;
    *r3.offset(3 as isize) -= m3 * s;
    s = *r0.offset(4 as isize);
    if s != 0.0f32 {
        *r1.offset(4 as isize) -= m1 * s;
        *r2.offset(4 as isize) -= m2 * s;
        *r3.offset(4 as isize) -= m3 * s;
    }
    s = *r0.offset(5 as isize);
    if s != 0.0f32 {
        *r1.offset(5 as isize) -= m1 * s;
        *r2.offset(5 as isize) -= m2 * s;
        *r3.offset(5 as isize) -= m3 * s;
    }
    s = *r0.offset(6 as isize);
    if s != 0.0f32 {
        *r1.offset(6 as isize) -= m1 * s;
        *r2.offset(6 as isize) -= m2 * s;
        *r3.offset(6 as isize) -= m3 * s;
    }
    s = *r0.offset(7 as isize);
    if s != 0.0f32 {
        *r1.offset(7 as isize) -= m1 * s;
        *r2.offset(7 as isize) -= m2 * s;
        *r3.offset(7 as isize) -= m3 * s;
    }

    /* Choose pivot - or die. */
    if fabsf(*r3.offset(1 as isize))
        > fabsf(*r2.offset(1 as isize))
    {
        swap_rows(&raw mut r3, &raw mut r2);
    }
    if fabsf(*r2.offset(1 as isize))
        > fabsf(*r1.offset(1 as isize))
    {
        swap_rows(&raw mut r2, &raw mut r1);
    }
    if *r1.offset(1 as isize) == 0.0f32 {
        return (FALSE) as BOOL;
    }

    /* Eliminate second variable. */
    m2 = *r2.offset(1 as isize) / *r1.offset(1 as isize);
    m3 = *r3.offset(1 as isize) / *r1.offset(1 as isize);
    *r2.offset(2 as isize) -= m2 * *r1.offset(2 as isize);
    *r3.offset(2 as isize) -= m3 * *r1.offset(2 as isize);
    *r2.offset(3 as isize) -= m2 * *r1.offset(3 as isize);
    *r3.offset(3 as isize) -= m3 * *r1.offset(3 as isize);
    s = *r1.offset(4 as isize);
    if s != 0.0f32 {
        *r2.offset(4 as isize) -= m2 * s;
        *r3.offset(4 as isize) -= m3 * s;
    }
    s = *r1.offset(5 as isize);
    if s != 0.0f32 {
        *r2.offset(5 as isize) -= m2 * s;
        *r3.offset(5 as isize) -= m3 * s;
    }
    s = *r1.offset(6 as isize);
    if s != 0.0f32 {
        *r2.offset(6 as isize) -= m2 * s;
        *r3.offset(6 as isize) -= m3 * s;
    }
    s = *r1.offset(7 as isize);
    if s != 0.0f32 {
        *r2.offset(7 as isize) -= m2 * s;
        *r3.offset(7 as isize) -= m3 * s;
    }

    /* Choose pivot - or die. */
    if fabsf(*r3.offset(2 as isize))
        > fabsf(*r2.offset(2 as isize))
    {
        swap_rows(&raw mut r3, &raw mut r2);
    }
    if *r2.offset(2 as isize) == 0.0f32 {
        return (FALSE) as BOOL;
    }

    /* Eliminate third variable. */
    m3 = *r3.offset(2 as isize) / *r2.offset(2 as isize);
    *r3.offset(3 as isize) -= m3 * *r2.offset(3 as isize);
    *r3.offset(4 as isize) -= m3 * *r2.offset(4 as isize);
    *r3.offset(5 as isize) -= m3 * *r2.offset(5 as isize);
    *r3.offset(6 as isize) -= m3 * *r2.offset(6 as isize);
    *r3.offset(7 as isize) -= m3 * *r2.offset(7 as isize);

    /* Last check. */
    if *r3.offset(3 as isize) == 0.0f32 {
        return (FALSE) as BOOL;
    }

    /* Back substitute row 3. */
    s = 1.0f32 / *r3.offset(3 as isize);
    *r3.offset(4 as isize) *= s;
    *r3.offset(5 as isize) *= s;
    *r3.offset(6 as isize) *= s;
    *r3.offset(7 as isize) *= s;

    /* Back substitute row 2. */
    m2 = *r2.offset(3 as isize);
    s = 1.0f32 / *r2.offset(2 as isize);
    *r2.offset(4 as isize) = s
        * (*r2.offset(4 as isize) - *r3.offset(4 as isize) * m2);
    *r2.offset(5 as isize) = s
        * (*r2.offset(5 as isize) - *r3.offset(5 as isize) * m2);
    *r2.offset(6 as isize) = s
        * (*r2.offset(6 as isize) - *r3.offset(6 as isize) * m2);
    *r2.offset(7 as isize) = s
        * (*r2.offset(7 as isize) - *r3.offset(7 as isize) * m2);
    m1 = *r1.offset(3 as isize);
    *r1.offset(4 as isize) -= *r3.offset(4 as isize) * m1;
    *r1.offset(5 as isize) -= *r3.offset(5 as isize) * m1;
    *r1.offset(6 as isize) -= *r3.offset(6 as isize) * m1;
    *r1.offset(7 as isize) -= *r3.offset(7 as isize) * m1;
    m0 = *r0.offset(3 as isize);
    *r0.offset(4 as isize) -= *r3.offset(4 as isize) * m0;
    *r0.offset(5 as isize) -= *r3.offset(5 as isize) * m0;
    *r0.offset(6 as isize) -= *r3.offset(6 as isize) * m0;
    *r0.offset(7 as isize) -= *r3.offset(7 as isize) * m0;

    /* Back substitute row 1. */
    m1 = *r1.offset(2 as isize);
    s = 1.0f32 / *r1.offset(1 as isize);
    *r1.offset(4 as isize) = s
        * (*r1.offset(4 as isize) - *r2.offset(4 as isize) * m1);
    *r1.offset(5 as isize) = s
        * (*r1.offset(5 as isize) - *r2.offset(5 as isize) * m1);
    *r1.offset(6 as isize) = s
        * (*r1.offset(6 as isize) - *r2.offset(6 as isize) * m1);
    *r1.offset(7 as isize) = s
        * (*r1.offset(7 as isize) - *r2.offset(7 as isize) * m1);
    m0 = *r0.offset(2 as isize);
    *r0.offset(4 as isize) -= *r2.offset(4 as isize) * m0;
    *r0.offset(5 as isize) -= *r2.offset(5 as isize) * m0;
    *r0.offset(6 as isize) -= *r2.offset(6 as isize) * m0;
    *r0.offset(7 as isize) -= *r2.offset(7 as isize) * m0;

    /* Back substitute row 0. */
    m0 = *r0.offset(1 as isize);
    s = 1.0f32 / *r0.offset(0 as isize);
    *r0.offset(4 as isize) = s
        * (*r0.offset(4 as isize) - *r1.offset(4 as isize) * m0);
    *r0.offset(5 as isize) = s
        * (*r0.offset(5 as isize) - *r1.offset(5 as isize) * m0);
    *r0.offset(6 as isize) = s
        * (*r0.offset(6 as isize) - *r1.offset(6 as isize) * m0);
    *r0.offset(7 as isize) = s
        * (*r0.offset(7 as isize) - *r1.offset(7 as isize) * m0);

    (*out)._11 = *r0.offset(4 as isize);
    (*out)._12 = *r0.offset(5 as isize);
    (*out)._13 = *r0.offset(6 as isize);
    (*out)._14 = *r0.offset(7 as isize);
    (*out)._21 = *r1.offset(4 as isize);
    (*out)._22 = *r1.offset(5 as isize);
    (*out)._23 = *r1.offset(6 as isize);
    (*out)._24 = *r1.offset(7 as isize);
    (*out)._31 = *r2.offset(4 as isize);
    (*out)._32 = *r2.offset(5 as isize);
    (*out)._33 = *r2.offset(6 as isize);
    (*out)._34 = *r2.offset(7 as isize);
    (*out)._41 = *r3.offset(4 as isize);
    (*out)._42 = *r3.offset(5 as isize);
    (*out)._43 = *r3.offset(6 as isize);
    (*out)._44 = *r3.offset(7 as isize);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn invert_matrix_3d(mut out: *mut wined3d_matrix, mut in_0: *const wined3d_matrix) -> BOOL {
    let mut pos: f32;
    let mut neg: f32;
    let mut t: f32;
    let mut det: f32;
    let mut temp: wined3d_matrix = wined3d_matrix {
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

    /* Calculate the determinant of upper left 3x3 submatrix and
     * determine if the matrix is singular. */
    neg = 0.0f32;
    pos = neg;
    t = (*in_0)._11 * (*in_0)._22 * (*in_0)._33;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }

    t = (*in_0)._21 * (*in_0)._32 * (*in_0)._13;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }
    t = (*in_0)._31 * (*in_0)._12 * (*in_0)._23;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }

    t = -(*in_0)._31 * (*in_0)._22 * (*in_0)._13;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }
    t = -(*in_0)._21 * (*in_0)._12 * (*in_0)._33;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }

    t = -(*in_0)._11 * (*in_0)._32 * (*in_0)._23;
    if t >= 0.0f32 {
        pos += t;
    } else {
        neg += t;
    }

    det = pos + neg;

    if fabsf(det) < 1e-25f32 {
        return (FALSE) as BOOL;
    }

    det = 1.0f32 / det;
    temp._11 = ((*in_0)._22 * (*in_0)._33 - (*in_0)._32 * (*in_0)._23) * det;
    temp._12 = -((*in_0)._12 * (*in_0)._33 - (*in_0)._32 * (*in_0)._13) * det;
    temp._13 = ((*in_0)._12 * (*in_0)._23 - (*in_0)._22 * (*in_0)._13) * det;
    temp._21 = -((*in_0)._21 * (*in_0)._33 - (*in_0)._31 * (*in_0)._23) * det;
    temp._22 = ((*in_0)._11 * (*in_0)._33 - (*in_0)._31 * (*in_0)._13) * det;
    temp._23 = -((*in_0)._11 * (*in_0)._23 - (*in_0)._21 * (*in_0)._13) * det;
    temp._31 = ((*in_0)._21 * (*in_0)._32 - (*in_0)._31 * (*in_0)._22) * det;
    temp._32 = -((*in_0)._11 * (*in_0)._32 - (*in_0)._31 * (*in_0)._12) * det;
    temp._33 = ((*in_0)._11 * (*in_0)._22 - (*in_0)._21 * (*in_0)._12) * det;

    *out = temp;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn compute_normal_matrix(normal_matrix: *mut wined3d_matrix, legacy_lighting: BOOL, modelview: *const wined3d_matrix) {
    let mut mv: wined3d_matrix = core::mem::zeroed();

    mv = *modelview;
    if legacy_lighting != 0 {
        invert_matrix_3d(core::ptr::addr_of_mut!(mv), core::ptr::addr_of_mut!(mv));
    } else {
        invert_matrix(core::ptr::addr_of_mut!(mv), core::ptr::addr_of_mut!(mv));
    }
    /* Tests show that singular modelview matrices are used unchanged as normal
     * matrices on D3D3 and older. There seems to be no clearly consistent
     * behavior on newer D3D versions so always follow older ddraw behavior. */

    (*normal_matrix)._11 = mv._11;
    (*normal_matrix)._12 = mv._21;
    (*normal_matrix)._13 = mv._31;
    (*normal_matrix)._21 = mv._12;
    (*normal_matrix)._22 = mv._22;
    (*normal_matrix)._23 = mv._32;
    (*normal_matrix)._31 = mv._13;
    (*normal_matrix)._32 = mv._23;
    (*normal_matrix)._33 = mv._33;
}

unsafe extern "C" fn wined3d_allocator_release_block(
    mut allocator: *mut wined3d_allocator,
    mut block: *mut wined3d_allocator_block,
) {
    (*block).parent = (*allocator).free;
    (*allocator).free = block;
}

unsafe extern "C" fn wined3d_allocator_acquire_block(
    allocator: *mut wined3d_allocator,
) -> *mut wined3d_allocator_block {
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut();

    if (*allocator).free.is_null() {
        return malloc(core::mem::size_of::<wined3d_allocator_block>()) as *mut wined3d_allocator_block;
    }

    block = (*allocator).free;
    (*allocator).free = (*block).parent;

    return block;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_block_free(mut block: *mut wined3d_allocator_block) {
    let mut allocator: *mut wined3d_allocator = (*(*block).chunk).allocator as *mut wined3d_allocator;
    let mut parent: *mut wined3d_allocator_block = core::ptr::null_mut::<wined3d_allocator_block>();
    loop {
        parent = (*block).parent;
        if !(!parent.is_null() && (*(*block).sibling).free as i32 != 0) {
            break;
        }
        list_remove(&raw mut (*(*block).sibling).entry);
        wined3d_allocator_release_block(allocator, (*block).sibling);
        wined3d_allocator_release_block(allocator, block);
        block = parent;
    }
    (*block).free = true;
    list_add_head(
        (&raw mut (*(*block).chunk).available as *mut list).offset((*block).order as isize) as *mut list,
        &raw mut (*block).entry,
    );
}

unsafe extern "C" fn wined3d_allocator_block_init(
    block: *mut wined3d_allocator_block,
    chunk: *mut wined3d_allocator_chunk,
    parent: *mut wined3d_allocator_block,
    sibling: *mut wined3d_allocator_block,
    order: core::ffi::c_uint,
    offset: usize,
    free: bool,
) {
    list_init(core::ptr::addr_of_mut!((*block).entry));
    (*block).chunk = chunk;
    (*block).parent = parent;
    (*block).sibling = sibling;
    (*block).order = order;
    (*block).offset = offset;
    (*block).free = free;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_chunk_cleanup(chunk: *mut wined3d_allocator_chunk) {
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut i: usize;

    if list_empty(core::ptr::addr_of_mut!((*chunk).available[0])) != 0 {
        ERR!("Chunk %p is not empty.\n", chunk);
        return;
    }

    i = 1;
    while i < ARRAY_SIZE!((*chunk).available) as usize {
        if list_empty(core::ptr::addr_of_mut!((*chunk).available[i])) == 0 {
            ERR!("Chunk %p is not empty.\n", chunk);
            return;
        }
        i = i.wrapping_add(1);
    }

    block = LIST_ENTRY!(
        list_head(core::ptr::addr_of_mut!((*chunk).available[0])),
        wined3d_allocator_block,
        entry
    );
    wined3d_allocator_release_block((*chunk).allocator, block);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_chunk_init(chunk: *mut wined3d_allocator_chunk, allocator: *mut wined3d_allocator) -> bool {
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut i: u32;

    block = wined3d_allocator_acquire_block(allocator);
    if block.is_null() {
        return false;
    }
    wined3d_allocator_block_init(block, chunk, core::ptr::null_mut(), core::ptr::null_mut(), 0, 0, true);

    list_init(core::ptr::addr_of_mut!((*chunk).entry));
    i = 0;
    while (i as usize) < ARRAY_SIZE!((*chunk).available) as usize {
        list_init(core::ptr::addr_of_mut!((*chunk).available[i as usize]));
        i = i.wrapping_add(1);
    }
    list_add_head(core::ptr::addr_of_mut!((*chunk).available[0 as usize]), core::ptr::addr_of_mut!((*block).entry));
    (*chunk).allocator = allocator;
    (*chunk).map_count = 0;
    (*chunk).map_ptr = core::ptr::null_mut();

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_cleanup(allocator: *mut wined3d_allocator) {
    let mut chunk: *mut wined3d_allocator_chunk = core::ptr::null_mut();
    let mut chunk2: *mut wined3d_allocator_chunk = core::ptr::null_mut();
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut next: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut i: usize = 0;
    i = 0 as usize;
    while i < (*allocator).pool_count {
        chunk = ((*(*allocator).pools.offset(i as isize)).chunks.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_allocator_chunk, entry))
            as *mut wined3d_allocator_chunk;
        chunk2 = ((*chunk).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_allocator_chunk, entry))
            as *mut wined3d_allocator_chunk;
        while core::ptr::addr_of_mut!((*chunk).entry)
            != core::ptr::addr_of_mut!((*(*allocator).pools.offset(i as isize)).chunks)
        {
            list_remove(core::ptr::addr_of_mut!((*chunk).entry));
            (*(*allocator).ops)
                .allocator_destroy_chunk
                .unwrap_unchecked()(chunk);
            chunk = chunk2;
            chunk2 = ((*chunk).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_allocator_chunk, entry))
                as *mut wined3d_allocator_chunk;
        }
        i = i.wrapping_add(1);
    }
    free((*allocator).pools as *mut core::ffi::c_void);

    next = (*allocator).free;
    loop {
        block = next;
        if block.is_null() {
            break;
        }
        next = (*block).parent;
        free(block as *mut core::ffi::c_void);
    }
}

unsafe extern "C" fn wined3d_allocator_chunk_allocate(
    chunk: *mut wined3d_allocator_chunk,
    order: u32,
) -> *mut wined3d_allocator_block {
    let mut block: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut left: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut right: *mut wined3d_allocator_block = core::ptr::null_mut();
    let mut i: u32 = order;

    while i != 0 {
        if list_empty(core::ptr::addr_of_mut!((*chunk).available[i as usize])) == 0 {
            break;
        }
        i = i.wrapping_sub(1);
    }

    if list_empty(core::ptr::addr_of_mut!((*chunk).available[i as usize])) != 0 {
        return core::ptr::null_mut();
    }

    block = LIST_ENTRY!(
        list_head(core::ptr::addr_of_mut!((*chunk).available[i as usize])),
        wined3d_allocator_block,
        entry,
    );
    list_remove(core::ptr::addr_of_mut!((*block).entry));
    (*block).free = false;

    while i < order {
        left = wined3d_allocator_acquire_block((*chunk).allocator);
        if left.is_null() {
            ERR!("Failed to allocate left.\n");
            break;
        }

        right = wined3d_allocator_acquire_block((*chunk).allocator);
        if right.is_null() {
            ERR!("Failed to allocate right.\n");
            wined3d_allocator_release_block((*chunk).allocator, left);
            break;
        }

        wined3d_allocator_block_init(
            left,
            chunk,
            block,
            right,
            (*block).order.wrapping_add(1),
            (*block).offset,
            false,
        );
        wined3d_allocator_block_init(
            right,
            chunk,
            block,
            left,
            (*block).order.wrapping_add(1),
            (*block)
                .offset
                .wrapping_add((WINED3D_ALLOCATOR_CHUNK_SIZE >> (*left).order) as usize),
            true,
        );
        list_add_head(
            core::ptr::addr_of_mut!((*chunk).available[(*right).order as usize]),
            core::ptr::addr_of_mut!((*right).entry),
        );

        block = left;
        i = i.wrapping_add(1);
    }

    block
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_allocate(
    mut allocator: *mut wined3d_allocator,
    mut context: *mut wined3d_context,
    mut memory_type: ::core::ffi::c_uint,
    mut size: size_t,
) -> *mut wined3d_allocator_block {
    let mut chunk: *mut wined3d_allocator_chunk =
        ::core::ptr::null_mut::<wined3d_allocator_chunk>();
    let mut block: *mut wined3d_allocator_block =
        ::core::ptr::null_mut::<wined3d_allocator_block>();
    let mut order: ::core::ffi::c_uint = 0;
    if size > ((WINED3D_ALLOCATOR_CHUNK_SIZE / 2) as size_t) {
        return ::core::ptr::null_mut::<wined3d_allocator_block>();
    }
    if size < WINED3D_ALLOCATOR_MIN_BLOCK_SIZE as size_t {
        order = (WINED3D_ALLOCATOR_CHUNK_ORDER_COUNT - 1) as ::core::ffi::c_uint;
    } else {
        order = wined3d_log2i(
            (WINED3D_ALLOCATOR_CHUNK_SIZE as size_t).wrapping_div(size) as ::core::ffi::c_uint
        );
    }
    chunk = ((*(*allocator).pools.offset(memory_type as isize))
        .chunks
        .next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(wined3d_allocator_chunk, entry) as isize))
        as *mut wined3d_allocator_chunk;
    while &raw mut (*chunk).entry
        != &raw mut (*(*allocator).pools.offset(memory_type as isize)).chunks
    {
        block = wined3d_allocator_chunk_allocate(chunk, order);
        if !block.is_null() {
            return block;
        }
        chunk = ((*chunk).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(wined3d_allocator_chunk, entry) as isize))
            as *mut wined3d_allocator_chunk;
    }
    chunk = (*(*allocator).ops)
        .allocator_create_chunk
        .unwrap_unchecked()(
        allocator,
        context,
        memory_type,
        WINED3D_ALLOCATOR_CHUNK_SIZE as size_t,
    );
    if chunk.is_null() {
        return ::core::ptr::null_mut::<wined3d_allocator_block>();
    }
    block = wined3d_allocator_chunk_allocate(chunk, order);
    if block.is_null() {
        return ::core::ptr::null_mut::<wined3d_allocator_block>();
    }
    return block;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_allocator_init(allocator: *mut wined3d_allocator, pool_count: size_t, allocator_ops: *const wined3d_allocator_ops) -> bool {
    let mut i: size_t = 0;

    (*allocator).ops = allocator_ops;
    (*allocator).pool_count = pool_count;
    (*allocator).pools = calloc(
        (pool_count) as _,
        (core::mem::size_of::<wined3d_allocator_pool>() as size_t) as _,
    ) as *mut wined3d_allocator_pool;
    if (*allocator).pools.is_null() {
        return false;
    }
    while i < pool_count {
        list_init(core::ptr::addr_of_mut!(
            (*(*allocator).pools.offset(i as isize)).chunks
        ));
        i = i.wrapping_add(1);
    }

    (*allocator).free = core::ptr::null_mut::<wined3d_allocator_block>();

    return true;
}
