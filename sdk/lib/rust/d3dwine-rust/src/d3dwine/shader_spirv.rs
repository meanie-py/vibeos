use crate::d3dwine::ffp_gl::state_nop;
use crate::d3dwine::context_vk::vk_shader_stage_from_wined3d;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_pipeline;
use crate::d3dwine::context_vk::wined3d_context_vk_get_pipeline_layout;
use crate::prelude::*;
use core::ffi::c_void;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_51 {
    pub varying_map: [vkd3d_shader_varying_map; 12],
    pub varying_count: ::core::ffi::c_uint,
}

extern "C" {
    fn vkd3d_shader_get_version(major: *mut core::ffi::c_uint, minor: *mut core::ffi::c_uint) -> *const core::ffi::c_char;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut spirv_compile_options: [vkd3d_shader_compile_option; 2] = [
    vkd3d_shader_compile_option {
        name: VKD3D_SHADER_COMPILE_OPTION_API_VERSION,
        value: VKD3D_SHADER_API_VERSION_1_7 as ::core::ffi::c_int as ::core::ffi::c_uint,
    },
    vkd3d_shader_compile_option {
        name: VKD3D_SHADER_COMPILE_OPTION_WRITE_TESS_GEOM_POINT_SIZE,
        value: 0 as ::core::ffi::c_uint,
    },
];

static mut spirv_shader_backend_vk: wined3d_shader_backend_ops = unsafe {
    wined3d_shader_backend_ops {
        shader_handle_instruction: Some(
            shader_spirv_handle_instruction
                as unsafe extern "C" fn(*const wined3d_shader_instruction) -> (),
        ),
        shader_precompile: Some(
            shader_spirv_precompile
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut wined3d_shader) -> (),
        ),
        shader_apply_draw_state: Some(
            shader_spirv_apply_draw_state
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut wined3d_context,
                    *const wined3d_state,
                ) -> (),
        ),
        shader_apply_compute_state: Some(
            shader_spirv_apply_compute_state
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut wined3d_context,
                    *const wined3d_state,
                ) -> (),
        ),
        shader_disable: Some(
            shader_spirv_disable
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut wined3d_context) -> (),
        ),
        shader_update_float_vertex_constants: Some(
            shader_spirv_update_float_vertex_constants
                as unsafe extern "C" fn(*mut wined3d_device, UINT, UINT) -> (),
        ),
        shader_update_float_pixel_constants: Some(
            shader_spirv_update_float_pixel_constants
                as unsafe extern "C" fn(*mut wined3d_device, UINT, UINT) -> (),
        ),
        shader_destroy: Some(
            shader_spirv_destroy as unsafe extern "C" fn(*mut wined3d_shader) -> (),
        ),
        shader_alloc_private: Some(
            shader_spirv_alloc
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_vertex_pipe_ops,
                    *const wined3d_fragment_pipe_ops,
                ) -> HRESULT,
        ),
        shader_free_private: Some(
            shader_spirv_free
                as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        shader_allocate_context_data: Some(
            shader_spirv_allocate_context_data
                as unsafe extern "C" fn(*mut wined3d_context) -> BOOL,
        ),
        shader_free_context_data: Some(
            shader_spirv_free_context_data as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        shader_init_context_state: Some(
            shader_spirv_init_context_state as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        shader_get_caps: Some(
            shader_spirv_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut shader_caps) -> (),
        ),
        shader_color_fixup_supported: Some(
            shader_spirv_color_fixup_supported as unsafe extern "C" fn(color_fixup_desc) -> BOOL,
        ),
        shader_compile: Some(
            shader_spirv_compile
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_shader_desc,
                    wined3d_shader_type,
                ) -> uint64_t,
        ),
    }
};

static mut spirv_vertex_pipe_vk_vp_states: [wined3d_state_entry_template; 7] = [
        wined3d_state_entry_template {
            state: WINED3D_RS_CLIPPING as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_CLIPPING as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_CLIPPLANEENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_CLIPPLANEENABLE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_POINTSIZE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_POINTSIZE as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_POINTSIZE_MIN as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_POINTSIZE_MIN as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_POINTSIZE_MAX as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_POINTSIZE_MAX as ::core::ffi::c_int
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
            extension: 0,
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
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: 0 as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: 0,
                apply: None,
            },
            extension: 0,
        },
    ];

static mut spirv_vertex_pipe_vk: wined3d_vertex_pipe_ops = unsafe {
    wined3d_vertex_pipe_ops {
        vp_apply_draw_state: Some(
            spirv_vertex_pipe_vk_vp_apply_draw_state
                as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state) -> (),
        ),
        vp_disable: Some(
            spirv_vertex_pipe_vk_vp_disable as unsafe extern "C" fn(*const wined3d_context) -> (),
        ),
        vp_get_caps: Some(
            spirv_vertex_pipe_vk_vp_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut wined3d_vertex_caps) -> (),
        ),
        vp_get_emul_mask: Some(
            spirv_vertex_pipe_vk_vp_get_emul_mask
                as unsafe extern "C" fn(*const wined3d_adapter) -> ::core::ffi::c_uint,
        ),
        vp_alloc: Some(
            spirv_vertex_pipe_vk_vp_alloc
                as unsafe extern "C" fn(
                    *const wined3d_shader_backend_ops,
                    *mut ::core::ffi::c_void,
                ) -> *mut ::core::ffi::c_void,
        ),
        vp_free: Some(
            spirv_vertex_pipe_vk_vp_free
                as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        vp_states: &raw const spirv_vertex_pipe_vk_vp_states as *const wined3d_state_entry_template,
    }
};

static mut spirv_fragment_pipe_vk_fp_states: [wined3d_state_entry_template; 13] = unsafe {
    [
        wined3d_state_entry_template {
            state: WINED3D_RS_SHADEMODE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_SHADEMODE as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_ALPHATESTENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_ALPHATESTENABLE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_ALPHAFUNC as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_ALPHAFUNC as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGCOLOR as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGCOLOR as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGTABLEMODE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGTABLEMODE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGSTART as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGSTART as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGEND as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGEND as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGDENSITY as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGDENSITY as ::core::ffi::c_int as ::core::ffi::c_uint,
                apply: Some(
                    state_nop
                        as unsafe extern "C" fn(
                            *mut wined3d_context,
                            *const wined3d_state,
                            DWORD,
                        ) -> (),
                ),
            },
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_FOGVERTEXMODE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_FOGVERTEXMODE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_POINTSPRITEENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_POINTSPRITEENABLE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: WINED3D_RS_SRGBWRITEENABLE as ::core::ffi::c_int as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: WINED3D_RS_SRGBWRITEENABLE as ::core::ffi::c_int
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
            extension: 0,
        },
        wined3d_state_entry_template {
            state: 0 as ::core::ffi::c_uint,
            content: wined3d_state_entry {
                representative: 0,
                apply: None,
            },
            extension: 0,
        },
    ]
};

static mut spirv_fragment_pipe_vk: wined3d_fragment_pipe_ops = unsafe {
    wined3d_fragment_pipe_ops {
        fp_apply_draw_state: Some(
            spirv_fragment_pipe_vk_fp_apply_draw_state
                as unsafe extern "C" fn(*mut wined3d_context, *const wined3d_state) -> (),
        ),
        fp_disable: Some(
            spirv_fragment_pipe_vk_fp_disable as unsafe extern "C" fn(*const wined3d_context) -> (),
        ),
        get_caps: Some(
            spirv_fragment_pipe_vk_fp_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut fragment_caps) -> (),
        ),
        get_emul_mask: Some(
            spirv_fragment_pipe_vk_fp_get_emul_mask
                as unsafe extern "C" fn(*const wined3d_adapter) -> ::core::ffi::c_uint,
        ),
        alloc_private: Some(
            spirv_fragment_pipe_vk_fp_alloc
                as unsafe extern "C" fn(
                    *const wined3d_shader_backend_ops,
                    *mut ::core::ffi::c_void,
                ) -> *mut ::core::ffi::c_void,
        ),
        free_private: Some(
            spirv_fragment_pipe_vk_fp_free
                as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        allocate_context_data: Some(
            spirv_fragment_pipe_vk_fp_alloc_context_data
                as unsafe extern "C" fn(*mut wined3d_context) -> BOOL,
        ),
        free_context_data: Some(
            spirv_fragment_pipe_vk_fp_free_context_data
                as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        color_fixup_supported: Some(
            shader_spirv_color_fixup_supported as unsafe extern "C" fn(color_fixup_desc) -> BOOL,
        ),
        states: &raw const spirv_fragment_pipe_vk_fp_states as *const wined3d_state_entry_template,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d_shader);  -- debug channel declaration; provided by the crate's debug macros

#[repr(C)]
pub struct shader_spirv_resource_bindings {
    pub bindings: *mut vkd3d_shader_resource_binding,
    pub bindings_size: usize,
    pub binding_count: usize,

    pub uav_counters: [vkd3d_shader_uav_counter_binding; (MAX_UNORDERED_ACCESS_VIEWS as _)],
    pub uav_counter_count: usize,

    pub vk_bindings: *mut VkDescriptorSetLayoutBinding,
    pub vk_bindings_size: usize,
    pub vk_binding_count: usize,

    pub binding_base: [usize; (WINED3D_SHADER_TYPE_COUNT) as _],
    pub so_stage: wined3d_shader_type,

    pub ffp_extra_binding: [u32; 2],
}

#[repr(C)]
pub struct shader_spirv_priv {
    pub vertex_pipe: *const wined3d_vertex_pipe_ops,
    pub fragment_pipe: *const wined3d_fragment_pipe_ops,

    pub bindings: shader_spirv_resource_bindings,
}

pub const MAX_SM1_INTER_STAGE_VARYINGS: usize = 12;

#[repr(C)]
pub struct shader_spirv_compile_arguments_vs {
    pub varying_map: [vkd3d_shader_varying_map; MAX_SM1_INTER_STAGE_VARYINGS],
    pub varying_count: u32,
}

#[repr(C)]
pub struct shader_spirv_compile_arguments_fs {
    pub args: ps_compile_args,
    pub sample_count: u32,
}

#[repr(C)]
pub union shader_spirv_compile_arguments_u {
    pub vs: core::mem::ManuallyDrop<shader_spirv_compile_arguments_vs>,
    pub fs: core::mem::ManuallyDrop<shader_spirv_compile_arguments_fs>,
}

#[repr(C)]
pub struct shader_spirv_compile_arguments {
    pub u: shader_spirv_compile_arguments_u,
}

#[repr(C)]
pub struct shader_spirv_graphics_program_variant_vk {
    pub compile_args: shader_spirv_compile_arguments,
    pub so_desc: *const wined3d_stream_output_desc,
    pub binding_base: usize,

    pub vk_module: VkShaderModule,
}

#[repr(C)]
pub struct shader_spirv_graphics_program_vk {
    pub variants: *mut shader_spirv_graphics_program_variant_vk,
    pub variants_size: usize,
    pub variant_count: usize,

    pub descriptor_info: vkd3d_shader_scan_descriptor_info,
    pub signature_info: vkd3d_shader_scan_signature_info,
}

#[repr(C)]
pub struct shader_spirv_compute_program_vk {
    pub vk_module: VkShaderModule,
    pub vk_pipeline: VkPipeline,
    pub vk_pipeline_layout: VkPipelineLayout,
    pub vk_set_layout: VkDescriptorSetLayout,

    pub descriptor_info: vkd3d_shader_scan_descriptor_info,
}

#[repr(C)]
pub struct wined3d_shader_spirv_compile_args {
    pub varying_map: vkd3d_shader_varying_map_info,
    pub spirv_target: vkd3d_shader_spirv_target_info,
    pub parameter_info: vkd3d_shader_parameter_info,
    pub extensions: [vkd3d_shader_spirv_extension; 1],
    pub parameters: [vkd3d_shader_parameter1; 4],
    pub ps_alpha_swizzle: [u32; (WINED3D_MAX_RENDER_TARGETS as _)],
}

#[repr(C)]
pub struct wined3d_shader_spirv_shader_interface {
    pub vkd3d_interface: vkd3d_shader_interface_info,
    pub xfb_info: vkd3d_shader_transform_feedback_info,
}

// Designated initializer; remaining (unlisted) ops fields are zero-filled as in C.

// Designated initializer; remaining (unlisted) vp ops fields are zero-filled as in C.

// Designated initializer; remaining (unlisted) fp ops fields are zero-filled as in C.

unsafe extern "C" fn shader_spirv_handle_instruction(ins: *const wined3d_shader_instruction) {}

unsafe extern "C" fn shader_spirv_compile_arguments_init(
    mut args: *mut shader_spirv_compile_arguments,
    mut context: *const wined3d_context,
    mut shader: *const wined3d_shader,
    mut state: *const wined3d_state,
    mut sample_count: ::core::ffi::c_uint,
) {
    memset(
        args as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<shader_spirv_compile_arguments>() as size_t,
    );

    match (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint {
        WINED3D_SHADER_TYPE_PIXEL => {
            /* The context here doesn't have a valid stream info, but that's
             * fine, because we have full_ffp_varyings. */
            find_ps_compile_args(
                state,
                shader,
                (!(*state).vertex_declaration.is_null()
                    && (*(*state).vertex_declaration).position_transformed as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int,
                &raw mut (*(*args).u.fs).args,
                context,
            );
            (*(*args).u.fs).sample_count = sample_count;
        }

        WINED3D_SHADER_TYPE_VERTEX => {
            let mut ps: *mut wined3d_shader =
                (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize];

            if ((*shader).reg_maps.shader_version.major as ::core::ffi::c_int)
                < 4 as ::core::ffi::c_int
                && !ps.is_null()
            {
                let mut vs_program: *mut shader_spirv_graphics_program_vk =
                    (*shader).backend_data as *mut shader_spirv_graphics_program_vk;
                let mut ps_program: *mut shader_spirv_graphics_program_vk =
                    (*ps).backend_data as *mut shader_spirv_graphics_program_vk;

                if (*ps_program).signature_info.input.element_count as usize
                    > ARRAY_SIZE!((*(*args).u.vs).varying_map) as usize
                {
                    ERR!(
                        "Unexpected inter-stage varying count %u.\n",
                        (*ps_program).signature_info.input.element_count
                    );
                }
                vkd3d_shader_build_varying_map(
                    &raw mut (*vs_program).signature_info.output,
                    &raw mut (*ps_program).signature_info.input,
                    &raw mut (*(*args).u.vs).varying_count,
                    &raw mut (*(*args).u.vs).varying_map as *mut vkd3d_shader_varying_map,
                );
            }
            // break;
        }

        _ => {}
    };
}

unsafe extern "C" fn fill_ps_parameters(
    mut vkd3d_args: *mut wined3d_shader_spirv_compile_args,
    mut compile_args: *const shader_spirv_compile_arguments,
    mut ffp_extra_binding: uint32_t,
) {
    let mut parameters: *mut vkd3d_shader_parameter1 =
        core::ptr::addr_of_mut!((*vkd3d_args).parameters) as *mut vkd3d_shader_parameter1;

    (*parameters.offset(0 as isize)).name = VKD3D_SHADER_PARAMETER_NAME_RASTERIZER_SAMPLE_COUNT;
    (*parameters.offset(0 as isize)).type_ = VKD3D_SHADER_PARAMETER_TYPE_IMMEDIATE_CONSTANT;
    (*parameters.offset(0 as isize)).data_type = VKD3D_SHADER_PARAMETER_DATA_TYPE_UINT32;
    (*parameters.offset(0 as isize))
        .u
        .immediate_constant
        .u
        .u32_ = (&(*compile_args).u.fs).sample_count as uint32_t;

    (*parameters.offset(1 as isize)).name = VKD3D_SHADER_PARAMETER_NAME_FLAT_INTERPOLATION;
    (*parameters.offset(1 as isize)).type_ = VKD3D_SHADER_PARAMETER_TYPE_IMMEDIATE_CONSTANT;
    (*parameters.offset(1 as isize)).data_type = VKD3D_SHADER_PARAMETER_DATA_TYPE_UINT32;
    (*parameters.offset(1 as isize))
        .u
        .immediate_constant
        .u
        .u32_ = (&(*compile_args).u.fs).args.flatshading() as uint32_t;

    (*parameters.offset(2 as isize)).name = VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_FUNC;
    (*parameters.offset(2 as isize)).type_ = VKD3D_SHADER_PARAMETER_TYPE_IMMEDIATE_CONSTANT;
    (*parameters.offset(2 as isize)).data_type = VKD3D_SHADER_PARAMETER_DATA_TYPE_UINT32;
    (*parameters.offset(2 as isize))
        .u
        .immediate_constant
        .u
        .u32_ = ((&(*compile_args).u.fs).args.alpha_test_func() as ::core::ffi::c_int
        + 1) as uint32_t;

    (*parameters.offset(3 as isize)).name = VKD3D_SHADER_PARAMETER_NAME_ALPHA_TEST_REF;
    (*parameters.offset(3 as isize)).type_ = VKD3D_SHADER_PARAMETER_TYPE_BUFFER;
    (*parameters.offset(3 as isize)).data_type = VKD3D_SHADER_PARAMETER_DATA_TYPE_FLOAT32;
    (*parameters.offset(3 as isize))
        .u
        .buffer
        .set = 0;
    (*parameters.offset(3 as isize))
        .u
        .buffer
        .binding = ffp_extra_binding;
    (*parameters.offset(3 as isize))
        .u
        .buffer
        .offset = core::mem::offset_of!(wined3d_ffp_ps_constants, alpha_test_ref) as uint32_t;

    (*vkd3d_args).parameter_info.parameter_count = 4;
    (*vkd3d_args).parameter_info.parameters =
        core::ptr::addr_of_mut!((*vkd3d_args).parameters) as *mut vkd3d_shader_parameter1;
}

unsafe extern "C" fn shader_spirv_init_compile_args(
    vk_info: *const wined3d_vk_info,
    args: *mut wined3d_shader_spirv_compile_args,
    vkd3d_interface: *mut vkd3d_shader_interface_info,
    environment: vkd3d_shader_spirv_environment,
    shader_type: wined3d_shader_type,
    source_type: vkd3d_shader_source_type,
    compile_args: *const shader_spirv_compile_arguments,
    bindings: *const shader_spirv_resource_bindings,
) {
    let mut i: u32;

    memset(
        args as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<wined3d_shader_spirv_compile_args>(),
    );
    (*args).spirv_target.type_ = VKD3D_SHADER_STRUCTURE_TYPE_SPIRV_TARGET_INFO;
    (*args).spirv_target.next =
        core::ptr::addr_of_mut!((*args).parameter_info) as *const core::ffi::c_void;
    (*args).spirv_target.entry_point = b"main\0".as_ptr() as *const core::ffi::c_char;
    (*args).spirv_target.environment = environment;

    (*args).parameter_info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_PARAMETER_INFO;
    (*args).parameter_info.next = vkd3d_interface as *const core::ffi::c_void;

    (*args).spirv_target.extensions =
        core::ptr::addr_of_mut!((*args).extensions) as *mut vkd3d_shader_spirv_extension;

    if (*vk_info).supported[WINED3D_VK_EXT_SHADER_STENCIL_EXPORT as usize] != 0 {
        let fresh15 = (*args).spirv_target.extension_count;
        (*args).spirv_target.extension_count = fresh15.wrapping_add(1);
        (*args).extensions[fresh15 as usize] = VKD3D_SHADER_SPIRV_EXTENSION_EXT_STENCIL_EXPORT;
    }
    assert!(
        ((*args).spirv_target.extension_count as usize)
            <= ARRAY_SIZE!((*args).extensions) as usize,
    );

    if shader_type == WINED3D_SHADER_TYPE_PIXEL {
        let rt_alpha_swizzle: u32 = (&(*compile_args).u.fs).args.rt_alpha_swizzle() as u32;

        fill_ps_parameters(
            args,
            compile_args,
            (*bindings).ffp_extra_binding[WINED3D_SHADER_TYPE_PIXEL as usize],
        );

        (*args).spirv_target.dual_source_blending =
            (&(*compile_args).u.fs).args.dual_source_blend() != 0;

        i = 0;
        while (i as usize) < (ARRAY_SIZE!((*args).ps_alpha_swizzle) as usize) {
            // DIVERGES from the C, which tests `rt_alpha_swizzle && (1u << i)`: the second
            // operand is a nonzero constant, so every render target was swizzled whenever ANY
            // was. rt_alpha_swizzle is a per-RT bit mask; test this RT's bit.
            if rt_alpha_swizzle & (1u32 << i) != 0 {
                // VKD3D_SHADER_SWIZZLE(W, X, Y, Z): component i in byte i (vkd3d_shader.h).
                (*args).ps_alpha_swizzle[i as usize] =
                    (vkd3d_shader_swizzle_component_VKD3D_SHADER_SWIZZLE_W as u32)
                        | (vkd3d_shader_swizzle_component_VKD3D_SHADER_SWIZZLE_X as u32) << 8
                        | (vkd3d_shader_swizzle_component_VKD3D_SHADER_SWIZZLE_Y as u32) << 16
                        | (vkd3d_shader_swizzle_component_VKD3D_SHADER_SWIZZLE_Z as u32) << 24;
            } else {
                (*args).ps_alpha_swizzle[i as usize] = (VKD3D_SHADER_NO_SWIZZLE as u32);
            }
            i = i.wrapping_add(1);
        }

        (*args).spirv_target.output_swizzles =
            core::ptr::addr_of_mut!((*args).ps_alpha_swizzle) as *mut u32;
        (*args).spirv_target.output_swizzle_count =
            ARRAY_SIZE!((*args).ps_alpha_swizzle) as u32;
    } else if shader_type == WINED3D_SHADER_TYPE_VERTEX {
        if source_type == VKD3D_SHADER_SOURCE_D3D_BYTECODE {
            (*args).spirv_target.next =
                core::ptr::addr_of_mut!((*args).varying_map) as *const core::ffi::c_void;

            (*args).varying_map.type_ = VKD3D_SHADER_STRUCTURE_TYPE_VARYING_MAP_INFO;
            (*args).varying_map.next = vkd3d_interface as *const core::ffi::c_void;
            (*args).varying_map.varying_map =
                core::ptr::addr_of!((&(*compile_args).u.vs).varying_map) as *const vkd3d_shader_varying_map;
            (*args).varying_map.varying_count = (&(*compile_args).u.vs).varying_count;
        }
    }
}

unsafe extern "C" fn shader_spirv_init_shader_interface_vk(
    mut iface: *mut wined3d_shader_spirv_shader_interface,
    mut b: *const shader_spirv_resource_bindings,
    mut so_desc: *const wined3d_stream_output_desc,
) {
    memset(
        iface as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<wined3d_shader_spirv_shader_interface>() as size_t,
    );
    (*iface).vkd3d_interface.type_ = VKD3D_SHADER_STRUCTURE_TYPE_INTERFACE_INFO;
    if !so_desc.is_null() {
        (*iface).xfb_info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_TRANSFORM_FEEDBACK_INFO;
        (*iface).xfb_info.next = ::core::ptr::null::<::core::ffi::c_void>();
        (*iface).xfb_info.elements =
            (*so_desc).elements as *const vkd3d_shader_transform_feedback_element;
        (*iface).xfb_info.element_count = (*so_desc).element_count;
        (*iface).xfb_info.buffer_strides =
            &raw const (*so_desc).buffer_strides as *const ::core::ffi::c_uint;
        (*iface).xfb_info.buffer_stride_count = (*so_desc).buffer_stride_count;
        (*iface).vkd3d_interface.next = &raw mut (*iface).xfb_info as *const ::core::ffi::c_void;
    }
    (*iface).vkd3d_interface.bindings = (*b).bindings;
    (*iface).vkd3d_interface.binding_count = (*b).binding_count as ::core::ffi::c_uint;
    (*iface).vkd3d_interface.uav_counters =
        &raw const (*b).uav_counters as *const vkd3d_shader_uav_counter_binding;
    (*iface).vkd3d_interface.uav_counter_count = (*b).uav_counter_count as ::core::ffi::c_uint;
}

unsafe extern "C" fn shader_spirv_compile_shader(
    context_vk: *mut wined3d_context_vk,
    shader_desc: *const wined3d_shader_desc,
    source_type: vkd3d_shader_source_type,
    shader_type: wined3d_shader_type,
    args: *const shader_spirv_compile_arguments,
    bindings: *const shader_spirv_resource_bindings,
    so_desc: *const wined3d_stream_output_desc,
) -> VkShaderModule {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*device_vk).vk_info);
    let mut compile_args: wined3d_shader_spirv_compile_args = core::mem::zeroed();
    let mut iface: wined3d_shader_spirv_shader_interface = core::mem::zeroed();
    let mut shader_create_info: VkShaderModuleCreateInfo = core::mem::zeroed();
    let mut info: vkd3d_shader_compile_info = core::mem::zeroed();
    let mut spirv: vkd3d_shader_code = core::mem::zeroed();
    let mut module: VkShaderModule = 0;
    let mut messages: *mut core::ffi::c_char = core::ptr::null_mut();
    let mut vr: VkResult = 0;
    let mut ret: core::ffi::c_int = 0;

    shader_spirv_init_shader_interface_vk(core::ptr::addr_of_mut!(iface), bindings, so_desc);
    shader_spirv_init_compile_args(
        vk_info,
        core::ptr::addr_of_mut!(compile_args),
        core::ptr::addr_of_mut!(iface.vkd3d_interface),
        VKD3D_SHADER_SPIRV_ENVIRONMENT_VULKAN_1_0,
        shader_type,
        source_type,
        args,
        bindings,
    );

    info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO;
    info.next = core::ptr::addr_of_mut!(compile_args.spirv_target) as *const core::ffi::c_void;
    info.source.code = (*shader_desc).byte_code as *const core::ffi::c_void;
    info.source.size = (*shader_desc).byte_code_size;
    info.source_type = source_type;
    info.target_type = VKD3D_SHADER_TARGET_SPIRV_BINARY;
    info.options = core::ptr::addr_of!(spirv_compile_options) as *const vkd3d_shader_compile_option;
    info.option_count = ARRAY_SIZE!(spirv_compile_options) as u32;
    info.log_level = VKD3D_SHADER_LOG_WARNING;
    info.source_name = core::ptr::null();

    ret = vkd3d_shader_compile(
        core::ptr::addr_of_mut!(info),
        core::ptr::addr_of_mut!(spirv),
        core::ptr::addr_of_mut!(messages),
    );
    if !messages.is_null() && *messages != 0 && ((FIXME_ON!(__wine_dbch_d3d_shader)) != 0) {
        let mut ptr: *const core::ffi::c_char = core::ptr::null();
        let mut end: *const core::ffi::c_char = core::ptr::null();
        let mut line: *const core::ffi::c_char = core::ptr::null();

        ptr = messages;
        end = ptr.offset(strlen(ptr) as isize);
        loop {
            line = wined3d_get_line(core::ptr::addr_of_mut!(ptr), end);
            if line.is_null() {
                break;
            }
            FIXME!("    %.*s\n", ptr.offset_from(line) as core::ffi::c_int, line);
        }
        FIXME!("\n");
    }
    vkd3d_shader_free_messages(messages);

    if ret < 0 {
        ERR!("Failed to compile shader, ret %d.\n", ret);
        return (VK_NULL_HANDLE) as VkShaderModule;
    }

    shader_create_info.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
    shader_create_info.pNext = core::ptr::null();
    shader_create_info.flags = 0;
    shader_create_info.codeSize = spirv.size;
    shader_create_info.pCode = spirv.code as *const u32;
    vr = ((*vk_info).vk_ops.vkCreateShaderModule.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(shader_create_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!(module),
    );
    if (vr as i32) < 0 {
        vkd3d_shader_free_shader_code(core::ptr::addr_of_mut!(spirv));
        WARN!(
            "Failed to create Vulkan shader module, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return (VK_NULL_HANDLE) as VkShaderModule;
    }

    vkd3d_shader_free_shader_code(core::ptr::addr_of_mut!(spirv));

    return module;
}

unsafe extern "C" fn shader_spirv_find_graphics_program_variant_vk(
    mut priv_0: *mut shader_spirv_priv,
    mut context_vk: *mut wined3d_context_vk,
    mut shader: *mut wined3d_shader,
    mut state: *const wined3d_state,
    mut bindings: *const shader_spirv_resource_bindings,
) -> *mut shader_spirv_graphics_program_variant_vk {
    let mut shader_type: wined3d_shader_type = (*shader).reg_maps.shader_version.type_;
    let mut variant_vk: *mut shader_spirv_graphics_program_variant_vk =
        ::core::ptr::null_mut::<shader_spirv_graphics_program_variant_vk>();
    let mut binding_base: size_t = (*bindings).binding_base[shader_type as usize];
    let mut so_desc: *const wined3d_stream_output_desc =
        ::core::ptr::null::<wined3d_stream_output_desc>();
    let mut program_vk: *mut shader_spirv_graphics_program_vk =
        ::core::ptr::null_mut::<shader_spirv_graphics_program_vk>();
    let mut args: shader_spirv_compile_arguments =
        ::core::mem::zeroed::<shader_spirv_compile_arguments>();
    let mut shader_desc: wined3d_shader_desc = wined3d_shader_desc {
        byte_code: ::core::ptr::null::<DWORD>(),
        byte_code_size: 0,
    };
    let mut variant_count: size_t = 0;
    let mut i: size_t = 0;
    shader_spirv_compile_arguments_init(
        &raw mut args,
        &raw mut (*context_vk).c,
        shader,
        state,
        (*context_vk).sample_count as ::core::ffi::c_uint,
    );
    if (*bindings).so_stage as ::core::ffi::c_uint == shader_type as ::core::ffi::c_uint {
        so_desc = (*(*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize])
            .u
            .gs
            .so_desc;
    }
    program_vk = (*shader).backend_data as *mut shader_spirv_graphics_program_vk;
    if program_vk.is_null() {
        return ::core::ptr::null_mut::<shader_spirv_graphics_program_variant_vk>();
    }
    variant_count = (*program_vk).variant_count as size_t;
    i = 0 as size_t;
    while i < variant_count {
        variant_vk = (*program_vk).variants.offset(i as isize)
            as *mut shader_spirv_graphics_program_variant_vk;
        if (*variant_vk).so_desc == so_desc
            && (*variant_vk).binding_base == binding_base
            && memcmp(
                &raw mut (*variant_vk).compile_args as *const ::core::ffi::c_void,
                &raw mut args as *const ::core::ffi::c_void,
                ::core::mem::size_of::<shader_spirv_compile_arguments>() as size_t,
            ) == 0
        {
            return variant_vk;
        }
        i = i.wrapping_add(1);
    }
    if wined3d_array_reserve(
        &raw mut (*program_vk).variants as *mut *mut ::core::ffi::c_void,
        &raw mut (*program_vk).variants_size as *mut SIZE_T,
        (variant_count as SIZE_T).wrapping_add(1 as SIZE_T),
        ::core::mem::size_of::<shader_spirv_graphics_program_variant_vk>() as SIZE_T,
    ) == 0
    {
        return ::core::ptr::null_mut::<shader_spirv_graphics_program_variant_vk>();
    }
    variant_vk = (*program_vk).variants.offset(variant_count as isize)
        as *mut shader_spirv_graphics_program_variant_vk;
    (*variant_vk).compile_args = ::core::ptr::read(&raw const args);
    (*variant_vk).binding_base = binding_base;
    if (*shader).source_type as ::core::ffi::c_uint
        == VKD3D_SHADER_SOURCE_D3D_BYTECODE as ::core::ffi::c_uint
    {
        shader_desc.byte_code = (*shader).function;
        shader_desc.byte_code_size = (*shader).functionLength as size_t;
    } else {
        shader_desc.byte_code = (*shader).byte_code as *const DWORD;
        shader_desc.byte_code_size = (*shader).byte_code_size as size_t;
    }
    (*variant_vk).vk_module = shader_spirv_compile_shader(
        context_vk,
        &raw mut shader_desc,
        (*shader).source_type,
        shader_type,
        &raw mut args,
        bindings,
        so_desc,
    );
    if (*variant_vk).vk_module == 0 {
        return ::core::ptr::null_mut::<shader_spirv_graphics_program_variant_vk>();
    }
    (*program_vk).variant_count = (*program_vk).variant_count.wrapping_add(1);
    return variant_vk;
}

unsafe extern "C" fn shader_spirv_find_compute_program_vk(
    priv_: *mut shader_spirv_priv,
    context_vk: *mut wined3d_context_vk,
    shader: *mut wined3d_shader,
    bindings: *const shader_spirv_resource_bindings,
) -> *mut shader_spirv_compute_program_vk {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut program: *mut shader_spirv_compute_program_vk = core::ptr::null_mut();
    let mut layout: *mut wined3d_pipeline_layout_vk = core::ptr::null_mut();
    let mut pipeline_info: VkComputePipelineCreateInfo = core::mem::zeroed();
    let mut shader_desc: wined3d_shader_desc = core::mem::zeroed();
    let mut vr: VkResult = 0;

    program = (*shader).backend_data as *mut shader_spirv_compute_program_vk;
    if program.is_null() {
        return core::ptr::null_mut();
    }

    if !((*program).vk_module == 0) {
        return program;
    }

    shader_desc.byte_code = (*shader).byte_code as *const DWORD;
    shader_desc.byte_code_size = (*shader).byte_code_size as size_t;

    (*program).vk_module = shader_spirv_compile_shader(
        context_vk,
        &mut shader_desc,
        (*shader).source_type,
        WINED3D_SHADER_TYPE_COMPUTE,
        core::ptr::null(),
        bindings,
        core::ptr::null(),
    );
    if ((*program).vk_module == 0) {
        return core::ptr::null_mut();
    }

    layout = wined3d_context_vk_get_pipeline_layout(
        context_vk,
        (*bindings).vk_bindings,
        ((*bindings).vk_binding_count) as SIZE_T,
    );
    if layout.is_null() {
        ((*vk_info).vk_ops.vkDestroyShaderModule.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*program).vk_module,
            core::ptr::null(),
        );
        (*program).vk_module = (VK_NULL_HANDLE as _);
        return core::ptr::null_mut();
    }
    (*program).vk_set_layout = (*layout).vk_set_layout;
    (*program).vk_pipeline_layout = (*layout).vk_pipeline_layout;

    pipeline_info.sType = VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO;
    pipeline_info.pNext = core::ptr::null();
    pipeline_info.flags = 0;
    pipeline_info.stage.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
    pipeline_info.stage.pNext = core::ptr::null();
    pipeline_info.stage.flags = 0;
    pipeline_info.stage.stage = VK_SHADER_STAGE_COMPUTE_BIT;
    pipeline_info.stage.pName = b"main\0".as_ptr() as *const core::ffi::c_char;
    pipeline_info.stage.pSpecializationInfo = core::ptr::null();
    pipeline_info.stage.module = (*program).vk_module;
    pipeline_info.layout = (*program).vk_pipeline_layout;
    pipeline_info.basePipelineHandle = (VK_NULL_HANDLE as _);
    pipeline_info.basePipelineIndex = -1;

    vr = ((*vk_info).vk_ops.vkCreateComputePipelines.unwrap_unchecked())(
        (*device_vk).vk_device,
        (VK_NULL_HANDLE as _),
        1,
        &mut pipeline_info,
        core::ptr::null(),
        &mut (*program).vk_pipeline,
    );
    if vr < 0 {
        ERR!(
            "Failed to create Vulkan compute pipeline, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        ((*vk_info).vk_ops.vkDestroyShaderModule.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*program).vk_module,
            core::ptr::null(),
        );
        (*program).vk_module = (VK_NULL_HANDLE as _);
        return core::ptr::null_mut();
    }

    program
}

unsafe extern "C" fn shader_spirv_resource_bindings_cleanup(bindings: *mut shader_spirv_resource_bindings) {
    free((*bindings).vk_bindings as *mut ::core::ffi::c_void);
    free((*bindings).bindings as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn shader_spirv_resource_bindings_add_vk_binding(
    bindings: *mut shader_spirv_resource_bindings,
    vk_type: VkDescriptorType,
    vk_stage: VkShaderStageFlagBits,
    binding_idx: *mut size_t,
) -> bool {
    let mut binding_count: SIZE_T = ((*bindings).vk_binding_count) as _;
    let mut binding: *mut VkDescriptorSetLayoutBinding = core::ptr::null_mut();
    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*bindings).vk_bindings) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*bindings).vk_bindings_size) as *mut SIZE_T,
        binding_count + 1 as SIZE_T,
        core::mem::size_of::<VkDescriptorSetLayoutBinding>() as SIZE_T,
    ) == 0
    {
        return false;
    }
    *binding_idx = binding_count as size_t;
    binding = (*bindings).vk_bindings.add(binding_count as usize) as *mut VkDescriptorSetLayoutBinding;
    (*binding).binding = binding_count as uint32_t;
    (*binding).descriptorType = vk_type;
    (*binding).descriptorCount = 1 as uint32_t;
    (*binding).stageFlags = vk_stage as VkShaderStageFlags;
    (*binding).pImmutableSamplers = core::ptr::null::<VkSampler>() as *const VkSampler;
    (*bindings).vk_binding_count = (*bindings).vk_binding_count.wrapping_add(1);
    return true;
}

unsafe extern "C" fn shader_spirv_resource_bindings_add_binding(
    bindings: *mut shader_spirv_resource_bindings,
    vkd3d_type: vkd3d_shader_descriptor_type,
    vk_type: VkDescriptorType,
    register_idx: size_t,
    vkd3d_visibility: vkd3d_shader_visibility,
    vk_stage: VkShaderStageFlagBits,
    flags: uint32_t,
    binding_idx: *mut size_t,
) -> bool {
    let binding_count: SIZE_T = ((*bindings).binding_count) as _;
    let mut binding: *mut vkd3d_shader_resource_binding =
        ::core::ptr::null_mut::<vkd3d_shader_resource_binding>();
    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*bindings).bindings) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!((*bindings).bindings_size) as *mut SIZE_T,
        binding_count.wrapping_add(1 as SIZE_T),
        ::core::mem::size_of::<vkd3d_shader_resource_binding>() as SIZE_T,
    ) == 0
    {
        return false;
    }
    if !shader_spirv_resource_bindings_add_vk_binding(bindings, vk_type, vk_stage, binding_idx) {
        return false;
    }
    binding =
        (*bindings).bindings.offset(binding_count as isize) as *mut vkd3d_shader_resource_binding;
    (*binding).type_ = vkd3d_type;
    (*binding).register_space = 0 as ::core::ffi::c_uint;
    (*binding).register_index = register_idx as ::core::ffi::c_uint;
    (*binding).shader_visibility = vkd3d_visibility;
    (*binding).flags = flags as ::core::ffi::c_uint;
    (*binding).binding.set = 0 as ::core::ffi::c_uint;
    (*binding).binding.binding = *binding_idx as ::core::ffi::c_uint;
    (*binding).binding.count = 1 as ::core::ffi::c_uint;
    (*bindings).binding_count = (*bindings).binding_count.wrapping_add(1);
    return true;
}

unsafe extern "C" fn shader_spirv_resource_bindings_add_uav_counter_binding(
    bindings: *mut shader_spirv_resource_bindings,
    register_idx: size_t,
    vkd3d_visibility: vkd3d_shader_visibility,
    vk_stage: VkShaderStageFlagBits,
    binding_idx: *mut size_t,
) -> bool {
    let uav_counter_count: SIZE_T = ((*bindings).uav_counter_count) as _;
    let mut counter: *mut vkd3d_shader_uav_counter_binding = core::ptr::null_mut();

    if uav_counter_count >= ARRAY_SIZE!((*bindings).uav_counters) as SIZE_T {
        return false;
    }

    if !shader_spirv_resource_bindings_add_vk_binding(
        bindings,
        VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER,
        vk_stage,
        binding_idx,
    ) {
        return false;
    }

    counter = core::ptr::addr_of_mut!((*bindings).uav_counters) as *mut vkd3d_shader_uav_counter_binding;
    counter = counter.offset(uav_counter_count as isize) as *mut vkd3d_shader_uav_counter_binding;
    (*counter).register_space = 0;
    (*counter).register_index = register_idx as ::core::ffi::c_uint;
    (*counter).shader_visibility = vkd3d_visibility;
    (*counter).binding.set = 0;
    (*counter).binding.binding = *binding_idx as ::core::ffi::c_uint;
    (*counter).binding.count = 1;
    (*counter).offset = 0;
    (*bindings).uav_counter_count = (*bindings).uav_counter_count.wrapping_add(1);

    return true;
}

unsafe extern "C" fn wined3d_shader_resource_bindings_add_binding(bindings: *mut wined3d_shader_resource_bindings, shader_type: wined3d_shader_type, shader_descriptor_type: wined3d_shader_descriptor_type, resource_idx: size_t, resource_type: wined3d_shader_resource_type, resource_data_type: wined3d_data_type, binding_idx: size_t) -> bool {
    let mut binding: *mut wined3d_shader_resource_binding;
    let mut binding_count: SIZE_T = (*bindings).count;

    if ((wined3d_array_reserve(core::ptr::addr_of_mut!((*bindings).bindings) as *mut *mut core::ffi::c_void, core::ptr::addr_of_mut!((*bindings).size), binding_count + 1, core::mem::size_of::<wined3d_shader_resource_binding>() as SIZE_T)) == 0) {
        return false;
    }

    binding = (*bindings).bindings.offset(binding_count as isize) as *mut wined3d_shader_resource_binding;
    (*binding).shader_type = shader_type;
    (*binding).shader_descriptor_type = shader_descriptor_type;
    (*binding).resource_idx = resource_idx;
    (*binding).resource_type = resource_type;
    (*binding).resource_data_type = resource_data_type;
    (*binding).binding_idx = binding_idx;
    (*bindings).count += 1;

    return true;
}

unsafe extern "C" fn vk_descriptor_type_from_vkd3d(
    type_0: vkd3d_shader_descriptor_type,
    resource_type: vkd3d_shader_resource_type,
) -> VkDescriptorType {
    match type_0 {
        VKD3D_SHADER_DESCRIPTOR_TYPE_CBV => {
            return VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_SRV => {
            if resource_type == VKD3D_SHADER_RESOURCE_BUFFER {
                return VK_DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER;
            }
            return VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_UAV => {
            if resource_type == VKD3D_SHADER_RESOURCE_BUFFER {
                return VK_DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER;
            }
            return VK_DESCRIPTOR_TYPE_STORAGE_IMAGE;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_SAMPLER => {
            return VK_DESCRIPTOR_TYPE_SAMPLER;
        }
        _ => {
            FIXME!("Unhandled descriptor type %#x.\n", type_0);
            return !(0 as u32) as VkDescriptorType;
        }
    }
}

unsafe extern "C" fn wined3d_descriptor_type_from_vkd3d(
    mut type_0: vkd3d_shader_descriptor_type,
) -> wined3d_shader_descriptor_type {
    match type_0 as u32 {
        VKD3D_SHADER_DESCRIPTOR_TYPE_CBV => {
            return WINED3D_SHADER_DESCRIPTOR_TYPE_CBV;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_SRV => {
            return WINED3D_SHADER_DESCRIPTOR_TYPE_SRV;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_UAV => {
            return WINED3D_SHADER_DESCRIPTOR_TYPE_UAV;
        }
        VKD3D_SHADER_DESCRIPTOR_TYPE_SAMPLER => {
            return WINED3D_SHADER_DESCRIPTOR_TYPE_SAMPLER;
        }
        _ => {
            FIXME!("Unhandled descriptor type %#x.\n", type_0);
            return !(0) as wined3d_shader_descriptor_type;
        }
    }
}

unsafe extern "C" fn wined3d_shader_resource_type_from_vkd3d(t: vkd3d_shader_resource_type) -> wined3d_shader_resource_type {
    return t as wined3d_shader_resource_type;
}

unsafe extern "C" fn wined3d_data_type_from_vkd3d(t: vkd3d_shader_resource_data_type) -> wined3d_data_type {
    match t {
        VKD3D_SHADER_RESOURCE_DATA_UNORM => {
            return WINED3D_DATA_UNORM;
        }
        VKD3D_SHADER_RESOURCE_DATA_SNORM => {
            return WINED3D_DATA_SNORM;
        }
        VKD3D_SHADER_RESOURCE_DATA_INT => {
            return WINED3D_DATA_INT;
        }
        VKD3D_SHADER_RESOURCE_DATA_UINT => {
            return WINED3D_DATA_UINT;
        }
        VKD3D_SHADER_RESOURCE_DATA_FLOAT => {
            return WINED3D_DATA_FLOAT;
        }
        VKD3D_SHADER_RESOURCE_DATA_MIXED => {
            return WINED3D_DATA_UINT;
        }
        _ => {
            FIXME!("Unhandled resource data type %#x.\n", t);
            return WINED3D_DATA_FLOAT;
        }
    }
}

unsafe extern "C" fn shader_spirv_resource_bindings_init(
    bindings: *mut shader_spirv_resource_bindings,
    wined3d_bindings: *mut wined3d_shader_resource_bindings,
    state: *const wined3d_state,
    shader_mask: uint32_t,
) -> bool {
    let mut descriptor_info: *const vkd3d_shader_scan_descriptor_info =
        core::ptr::null::<vkd3d_shader_scan_descriptor_info>();
    let mut wined3d_type: wined3d_shader_descriptor_type = WINED3D_SHADER_DESCRIPTOR_TYPE_CBV;
    let mut shader_visibility: vkd3d_shader_visibility = VKD3D_SHADER_VISIBILITY_ALL;
    let mut shader_type: wined3d_shader_type = WINED3D_SHADER_TYPE_PIXEL;
    let mut vk_descriptor_type: VkDescriptorType = VK_DESCRIPTOR_TYPE_SAMPLER;
    let mut vk_stage: VkShaderStageFlagBits = 0 as VkShaderStageFlagBits;
    let mut shader: *mut wined3d_shader = core::ptr::null_mut::<wined3d_shader>();
    let mut binding_idx: size_t = 0;
    let mut i: core::ffi::c_uint = 0;

    (*bindings).binding_count = (0 as SIZE_T) as _;
    (*bindings).uav_counter_count = (0 as SIZE_T) as _;
    (*bindings).vk_binding_count = (0 as SIZE_T) as _;
    (*bindings).so_stage = WINED3D_SHADER_TYPE_GEOMETRY;
    (*wined3d_bindings).count = 0 as SIZE_T;

    shader_type = 0;
    while shader_type < WINED3D_SHADER_TYPE_COUNT {
        (*bindings).binding_base[shader_type as usize] = (*bindings).vk_binding_count as size_t;

        if !((shader_mask & (1 << shader_type)) == 0 || {
            shader = (*state).shader[shader_type as usize];
            shader.is_null()
        }) {
            if shader_type == WINED3D_SHADER_TYPE_COMPUTE {
                descriptor_info = core::ptr::addr_of!(
                    (*((*shader).backend_data as *mut shader_spirv_compute_program_vk))
                        .descriptor_info
                );
            } else {
                descriptor_info = core::ptr::addr_of!(
                    (*((*shader).backend_data as *mut shader_spirv_graphics_program_vk))
                        .descriptor_info
                );
                if shader_type == WINED3D_SHADER_TYPE_GEOMETRY && (*shader).function.is_null() {
                    (*bindings).so_stage = WINED3D_SHADER_TYPE_VERTEX;
                }
            }

            vk_stage = vk_shader_stage_from_wined3d(shader_type);
            shader_visibility = vkd3d_shader_visibility_from_wined3d(shader_type);

            i = 0;
            while i < (*descriptor_info).descriptor_count {
                let d: *const vkd3d_shader_descriptor_info =
                    (*descriptor_info).descriptors.offset(i as isize) as *const vkd3d_shader_descriptor_info;
                let mut flags: uint32_t = 0;

                if (*d).register_space != 0 {
                    WARN!("Unsupported register space %u.\n", (*d).register_space);
                    return false;
                }

                if (*d).resource_type == VKD3D_SHADER_RESOURCE_BUFFER {
                    flags = VKD3D_SHADER_BINDING_FLAG_BUFFER;
                } else {
                    flags = VKD3D_SHADER_BINDING_FLAG_IMAGE;
                }

                vk_descriptor_type = vk_descriptor_type_from_vkd3d((*d).type_, (*d).resource_type);
                if !shader_spirv_resource_bindings_add_binding(
                    bindings,
                    (*d).type_,
                    vk_descriptor_type,
                    (*d).register_index as size_t,
                    shader_visibility,
                    vk_stage,
                    flags,
                    core::ptr::addr_of_mut!(binding_idx),
                ) {
                    return false;
                }

                wined3d_type = wined3d_descriptor_type_from_vkd3d((*d).type_);
                if !wined3d_shader_resource_bindings_add_binding(
                    wined3d_bindings,
                    shader_type,
                    wined3d_type,
                    (*d).register_index as size_t,
                    wined3d_shader_resource_type_from_vkd3d((*d).resource_type),
                    wined3d_data_type_from_vkd3d((*d).resource_data_type),
                    binding_idx,
                ) {
                    return false;
                }

                if (*d).type_ == VKD3D_SHADER_DESCRIPTOR_TYPE_UAV
                    && ((*d).flags & VKD3D_SHADER_DESCRIPTOR_INFO_FLAG_UAV_COUNTER) != 0
                {
                    if !shader_spirv_resource_bindings_add_uav_counter_binding(
                        bindings,
                        (*d).register_index as size_t,
                        shader_visibility,
                        vk_stage,
                        core::ptr::addr_of_mut!(binding_idx),
                    ) {
                        return false;
                    }
                    if !wined3d_shader_resource_bindings_add_binding(
                        wined3d_bindings,
                        shader_type,
                        WINED3D_SHADER_DESCRIPTOR_TYPE_UAV_COUNTER,
                        (*d).register_index as size_t,
                        WINED3D_SHADER_RESOURCE_BUFFER,
                        WINED3D_DATA_UINT,
                        binding_idx,
                    ) {
                        return false;
                    }
                }

                i = i.wrapping_add(1);
            }

            /* vkd3d-shader doesn't scan the uniform buffers we use for parameters.
             * It can't really be reasonably modified to do so either, because
             * parameters are specified in terms of the destination environment
             * (SPIRV binding set and index). Add them manually here.
             *
             * Note that vkd3d-shader will ignore the extra binding specified in
             * struct shader_spirv_resource_bindings, but we use it elsewhere. */

            if (*shader).source_type == VKD3D_SHADER_SOURCE_D3D_BYTECODE {
                if !shader_spirv_resource_bindings_add_binding(
                    bindings,
                    VKD3D_SHADER_DESCRIPTOR_TYPE_CBV,
                    VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                    (WINED3D_FFP_CONSTANTS_EXTRA_REGISTER) as size_t,
                    shader_visibility,
                    vk_stage,
                    VKD3D_SHADER_BINDING_FLAG_BUFFER,
                    core::ptr::addr_of_mut!(binding_idx),
                ) {
                    return false;
                }
                if !wined3d_shader_resource_bindings_add_binding(
                    wined3d_bindings,
                    shader_type,
                    WINED3D_SHADER_DESCRIPTOR_TYPE_CBV,
                    (WINED3D_FFP_CONSTANTS_EXTRA_REGISTER) as size_t,
                    WINED3D_SHADER_RESOURCE_BUFFER,
                    WINED3D_DATA_FLOAT,
                    binding_idx,
                ) {
                    return false;
                }
                (*bindings).ffp_extra_binding[shader_type as usize] = binding_idx as uint32_t;
            }
        }

        shader_type += 1;
    }

    return true;
}

unsafe extern "C" fn shader_spirv_scan_shader(
    mut shader: *mut wined3d_shader,
    mut descriptor_info: *mut vkd3d_shader_scan_descriptor_info,
    mut signature_info: *mut vkd3d_shader_scan_signature_info,
) {
    let mut info: vkd3d_shader_compile_info = vkd3d_shader_compile_info {
        type_: VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO,
        next: core::ptr::null::<core::ffi::c_void>(),
        source: vkd3d_shader_code {
            code: core::ptr::null::<core::ffi::c_void>(),
            size: 0,
        },
        source_type: VKD3D_SHADER_SOURCE_NONE,
        target_type: VKD3D_SHADER_TARGET_NONE,
        options: core::ptr::null::<vkd3d_shader_compile_option>(),
        option_count: 0,
        log_level: VKD3D_SHADER_LOG_NONE,
        source_name: core::ptr::null::<core::ffi::c_char>(),
    };
    let mut messages: *mut core::ffi::c_char = core::ptr::null_mut::<core::ffi::c_char>();
    let mut ret: core::ffi::c_int = 0;

    memset(
        descriptor_info as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<vkd3d_shader_scan_descriptor_info>() as usize,
    );
    (*descriptor_info).type_ = VKD3D_SHADER_STRUCTURE_TYPE_SCAN_DESCRIPTOR_INFO;

    if !signature_info.is_null() {
        memset(
            signature_info as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<vkd3d_shader_scan_signature_info>() as usize,
        );
        (*signature_info).type_ = VKD3D_SHADER_STRUCTURE_TYPE_SCAN_SIGNATURE_INFO;
        (*descriptor_info).next = signature_info as *const core::ffi::c_void;
    }

    info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO;
    info.next = descriptor_info as *const core::ffi::c_void;
    if (*shader).source_type as core::ffi::c_uint
        == VKD3D_SHADER_SOURCE_D3D_BYTECODE as core::ffi::c_int as core::ffi::c_uint
    {
        info.source.code = (*shader).function as *const core::ffi::c_void;
        info.source.size = (*shader).functionLength as usize;
    } else {
        info.source.code = (*shader).byte_code;
        info.source.size = (*shader).byte_code_size as usize;
    }
    info.source_type = (*shader).source_type;
    info.target_type = VKD3D_SHADER_TARGET_SPIRV_BINARY;
    info.options = &raw const spirv_compile_options as *const vkd3d_shader_compile_option;
    info.option_count = ARRAY_SIZE!(spirv_compile_options) as core::ffi::c_uint;
    info.log_level = VKD3D_SHADER_LOG_WARNING;
    info.source_name = core::ptr::null::<core::ffi::c_char>();

    ret = vkd3d_shader_scan(&raw mut info, &raw mut messages);
    if ret < 0 {
        ERR!("Failed to scan shader, ret %d.\n", ret);
    }
    if !messages.is_null() && *messages != 0 && FIXME_ON!(__wine_dbch_d3d_shader) != 0 {
        let mut ptr: *const core::ffi::c_char = core::ptr::null::<core::ffi::c_char>();
        let mut end: *const core::ffi::c_char = core::ptr::null::<core::ffi::c_char>();
        let mut line: *const core::ffi::c_char = core::ptr::null::<core::ffi::c_char>();

        ptr = messages;
        end = ptr.offset(strlen(ptr) as isize);
        loop {
            line = wined3d_get_line(&raw mut ptr, end);
            if line.is_null() {
                break;
            }
        }
    }
    vkd3d_shader_free_messages(messages);
}

unsafe extern "C" fn shader_spirv_precompile_compute(shader: *mut wined3d_shader) {
    let mut program_vk: *mut shader_spirv_compute_program_vk =
        core::ptr::null_mut::<shader_spirv_compute_program_vk>();
    program_vk = (*shader).backend_data as *mut shader_spirv_compute_program_vk;
    if program_vk.is_null() {
        program_vk = calloc(
            1,
            (core::mem::size_of::<shader_spirv_compute_program_vk>()) as _,
        ) as *mut shader_spirv_compute_program_vk;
        if program_vk.is_null() {
            ERR!("Failed to allocate program.\n");
        }
        (*shader).backend_data = program_vk as *mut core::ffi::c_void;
    }
    shader_spirv_scan_shader(
        shader,
        core::ptr::addr_of_mut!((*program_vk).descriptor_info),
        core::ptr::null_mut::<vkd3d_shader_scan_signature_info>(),
    );
}

unsafe extern "C" fn shader_spirv_precompile(shader_priv: *mut ::core::ffi::c_void, shader: *mut wined3d_shader) {
    let mut program_vk: *mut shader_spirv_graphics_program_vk = core::ptr::null_mut();

    TRACE!("shader_priv %p, shader %p.\n", shader_priv, shader);

    if (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint {
        shader_spirv_precompile_compute(shader);
        return;
    }

    program_vk = (*shader).backend_data as *mut shader_spirv_graphics_program_vk;
    if program_vk.is_null() {
        program_vk = calloc(
            (1 as usize) as _,
            (core::mem::size_of::<shader_spirv_graphics_program_vk>() as usize) as _,
        ) as *mut shader_spirv_graphics_program_vk;
        if program_vk.is_null() {
            ERR!("Failed to allocate program.\n");
        }
        (*shader).backend_data = program_vk as *mut ::core::ffi::c_void;
    }

    shader_spirv_scan_shader(
        shader,
        core::ptr::addr_of_mut!((*program_vk).descriptor_info),
        core::ptr::addr_of_mut!((*program_vk).signature_info),
    );
}

unsafe extern "C" fn shader_spirv_apply_draw_state(shader_priv: *mut core::ffi::c_void, context: *mut wined3d_context, state: *const wined3d_state) {
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut variant_vk: *mut shader_spirv_graphics_program_variant_vk = core::ptr::null_mut();
    let mut bindings: *mut shader_spirv_resource_bindings = core::ptr::null_mut();
    let mut binding_base: [size_t; (WINED3D_SHADER_TYPE_COUNT) as _] = [0; (WINED3D_SHADER_TYPE_COUNT) as _];
    let mut layout_vk: *mut wined3d_pipeline_layout_vk = core::ptr::null_mut();
    let priv_: *mut shader_spirv_priv = shader_priv as *mut shader_spirv_priv;
    let mut shader_type: wined3d_shader_type = 0;
    let mut shader: *mut wined3d_shader = core::ptr::null_mut();

    (*(*priv_).vertex_pipe).vp_apply_draw_state.unwrap_unchecked()(context, state);
    (*(*priv_).fragment_pipe).fp_apply_draw_state.unwrap_unchecked()(context, state);

    bindings = core::ptr::addr_of_mut!((*priv_).bindings);
    memcpy(
        core::ptr::addr_of_mut!(binding_base) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*bindings).binding_base) as *const core::ffi::c_void,
        core::mem::size_of::<[size_t; (WINED3D_SHADER_TYPE_COUNT) as _]>() as size_t,
    );

    'fail: {
        if !shader_spirv_resource_bindings_init(
            bindings,
            core::ptr::addr_of_mut!((*context_vk).graphics.bindings),
            state,
            !((1 as uint32_t) << (WINED3D_SHADER_TYPE_COMPUTE as u32)),
        ) {
            ERR!("Failed to initialise shader resource bindings.\n");
            break 'fail;
        }
        if ((*context).shader_update_mask() & ((1 as DWORD) << (WINED3D_SHADER_TYPE_GEOMETRY as u32))) != 0 {
            (*context).set_shader_update_mask(((*context).shader_update_mask() | ((1 as DWORD) << ((*bindings).so_stage as u32))) as _);
        }

        layout_vk = wined3d_context_vk_get_pipeline_layout(context_vk, (*bindings).vk_bindings, ((*bindings).vk_binding_count) as SIZE_T);
        (*context_vk).graphics.vk_set_layout = (*layout_vk).vk_set_layout;
        (*context_vk).graphics.vk_pipeline_layout = (*layout_vk).vk_pipeline_layout;

        while (shader_type as usize) < (ARRAY_SIZE!((*context_vk).graphics.vk_modules) as usize) {
            'body: {
                let skip = ((*context).shader_update_mask() & ((1 as DWORD) << (shader_type as u32))) == 0
                    && ((*context_vk).graphics.vk_modules[shader_type as usize] == 0
                        || binding_base[shader_type as usize] == (*bindings).binding_base[shader_type as usize]);
                if skip {
                    break 'body;
                }

                shader = (*state).shader[shader_type as usize];
                if shader.is_null() || (*shader).function.is_null() {
                    (*context_vk).graphics.vk_modules[shader_type as usize] = (VK_NULL_HANDLE as _);
                    break 'body;
                }

                variant_vk = shader_spirv_find_graphics_program_variant_vk(priv_, context_vk, shader, state, bindings);
                if variant_vk.is_null() {
                    break 'fail;
                }
                (*context_vk).graphics.vk_modules[shader_type as usize] = (*variant_vk).vk_module;
            }
            shader_type += 1;
        }

        return;
    }

    (*context_vk).graphics.vk_set_layout = (VK_NULL_HANDLE as _);
    (*context_vk).graphics.vk_pipeline_layout = (VK_NULL_HANDLE as _);
}
unsafe extern "C" fn shader_spirv_apply_compute_state(
    shader_priv: *mut core::ffi::c_void,
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut program: *mut shader_spirv_compute_program_vk;
    let priv_: *mut shader_spirv_priv = shader_priv as *mut shader_spirv_priv;
    let mut shader: *mut wined3d_shader;

    if !shader_spirv_resource_bindings_init(
        core::ptr::addr_of_mut!((*priv_).bindings),
        core::ptr::addr_of_mut!((*context_vk).compute.bindings),
        state,
        1 << WINED3D_SHADER_TYPE_COMPUTE,
    ) {
        ERR!("Failed to initialise shader resource bindings.\n");
    }

    shader = (*state).shader[WINED3D_SHADER_TYPE_COMPUTE as usize];
    if !shader.is_null() {
        program = shader_spirv_find_compute_program_vk(
            priv_,
            context_vk,
            shader,
            core::ptr::addr_of_mut!((*priv_).bindings),
        );
    } else {
        program = core::ptr::null_mut();
    }

    if !program.is_null() {
        (*context_vk).compute.vk_pipeline = (*program).vk_pipeline;
        (*context_vk).compute.vk_set_layout = (*program).vk_set_layout;
        (*context_vk).compute.vk_pipeline_layout = (*program).vk_pipeline_layout;
    } else {
        (*context_vk).compute.vk_pipeline = VK_NULL_HANDLE as VkPipeline;
        (*context_vk).compute.vk_set_layout = VK_NULL_HANDLE as VkDescriptorSetLayout;
        (*context_vk).compute.vk_pipeline_layout = VK_NULL_HANDLE as VkPipelineLayout;
    }
}

unsafe extern "C" fn shader_spirv_disable(shader_priv: *mut core::ffi::c_void, context: *mut wined3d_context) {
    let context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let priv_: *mut shader_spirv_priv = shader_priv as *mut shader_spirv_priv;

    ((*(*priv_).vertex_pipe).vp_disable).unwrap_unchecked()(context);
    ((*(*priv_).fragment_pipe).fp_disable).unwrap_unchecked()(context);

    (*context_vk).compute.vk_pipeline = (VK_NULL_HANDLE as _);
    (*context).set_shader_update_mask((((1 << WINED3D_SHADER_TYPE_PIXEL)
        | (1 << WINED3D_SHADER_TYPE_VERTEX)
        | (1 << WINED3D_SHADER_TYPE_GEOMETRY)
        | (1 << WINED3D_SHADER_TYPE_HULL)
        | (1 << WINED3D_SHADER_TYPE_DOMAIN)
        | (1 << WINED3D_SHADER_TYPE_COMPUTE)) as u32) as _);
}

unsafe extern "C" fn shader_spirv_update_float_vertex_constants(
    device: *mut wined3d_device,
    start: UINT,
    count: UINT,
) {
    WARN!("Not implemented.\n");
}

unsafe extern "C" fn shader_spirv_update_float_pixel_constants(
    device: *mut wined3d_device,
    start: UINT,
    count: UINT,
) {
    WARN!("Not implemented.\n");
}

unsafe extern "C" fn shader_spirv_invalidate_compute_program(
    context_vk: *mut wined3d_context_vk,
    program: *const shader_spirv_compute_program_vk,
) {
    if (*context_vk).compute.vk_pipeline == (*program).vk_pipeline {
        (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << WINED3D_SHADER_TYPE_COMPUTE))) as _);
        (*context_vk).compute.vk_pipeline = (VK_NULL_HANDLE as _);
    }
}

unsafe extern "C" fn shader_spirv_invalidate_contexts_compute_program(device: *mut wined3d_device, program: *const shader_spirv_compute_program_vk) {
    let mut i: u32 = 0;
    while i < (*device).context_count {
        shader_spirv_invalidate_compute_program(wined3d_context_vk(*(*device).contexts.offset(i as isize)), program);
        i += 1;
    }
}

unsafe extern "C" fn shader_spirv_invalidate_graphics_program_variant(
    mut context_vk: *mut wined3d_context_vk,
    mut variant: *const shader_spirv_graphics_program_variant_vk,
) {
    let mut shader_type: wined3d_shader_type = 0;
    shader_type = 0;
    while (shader_type as ::core::ffi::c_uint)
        < WINED3D_SHADER_TYPE_GRAPHICS_COUNT as ::core::ffi::c_uint
    {
        if !((*context_vk).graphics.vk_modules[shader_type as usize] != (*variant).vk_module) {
            (*context_vk).graphics.vk_modules[shader_type as usize] =
                VK_NULL_HANDLE as VkShaderModule;
            (*context_vk).c.set_shader_update_mask(((*context_vk).c.shader_update_mask() | ((1 << shader_type as ::core::ffi::c_uint) as DWORD)) as _);
        }
        shader_type += 1;
    }
}

unsafe extern "C" fn shader_spirv_invalidate_contexts_graphics_program_variant(
    mut device: *mut wined3d_device,
    mut variant: *const shader_spirv_graphics_program_variant_vk,
) {
    let mut i: ::core::ffi::c_uint;
    i = 0;
    while i < (*device).context_count {
        shader_spirv_invalidate_graphics_program_variant(
            wined3d_context_vk(*(*device).contexts.offset(i as isize)),
            variant,
        );
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn shader_spirv_destroy_compute_vk(shader: *mut wined3d_shader) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*shader).device);
    let program: *mut shader_spirv_compute_program_vk =
        (*shader).backend_data as *mut shader_spirv_compute_program_vk;
    let context_vk: *mut wined3d_context_vk = &raw mut (*device_vk).context_vk;
    let vk_info: *mut wined3d_vk_info = &raw mut (*device_vk).vk_info;

    shader_spirv_invalidate_contexts_compute_program(&raw mut (*device_vk).d, program);
    wined3d_context_vk_destroy_vk_pipeline(context_vk, (*program).vk_pipeline, (*context_vk).current_command_buffer.id);
    (*vk_info)
        .vk_ops
        .vkDestroyShaderModule
        .unwrap_unchecked()(
        (*device_vk).vk_device,
        (*program).vk_module,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    vkd3d_shader_free_scan_descriptor_info(&raw mut (*program).descriptor_info);
    (*shader).backend_data = core::ptr::null_mut();
    free(program as *mut core::ffi::c_void);
}

unsafe extern "C" fn shader_spirv_destroy(shader: *mut wined3d_shader) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*shader).device);
    let mut variant_vk: *mut shader_spirv_graphics_program_variant_vk = core::ptr::null_mut();
    let mut vk_info: *mut wined3d_vk_info = core::ptr::addr_of_mut!((*device_vk).vk_info);
    let mut program_vk: *mut shader_spirv_graphics_program_vk = core::ptr::null_mut();
    let mut i: size_t = 0;

    if (*shader).backend_data.is_null() {
        return;
    }

    if ((*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint)
        == (WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint)
    {
        shader_spirv_destroy_compute_vk(shader);
        return;
    }

    program_vk = (*shader).backend_data as *mut shader_spirv_graphics_program_vk;
    i = 0 as size_t;
    while i < (*program_vk).variant_count {
        variant_vk = (*program_vk).variants.offset(i as isize) as *mut shader_spirv_graphics_program_variant_vk;
        shader_spirv_invalidate_contexts_graphics_program_variant(core::ptr::addr_of_mut!((*device_vk).d), variant_vk);
        (*vk_info)
            .vk_ops
            .vkDestroyShaderModule
            .unwrap_unchecked()(
            (*device_vk).vk_device,
            (*variant_vk).vk_module,
            core::ptr::null::<VkAllocationCallbacks>(),
        );
        i = i.wrapping_add(1);
    }
    free((*program_vk).variants as *mut ::core::ffi::c_void);
    vkd3d_shader_free_scan_descriptor_info(core::ptr::addr_of_mut!((*program_vk).descriptor_info));
    vkd3d_shader_free_scan_signature_info(core::ptr::addr_of_mut!((*program_vk).signature_info));

    (*shader).backend_data = core::ptr::null_mut();
    free(program_vk as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn shader_spirv_alloc(
    device: *mut wined3d_device,
    vertex_pipe: *const wined3d_vertex_pipe_ops,
    fragment_pipe: *const wined3d_fragment_pipe_ops,
) -> HRESULT {
    let mut vertex_priv: *mut core::ffi::c_void;
    let mut fragment_priv: *mut core::ffi::c_void;
    let mut priv_: *mut shader_spirv_priv;

    priv_ = malloc(core::mem::size_of::<shader_spirv_priv>()) as *mut shader_spirv_priv;
    if priv_.is_null() {
        return E_OUTOFMEMORY;
    }

    vertex_priv = ((*vertex_pipe).vp_alloc.unwrap_unchecked())(
        core::ptr::addr_of!(spirv_shader_backend_vk),
        priv_ as *mut core::ffi::c_void,
    );
    if vertex_priv.is_null() {
        ERR!("Failed to initialise vertex pipe.\n");
        free(priv_ as *mut core::ffi::c_void);
        return E_FAIL;
    }

    fragment_priv = ((*fragment_pipe).alloc_private.unwrap_unchecked())(
        core::ptr::addr_of!(spirv_shader_backend_vk),
        priv_ as *mut core::ffi::c_void,
    );
    if fragment_priv.is_null() {
        ERR!("Failed to initialise fragment pipe.\n");
        ((*vertex_pipe).vp_free.unwrap_unchecked())(
            device,
            core::ptr::null_mut(),
        );
        free(priv_ as *mut core::ffi::c_void);
        return E_FAIL;
    }

    (*priv_).vertex_pipe = vertex_pipe;
    (*priv_).fragment_pipe = fragment_pipe;
    memset(
        core::ptr::addr_of_mut!((*priv_).bindings) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<shader_spirv_resource_bindings>(),
    );

    (*device).vertex_priv = vertex_priv;
    (*device).fragment_priv = fragment_priv;
    (*device).shader_priv = priv_ as *mut core::ffi::c_void;

    return WINED3D_OK;
}

unsafe extern "C" fn shader_spirv_free(device: *mut wined3d_device, context: *mut wined3d_context) {
    let priv_0: *mut shader_spirv_priv = (*device).shader_priv as *mut shader_spirv_priv;
    shader_spirv_resource_bindings_cleanup(core::ptr::addr_of_mut!((*priv_0).bindings));
    ((*(*priv_0).fragment_pipe).free_private.unwrap_unchecked())(device, context);
    ((*(*priv_0).vertex_pipe).vp_free.unwrap_unchecked())(device, context);
    free(priv_0 as *mut core::ffi::c_void);
}

unsafe extern "C" fn shader_spirv_allocate_context_data(context: *mut wined3d_context) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn shader_spirv_free_context_data(context: *mut wined3d_context) {}

unsafe extern "C" fn shader_spirv_init_context_state(context: *mut wined3d_context) {}

unsafe extern "C" fn shader_spirv_get_caps(adapter: *const wined3d_adapter, caps: *mut shader_caps) {
    memset(caps as *mut core::ffi::c_void, 0, core::mem::size_of::<shader_caps>());

    (*caps).vs_version = min!(wined3d_settings.max_sm_vs, 5);
    (*caps).hs_version = min!(wined3d_settings.max_sm_hs, 5);
    (*caps).ds_version = min!(wined3d_settings.max_sm_ds, 5);
    (*caps).gs_version = min!(wined3d_settings.max_sm_gs, 5);
    (*caps).ps_version = min!(wined3d_settings.max_sm_ps, 5);
    (*caps).cs_version = min!(wined3d_settings.max_sm_cs, 5);

    (*caps).vs_uniform_count = WINED3D_MAX_VS_CONSTS_F;
    (*caps).ps_uniform_count = WINED3D_MAX_PS_CONSTS_F;
    (*caps).ps_1x_max_value = FLT_MAX;
    (*caps).varying_count = 0;
    (*caps).wined3d_caps = WINED3D_SHADER_CAP_FULL_FFP_VARYINGS;
}

unsafe extern "C" fn shader_spirv_color_fixup_supported(fixup: color_fixup_desc) -> BOOL {
    is_identity_fixup(fixup)
}

unsafe extern "C" fn shader_spirv_compile(
    context: *mut wined3d_context,
    shader_desc: *const wined3d_shader_desc,
    shader_type: wined3d_shader_type,
) -> uint64_t {
    let mut bindings: shader_spirv_resource_bindings = core::mem::zeroed();
    return shader_spirv_compile_shader(
        wined3d_context_vk(context),
        shader_desc,
        VKD3D_SHADER_SOURCE_DXBC_TPF,
        shader_type,
        core::ptr::null(),
        &mut bindings,
        core::ptr::null(),
    ) as uint64_t;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_spirv_shader_backend_init_vk() -> *const wined3d_shader_backend_ops {
    WINE_TRACE!("Using %s.\n", vkd3d_shader_get_version( core::ptr::null_mut::<core::ffi::c_uint>(), core::ptr::null_mut::<core::ffi::c_uint>(), ));
    return core::ptr::addr_of!(spirv_shader_backend_vk);
}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_apply_draw_state(
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_disable(context: *const wined3d_context) {}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_get_caps(
    mut adapter: *const wined3d_adapter,
    mut caps: *mut wined3d_vertex_caps,
) {
    memset(
        caps as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<wined3d_vertex_caps>() as size_t,
    );
    (*caps).emulated_flatshading = true as BOOL;
}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_get_emul_mask(
    adapter: *const wined3d_adapter,
) -> u32 {
    0
}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_alloc(
    shader_backend: *const wined3d_shader_backend_ops,
    shader_priv: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    if shader_backend != core::ptr::addr_of!(spirv_shader_backend_vk) {
        FIXME!("SPIR-V vertex pipe without SPIR-V shader backend not implemented.\n");
        return core::ptr::null_mut();
    }

    return shader_priv;
}

unsafe extern "C" fn spirv_vertex_pipe_vk_vp_free(
    device: *mut wined3d_device,
    context: *mut wined3d_context,
) {}
#[no_mangle]
pub unsafe extern "C" fn wined3d_spirv_vertex_pipe_init_vk() -> *const wined3d_vertex_pipe_ops {
    return core::ptr::addr_of!(spirv_vertex_pipe_vk);
}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_apply_draw_state(
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_disable(context: *const wined3d_context) {}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_get_caps(
    adapter: *const wined3d_adapter,
    caps: *mut fragment_caps,
) {
    memset(
        caps as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<fragment_caps>() as usize,
    );
    (*caps).max_blend_stages = WINED3D_MAX_FFP_TEXTURES as core::ffi::c_uint;
}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_get_emul_mask(
    adapter: *const wined3d_adapter,
) -> u32 {
    0
}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_alloc(
    shader_backend: *const wined3d_shader_backend_ops,
    shader_priv: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    if shader_backend != &raw const spirv_shader_backend_vk {
        FIXME!("SPIR-V fragment pipe without SPIR-V shader backend not implemented.\n");
        return core::ptr::null_mut();
    }

    return shader_priv;
}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_free(
    device: *mut wined3d_device,
    context: *mut wined3d_context,
) {}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_alloc_context_data(
    context: *mut wined3d_context,
) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn spirv_fragment_pipe_vk_fp_free_context_data(
    context: *mut wined3d_context,
) {}

#[no_mangle]
pub unsafe extern "C" fn wined3d_spirv_fragment_pipe_init_vk() -> *const wined3d_fragment_pipe_ops {
    return core::ptr::addr_of!(spirv_fragment_pipe_vk);
}
