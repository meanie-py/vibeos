use crate::d3dwine::context_gl::context_state_drawbuf;
use crate::d3dwine::context_gl::context_state_fb;
use crate::d3dwine::context_gl::wined3d_context_gl_enable_clip_distances;
use crate::d3dwine::context_gl::wined3d_context_gl_end_transform_feedback;
use crate::d3dwine::context_gl::wined3d_context_gl_update_stream_sources;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut simple_states: [::core::ffi::c_uint; 29] = [
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
        } else {
            WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
            as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
        (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
            + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            + 8 as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
    ];

#[no_mangle]
pub static mut misc_state_template_gl: [wined3d_state_entry_template; 54] = [
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_UNIFORM_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_cb_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
                as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_shader_resource_binding
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_uav_binding
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_SHADER_IMAGE_LOAD_STORE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_uav_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_cs_resource_binding
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_cs_uav_binding
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_SHADER_IMAGE_LOAD_STORE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_uav_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_so
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_VERSION_3_2 as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_so_warn
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    blend_dbb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_DRAW_BUFFERS_BLEND as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    blend_db2
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: EXT_DRAW_BUFFERS2 as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    blend
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_blend_factor
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: EXT_BLEND_COLOR as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_blend_factor_w
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_sample_mask
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_TEXTURE_MULTISAMPLE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_sample_mask_w
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    depth_stencil_2s
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: EXT_STENCIL_TWO_SIDE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    depth_stencil
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    streamsrc
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    vdecl_miscpart
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    rasterizer_cc
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_CLIP_CONTROL as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    rasterizer
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    scissorrect
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    viewport_miscpart_cc
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_CLIP_CONTROL as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    viewport_miscpart
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    indexbuffer
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_VERTEX_BUFFER_OBJECT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_linepattern
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_LEGACY_CONTEXT as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_LINEPATTERN as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_linepattern_w
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_DITHERENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_DITHERENABLE as ::core::ffi::c_int
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_ditherenable
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_msaa
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_MULTISAMPLE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_MULTISAMPLEANTIALIAS as ::core::ffi::c_int
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_msaa_w
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: ARB_DRAW_ELEMENTS_BASE_VERTEX as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: None,
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                    * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 8 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_uint,
                apply: Some(
                    context_state_fb
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    context_state_drawbuf
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_shader
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_shader
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_shader
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
            } else {
                WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                        * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                    + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 8 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
                    + 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: (if WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                    != WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_int
                } else {
                    WINED3D_RS_BLENDOPALPHA as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + (8 as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            * (WINED3D_TSS_CONSTANT as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                        + WINED3D_TSS_CONSTANT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 8 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int
                }) as ::core::ffi::c_uint,
                apply: Some(
                    state_compute_shader
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
        wined3d_state_entry_template {
            state: 0 as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: 0 as ::core::ffi::c_uint,
                apply: None,
            },
            extension: WINED3D_GL_EXT_NONE as ::core::ffi::c_int as ::core::ffi::c_uint,
        },
    ];

#[no_mangle]
pub static mut none_vertex_pipe: wined3d_vertex_pipe_ops = unsafe {
    wined3d_vertex_pipe_ops {
        vp_apply_draw_state: Some(
            none_pipe_apply_draw_state
                as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state) -> (),
        ),
        vp_disable: Some(none_pipe_disable as unsafe extern "C" fn(*const wined3d_context) -> ()),
        vp_get_caps: Some(
            vp_none_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut wined3d_vertex_caps) -> (),
        ),
        vp_get_emul_mask: Some(
            vp_none_get_emul_mask
                as unsafe extern "C" fn(*const wined3d_adapter) -> ::core::ffi::c_uint,
        ),
        vp_alloc: Some(
            none_alloc
                as unsafe extern "C" fn(
                    *const wined3d_shader_backend_ops,
                    *mut ::core::ffi::c_void,
                ) -> *mut ::core::ffi::c_void,
        ),
        vp_free: Some(
            none_free as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        vp_states: ::core::ptr::null::<wined3d_state_entry_template>(),
    }
};

#[no_mangle]
pub static mut none_fragment_pipe: wined3d_fragment_pipe_ops = unsafe {
    wined3d_fragment_pipe_ops {
        fp_apply_draw_state: Some(
            none_pipe_apply_draw_state
                as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state) -> (),
        ),
        fp_disable: Some(none_pipe_disable as unsafe extern "C" fn(*const wined3d_context) -> ()),
        get_caps: Some(
            fp_none_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut fragment_caps) -> (),
        ),
        get_emul_mask: Some(
            fp_none_get_emul_mask
                as unsafe extern "C" fn(*const wined3d_adapter) -> ::core::ffi::c_uint,
        ),
        alloc_private: Some(
            none_alloc
                as unsafe extern "C" fn(
                    *const wined3d_shader_backend_ops,
                    *mut ::core::ffi::c_void,
                ) -> *mut ::core::ffi::c_void,
        ),
        free_private: Some(
            none_free as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        allocate_context_data: Some(
            ffp_none_context_alloc as unsafe extern "C" fn(*mut wined3d_context) -> BOOL,
        ),
        free_context_data: Some(
            ffp_none_context_free as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        color_fixup_supported: Some(
            fp_none_color_fixup_supported as unsafe extern "C" fn(color_fixup_desc) -> BOOL,
        ),
        states: ::core::ptr::null::<wined3d_state_entry_template>(),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d): debug-channel setup macro, handled by the crate's DPRINT/WARN/ERR macros.

unsafe extern "C" fn state_undefined(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    ERR!("Undefined state %s (%#lx).\n", debug_d3dstate(state_id), state_id);
}
#[no_mangle]
pub unsafe extern "C" fn state_nop(mut context: *mut wined3d_context, mut state: *const wined3d_state, mut state_id: DWORD) {
    WINE_TRACE!("%s: nop in current pipe config.\n", debug_d3dstate(state_id as uint32_t));
}

unsafe extern "C" fn fillmode(
    r: *const wined3d_rasterizer_state,
    gl_info: *const wined3d_gl_info,
) {
    let mut mode: wined3d_fill_mode = if !r.is_null() {
        (*r).desc.fill_mode
    } else {
        WINED3D_FILL_SOLID
    };

    match mode {
        WINED3D_FILL_POINT => {
            (*gl_info).gl_ops.gl.p_glPolygonMode.unwrap_unchecked()(
                GL_FRONT_AND_BACK as GLenum,
                GL_POINT as GLenum,
            );
            checkGLcall!(gl_info, b"glPolygonMode(GL_FRONT_AND_BACK, GL_POINT)\0".as_ptr() as *const i8);
        }
        WINED3D_FILL_WIREFRAME => {
            (*gl_info).gl_ops.gl.p_glPolygonMode.unwrap_unchecked()(
                GL_FRONT_AND_BACK as GLenum,
                GL_LINE as GLenum,
            );
            checkGLcall!(gl_info, b"glPolygonMode(GL_FRONT_AND_BACK, GL_LINE)\0".as_ptr() as *const i8);
        }
        WINED3D_FILL_SOLID => {
            (*gl_info).gl_ops.gl.p_glPolygonMode.unwrap_unchecked()(
                GL_FRONT_AND_BACK as GLenum,
                GL_FILL as GLenum,
            );
            checkGLcall!(gl_info, b"glPolygonMode(GL_FRONT_AND_BACK, GL_FILL)\0".as_ptr() as *const i8);
        }
        _ => {
            FIXME!("Unrecognized fill mode %#x.\n", mode);
        }
    }
}

unsafe extern "C" fn cullmode(
    r: *const wined3d_rasterizer_state,
    gl_info: *const wined3d_gl_info,
) {
    let mut mode: wined3d_cull = if !r.is_null() {
        (*r).desc.cull_mode
    } else {
        WINED3D_CULL_BACK as wined3d_cull
    };

    /* glFrontFace() is set in context.c at context init and on an
     * offscreen / onscreen rendering switch. */
    match mode as ::core::ffi::c_uint {
        WINED3D_CULL_NONE => {
            (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
                GL_CULL_FACE as GLenum,
            );
            checkGLcall!(gl_info, "glDisable GL_CULL_FACE");
        }
        WINED3D_CULL_FRONT => {
            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                GL_CULL_FACE as GLenum,
            );
            checkGLcall!(gl_info, "glEnable GL_CULL_FACE");
            (*gl_info).gl_ops.gl.p_glCullFace.unwrap_unchecked()(
                GL_FRONT as GLenum,
            );
            checkGLcall!(gl_info, "glCullFace(GL_FRONT)");
        }
        WINED3D_CULL_BACK => {
            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                GL_CULL_FACE as GLenum,
            );
            checkGLcall!(gl_info, "glEnable GL_CULL_FACE");
            (*gl_info).gl_ops.gl.p_glCullFace.unwrap_unchecked()(
                GL_BACK as GLenum,
            );
            checkGLcall!(gl_info, "glCullFace(GL_BACK)");
        }
        _ => {
            FIXME!("Unrecognized cull mode %#x.\n", mode as ::core::ffi::c_uint);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn state_shademode(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;

    match (*state).render_states[WINED3D_RS_SHADEMODE as usize] {
        WINED3D_SHADE_FLAT => {
            (*gl_info).gl_ops.gl.p_glShadeModel.unwrap_unchecked()(GL_FLAT);
            checkGLcall!(gl_info, "glShadeModel(GL_FLAT)");
        }
        /* WINED3D_SHADE_PHONG in practice is the same as WINED3D_SHADE_GOURAUD
         * in D3D. */
        WINED3D_SHADE_GOURAUD | WINED3D_SHADE_PHONG => {
            (*gl_info).gl_ops.gl.p_glShadeModel.unwrap_unchecked()(GL_SMOOTH);
            checkGLcall!(gl_info, "glShadeModel(GL_SMOOTH)");
        }
        _ => {
            FIXME!("Unrecognized shade mode %#x.\n",
                    (*state).render_states[WINED3D_RS_SHADEMODE as usize]);
        }
    }
}

unsafe extern "C" fn state_ditherenable(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;

    if (*state).render_states[WINED3D_RS_DITHERENABLE as usize] != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_DITHER as GLenum);
        checkGLcall!(gl_info, b"glEnable GL_DITHER\0".as_ptr() as *const i8);
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_DITHER as GLenum);
        checkGLcall!(gl_info, b"glDisable GL_DITHER\0".as_ptr() as *const i8);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_gl_compare_func(f: wined3d_cmp_func) -> GLenum {
    match f {
        WINED3D_CMP_NEVER => {
            return GL_NEVER;
        }
        WINED3D_CMP_LESS => {
            return GL_LESS;
        }
        WINED3D_CMP_EQUAL => {
            return GL_EQUAL;
        }
        WINED3D_CMP_LESSEQUAL => {
            return GL_LEQUAL;
        }
        WINED3D_CMP_GREATER => {
            return GL_GREATER;
        }
        WINED3D_CMP_NOTEQUAL => {
            return GL_NOTEQUAL;
        }
        WINED3D_CMP_GREATEREQUAL => {
            return GL_GEQUAL;
        }
        WINED3D_CMP_ALWAYS => {
            return GL_ALWAYS;
        }
        _ => {
            if f as u64 == 0 {
                WARN!("Unrecognized compare function %#x.\n", f);
            } else {
                FIXME!("Unrecognized compare function %#x.\n", f);
            }
            return GL_NONE;
        }
    }
}

unsafe extern "C" fn gl_blend_op(gl_info: *const wined3d_gl_info, op: wined3d_blend_op) -> GLenum {
    match op {
        WINED3D_BLEND_OP_ADD => {
            return GL_FUNC_ADD as GLenum;
        }
        WINED3D_BLEND_OP_SUBTRACT => {
            return (if (*gl_info).supported[EXT_BLEND_SUBTRACT as usize] != 0 {
                GL_FUNC_SUBTRACT
            } else {
                GL_FUNC_ADD
            }) as GLenum;
        }
        WINED3D_BLEND_OP_REVSUBTRACT => {
            return (if (*gl_info).supported[EXT_BLEND_SUBTRACT as usize] != 0 {
                GL_FUNC_REVERSE_SUBTRACT
            } else {
                GL_FUNC_ADD
            }) as GLenum;
        }
        WINED3D_BLEND_OP_MIN => {
            return (if (*gl_info).supported[EXT_BLEND_MINMAX as usize] != 0 {
                GL_MIN
            } else {
                GL_FUNC_ADD
            }) as GLenum;
        }
        WINED3D_BLEND_OP_MAX => {
            return (if (*gl_info).supported[EXT_BLEND_MINMAX as usize] != 0 {
                GL_MAX
            } else {
                GL_FUNC_ADD
            }) as GLenum;
        }
        _ => {
            if op as u64 == 0 {
                WARN!("Unhandled blend op %#x.\n", op);
            } else {
                FIXME!("Unhandled blend op %#x.\n", op);
            }
            return GL_FUNC_ADD as GLenum;
        }
    };
}

unsafe extern "C" fn blendop(state: *const wined3d_state, gl_info: *const wined3d_gl_info) {
    let b: *const wined3d_blend_state = (*state).blend_state;
    let mut blend_equation_alpha: GLenum = GL_FUNC_ADD_EXT;
    let mut blend_equation: GLenum = GL_FUNC_ADD_EXT;

    if (*gl_info).supported[WINED3D_GL_BLEND_EQUATION as usize] == 0 {
        WARN!("Unsupported in local OpenGL implementation: glBlendEquation.\n");
        return;
    }

    /* BLENDOPALPHA requires GL_EXT_blend_equation_separate, so make sure it is around */
    if ((*b).desc.rt[0 as usize].op_alpha as u32) != 0
        && (*gl_info).supported[EXT_BLEND_EQUATION_SEPARATE as usize] == 0
    {
        WARN!("Unsupported in local OpenGL implementation: glBlendEquationSeparate.\n");
        return;
    }

    blend_equation = gl_blend_op(gl_info, (*b).desc.rt[0 as usize].op);
    blend_equation_alpha = gl_blend_op(gl_info, (*b).desc.rt[0 as usize].op_alpha);

    if (*b).desc.rt[0 as usize].op as u32
        != (*b).desc.rt[0 as usize].op_alpha as u32
    {
        ((*gl_info).gl_ops.ext.p_glBlendEquationSeparate.unwrap_unchecked())(blend_equation, blend_equation_alpha);
        checkGLcall!(gl_info, "glBlendEquationSeparate");
    } else {
        ((*gl_info).gl_ops.ext.p_glBlendEquation.unwrap_unchecked())(blend_equation);
        checkGLcall!(gl_info, "glBlendEquation");
    }
}

unsafe extern "C" fn gl_blend_factor(factor: wined3d_blend, dst_format: *const wined3d_format) -> GLenum {
    match factor {
        WINED3D_BLEND_ZERO => {
            return GL_ZERO as GLenum;
        }
        WINED3D_BLEND_ONE => {
            return GL_ONE as GLenum;
        }
        WINED3D_BLEND_SRCCOLOR => {
            return GL_SRC_COLOR as GLenum;
        }
        WINED3D_BLEND_INVSRCCOLOR => {
            return GL_ONE_MINUS_SRC_COLOR as GLenum;
        }
        WINED3D_BLEND_SRCALPHA => {
            return GL_SRC_ALPHA as GLenum;
        }
        WINED3D_BLEND_INVSRCALPHA => {
            return GL_ONE_MINUS_SRC_ALPHA as GLenum;
        }
        WINED3D_BLEND_DESTCOLOR => {
            return GL_DST_COLOR as GLenum;
        }
        WINED3D_BLEND_INVDESTCOLOR => {
            return GL_ONE_MINUS_DST_COLOR as GLenum;
        }
        // To compensate for the lack of format switching with backbuffer
        // offscreen rendering, and with onscreen rendering, we modify the
        // alpha test parameters for (INV)DESTALPHA if the render target
        // doesn't support alpha blending. A nonexistent alpha channel
        // returns 1.0, so WINED3D_BLEND_DESTALPHA becomes GL_ONE, and
        // WINED3D_BLEND_INVDESTALPHA becomes GL_ZERO.
        WINED3D_BLEND_DESTALPHA => {
            return (if (*dst_format).alpha_size != 0 {
                GL_DST_ALPHA
            } else {
                GL_ONE
            }) as GLenum;
        }
        WINED3D_BLEND_INVDESTALPHA => {
            return (if (*dst_format).alpha_size != 0 {
                GL_ONE_MINUS_DST_ALPHA
            } else {
                GL_ZERO
            }) as GLenum;
        }
        WINED3D_BLEND_SRCALPHASAT => {
            return GL_SRC_ALPHA_SATURATE as GLenum;
        }
        WINED3D_BLEND_BLENDFACTOR => {
            return GL_CONSTANT_COLOR_EXT as GLenum;
        }
        WINED3D_BLEND_INVBLENDFACTOR => {
            return GL_ONE_MINUS_CONSTANT_COLOR_EXT as GLenum;
        }
        WINED3D_BLEND_SRC1COLOR => {
            return GL_SRC1_COLOR as GLenum;
        }
        WINED3D_BLEND_INVSRC1COLOR => {
            return GL_ONE_MINUS_SRC1_COLOR as GLenum;
        }
        WINED3D_BLEND_SRC1ALPHA => {
            return GL_SRC1_ALPHA as GLenum;
        }
        WINED3D_BLEND_INVSRC1ALPHA => {
            return GL_ONE_MINUS_SRC1_ALPHA as GLenum;
        }
        _ => {
            if factor == 0 {
                WARN!("Unhandled blend factor %#x.\n", factor);
            } else {
                FIXME!("Unhandled blend factor %#x.\n", factor);
            }
            return GL_NONE as GLenum;
        }
    }
}

unsafe extern "C" fn gl_blend_from_d3d(src_blend: *mut GLenum, dst_blend: *mut GLenum, d3d_src_blend: wined3d_blend, d3d_dst_blend: wined3d_blend, rt_format: *const wined3d_format) {
    /* WINED3D_BLEND_BOTHSRCALPHA and WINED3D_BLEND_BOTHINVSRCALPHA are legacy
     * source blending values which are still valid up to d3d9. They should
     * not occur as dest blend values. */
    if d3d_src_blend as u32 == WINED3D_BLEND_BOTHSRCALPHA as u32 {
        *src_blend = GL_SRC_ALPHA as GLenum;
        *dst_blend = GL_ONE_MINUS_SRC_ALPHA as GLenum;
    } else if d3d_src_blend as u32 == WINED3D_BLEND_BOTHINVSRCALPHA as u32 {
        *src_blend = GL_ONE_MINUS_SRC_ALPHA as GLenum;
        *dst_blend = GL_SRC_ALPHA as GLenum;
    } else {
        *src_blend = gl_blend_factor(d3d_src_blend, rt_format);
        *dst_blend = gl_blend_factor(d3d_dst_blend, rt_format);
    }
}

unsafe extern "C" fn state_blend_factor_w(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    WARN!("Unsupported in local OpenGL implementation: glBlendColor.\n");
}

unsafe extern "C" fn state_blend_factor(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let factor: *const wined3d_color = core::ptr::addr_of!((*state).blend_factor);

    TRACE!("Setting blend factor to %s.\n", debug_color(factor));

    ((*gl_info).gl_ops.ext.p_glBlendColor.unwrap_unchecked())((*factor).r, (*factor).g, (*factor).b, (*factor).a);
    checkGLcall!(gl_info, "glBlendColor");
}

unsafe extern "C" fn is_blend_enabled(context: *mut wined3d_context, state: *const wined3d_state, index: UINT) -> BOOL {
    let b: *const wined3d_blend_state = (*state).blend_state;

    if (*state).fb.render_targets[index as usize].is_null() {
        return (FALSE) as BOOL;
    }

    if (*b).desc.rt[index as usize].enable == 0 {
        return (FALSE) as BOOL;
    }

    /* Disable blending in all cases even without pixel shaders.
     * With blending on we could face a big performance penalty.
     * The d3d9 visual test confirms the behavior. */
    if ((*(*state).fb.render_targets[index as usize]).format_caps & WINED3D_FORMAT_CAP_POSTPIXELSHADER_BLENDING) == 0 {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn blend(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let b: *const wined3d_blend_state = (*state).blend_state;
    let mut rt_format: *const wined3d_format = core::ptr::null();
    let mut src_blend: GLenum = 0;
    let mut dst_blend: GLenum = 0;
    let mut mask: u32 = 0;

    if (*gl_info).supported[ARB_MULTISAMPLE as usize] != 0 {
        if !b.is_null() && (*b).desc.alpha_to_coverage != 0 {
            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
                GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum,
            );
        } else {
            (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
                GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum,
            );
        }
        checkGLcall!(gl_info, "glEnable GL_SAMPLE_ALPHA_TO_COVERAGE");
    }

    if !b.is_null() && (*b).desc.independent != 0 {
        WARN!("Independent blend is not supported by this GL implementation.\n");
    }

    mask = if !b.is_null() {
        (*b).desc.rt[0 as usize].writemask
    } else {
        0xf as u32
    };

    (*gl_info).gl_ops.gl.p_glColorMask.unwrap_unchecked()(
        (if mask & WINED3DCOLORWRITEENABLE_RED as u32 != 0 {
            GL_TRUE
        } else {
            GL_FALSE
        }) as GLboolean,
        (if mask & WINED3DCOLORWRITEENABLE_GREEN as u32 != 0 {
            GL_TRUE
        } else {
            GL_FALSE
        }) as GLboolean,
        (if mask & WINED3DCOLORWRITEENABLE_BLUE as u32 != 0 {
            GL_TRUE
        } else {
            GL_FALSE
        }) as GLboolean,
        (if mask & WINED3DCOLORWRITEENABLE_ALPHA as u32 != 0 {
            GL_TRUE
        } else {
            GL_FALSE
        }) as GLboolean,
    );
    checkGLcall!(gl_info, "glColorMask");

    if b.is_null() || is_blend_enabled(context, state, 0 as UINT) == 0 {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(
            GL_BLEND as GLenum,
        );
        checkGLcall!(gl_info, "glDisable GL_BLEND");
        return;
    }

    (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(
        GL_BLEND as GLenum,
    );
    checkGLcall!(gl_info, "glEnable GL_BLEND");

    rt_format = (*(*state).fb.render_targets[0 as usize]).format;

    gl_blend_from_d3d(
        &raw mut src_blend,
        &raw mut dst_blend,
        (*b).desc.rt[0 as usize].src,
        (*b).desc.rt[0 as usize].dst,
        rt_format,
    );

    blendop(state, gl_info);

    if (*b).desc.rt[0 as usize].src as u32
        != (*b).desc.rt[0 as usize].src_alpha as u32
        || (*b).desc.rt[0 as usize].dst as u32
            != (*b).desc.rt[0 as usize].dst_alpha as u32
    {
        let mut src_blend_alpha: GLenum = 0;
        let mut dst_blend_alpha: GLenum = 0;

        /* Separate alpha blending requires GL_EXT_blend_function_separate, so make sure it is around */
        if (*gl_info).supported[EXT_BLEND_FUNC_SEPARATE as usize] == 0 {
            WARN!("Unsupported in local OpenGL implementation: glBlendFuncSeparate.\n");
            return;
        }

        gl_blend_from_d3d(
            &raw mut src_blend_alpha,
            &raw mut dst_blend_alpha,
            (*b).desc.rt[0 as usize].src_alpha,
            (*b).desc.rt[0 as usize].dst_alpha,
            rt_format,
        );

        ((*gl_info).gl_ops.ext.p_glBlendFuncSeparate.unwrap_unchecked())(
            src_blend,
            dst_blend,
            src_blend_alpha,
            dst_blend_alpha,
        );
        checkGLcall!(gl_info, "glBlendFuncSeparate");
    } else {
        TRACE!(
            "glBlendFunc src=%x, dst=%x.\n",
            src_blend,
            dst_blend
        );
        (*gl_info).gl_ops.gl.p_glBlendFunc.unwrap_unchecked()(
            src_blend,
            dst_blend,
        );
        checkGLcall!(gl_info, "glBlendFunc");
    }
}

unsafe extern "C" fn set_color_mask(gl_info: *const wined3d_gl_info, index: UINT, mask: DWORD) {
    ((*gl_info).gl_ops.ext.p_glColorMaski.unwrap_unchecked())(
        index,
        if (mask & WINED3DCOLORWRITEENABLE_RED) != 0 { (GL_TRUE as u8) } else { (GL_FALSE as u8) },
        if (mask & WINED3DCOLORWRITEENABLE_GREEN) != 0 { (GL_TRUE as u8) } else { (GL_FALSE as u8) },
        if (mask & WINED3DCOLORWRITEENABLE_BLUE) != 0 { (GL_TRUE as u8) } else { (GL_FALSE as u8) },
        if (mask & WINED3DCOLORWRITEENABLE_ALPHA) != 0 { (GL_TRUE as u8) } else { (GL_FALSE as u8) },
    );
    checkGLcall!(gl_info, "glColorMaski");
}

unsafe extern "C" fn blend_db2(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let mut src_blend: GLenum = core::mem::zeroed();
    let mut dst_blend: GLenum = core::mem::zeroed();
    let mut src_blend_alpha: GLenum = core::mem::zeroed();
    let mut dst_blend_alpha: GLenum = core::mem::zeroed();
    let b: *const wined3d_blend_state = (*state).blend_state;
    let mut rt_format: *const wined3d_format;
    let dual_source: BOOL = (!b.is_null() && (*b).dual_source != 0) as BOOL;
    let mut i: u32;

    if !b.is_null() && (*b).desc.alpha_to_coverage != 0 {
        (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum);
    } else {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum);
    }
    checkGLcall!(gl_info, "glEnable GL_SAMPLE_ALPHA_TO_COVERAGE");

    if (*context).last_was_dual_source_blend() != (dual_source) as u32 {
        /* Dual source blending changes the location of the output varyings. */
        (*context).set_shader_update_mask(((*context).shader_update_mask() | (1 << WINED3D_SHADER_TYPE_PIXEL)) as _);
        (*context).set_last_was_dual_source_blend((dual_source) as _);
    }

    if b.is_null() || (*b).desc.independent == 0 {
        blend(context, state, state_id);
        return;
    }

    rt_format = (*(*state).fb.render_targets[0 as usize]).format;
    gl_blend_from_d3d(
        &mut src_blend,
        &mut dst_blend,
        (*b).desc.rt[0 as usize].src,
        (*b).desc.rt[0 as usize].dst,
        rt_format,
    );
    gl_blend_from_d3d(
        &mut src_blend_alpha,
        &mut dst_blend_alpha,
        (*b).desc.rt[0 as usize].src_alpha,
        (*b).desc.rt[0 as usize].dst_alpha,
        rt_format,
    );

    ((*gl_info).gl_ops.ext.p_glBlendFuncSeparate.unwrap_unchecked())(src_blend, dst_blend, src_blend_alpha, dst_blend_alpha);
    checkGLcall!(gl_info, "glBlendFuncSeparate");

    ((*gl_info).gl_ops.ext.p_glBlendEquationSeparate.unwrap_unchecked())(
        gl_blend_op(gl_info, (*b).desc.rt[0 as usize].op),
        gl_blend_op(gl_info, (*b).desc.rt[0 as usize].op_alpha),
    );
    checkGLcall!(gl_info, "glBlendEquationSeparate");

    i = 0 as u32;
    while i < WINED3D_MAX_RENDER_TARGETS as u32 {
        'body: {
            set_color_mask(gl_info, i as UINT, (*b).desc.rt[i as usize].writemask as DWORD);

            if is_blend_enabled(context, state, i as UINT) == 0 {
                ((*gl_info).gl_ops.ext.p_glDisablei.unwrap_unchecked())(GL_BLEND as GLenum, i as GLuint);
                checkGLcall!(gl_info, "glDisablei GL_BLEND");
                break 'body;
            }

            ((*gl_info).gl_ops.ext.p_glEnablei.unwrap_unchecked())(GL_BLEND as GLenum, i as GLuint);
            checkGLcall!(gl_info, "glEnablei GL_BLEND");

            if (*b).desc.rt[i as usize].src != (*b).desc.rt[0 as usize].src
                || (*b).desc.rt[i as usize].dst != (*b).desc.rt[0 as usize].dst
                || (*b).desc.rt[i as usize].op != (*b).desc.rt[0 as usize].op
                || (*b).desc.rt[i as usize].src_alpha != (*b).desc.rt[0 as usize].src_alpha
                || (*b).desc.rt[i as usize].dst_alpha != (*b).desc.rt[0 as usize].dst_alpha
                || (*b).desc.rt[i as usize].op_alpha != (*b).desc.rt[0 as usize].op_alpha
            {
                WARN!("Independent blend equations and blend functions are not supported by this GL implementation.\n");
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn blend_dbb(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let b: *const wined3d_blend_state = (*state).blend_state;
    let dual_source: BOOL = (!b.is_null() && (*b).dual_source != 0) as BOOL;
    let mut i: UINT;

    if !b.is_null() && (*b).desc.alpha_to_coverage != 0 {
        (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum);
    } else {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_SAMPLE_ALPHA_TO_COVERAGE as GLenum);
    }
    checkGLcall!(gl_info, b"glEnable GL_SAMPLE_ALPHA_TO_COVERAGE\0".as_ptr() as *const i8);

    if (*context).last_was_dual_source_blend() != (dual_source) as u32 {
        /* Dual source blending changes the location of the output varyings. */
        (*context).set_shader_update_mask(((*context).shader_update_mask() | (1 << WINED3D_SHADER_TYPE_PIXEL)) as _);
        (*context).set_last_was_dual_source_blend((dual_source) as _);
    }

    if b.is_null() || (*b).desc.independent == 0 {
        blend(context, state, state_id);
        return;
    }

    i = 0;
    while i < WINED3D_MAX_RENDER_TARGETS {
        let mut src_blend: GLenum = 0;
        let mut dst_blend: GLenum = 0;
        let mut src_blend_alpha: GLenum = 0;
        let mut dst_blend_alpha: GLenum = 0;
        let mut rt_format: *const wined3d_format = core::ptr::null();

        'body: {
            set_color_mask(gl_info, i, (*b).desc.rt[i as usize].writemask as DWORD);

            if is_blend_enabled(context, state, i) == 0 {
                ((*gl_info).gl_ops.ext.p_glDisablei.unwrap_unchecked())(GL_BLEND, i);
                checkGLcall!(gl_info, b"glDisablei GL_BLEND\0".as_ptr() as *const i8);
                break 'body;
            }

            ((*gl_info).gl_ops.ext.p_glEnablei.unwrap_unchecked())(GL_BLEND, i);
            checkGLcall!(gl_info, b"glEnablei GL_BLEND\0".as_ptr() as *const i8);

            rt_format = (*(*state).fb.render_targets[i as usize]).format;
            gl_blend_from_d3d(
                core::ptr::addr_of_mut!(src_blend),
                core::ptr::addr_of_mut!(dst_blend),
                (*b).desc.rt[i as usize].src,
                (*b).desc.rt[i as usize].dst,
                rt_format,
            );
            gl_blend_from_d3d(
                core::ptr::addr_of_mut!(src_blend_alpha),
                core::ptr::addr_of_mut!(dst_blend_alpha),
                (*b).desc.rt[i as usize].src_alpha,
                (*b).desc.rt[i as usize].dst_alpha,
                rt_format,
            );

            ((*gl_info).gl_ops.ext.p_glBlendFuncSeparatei.unwrap_unchecked())(i, src_blend, dst_blend, src_blend_alpha, dst_blend_alpha);
            checkGLcall!(gl_info, b"glBlendFuncSeparatei\0".as_ptr() as *const i8);

            ((*gl_info).gl_ops.ext.p_glBlendEquationSeparatei.unwrap_unchecked())(
                i,
                gl_blend_op(gl_info, (*b).desc.rt[i as usize].op),
                gl_blend_op(gl_info, (*b).desc.rt[i as usize].op_alpha)
            );
            checkGLcall!(gl_info, b"glBlendEquationSeparatei\0".as_ptr() as *const i8);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn state_clipping(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut enable_mask: uint32_t;

    (*context).set_shader_update_mask(((*context).shader_update_mask() | ((1 as DWORD) << WINED3D_SHADER_TYPE_VERTEX)) as _);

    enable_mask = if (*state).render_states[WINED3D_RS_CLIPPING as usize] != 0 {
        (*state).render_states[WINED3D_RS_CLIPPLANEENABLE as usize]
    } else {
        0
    };
    wined3d_context_gl_enable_clip_distances(context_gl, enable_mask);
}

unsafe extern "C" fn renderstate_stencil_twosided(
    context: *mut wined3d_context,
    face: GLint,
    func: GLint,
    ref_0: GLint,
    mask: GLuint,
    stencilFail: GLint,
    depthFail: GLint,
    stencilPass: GLint,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;

    (*gl_info)
        .gl_ops
        .gl
        .p_glEnable
        .unwrap_unchecked()(GL_STENCIL_TEST_TWO_SIDE_EXT as GLenum);
    checkGLcall!(gl_info, b"glEnable(GL_STENCIL_TEST_TWO_SIDE_EXT)\0".as_ptr() as *const i8);
    ((*gl_info).gl_ops.ext.p_glActiveStencilFaceEXT.unwrap_unchecked())((face) as u32);
    checkGLcall!(gl_info, b"glActiveStencilFaceEXT(...)\0".as_ptr() as *const i8);
    (*gl_info)
        .gl_ops
        .gl
        .p_glStencilFunc
        .unwrap_unchecked()(func as GLenum, ref_0, mask);
    checkGLcall!(gl_info, b"glStencilFunc(...)\0".as_ptr() as *const i8);
    (*gl_info)
        .gl_ops
        .gl
        .p_glStencilOp
        .unwrap_unchecked()(
        stencilFail as GLenum,
        depthFail as GLenum,
        stencilPass as GLenum,
    );
    checkGLcall!(gl_info, b"glStencilOp(...)\0".as_ptr() as *const i8);
}

unsafe extern "C" fn gl_stencil_op(op: wined3d_stencil_op) -> GLenum {
    match op {
        WINED3D_STENCIL_OP_KEEP => {
            return GL_KEEP;
        }
        WINED3D_STENCIL_OP_ZERO => {
            return GL_ZERO;
        }
        WINED3D_STENCIL_OP_REPLACE => {
            return GL_REPLACE;
        }
        WINED3D_STENCIL_OP_INCR_SAT => {
            return GL_INCR;
        }
        WINED3D_STENCIL_OP_DECR_SAT => {
            return GL_DECR;
        }
        WINED3D_STENCIL_OP_INVERT => {
            return GL_INVERT;
        }
        WINED3D_STENCIL_OP_INCR => {
            return GL_INCR_WRAP;
        }
        WINED3D_STENCIL_OP_DECR => {
            return GL_DECR_WRAP;
        }
        _ => {
            if (op as u32) == 0 {
                WARN!("Unrecognized stencil op %#x.\n", op);
            } else {
                FIXME!("Unrecognized stencil op %#x.\n", op);
            }
            return GL_KEEP;
        }
    }
}

unsafe extern "C" fn state_stencil(
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;
    let mut func: GLint = 0;
    let mut func_back: GLint = 0;
    let mut ref_0: GLint = 0;
    let mut mask: GLuint = 0;
    let mut stencilFail: GLint = 0;
    let mut stencilFail_back: GLint = 0;
    let mut stencilPass: GLint = 0;
    let mut stencilPass_back: GLint = 0;
    let mut depthFail: GLint = 0;
    let mut depthFail_back: GLint = 0;

    /* No stencil test without a stencil buffer. */
    if (*state).fb.depth_stencil.is_null() || d.is_null() || (*d).desc.stencil == 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_STENCIL_TEST as GLenum);
        checkGLcall!(gl_info, "glDisable GL_STENCIL_TEST");
        return;
    }

    func = wined3d_gl_compare_func((*d).desc.front.func) as GLint;
    if func == 0 {
        func = GL_ALWAYS as GLint;
    }
    func_back = wined3d_gl_compare_func((*d).desc.back.func) as GLint;
    if func_back == 0 {
        func_back = GL_ALWAYS as GLint;
    }
    mask = (*d).desc.stencil_read_mask as GLuint;
    ref_0 = ((*state).stencil_ref as uint32_t
        & wined3d_mask_from_size(
            (*(*(*state).fb.depth_stencil).format).stencil_size as ::core::ffi::c_uint,
        )) as GLint;
    stencilFail = gl_stencil_op((*d).desc.front.fail_op) as GLint;
    depthFail = gl_stencil_op((*d).desc.front.depth_fail_op) as GLint;
    stencilPass = gl_stencil_op((*d).desc.front.pass_op) as GLint;
    stencilFail_back = gl_stencil_op((*d).desc.back.fail_op) as GLint;
    depthFail_back = gl_stencil_op((*d).desc.back.depth_fail_op) as GLint;
    stencilPass_back = gl_stencil_op((*d).desc.back.pass_op) as GLint;

    if memcmp(
        core::ptr::addr_of!((*d).desc.front) as *const ::core::ffi::c_void,
        core::ptr::addr_of!((*d).desc.back) as *const ::core::ffi::c_void,
        core::mem::size_of::<wined3d_stencil_op_desc>() as usize,
    ) != 0
    {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_STENCIL_TEST as GLenum);
        checkGLcall!(gl_info, "glEnable GL_STENCIL_TEST");

        if (*gl_info).supported[WINED3D_GL_VERSION_2_0 as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glStencilFuncSeparate.unwrap_unchecked())(
                GL_FRONT as GLenum,
                func as GLenum,
                ref_0,
                mask,
            );
            ((*gl_info).gl_ops.ext.p_glStencilOpSeparate.unwrap_unchecked())(
                GL_FRONT as GLenum,
                stencilFail as GLenum,
                depthFail as GLenum,
                stencilPass as GLenum,
            );
            ((*gl_info).gl_ops.ext.p_glStencilFuncSeparate.unwrap_unchecked())(
                GL_BACK as GLenum,
                func_back as GLenum,
                ref_0,
                mask,
            );
            ((*gl_info).gl_ops.ext.p_glStencilOpSeparate.unwrap_unchecked())(
                GL_BACK as GLenum,
                stencilFail_back as GLenum,
                depthFail_back as GLenum,
                stencilPass_back as GLenum,
            );
            checkGLcall!(gl_info, "setting two sided stencil state");
        } else if (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0 {
            /* Apply back first, then front. This function calls glActiveStencilFaceEXT,
             * which has an effect on the code below too. If we apply the front face
             * afterwards, we are sure that the active stencil face is set to front,
             * and other stencil functions which do not use two sided stencil do not have
             * to set it back
             */
            renderstate_stencil_twosided(
                context,
                (GL_BACK as GLenum) as GLint,
                func_back,
                ref_0,
                mask,
                stencilFail_back,
                depthFail_back,
                stencilPass_back,
            );
            renderstate_stencil_twosided(
                context,
                (GL_FRONT as GLenum) as GLint,
                func,
                ref_0,
                mask,
                stencilFail,
                depthFail,
                stencilPass,
            );
        } else if (*gl_info).supported[ATI_SEPARATE_STENCIL as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glStencilFuncSeparateATI.unwrap_unchecked())(
                func as GLenum,
                func_back as GLenum,
                ref_0,
                mask,
            );
            checkGLcall!(gl_info, "glStencilFuncSeparateATI(...)");
            ((*gl_info).gl_ops.ext.p_glStencilOpSeparateATI.unwrap_unchecked())(
                GL_FRONT as GLenum,
                stencilFail as GLenum,
                depthFail as GLenum,
                stencilPass as GLenum,
            );
            checkGLcall!(gl_info, "glStencilOpSeparateATI(GL_FRONT, ...)");
            ((*gl_info).gl_ops.ext.p_glStencilOpSeparateATI.unwrap_unchecked())(
                GL_BACK as GLenum,
                stencilFail_back as GLenum,
                depthFail_back as GLenum,
                stencilPass_back as GLenum,
            );
            checkGLcall!(gl_info, "glStencilOpSeparateATI(GL_BACK, ...)");
        } else {
            FIXME!(
                "Separate (two sided) stencil not supported on this version of OpenGL. Caps weren't honored?\n"
            );
        }
    } else {
        if (*gl_info).supported[EXT_STENCIL_TWO_SIDE as usize] != 0 {
            (*gl_info)
                .gl_ops
                .gl
                .p_glDisable
                .unwrap_unchecked()(GL_STENCIL_TEST_TWO_SIDE_EXT as GLenum);
            checkGLcall!(gl_info, "glDisable(GL_STENCIL_TEST_TWO_SIDE_EXT)");
        }

        /* This code disables the ATI extension as well, since the standard stencil functions are equal
         * to calling the ATI functions with GL_FRONT_AND_BACK as face parameter
         */
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_STENCIL_TEST as GLenum);
        checkGLcall!(gl_info, "glEnable GL_STENCIL_TEST");
        (*gl_info)
            .gl_ops
            .gl
            .p_glStencilFunc
            .unwrap_unchecked()(func as GLenum, ref_0, mask);
        checkGLcall!(gl_info, "glStencilFunc(...)");
        (*gl_info)
            .gl_ops
            .gl
            .p_glStencilOp
            .unwrap_unchecked()(
            stencilFail as GLenum,
            depthFail as GLenum,
            stencilPass as GLenum,
        );
        checkGLcall!(gl_info, "glStencilOp(...)");
    }
}

unsafe extern "C" fn depth(mut context: *mut wined3d_context, mut state: *const wined3d_state) {
    let mut gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let mut d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;
    let mut enable_depth: BOOL = if !d.is_null() { (*d).desc.depth } else { (TRUE as i32) };
    let mut depth_func: GLenum = GL_LESS;

    if (*state).fb.depth_stencil.is_null() {
        enable_depth = (FALSE as i32);
    }

    if enable_depth != 0 {
        (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_DEPTH_TEST);
        checkGLcall!(gl_info, b"glEnable GL_DEPTH_TEST\0".as_ptr() as *const i8);
    } else {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_DEPTH_TEST);
        checkGLcall!(gl_info, b"glDisable GL_DEPTH_TEST\0".as_ptr() as *const i8);
    }

    if d.is_null() || (*d).desc.depth_write != 0 {
        (*gl_info).gl_ops.gl.p_glDepthMask.unwrap_unchecked()((GL_TRUE as u8));
        checkGLcall!(gl_info, b"glDepthMask(GL_TRUE)\0".as_ptr() as *const i8);
    } else {
        (*gl_info).gl_ops.gl.p_glDepthMask.unwrap_unchecked()((GL_FALSE as u8));
        checkGLcall!(gl_info, b"glDepthMask(GL_FALSE)\0".as_ptr() as *const i8);
    }

    if !d.is_null() {
        depth_func = wined3d_gl_compare_func((*d).desc.depth_func);
    }
    if depth_func != 0 {
        (*gl_info).gl_ops.gl.p_glDepthFunc.unwrap_unchecked()(depth_func);
        checkGLcall!(gl_info, b"glDepthFunc\0".as_ptr() as *const i8);
    }

    if (*gl_info).supported[EXT_DEPTH_BOUNDS_TEST as usize] != 0 {
        /* If min is larger than max, an INVALID_VALUE error is generated.
         * In d3d9, the test is not performed in this case. */
        if (*state).depth_bounds_enable as i32 != 0
            && (*state).depth_bounds_min <= (*state).depth_bounds_max
        {
            (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_DEPTH_BOUNDS_TEST_EXT);
            checkGLcall!(gl_info, b"glEnable(GL_DEPTH_BOUNDS_TEST_EXT)\0".as_ptr() as *const i8);
            ((*gl_info).gl_ops.ext.p_glDepthBoundsEXT.unwrap_unchecked())(
                (*state).depth_bounds_min as GLclampd,
                (*state).depth_bounds_max as GLclampd,
            );
            checkGLcall!(gl_info, b"glDepthBoundsEXT\0".as_ptr() as *const i8);
        } else {
            (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_DEPTH_BOUNDS_TEST_EXT);
            checkGLcall!(gl_info, b"glDisable(GL_DEPTH_BOUNDS_TEST_EXT)\0".as_ptr() as *const i8);
        }
    }

    if (*context).stream_info.position_transformed() != 0 {
        (*context).constant_update_mask |= WINED3D_SHADER_CONST_FFP_PROJ as DWORD;
    }
}

unsafe extern "C" fn depth_stencil(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;
    let mut stencil_write_mask: GLuint = 0 as GLuint;

    depth(context, state);
    state_stencil(context, state);

    if !(*state).fb.depth_stencil.is_null() {
        stencil_write_mask = (if !d.is_null() {
            (*d).desc.stencil_write_mask
        } else {
            !(0 as ::core::ffi::c_uint)
        }) as GLuint;
    }

    (*gl_info)
        .gl_ops
        .gl
        .p_glStencilMask
        .unwrap_unchecked()(stencil_write_mask);
    checkGLcall!(gl_info, "glStencilMask");
}

unsafe extern "C" fn depth_stencil_2s(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let d: *const wined3d_depth_stencil_state = (*state).depth_stencil_state;
    let mut stencil_write_mask: GLuint = 0;

    depth(context, state);
    state_stencil(context, state);

    if !(*state).fb.depth_stencil.is_null() {
        stencil_write_mask = (if !d.is_null() {
            (*d).desc.stencil_write_mask
        } else {
            !0
        }) as GLuint;
    }

    ((*gl_info).gl_ops.ext.p_glActiveStencilFaceEXT.unwrap_unchecked())(GL_BACK);
    checkGLcall!(gl_info, "glActiveStencilFaceEXT(GL_BACK)");
    (*gl_info).gl_ops.gl.p_glStencilMask.unwrap_unchecked()(stencil_write_mask);
    checkGLcall!(gl_info, "glStencilMask");
    ((*gl_info).gl_ops.ext.p_glActiveStencilFaceEXT.unwrap_unchecked())(GL_FRONT);
    checkGLcall!(gl_info, "glActiveStencilFaceEXT(GL_FRONT)");
    (*gl_info).gl_ops.gl.p_glStencilMask.unwrap_unchecked()(stencil_write_mask);
}

unsafe extern "C" fn state_linepattern(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    // ffp_gl.c's function-local `union { DWORD d; struct wined3d_line_pattern lp; } tmppattern`.
    #[repr(C)]
    #[derive(Copy, Clone)]
    union C2RustUnnamed_51 {
        d: DWORD,
        lp: wined3d_line_pattern,
    }
    let mut tmppattern: C2RustUnnamed_51 = C2RustUnnamed_51 { d: 0 };
    tmppattern.d = (*state).render_states[WINED3D_RS_LINEPATTERN as usize] as DWORD;

    WINE_TRACE!("Line pattern: repeat %d bits %x.\n", tmppattern.lp.repeat_factor as ::core::ffi::c_int, tmppattern.lp.line_pattern as ::core::ffi::c_int);

    if tmppattern.lp.repeat_factor != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glLineStipple
            .unwrap_unchecked()(
            tmppattern.lp.repeat_factor as GLint,
            tmppattern.lp.line_pattern as GLushort,
        );
        checkGLcall!(gl_info, "glLineStipple(repeat, linepattern)");
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_LINE_STIPPLE as GLenum);
        checkGLcall!(gl_info, "glEnable(GL_LINE_STIPPLE);");
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_LINE_STIPPLE as GLenum);
        checkGLcall!(gl_info, "glDisable(GL_LINE_STIPPLE);");
    }
}

unsafe extern "C" fn state_linepattern_w(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    static mut once: core::ffi::c_uint = 0;
    let fresh15 = once;
    once = once.wrapping_add(1);
    if fresh15 == 0 {
        FIXME!("Setting line patterns is not supported in OpenGL core contexts.\n");
    }
}

unsafe extern "C" fn state_msaa_w(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    if (*state).render_states[WINED3D_RS_MULTISAMPLEANTIALIAS as usize] != 0 {
        WARN!("Multisample antialiasing not supported by GL.\n");
    }
}

unsafe extern "C" fn state_msaa(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;

    if (*state).render_states[WINED3D_RS_MULTISAMPLEANTIALIAS as usize] != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_MULTISAMPLE_ARB as GLenum);
        checkGLcall!(gl_info, "glEnable(GL_MULTISAMPLE_ARB)");
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_MULTISAMPLE_ARB as GLenum);
        checkGLcall!(gl_info, "glDisable(GL_MULTISAMPLE_ARB)");
    }
}

unsafe extern "C" fn line_antialias(
    r: *const wined3d_rasterizer_state,
    gl_info: *const wined3d_gl_info,
) {
    if !r.is_null() && (*r).desc.line_antialias != 0 {
        (*gl_info).gl_ops.gl.p_glEnable.unwrap_unchecked()(GL_LINE_SMOOTH);
        checkGLcall!(gl_info, b"glEnable(GL_LINE_SMOOTH)\0".as_ptr() as *const i8);
    } else {
        (*gl_info).gl_ops.gl.p_glDisable.unwrap_unchecked()(GL_LINE_SMOOTH);
        checkGLcall!(gl_info, b"glDisable(GL_LINE_SMOOTH)\0".as_ptr() as *const i8);
    }
}

unsafe extern "C" fn scissor(r: *const wined3d_rasterizer_state, gl_info: *const wined3d_gl_info) {
    if !r.is_null() && (*r).desc.scissor != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_SCISSOR_TEST as GLenum);
        checkGLcall!(gl_info, b"glEnable(GL_SCISSOR_TEST)\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_SCISSOR_TEST as GLenum);
        checkGLcall!(gl_info, b"glDisable(GL_SCISSOR_TEST)\0".as_ptr() as *const ::core::ffi::c_char);
    }
}

unsafe extern "C" fn depthbias(context: *mut wined3d_context, state: *const wined3d_state) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let r: *const wined3d_rasterizer_state = (*state).rasterizer_state;
    let scale_bias: f32 = if !r.is_null() {
        (*r).desc.scale_bias
    } else {
        0.0f32
    };
    // union { DWORD d; float f; } const_bias;
    let mut const_bias_d: DWORD = 0;
    const_bias_d = (if !r.is_null() {
        (*r).desc.depth_bias
    } else {
        0.0f32
    })
    .to_bits();

    if scale_bias != 0.0f32 || f32::from_bits(const_bias_d) != 0.0f32 {
        let depth: *const wined3d_rendertarget_view = (*state).fb.depth_stencil;
        let mut factor: f32 = 0.0f32;
        let mut units: f32 = 0.0f32;
        let mut scale: f32 = 0.0f32;
        let mut clamp: f32 = 0.0f32;

        clamp = if !r.is_null() {
            (*r).desc.depth_bias_clamp
        } else {
            0.0f32
        };

        if (*(*context).d3d_info).wined3d_creation_flags & WINED3D_LEGACY_DEPTH_BIAS != 0 {
            units = -(const_bias_d as f32);
            factor = units;
        } else {
            if !depth.is_null() {
                scale = (*(*depth).format).depth_bias_scale;
            } else {
                /* The context manager will reapply this state on a depth stencil change */
                scale = 0.0f32;
            }

            factor = scale_bias;
            units = f32::from_bits(const_bias_d) * scale;
        }

        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_POLYGON_OFFSET_FILL as GLenum);
        if (*gl_info).supported[ARB_POLYGON_OFFSET_CLAMP as usize] != 0 {
            (*gl_info)
                .gl_ops
                .ext
                .p_glPolygonOffsetClamp
                .unwrap_unchecked()(
                factor as GLfloat,
                units as GLfloat,
                clamp as GLfloat,
            );
        } else {
            if clamp != 0.0f32 {
                WARN!("Ignoring depth bias clamp %.8e.\n", clamp as f64);
            }
            (*gl_info)
                .gl_ops
                .gl
                .p_glPolygonOffset
                .unwrap_unchecked()(
                factor as GLfloat, units as GLfloat
            );
        }
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_POLYGON_OFFSET_FILL as GLenum);
    }

    checkGLcall!(gl_info, "depth bias");
}

unsafe extern "C" fn state_sample_mask(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let sample_mask: ::core::ffi::c_uint = (*state).sample_mask as ::core::ffi::c_uint;

    TRACE!("Setting sample mask to %#x.\n", sample_mask);
    if sample_mask != 0xffffffff {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_SAMPLE_MASK);
        checkGLcall!(gl_info, "glEnable GL_SAMPLE_MASK");
        ((*gl_info).gl_ops.ext.p_glSampleMaski.unwrap_unchecked())(0, sample_mask);
        checkGLcall!(gl_info, "glSampleMaski");
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_SAMPLE_MASK);
        checkGLcall!(gl_info, "glDisable GL_SAMPLE_MASK");
    }
}

unsafe extern "C" fn state_sample_mask_w(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    WARN!("Unsupported in local OpenGL implementation: glSampleMaski.\n");
}

unsafe extern "C" fn state_compute_shader(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    (*context).set_shader_update_mask(((*context).shader_update_mask() | (1 << WINED3D_SHADER_TYPE_COMPUTE)) as _);
}

unsafe extern "C" fn state_shader(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let shader_type: wined3d_shader_type = state_id.wrapping_sub(STATE_SHADER!(0) as DWORD) as wined3d_shader_type;
    (*context).set_shader_update_mask(((*context).shader_update_mask() | ((1 as DWORD) << (shader_type as DWORD))) as _);
}
#[no_mangle]
pub unsafe extern "C" fn clipplane(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    (*context).constant_update_mask |= WINED3D_SHADER_CONST_VS_CLIP_PLANES as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn ffp_vertex_update_clip_plane_constants(gl_info: *const wined3d_gl_info, state: *const wined3d_state) {
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while i < (*gl_info).limits.user_clip_distances {
        let mut plane: [GLdouble; 4] = [0.0; 4];

        (*gl_info)
            .gl_ops
            .gl
            .p_glMatrixMode
            .unwrap_unchecked()(GL_MODELVIEW as GLenum);
        (*gl_info)
            .gl_ops
            .gl
            .p_glPushMatrix
            .unwrap_unchecked()();
        (*gl_info)
            .gl_ops
            .gl
            .p_glLoadIdentity
            .unwrap_unchecked()();

        plane[0 as usize] = (*state).clip_planes[i as usize].x as GLdouble;
        plane[1 as usize] = (*state).clip_planes[i as usize].y as GLdouble;
        plane[2 as usize] = (*state).clip_planes[i as usize].z as GLdouble;
        plane[3 as usize] = (*state).clip_planes[i as usize].w as GLdouble;

        (*gl_info)
            .gl_ops
            .gl
            .p_glClipPlane
            .unwrap_unchecked()(
            (GL_CLIP_PLANE0 as GLenum).wrapping_add(i as GLenum),
            core::ptr::addr_of_mut!(plane) as *mut GLdouble,
        );
        checkGLcall!(gl_info, b"glClipPlane\0".as_ptr() as *const ::core::ffi::c_char);

        (*gl_info)
            .gl_ops
            .gl
            .p_glPopMatrix
            .unwrap_unchecked()();

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn streamsrc(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    wined3d_context_gl_update_stream_sources(wined3d_context_gl(context), state);
}

unsafe extern "C" fn vdecl_miscpart(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    if isStateDirty(context, (STATE_STREAMSRC as u32)) != false {
        return;
    }
    wined3d_context_gl_update_stream_sources(wined3d_context_gl(context), state);
}

unsafe extern "C" fn viewport_miscpart(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let mut min_z: ::core::ffi::c_float = 0.0;
    let mut max_z: ::core::ffi::c_float = 0.0;

    if (*gl_info).supported[ARB_VIEWPORT_ARRAY as usize] != 0 {
        let mut depth_ranges: [GLdouble; (2 * WINED3D_MAX_VIEWPORTS) as _] =
            [0.0; (2 * WINED3D_MAX_VIEWPORTS) as _];
        let mut viewports: [GLfloat; (4 * WINED3D_MAX_VIEWPORTS) as _] =
            [0.0; (4 * WINED3D_MAX_VIEWPORTS) as _];

        let mut i: ::core::ffi::c_uint = 0;
        let mut reset_count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

        i = 0 as ::core::ffi::c_uint;
        while i < (*state).viewport_count {
            wined3d_viewport_get_z_range(
                (&raw const (*state).viewports as *const wined3d_viewport).offset(i as isize)
                    as *const wined3d_viewport,
                &raw mut min_z,
                &raw mut max_z,
            );
            depth_ranges[i as usize * 2] = min_z as GLdouble;
            depth_ranges[i as usize * 2 + 1] =
                max_z as GLdouble;

            viewports[i as usize * 4] =
                (*state).viewports[i as usize].x as GLfloat;
            viewports[i as usize * 4 + 1] =
                (*state).viewports[i as usize].y as GLfloat;
            viewports[i as usize * 4 + 2] =
                (*state).viewports[i as usize].width as GLfloat;
            viewports[i as usize * 4 + 3] =
                (*state).viewports[i as usize].height as GLfloat;

            /* Don't pass fractionals to GL if we earlier decided not to use
             * this functionality for two reasons: First, GL might offer us
             * fewer than 8 bits, and still make use of the fractional, in
             * addition to the emulation we apply in shader_get_position_fixup.
             * Second, even if GL tells us it has no subpixel precision (Mac OS!)
             * it might still do something with the fractional amount, e.g.
             * round it upwards. I can't find any info on rounding in
             * GL_ARB_viewport_array. */
            if (*(*context).d3d_info).subpixel_viewport() == 0 {
                viewports[i as usize * 4] = floor(
                    viewports[i as usize * 4] as ::core::ffi::c_double,
                ) as GLfloat;
                viewports[i as usize * 4 + 1] =
                    floor(
                        viewports[i as usize * 4 + 1]
                            as ::core::ffi::c_double,
                    ) as GLfloat;
                viewports[i as usize * 4 + 2] =
                    floor(
                        viewports[i as usize * 4 + 2]
                            as ::core::ffi::c_double,
                    ) as GLfloat;
                viewports[i as usize * 4 + 3] =
                    floor(
                        viewports[i as usize * 4 + 3]
                            as ::core::ffi::c_double,
                    ) as GLfloat;
            }
            i = i.wrapping_add(1);
        }

        if (*context).viewport_count > (*state).viewport_count {
            reset_count = (*context).viewport_count - (*state).viewport_count;
        }

        if reset_count != 0 {
            memset(
                (&raw mut depth_ranges as *mut GLdouble)
                    .add((*state).viewport_count as usize * 2)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                reset_count as usize
                    * 2
                    * ::core::mem::size_of::<GLdouble>(),
            );
            memset(
                (&raw mut viewports as *mut GLfloat)
                    .add((*state).viewport_count as usize * 4)
                    as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                reset_count as usize
                    * 4
                    * ::core::mem::size_of::<GLfloat>(),
            );
        }

        ((*gl_info).gl_ops.ext.p_glDepthRangeArrayv.unwrap_unchecked())(
            0 as GLuint,
            (*state).viewport_count.wrapping_add(reset_count) as GLsizei,
            &raw mut depth_ranges as *mut GLdouble,
        );
        ((*gl_info).gl_ops.ext.p_glViewportArrayv.unwrap_unchecked())(
            0 as GLuint,
            (*state).viewport_count.wrapping_add(reset_count) as GLsizei,
            &raw mut viewports as *mut GLfloat,
        );
        (*context).viewport_count = (*state).viewport_count;
    } else {
        wined3d_viewport_get_z_range(
            (&raw const (*state).viewports as *const wined3d_viewport)
                .offset(0 as isize) as *const wined3d_viewport,
            &raw mut min_z,
            &raw mut max_z,
        );
        (*gl_info)
            .gl_ops
            .gl
            .p_glDepthRange
            .unwrap_unchecked()(min_z as GLdouble, max_z as GLdouble);
        (*gl_info)
            .gl_ops
            .gl
            .p_glViewport
            .unwrap_unchecked()(
            (*state).viewports[0 as usize].x as GLint,
            (*state).viewports[0 as usize].y as GLint,
            (*state).viewports[0 as usize].width as GLsizei,
            (*state).viewports[0 as usize].height as GLsizei,
        );
    }
    checkGLcall!(gl_info, b"setting clip space and viewport\0".as_ptr() as *const ::core::ffi::c_char);
}

unsafe extern "C" fn viewport_miscpart_cc(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    /* See get_projection_matrix() in glsl_shader.c for a discussion about those values. */
    let mut pixel_center_offset: f32 = if (*(*context).d3d_info).wined3d_creation_flags
        & WINED3D_PIXEL_CENTER_INTEGER != 0
    {
        0.5f32
    } else {
        0.0f32
    };
    let mut depth_ranges: [GLdouble; (2 * WINED3D_MAX_VIEWPORTS) as _] =
        [0.0; (2 * WINED3D_MAX_VIEWPORTS) as _];
    let mut viewports: [GLfloat; (4 * WINED3D_MAX_VIEWPORTS) as _] =
        [0.0; (4 * WINED3D_MAX_VIEWPORTS) as _];
    let mut i: u32 = 0;
    let mut reset_count: u32 = 0;
    let mut min_z: f32 = 0.0;
    let mut max_z: f32 = 0.0;

    pixel_center_offset += (*(*context).d3d_info).filling_convention_offset / 2.0f32;

    ((*gl_info).gl_ops.ext.p_glClipControl.unwrap_unchecked())(GL_UPPER_LEFT, GL_ZERO_TO_ONE);

    while i < (*state).viewport_count {
        wined3d_viewport_get_z_range(
            core::ptr::addr_of!((*state).viewports[i as usize]),
            &raw mut min_z,
            &raw mut max_z,
        );
        depth_ranges[i.wrapping_mul(2) as usize] = min_z as GLdouble;
        depth_ranges[i
            .wrapping_mul(2)
            .wrapping_add(1) as usize] = max_z as GLdouble;

        viewports[i.wrapping_mul(4) as usize] =
            ((*state).viewports[i as usize].x + pixel_center_offset) as GLfloat;
        viewports[i
            .wrapping_mul(4)
            .wrapping_add(1) as usize] =
            ((*state).viewports[i as usize].y + pixel_center_offset) as GLfloat;
        viewports[i
            .wrapping_mul(4)
            .wrapping_add(2) as usize] =
            (*state).viewports[i as usize].width as GLfloat;
        viewports[i
            .wrapping_mul(4)
            .wrapping_add(3) as usize] =
            (*state).viewports[i as usize].height as GLfloat;
        i = i.wrapping_add(1);
    }

    if (*context).viewport_count > (*state).viewport_count {
        reset_count = (*context).viewport_count.wrapping_sub((*state).viewport_count);
    }

    if reset_count != 0 {
        memset(
            (&raw mut depth_ranges as *mut GLdouble).offset(
                (*state).viewport_count.wrapping_mul(2) as isize,
            ) as *mut GLdouble as *mut core::ffi::c_void,
            0,
            (reset_count.wrapping_mul(2) as usize)
                .wrapping_mul(core::mem::size_of::<GLdouble>()),
        );
        memset(
            (&raw mut viewports as *mut GLfloat).offset(
                (*state).viewport_count.wrapping_mul(4) as isize,
            ) as *mut GLfloat as *mut core::ffi::c_void,
            0,
            (reset_count.wrapping_mul(4) as usize)
                .wrapping_mul(core::mem::size_of::<GLfloat>()),
        );
    }

    ((*gl_info).gl_ops.ext.p_glDepthRangeArrayv.unwrap_unchecked())(
        0,
        (*state).viewport_count.wrapping_add(reset_count) as GLsizei,
        &raw mut depth_ranges as *mut GLdouble,
    );
    ((*gl_info).gl_ops.ext.p_glViewportArrayv.unwrap_unchecked())(
        0,
        (*state).viewport_count.wrapping_add(reset_count) as GLsizei,
        &raw mut viewports as *mut GLfloat,
    );
    (*context).viewport_count = (*state).viewport_count;

    checkGLcall!(gl_info, "setting clip space and viewport");
}

unsafe extern "C" fn scissorrect(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let mut r: *const RECT = core::ptr::null();

    /* Warning: glScissor uses window coordinates, not viewport coordinates,
     * so our viewport correction does not apply. Warning2: Even in windowed
     * mode the coords are relative to the window, not the screen. */

    if (*gl_info).supported[ARB_VIEWPORT_ARRAY as usize] != 0 {
        let mut sr: [GLint; (4 * WINED3D_MAX_VIEWPORTS) as _] = [0; (4 * WINED3D_MAX_VIEWPORTS) as _];
        let mut i: u32 = 0;
        let mut reset_count: u32 = 0;

        i = 0;
        while i < (*state).scissor_rect_count {
            r = (core::ptr::addr_of!((*state).scissor_rects) as *const RECT).offset(i as isize) as *const RECT;

            sr[(i * 4) as usize] = (*r).left as GLint;
            sr[(i * 4 + 1) as usize] = (*r).top as GLint;
            sr[(i * 4 + 2) as usize] = ((*r).right - (*r).left) as GLint;
            sr[(i * 4 + 3) as usize] = ((*r).bottom - (*r).top) as GLint;
            i = i.wrapping_add(1);
        }

        if (*context).scissor_rect_count > (*state).scissor_rect_count {
            reset_count = (*context).scissor_rect_count.wrapping_sub((*state).scissor_rect_count);
        }

        if reset_count != 0 {
            memset(
                (core::ptr::addr_of_mut!(sr) as *mut GLint).offset(((*state).scissor_rect_count * 4) as isize) as *mut core::ffi::c_void,
                0,
                (reset_count as usize * 4) * core::mem::size_of::<GLint>(),
            );
        }

        ((*gl_info).gl_ops.ext.p_glScissorArrayv.unwrap_unchecked())(
            0 as GLuint,
            ((*state).scissor_rect_count + reset_count) as GLsizei,
            core::ptr::addr_of_mut!(sr) as *mut GLint,
        );
        checkGLcall!(gl_info, "glScissorArrayv");
        (*context).scissor_rect_count = (*state).scissor_rect_count;
    } else {
        r = (core::ptr::addr_of!((*state).scissor_rects) as *const RECT).offset(0 as isize) as *const RECT;
        ((*gl_info).gl_ops.gl.p_glScissor.unwrap_unchecked())(
            (*r).left as GLint,
            (*r).top as GLint,
            ((*r).right - (*r).left) as GLsizei,
            ((*r).bottom - (*r).top) as GLsizei,
        );
        checkGLcall!(gl_info, "glScissor");
    }
}

unsafe extern "C" fn indexbuffer(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let stream_info: *const wined3d_stream_info = core::ptr::addr_of!((*context).stream_info);
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut();

    if (*state).index_buffer.is_null() || (*stream_info).all_vbo() == 0 {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ELEMENT_ARRAY_BUFFER, 0);
        return;
    }

    buffer = (*state).index_buffer;
    if !(*buffer).buffer_object.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ELEMENT_ARRAY_BUFFER, (*wined3d_bo_gl((*buffer).buffer_object)).id);
        wined3d_buffer_validate_user(buffer);
    } else {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_ELEMENT_ARRAY_BUFFER, 0);
    }
}

unsafe extern "C" fn depth_clip(r: *const wined3d_rasterizer_state, gl_info: *const wined3d_gl_info) {
    if (*gl_info).supported[ARB_DEPTH_CLAMP as usize] == 0 {
        if !r.is_null() && (*r).desc.depth_clip == 0 {
            FIXME!("Depth clamp not supported by this GL implementation.\n");
        }
        return;
    }

    if !r.is_null() && (*r).desc.depth_clip == 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_DEPTH_CLAMP as GLenum);
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_DEPTH_CLAMP as GLenum);
    }
    checkGLcall!(gl_info, b"depth clip\0" as *const u8 as *const ::core::ffi::c_char);
}

unsafe extern "C" fn rasterizer(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let r: *const wined3d_rasterizer_state = (*state).rasterizer_state;

    /* Rendering without ARB_clip_control requires flipping position manually.
     * This also means that all primitives will be backwards, so we need to
     * also swap which side is the front face. */
    (*gl_info).gl_ops.gl.p_glFrontFace.unwrap_unchecked()(
        (if !r.is_null() && (*r).desc.front_ccw != 0 {
            GL_CW
        } else {
            GL_CCW
        }) as GLenum,
    );
    checkGLcall!(gl_info, "glFrontFace");
    depthbias(context, state);
    fillmode(r, gl_info);
    cullmode(r, gl_info);
    depth_clip(r, gl_info);
    scissor(r, gl_info);
    line_antialias(r, gl_info);
}

unsafe extern "C" fn rasterizer_cc(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let r: *const wined3d_rasterizer_state = (*state).rasterizer_state;
    let mut mode: GLenum = 0;

    mode = if !r.is_null() && (*r).desc.front_ccw != 0 {
        GL_CCW
    } else {
        GL_CW
    };

    ((*gl_info).gl_ops.gl.p_glFrontFace.unwrap_unchecked())(mode);
    checkGLcall!(gl_info, "glFrontFace");
    depthbias(context, state);
    fillmode(r, gl_info);
    cullmode(r, gl_info);
    depth_clip(r, gl_info);
    scissor(r, gl_info);
    line_antialias(r, gl_info);
}
#[no_mangle]
pub unsafe extern "C" fn state_srgbwrite(mut context: *mut wined3d_context, mut state: *const wined3d_state, mut state_id: DWORD) {
    let mut gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    TRACE!("context %p, state %p, state_id %#lx.\n", context, state, state_id);
    if needs_srgb_write((*context).d3d_info, state, core::ptr::addr_of!((*state).fb)) != 0 {
        (*gl_info)
            .gl_ops
            .gl
            .p_glEnable
            .unwrap_unchecked()(GL_FRAMEBUFFER_SRGB as GLenum);
    } else {
        (*gl_info)
            .gl_ops
            .gl
            .p_glDisable
            .unwrap_unchecked()(GL_FRAMEBUFFER_SRGB as GLenum);
    }
}

unsafe extern "C" fn state_cb(
    mut context: *mut wined3d_context,
    mut state: *const wined3d_state,
    mut state_id: DWORD,
) {
    let gl_info: *const wined3d_gl_info = (*wined3d_context_gl(context)).gl_info;
    let mut shader_type: wined3d_shader_type;
    let mut buffer: *mut wined3d_buffer;
    let mut i: ::core::ffi::c_uint;
    let mut base: ::core::ffi::c_uint = 0;
    let mut count: ::core::ffi::c_uint = 0;
    let mut bo_gl: *mut wined3d_bo_gl;

    if STATE_IS_GRAPHICS_CONSTANT_BUFFER!(state_id) {
        shader_type = (state_id - STATE_GRAPHICS_CONSTANT_BUFFER!(0)) as wined3d_shader_type;
    } else {
        shader_type = WINED3D_SHADER_TYPE_COMPUTE;
    }

    /* If a shader has not been set, buffer objects are not yet initialised. */
    if (*state).shader[shader_type as usize].is_null() {
        return;
    }

    wined3d_gl_limits_get_uniform_block_range(
        &raw const (*gl_info).limits,
        shader_type,
        &raw mut base,
        &raw mut count,
    );
    i = 0;
    while i < count {
        let buffer_state: *const wined3d_constant_buffer_state =
            &raw const (*state).cb[shader_type as usize][i as usize];

        'body: {
            if (*buffer_state).buffer.is_null() {
                ((*gl_info).gl_ops.ext.p_glBindBufferBase.unwrap_unchecked())(
                    GL_UNIFORM_BUFFER,
                    base.wrapping_add(i),
                    0,
                );
                break 'body;
            }

            buffer = (*buffer_state).buffer;
            bo_gl = wined3d_bo_gl((*buffer).buffer_object);
            ((*gl_info).gl_ops.ext.p_glBindBufferRange.unwrap_unchecked())(
                GL_UNIFORM_BUFFER,
                base.wrapping_add(i),
                (*bo_gl).id,
                (*bo_gl).b.buffer_offset.wrapping_add((*buffer_state).offset as size_t) as GLintptr,
                min!(
                    ((*buffer_state).size) as GLsizeiptr,
                    (((*buffer).resource.size as ::core::ffi::c_uint)
                        .wrapping_sub((*buffer_state).offset)) as _,
                ),
            );

            wined3d_buffer_validate_user(buffer);
        }
        i = i.wrapping_add(1);
    }
    checkGLcall!(gl_info, b"bind constant buffers\0".as_ptr() as *const ::core::ffi::c_char);
}

unsafe extern "C" fn state_cb_warn(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    let _ = context;
    let _ = state;
    WARN!("Constant buffers (%s) no supported.\n", debug_d3dstate(state_id));
}

unsafe extern "C" fn state_shader_resource_binding(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    // TRACE("context %p, state %p, state_id %#lx.\n", context, state, state_id);
    (*context).set_update_shader_resource_bindings((1) as _);
}

unsafe extern "C" fn state_cs_resource_binding(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    // TRACE("context %p, state %p, state_id %#lx.\n", context, state, state_id);
    (*context).set_update_compute_shader_resource_bindings((1) as _);
}

unsafe extern "C" fn state_uav_binding(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    // TRACE("context %p, state %p, state_id %#lx.\n", context, state, state_id);
    (*context).set_update_unordered_access_view_bindings((1) as _);
}

unsafe extern "C" fn state_cs_uav_binding(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    // TRACE("context %p, state %p, state_id %#lx.\n", context, state, state_id);
    (*context).set_update_compute_unordered_access_view_bindings((1) as _);
}

unsafe extern "C" fn state_uav_warn(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    WARN!("ARB_shader_image_load_store is not supported by this OpenGL implementation.\n");
}

unsafe extern "C" fn state_so(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut buffer: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();
    let mut offset: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut bo_gl: *mut wined3d_bo_gl = core::ptr::null_mut::<wined3d_bo_gl>();

    wined3d_context_gl_end_transform_feedback(context_gl);

    i = 0;
    while (i as usize) < (ARRAY_SIZE!((*state).stream_output) as usize) {
        'body: {
            if (*state).stream_output[i as usize].buffer.is_null() {
                ((*gl_info).gl_ops.ext.p_glBindBufferBase.unwrap_unchecked())(GL_TRANSFORM_FEEDBACK_BUFFER, i, 0);
                break 'body;
            }

            buffer = (*state).stream_output[i as usize].buffer;
            offset = (*state).stream_output[i as usize].offset;
            bo_gl = wined3d_bo_gl((*buffer).buffer_object);
            if offset == !0 {
                offset = 0;
            }
            size = ((*buffer).resource.size as ::core::ffi::c_uint).wrapping_sub(offset);
            ((*gl_info).gl_ops.ext.p_glBindBufferRange.unwrap_unchecked())(
                GL_TRANSFORM_FEEDBACK_BUFFER,
                i,
                (*bo_gl).id,
                (*bo_gl).b.buffer_offset.wrapping_add(offset as usize) as _,
                (size) as _,
            );
            wined3d_buffer_validate_user(buffer);
        }
        i = i.wrapping_add(1);
    }
    checkGLcall!(gl_info, b"bind transform feedback buffers\0" as *const u8 as *const ::core::ffi::c_char);
}

unsafe extern "C" fn state_so_warn(
    context: *mut wined3d_context,
    state: *const wined3d_state,
    state_id: DWORD,
) {
    WARN!("Transform feedback not supported.\n");
}

unsafe extern "C" fn ffp_none_context_alloc(context: *mut wined3d_context) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn ffp_none_context_free(context: *mut wined3d_context) {}

unsafe extern "C" fn none_pipe_apply_draw_state(
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {}

unsafe extern "C" fn none_pipe_disable(context: *const wined3d_context) {}

unsafe extern "C" fn none_alloc(
    shader_backend: *const wined3d_shader_backend_ops,
    shader_priv: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    shader_priv
}

unsafe extern "C" fn none_free(device: *mut wined3d_device, context: *mut wined3d_context) {}

unsafe extern "C" fn vp_none_get_caps(adapter: *const wined3d_adapter, caps: *mut wined3d_vertex_caps) {
    memset(caps as *mut core::ffi::c_void, 0 as core::ffi::c_int, core::mem::size_of::<wined3d_vertex_caps>());
}

unsafe extern "C" fn vp_none_get_emul_mask(adapter: *const wined3d_adapter) -> u32 {
    0
}

unsafe extern "C" fn fp_none_get_caps(adapter: *const wined3d_adapter, caps: *mut fragment_caps) {
    memset(
        caps as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<fragment_caps>() as usize,
    );
}

unsafe extern "C" fn fp_none_get_emul_mask(adapter: *const wined3d_adapter) -> u32 {
    0
}

unsafe extern "C" fn fp_none_color_fixup_supported(fixup: color_fixup_desc) -> BOOL {
    is_identity_fixup(fixup)
}

unsafe extern "C" fn num_handlers(funcs: *const APPLYSTATEFUNC) -> ::core::ffi::c_uint {
    let mut i: ::core::ffi::c_uint;
    i = 0;
    while (*funcs.add(i as usize)).is_some() {
        i = i.wrapping_add(1);
    }
    return i;
}

unsafe extern "C" fn multistate_apply_2(
    mut context: *mut wined3d_context,
    mut state: *const wined3d_state,
    mut state_id: DWORD,
) {
    (*(*(*context).device).multistate_funcs[state_id as usize]
        .offset(0 as isize))
    .unwrap_unchecked()(context, state, state_id);
    (*(*(*context).device).multistate_funcs[state_id as usize]
        .offset(1 as isize))
    .unwrap_unchecked()(context, state, state_id);
}

unsafe extern "C" fn multistate_apply_3(context: *mut wined3d_context, state: *const wined3d_state, state_id: DWORD) {
    (*(*(*context).device).multistate_funcs[state_id as usize].offset((0) as isize))
        .unwrap_unchecked()(context, state, state_id);
    (*(*(*context).device).multistate_funcs[state_id as usize].offset((1) as isize))
        .unwrap_unchecked()(context, state, state_id);
    (*(*(*context).device).multistate_funcs[state_id as usize].offset((2) as isize))
        .unwrap_unchecked()(context, state, state_id);
}

unsafe extern "C" fn prune_invalid_states(
    mut state_table: *mut wined3d_state_entry,
    mut d3d_info: *const wined3d_d3d_info,
) {
    let mut start: ::core::ffi::c_uint = 0;
    let mut last: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    start = STATE_TEXTURESTAGE!((*d3d_info).ffp_fragment_caps.max_blend_stages, 0);
    last = STATE_TEXTURESTAGE!(WINED3D_MAX_FFP_TEXTURES - 1, WINED3D_HIGHEST_TEXTURE_STATE);
    i = start;
    while i <= last {
        (*state_table.offset(i as isize)).representative = 0;
        let ref mut fresh25 = (*state_table.offset(i as isize)).apply;
        *fresh25 = Some(
            state_undefined
                as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state, DWORD) -> (),
        ) as APPLYSTATEFUNC;
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn validate_state_table(state_table: *mut wined3d_state_entry) {
    static rs_holes: [(u32, u32); 17] = [
        (1, 8),
        (11, 14),
        (16, 24),
        (27, 27),
        (30, 33),
        (39, 40),
        (42, 47),
        (49, 135),
        (138, 139),
        (144, 144),
        (149, 150),
        (153, 153),
        (157, 160),
        (162, 165),
        (167, 193),
        (195, 209),
        (0, 0),
    ];
    static simple_states: [core::ffi::c_uint; 29] = [
        (STATE_VDECL as u32),
        (STATE_STREAMSRC as u32),
        (STATE_INDEXBUFFER as u32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_VERTEX) as i32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_HULL) as i32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_DOMAIN) as i32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_PIXEL) as i32),
        STATE_SHADER!((WINED3D_SHADER_TYPE_COMPUTE) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_VERTEX) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_HULL) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_DOMAIN) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_GEOMETRY) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_PIXEL) as i32),
        STATE_CONSTANT_BUFFER!((WINED3D_SHADER_TYPE_COMPUTE) as i32),
        (STATE_COMPUTE_SHADER_RESOURCE_BINDING as u32),
        (STATE_GRAPHICS_SHADER_RESOURCE_BINDING as u32),
        (STATE_COMPUTE_UNORDERED_ACCESS_VIEW_BINDING as u32),
        (STATE_GRAPHICS_UNORDERED_ACCESS_VIEW_BINDING as u32),
        (STATE_VIEWPORT as u32),
        (STATE_SCISSORRECT as u32),
        (STATE_RASTERIZER as u32),
        (STATE_BASEVERTEXINDEX as u32),
        (STATE_FRAMEBUFFER as u32),
        (STATE_BLEND as u32),
        (STATE_BLEND_FACTOR as u32),
        (STATE_DEPTH_STENCIL as u32),
        (STATE_STENCIL_REF as u32),
        (STATE_DEPTH_BOUNDS as u32),
    ];
    let mut i: core::ffi::c_uint;
    let mut current: core::ffi::c_uint;

    i = STATE_RENDER!(1) as core::ffi::c_uint;
    current = 0 as core::ffi::c_uint;
    while i <= STATE_RENDER!(WINEHIGHEST_RENDER_STATE) as core::ffi::c_uint {
        if rs_holes[current as usize].0 == 0
            || i < STATE_RENDER!(rs_holes[current as usize].0) as core::ffi::c_uint
        {
            if (*state_table.offset(i as isize)).representative == 0 {
                ERR!(
                    "State %s (%#x) should have a representative.\n",
                    debug_d3dstate(i),
                    i
                );
            }
        } else if (*state_table.offset(i as isize)).representative != 0 {
            ERR!(
                "State %s (%#x) shouldn't have a representative.\n",
                debug_d3dstate(i),
                i
            );
        }

        if i == STATE_RENDER!(rs_holes[current as usize].1) as core::ffi::c_uint {
            current = current.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    i = 0 as core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(simple_states) as usize {
        if (*state_table.offset(simple_states[i as usize] as isize)).representative == 0 {
            ERR!(
                "State %s (%#x) should have a representative.\n",
                debug_d3dstate(simple_states[i as usize]),
                simple_states[i as usize]
            );
        }
        i = i.wrapping_add(1);
    }

    i = 0 as core::ffi::c_uint;
    while i < (STATE_HIGHEST + 1) as core::ffi::c_uint {
        let mut rep: core::ffi::c_uint = (*state_table.offset(i as isize)).representative;
        if rep != 0 {
            if (*state_table.offset(rep as isize)).representative != rep {
                ERR!(
                    "State %s (%#x) has invalid representative %s (%#x).\n",
                    debug_d3dstate(i),
                    i,
                    debug_d3dstate(rep),
                    rep
                );
                (*state_table.offset(i as isize)).representative =
                    0 as core::ffi::c_uint;
            }

            if rep != i {
                if (*state_table.offset(i as isize)).apply.is_some() {
                    ERR!(
                        "State %s (%#x) has both a handler and representative.\n",
                        debug_d3dstate(i),
                        i
                    );
                }
            } else if (*state_table.offset(i as isize)).apply.is_none() {
                ERR!(
                    "Self representing state %s (%#x) has no handler.\n",
                    debug_d3dstate(i),
                    i
                );
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn compile_state_table(
    state_table: *mut wined3d_state_entry,
    dev_multistate_funcs: *mut *mut APPLYSTATEFUNC,
    d3d_info: *const wined3d_d3d_info,
    supported_extensions: *const BOOL,
    vertex: *const wined3d_vertex_pipe_ops,
    fragment: *const wined3d_fragment_pipe_ops,
    misc: *const wined3d_state_entry_template,
) -> HRESULT {
    let mut multistate_funcs: [[APPLYSTATEFUNC; 3]; (STATE_HIGHEST + 1) as usize] =
        [[None; 3]; (STATE_HIGHEST + 1) as usize];
    let mut cur: *const wined3d_state_entry_template = core::ptr::null();
    let mut i: u32 = 0;
    let mut type_0: u32 = 0;
    let mut handlers: u32 = 0;
    let mut set: [BOOL; (STATE_HIGHEST + 1) as usize] =
        [0; (STATE_HIGHEST + 1) as usize];

    memset(
        core::ptr::addr_of_mut!(multistate_funcs) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<[[APPLYSTATEFUNC; 3]; (STATE_HIGHEST + 1) as usize]>(),
    );

    i = 0;
    while i < (STATE_HIGHEST + 1) as u32 {
        (*state_table.offset(i as isize)).representative = 0;
        (*state_table.offset(i as isize)).apply = Some(
            state_undefined as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state, DWORD) -> (),
        ) as APPLYSTATEFUNC;
        i = i.wrapping_add(1);
    }

    type_0 = 0;
    let oom: bool = 'body: {
        while type_0 < 3 {
            'typebody: {
                /* This switch decides the order in which the states are applied */
                match type_0 {
                    0 => {
                        cur = misc;
                    }
                    1 => {
                        cur = (*fragment).states;
                    }
                    2 => {
                        cur = (*vertex).vp_states;
                    }
                    _ => {
                        cur = core::ptr::null();
                    }
                }
                if cur.is_null() {
                    break 'typebody;
                }

                /* GL extension filtering should not prevent multiple handlers being applied from different
                 * pipeline parts
                 */
                memset(
                    &raw mut set as *mut BOOL as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<[BOOL; (STATE_HIGHEST + 1) as usize]>(),
                );

                i = 0;
                while (*cur.offset(i as isize)).state != 0 {
                    'next: {
                        let mut funcs_array: *mut APPLYSTATEFUNC = core::ptr::null_mut();

                        /* Only use the first matching state with the available extension from one template. */
                        if set[(*cur.offset(i as isize)).state as usize] != 0 {
                            break 'next;
                        }
                        /* Skip state lines depending on unsupported extensions */
                        if *supported_extensions.offset((*cur.offset(i as isize)).extension as isize) == 0 {
                            break 'next;
                        }
                        set[(*cur.offset(i as isize)).state as usize] = (TRUE as i32);
                        /* If the apply function is used, mark the state set(done above) to prevent
                         * applying later lines, but do not record anything in the state table
                         */
                        if (*cur.offset(i as isize)).content.representative == 0 {
                            break 'next;
                        }

                        handlers = num_handlers(
                            (core::ptr::addr_of_mut!(multistate_funcs) as *mut [APPLYSTATEFUNC; 3])
                                .offset((*cur.offset(i as isize)).state as isize)
                                as *mut APPLYSTATEFUNC,
                        );
                        multistate_funcs[(*cur.offset(i as isize)).state as usize][handlers as usize] =
                            (*cur.offset(i as isize)).content.apply;
                        match handlers {
                            0 => {
                                (*state_table.offset((*cur.offset(i as isize)).state as isize)).apply =
                                    (*cur.offset(i as isize)).content.apply;
                            }
                            1 => {
                                (*state_table.offset((*cur.offset(i as isize)).state as isize)).apply = Some(
                                    multistate_apply_2
                                        as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state, DWORD) -> (),
                                ) as APPLYSTATEFUNC;
                                *dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize) =
                                    calloc(
                                        2,
                                        (core::mem::size_of::<APPLYSTATEFUNC>()) as _,
                                    ) as *mut APPLYSTATEFUNC;
                                if (*dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize)).is_null() {
                                    break 'body true;
                                }
                                *(*dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize))
                                    .offset(0 as isize) =
                                    multistate_funcs[(*cur.offset(i as isize)).state as usize]
                                        [0 as usize];
                                *(*dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize))
                                    .offset(1 as isize) =
                                    multistate_funcs[(*cur.offset(i as isize)).state as usize]
                                        [1 as usize];
                            }
                            2 => {
                                (*state_table.offset((*cur.offset(i as isize)).state as isize)).apply = Some(
                                    multistate_apply_3
                                        as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state, DWORD) -> (),
                                ) as APPLYSTATEFUNC;
                                funcs_array = realloc(
                                    *dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize)
                                        as *mut core::ffi::c_void,
                                    core::mem::size_of::<APPLYSTATEFUNC>() * 3 as usize,
                                ) as *mut APPLYSTATEFUNC;
                                if funcs_array.is_null() {
                                    break 'body true;
                                }
                                *dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize) = funcs_array;
                                *(*dev_multistate_funcs.offset((*cur.offset(i as isize)).state as isize))
                                    .offset(2 as isize) =
                                    multistate_funcs[(*cur.offset(i as isize)).state as usize]
                                        [2 as usize];
                            }
                            _ => {
                                ERR!(
                                    "Unexpected amount of state handlers for state %u: %u.\n",
                                    (*cur.offset(i as isize)).state,
                                    handlers + 1
                                );
                            }
                        }

                        if (*state_table.offset((*cur.offset(i as isize)).state as isize)).representative != 0
                            && (*state_table.offset((*cur.offset(i as isize)).state as isize)).representative
                                != (*cur.offset(i as isize)).content.representative
                        {
                            FIXME!(
                                "State %s (%#x) has different representatives in different pipeline parts.\n",
                                debug_d3dstate((*cur.offset(i as isize)).state as uint32_t),
                                (*cur.offset(i as isize)).state
                            );
                        }
                        (*state_table.offset((*cur.offset(i as isize)).state as isize)).representative =
                            (*cur.offset(i as isize)).content.representative;
                    }
                    i = i.wrapping_add(1);
                }
            }
            type_0 = type_0.wrapping_add(1);
        }

        prune_invalid_states(state_table, d3d_info);
        validate_state_table(state_table);

        false
    };

    if !oom {
        return WINED3D_OK;
    }

    i = 0;
    while i <= (STATE_HIGHEST as u32) {
        free(*dev_multistate_funcs.offset(i as isize) as *mut core::ffi::c_void);
        i = i.wrapping_add(1);
    }

    memset(
        dev_multistate_funcs as *mut core::ffi::c_void,
        0,
        (STATE_HIGHEST + 1) as usize
            * core::mem::size_of::<*mut APPLYSTATEFUNC>(),
    );

    return E_OUTOFMEMORY;
}
