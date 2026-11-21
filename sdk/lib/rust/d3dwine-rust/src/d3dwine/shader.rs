use crate::d3dwine::utils::debug_shader_type;
use crate::d3dwine::ffp_hlsl::ffp_hlsl_compile_ps;
use crate::d3dwine::ffp_hlsl::ffp_hlsl_compile_vs;
use crate::d3dwine::utils::wined3d_array_reserve;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_init_object;
use crate::d3dwine::wined3d_main::wined3d_mutex_lock;
use crate::d3dwine::wined3d_main::wined3d_mutex_unlock;
use crate::d3dwine::shader_sm4::wined3d_shader_extract_from_dxbc;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

#[no_mangle]
pub static mut wined3d_srgb_const: [wined3d_vec4; 2] = [
    wined3d_vec4 {
        x: 0.41666f32,
        y: 1.055f32,
        z: 0.055f32,
        w: 12.92f32,
    },
    wined3d_vec4 {
        x: 0.0031308f32,
        y: 0.0f32,
        z: 0.0f32,
        w: 0.0f32,
    },
];

static mut shader_opcode_names: [*const ::core::ffi::c_char; 239] = [
    b"abs\0" as *const u8 as *const ::core::ffi::c_char,
    b"add\0" as *const u8 as *const ::core::ffi::c_char,
    b"and\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_and\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_cmp_store\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_iadd\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_imax\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_imin\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_or\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_umax\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_umin\0" as *const u8 as *const ::core::ffi::c_char,
    b"atomic_xor\0" as *const u8 as *const ::core::ffi::c_char,
    b"bem\0" as *const u8 as *const ::core::ffi::c_char,
    b"bfi\0" as *const u8 as *const ::core::ffi::c_char,
    b"bfrev\0" as *const u8 as *const ::core::ffi::c_char,
    b"break\0" as *const u8 as *const ::core::ffi::c_char,
    b"breakc\0" as *const u8 as *const ::core::ffi::c_char,
    b"breakp\0" as *const u8 as *const ::core::ffi::c_char,
    b"bufinfo\0" as *const u8 as *const ::core::ffi::c_char,
    b"call\0" as *const u8 as *const ::core::ffi::c_char,
    b"callnz\0" as *const u8 as *const ::core::ffi::c_char,
    b"case\0" as *const u8 as *const ::core::ffi::c_char,
    b"cmp\0" as *const u8 as *const ::core::ffi::c_char,
    b"cnd\0" as *const u8 as *const ::core::ffi::c_char,
    b"continue\0" as *const u8 as *const ::core::ffi::c_char,
    b"continuec\0" as *const u8 as *const ::core::ffi::c_char,
    b"countbits\0" as *const u8 as *const ::core::ffi::c_char,
    b"crs\0" as *const u8 as *const ::core::ffi::c_char,
    b"cut\0" as *const u8 as *const ::core::ffi::c_char,
    b"cut_stream\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_constantBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_function_body\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_function_table\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_globalFlags\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_gs_instances\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_hs_fork_phase_instance_count\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_hs_join_phase_instance_count\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_hs_max_tessfactor\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_immediateConstantBuffer\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_index_range\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_indexableTemp\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_control_point_count\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_inputPrimitive\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_ps\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_ps_sgv\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_ps_siv\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_sgv\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_input_siv\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_interface\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_output\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_output_control_point_count\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_output_siv\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_outputTopology\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_resource_raw\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_resource_structured\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_sampler\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_stream\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_temps\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_tessellator_domain\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_tessellator_output_primitive\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_tessellator_partitioning\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_tgsm_raw\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_tgsm_structured\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_thread_group\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_uav_raw\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_uav_structured\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_uav_typed\0" as *const u8 as *const ::core::ffi::c_char,
    b"dcl_maxOutputVertexCount\0" as *const u8 as *const ::core::ffi::c_char,
    b"def\0" as *const u8 as *const ::core::ffi::c_char,
    b"default\0" as *const u8 as *const ::core::ffi::c_char,
    b"defb\0" as *const u8 as *const ::core::ffi::c_char,
    b"defi\0" as *const u8 as *const ::core::ffi::c_char,
    b"div\0" as *const u8 as *const ::core::ffi::c_char,
    b"dp2\0" as *const u8 as *const ::core::ffi::c_char,
    b"dp2add\0" as *const u8 as *const ::core::ffi::c_char,
    b"dp3\0" as *const u8 as *const ::core::ffi::c_char,
    b"dp4\0" as *const u8 as *const ::core::ffi::c_char,
    b"dst\0" as *const u8 as *const ::core::ffi::c_char,
    b"dsx\0" as *const u8 as *const ::core::ffi::c_char,
    b"deriv_rtx_coarse\0" as *const u8 as *const ::core::ffi::c_char,
    b"deriv_rtx_fine\0" as *const u8 as *const ::core::ffi::c_char,
    b"dsy\0" as *const u8 as *const ::core::ffi::c_char,
    b"deriv_rty_coarse\0" as *const u8 as *const ::core::ffi::c_char,
    b"deriv_rty_fine\0" as *const u8 as *const ::core::ffi::c_char,
    b"else\0" as *const u8 as *const ::core::ffi::c_char,
    b"emit\0" as *const u8 as *const ::core::ffi::c_char,
    b"emit_stream\0" as *const u8 as *const ::core::ffi::c_char,
    b"endif\0" as *const u8 as *const ::core::ffi::c_char,
    b"endloop\0" as *const u8 as *const ::core::ffi::c_char,
    b"endrep\0" as *const u8 as *const ::core::ffi::c_char,
    b"endswitch\0" as *const u8 as *const ::core::ffi::c_char,
    b"eq\0" as *const u8 as *const ::core::ffi::c_char,
    b"eval_centroid\0" as *const u8 as *const ::core::ffi::c_char,
    b"eval_sample_index\0" as *const u8 as *const ::core::ffi::c_char,
    b"exp\0" as *const u8 as *const ::core::ffi::c_char,
    b"expp\0" as *const u8 as *const ::core::ffi::c_char,
    b"f16tof32\0" as *const u8 as *const ::core::ffi::c_char,
    b"f32tof16\0" as *const u8 as *const ::core::ffi::c_char,
    b"fcall\0" as *const u8 as *const ::core::ffi::c_char,
    b"firstbit_hi\0" as *const u8 as *const ::core::ffi::c_char,
    b"firstbit_lo\0" as *const u8 as *const ::core::ffi::c_char,
    b"firstbit_shi\0" as *const u8 as *const ::core::ffi::c_char,
    b"frc\0" as *const u8 as *const ::core::ffi::c_char,
    b"ftoi\0" as *const u8 as *const ::core::ffi::c_char,
    b"ftou\0" as *const u8 as *const ::core::ffi::c_char,
    b"gather4\0" as *const u8 as *const ::core::ffi::c_char,
    b"gather4_c\0" as *const u8 as *const ::core::ffi::c_char,
    b"gather4_po\0" as *const u8 as *const ::core::ffi::c_char,
    b"gather4_po_c\0" as *const u8 as *const ::core::ffi::c_char,
    b"ge\0" as *const u8 as *const ::core::ffi::c_char,
    b"hs_control_point_phase\0" as *const u8 as *const ::core::ffi::c_char,
    b"hs_decls\0" as *const u8 as *const ::core::ffi::c_char,
    b"hs_fork_phase\0" as *const u8 as *const ::core::ffi::c_char,
    b"hs_join_phase\0" as *const u8 as *const ::core::ffi::c_char,
    b"iadd\0" as *const u8 as *const ::core::ffi::c_char,
    b"ibfe\0" as *const u8 as *const ::core::ffi::c_char,
    b"ieq\0" as *const u8 as *const ::core::ffi::c_char,
    b"if\0" as *const u8 as *const ::core::ffi::c_char,
    b"ifc\0" as *const u8 as *const ::core::ffi::c_char,
    b"ige\0" as *const u8 as *const ::core::ffi::c_char,
    b"ilt\0" as *const u8 as *const ::core::ffi::c_char,
    b"imad\0" as *const u8 as *const ::core::ffi::c_char,
    b"imax\0" as *const u8 as *const ::core::ffi::c_char,
    b"imin\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_alloc\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_and\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_cmp_exch\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_consume\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_exch\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_iadd\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_imax\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_imin\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_or\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_umax\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_umin\0" as *const u8 as *const ::core::ffi::c_char,
    b"imm_atomic_xor\0" as *const u8 as *const ::core::ffi::c_char,
    b"imul\0" as *const u8 as *const ::core::ffi::c_char,
    b"ine\0" as *const u8 as *const ::core::ffi::c_char,
    b"ineg\0" as *const u8 as *const ::core::ffi::c_char,
    b"ishl\0" as *const u8 as *const ::core::ffi::c_char,
    b"ishr\0" as *const u8 as *const ::core::ffi::c_char,
    b"itof\0" as *const u8 as *const ::core::ffi::c_char,
    b"label\0" as *const u8 as *const ::core::ffi::c_char,
    b"ld\0" as *const u8 as *const ::core::ffi::c_char,
    b"ld2dms\0" as *const u8 as *const ::core::ffi::c_char,
    b"ld_raw\0" as *const u8 as *const ::core::ffi::c_char,
    b"ld_structured\0" as *const u8 as *const ::core::ffi::c_char,
    b"ld_uav_typed\0" as *const u8 as *const ::core::ffi::c_char,
    b"lit\0" as *const u8 as *const ::core::ffi::c_char,
    b"lod\0" as *const u8 as *const ::core::ffi::c_char,
    b"log\0" as *const u8 as *const ::core::ffi::c_char,
    b"logp\0" as *const u8 as *const ::core::ffi::c_char,
    b"loop\0" as *const u8 as *const ::core::ffi::c_char,
    b"lrp\0" as *const u8 as *const ::core::ffi::c_char,
    b"lt\0" as *const u8 as *const ::core::ffi::c_char,
    b"m3x2\0" as *const u8 as *const ::core::ffi::c_char,
    b"m3x3\0" as *const u8 as *const ::core::ffi::c_char,
    b"m3x4\0" as *const u8 as *const ::core::ffi::c_char,
    b"m4x3\0" as *const u8 as *const ::core::ffi::c_char,
    b"m4x4\0" as *const u8 as *const ::core::ffi::c_char,
    b"mad\0" as *const u8 as *const ::core::ffi::c_char,
    b"max\0" as *const u8 as *const ::core::ffi::c_char,
    b"min\0" as *const u8 as *const ::core::ffi::c_char,
    b"mov\0" as *const u8 as *const ::core::ffi::c_char,
    b"mova\0" as *const u8 as *const ::core::ffi::c_char,
    b"movc\0" as *const u8 as *const ::core::ffi::c_char,
    b"mul\0" as *const u8 as *const ::core::ffi::c_char,
    b"ne\0" as *const u8 as *const ::core::ffi::c_char,
    b"nop\0" as *const u8 as *const ::core::ffi::c_char,
    b"not\0" as *const u8 as *const ::core::ffi::c_char,
    b"nrm\0" as *const u8 as *const ::core::ffi::c_char,
    b"or\0" as *const u8 as *const ::core::ffi::c_char,
    b"phase\0" as *const u8 as *const ::core::ffi::c_char,
    b"pow\0" as *const u8 as *const ::core::ffi::c_char,
    b"rcp\0" as *const u8 as *const ::core::ffi::c_char,
    b"rep\0" as *const u8 as *const ::core::ffi::c_char,
    b"resinfo\0" as *const u8 as *const ::core::ffi::c_char,
    b"ret\0" as *const u8 as *const ::core::ffi::c_char,
    b"retp\0" as *const u8 as *const ::core::ffi::c_char,
    b"round_ne\0" as *const u8 as *const ::core::ffi::c_char,
    b"round_ni\0" as *const u8 as *const ::core::ffi::c_char,
    b"round_pi\0" as *const u8 as *const ::core::ffi::c_char,
    b"round_z\0" as *const u8 as *const ::core::ffi::c_char,
    b"rsq\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_b\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_c\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_c_lz\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_d\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_info\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_l\0" as *const u8 as *const ::core::ffi::c_char,
    b"sample_pos\0" as *const u8 as *const ::core::ffi::c_char,
    b"setp\0" as *const u8 as *const ::core::ffi::c_char,
    b"sge\0" as *const u8 as *const ::core::ffi::c_char,
    b"sgn\0" as *const u8 as *const ::core::ffi::c_char,
    b"sincos\0" as *const u8 as *const ::core::ffi::c_char,
    b"slt\0" as *const u8 as *const ::core::ffi::c_char,
    b"sqrt\0" as *const u8 as *const ::core::ffi::c_char,
    b"store_raw\0" as *const u8 as *const ::core::ffi::c_char,
    b"store_structured\0" as *const u8 as *const ::core::ffi::c_char,
    b"store_uav_typed\0" as *const u8 as *const ::core::ffi::c_char,
    b"sub\0" as *const u8 as *const ::core::ffi::c_char,
    b"swapc\0" as *const u8 as *const ::core::ffi::c_char,
    b"switch\0" as *const u8 as *const ::core::ffi::c_char,
    b"sync\0" as *const u8 as *const ::core::ffi::c_char,
    b"texld\0" as *const u8 as *const ::core::ffi::c_char,
    b"texbem\0" as *const u8 as *const ::core::ffi::c_char,
    b"texbeml\0" as *const u8 as *const ::core::ffi::c_char,
    b"texcrd\0" as *const u8 as *const ::core::ffi::c_char,
    b"texdepth\0" as *const u8 as *const ::core::ffi::c_char,
    b"texdp3\0" as *const u8 as *const ::core::ffi::c_char,
    b"texdp3tex\0" as *const u8 as *const ::core::ffi::c_char,
    b"texkill\0" as *const u8 as *const ::core::ffi::c_char,
    b"texldd\0" as *const u8 as *const ::core::ffi::c_char,
    b"texldl\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x2depth\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x2pad\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x2tex\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3diff\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3pad\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3spec\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3tex\0" as *const u8 as *const ::core::ffi::c_char,
    b"texm3x3vspec\0" as *const u8 as *const ::core::ffi::c_char,
    b"texreg2ar\0" as *const u8 as *const ::core::ffi::c_char,
    b"texreg2gb\0" as *const u8 as *const ::core::ffi::c_char,
    b"texreg2rgb\0" as *const u8 as *const ::core::ffi::c_char,
    b"ubfe\0" as *const u8 as *const ::core::ffi::c_char,
    b"udiv\0" as *const u8 as *const ::core::ffi::c_char,
    b"uge\0" as *const u8 as *const ::core::ffi::c_char,
    b"ult\0" as *const u8 as *const ::core::ffi::c_char,
    b"umax\0" as *const u8 as *const ::core::ffi::c_char,
    b"umin\0" as *const u8 as *const ::core::ffi::c_char,
    b"umul\0" as *const u8 as *const ::core::ffi::c_char,
    b"ushr\0" as *const u8 as *const ::core::ffi::c_char,
    b"utof\0" as *const u8 as *const ::core::ffi::c_char,
    b"xor\0" as *const u8 as *const ::core::ffi::c_char,
];

static mut semantic_names: [*const ::core::ffi::c_char; 14] = [
    b"SV_POSITION\0" as *const u8 as *const ::core::ffi::c_char,
    b"BLENDWEIGHT\0" as *const u8 as *const ::core::ffi::c_char,
    b"BLENDINDICES\0" as *const u8 as *const ::core::ffi::c_char,
    b"NORMAL\0" as *const u8 as *const ::core::ffi::c_char,
    b"PSIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"TEXCOORD\0" as *const u8 as *const ::core::ffi::c_char,
    b"TANGENT\0" as *const u8 as *const ::core::ffi::c_char,
    b"BINORMAL\0" as *const u8 as *const ::core::ffi::c_char,
    b"TESSFACTOR\0" as *const u8 as *const ::core::ffi::c_char,
    b"POSITIONT\0" as *const u8 as *const ::core::ffi::c_char,
    b"COLOR\0" as *const u8 as *const ::core::ffi::c_char,
    b"FOG\0" as *const u8 as *const ::core::ffi::c_char,
    b"DEPTH\0" as *const u8 as *const ::core::ffi::c_char,
    b"SAMPLE\0" as *const u8 as *const ::core::ffi::c_char,
];

#[no_mangle]
pub static mut none_shader_backend: wined3d_shader_backend_ops = unsafe {
    wined3d_shader_backend_ops {
        shader_handle_instruction: Some(
            shader_none_handle_instruction
                as unsafe extern "C" fn(*const wined3d_shader_instruction) -> (),
        ),
        shader_precompile: Some(
            shader_none_precompile
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut wined3d_shader) -> (),
        ),
        shader_apply_draw_state: Some(
            shader_none_apply_draw_state
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut wined3d_context,
                    *const wined3d_state,
                ) -> (),
        ),
        shader_apply_compute_state: Some(
            shader_none_apply_compute_state
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut wined3d_context,
                    *const wined3d_state,
                ) -> (),
        ),
        shader_disable: Some(
            shader_none_disable
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut wined3d_context) -> (),
        ),
        shader_update_float_vertex_constants: Some(
            shader_none_update_float_vertex_constants
                as unsafe extern "C" fn(*mut wined3d_device, UINT, UINT) -> (),
        ),
        shader_update_float_pixel_constants: Some(
            shader_none_update_float_pixel_constants
                as unsafe extern "C" fn(*mut wined3d_device, UINT, UINT) -> (),
        ),
        shader_destroy: Some(
            shader_none_destroy as unsafe extern "C" fn(*mut wined3d_shader) -> (),
        ),
        shader_alloc_private: Some(
            shader_none_alloc
                as unsafe extern "C" fn(
                    *mut wined3d_device,
                    *const wined3d_vertex_pipe_ops,
                    *const wined3d_fragment_pipe_ops,
                ) -> HRESULT,
        ),
        shader_free_private: Some(
            shader_none_free
                as unsafe extern "C" fn(*mut wined3d_device, *mut wined3d_context) -> (),
        ),
        shader_allocate_context_data: Some(
            shader_none_allocate_context_data as unsafe extern "C" fn(*mut wined3d_context) -> BOOL,
        ),
        shader_free_context_data: Some(
            shader_none_free_context_data as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        shader_init_context_state: Some(
            shader_none_init_context_state as unsafe extern "C" fn(*mut wined3d_context) -> (),
        ),
        shader_get_caps: Some(
            shader_none_get_caps
                as unsafe extern "C" fn(*const wined3d_adapter, *mut shader_caps) -> (),
        ),
        shader_color_fixup_supported: Some(
            shader_none_color_fixup_supported as unsafe extern "C" fn(color_fixup_desc) -> BOOL,
        ),
        shader_compile: Some(
            shader_none_shader_compile
                as unsafe extern "C" fn(
                    *mut wined3d_context,
                    *const wined3d_shader_desc,
                    wined3d_shader_type,
                ) -> uint64_t,
        ),
    }
};

#[repr(C)]
pub struct shader_none_priv {
    pub vertex_pipe: *const wined3d_vertex_pipe_ops,
    pub fragment_pipe: *const wined3d_fragment_pipe_ops,
}
#[no_mangle]
pub unsafe extern "C" fn debug_d3dshaderinstructionhandler(handler_idx: WINED3D_SHADER_INSTRUCTION_HANDLER) -> *const ::core::ffi::c_char {
    if handler_idx as usize >= ARRAY_SIZE!(shader_opcode_names) as usize {
        return wine_dbg_sprintf(
            b"UNRECOGNIZED(%#x)\0".as_ptr() as *const ::core::ffi::c_char,
            handler_idx as ::core::ffi::c_uint,
        );
    }

    return shader_opcode_names[handler_idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn vkd3d_shader_visibility_from_wined3d(shader_type: wined3d_shader_type) -> vkd3d_shader_visibility {
    match shader_type {
        WINED3D_SHADER_TYPE_VERTEX => {
            return VKD3D_SHADER_VISIBILITY_VERTEX;
        }
        WINED3D_SHADER_TYPE_HULL => {
            return VKD3D_SHADER_VISIBILITY_HULL;
        }
        WINED3D_SHADER_TYPE_DOMAIN => {
            return VKD3D_SHADER_VISIBILITY_DOMAIN;
        }
        WINED3D_SHADER_TYPE_GEOMETRY => {
            return VKD3D_SHADER_VISIBILITY_GEOMETRY;
        }
        WINED3D_SHADER_TYPE_PIXEL => {
            return VKD3D_SHADER_VISIBILITY_PIXEL;
        }
        WINED3D_SHADER_TYPE_COMPUTE => {
            return VKD3D_SHADER_VISIBILITY_COMPUTE;
        }
        _ => {
            ERR!("Invalid shader type %s.\n", debug_shader_type(shader_type));
            return VKD3D_SHADER_VISIBILITY_ALL;
        }
    }
}

unsafe extern "C" fn shader_semantic_name_from_usage(usage: wined3d_decl_usage) -> *const core::ffi::c_char {
    if (usage as usize) >= ARRAY_SIZE!(semantic_names) as usize {
        WINE_FIXME!("Unrecognized usage %#x.\n", usage as core::ffi::c_uint);
        return b"UNRECOGNIZED\0" as *const u8 as *const core::ffi::c_char;
    }

    return semantic_names[usage as usize];
}

unsafe extern "C" fn shader_usage_from_semantic_name(name: *const ::core::ffi::c_char) -> wined3d_decl_usage {
    let mut i: ::core::ffi::c_uint;
    i = (0) as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(semantic_names) as usize {
        if strcmp(name, semantic_names[i as usize]) == 0 {
            return i as wined3d_decl_usage;
        }
        i = i.wrapping_add(1);
    }

    return !(0 as ::core::ffi::c_uint) as wined3d_decl_usage;
}

unsafe extern "C" fn shader_sysval_semantic_from_usage(usage: wined3d_decl_usage) -> wined3d_sysval_semantic {
    match usage {
        WINED3D_DECL_USAGE_POSITION => {
            return WINED3D_SV_POSITION;
        }
        _ => {
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn shader_match_semantic(semantic_name: *const i8, usage: wined3d_decl_usage) -> BOOL {
    return (strcmp(semantic_name, shader_semantic_name_from_usage(usage)) == 0) as BOOL;
}

unsafe extern "C" fn shader_signature_from_semantic(
    e: *mut wined3d_shader_signature_element,
    s: *const wined3d_shader_semantic,
) {
    (*e).semantic_name = shader_semantic_name_from_usage((*s).usage);
    (*e).semantic_idx = (*s).usage_idx as u32;
    (*e).stream_idx = 0;
    (*e).sysval_semantic = shader_sysval_semantic_from_usage((*s).usage);
    (*e).component_type = WINED3D_TYPE_FLOAT;
    (*e).register_idx = (*s).reg.reg.idx[0 as usize].offset;
    (*e).mask = (*s).reg.write_mask;
}

unsafe extern "C" fn shader_signature_from_usage(
    e: *mut wined3d_shader_signature_element,
    usage: wined3d_decl_usage,
    usage_idx: UINT,
    reg_idx: UINT,
    write_mask: DWORD,
) {
    (*e).semantic_name = shader_semantic_name_from_usage(usage);
    (*e).semantic_idx = usage_idx;
    (*e).stream_idx = 0;
    (*e).sysval_semantic = shader_sysval_semantic_from_usage(usage);
    (*e).component_type = WINED3D_TYPE_FLOAT;
    (*e).register_idx = reg_idx;
    (*e).mask = write_mask;
}

unsafe extern "C" fn shader_select_frontend(source_type: vkd3d_shader_source_type) -> *const wined3d_shader_frontend {
    match source_type as u32 {
        VKD3D_SHADER_SOURCE_D3D_BYTECODE => {
            return core::ptr::addr_of!(sm1_shader_frontend);
        }

        VKD3D_SHADER_SOURCE_DXBC_TPF => {
            return core::ptr::addr_of!(sm4_shader_frontend);
        }

        _ => {
            WARN!("Invalid source type %#x specified.\n", source_type);
            return core::ptr::null();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_clear(buffer: *mut wined3d_string_buffer) {
    *(*buffer).buffer.offset(0 as isize) = 0 as core::ffi::c_char;
    (*buffer).content_size = 0;
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_init(buffer: *mut wined3d_string_buffer) -> BOOL {
    (*buffer).buffer_size = 32;
    (*buffer).buffer = malloc((*buffer).buffer_size as usize) as *mut core::ffi::c_char;
    if (*buffer).buffer.is_null() {
        ERR!("Failed to allocate shader buffer memory.\n");
        return (FALSE) as BOOL;
    }

    string_buffer_clear(buffer);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_free(buffer: *mut wined3d_string_buffer) {
    free((*buffer).buffer as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_resize(buffer: *mut wined3d_string_buffer, rc: ::core::ffi::c_int) -> BOOL {
    let mut new_buffer: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut new_buffer_size: ::core::ffi::c_uint = (*buffer).buffer_size.wrapping_mul(2);
    while rc > 0 && (rc as ::core::ffi::c_uint) >= new_buffer_size.wrapping_sub((*buffer).content_size) {
        new_buffer_size = new_buffer_size.wrapping_mul(2);
    }
    new_buffer = realloc((*buffer).buffer as *mut ::core::ffi::c_void, new_buffer_size as usize) as *mut ::core::ffi::c_char;
    if new_buffer.is_null() {
        ERR!("Failed to grow buffer.\n");
        *(*buffer).buffer.offset((*buffer).content_size as isize) = 0 as ::core::ffi::c_char;
        return (FALSE) as BOOL;
    }
    (*buffer).buffer = new_buffer;
    (*buffer).buffer_size = new_buffer_size;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn shader_vaddline(
    buffer: *mut wined3d_string_buffer,
    format: *const core::ffi::c_char,
    args: va_list,
) -> core::ffi::c_int {
    let mut rem: core::ffi::c_uint = 0;
    let mut rc: core::ffi::c_int = 0;

    rem = (*buffer).buffer_size.wrapping_sub((*buffer).content_size);
    rc = vsnprintf(
        (*buffer).buffer.add((*buffer).content_size as usize),
        rem as usize,
        format,
        args,
    );
    if rc < 0 || (rc as core::ffi::c_uint) >= rem {
        return rc;
    }

    (*buffer).content_size = (*buffer)
        .content_size
        .wrapping_add(rc as core::ffi::c_uint);
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn shader_addline_v(buffer: *mut wined3d_string_buffer, format: *const i8, ap_list: *mut ::core::ffi::c_void) -> i32 {
    let mut args: va_list = *(ap_list as *mut va_list);
    let mut args_0: va_list = *(ap_list as *mut va_list);
    let mut ret: i32 = 0;
    loop {
        //

        ret = shader_vaddline(buffer, format, args_0);
        //
        if ret == 0 {
            return ret;
        }
        if string_buffer_resize(buffer, ret) == 0 {
            return -1;
        }
    }
}


#[no_mangle]
pub unsafe extern "C" fn string_buffer_get(
    list: *mut wined3d_string_buffer_list,
) -> *mut wined3d_string_buffer {
    let mut buffer: *mut wined3d_string_buffer = core::ptr::null_mut();

    if list_empty(core::ptr::addr_of_mut!((*list).list)) != 0 {
        buffer = malloc(core::mem::size_of::<wined3d_string_buffer>())
            as *mut wined3d_string_buffer;
        if buffer.is_null() || string_buffer_init(buffer) == 0 {
            ERR!("Couldn't allocate buffer for temporary string.\n");
            free(buffer as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }
    } else {
        buffer = (list_head(core::ptr::addr_of_mut!((*list).list))
            as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_string_buffer, entry) as isize))
            as *mut wined3d_string_buffer;
        list_remove(core::ptr::addr_of_mut!((*buffer).entry));
    }
    string_buffer_clear(buffer);
    return buffer;
}

unsafe extern "C" fn string_buffer_vsprintf(buffer: *mut wined3d_string_buffer, format: *const i8, args: va_list) -> i32 {
    if buffer.is_null() {
        return 0;
    }
    string_buffer_clear(buffer);
    return shader_vaddline(buffer, format, args);
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_sprintf_v(buffer: *mut wined3d_string_buffer, format: *const i8, ap_list: *mut ::core::ffi::c_void) {
    // Windows' va_list is a `char*` on all three targets: passing it by value is the C loop's
    // fresh va_start each iteration (the callee advances its own copy).
    let args: va_list = *(ap_list as *mut va_list);
    let mut ret: i32;
    loop {
        ret = string_buffer_vsprintf(buffer, format, args);
        if ret == 0 {
            return;
        }
        if string_buffer_resize(buffer, ret) == 0 {
            return;
        }
    }
}


#[no_mangle]
pub unsafe extern "C" fn string_buffer_release(list: *mut wined3d_string_buffer_list, buffer: *mut wined3d_string_buffer) {
    if buffer.is_null() {
        return;
    }
    list_add_head(core::ptr::addr_of_mut!((*list).list), core::ptr::addr_of_mut!((*buffer).entry));
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_list_init(list: *mut wined3d_string_buffer_list) {
    list_init(core::ptr::addr_of_mut!((*list).list));
}
#[no_mangle]
pub unsafe extern "C" fn string_buffer_list_cleanup(list: *mut wined3d_string_buffer_list) {
    let mut buffer: *mut wined3d_string_buffer = core::ptr::null_mut();
    let mut buffer_next: *mut wined3d_string_buffer = core::ptr::null_mut();
    // LIST_FOR_EACH_ENTRY_SAFE(buffer, buffer_next, &list->list, struct wined3d_string_buffer, entry) expanded
    buffer = ((*list).list.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_string_buffer, entry)) as *mut wined3d_string_buffer;
    buffer_next = ((*buffer).entry.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_string_buffer, entry)) as *mut wined3d_string_buffer;
    while core::ptr::addr_of_mut!((*buffer).entry) != core::ptr::addr_of_mut!((*list).list) {
        string_buffer_free(buffer);
        free(buffer as *mut core::ffi::c_void);
        buffer = buffer_next;
        buffer_next = ((*buffer).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_string_buffer, entry)) as *mut wined3d_string_buffer;
    }
    list_init(core::ptr::addr_of_mut!((*list).list));
}

unsafe extern "C" fn shader_delete_constant_list(clist: *mut list) {
    let mut constant: *mut wined3d_shader_lconst = ((*clist).next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_shader_lconst, entry)) as *mut wined3d_shader_lconst;
    let mut constant_next: *mut wined3d_shader_lconst = ((*constant).entry.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_shader_lconst, entry)) as *mut wined3d_shader_lconst;
    while &raw mut (*constant).entry != clist {
        free(constant as *mut ::core::ffi::c_void);
        constant = constant_next;
        constant_next = ((*constant).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_shader_lconst, entry)) as *mut wined3d_shader_lconst;
    }
    list_init(clist);
}

unsafe extern "C" fn shader_set_limits(shader: *mut wined3d_shader) {
    static mut vs_limits: [limits_entry; 6] = [
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(1, 0),
            max_version: WINED3D_SHADER_VERSION!(1, 1),
            limits: wined3d_shader_limits {
                sampler: 0,
                constant_int: 0,
                constant_float: 256,
                constant_bool: 0,
                packed_output: 12,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(2, 0),
            max_version: WINED3D_SHADER_VERSION!(2, 1),
            limits: wined3d_shader_limits {
                sampler: 0,
                constant_int: 16,
                constant_float: 256,
                constant_bool: 16,
                packed_output: 12,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(3, 0),
            max_version: WINED3D_SHADER_VERSION!(3, 0),
            limits: wined3d_shader_limits {
                sampler: 4,
                constant_int: 16,
                constant_float: 256,
                constant_bool: 16,
                packed_output: 12,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(4, 0),
            max_version: WINED3D_SHADER_VERSION!(4, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 16,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(4, 1),
            max_version: WINED3D_SHADER_VERSION!(5, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 32,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: 0,
            max_version: 0,
            limits: wined3d_shader_limits {
                sampler: 0,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
    ];
    static mut hs_limits: [limits_entry; 1] = [limits_entry {
        min_version: WINED3D_SHADER_VERSION!(5, 0),
        max_version: WINED3D_SHADER_VERSION!(5, 0),
        limits: wined3d_shader_limits {
            sampler: 16,
            constant_int: 0,
            constant_float: 0,
            constant_bool: 0,
            packed_output: 32,
            packed_input: 32,
        },
    }];
    static mut ds_limits: [limits_entry; 1] = [limits_entry {
        min_version: WINED3D_SHADER_VERSION!(5, 0),
        max_version: WINED3D_SHADER_VERSION!(5, 0),
        limits: wined3d_shader_limits {
            sampler: 16,
            constant_int: 0,
            constant_float: 0,
            constant_bool: 0,
            packed_output: 32,
            packed_input: 32,
        },
    }];
    static mut gs_limits: [limits_entry; 3] = [
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(4, 0),
            max_version: WINED3D_SHADER_VERSION!(4, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 32,
                packed_input: 16,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(4, 1),
            max_version: WINED3D_SHADER_VERSION!(5, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 32,
                packed_input: 32,
            },
        },
        limits_entry {
            min_version: 0,
            max_version: 0,
            limits: wined3d_shader_limits {
                sampler: 0,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
    ];
    static mut ps_limits: [limits_entry; 7] = [
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(1, 0),
            max_version: WINED3D_SHADER_VERSION!(1, 3),
            limits: wined3d_shader_limits {
                sampler: 4,
                constant_int: 0,
                constant_float: 8,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(1, 4),
            max_version: WINED3D_SHADER_VERSION!(1, 4),
            limits: wined3d_shader_limits {
                sampler: 6,
                constant_int: 0,
                constant_float: 8,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(2, 0),
            max_version: WINED3D_SHADER_VERSION!(2, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 32,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(2, 1),
            max_version: WINED3D_SHADER_VERSION!(2, 1),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 16,
                constant_float: 32,
                constant_bool: 16,
                packed_output: 0,
                packed_input: 0,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(3, 0),
            max_version: WINED3D_SHADER_VERSION!(3, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 16,
                constant_float: 224,
                constant_bool: 16,
                packed_output: 0,
                packed_input: 10,
            },
        },
        limits_entry {
            min_version: WINED3D_SHADER_VERSION!(4, 0),
            max_version: WINED3D_SHADER_VERSION!(5, 0),
            limits: wined3d_shader_limits {
                sampler: 16,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 32,
            },
        },
        limits_entry {
            min_version: 0,
            max_version: 0,
            limits: wined3d_shader_limits {
                sampler: 0,
                constant_int: 0,
                constant_float: 0,
                constant_bool: 0,
                packed_output: 0,
                packed_input: 0,
            },
        },
    ];
    static mut cs_limits: [limits_entry; 1] = [limits_entry {
        min_version: WINED3D_SHADER_VERSION!(5, 0),
        max_version: WINED3D_SHADER_VERSION!(5, 0),
        limits: wined3d_shader_limits {
            sampler: 16,
            constant_int: 0,
            constant_float: 0,
            constant_bool: 0,
            packed_output: 0,
            packed_input: 0,
        },
    }];
    let mut limits_array: *const limits_entry = ::core::ptr::null::<limits_entry>();
    let mut shader_version: DWORD = WINED3D_SHADER_VERSION!(
        (*shader).reg_maps.shader_version.major as ::core::ffi::c_uint,
        (*shader).reg_maps.shader_version.minor as ::core::ffi::c_uint,
    ) as DWORD;
    let mut i: ::core::ffi::c_int = 0;

    let shader_type: ::core::ffi::c_uint = (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint;
    if shader_type == WINED3D_SHADER_TYPE_HULL {
        limits_array = &raw const hs_limits as *const limits_entry;
    } else if shader_type == WINED3D_SHADER_TYPE_DOMAIN {
        limits_array = &raw const ds_limits as *const limits_entry;
    } else if shader_type == WINED3D_SHADER_TYPE_GEOMETRY {
        limits_array = &raw const gs_limits as *const limits_entry;
    } else if shader_type == WINED3D_SHADER_TYPE_PIXEL {
        limits_array = &raw const ps_limits as *const limits_entry;
    } else if shader_type == WINED3D_SHADER_TYPE_COMPUTE {
        limits_array = &raw const cs_limits as *const limits_entry;
    } else if shader_type == WINED3D_SHADER_TYPE_VERTEX {
        limits_array = &raw const vs_limits as *const limits_entry;
    } else {
        FIXME!(
            "Unexpected shader type %u found.\n",
            (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint
        );
        limits_array = &raw const vs_limits as *const limits_entry;
    }

    while (*limits_array.offset(i as isize)).min_version != 0
        && (*limits_array.offset(i as isize)).min_version <= shader_version
    {
        if shader_version <= (*limits_array.offset(i as isize)).max_version {
            (*shader).limits = &raw const (*limits_array.offset(i as isize)).limits;
            break;
        } else {
            i += 1;
        }
    }
    if (*shader).limits.is_null() {
        FIXME!(
            "Unexpected shader version \"%u.%u\" (shader type %u).\n",
            (*shader).reg_maps.shader_version.major as ::core::ffi::c_int,
            (*shader).reg_maps.shader_version.minor as ::core::ffi::c_int,
            (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint
        );
        (*shader).limits = &raw const (*limits_array.offset(
            max!(0, i - 1) as isize,
        ))
        .limits;
    }
}

unsafe extern "C" fn shader_record_register_usage(
    mut shader: *mut wined3d_shader,
    mut reg_maps: *mut wined3d_shader_reg_maps,
    mut reg: *const wined3d_shader_register,
    mut shader_type: wined3d_shader_type,
    mut constf_size: ::core::ffi::c_uint,
) -> BOOL {
    match (*reg).type_ as ::core::ffi::c_uint {
        WINED3DSPR_TEXTURE => {
            if shader_type as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint {
                (*reg_maps).texcoord = ((*reg_maps).texcoord as ::core::ffi::c_uint
                    | (1 as ::core::ffi::c_uint)
                        << (*reg).idx[0 as usize].offset)
                    as BYTE;
            } else {
                (*reg_maps).address = ((*reg_maps).address as ::core::ffi::c_uint
                    | (1 as ::core::ffi::c_uint)
                        << (*reg).idx[0 as usize].offset)
                    as BYTE;
            }
        }
        WINED3DSPR_TEMP => {
            (*reg_maps).temporary |=
                (1 as ::core::ffi::c_uint) << (*reg).idx[0 as usize].offset;
        }
        WINED3DSPR_INPUT => {
            if !(*reg).idx[0 as usize].rel_addr.is_null() {
                (*reg_maps).set_input_rel_addressing(1 as DWORD);
            }
            if shader_type as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint {
                if !(*reg).idx[0 as usize].rel_addr.is_null() {
                    (*shader).u.ps.input_reg_used = !(0 as DWORD);
                } else {
                    (*shader).u.ps.input_reg_used |= (1 as ::core::ffi::c_uint)
                        << (*reg).idx[0 as usize].offset;
                }
            } else {
                (*reg_maps).input_registers |= (1 as ::core::ffi::c_uint)
                    << (*reg).idx[0 as usize].offset;
            }
        }
        WINED3DSPR_RASTOUT => {
            if (*reg).idx[0 as usize].offset == 1 {
                (*reg_maps).set_fog(1 as DWORD);
            }
            if (*reg).idx[0 as usize].offset == 2 {
                (*reg_maps).set_point_size(1 as DWORD);
            }
        }
        WINED3DSPR_MISCTYPE => {
            if shader_type as ::core::ffi::c_uint == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint {
                if (*reg).idx[0 as usize].offset == 0 {
                    (*reg_maps).set_vpos(1 as DWORD);
                } else if (*reg).idx[0 as usize].offset == 1 {
                    (*reg_maps).set_usesfacing(1 as DWORD);
                }
            }
        }
        WINED3DSPR_CONST => {
            if !(*reg).idx[0 as usize].rel_addr.is_null() {
                if (*reg).idx[0 as usize].offset < (*reg_maps).min_rel_offset {
                    (*reg_maps).min_rel_offset = (*reg).idx[0 as usize].offset;
                }
                if (*reg).idx[0 as usize].offset > (*reg_maps).max_rel_offset {
                    (*reg_maps).max_rel_offset = (*reg).idx[0 as usize].offset;
                }
                (*reg_maps).set_usesrelconstF(TRUE as DWORD);
            } else {
                if (*reg).idx[0 as usize].offset
                    >= min!((*(*shader).limits).constant_float, constf_size)
                {
                    WARN!(
                        "Shader using float constant %u which is not supported.\n",
                        (*reg).idx[0 as usize].offset
                    );
                    return (FALSE) as BOOL;
                } else {
                    wined3d_insert_bits(
                        (*reg_maps).constf,
                        (*reg).idx[0 as usize].offset,
                        1,
                        0x1,
                    );
                }
            }
        }
        WINED3DSPR_CONSTINT => {
            if (*reg).idx[0 as usize].offset >= (*(*shader).limits).constant_int {
                WARN!(
                    "Shader using integer constant %u which is not supported.\n",
                    (*reg).idx[0 as usize].offset
                );
                return (FALSE) as BOOL;
            } else {
                (*reg_maps).integer_constants = ((*reg_maps).integer_constants as ::core::ffi::c_uint
                    | (1 as ::core::ffi::c_uint)
                        << (*reg).idx[0 as usize].offset)
                    as WORD;
            }
        }
        WINED3DSPR_CONSTBOOL => {
            if (*reg).idx[0 as usize].offset >= (*(*shader).limits).constant_bool {
                WARN!(
                    "Shader using bool constant %u which is not supported.\n",
                    (*reg).idx[0 as usize].offset
                );
                return (FALSE) as BOOL;
            } else {
                (*reg_maps).boolean_constants = ((*reg_maps).boolean_constants as ::core::ffi::c_uint
                    | (1 as ::core::ffi::c_uint)
                        << (*reg).idx[0 as usize].offset)
                    as WORD;
            }
        }
        WINED3DSPR_COLOROUT => {
            (*reg_maps).rt_mask |=
                (1 as ::core::ffi::c_uint) << (*reg).idx[0 as usize].offset;
        }
        WINED3DSPR_OUTCONTROLPOINT => {
            (*reg_maps).set_vocp(1 as DWORD);
        }
        WINED3DSPR_SAMPLEMASK => {
            (*reg_maps).set_sample_mask(1 as DWORD);
        }
        WINED3DSPR_STENCILREF => {
            (*reg_maps).set_stencil_ref(1 as DWORD);
        }
        _ => {
            TRACE!(
                "Not recording register of type %#x and [%#x][%#x].\n",
                (*reg).type_ as ::core::ffi::c_uint,
                (*reg).idx[0 as usize].offset,
                (*reg).idx[1 as usize].offset
            );
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn shader_record_sample(
    reg_maps: *mut wined3d_shader_reg_maps,
    resource_idx: ::core::ffi::c_uint,
    sampler_idx: ::core::ffi::c_uint,
    bind_idx: ::core::ffi::c_uint,
) {
    let mut entries: *mut wined3d_shader_sampler_map_entry = core::ptr::null_mut();
    let mut entry: *mut wined3d_shader_sampler_map_entry = core::ptr::null_mut();
    let mut map: *mut wined3d_shader_sampler_map = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    map = core::ptr::addr_of_mut!((*reg_maps).sampler_map);
    entries = (*map).entries;
    i = 0;
    while (i as usize) < (*map).count {
        if (*entries.offset(i as isize)).resource_idx == resource_idx
            && (*entries.offset(i as isize)).sampler_idx == sampler_idx
        {
            return;
        }
        i = i.wrapping_add(1);
    }

    if (*map).size == 0 {
        entries = calloc(
            (4 as usize) as _,
            (core::mem::size_of::<wined3d_shader_sampler_map_entry>() as usize) as _,
        ) as *mut wined3d_shader_sampler_map_entry;
        if entries.is_null() {
            ERR!("Failed to allocate sampler map entries.\n");
            return;
        }
        (*map).size = 4 as usize;
        (*map).entries = entries;
    } else if (*map).count == (*map).size {
        let mut new_size: usize = (*map).size.wrapping_mul(2 as usize);

        if (core::mem::size_of::<wined3d_shader_sampler_map_entry>() as usize)
            .wrapping_mul(new_size)
            <= (core::mem::size_of::<wined3d_shader_sampler_map_entry>() as usize)
                .wrapping_mul((*map).size)
            || {
                entries = realloc(
                    entries as *mut ::core::ffi::c_void,
                    (core::mem::size_of::<wined3d_shader_sampler_map_entry>() as usize)
                        .wrapping_mul(new_size),
                ) as *mut wined3d_shader_sampler_map_entry;
                entries.is_null()
            }
        {
            ERR!("Failed to resize sampler map entries.\n");
            return;
        }
        (*map).size = new_size;
        (*map).entries = entries;
    }

    let fresh20 = (*map).count;
    (*map).count = (*map).count.wrapping_add(1);
    entry = entries.offset(fresh20 as isize);
    (*entry).resource_idx = resource_idx;
    (*entry).sampler_idx = sampler_idx;
    (*entry).bind_idx = bind_idx;
}

unsafe extern "C" fn get_instr_extra_regcount(instr: WINED3D_SHADER_INSTRUCTION_HANDLER, param: ::core::ffi::c_uint) -> ::core::ffi::c_uint {
    match instr as ::core::ffi::c_uint {
        // WINED3DSIH_M4x4 (161), WINED3DSIH_M3x4 (159)
        161 | 159 => {
            return if param == 1 { 3 } else { 0 };
        }
        // WINED3DSIH_M4x3 (160), WINED3DSIH_M3x3 (158)
        160 | 158 => {
            return if param == 1 { 2 } else { 0 };
        }
        // WINED3DSIH_M3x2 (157)
        157 => {
            return if param == 1 { 1 } else { 0 };
        }
        _ => return 0,
    }
}

unsafe extern "C" fn shader_reg_maps_add_tgsm(reg_maps: *mut wined3d_shader_reg_maps, register_idx: core::ffi::c_uint, size: core::ffi::c_uint, stride: core::ffi::c_uint) -> HRESULT {
    let mut tgsm: *mut wined3d_shader_tgsm = core::ptr::null_mut();

    if register_idx >= MAX_TGSM_REGISTERS {
        ERR!("Invalid TGSM register index %u.\n", register_idx);
        return S_OK;
    }
    if (*reg_maps).shader_version.type_ != WINED3D_SHADER_TYPE_COMPUTE {
        FIXME!("TGSM declarations are allowed only in compute shaders.\n");
        return S_OK;
    }

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*reg_maps).tgsm) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*reg_maps).tgsm_capacity),
        (register_idx + 1) as SIZE_T,
        (core::mem::size_of::<wined3d_shader_tgsm>()) as SIZE_T,
    ) == 0
    {
        return E_OUTOFMEMORY;
    }

    (*reg_maps).tgsm_count = max!(register_idx + 1, (*reg_maps).tgsm_count);
    tgsm = (*reg_maps).tgsm.offset(register_idx as isize);
    (*tgsm).size = size;
    (*tgsm).stride = stride;
    return S_OK;
}

unsafe extern "C" fn shader_record_shader_phase(
    shader: *mut wined3d_shader,
    current_phase: *mut *mut wined3d_shader_phase,
    ins: *const wined3d_shader_instruction,
    current_instruction_ptr: *const DWORD,
    previous_instruction_ptr: *const DWORD,
) -> HRESULT {
    let mut phase: *mut wined3d_shader_phase = core::ptr::null_mut::<wined3d_shader_phase>();

    phase = *current_phase;
    if !phase.is_null() {
        (*phase).end = previous_instruction_ptr;
        *current_phase = core::ptr::null_mut::<wined3d_shader_phase>();
    }

    if (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint != WINED3D_SHADER_TYPE_HULL {
        ERR!(
            "Unexpected shader type %s.\n",
            debug_shader_type((*shader).reg_maps.shader_version.type_)
        );
        return E_FAIL;
    }

    match (*ins).handler_idx as ::core::ffi::c_uint {
        WINED3DSIH_HS_CONTROL_POINT_PHASE => {
            if !(*shader).u.hs.phases.control_point.is_null() {
                FIXME!("Multiple control point phases.\n");
                free((*shader).u.hs.phases.control_point as *mut ::core::ffi::c_void);
            }
            (*shader).u.hs.phases.control_point = calloc(
                1,
                (core::mem::size_of::<wined3d_shader_phase>() as size_t) as _,
            ) as *mut wined3d_shader_phase;
            if (*shader).u.hs.phases.control_point.is_null() {
                return E_OUTOFMEMORY;
            }
            phase = (*shader).u.hs.phases.control_point;
        }
        WINED3DSIH_HS_FORK_PHASE => {
            if wined3d_array_reserve(
                &raw mut (*shader).u.hs.phases.fork as *mut *mut ::core::ffi::c_void,
                &raw mut (*shader).u.hs.phases.fork_size,
                (*shader).u.hs.phases.fork_count.wrapping_add(1) as SIZE_T,
                core::mem::size_of::<wined3d_shader_phase>() as SIZE_T,
            ) == 0
            {
                return E_OUTOFMEMORY;
            }
            let fresh22 = (*shader).u.hs.phases.fork_count;
            (*shader).u.hs.phases.fork_count = (*shader).u.hs.phases.fork_count.wrapping_add(1);
            phase = (*shader).u.hs.phases.fork.offset(fresh22 as isize) as *mut wined3d_shader_phase;
        }
        WINED3DSIH_HS_JOIN_PHASE => {
            if wined3d_array_reserve(
                &raw mut (*shader).u.hs.phases.join as *mut *mut ::core::ffi::c_void,
                &raw mut (*shader).u.hs.phases.join_size,
                (*shader).u.hs.phases.join_count.wrapping_add(1) as SIZE_T,
                core::mem::size_of::<wined3d_shader_phase>() as SIZE_T,
            ) == 0
            {
                return E_OUTOFMEMORY;
            }
            let fresh23 = (*shader).u.hs.phases.join_count;
            (*shader).u.hs.phases.join_count = (*shader).u.hs.phases.join_count.wrapping_add(1);
            phase = (*shader).u.hs.phases.join.offset(fresh23 as isize) as *mut wined3d_shader_phase;
        }
        _ => {
            ERR!(
                "Unexpected opcode %s.\n",
                debug_d3dshaderinstructionhandler((*ins).handler_idx)
            );
            return E_FAIL;
        }
    }

    (*phase).start = current_instruction_ptr;
    *current_phase = phase;

    return WINED3D_OK;
}

unsafe extern "C" fn shader_calculate_clip_or_cull_distance_mask(
    mut e: *const wined3d_shader_signature_element,
    mut mask: *mut ::core::ffi::c_uint,
) -> HRESULT {
    if (*e).semantic_idx >= (WINED3D_MAX_CLIP_DISTANCES / 4) as ::core::ffi::c_uint
    {
        *mask = 0 as ::core::ffi::c_uint;
        WARN!("Invalid clip/cull distance index %u.\n", (*e).semantic_idx);
        return WINED3DERR_INVALIDCALL;
    }
    *mask = (((*e).mask & WINED3DSP_WRITEMASK_ALL as u32)
        << (4 as u32).wrapping_mul((*e).semantic_idx))
        as ::core::ffi::c_uint;
    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_insert_interpolation_mode(
    packed_interpolation_mode: *mut uint32_t,
    register_idx: ::core::ffi::c_uint,
    mode: wined3d_shader_interpolation_mode,
) {
    if mode as ::core::ffi::c_uint > WINED3DSIM_LINEAR_NOPERSPECTIVE_SAMPLE as ::core::ffi::c_uint {
        FIXME!("Unexpected interpolation mode %#x.\n", mode as ::core::ffi::c_uint);
    }
    wined3d_insert_bits(
        packed_interpolation_mode,
        register_idx.wrapping_mul(WINED3D_PACKED_INTERPOLATION_BIT_COUNT as ::core::ffi::c_uint),
        WINED3D_PACKED_INTERPOLATION_BIT_COUNT as ::core::ffi::c_uint,
        mode as DWORD,
    );
}

unsafe extern "C" fn shader_scan_output_signature(shader: *mut wined3d_shader) -> HRESULT {
    let output_signature: *const wined3d_shader_signature = core::ptr::addr_of!((*shader).output_signature);
    let reg_maps: *mut wined3d_shader_reg_maps = core::ptr::addr_of_mut!((*shader).reg_maps);
    let mut i: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    i = 0 as ::core::ffi::c_uint;
    while i < (*output_signature).element_count {
        let e: *const wined3d_shader_signature_element =
            (*output_signature).elements.add(i as usize);
        let mut mask: ::core::ffi::c_uint = 0;

        (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << (*e).register_idx;
        if (*e).sysval_semantic as ::core::ffi::c_uint == WINED3D_SV_CLIP_DISTANCE as ::core::ffi::c_uint {
            hr = shader_calculate_clip_or_cull_distance_mask(e, core::ptr::addr_of_mut!(mask));
            if hr < 0 {
                return hr;
            }
            (*reg_maps).set_clip_distance_mask(((*reg_maps).clip_distance_mask() | (mask)) as _);
        } else if (*e).sysval_semantic as ::core::ffi::c_uint == WINED3D_SV_CULL_DISTANCE as ::core::ffi::c_uint {
            hr = shader_calculate_clip_or_cull_distance_mask(e, core::ptr::addr_of_mut!(mask));
            if hr < 0 {
                return hr;
            }
            (*reg_maps).set_cull_distance_mask(((*reg_maps).cull_distance_mask() | (mask)) as _);
        } else if (*e).sysval_semantic as ::core::ffi::c_uint == WINED3D_SV_VIEWPORT_ARRAY_INDEX as ::core::ffi::c_uint {
            (*reg_maps).set_viewport_array((1 as DWORD) as _);
        }
        i = i.wrapping_add(1);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn shader_get_registers_used(mut shader: *mut wined3d_shader, mut constf_size: DWORD) -> HRESULT {
    let mut input_signature_elements: [wined3d_shader_signature_element; max!((MAX_ATTRIBS as usize), (MAX_REG_INPUT as usize))] = core::mem::zeroed();
    let mut output_signature_elements: [wined3d_shader_signature_element; (MAX_REG_OUTPUT as _)] = core::mem::zeroed();
    let output_signature: *mut wined3d_shader_signature = &raw mut (*shader).output_signature;
    let input_signature: *mut wined3d_shader_signature = &raw mut (*shader).input_signature;
    let reg_maps: *mut wined3d_shader_reg_maps = &raw mut (*shader).reg_maps;
    let fe: *const wined3d_shader_frontend = (*shader).frontend;
    let mut cur_loop_depth: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut max_loop_depth: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut shader_version: wined3d_shader_version = core::mem::zeroed();
    let mut phase: *mut wined3d_shader_phase = core::ptr::null_mut();
    let mut ptr: *const DWORD = core::ptr::null();
    let mut prev_ins: *const DWORD = core::ptr::null();
    let mut current_ins: *const DWORD = core::ptr::null();
    let fe_data: *mut ::core::ffi::c_void = (*shader).frontend_data;
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    memset(reg_maps as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, core::mem::size_of::<wined3d_shader_reg_maps>() as size_t);
    memset(&raw mut input_signature_elements as *mut wined3d_shader_signature_element as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, core::mem::size_of::<[wined3d_shader_signature_element; max!((MAX_ATTRIBS as usize), (MAX_REG_INPUT as usize))]>() as size_t);
    memset(&raw mut output_signature_elements as *mut wined3d_shader_signature_element as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, core::mem::size_of::<[wined3d_shader_signature_element; (MAX_REG_OUTPUT as _)]>() as size_t);
    (*reg_maps).min_rel_offset = !(0 as ::core::ffi::c_uint);
    list_init(&raw mut (*reg_maps).indexable_temps);

    (*fe).shader_read_header.unwrap_unchecked()(fe_data, &raw mut ptr, &raw mut shader_version);
    current_ins = ptr;
    prev_ins = current_ins;
    (*reg_maps).shader_version = shader_version;

    shader_set_limits(shader);

    (*reg_maps).constf = calloc(
        ((((min!((*(*shader).limits).constant_float, constf_size) + 31) / 32) as size_t)) as _,
        (core::mem::size_of::<uint32_t>() as size_t) as _,
    ) as *mut uint32_t;
    if (*reg_maps).constf.is_null() {
        ERR!("Failed to allocate constant map memory.\n");
        return E_OUTOFMEMORY;
    }

    while (*fe).shader_is_end.unwrap_unchecked()(fe_data, &raw mut ptr) == 0 {
        let mut ins: wined3d_shader_instruction = core::mem::zeroed();

        current_ins = ptr;
        (*fe).shader_read_instruction.unwrap_unchecked()(fe_data, &raw mut ptr, &raw mut ins);

        if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TABLE_SIZE as ::core::ffi::c_uint) {
            WARN!("Encountered unrecognised or invalid instruction.\n");
            return WINED3DERR_INVALIDCALL;
        }

        if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL as ::core::ffi::c_uint)
            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_UAV_TYPED as ::core::ffi::c_uint)
        {
            let mut semantic: *mut wined3d_shader_semantic = &raw mut ins.declaration.semantic;
            let mut reg_idx: ::core::ffi::c_uint = (*semantic).reg.reg.idx[0 as usize].offset;

            'sw: {
                match (*semantic).reg.reg.type_ as ::core::ffi::c_uint {
                    WINED3DSPR_INPUT => {
                        if reg_idx >= MAX_REG_INPUT as ::core::ffi::c_uint {
                            ERR!("Invalid input register index %u.\n", reg_idx);
                            break 'sw;
                        }
                        if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint)
                            && (shader_version.major as ::core::ffi::c_int) == (3 as ::core::ffi::c_int)
                            && ((*semantic).usage as ::core::ffi::c_uint) == (WINED3D_DECL_USAGE_POSITION as ::core::ffi::c_uint)
                            && (*semantic).usage_idx == 0
                        {
                            return WINED3DERR_INVALIDCALL;
                        }
                        (*reg_maps).input_registers |= (1 as ::core::ffi::c_uint) << reg_idx;
                        shader_signature_from_semantic(&raw mut input_signature_elements[reg_idx as usize], semantic);
                        break 'sw;
                    }
                    WINED3DSPR_OUTPUT => {
                        if reg_idx >= MAX_REG_OUTPUT as ::core::ffi::c_uint {
                            ERR!("Invalid output register index %u.\n", reg_idx);
                            break 'sw;
                        }
                        (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << reg_idx;
                        shader_signature_from_semantic(&raw mut output_signature_elements[reg_idx as usize], semantic);
                        if ((*semantic).usage as ::core::ffi::c_uint) == (WINED3D_DECL_USAGE_FOG as ::core::ffi::c_uint) {
                            (*reg_maps).set_fog(1 as DWORD);
                        }
                        if ((*semantic).usage as ::core::ffi::c_uint) == (WINED3D_DECL_USAGE_PSIZE as ::core::ffi::c_uint) {
                            (*reg_maps).set_point_size(1 as DWORD);
                        }
                        break 'sw;
                    }
                    WINED3DSPR_SAMPLER => {
                        shader_record_sample(reg_maps, reg_idx, reg_idx, reg_idx);
                    }
                    WINED3DSPR_RESOURCE => {}
                    WINED3DSPR_UAV => {
                        if reg_idx >= ARRAY_SIZE!((*reg_maps).uav_resource_info) as ::core::ffi::c_uint {
                            ERR!("Invalid UAV resource index %u.\n", reg_idx);
                            break 'sw;
                        }
                        (*reg_maps).uav_resource_info[reg_idx as usize].type_ = (*semantic).resource_type;
                        (*reg_maps).uav_resource_info[reg_idx as usize].data_type = (*semantic).resource_data_type;
                        if ins.flags != 0 {
                            FIXME!("Ignoring typed UAV flags %#x.\n", ins.flags);
                        }
                        break 'sw;
                    }
                    _ => {
                        TRACE!("Not recording DCL register type %#x.\n", (*semantic).reg.reg.type_ as ::core::ffi::c_uint);
                        break 'sw;
                    }
                }
                // Resource declarations (reached from the 31 and 32 cases).
                if reg_idx >= ARRAY_SIZE!((*reg_maps).resource_info) as ::core::ffi::c_uint {
                    ERR!("Invalid resource index %u.\n", reg_idx);
                    break 'sw;
                }
                (*reg_maps).resource_info[reg_idx as usize].type_ = (*semantic).resource_type;
                if ((*semantic).resource_type as ::core::ffi::c_uint) == (WINED3D_SHADER_RESOURCE_TEXTURE_2DMS as ::core::ffi::c_uint)
                    && (*semantic).sample_count == 1 as ::core::ffi::c_uint
                {
                    (*reg_maps).resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_TEXTURE_2D;
                }
                if ((*semantic).resource_type as ::core::ffi::c_uint) == (WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY as ::core::ffi::c_uint)
                    && (*semantic).sample_count == 1 as ::core::ffi::c_uint
                {
                    (*reg_maps).resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY;
                }
                (*reg_maps).resource_info[reg_idx as usize].data_type = (*semantic).resource_data_type;
                wined3d_bitmap_set(&raw mut (*reg_maps).resource_map as *mut uint32_t, reg_idx);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_CONSTANT_BUFFER as ::core::ffi::c_uint)
        {
            let mut reg: *mut wined3d_shader_register = &raw mut ins.declaration.src.reg;
            if (*reg).idx[0 as usize].offset >= WINED3D_MAX_CBS as ::core::ffi::c_uint {
                ERR!("Invalid CB index %u.\n", (*reg).idx[0 as usize].offset);
            }
            else {
                (*reg_maps).cb_sizes[(*reg).idx[0 as usize].offset as usize] =
                    (*reg).idx[1 as usize].offset as UINT;
                wined3d_bitmap_set(&raw mut (*reg_maps).cb_map, (*reg).idx[0 as usize].offset);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_GLOBAL_FLAGS as ::core::ffi::c_uint)
        {
            if (ins.flags & (WINED3DSGF_FORCE_EARLY_DEPTH_STENCIL as uint32_t)) != 0 {
                if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint) {
                    (*shader).u.ps.force_early_depth_stencil = TRUE as BOOL;
                }
                else {
                    FIXME!("Invalid instruction %#x for shader type %#x.\n",
                        ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
                }
            }
            else {
                WARN!("Ignoring global flags %#x.\n", ins.flags);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_GS_INSTANCES as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_uint) {
                (*shader).u.gs.instance_count = ins.declaration.count;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_HS_FORK_PHASE_INSTANCE_COUNT as ::core::ffi::c_uint)
            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_HS_JOIN_PHASE_INSTANCE_COUNT as ::core::ffi::c_uint)
        {
            if !phase.is_null() {
                (*phase).instance_count = ins.declaration.count;
            }
            else {
                FIXME!("Instruction %s outside of shader phase.\n", debug_d3dshaderinstructionhandler(ins.handler_idx));
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_IMMEDIATE_CONSTANT_BUFFER as ::core::ffi::c_uint)
        {
            if !(*reg_maps).icb.is_null() {
                FIXME!("Multiple immediate constant buffers.\n");
            }
            (*reg_maps).icb = ins.declaration.icb;
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_INDEXABLE_TEMP as ::core::ffi::c_uint)
        {
            if !phase.is_null() {
            }
            else {
                let mut reg: *mut wined3d_shader_indexable_temp = core::ptr::null_mut();

                reg = malloc(core::mem::size_of::<wined3d_shader_indexable_temp>() as size_t) as *mut wined3d_shader_indexable_temp;
                if reg.is_null() {
                    return E_OUTOFMEMORY;
                }

                *reg = ins.declaration.indexable_temp;
                list_add_tail(&raw mut (*reg_maps).indexable_temps, &raw mut (*reg).entry);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_INPUT_PRIMITIVE as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_uint) {
                (*shader).u.gs.input_type = ins.declaration.primitive_type.type_;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_INPUT_PS as ::core::ffi::c_uint)
        {
            let mut reg_idx: ::core::ffi::c_uint = ins.declaration.dst.reg.idx[0 as usize].offset;
            if reg_idx >= MAX_REG_INPUT as ::core::ffi::c_uint {
                ERR!("Invalid register index %u.\n", reg_idx);
                break;
            }
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint) {
                wined3d_insert_interpolation_mode(&raw mut (*shader).u.ps.interpolation_mode as *mut uint32_t, reg_idx,
                    ins.flags as wined3d_shader_interpolation_mode);
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_OUTPUT as ::core::ffi::c_uint)
        {
            if (ins.declaration.dst.reg.type_ as ::core::ffi::c_uint) == (WINED3DSPR_DEPTHOUT as ::core::ffi::c_uint)
                || (ins.declaration.dst.reg.type_ as ::core::ffi::c_uint) == (WINED3DSPR_DEPTHOUTGE as ::core::ffi::c_uint)
                || (ins.declaration.dst.reg.type_ as ::core::ffi::c_uint) == (WINED3DSPR_DEPTHOUTLE as ::core::ffi::c_uint)
            {
                if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint) {
                    (*shader).u.ps.depth_output = ins.declaration.dst.reg.type_;
                }
                else {
                    FIXME!("Invalid instruction %#x for shader type %#x.\n",
                        ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
                }
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_OUTPUT_CONTROL_POINT_COUNT as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_uint) {
                (*shader).u.hs.output_vertex_count = ins.declaration.count;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_OUTPUT_TOPOLOGY as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_uint) {
                (*shader).u.gs.output_type = ins.declaration.primitive_type.type_;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_RESOURCE_RAW as ::core::ffi::c_uint)
        {
            let mut reg_idx: ::core::ffi::c_uint = ins.declaration.dst.reg.idx[0 as usize].offset;
            if reg_idx >= ARRAY_SIZE!((*reg_maps).resource_info) as ::core::ffi::c_uint {
                ERR!("Invalid resource index %u.\n", reg_idx);
                break;
            }
            (*reg_maps).resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_BUFFER;
            (*reg_maps).resource_info[reg_idx as usize].data_type = WINED3D_DATA_UINT;
            (*reg_maps).resource_info[reg_idx as usize].flags = WINED3D_VIEW_BUFFER_RAW as ::core::ffi::c_uint;
            wined3d_bitmap_set(&raw mut (*reg_maps).resource_map as *mut uint32_t, reg_idx);
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_RESOURCE_STRUCTURED as ::core::ffi::c_uint)
        {
            let mut reg_idx: ::core::ffi::c_uint = ins.declaration.structured_resource.reg.reg.idx[0 as usize].offset;
            if reg_idx >= ARRAY_SIZE!((*reg_maps).resource_info) as ::core::ffi::c_uint {
                ERR!("Invalid resource index %u.\n", reg_idx);
                break;
            }
            (*reg_maps).resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_BUFFER;
            (*reg_maps).resource_info[reg_idx as usize].data_type = WINED3D_DATA_UINT;
            (*reg_maps).resource_info[reg_idx as usize].flags = 0 as ::core::ffi::c_uint;
            (*reg_maps).resource_info[reg_idx as usize].stride =
                ins.declaration.structured_resource.byte_stride / (4 as ::core::ffi::c_uint);
            wined3d_bitmap_set(&raw mut (*reg_maps).resource_map as *mut uint32_t, reg_idx);
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_SAMPLER as ::core::ffi::c_uint)
        {
            if (ins.flags & (WINED3DSI_SAMPLER_COMPARISON_MODE as uint32_t)) != 0 {
                (*reg_maps).sampler_comparison_mode |=
                    (1 as ::core::ffi::c_uint) << ins.declaration.dst.reg.idx[0 as usize].offset;
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TEMPS as ::core::ffi::c_uint)
        {
            if !phase.is_null() {
                (*phase).temporary_count = ins.declaration.count;
            }
            else {
                (*reg_maps).temporary_count = ins.declaration.count;
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TESSELLATOR_DOMAIN as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_DOMAIN as ::core::ffi::c_uint) {
                (*shader).u.ds.tessellator_domain = ins.declaration.tessellator_domain;
            }
            else if (shader_version.type_ as ::core::ffi::c_uint) != (WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_uint) {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TESSELLATOR_OUTPUT_PRIMITIVE as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_uint) {
                (*shader).u.hs.tessellator_output_primitive = ins.declaration.tessellator_output_primitive;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TESSELLATOR_PARTITIONING as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_uint) {
                (*shader).u.hs.tessellator_partitioning = ins.declaration.tessellator_partitioning;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TGSM_RAW as ::core::ffi::c_uint)
        {
            hr = shader_reg_maps_add_tgsm(reg_maps,
                ins.declaration.tgsm_raw.reg.reg.idx[0 as usize].offset,
                ins.declaration.tgsm_raw.byte_count / (4 as ::core::ffi::c_uint),
                0 as ::core::ffi::c_uint);
            if hr < (0 as HRESULT) {
                return hr;
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_TGSM_STRUCTURED as ::core::ffi::c_uint)
        {
            let mut stride: ::core::ffi::c_uint = ins.declaration.tgsm_structured.byte_stride / (4 as ::core::ffi::c_uint);
            let mut size: ::core::ffi::c_uint = stride.wrapping_mul(ins.declaration.tgsm_structured.structure_count);
            hr = shader_reg_maps_add_tgsm(reg_maps,
                ins.declaration.tgsm_structured.reg.reg.idx[0 as usize].offset, size, stride);
            if hr < (0 as HRESULT) {
                return hr;
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_THREAD_GROUP as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_COMPUTE as ::core::ffi::c_uint) {
                (*shader).u.cs.thread_group_size = ins.declaration.thread_group_size;
            }
            else {
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_UAV_RAW as ::core::ffi::c_uint)
        {
            let mut reg_idx: ::core::ffi::c_uint = ins.declaration.dst.reg.idx[0 as usize].offset;
            if reg_idx >= ARRAY_SIZE!((*reg_maps).uav_resource_info) as ::core::ffi::c_uint {
                ERR!("Invalid UAV resource index %u.\n", reg_idx);
                break;
            }
            if ins.flags != 0 {
                FIXME!("Ignoring raw UAV flags %#x.\n", ins.flags);
            }
            (*reg_maps).uav_resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_BUFFER;
            (*reg_maps).uav_resource_info[reg_idx as usize].data_type = WINED3D_DATA_UINT;
            (*reg_maps).uav_resource_info[reg_idx as usize].flags = WINED3D_VIEW_BUFFER_RAW as ::core::ffi::c_uint;
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_UAV_STRUCTURED as ::core::ffi::c_uint)
        {
            let mut reg_idx: ::core::ffi::c_uint = ins.declaration.structured_resource.reg.reg.idx[0 as usize].offset;
            if reg_idx >= ARRAY_SIZE!((*reg_maps).uav_resource_info) as ::core::ffi::c_uint {
                ERR!("Invalid UAV resource index %u.\n", reg_idx);
                break;
            }
            if ins.flags != 0 {
                FIXME!("Ignoring structured UAV flags %#x.\n", ins.flags);
            }
            (*reg_maps).uav_resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_BUFFER;
            (*reg_maps).uav_resource_info[reg_idx as usize].data_type = WINED3D_DATA_UINT;
            (*reg_maps).uav_resource_info[reg_idx as usize].flags = 0 as ::core::ffi::c_uint;
            (*reg_maps).uav_resource_info[reg_idx as usize].stride =
                ins.declaration.structured_resource.byte_stride / (4 as ::core::ffi::c_uint);
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DCL_VERTICES_OUT as ::core::ffi::c_uint)
        {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_uint) {
                (*shader).u.gs.vertices_out = ins.declaration.count;
            }
            else {
                FIXME!("Invalid instruction %#x for shader type %#x.\n",
                    ins.handler_idx as ::core::ffi::c_uint, shader_version.type_ as ::core::ffi::c_uint);
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DEF as ::core::ffi::c_uint)
        {
            let mut lconst: *mut wined3d_shader_lconst = core::ptr::null_mut();
            let mut value: *mut ::core::ffi::c_float = core::ptr::null_mut();

            lconst = malloc(core::mem::size_of::<wined3d_shader_lconst>() as size_t) as *mut wined3d_shader_lconst;
            if lconst.is_null() {
                return E_OUTOFMEMORY;
            }

            (*lconst).idx = (*ins.dst.offset(0 as isize)).reg.idx[0 as usize].offset;
            memcpy(&raw mut (*lconst).value as *mut ::core::ffi::c_uint as *mut ::core::ffi::c_void,
                &raw const (*ins.src.offset(0 as isize)).reg.u.immconst_data as *const ::core::ffi::c_uint as *const ::core::ffi::c_void,
                (4 as size_t).wrapping_mul(core::mem::size_of::<DWORD>() as size_t));
            value = &raw mut (*lconst).value as *mut ::core::ffi::c_uint as *mut ::core::ffi::c_float;

            /* In pixel shader 1.X shaders, the constants are clamped between [-1;1] */
            if (shader_version.major as ::core::ffi::c_int) == (1 as ::core::ffi::c_int)
                && (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint)
            {
                if *value.offset(0 as isize) < -1.0f32 {
                    *value.offset(0 as isize) = -1.0f32;
                }
                else if *value.offset(0 as isize) > 1.0f32 {
                    *value.offset(0 as isize) = 1.0f32;
                }
                if *value.offset(1 as isize) < -1.0f32 {
                    *value.offset(1 as isize) = -1.0f32;
                }
                else if *value.offset(1 as isize) > 1.0f32 {
                    *value.offset(1 as isize) = 1.0f32;
                }
                if *value.offset(2 as isize) < -1.0f32 {
                    *value.offset(2 as isize) = -1.0f32;
                }
                else if *value.offset(2 as isize) > 1.0f32 {
                    *value.offset(2 as isize) = 1.0f32;
                }
                if *value.offset(3 as isize) < -1.0f32 {
                    *value.offset(3 as isize) = -1.0f32;
                }
                else if *value.offset(3 as isize) > 1.0f32 {
                    *value.offset(3 as isize) = 1.0f32;
                }
            }

            list_add_head(&raw mut (*shader).constantsF, &raw mut (*lconst).entry);

            if ((isinf!(*value.offset(0 as isize))) != 0) || ((isnan!(*value.offset(0 as isize))) != 0)
                || ((isinf!(*value.offset(1 as isize))) != 0) || ((isnan!(*value.offset(1 as isize))) != 0)
                || ((isinf!(*value.offset(2 as isize))) != 0) || ((isnan!(*value.offset(2 as isize))) != 0)
                || ((isinf!(*value.offset(3 as isize))) != 0) || ((isnan!(*value.offset(3 as isize))) != 0)
            {
                (*shader).lconst_inf_or_nan = TRUE as BOOL;
            }
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DEFI as ::core::ffi::c_uint)
        {
            let mut lconst: *mut wined3d_shader_lconst = core::ptr::null_mut();

            lconst = malloc(core::mem::size_of::<wined3d_shader_lconst>() as size_t) as *mut wined3d_shader_lconst;
            if lconst.is_null() {
                return E_OUTOFMEMORY;
            }

            (*lconst).idx = (*ins.dst.offset(0 as isize)).reg.idx[0 as usize].offset;
            memcpy(&raw mut (*lconst).value as *mut ::core::ffi::c_uint as *mut ::core::ffi::c_void,
                &raw const (*ins.src.offset(0 as isize)).reg.u.immconst_data as *const ::core::ffi::c_uint as *const ::core::ffi::c_void,
                (4 as size_t).wrapping_mul(core::mem::size_of::<DWORD>() as size_t));

            list_add_head(&raw mut (*shader).constantsI, &raw mut (*lconst).entry);
            (*reg_maps).local_int_consts = ((*reg_maps).local_int_consts as ::core::ffi::c_uint
                | ((1 as ::core::ffi::c_uint) << (*lconst).idx)) as WORD;
        }
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DEFB as ::core::ffi::c_uint)
        {
            let mut lconst: *mut wined3d_shader_lconst = core::ptr::null_mut();

            lconst = malloc(core::mem::size_of::<wined3d_shader_lconst>() as size_t) as *mut wined3d_shader_lconst;
            if lconst.is_null() {
                return E_OUTOFMEMORY;
            }

            (*lconst).idx = (*ins.dst.offset(0 as isize)).reg.idx[0 as usize].offset;
            memcpy(&raw mut (*lconst).value as *mut ::core::ffi::c_uint as *mut ::core::ffi::c_void,
                &raw const (*ins.src.offset(0 as isize)).reg.u.immconst_data as *const ::core::ffi::c_uint as *const ::core::ffi::c_void,
                core::mem::size_of::<DWORD>() as size_t);

            list_add_head(&raw mut (*shader).constantsB, &raw mut (*lconst).entry);
            (*reg_maps).local_bool_consts = ((*reg_maps).local_bool_consts as ::core::ffi::c_uint
                | ((1 as ::core::ffi::c_uint) << (*lconst).idx)) as WORD;
        }
        /* Handle shader phases. */
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_HS_CONTROL_POINT_PHASE as ::core::ffi::c_uint)
            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_HS_FORK_PHASE as ::core::ffi::c_uint)
            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_HS_JOIN_PHASE as ::core::ffi::c_uint)
        {
            hr = shader_record_shader_phase(shader, &raw mut phase, &raw mut ins, current_ins, prev_ins);
            if hr < (0 as HRESULT) {
                return hr;
            }
        }
        /* For subroutine prototypes. */
        else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LABEL as ::core::ffi::c_uint)
        {
            (*reg_maps).labels = ((*reg_maps).labels as ::core::ffi::c_uint
                | ((1 as ::core::ffi::c_uint)
                    << (*ins.src.offset(0 as isize)).reg.idx[0 as usize].offset)) as WORD;
        }
        /* Set texture, address, temporary registers. */
        else
        {
            let mut color0_mov: BOOL = FALSE as BOOL;
            let mut i: ::core::ffi::c_uint = 0;

            /* This will loop over all the registers and try to
             * make a bitmask of the ones we're interested in.
             *
             * Relative addressing tokens are ignored, but that's
             * okay, since we'll catch any address registers when
             * they are initialized (required by spec). */
            i = 0 as ::core::ffi::c_uint;
            while i < ins.dst_count {
                'body: {
                    if shader_record_register_usage(shader, reg_maps, &raw const (*ins.dst.offset(i as isize)).reg,
                        shader_version.type_, constf_size) == 0
                    {
                        return WINED3DERR_INVALIDCALL;
                    }

                    if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_uint) {
                        let mut idx: UINT = (*ins.dst.offset(i as isize)).reg.idx[0 as usize].offset as UINT;

                        'sw1: {
                            match (*ins.dst.offset(i as isize)).reg.type_ as ::core::ffi::c_uint {
                                WINED3DSPR_RASTOUT => {
                                    if (shader_version.major as ::core::ffi::c_int) >= (3 as ::core::ffi::c_int) {
                                        break 'sw1;
                                    }
                                    match idx {
                                        0 => { /* oPos */
                                            (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << 10;
                                            shader_signature_from_usage(&raw mut output_signature_elements[10 as usize],
                                                WINED3D_DECL_USAGE_POSITION, 0, 10, WINED3DSP_WRITEMASK_ALL);
                                        }
                                        1 => { /* oFog */
                                            (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << 11;
                                            shader_signature_from_usage(&raw mut output_signature_elements[11 as usize],
                                                WINED3D_DECL_USAGE_FOG, 0, 11, WINED3DSP_WRITEMASK_0);
                                        }
                                        2 => { /* oPts */
                                            (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << 11;
                                            shader_signature_from_usage(&raw mut output_signature_elements[12 as usize],
                                                WINED3D_DECL_USAGE_PSIZE, 0, 11, WINED3DSP_WRITEMASK_1);
                                        }
                                        _ => {}
                                    }
                                    break 'sw1;
                                }
                                WINED3DSPR_ATTROUT => {
                                    if (shader_version.major as ::core::ffi::c_int) >= (3 as ::core::ffi::c_int) {
                                        break 'sw1;
                                    }
                                    if idx < (2 as UINT) {
                                        idx = idx.wrapping_add(8 as UINT);
                                        if ((*reg_maps).output_registers & ((1 as ::core::ffi::c_uint) << idx)) != 0 {
                                            output_signature_elements[idx as usize].mask |= (*ins.dst.offset(i as isize)).write_mask;
                                        }
                                        else {
                                            (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << idx;
                                            shader_signature_from_usage(&raw mut output_signature_elements[idx as usize],
                                                WINED3D_DECL_USAGE_COLOR, idx.wrapping_sub(8 as UINT), idx,
                                                (*ins.dst.offset(i as isize)).write_mask as DWORD);
                                        }
                                    }
                                    break 'sw1;
                                }
                                WINED3DSPR_TEXCRDOUT => { /* WINED3DSPR_OUTPUT */
                                    if (shader_version.major as ::core::ffi::c_int) >= (3 as ::core::ffi::c_int) {
                                        if idx >= ARRAY_SIZE!((*reg_maps).u.output_registers_mask) as UINT {
                                            WARN!("Invalid output register index %u.\n", idx);
                                            break 'sw1;
                                        }
                                        (*reg_maps).u.output_registers_mask[idx as usize] =
                                            (((*reg_maps).u.output_registers_mask[idx as usize] as uint32_t)
                                                | (*ins.dst.offset(i as isize)).write_mask) as BYTE;
                                        break 'sw1;
                                    }
                                    if idx >= ARRAY_SIZE!((*reg_maps).u.texcoord_mask) as UINT {
                                        WARN!("Invalid texcoord index %u.\n", idx);
                                        break 'sw1;
                                    }
                                    (*reg_maps).u.texcoord_mask[idx as usize] |= (*ins.dst.offset(i as isize)).write_mask as ::core::ffi::c_uint;
                                    if ((*reg_maps).output_registers & ((1 as ::core::ffi::c_uint) << idx)) != 0 {
                                        output_signature_elements[idx as usize].mask |= (*ins.dst.offset(i as isize)).write_mask;
                                    }
                                    else {
                                        (*reg_maps).output_registers |= (1 as ::core::ffi::c_uint) << idx;
                                        shader_signature_from_usage(&raw mut output_signature_elements[idx as usize],
                                            WINED3D_DECL_USAGE_TEXCOORD, idx, idx, (*ins.dst.offset(i as isize)).write_mask as DWORD);
                                    }
                                    break 'sw1;
                                }
                                _ => {
                                    break 'sw1;
                                }
                            }
                        }
                    }

                    if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint) {
                        if (*ins.dst.offset(i as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_COLOROUT as ::core::ffi::c_uint)
                            && (*ins.dst.offset(i as isize)).reg.idx[0 as usize].offset == 0
                        {
                            /* Many 2.0 and 3.0 pixel shaders end with a MOV from a temp register to
                             * COLOROUT 0. If we know this in advance, the ARB shader backend can skip
                             * the mov and perform the sRGB write correction from the source register.
                             *
                             * However, if the mov is only partial, we can't do this, and if the write
                             * comes from an instruction other than MOV it is hard to do as well. If
                             * COLOROUT 0 is overwritten partially later, the marker is dropped again. */
                            (*shader).u.ps.color0_mov = FALSE as BOOL;
                            if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_MOV as ::core::ffi::c_uint)
                                && (*ins.dst.offset(i as isize)).write_mask == (WINED3DSP_WRITEMASK_ALL as uint32_t)
                            {
                                /* Used later when the source register is read. */
                                color0_mov = TRUE as BOOL;
                            }
                        }
                        /* Also drop the MOV marker if the source register is overwritten prior to the shader
                         * end
                         */
                        else if (*ins.dst.offset(i as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_TEMP as ::core::ffi::c_uint)
                            && (*ins.dst.offset(i as isize)).reg.idx[0 as usize].offset == (*shader).u.ps.color0_reg
                        {
                            (*shader).u.ps.color0_mov = FALSE as BOOL;
                        }
                    }

                    /* Declare 1.x samplers implicitly, based on the destination reg. number. */
                    if (shader_version.major as ::core::ffi::c_int) == (1 as ::core::ffi::c_int)
                        && ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEX as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXBEM as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXBEML as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXDP3TEX as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXM3x2TEX as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXM3x3SPEC as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXM3x3TEX as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXM3x3VSPEC as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXREG2AR as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXREG2GB as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXREG2RGB as ::core::ffi::c_uint))
                    {
                        let mut reg_idx: ::core::ffi::c_uint = (*ins.dst.offset(i as isize)).reg.idx[0 as usize].offset;

                        if reg_idx >= ARRAY_SIZE!((*reg_maps).resource_info) as ::core::ffi::c_uint {
                            WARN!("Invalid 1.x sampler index %u.\n", reg_idx);
                            break 'body;
                        }

                        (*reg_maps).resource_info[reg_idx as usize].type_ = WINED3D_SHADER_RESOURCE_TEXTURE_2D;
                        (*reg_maps).resource_info[reg_idx as usize].data_type = WINED3D_DATA_FLOAT;
                        shader_record_sample(reg_maps, reg_idx, reg_idx, reg_idx);
                        wined3d_bitmap_set(&raw mut (*reg_maps).resource_map as *mut uint32_t, reg_idx);

                        /* texbem is only valid with < 1.4 pixel shaders */
                        if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXBEM as ::core::ffi::c_uint)
                            || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXBEML as ::core::ffi::c_uint)
                        {
                            (*reg_maps).bumpmat = ((*reg_maps).bumpmat as ::core::ffi::c_uint
                                | ((1 as ::core::ffi::c_uint) << reg_idx)) as BYTE;
                            if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXBEML as ::core::ffi::c_uint) {
                                (*reg_maps).luminanceparams = ((*reg_maps).luminanceparams as ::core::ffi::c_uint
                                    | ((1 as ::core::ffi::c_uint) << reg_idx)) as BYTE;
                            }
                        }
                    }
                    else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_BEM as ::core::ffi::c_uint)
                    {
                        (*reg_maps).bumpmat = ((*reg_maps).bumpmat as ::core::ffi::c_uint
                            | ((1 as ::core::ffi::c_uint)
                                << (*ins.dst.offset(i as isize)).reg.idx[0 as usize].offset)) as BYTE;
                    }
                }
                i = i.wrapping_add(1);
            }

            if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_IMM_ATOMIC_ALLOC as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_IMM_ATOMIC_CONSUME as ::core::ffi::c_uint)
            {
                let mut reg_idx: ::core::ffi::c_uint = (*ins.src.offset(0 as isize)).reg.idx[0 as usize].offset;
                if reg_idx >= (MAX_UNORDERED_ACCESS_VIEWS as ::core::ffi::c_uint) {
                    ERR!("Invalid UAV index %u.\n", reg_idx);
                    break;
                }
                (*reg_maps).set_uav_counter_mask((*reg_maps).uav_counter_mask()
                    | (((1 as ::core::ffi::c_uint) << reg_idx) as DWORD));
            }
            else if ((WINED3DSIH_ATOMIC_AND as ::core::ffi::c_uint) <= (ins.handler_idx as ::core::ffi::c_uint)
                    && (ins.handler_idx as ::core::ffi::c_uint) <= (WINED3DSIH_ATOMIC_XOR as ::core::ffi::c_uint))
                || ((WINED3DSIH_IMM_ATOMIC_AND as ::core::ffi::c_uint) <= (ins.handler_idx as ::core::ffi::c_uint)
                    && (ins.handler_idx as ::core::ffi::c_uint) <= (WINED3DSIH_IMM_ATOMIC_XOR as ::core::ffi::c_uint))
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_BUFINFO as ::core::ffi::c_uint)
                    && (*ins.src.offset(0 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_UAV as ::core::ffi::c_uint))
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_UAV_TYPED as ::core::ffi::c_uint)
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_RAW as ::core::ffi::c_uint)
                    && (*ins.src.offset(1 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_UAV as ::core::ffi::c_uint))
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_STRUCTURED as ::core::ffi::c_uint)
                    && (*ins.src.offset(2 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_UAV as ::core::ffi::c_uint))
            {
                let mut reg: *const wined3d_shader_register = core::ptr::null();

                if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_UAV_TYPED as ::core::ffi::c_uint)
                    || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_RAW as ::core::ffi::c_uint)
                {
                    reg = &raw const (*ins.src.offset(1 as isize)).reg;
                }
                else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_STRUCTURED as ::core::ffi::c_uint)
                {
                    reg = &raw const (*ins.src.offset(2 as isize)).reg;
                }
                else if (WINED3DSIH_ATOMIC_AND as ::core::ffi::c_uint) <= (ins.handler_idx as ::core::ffi::c_uint)
                    && (ins.handler_idx as ::core::ffi::c_uint) <= (WINED3DSIH_ATOMIC_XOR as ::core::ffi::c_uint)
                {
                    reg = &raw const (*ins.dst.offset(0 as isize)).reg;
                }
                else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_BUFINFO as ::core::ffi::c_uint)
                {
                    reg = &raw const (*ins.src.offset(0 as isize)).reg;
                }
                else
                {
                    reg = &raw const (*ins.dst.offset(1 as isize)).reg;
                }

                if (*reg).type_ as ::core::ffi::c_uint == (WINED3DSPR_UAV as ::core::ffi::c_uint)
                {
                    if (*reg).idx[0 as usize].offset >= (MAX_UNORDERED_ACCESS_VIEWS as ::core::ffi::c_uint)
                    {
                        ERR!("Invalid UAV index %u.\n", (*reg).idx[0 as usize].offset);
                        break;
                    }
                    (*reg_maps).set_uav_read_mask((*reg_maps).uav_read_mask()
                        | (((1 as ::core::ffi::c_uint) << (*reg).idx[0 as usize].offset) as DWORD));
                }
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_NRM as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usesnrm(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSY as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSY_COARSE as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSY_FINE as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usesdsy(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSX as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSX_COARSE as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_DSX_FINE as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usesdsx(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXLDD as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usestexldd(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_TEXLDL as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usestexldl(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_MOVA as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usesmova(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_IFC as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usesifc(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_CALL as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usescall(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_POW as ::core::ffi::c_uint)
            {
                (*reg_maps).set_usespow(1 as DWORD);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LOOP as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_REP as ::core::ffi::c_uint)
            {
                cur_loop_depth = cur_loop_depth.wrapping_add(1);
                if cur_loop_depth > max_loop_depth {
                    max_loop_depth = cur_loop_depth;
                }
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_ENDLOOP as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_ENDREP as ::core::ffi::c_uint)
            {
                cur_loop_depth = cur_loop_depth.wrapping_sub(1);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_GATHER4 as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_GATHER4_C as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_B as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_C as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_C_LZ as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_GRAD as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_LOD as ::core::ffi::c_uint)
            {
                shader_record_sample(reg_maps,
                    (*ins.src.offset(1 as isize)).reg.idx[0 as usize].offset,
                    (*ins.src.offset(2 as isize)).reg.idx[0 as usize].offset,
                    (*reg_maps).sampler_map.count as ::core::ffi::c_uint);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_GATHER4_PO as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_GATHER4_PO_C as ::core::ffi::c_uint)
            {
                shader_record_sample(reg_maps,
                    (*ins.src.offset(2 as isize)).reg.idx[0 as usize].offset,
                    (*ins.src.offset(3 as isize)).reg.idx[0 as usize].offset,
                    (*reg_maps).sampler_map.count as ::core::ffi::c_uint);
            }
            else if ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_BUFINFO as ::core::ffi::c_uint)
                    && (*ins.src.offset(0 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_RESOURCE as ::core::ffi::c_uint))
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_SAMPLE_INFO as ::core::ffi::c_uint)
                    && (*ins.src.offset(0 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_RESOURCE as ::core::ffi::c_uint))
            {
                shader_record_sample(reg_maps,
                    (*ins.src.offset(0 as isize)).reg.idx[0 as usize].offset,
                    WINED3D_SAMPLER_DEFAULT as ::core::ffi::c_uint,
                    (*reg_maps).sampler_map.count as ::core::ffi::c_uint);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD as ::core::ffi::c_uint)
                || (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD2DMS as ::core::ffi::c_uint)
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_RAW as ::core::ffi::c_uint)
                    && (*ins.src.offset(1 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_RESOURCE as ::core::ffi::c_uint))
                || ((ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_RESINFO as ::core::ffi::c_uint)
                    && (*ins.src.offset(1 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_RESOURCE as ::core::ffi::c_uint))
            {
                shader_record_sample(reg_maps,
                    (*ins.src.offset(1 as isize)).reg.idx[0 as usize].offset,
                    WINED3D_SAMPLER_DEFAULT as ::core::ffi::c_uint,
                    (*reg_maps).sampler_map.count as ::core::ffi::c_uint);
            }
            else if (ins.handler_idx as ::core::ffi::c_uint) == (WINED3DSIH_LD_STRUCTURED as ::core::ffi::c_uint)
                && (*ins.src.offset(2 as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_RESOURCE as ::core::ffi::c_uint)
            {
                shader_record_sample(reg_maps,
                    (*ins.src.offset(2 as isize)).reg.idx[0 as usize].offset,
                    WINED3D_SAMPLER_DEFAULT as ::core::ffi::c_uint,
                    (*reg_maps).sampler_map.count as ::core::ffi::c_uint);
            }

            if !ins.predicate.is_null() {
                if shader_record_register_usage(shader, reg_maps, &raw const (*ins.predicate).reg,
                    shader_version.type_, constf_size) == 0
                {
                    return WINED3DERR_INVALIDCALL;
                }
            }

            i = 0 as ::core::ffi::c_uint;
            while i < ins.src_count {
                let mut count: ::core::ffi::c_uint = get_instr_extra_regcount(ins.handler_idx, i);
                let mut reg: wined3d_shader_register = (*ins.src.offset(i as isize)).reg;

                if shader_record_register_usage(shader, reg_maps, &raw const (*ins.src.offset(i as isize)).reg,
                    shader_version.type_, constf_size) == 0
                {
                    return WINED3DERR_INVALIDCALL;
                }
                while count != 0 {
                    reg.idx[0 as usize].offset = reg.idx[0 as usize].offset.wrapping_add(1);
                    if shader_record_register_usage(shader, reg_maps, &raw mut reg,
                        shader_version.type_, constf_size) == 0
                    {
                        return WINED3DERR_INVALIDCALL;
                    }
                    count = count.wrapping_sub(1);
                }

                if color0_mov != 0 {
                    if (*ins.src.offset(i as isize)).reg.type_ as ::core::ffi::c_uint == (WINED3DSPR_TEMP as ::core::ffi::c_uint)
                        && (*ins.src.offset(i as isize)).swizzle == (WINED3DSP_NOSWIZZLE as _)
                    {
                        (*shader).u.ps.color0_mov = TRUE as BOOL;
                        (*shader).u.ps.color0_reg = (*ins.src.offset(i as isize)).reg.idx[0 as usize].offset;
                    }
                }
                i = i.wrapping_add(1);
            }
        }

        prev_ins = current_ins;
    }
    (*reg_maps).loop_depth = max_loop_depth;

    if !phase.is_null() {
        (*phase).end = prev_ins;
        phase = core::ptr::null_mut();
    }

    /* PS before 2.0 don't have explicit color outputs. Instead the value of
     * R0 is written to the render target. */
    if ((shader_version.major as ::core::ffi::c_int) < (2 as ::core::ffi::c_int))
        && (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint)
    {
        (*reg_maps).rt_mask |= (1 as ::core::ffi::c_uint) << 0;
    }

    if !(*input_signature).elements.is_null() {
        i = 0 as ::core::ffi::c_uint;
        while i < (*input_signature).element_count {
            if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_VERTEX as ::core::ffi::c_uint) {
                if (*(*input_signature).elements.offset(i as isize)).register_idx as usize
                    >= ARRAY_SIZE!((*shader).u.vs.attributes) as usize
                {
                    WARN!("Invalid input signature register index %u.\n",
                        (*(*input_signature).elements.offset(i as isize)).register_idx);
                    return WINED3DERR_INVALIDCALL;
                }
            }
            else if (shader_version.type_ as ::core::ffi::c_uint) == (WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_uint) {
                if (*(*input_signature).elements.offset(i as isize)).sysval_semantic as ::core::ffi::c_uint
                    == (WINED3D_SV_POSITION as ::core::ffi::c_uint)
                {
                    (*reg_maps).set_vpos(1 as DWORD);
                }
                else if (*(*input_signature).elements.offset(i as isize)).sysval_semantic as ::core::ffi::c_uint
                    == (WINED3D_SV_IS_FRONT_FACE as ::core::ffi::c_uint)
                {
                    (*reg_maps).set_usesfacing(1 as DWORD);
                }
            }
            (*reg_maps).input_registers |= (1 as ::core::ffi::c_uint)
                << (*(*input_signature).elements.offset(i as isize)).register_idx;
            i = i.wrapping_add(1);
        }
    }
    else if (*input_signature).elements.is_null() && (*reg_maps).input_registers != 0 {
        let mut count: ::core::ffi::c_uint = wined3d_popcount((*reg_maps).input_registers);
        let mut e: *mut wined3d_shader_signature_element = core::ptr::null_mut();
        let mut i: ::core::ffi::c_uint = 0;

        (*input_signature).elements = calloc((count as size_t) as _,
            (core::mem::size_of::<wined3d_shader_signature_element>() as size_t) as _) as *mut wined3d_shader_signature_element;
        if (*input_signature).elements.is_null() {
            return E_OUTOFMEMORY;
        }
        (*input_signature).element_count = count;

        e = (*input_signature).elements;
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(input_signature_elements) as usize {
            'ibody: {
                if ((*reg_maps).input_registers & ((1 as ::core::ffi::c_uint) << i)) == 0 {
                    break 'ibody;
                }
                input_signature_elements[i as usize].register_idx = i;
                *e = input_signature_elements[i as usize];
                e = e.offset(1);
            }
            i = i.wrapping_add(1);
        }
    }

    if !(*output_signature).elements.is_null() {
        hr = shader_scan_output_signature(shader);
        if hr < (0 as HRESULT) {
            return hr;
        }
    }
    else if (*reg_maps).output_registers != 0 {
        let mut e: *mut wined3d_shader_signature_element = core::ptr::null_mut();
        let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(output_signature_elements) as usize {
            if !output_signature_elements[i as usize].semantic_name.is_null() {
                count = count.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        (*output_signature).elements = calloc((count as size_t) as _,
            (core::mem::size_of::<wined3d_shader_signature_element>() as size_t) as _) as *mut wined3d_shader_signature_element;
        if (*output_signature).elements.is_null() {
            return E_OUTOFMEMORY;
        }
        (*output_signature).element_count = count;

        e = (*output_signature).elements;
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(output_signature_elements) as usize {
            if !output_signature_elements[i as usize].semantic_name.is_null() {
                *e = output_signature_elements[i as usize];
                e = e.offset(1);
            }
            i = i.wrapping_add(1);
        }
    }

    return WINED3D_OK;
}

unsafe extern "C" fn shader_cleanup_reg_maps(reg_maps: *mut wined3d_shader_reg_maps) {
    let mut reg: *mut wined3d_shader_indexable_temp = core::ptr::null_mut();
    let mut reg_next: *mut wined3d_shader_indexable_temp = core::ptr::null_mut();

    free((*reg_maps).constf as *mut core::ffi::c_void);
    free((*reg_maps).sampler_map.entries as *mut core::ffi::c_void);

    reg = ((*reg_maps).indexable_temps.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_shader_indexable_temp, entry) as isize))
        as *mut wined3d_shader_indexable_temp;
    reg_next = ((*reg).entry.next as *mut u8)
        .offset(-(core::mem::offset_of!(wined3d_shader_indexable_temp, entry) as isize))
        as *mut wined3d_shader_indexable_temp;
    while core::ptr::addr_of_mut!((*reg).entry) != core::ptr::addr_of_mut!((*reg_maps).indexable_temps) {
        free(reg as *mut core::ffi::c_void);
        reg = reg_next;
        reg_next = ((*reg).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(wined3d_shader_indexable_temp, entry) as isize))
            as *mut wined3d_shader_indexable_temp;
    }

    list_init(core::ptr::addr_of_mut!((*reg_maps).indexable_temps));

    free((*reg_maps).tgsm as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn shader_find_free_input_register(
    reg_maps: *const wined3d_shader_reg_maps,
    max: ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut map: DWORD = 1 << max;
    map |= map.wrapping_sub(1);
    map &= if ((*reg_maps).shader_version.major as ::core::ffi::c_int) < 3 {
        !((*reg_maps).texcoord as DWORD)
    } else {
        !(*reg_maps).input_registers
    };

    return wined3d_log2i(map as ::core::ffi::c_uint);
}
#[no_mangle]
pub unsafe extern "C" fn shader_generate_code(
    shader: *const wined3d_shader,
    buffer: *mut wined3d_string_buffer,
    reg_maps: *const wined3d_shader_reg_maps,
    backend_ctx: *mut ::core::ffi::c_void,
    start: *const DWORD,
    end: *const DWORD,
) -> HRESULT {
    let device: *mut wined3d_device = (*shader).device;
    let fe: *const wined3d_shader_frontend = (*shader).frontend;
    let fe_data: *mut ::core::ffi::c_void = (*shader).frontend_data;
    let mut shader_version: wined3d_shader_version = core::mem::zeroed();
    let mut state: wined3d_shader_parser_state = core::mem::zeroed();
    let mut ins: wined3d_shader_instruction = core::mem::zeroed();
    let mut tex_mx: wined3d_shader_tex_mx = core::mem::zeroed();
    let mut ctx: wined3d_shader_context = core::mem::zeroed();
    let mut ptr: *const DWORD = core::ptr::null();

    /* Initialize current parsing state. */
    tex_mx.current_row = 0;
    state.current_loop_depth = 0;
    state.current_loop_reg = 0;
    state.in_subroutine = (FALSE as i32);

    ctx.shader = shader;
    ctx.reg_maps = reg_maps;
    ctx.buffer = buffer;
    ctx.tex_mx = core::ptr::addr_of_mut!(tex_mx);
    ctx.state = core::ptr::addr_of_mut!(state);
    ctx.backend_data = backend_ctx;
    ins.ctx = core::ptr::addr_of_mut!(ctx);

    (*fe).shader_read_header.unwrap_unchecked()(
        fe_data,
        core::ptr::addr_of_mut!(ptr),
        core::ptr::addr_of_mut!(shader_version),
    );
    if !start.is_null() {
        ptr = start;
    }

    while (*fe).shader_is_end.unwrap_unchecked()(
        fe_data,
        core::ptr::addr_of_mut!(ptr),
    ) == 0
        && ptr != end
    {
        /* Read opcode. */
        (*fe).shader_read_instruction.unwrap_unchecked()(
            fe_data,
            core::ptr::addr_of_mut!(ptr),
            core::ptr::addr_of_mut!(ins),
        );

        /* Unknown opcode and its parameters. */
        if ins.handler_idx as ::core::ffi::c_uint == WINED3DSIH_TABLE_SIZE {
            WARN!("Encountered unrecognised or invalid instruction.\n");
            return WINED3DERR_INVALIDCALL;
        }

        if !ins.predicate.is_null() {
            FIXME!("Predicates not implemented.\n");
        }

        /* Call appropriate function for output target */
        (*(*device).shader_backend).shader_handle_instruction.unwrap_unchecked()(
            core::ptr::addr_of_mut!(ins),
        );
    }

    return WINED3D_OK;
}

unsafe extern "C" fn shader_cleanup(shader: *mut wined3d_shader) {
    if (*shader).reg_maps.shader_version.type_ as ::core::ffi::c_uint
        == WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_uint
    {
        free((*shader).u.hs.phases.control_point as *mut ::core::ffi::c_void);
        free((*shader).u.hs.phases.fork as *mut ::core::ffi::c_void);
        free((*shader).u.hs.phases.join as *mut ::core::ffi::c_void);
    }

    free((*shader).patch_constant_signature.elements as *mut ::core::ffi::c_void);
    free((*shader).output_signature.elements as *mut ::core::ffi::c_void);
    free((*shader).input_signature.elements as *mut ::core::ffi::c_void);
    (*(*(*shader).device).shader_backend)
        .shader_destroy
        .unwrap_unchecked()(shader);
    shader_cleanup_reg_maps(&raw mut (*shader).reg_maps);
    free((*shader).byte_code as *mut ::core::ffi::c_void);
    shader_delete_constant_list(&raw mut (*shader).constantsF);
    shader_delete_constant_list(&raw mut (*shader).constantsB);
    shader_delete_constant_list(&raw mut (*shader).constantsI);
    list_remove(&raw mut (*shader).shader_list_entry);

    if !(*shader).frontend.is_null() && !(*shader).frontend_data.is_null() {
        (*(*shader).frontend)
            .shader_free
            .unwrap_unchecked()((*shader).frontend_data);
    }
}

unsafe extern "C" fn shader_none_handle_instruction(ins: *const wined3d_shader_instruction) {}

unsafe extern "C" fn shader_none_precompile(
    shader_priv: *mut core::ffi::c_void,
    shader: *mut wined3d_shader,
) {}

unsafe extern "C" fn shader_none_apply_compute_state(
    shader_priv: *mut core::ffi::c_void,
    context: *mut wined3d_context,
    state: *const wined3d_state,
) {}

unsafe extern "C" fn shader_none_update_float_vertex_constants(
    device: *mut wined3d_device,
    start: UINT,
    count: UINT,
) {}

unsafe extern "C" fn shader_none_update_float_pixel_constants(
    device: *mut wined3d_device,
    start: UINT,
    count: UINT,
) {}

unsafe extern "C" fn shader_none_destroy(shader: *mut wined3d_shader) {}

unsafe extern "C" fn shader_none_free_context_data(context: *mut wined3d_context) {}

unsafe extern "C" fn shader_none_init_context_state(context: *mut wined3d_context) {}

unsafe extern "C" fn shader_none_apply_draw_state(
    mut shader_priv: *mut ::core::ffi::c_void,
    mut context: *mut wined3d_context,
    mut state: *const wined3d_state,
) {
    let mut priv_0: *mut shader_none_priv = shader_priv as *mut shader_none_priv;
    (*(*priv_0).vertex_pipe)
        .vp_apply_draw_state
        .unwrap_unchecked()(context, state);
    (*(*priv_0).fragment_pipe)
        .fp_apply_draw_state
        .unwrap_unchecked()(context, state);
}

unsafe extern "C" fn shader_none_disable(shader_priv: *mut ::core::ffi::c_void, context: *mut wined3d_context) {
    let mut priv_0: *mut shader_none_priv = shader_priv as *mut shader_none_priv;
    (*(*priv_0).vertex_pipe)
        .vp_disable
        .unwrap_unchecked()(context);
    (*(*priv_0).fragment_pipe)
        .fp_disable
        .unwrap_unchecked()(context);
    (*context).set_shader_update_mask(((((1 as DWORD) << (WINED3D_SHADER_TYPE_PIXEL as u32))
        | ((1 as DWORD) << (WINED3D_SHADER_TYPE_VERTEX as u32))
        | ((1 as DWORD) << (WINED3D_SHADER_TYPE_GEOMETRY as u32))
        | ((1 as DWORD) << (WINED3D_SHADER_TYPE_HULL as u32))
        | ((1 as DWORD) << (WINED3D_SHADER_TYPE_DOMAIN as u32))
        | ((1 as DWORD) << (WINED3D_SHADER_TYPE_COMPUTE as u32))) as DWORD) as _);
}

unsafe extern "C" fn shader_none_alloc(device: *mut wined3d_device, vertex_pipe: *const wined3d_vertex_pipe_ops, fragment_pipe: *const wined3d_fragment_pipe_ops) -> HRESULT {
    let mut vertex_priv: *mut core::ffi::c_void;
    let mut fragment_priv: *mut core::ffi::c_void;
    let mut priv_: *mut shader_none_priv;

    priv_ = malloc(core::mem::size_of::<shader_none_priv>() as usize) as *mut shader_none_priv;
    if priv_.is_null() {
        return E_OUTOFMEMORY;
    }

    vertex_priv = ((*vertex_pipe).vp_alloc.unwrap_unchecked())(
        core::ptr::addr_of!(none_shader_backend),
        priv_ as *mut core::ffi::c_void,
    );
    if vertex_priv.is_null() {
        ERR!("Failed to initialize vertex pipe.\n");
        free(priv_ as *mut core::ffi::c_void);
        return E_FAIL;
    }

    fragment_priv = ((*fragment_pipe).alloc_private.unwrap_unchecked())(
        core::ptr::addr_of!(none_shader_backend),
        priv_ as *mut core::ffi::c_void,
    );
    if fragment_priv.is_null() {
        ERR!("Failed to initialize fragment pipe.\n");
        ((*vertex_pipe).vp_free.unwrap_unchecked())(
            device,
            core::ptr::null_mut(),
        );
        free(priv_ as *mut core::ffi::c_void);
        return E_FAIL;
    }

    (*priv_).vertex_pipe = vertex_pipe;
    (*priv_).fragment_pipe = fragment_pipe;

    (*device).vertex_priv = vertex_priv;
    (*device).fragment_priv = fragment_priv;
    (*device).shader_priv = priv_ as *mut core::ffi::c_void;

    return WINED3D_OK;
}

unsafe extern "C" fn shader_none_free(
    device: *mut wined3d_device,
    context: *mut wined3d_context,
) {
    let mut priv_0: *mut shader_none_priv = (*device).shader_priv as *mut shader_none_priv;
    ((*(*priv_0).fragment_pipe).free_private)
        .unwrap_unchecked()(device, context);
    ((*(*priv_0).vertex_pipe).vp_free)
        .unwrap_unchecked()(device, context);
    free(priv_0 as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn shader_none_allocate_context_data(context: *mut wined3d_context) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn shader_none_get_caps(adapter: *const wined3d_adapter, caps: *mut shader_caps) {
    let _ = adapter;
    memset(caps as *mut core::ffi::c_void, 0, core::mem::size_of::<shader_caps>());
}

unsafe extern "C" fn shader_none_color_fixup_supported(fixup: color_fixup_desc) -> BOOL {
    (TRUE as i32)
}

unsafe extern "C" fn shader_none_shader_compile(
    context: *mut wined3d_context,
    shader_desc: *const wined3d_shader_desc,
    shader_type: wined3d_shader_type,
) -> uint64_t {
    0
}

unsafe extern "C" fn shader_max_version_from_feature_level(level: wined3d_feature_level) -> u32 {
    match level {
        WINED3D_FEATURE_LEVEL_11_1 | WINED3D_FEATURE_LEVEL_11 => {
            return 5;
        }
        WINED3D_FEATURE_LEVEL_10_1 | WINED3D_FEATURE_LEVEL_10 => {
            return 4;
        }
        WINED3D_FEATURE_LEVEL_9_3 => {
            return 3;
        }
        WINED3D_FEATURE_LEVEL_9_2 | WINED3D_FEATURE_LEVEL_9_1 => {
            return 2;
        }
        _ => {
            return 1;
        }
    }
}

unsafe extern "C" fn shader_find_signature_element(
    s: *const wined3d_shader_signature,
    stream_idx: u32,
    semantic_name: *const i8,
    semantic_idx: u32,
) -> *mut wined3d_shader_signature_element {
    let e: *mut wined3d_shader_signature_element = (*s).elements;
    let mut i: u32;
    i = 0 as u32;
    while i < (*s).element_count {
        if (*e.offset(i as isize)).stream_idx == stream_idx
            && stricmp((*e.offset(i as isize)).semantic_name, semantic_name) == 0
            && (*e.offset(i as isize)).semantic_idx == semantic_idx
        {
            return e.offset(i as isize) as *mut wined3d_shader_signature_element;
        }
        i = i.wrapping_add(1);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn shader_get_stream_output_register_info(
    shader: *const wined3d_shader,
    so_element: *const wined3d_stream_output_element,
    register_idx: *mut ::core::ffi::c_uint,
    component_idx: *mut ::core::ffi::c_uint,
) -> BOOL {
    let mut output: *const wined3d_shader_signature_element =
        core::ptr::null::<wined3d_shader_signature_element>();
    let mut idx: ::core::ffi::c_uint = 0;

    output = shader_find_signature_element(
        core::ptr::addr_of!((*shader).output_signature),
        (*so_element).stream_idx,
        (*so_element).semantic_name,
        (*so_element).semantic_idx,
    );
    if output.is_null() {
        return (FALSE) as BOOL;
    }

    idx = 0;
    while idx < 4 {
        if (*output).mask & (1 << idx) != 0 {
            break;
        }
        idx = idx.wrapping_add(1);
    }
    idx = idx.wrapping_add((*so_element).component_idx as ::core::ffi::c_uint);

    *register_idx = (*output).register_idx;
    *component_idx = idx;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn geometry_shader_init_so_desc(gs: *mut wined3d_geometry_shader, device: *mut wined3d_device, so_desc: *const wined3d_stream_output_desc) -> HRESULT {
    let mut s: *mut wined3d_so_desc_entry;
    let mut entry: *mut wine_rb_entry;
    let mut i: u32;
    let mut size: usize;
    let mut name: *mut ::core::ffi::c_char;

    entry = wine_rb_get(core::ptr::addr_of_mut!((*device).so_descs), so_desc as *const ::core::ffi::c_void) as *mut wine_rb_entry;
    if !entry.is_null() {
        (*gs).so_desc = core::ptr::addr_of_mut!(
            (*((entry as *mut u8).sub(core::mem::offset_of!(wined3d_so_desc_entry, entry)) as *mut wined3d_so_desc_entry)).desc
        );
        return WINED3D_OK;
    }

    size = core::mem::offset_of!(wined3d_so_desc_entry, elements)
        + ((*so_desc).element_count as usize) * core::mem::size_of::<wined3d_stream_output_element>();
    i = 0;
    while i < (*so_desc).element_count {
        let n: *const ::core::ffi::c_char = (*(*so_desc).elements.offset(i as isize)).semantic_name;

        if !n.is_null() {
            size = size.wrapping_add(strlen(n).wrapping_add(1));
        }
        i = i.wrapping_add(1);
    }
    s = malloc(size) as *mut wined3d_so_desc_entry;
    if s.is_null() {
        return E_OUTOFMEMORY;
    }

    (*s).desc = *so_desc;

    memcpy(
        core::ptr::addr_of_mut!((*s).elements) as *mut wined3d_stream_output_element as *mut ::core::ffi::c_void,
        (*so_desc).elements as *const ::core::ffi::c_void,
        ((*so_desc).element_count as usize).wrapping_mul(core::mem::size_of::<wined3d_stream_output_element>()),
    );
    (*s).desc.elements = core::ptr::addr_of_mut!((*s).elements) as *mut wined3d_stream_output_element;

    name = (core::ptr::addr_of_mut!((*s).elements) as *mut wined3d_stream_output_element)
        .offset((*s).desc.element_count as isize) as *mut ::core::ffi::c_char;

    i = 0;
    while i < (*so_desc).element_count {
        let e: *mut wined3d_stream_output_element =
            (core::ptr::addr_of_mut!((*s).elements) as *mut wined3d_stream_output_element).offset(i as isize);

        'body: {
            if (*e).semantic_name.is_null() {
                break 'body;
            }

            size = strlen((*e).semantic_name).wrapping_add(1);
            memcpy(
                name as *mut ::core::ffi::c_void,
                (*e).semantic_name as *const ::core::ffi::c_void,
                size,
            );
            (*e).semantic_name = name;
            name = name.offset(size as isize);
        }
        i = i.wrapping_add(1);
    }

    if wine_rb_put(
        core::ptr::addr_of_mut!((*device).so_descs),
        core::ptr::addr_of_mut!((*s).desc) as *const ::core::ffi::c_void,
        core::ptr::addr_of_mut!((*s).entry),
    ) == -1
    {
        free(s as *mut ::core::ffi::c_void);
        return E_FAIL;
    }
    (*gs).so_desc = core::ptr::addr_of_mut!((*s).desc);

    return WINED3D_OK;
}

unsafe extern "C" fn geometry_shader_init_stream_output(
    shader: *mut wined3d_shader,
    so_desc: *const wined3d_stream_output_desc,
) -> HRESULT {
    let fe: *const wined3d_shader_frontend = (*shader).frontend;
    let mut output: *const wined3d_shader_signature_element =
        core::ptr::null::<wined3d_shader_signature_element>();
    let mut i: c_uint;
    let mut component_idx: c_uint = core::mem::zeroed();
    let mut register_idx: c_uint = core::mem::zeroed();
    let mut mask: c_uint;
    let mut shader_version: wined3d_shader_version = wined3d_shader_version {
        type_: WINED3D_SHADER_TYPE_PIXEL,
        major: 0,
        minor: 0,
    };
    let mut ptr: *const DWORD = core::ptr::null::<DWORD>();
    let mut fe_data: *mut c_void;
    let mut hr: HRESULT;

    if so_desc.is_null() {
        return WINED3D_OK;
    }

    fe_data = (*fe).shader_init.unwrap_unchecked()(
        (*shader).function,
        (*shader).functionLength as size_t,
        core::ptr::addr_of_mut!((*shader).output_signature),
    );
    if fe_data.is_null() {
        WARN!("Failed to initialise frontend data.\n");
        return WINED3DERR_INVALIDCALL;
    }
    (*fe).shader_read_header.unwrap_unchecked()(
        fe_data,
        core::ptr::addr_of_mut!(ptr),
        core::ptr::addr_of_mut!(shader_version),
    );
    (*fe).shader_free.unwrap_unchecked()(fe_data);

    match shader_version.type_ as c_uint {
        WINED3D_SHADER_TYPE_VERTEX | WINED3D_SHADER_TYPE_DOMAIN => {
            (*shader).function = core::ptr::null::<DWORD>();
            (*shader).functionLength = 0;
        }
        WINED3D_SHADER_TYPE_GEOMETRY => {}
        _ => {
            WARN!(
                "Wrong shader type %s.\n",
                debug_shader_type(shader_version.type_)
            );
            return E_INVALIDARG;
        }
    }

    if (*shader).function.is_null() {
        (*shader).reg_maps.shader_version = shader_version;
        (*shader).reg_maps.shader_version.type_ = WINED3D_SHADER_TYPE_GEOMETRY;
        shader_set_limits(shader);
        hr = shader_scan_output_signature(shader);
        if hr < 0 {
            return hr;
        }
    }

    i = 0;
    while i < (*so_desc).element_count {
        let e: *const wined3d_stream_output_element =
            (*so_desc).elements.offset(i as isize) as *const wined3d_stream_output_element;

        if (*e).semantic_name.is_null() {
            i = i.wrapping_add(1);
            continue;
        }

        output = shader_find_signature_element(
            core::ptr::addr_of_mut!((*shader).output_signature),
            (*e).stream_idx,
            (*e).semantic_name,
            (*e).semantic_idx,
        );
        if output.is_null()
            || shader_get_stream_output_register_info(
                shader,
                e,
                core::ptr::addr_of_mut!(register_idx),
                core::ptr::addr_of_mut!(component_idx),
            ) == 0
        {
            WARN!("Failed to find output signature element for stream output entry.\n");
            return E_INVALIDARG;
        }

        mask = (wined3d_mask_from_size((*e).component_count as c_uint) << component_idx) as c_uint;
        if ((*output).mask & 0xff & mask as uint32_t) != mask as uint32_t {
            WARN!(
                "Invalid component range %u-%u (mask %#x), output mask %#x.\n",
                component_idx,
                (*e).component_count as c_int,
                mask,
                (*output).mask & 0xff
            );
            return E_INVALIDARG;
        }

        i = i.wrapping_add(1);
    }

    hr = geometry_shader_init_so_desc(
        core::ptr::addr_of_mut!((*shader).u.gs),
        (*shader).device,
        so_desc,
    );
    if hr < 0 {
        WARN!("Failed to initialise stream output description, hr %#lx.\n", hr);
        return hr;
    }

    WINED3D_OK
}

unsafe extern "C" fn shader_trace(
    mut code: *const ::core::ffi::c_void,
    mut size: size_t,
    mut source_type: vkd3d_shader_source_type,
) {
    let mut info: vkd3d_shader_compile_info = core::mem::zeroed();
    info.type_ = VKD3D_SHADER_STRUCTURE_TYPE_COMPILE_INFO;
    let mut d3d_asm: vkd3d_shader_code = core::mem::zeroed();
    let mut ptr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut messages: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = 0;
    static mut COMPILE_OPTIONS: [vkd3d_shader_compile_option; 1] = [vkd3d_shader_compile_option {
        name: VKD3D_SHADER_COMPILE_OPTION_API_VERSION,
        value: VKD3D_SHADER_API_VERSION_1_6,
    }];

    info.source.code = code;
    info.source.size = size;
    info.source_type = source_type;
    info.target_type = VKD3D_SHADER_TARGET_D3D_ASM;
    info.options = core::ptr::addr_of!(COMPILE_OPTIONS) as *const vkd3d_shader_compile_option;
    info.option_count = ARRAY_SIZE!(COMPILE_OPTIONS) as ::core::ffi::c_uint;
    info.log_level = VKD3D_SHADER_LOG_WARNING;

    ret = vkd3d_shader_compile(
        core::ptr::addr_of_mut!(info),
        core::ptr::addr_of_mut!(d3d_asm),
        core::ptr::addr_of_mut!(messages),
    );
    if !messages.is_null() && *messages as ::core::ffi::c_int != 0 && ((FIXME_ON!(__wine_dbch_d3d_shader)) != 0) {
        ptr = messages;
        end = ptr.offset(strlen(ptr as *const ::core::ffi::c_char) as isize);
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
        ERR!("Failed to disassemble, ret %d.\n", ret);
        return;
    }

    ptr = d3d_asm.code as *const ::core::ffi::c_char;
    end = ptr.offset(d3d_asm.size as isize);
    loop {
        line = wined3d_get_line(core::ptr::addr_of_mut!(ptr), end);
        if line.is_null() {
            break;
        }
        TRACE!(
            "    %.*s",
            ptr.offset_from(line) as ::core::ffi::c_int,
            line
        );
    }
    TRACE!("\n");

    vkd3d_shader_free_shader_code(core::ptr::addr_of_mut!(d3d_asm));
}

unsafe extern "C" fn shader_set_function(
    shader: *mut wined3d_shader,
    desc: *const wined3d_shader_desc,
    type_0: wined3d_shader_type,
    so_desc: *const wined3d_stream_output_desc,
    float_const_count: ::core::ffi::c_uint,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*(*shader).device).adapter).d3d_info);
    let reg_maps: *mut wined3d_shader_reg_maps = core::ptr::addr_of_mut!((*shader).reg_maps);
    let version: *const wined3d_shader_version = core::ptr::addr_of!((*reg_maps).shader_version);
    let mut highest_reg_used: ::core::ffi::c_uint = 0;
    let mut num_regs_used: ::core::ffi::c_uint = 0;
    let mut fe: *const wined3d_shader_frontend = core::ptr::null::<wined3d_shader_frontend>();
    let mut backend_version: ::core::ffi::c_uint;
    let mut hr: HRESULT;

    TRACE!(
        "shader %p, byte_code %p, size %#Ix, type %s, float_const_count %u.\n",
        shader,
        (*desc).byte_code,
        (*desc).byte_code_size,
        debug_shader_type(type_0),
        float_const_count
    );

    if (*desc).byte_code.is_null() {
        return WINED3DERR_INVALIDCALL;
    }

    if (*desc).byte_code_size == !(0 as usize) {
        let mut shader_version: wined3d_shader_version = core::mem::zeroed();
        let mut fe_0: *const wined3d_shader_frontend = core::ptr::null::<wined3d_shader_frontend>();
        let mut ins: wined3d_shader_instruction = core::mem::zeroed();
        let mut ptr: *const DWORD = core::ptr::null::<DWORD>();
        let mut fe_data: *mut ::core::ffi::c_void = core::ptr::null_mut::<::core::ffi::c_void>();

        (*shader).source_type = VKD3D_SHADER_SOURCE_D3D_BYTECODE;
        (*shader).frontend = shader_select_frontend((*shader).source_type);
        if (*shader).frontend.is_null() {
            FIXME!("Unable to find frontend for shader.\n");
            shader_cleanup(shader);
            return WINED3DERR_INVALIDCALL;
        }
        fe_0 = (*shader).frontend;
        fe_data = (*fe_0).shader_init.unwrap_unchecked()(
            (*desc).byte_code,
            (*desc).byte_code_size,
            core::ptr::addr_of_mut!((*shader).output_signature),
        );
        if fe_data.is_null() {
            WARN!("Failed to initialise frontend data.\n");
            shader_cleanup(shader);
            return WINED3DERR_INVALIDCALL;
        }

        (*fe_0).shader_read_header.unwrap_unchecked()(
            fe_data,
            core::ptr::addr_of_mut!(ptr),
            core::ptr::addr_of_mut!(shader_version),
        );
        while (*fe_0).shader_is_end.unwrap_unchecked()(
            fe_data,
            core::ptr::addr_of_mut!(ptr),
        ) == 0
        {
            (*fe_0).shader_read_instruction.unwrap_unchecked()(
                fe_data,
                core::ptr::addr_of_mut!(ptr),
                core::ptr::addr_of_mut!(ins),
            );
        }

        (*fe_0).shader_free.unwrap_unchecked()(fe_data);

        (*shader).byte_code_size = (ptr.offset_from((*desc).byte_code) as ::core::ffi::c_longlong as usize)
            .wrapping_mul(core::mem::size_of::<DWORD>())
            as ::core::ffi::c_uint;

        (*shader).byte_code = malloc((*shader).byte_code_size as usize);
        if (*shader).byte_code.is_null() {
            shader_cleanup(shader);
            return E_OUTOFMEMORY;
        }
        memcpy(
            (*shader).byte_code,
            (*desc).byte_code as *const ::core::ffi::c_void,
            (*shader).byte_code_size as usize,
        );

        (*shader).function = (*shader).byte_code as *const DWORD;
        (*shader).functionLength = (*shader).byte_code_size;
    } else {
        let mut max_version: ::core::ffi::c_uint = 0;

        (*shader).byte_code = malloc((*desc).byte_code_size);
        if (*shader).byte_code.is_null() {
            shader_cleanup(shader);
            return E_OUTOFMEMORY;
        }
        memcpy(
            (*shader).byte_code,
            (*desc).byte_code as *const ::core::ffi::c_void,
            (*desc).byte_code_size,
        );
        (*shader).byte_code_size = (*desc).byte_code_size as ::core::ffi::c_uint;

        max_version = shader_max_version_from_feature_level((*(*(*(*shader).device).cs).c.state).feature_level);
        hr = wined3d_shader_extract_from_dxbc(shader, max_version, core::ptr::addr_of_mut!((*shader).source_type));
        if hr < 0 {
            shader_cleanup(shader);
            return hr;
        }

        (*shader).frontend = shader_select_frontend((*shader).source_type);
        if (*shader).frontend.is_null() {
            FIXME!("Unable to find frontend for shader.\n");
            shader_cleanup(shader);
            return WINED3DERR_INVALIDCALL;
        }
    }

    if ((TRACE_ON!(__wine_dbch_d3d_shader)) != 0) {
        if (*shader).source_type == VKD3D_SHADER_SOURCE_D3D_BYTECODE {
            shader_trace(
                (*shader).function as *const ::core::ffi::c_void,
                (*shader).functionLength as usize,
                (*shader).source_type,
            );
        } else {
            shader_trace(
                (*shader).byte_code,
                (*shader).byte_code_size as usize,
                (*shader).source_type,
            );
        }
    }

    if type_0 == WINED3D_SHADER_TYPE_GEOMETRY {
        hr = geometry_shader_init_stream_output(shader, so_desc);
        if hr < 0 {
            return hr;
        }
        if (*shader).function.is_null() {
            return S_OK;
        }
    }

    fe = (*shader).frontend;
    (*shader).frontend_data = (*fe).shader_init.unwrap_unchecked()(
        (*shader).function,
        (*shader).functionLength as usize,
        core::ptr::addr_of_mut!((*shader).output_signature),
    );
    if (*shader).frontend_data.is_null() {
        FIXME!("Failed to initialize frontend.\n");
        return WINED3DERR_INVALIDCALL;
    }

    hr = shader_get_registers_used(shader, float_const_count as DWORD);
    if hr < 0 {
        return hr;
    }

    (*shader).load_local_constsF = (*shader).lconst_inf_or_nan != 0;

    if (*version).type_ as ::core::ffi::c_uint != type_0 as ::core::ffi::c_uint {
        WARN!(
            "Wrong shader type %s.\n",
            debug_shader_type((*reg_maps).shader_version.type_)
        );
        return WINED3DERR_INVALIDCALL;
    }
    if !(*shader).is_ffp_vs
        && !(*shader).is_ffp_ps
        && (*version).major as ::core::ffi::c_uint
            > shader_max_version_from_feature_level((*(*(*(*shader).device).cs).c.state).feature_level)
    {
        WARN!(
            "Shader version %u not supported by this device.\n",
            (*version).major as ::core::ffi::c_int
        );
        return WINED3DERR_INVALIDCALL;
    }

    match type_0 as ::core::ffi::c_uint {
        WINED3D_SHADER_TYPE_VERTEX => {
            let mut i: ::core::ffi::c_uint = 0;
            while i < (*shader).input_signature.element_count {
                let input: *const wined3d_shader_signature_element =
                    (*shader).input_signature.elements.offset(i as isize);

                if !(((*reg_maps).input_registers as ::core::ffi::c_uint
                    & ((1 as ::core::ffi::c_uint) << (*input).register_idx)
                    == 0)
                    || (*input).semantic_name.is_null())
                {
                    (*shader).u.vs.attributes[(*input).register_idx as usize].usage =
                        shader_usage_from_semantic_name((*input).semantic_name);
                    (*shader).u.vs.attributes[(*input).register_idx as usize].usage_idx =
                        (*input).semantic_idx as UINT;
                }
                i = i.wrapping_add(1);
            }

            if (*reg_maps).usesrelconstF() as ::core::ffi::c_int != 0
                && list_empty(core::ptr::addr_of_mut!((*shader).constantsF)) == 0
            {
                (*shader).load_local_constsF = TRUE != 0;
            }

            backend_version = (*d3d_info).limits.vs_version;
        }
        WINED3D_SHADER_TYPE_HULL => {
            backend_version = (*d3d_info).limits.hs_version;
        }
        WINED3D_SHADER_TYPE_DOMAIN => {
            backend_version = (*d3d_info).limits.ds_version;
        }
        WINED3D_SHADER_TYPE_GEOMETRY => {
            backend_version = (*d3d_info).limits.gs_version;
        }
        WINED3D_SHADER_TYPE_PIXEL => {
            let mut i_0: ::core::ffi::c_uint = 0;
            while i_0 < MAX_REG_INPUT as ::core::ffi::c_uint {
                if (*shader).u.ps.input_reg_used as ::core::ffi::c_uint
                    & ((1 as ::core::ffi::c_uint) << i_0)
                    != 0
                {
                    num_regs_used = num_regs_used.wrapping_add(1);
                    highest_reg_used = i_0;
                }
                i_0 = i_0.wrapping_add(1);
            }

            /* Don't do any register mapping magic if it is not needed,
             * or if we can't achieve anything anyway. */
            if highest_reg_used
                < (*d3d_info).limits.varying_count.wrapping_div(4 as ::core::ffi::c_uint)
                || num_regs_used
                    > (*d3d_info).limits.varying_count.wrapping_div(4 as ::core::ffi::c_uint)
                || (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
                    >= 4
            {
                if num_regs_used
                    > (*d3d_info).limits.varying_count.wrapping_div(4 as ::core::ffi::c_uint)
                {
                    /* This happens with relative addressing. The input mapper
                     * function warns about this if the higher registers are
                     * declared too, so don't write a FIXME here. */
                    WARN!("More varying registers used than supported.\n");
                }

                let mut i_1: ::core::ffi::c_uint = 0;
                while i_1 < MAX_REG_INPUT as ::core::ffi::c_uint {
                    (*shader).u.ps.input_reg_map[i_1 as usize] = i_1;
                    i_1 = i_1.wrapping_add(1);
                }

                (*shader).u.ps.declared_in_count =
                    highest_reg_used.wrapping_add(1 as ::core::ffi::c_uint);
            } else {
                (*shader).u.ps.declared_in_count = 0;
                let mut i_2: ::core::ffi::c_uint = 0;
                while i_2 < MAX_REG_INPUT as ::core::ffi::c_uint {
                    if (*shader).u.ps.input_reg_used as ::core::ffi::c_uint
                        & ((1 as ::core::ffi::c_uint) << i_2)
                        != 0
                    {
                        let fresh15 = (*shader).u.ps.declared_in_count;
                        (*shader).u.ps.declared_in_count =
                            (*shader).u.ps.declared_in_count.wrapping_add(1);
                        (*shader).u.ps.input_reg_map[i_2 as usize] = fresh15;
                    } else {
                        (*shader).u.ps.input_reg_map[i_2 as usize] =
                            !(0 as ::core::ffi::c_uint);
                    }
                    i_2 = i_2.wrapping_add(1);
                }
            }

            backend_version = (*d3d_info).limits.ps_version;
        }
        WINED3D_SHADER_TYPE_COMPUTE => {
            backend_version = (*d3d_info).limits.cs_version;
        }
        _ => {
            FIXME!("No backend version-checking for this shader type.\n");
            backend_version = 0;
        }
    }

    if (*version).major as ::core::ffi::c_uint > backend_version {
        WARN!(
            "Shader version %u.%u not supported by the current shader backend.\n",
            (*version).major as ::core::ffi::c_int,
            (*version).minor as ::core::ffi::c_int
        );
        return WINED3DERR_INVALIDCALL;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_incref(shader: *mut wined3d_shader) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(
        core::ptr::addr_of_mut!((*shader).ref_) as *mut i32,
    ) as u32;
    TRACE!("%p increasing refcount to %u.\n", shader, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_shader_init_object(mut object: *mut ::core::ffi::c_void) {
    let mut shader: *mut wined3d_shader = object as *mut wined3d_shader;
    let mut device: *mut wined3d_device = (*shader).device;
    WINE_TRACE!("shader %p.\n", shader);
    list_add_head(
        &raw mut (*device).shaders,
        &raw mut (*shader).shader_list_entry,
    );
    if (*shader).is_ffp_vs {
        let mut settings: *mut wined3d_ffp_vs_settings =
            (*shader).byte_code as *mut wined3d_ffp_vs_settings;
        let mut desc: wined3d_shader_desc = wined3d_shader_desc {
            byte_code: ::core::ptr::null::<DWORD>(),
            byte_code_size: 0,
        };
        if !ffp_hlsl_compile_vs(settings, &raw mut desc, device) {
            return;
        }
        free(settings as *mut ::core::ffi::c_void);
        shader_set_function(
            shader,
            &raw mut desc,
            WINED3D_SHADER_TYPE_VERTEX,
            ::core::ptr::null::<wined3d_stream_output_desc>(),
            (*(*device).adapter).d3d_info.limits.vs_uniform_count as ::core::ffi::c_uint,
        );
    }
    if (*shader).is_ffp_ps {
        let mut settings_0: *mut ffp_frag_settings = (*shader).byte_code as *mut ffp_frag_settings;
        let mut desc_0: wined3d_shader_desc = wined3d_shader_desc {
            byte_code: ::core::ptr::null::<DWORD>(),
            byte_code_size: 0,
        };
        if !ffp_hlsl_compile_ps(settings_0, &raw mut desc_0) {
            return;
        }
        free(settings_0 as *mut ::core::ffi::c_void);
        shader_set_function(
            shader,
            &raw mut desc_0,
            WINED3D_SHADER_TYPE_PIXEL,
            ::core::ptr::null::<wined3d_stream_output_desc>(),
            (*(*device).adapter).d3d_info.limits.ps_uniform_count as ::core::ffi::c_uint,
        );
    }
    (*(*device).shader_backend)
        .shader_precompile
        .unwrap_unchecked()((*device).shader_priv, shader);
}

unsafe extern "C" fn wined3d_shader_destroy_object(object: *mut ::core::ffi::c_void) {
    shader_cleanup((object).cast());
    free(object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_decref(shader: *mut wined3d_shader) -> ULONG {
    let mut refcount: u32 = InterlockedDecrement(&raw mut (*shader).ref_) as u32;

    if refcount == 0 {
        wined3d_mutex_lock();
        (((*(*shader).parent_ops).wined3d_object_destroyed).unwrap_unchecked())((*shader).parent);
        wined3d_cs_destroy_object((*(*shader).device).cs, Some(wined3d_shader_destroy_object), (shader).cast());
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_get_parent(
    shader: *const wined3d_shader,
) -> *mut core::ffi::c_void {
    // TRACE("shader %p.\n", shader);
    (*shader).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_get_byte_code(
    shader: *const wined3d_shader,
    byte_code: *mut core::ffi::c_void,
    byte_code_size: *mut UINT,
) -> HRESULT {
    WINE_TRACE!("shader %p, byte_code %p, byte_code_size %p.\n", shader, byte_code, byte_code_size);
    if byte_code.is_null() {
        *byte_code_size = (*shader).byte_code_size as UINT;
        return WINED3D_OK;
    }
    if *byte_code_size < (*shader).byte_code_size {
        return WINED3DERR_INVALIDCALL;
    }
    memcpy(
        byte_code,
        (*shader).byte_code,
        (*shader).byte_code_size as size_t,
    );
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_set_local_constants_float(
    mut shader: *mut wined3d_shader,
    mut start_idx: UINT,
    mut src_data: *const f32,
    mut count: UINT,
) -> HRESULT {
    let mut end_idx: UINT = start_idx.wrapping_add(count);
    let mut i: UINT;

    if end_idx > (*(*shader).limits).constant_float as UINT {
        WARN!(
            "end_idx %u > float constants limit %u.\n",
            end_idx,
            (*(*shader).limits).constant_float
        );
        end_idx = (*(*shader).limits).constant_float as UINT;
    }

    i = start_idx;
    while i < end_idx {
        let mut lconst: *mut wined3d_shader_lconst = core::ptr::null_mut::<wined3d_shader_lconst>();
        let mut value: *mut f32 = core::ptr::null_mut::<f32>();

        lconst = malloc(core::mem::size_of::<wined3d_shader_lconst>() as usize)
            as *mut wined3d_shader_lconst;
        if lconst.is_null() {
            return E_OUTOFMEMORY;
        }

        (*lconst).idx = i as u32;
        value = &raw mut (*lconst).value as *mut f32;
        memcpy(
            value as *mut core::ffi::c_void,
            src_data.offset(
                (i.wrapping_sub(start_idx)).wrapping_mul(4 as UINT) as isize,
            ) as *const core::ffi::c_void,
            (4 as usize).wrapping_mul(core::mem::size_of::<f32>()),
        );
        list_add_head(
            core::ptr::addr_of_mut!((*shader).constantsF),
            core::ptr::addr_of_mut!((*lconst).entry),
        );

        if isinf!(*value.offset(0 as isize)) != 0
            || isnan!(*value.offset(0 as isize)) != 0
            || isinf!(*value.offset(1 as isize)) != 0
            || isnan!(*value.offset(1 as isize)) != 0
            || isinf!(*value.offset(2 as isize)) != 0
            || isnan!(*value.offset(2 as isize)) != 0
            || isinf!(*value.offset(3 as isize)) != 0
            || isnan!(*value.offset(3 as isize)) != 0
        {
            (*shader).lconst_inf_or_nan = TRUE as BOOL;
        }

        i = i.wrapping_add(1);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn init_interpolation_compile_args(
    mut interpolation_args: *mut uint32_t,
    mut pixel_shader: *const wined3d_shader,
    mut d3d_info: *const wined3d_d3d_info,
) {
    if (*d3d_info).shader_output_interpolation() == 0
        || pixel_shader.is_null()
        || ((*pixel_shader).reg_maps.shader_version.major as ::core::ffi::c_int)
            < 4
    {
        memset(
            interpolation_args as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<[uint32_t; 3]>() as size_t,
        );
        return;
    }
    memcpy(
        interpolation_args as *mut ::core::ffi::c_void,
        core::ptr::addr_of!((*pixel_shader).u.ps.interpolation_mode) as *const uint32_t
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[uint32_t; 3]>() as size_t,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_vs_compile_args(
    state: *const wined3d_state,
    shader: *const wined3d_shader,
    args: *mut vs_compile_args,
    context: *const wined3d_context,
) {
    let geometry_shader: *const wined3d_shader =
        (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize];
    let pixel_shader: *const wined3d_shader =
        (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize];
    let hull_shader: *const wined3d_shader =
        (*state).shader[WINED3D_SHADER_TYPE_HULL as usize];
    let d3d_info: *const wined3d_d3d_info = (*context).d3d_info;
    let swizzle_map: WORD = (*context).stream_info.swizzle_map as WORD;

    if (*state).render_states[WINED3D_RS_FOGTABLEMODE as usize]
        != WINED3D_FOG_NONE as ::core::ffi::c_uint
    {
        if (*state).transforms[WINED3D_TS_PROJECTION as usize]._14 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._24 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._34 == 0.0f32
            && (*state).transforms[WINED3D_TS_PROJECTION as usize]._44 == 1.0f32
        {
            /* Fog source is vertex output Z.
             *
             * However, if drawing RHW (which means we are using an HLSL
             * replacement shader, since we got here), and depth testing is
             * disabled, primitives are not supposed to be clipped by the
             * viewport. We handle this in the vertex shader by essentially
             * flushing output Z to zero.
             *
             * Fog needs to still read from the original Z, however. In this
             * case we read from oFog, which contains the original Z. */

            if (*(*state).vertex_declaration).position_transformed {
                (*args).fog_src = VS_FOG_COORD as BYTE;
            } else {
                (*args).fog_src = VS_FOG_Z as BYTE;
            }
        } else {
            (*args).fog_src = VS_FOG_W as BYTE;
        }
    } else {
        (*args).fog_src = VS_FOG_COORD as BYTE;
    }

    (*args).set_clip_enabled(
        ((*state).render_states[WINED3D_RS_CLIPPING as usize] != 0
            && (*state).render_states[WINED3D_RS_CLIPPLANEENABLE as usize] != 0)
            as ::core::ffi::c_int as BYTE,
    );
    (*args).set_point_size(
        ((*state).primitive_type as ::core::ffi::c_uint
            == WINED3D_PT_POINTLIST as ::core::ffi::c_uint) as ::core::ffi::c_int as BYTE,
    );
    (*args).set_next_shader_type(
        (if !hull_shader.is_null() {
            WINED3D_SHADER_TYPE_HULL as ::core::ffi::c_int
        } else if !geometry_shader.is_null() {
            WINED3D_SHADER_TYPE_GEOMETRY as ::core::ffi::c_int
        } else {
            WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
        }) as BYTE,
    );
    if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        >= 4 as ::core::ffi::c_int
    {
        (*args).next_shader_input_count = if !hull_shader.is_null() {
            (*(*hull_shader).limits).packed_input
        } else if !geometry_shader.is_null() {
            (*(*geometry_shader).limits).packed_input
        } else if !pixel_shader.is_null() {
            (*(*pixel_shader).limits).packed_input
        } else {
            0 as ::core::ffi::c_uint
        };
    } else {
        (*args).next_shader_input_count = 0 as ::core::ffi::c_uint;
    }
    (*args).swizzle_map = swizzle_map as DWORD;
    if (*d3d_info).emulated_flatshading() != 0 {
        (*args).set_flatshading(
            ((*state).render_states[WINED3D_RS_SHADEMODE as usize]
                == WINED3D_SHADE_FLAT as ::core::ffi::c_uint)
                as ::core::ffi::c_int as BYTE,
        );
    } else {
        (*args).set_flatshading(0 as BYTE);
    }

    init_interpolation_compile_args(
        &raw mut (*args).interpolation_mode as *mut uint32_t,
        if (*args).next_shader_type() as ::core::ffi::c_int
            == WINED3D_SHADER_TYPE_PIXEL as ::core::ffi::c_int
        {
            pixel_shader
        } else {
            ::core::ptr::null::<wined3d_shader>()
        },
        d3d_info,
    );
}

unsafe extern "C" fn match_usage(usage1: BYTE, usage_idx1: BYTE, usage2: BYTE, usage_idx2: BYTE) -> BOOL {
    if usage_idx1 != usage_idx2 {
        return (FALSE) as BOOL;
    }
    if usage1 == usage2 {
        return (TRUE) as BOOL;
    }
    if (usage1) as u32 == WINED3D_DECL_USAGE_POSITION && (usage2) as u32 == WINED3D_DECL_USAGE_POSITIONT {
        return (TRUE) as BOOL;
    }
    if (usage2) as u32 == WINED3D_DECL_USAGE_POSITION && (usage1) as u32 == WINED3D_DECL_USAGE_POSITIONT {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn vshader_get_input(
    shader: *const wined3d_shader,
    usage_req: uint8_t,
    usage_idx_req: uint8_t,
    regnum: *mut ::core::ffi::c_uint,
) -> bool {
    let mut map: uint32_t = (*shader).reg_maps.input_registers as uint32_t & 0xffff;
    let mut i: ::core::ffi::c_uint;

    while map != 0 {
        i = wined3d_bit_scan(&mut map as *mut uint32_t) as ::core::ffi::c_uint;
        if match_usage(
            (*shader).u.vs.attributes[i as usize].usage as BYTE,
            (*shader).u.vs.attributes[i as usize].usage_idx as BYTE,
            usage_req as BYTE,
            usage_idx_req as BYTE,
        ) != 0
        {
            *regnum = i;
            return true;
        }
    }

    return false;
}

unsafe extern "C" fn shader_init(shader: *mut wined3d_shader, device: *mut wined3d_device, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops) {
    (*shader).ref_ = 1;
    (*shader).device = device;
    (*shader).parent = parent;
    (*shader).parent_ops = parent_ops;

    list_init(core::ptr::addr_of_mut!((*shader).linked_programs));
    list_init(core::ptr::addr_of_mut!((*shader).constantsF));
    list_init(core::ptr::addr_of_mut!((*shader).constantsB));
    list_init(core::ptr::addr_of_mut!((*shader).constantsI));
    (*shader).lconst_inf_or_nan = (FALSE as i32);
    list_init(core::ptr::addr_of_mut!((*shader).reg_maps.indexable_temps));
    list_init(core::ptr::addr_of_mut!((*shader).shader_list_entry));
}

unsafe extern "C" fn vertex_shader_init(
    shader: *mut wined3d_shader,
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    let mut hr: HRESULT;

    shader_init(shader, device, parent, parent_ops);

    hr = shader_set_function(
        shader,
        desc,
        WINED3D_SHADER_TYPE_VERTEX,
        core::ptr::null::<wined3d_stream_output_desc>(),
        (*(*device).adapter).d3d_info.limits.vs_uniform_count as core::ffi::c_uint,
    );
    if hr < 0 {
        shader_cleanup(shader);
        return hr;
    }

    return WINED3D_OK;
}

unsafe extern "C" fn geometry_shader_init(
    shader: *mut wined3d_shader,
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    so_desc: *const wined3d_stream_output_desc,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    let mut hr: HRESULT = 0;

    shader_init(shader, device, parent, parent_ops);

    'fail: {
        hr = shader_set_function(shader, desc, WINED3D_SHADER_TYPE_GEOMETRY, so_desc, 0);
        if hr < 0 {
            break 'fail;
        }

        return WINED3D_OK;
    }

    shader_cleanup(shader);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn find_ds_compile_args(
    state: *const wined3d_state,
    shader: *const wined3d_shader,
    args: *mut ds_compile_args,
    context: *const wined3d_context,
) {
    let geometry_shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_GEOMETRY as usize];
    let pixel_shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize];
    let hull_shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_HULL as usize];

    (*args).tessellator_output_primitive = (*hull_shader).u.hs.tessellator_output_primitive;
    (*args).tessellator_partitioning = (*hull_shader).u.hs.tessellator_partitioning;

    (*args).set_output_count((if !geometry_shader.is_null() {
        (*(*geometry_shader).limits).packed_input
    } else if !pixel_shader.is_null() {
        (*(*pixel_shader).limits).packed_input
    } else {
        (*(*shader).limits).packed_output
    }) as _);
    (*args).set_next_shader_type((if !geometry_shader.is_null() {
        WINED3D_SHADER_TYPE_GEOMETRY
    } else {
        WINED3D_SHADER_TYPE_PIXEL
    }) as _);

    init_interpolation_compile_args(
        core::ptr::addr_of_mut!((*args).interpolation_mode) as *mut uint32_t,
        if (*args).next_shader_type() == WINED3D_SHADER_TYPE_PIXEL {
            pixel_shader
        } else {
            core::ptr::null::<wined3d_shader>()
        },
        (*context).d3d_info,
    );

    (*args).set_padding((0) as _);
}
#[no_mangle]
pub unsafe extern "C" fn find_gs_compile_args(
    state: *const wined3d_state,
    shader: *const wined3d_shader,
    args: *mut gs_compile_args,
    context: *const wined3d_context,
) {
    let pixel_shader: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_PIXEL as usize];

    (*args).output_count = if !pixel_shader.is_null() {
        (*(*pixel_shader).limits).packed_input
    } else {
        (*(*shader).limits).packed_output
    };

    (*args).primitive_type = (*shader).u.gs.input_type;
    if (*args).primitive_type as u64 == 0 {
        (*args).primitive_type = (*state).primitive_type;
    }

    init_interpolation_compile_args(
        &raw mut (*args).interpolation_mode as *mut uint32_t,
        pixel_shader,
        (*context).d3d_info,
    );
}
#[no_mangle]
pub unsafe extern "C" fn find_ps_compile_args(
    state: *const wined3d_state,
    shader: *const wined3d_shader,
    position_transformed: BOOL,
    args: *mut ps_compile_args,
    context: *const wined3d_context,
) {
    let vs: *const wined3d_shader = (*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize];
    let d3d_info: *const wined3d_d3d_info = (*context).d3d_info;
    let mut view: *mut wined3d_shader_resource_view = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    memset(
        args as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<ps_compile_args>() as size_t,
    ); /* FIXME: Make sure all bits are set. */
    if (*d3d_info).srgb_write_control() == 0
        && needs_srgb_write(d3d_info, state, &raw const (*state).fb) != 0
    {
        static mut warned: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

        (*args).srgb_correction = 1 as WORD;
        if !(*state).blend_state.is_null()
            && (*(*state).blend_state).desc.rt[0 as usize].enable != 0
            && {
                let fresh24 = warned;
                warned = warned.wrapping_add(1);
                fresh24 == 0
            }
        {
            WARN!("Blending into a sRGB render target with no GL_ARB_framebuffer_sRGB support, expect rendering artifacts.\n");
        }
    }

    if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*shader).reg_maps.shader_version.minor as ::core::ffi::c_int
            <= 3 as ::core::ffi::c_int
    {
        i = 0 as ::core::ffi::c_uint;
        while i < (*(*shader).limits).sampler {
            let mut flags: uint32_t = (*state).texture_states[i as usize]
                [WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS as usize];

            if (flags & WINED3D_TTFF_PROJECTED as uint32_t) != 0 {
                let mut tex_transform: uint32_t =
                    flags & !(WINED3D_TTFF_PROJECTED as uint32_t);

                if vs.is_null() || (*vs).is_ffp_vs as ::core::ffi::c_int != 0 {
                    let mut resource_type: wined3d_shader_resource_type =
                        (*shader).reg_maps.resource_info[i as usize].type_;
                    let mut j: ::core::ffi::c_uint = 0;
                    let mut index: ::core::ffi::c_uint = (*state).texture_states[i as usize]
                        [WINED3D_TSS_TEXCOORD_INDEX as usize];
                    let mut max_valid: uint32_t = WINED3D_TTFF_COUNT4 as uint32_t;

                    j = 0 as ::core::ffi::c_uint;
                    while j < (*(*state).vertex_declaration).element_count {
                        let element: *mut wined3d_vertex_declaration_element =
                            (*(*state).vertex_declaration).elements.offset(j as isize)
                                as *mut wined3d_vertex_declaration_element;

                        if (*element).usage as ::core::ffi::c_int
                            == WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int
                            && (*element).usage_idx as ::core::ffi::c_uint == index
                        {
                            max_valid = (*(*element).format).component_count as uint32_t;
                            break;
                        }
                        j = j.wrapping_add(1);
                    }
                    if tex_transform == 0 || tex_transform > max_valid {
                        WARN!("Fixing up projected texture transform flags from %#x to %#x.\n",
                                tex_transform, max_valid);
                        tex_transform = max_valid;
                    }
                    if (resource_type as ::core::ffi::c_uint
                            == WINED3D_SHADER_RESOURCE_TEXTURE_1D as ::core::ffi::c_uint
                            && tex_transform > WINED3D_TTFF_COUNT1 as uint32_t)
                        || (resource_type as ::core::ffi::c_uint
                            == WINED3D_SHADER_RESOURCE_TEXTURE_2D as ::core::ffi::c_uint
                            && tex_transform > WINED3D_TTFF_COUNT2 as uint32_t)
                        || (resource_type as ::core::ffi::c_uint
                            == WINED3D_SHADER_RESOURCE_TEXTURE_3D as ::core::ffi::c_uint
                            && tex_transform > WINED3D_TTFF_COUNT3 as uint32_t)
                    {
                        tex_transform = tex_transform | WINED3D_PSARGS_PROJECTED as uint32_t;
                    } else {
                        WARN!("Application requested projected texture with unsuitable texture coordinates.\n");
                        WARN!("(texture unit %u, transform flags %#x, sampler type %u).\n",
                                i, tex_transform, resource_type as ::core::ffi::c_uint);
                    }
                } else {
                    tex_transform = (WINED3D_TTFF_COUNT4 as ::core::ffi::c_uint
                        | WINED3D_PSARGS_PROJECTED as ::core::ffi::c_uint) as uint32_t;
                }
                (*args).tex_transform = ((*args).tex_transform as uint32_t
                    | (tex_transform
                        << i.wrapping_mul(WINED3D_PSARGS_TEXTRANSFORM_SHIFT as ::core::ffi::c_uint)))
                    as WORD;
            }
            i = i.wrapping_add(1);
        }
    }

    if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        == 1 as ::core::ffi::c_int
        && (*shader).reg_maps.shader_version.minor as ::core::ffi::c_int
            <= 4 as ::core::ffi::c_int
    {
        i = 0 as ::core::ffi::c_uint;
        while i < (*(*shader).limits).sampler {
            'body: {
                let mut type_0: wined3d_shader_tex_types =
                    WINED3D_SHADER_TEX_2D as wined3d_shader_tex_types;

                if (*shader).reg_maps.resource_info[i as usize].type_ as u64 == 0 {
                    break 'body;
                }

                /* Treat unbound textures as 2D. The dummy texture will provide
                 * the proper sample value. The tex_types bitmap defaults to
                 * 2D because of the memset. */
                view = (*state).shader_resource_view
                    [WINED3D_SHADER_TYPE_PIXEL as usize][i as usize];
                if view.is_null() {
                    break 'body;
                }
                texture = texture_from_resource((*view).resource);

                match (*texture).resource.type_ as ::core::ffi::c_uint {
                    WINED3D_RTYPE_NONE
                    | WINED3D_RTYPE_BUFFER
                    | WINED3D_RTYPE_TEXTURE_1D => {
                        assert!(0 != 0);
                        if ((*texture).resource.usage & WINED3DUSAGE_LEGACY_CUBEMAP as uint32_t) != 0 {
                            type_0 = WINED3D_SHADER_TEX_CUBE as wined3d_shader_tex_types;
                        }
                    }
                    WINED3D_RTYPE_TEXTURE_2D => {
                        if ((*texture).resource.usage & WINED3DUSAGE_LEGACY_CUBEMAP as uint32_t) != 0 {
                            type_0 = WINED3D_SHADER_TEX_CUBE as wined3d_shader_tex_types;
                        }
                    }
                    WINED3D_RTYPE_TEXTURE_3D => {
                        type_0 = WINED3D_SHADER_TEX_3D as wined3d_shader_tex_types;
                    }
                    _ => {}
                }

                (*args).tex_types = (((*args).tex_types as ::core::ffi::c_uint
                    | ((type_0 as ::core::ffi::c_uint)
                        << i.wrapping_mul(WINED3D_PSARGS_TEXTYPE_SHIFT as ::core::ffi::c_uint)))
                    as WORD) as _;
            }
            i = i.wrapping_add(1);
        }
    } else if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        <= 3 as ::core::ffi::c_int
    {
        i = 0 as ::core::ffi::c_uint;
        while i < (*(*shader).limits).sampler {
            'body: {
                let resource_type: wined3d_shader_resource_type =
                    (*shader).reg_maps.resource_info[i as usize].type_;
                let tex_type: wined3d_shader_tex_types;

                if resource_type as u64 == 0 {
                    break 'body;
                }

                match resource_type as ::core::ffi::c_uint {
                    WINED3D_SHADER_RESOURCE_TEXTURE_3D => {
                        tex_type = WINED3D_SHADER_TEX_3D as wined3d_shader_tex_types;
                    }
                    WINED3D_SHADER_RESOURCE_TEXTURE_CUBE => {
                        tex_type = WINED3D_SHADER_TEX_CUBE as wined3d_shader_tex_types;
                    }
                    _ => {
                        tex_type = WINED3D_SHADER_TEX_2D as wined3d_shader_tex_types;
                    }
                }

                let mut tex_type_m = tex_type;

                view = (*state).shader_resource_view
                    [WINED3D_SHADER_TYPE_PIXEL as usize][i as usize];
                if !view.is_null() {
                    texture = texture_from_resource((*view).resource);
                    /* Star Wars: The Old Republic uses mismatched samplers for rendering water. */
                    if (*texture).resource.type_ as ::core::ffi::c_uint
                        == WINED3D_RTYPE_TEXTURE_2D as ::core::ffi::c_uint
                        && resource_type as ::core::ffi::c_uint
                            == WINED3D_SHADER_RESOURCE_TEXTURE_3D as ::core::ffi::c_uint
                        && ((*texture).resource.usage & WINED3DUSAGE_LEGACY_CUBEMAP as uint32_t) == 0
                    {
                        tex_type_m = WINED3D_SHADER_TEX_2D as wined3d_shader_tex_types;
                    } else if (*texture).resource.type_ as ::core::ffi::c_uint
                        == WINED3D_RTYPE_TEXTURE_3D as ::core::ffi::c_uint
                        && resource_type as ::core::ffi::c_uint
                            == WINED3D_SHADER_RESOURCE_TEXTURE_2D as ::core::ffi::c_uint
                    {
                        tex_type_m = WINED3D_SHADER_TEX_3D as wined3d_shader_tex_types;
                    }
                }
                (*args).tex_types = (((*args).tex_types as ::core::ffi::c_uint
                    | ((tex_type_m as ::core::ffi::c_uint)
                        << i.wrapping_mul(WINED3D_PSARGS_TEXTYPE_SHIFT as ::core::ffi::c_uint)))
                    as WORD) as _;
            }
            i = i.wrapping_add(1);
        }
    }

    if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        >= 4 as ::core::ffi::c_int
    {
        /* In SM4+ we use dcl_sampler in order to determine if we should use shadow sampler. */
        (*args).shadow = 0 as WORD;
        i = 0 as ::core::ffi::c_uint;
        while i < WINED3D_MAX_FRAGMENT_SAMPLERS as ::core::ffi::c_uint {
            (*args).color_fixup[i as usize] = COLOR_FIXUP_IDENTITY;
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as ::core::ffi::c_uint;
        while i < WINED3D_MAX_FRAGMENT_SAMPLERS as ::core::ffi::c_uint {
            'body: {
                if (*shader).reg_maps.resource_info[i as usize].type_ as u64 == 0 {
                    break 'body;
                }

                view = (*state).shader_resource_view
                    [WINED3D_SHADER_TYPE_PIXEL as usize][i as usize];
                if view.is_null() {
                    (*args).color_fixup[i as usize] = COLOR_FIXUP_IDENTITY;
                    break 'body;
                }
                texture = texture_from_resource((*view).resource);

                if can_use_texture_swizzle(d3d_info, (*texture).resource.format) != 0 {
                    (*args).color_fixup[i as usize] = COLOR_FIXUP_IDENTITY;
                } else {
                    (*args).color_fixup[i as usize] = (*(*texture).resource.format).color_fixup;
                }

                if ((*texture).resource.format_caps
                    & WINED3D_FORMAT_CAP_SHADOW as ::core::ffi::c_uint)
                    != 0
                {
                    (*args).shadow = ((*args).shadow as ::core::ffi::c_uint
                        | ((1 as ::core::ffi::c_uint) << i))
                        as WORD;
                }
            }
            i = i.wrapping_add(1);
        }
    }

    if (*shader).reg_maps.shader_version.major as ::core::ffi::c_int
        >= 3 as ::core::ffi::c_int
    {
        if position_transformed != 0 {
            (*args).vp_mode = WINED3D_VP_MODE_NONE;
        } else if use_vs(state) != 0 {
            (*args).vp_mode = WINED3D_VP_MODE_SHADER;
        } else {
            (*args).vp_mode = WINED3D_VP_MODE_FF;
        }
        (*args).fog = WINED3D_FFP_PS_FOG_OFF;
    } else {
        (*args).vp_mode = WINED3D_VP_MODE_SHADER;
        if (*state).render_states[WINED3D_RS_FOGENABLE as usize] != 0 {
            match (*state).render_states[WINED3D_RS_FOGTABLEMODE as usize] {
                WINED3D_FOG_NONE | WINED3D_FOG_LINEAR => {
                    (*args).fog = WINED3D_FFP_PS_FOG_LINEAR;
                }
                WINED3D_FOG_EXP => {
                    (*args).fog = WINED3D_FFP_PS_FOG_EXP;
                }
                WINED3D_FOG_EXP2 => {
                    (*args).fog = WINED3D_FFP_PS_FOG_EXP2;
                }
                _ => {}
            }
        } else {
            (*args).fog = WINED3D_FFP_PS_FOG_OFF;
        }
    }

    if (*d3d_info).full_ffp_varyings() == 0 {
        let vs_0: *const wined3d_shader =
            (*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize];

        (*args).texcoords_initialized = 0 as WORD;
        i = 0 as ::core::ffi::c_uint;
        while i < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint {
            if !vs_0.is_null() {
                if ((*(*state).shader[WINED3D_SHADER_TYPE_VERTEX as usize])
                    .reg_maps
                    .output_registers as ::core::ffi::c_uint
                    & ((1 as ::core::ffi::c_uint) << i))
                    != 0
                {
                    (*args).texcoords_initialized =
                        ((*args).texcoords_initialized as ::core::ffi::c_uint
                            | ((1 as ::core::ffi::c_uint) << i)) as WORD;
                }
            } else {
                let si: *const wined3d_stream_info = &raw const (*context).stream_info;
                let coord_idx: ::core::ffi::c_uint = (*state).texture_states[i as usize]
                    [WINED3D_TSS_TEXCOORD_INDEX as usize];

                if (((*state).texture_states[i as usize]
                    [WINED3D_TSS_TEXCOORD_INDEX as usize]
                    >> WINED3D_FFP_TCI_SHIFT as ::core::ffi::c_uint)
                    & WINED3D_FFP_TCI_MASK as uint32_t)
                    != 0
                    || (coord_idx < WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint
                        && ((*si).use_map as ::core::ffi::c_uint
                            & ((1 as ::core::ffi::c_uint)
                                << (WINED3D_FFP_TEXCOORD0 as ::core::ffi::c_uint)
                                    .wrapping_add(coord_idx)))
                            != 0)
                {
                    (*args).texcoords_initialized =
                        ((*args).texcoords_initialized as ::core::ffi::c_uint
                            | ((1 as ::core::ffi::c_uint) << i)) as WORD;
                }
            }
            i = i.wrapping_add(1);
        }
    } else {
        (*args).texcoords_initialized =
            wined3d_mask_from_size(WINED3D_MAX_FFP_TEXTURES as ::core::ffi::c_uint) as WORD;
    }

    (*args).set_pointsprite(
        (((*state).render_states[WINED3D_RS_POINTSPRITEENABLE as usize] != 0
            && (*state).primitive_type as ::core::ffi::c_uint
                == WINED3D_PT_POINTLIST as ::core::ffi::c_uint)
            as ::core::ffi::c_int) as DWORD,
    );

    if (*d3d_info).ffp_alpha_test() != 0 {
        (*args).set_alpha_test_func(
            ((WINED3D_CMP_ALWAYS as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int)) as DWORD,
        );
    } else {
        (*args).set_alpha_test_func(
            (if (*state).render_states[WINED3D_RS_ALPHATESTENABLE as usize] != 0 {
                wined3d_sanitize_cmp_func(
                    (*state).render_states[WINED3D_RS_ALPHAFUNC as usize]
                        as wined3d_cmp_func,
                ) as ::core::ffi::c_uint
            } else {
                WINED3D_CMP_ALWAYS as ::core::ffi::c_uint
            })
            .wrapping_sub(1 as ::core::ffi::c_uint) as DWORD,
        );
    }

    if (*d3d_info).emulated_flatshading() != 0 {
        (*args).set_flatshading(
            (((*state).render_states[WINED3D_RS_SHADEMODE as usize]
                == WINED3D_SHADE_FLAT as ::core::ffi::c_uint)
                as ::core::ffi::c_int) as DWORD,
        );
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!((*state).fb.render_targets) as usize {
        let rtv: *mut wined3d_rendertarget_view = (*state).fb.render_targets[i as usize];
        if !rtv.is_null()
            && (*(*rtv).format).id as ::core::ffi::c_uint
                == WINED3DFMT_A8_UNORM as ::core::ffi::c_uint
            && is_identity_fixup((*(*rtv).format).color_fixup) == 0
        {
            (*args).set_rt_alpha_swizzle(
                (*args).rt_alpha_swizzle()
                    | (((1 as ::core::ffi::c_uint) << i) as DWORD),
            );
        }
        i = i.wrapping_add(1);
    }

    (*args).set_dual_source_blend(
        ((!(*state).blend_state.is_null() && (*(*state).blend_state).dual_source != 0)
            as ::core::ffi::c_int) as DWORD,
    );
}

unsafe extern "C" fn pixel_shader_init(
    shader: *mut wined3d_shader,
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*device).adapter).d3d_info);
    let mut hr: HRESULT;

    shader_init(shader, device, parent, parent_ops);

    hr = shader_set_function(
        shader,
        desc,
        WINED3D_SHADER_TYPE_PIXEL,
        core::ptr::null(),
        (*d3d_info).limits.ps_uniform_count as core::ffi::c_uint,
    );
    if hr < 0 {
        shader_cleanup(shader);
        return hr;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn pixelshader_get_resource_type(
    reg_maps: *const wined3d_shader_reg_maps,
    resource_idx: ::core::ffi::c_uint,
    tex_types: DWORD,
) -> wined3d_shader_resource_type {
    static shader_resource_type_from_shader_tex_types: [wined3d_shader_resource_type; 3] = [
        WINED3D_SHADER_RESOURCE_TEXTURE_2D,
        WINED3D_SHADER_RESOURCE_TEXTURE_3D,
        WINED3D_SHADER_RESOURCE_TEXTURE_CUBE,
    ];

    let mut idx: ::core::ffi::c_uint = 0;

    if ((*reg_maps).shader_version.major as ::core::ffi::c_int) > (3 as ::core::ffi::c_int) {
        return (*reg_maps).resource_info[resource_idx as usize].type_;
    }

    if (*reg_maps).resource_info[resource_idx as usize].type_ as u64 == 0 {
        return 0;
    }

    idx = ((tex_types as ::core::ffi::c_uint)
        >> (resource_idx.wrapping_mul(WINED3D_PSARGS_TEXTYPE_SHIFT as ::core::ffi::c_uint)))
        & (WINED3D_PSARGS_TEXTYPE_MASK as ::core::ffi::c_uint);

    assert!(idx < ARRAY_SIZE!(shader_resource_type_from_shader_tex_types) as ::core::ffi::c_uint);
    return shader_resource_type_from_shader_tex_types[idx as usize];
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_cs(
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    shader: *mut *mut wined3d_shader,
) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc((1 as usize) as _, (core::mem::size_of::<wined3d_shader>() as usize) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    shader_init(object, device, parent, parent_ops);

    hr = shader_set_function(object, desc, WINED3D_SHADER_TYPE_COMPUTE, core::ptr::null(), 0);
    if hr < 0 {
        shader_cleanup(object);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object(
        (*device).cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        object as *mut core::ffi::c_void,
    );

    *shader = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_ds(
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    shader: *mut *mut wined3d_shader,
) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        "device %p, desc %p, parent %p, parent_ops %p, shader %p.\n",
        device,
        desc,
        parent,
        parent_ops,
        shader
    );

    object = calloc(
        1,
        (core::mem::size_of::<wined3d_shader>()) as _,
    ) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    shader_init(object, device, parent, parent_ops);

    hr = shader_set_function(
        object,
        desc,
        WINED3D_SHADER_TYPE_DOMAIN,
        core::ptr::null::<wined3d_stream_output_desc>(),
        0,
    );
    if hr < 0 {
        shader_cleanup(object);
        free(object as *mut ::core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        object as *mut ::core::ffi::c_void,
    );

    TRACE!("Created domain shader %p.\n", object);

    *shader = object;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_gs(device: *mut wined3d_device, desc: *const wined3d_shader_desc, so_desc: *const wined3d_stream_output_desc, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops, shader: *mut *mut wined3d_shader) -> HRESULT {
    let mut object: *mut wined3d_shader;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<wined3d_shader>()) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = geometry_shader_init(object, device, desc, so_desc, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialize geometry shader, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        object as *mut core::ffi::c_void,
    );

    *shader = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_hs(device: *mut wined3d_device, desc: *const wined3d_shader_desc, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops, shader: *mut *mut wined3d_shader) -> HRESULT {
    let mut object: *mut wined3d_shader;
    let mut hr: HRESULT;

    object = calloc((1 as size_t) as _, (core::mem::size_of::<wined3d_shader>() as size_t) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    shader_init(object, device, parent, parent_ops);

    hr = shader_set_function(object, desc, WINED3D_SHADER_TYPE_HULL, core::ptr::null::<wined3d_stream_output_desc>(), 0);
    if hr < 0 {
        shader_cleanup(object);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object((*device).cs as *mut wined3d_cs, Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()), object as *mut core::ffi::c_void);

    *shader = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_ps(device: *mut wined3d_device, desc: *const wined3d_shader_desc, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops, shader: *mut *mut wined3d_shader) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc(1, (core::mem::size_of::<wined3d_shader>()) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = pixel_shader_init(object, device, desc, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialize pixel shader, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object((*device).cs as *mut wined3d_cs, Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()), object as *mut core::ffi::c_void);

    *shader = object;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_vs(
    device: *mut wined3d_device,
    desc: *const wined3d_shader_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    shader: *mut *mut wined3d_shader,
) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<wined3d_shader>()) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = vertex_shader_init(object, device, desc, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialize vertex shader, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        object as *mut core::ffi::c_void,
    );

    *shader = object;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_ffp_vs(
    device: *mut wined3d_device,
    settings: *const wined3d_ffp_vs_settings,
    shader: *mut *mut wined3d_shader,
) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut::<wined3d_shader>();

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_shader>() as usize) as _,
    ) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    shader_init(
        (object as *mut core::ffi::c_void).cast(),
        (device as *mut core::ffi::c_void).cast(),
        core::ptr::null_mut(),
        (core::ptr::addr_of!(wined3d_null_parent_ops) as *const core::ffi::c_void).cast(),
    );
    (*object).is_ffp_vs = true;
    (*object).byte_code = malloc(
        core::mem::size_of::<wined3d_ffp_vs_settings>() as usize,
    );
    if (*object).byte_code.is_null() {
        free(object as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }
    memcpy(
        (*object).byte_code,
        settings as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_ffp_vs_settings>() as usize,
    );

    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        object as *mut core::ffi::c_void,
    );

    WINE_TRACE!("Created FFP vertex shader %p.\n", object);

    *shader = object;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_create_ffp_ps(
    device: *mut wined3d_device,
    settings: *const ffp_frag_settings,
    shader: *mut *mut wined3d_shader,
) -> HRESULT {
    let mut object: *mut wined3d_shader = core::ptr::null_mut();

    object = calloc((1 as size_t) as _, (core::mem::size_of::<wined3d_shader>() as size_t) as _) as *mut wined3d_shader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    shader_init(object, device, core::ptr::null_mut(), core::ptr::addr_of!(wined3d_null_parent_ops));
    (*object).is_ffp_ps = true;
    (*object).byte_code = malloc(core::mem::size_of::<ffp_frag_settings>() as size_t);
    if (*object).byte_code.is_null() {
        free(object as *mut core::ffi::c_void);
        return E_OUTOFMEMORY;
    }
    memcpy(
        (*object).byte_code,
        settings as *const core::ffi::c_void,
        core::mem::size_of::<ffp_frag_settings>() as size_t,
    );

    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_shader_init_object as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        object as *mut core::ffi::c_void,
    );

    WINE_TRACE!("Created FFP pixel shader %p.\n", object);

    *shader = object;

    WINED3D_OK
}
