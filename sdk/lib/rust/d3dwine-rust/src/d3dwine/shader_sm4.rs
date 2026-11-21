use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut output_primitive_type_table: [wined3d_primitive_type; 6] = [
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_POINTLIST,
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_LINESTRIP,
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_TRIANGLESTRIP,
];

static mut input_primitive_type_table: [wined3d_primitive_type; 8] = [
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_POINTLIST,
    WINED3D_PT_LINELIST,
    WINED3D_PT_TRIANGLELIST,
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_UNDEFINED,
    WINED3D_PT_LINELIST_ADJ,
    WINED3D_PT_TRIANGLELIST_ADJ,
];

static mut resource_type_table: [wined3d_shader_resource_type; 11] = [
    WINED3D_SHADER_RESOURCE_NONE,
    WINED3D_SHADER_RESOURCE_BUFFER,
    WINED3D_SHADER_RESOURCE_TEXTURE_1D,
    WINED3D_SHADER_RESOURCE_TEXTURE_2D,
    WINED3D_SHADER_RESOURCE_TEXTURE_2DMS,
    WINED3D_SHADER_RESOURCE_TEXTURE_3D,
    WINED3D_SHADER_RESOURCE_TEXTURE_CUBE,
    WINED3D_SHADER_RESOURCE_TEXTURE_1DARRAY,
    WINED3D_SHADER_RESOURCE_TEXTURE_2DARRAY,
    WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY,
    WINED3D_SHADER_RESOURCE_TEXTURE_CUBEARRAY,
];

static mut data_type_table: [wined3d_data_type; 6] = [
    WINED3D_DATA_FLOAT,
    WINED3D_DATA_UNORM,
    WINED3D_DATA_SNORM,
    WINED3D_DATA_INT,
    WINED3D_DATA_UINT,
    WINED3D_DATA_FLOAT,
];

static mut opcode_table: [wined3d_sm4_opcode_info; 184] = unsafe {
    [
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ADD,
            handler_idx: WINED3DSIH_ADD,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_AND,
            handler_idx: WINED3DSIH_AND,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_BREAK,
            handler_idx: WINED3DSIH_BREAK,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_BREAKC,
            handler_idx: WINED3DSIH_BREAKP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_conditional_op
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_CASE,
            handler_idx: WINED3DSIH_CASE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_CONTINUE,
            handler_idx: WINED3DSIH_CONTINUE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_CONTINUEC,
            handler_idx: WINED3DSIH_CONTINUEP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_conditional_op
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_CUT,
            handler_idx: WINED3DSIH_CUT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DEFAULT,
            handler_idx: WINED3DSIH_DEFAULT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DERIV_RTX,
            handler_idx: WINED3DSIH_DSX,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DERIV_RTY,
            handler_idx: WINED3DSIH_DSY,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DISCARD,
            handler_idx: WINED3DSIH_TEXKILL,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_conditional_op
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DIV,
            handler_idx: WINED3DSIH_DIV,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DP2,
            handler_idx: WINED3DSIH_DP2,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DP3,
            handler_idx: WINED3DSIH_DP3,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DP4,
            handler_idx: WINED3DSIH_DP4,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ELSE,
            handler_idx: WINED3DSIH_ELSE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_EMIT,
            handler_idx: WINED3DSIH_EMIT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ENDIF,
            handler_idx: WINED3DSIH_ENDIF,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ENDLOOP,
            handler_idx: WINED3DSIH_ENDLOOP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ENDSWITCH,
            handler_idx: WINED3DSIH_ENDSWITCH,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_EQ,
            handler_idx: WINED3DSIH_EQ,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_EXP,
            handler_idx: WINED3DSIH_EXP,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_FRC,
            handler_idx: WINED3DSIH_FRC,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_FTOI,
            handler_idx: WINED3DSIH_FTOI,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_FTOU,
            handler_idx: WINED3DSIH_FTOU,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_GE,
            handler_idx: WINED3DSIH_GE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IADD,
            handler_idx: WINED3DSIH_IADD,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IF,
            handler_idx: WINED3DSIH_IF,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_conditional_op
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IEQ,
            handler_idx: WINED3DSIH_IEQ,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IGE,
            handler_idx: WINED3DSIH_IGE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ILT,
            handler_idx: WINED3DSIH_ILT,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IMAD,
            handler_idx: WINED3DSIH_IMAD,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IMAX,
            handler_idx: WINED3DSIH_IMAX,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IMIN,
            handler_idx: WINED3DSIH_IMIN,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_IMUL,
            handler_idx: WINED3DSIH_IMUL,
            dst_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_INE,
            handler_idx: WINED3DSIH_INE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_INEG,
            handler_idx: WINED3DSIH_INEG,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ISHL,
            handler_idx: WINED3DSIH_ISHL,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ISHR,
            handler_idx: WINED3DSIH_ISHR,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ITOF,
            handler_idx: WINED3DSIH_ITOF,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LABEL,
            handler_idx: WINED3DSIH_LABEL,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"O\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LD,
            handler_idx: WINED3DSIH_LD,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iR\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LD2DMS,
            handler_idx: WINED3DSIH_LD2DMS,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iRi\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LOG,
            handler_idx: WINED3DSIH_LOG,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LOOP,
            handler_idx: WINED3DSIH_LOOP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LT,
            handler_idx: WINED3DSIH_LT,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MAD,
            handler_idx: WINED3DSIH_MAD,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MIN,
            handler_idx: WINED3DSIH_MIN,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MAX,
            handler_idx: WINED3DSIH_MAX,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SHADER_DATA,
            handler_idx: WINED3DSIH_DCL_IMMEDIATE_CONSTANT_BUFFER,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_shader_data
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MOV,
            handler_idx: WINED3DSIH_MOV,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MOVC,
            handler_idx: WINED3DSIH_MOVC,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_MUL,
            handler_idx: WINED3DSIH_MUL,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_NE,
            handler_idx: WINED3DSIH_NE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_NOP,
            handler_idx: WINED3DSIH_NOP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_NOT,
            handler_idx: WINED3DSIH_NOT,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_OR,
            handler_idx: WINED3DSIH_OR,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_RESINFO,
            handler_idx: WINED3DSIH_RESINFO,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iR\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_RET,
            handler_idx: WINED3DSIH_RET,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_RETC,
            handler_idx: WINED3DSIH_RETP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_conditional_op
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ROUND_NE,
            handler_idx: WINED3DSIH_ROUND_NE,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ROUND_NI,
            handler_idx: WINED3DSIH_ROUND_NI,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ROUND_PI,
            handler_idx: WINED3DSIH_ROUND_PI,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ROUND_Z,
            handler_idx: WINED3DSIH_ROUND_Z,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_RSQ,
            handler_idx: WINED3DSIH_RSQ,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE,
            handler_idx: WINED3DSIH_SAMPLE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRS\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_C,
            handler_idx: WINED3DSIH_SAMPLE_C,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_C_LZ,
            handler_idx: WINED3DSIH_SAMPLE_C_LZ,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_LOD,
            handler_idx: WINED3DSIH_SAMPLE_LOD,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_GRAD,
            handler_idx: WINED3DSIH_SAMPLE_GRAD,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_B,
            handler_idx: WINED3DSIH_SAMPLE_B,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SQRT,
            handler_idx: WINED3DSIH_SQRT,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SWITCH,
            handler_idx: WINED3DSIH_SWITCH,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SINCOS,
            handler_idx: WINED3DSIH_SINCOS,
            dst_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UDIV,
            handler_idx: WINED3DSIH_UDIV,
            dst_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_ULT,
            handler_idx: WINED3DSIH_ULT,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UGE,
            handler_idx: WINED3DSIH_UGE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UMUL,
            handler_idx: WINED3DSIH_UMUL,
            dst_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UMAX,
            handler_idx: WINED3DSIH_UMAX,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UMIN,
            handler_idx: WINED3DSIH_UMIN,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_USHR,
            handler_idx: WINED3DSIH_USHR,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_UTOF,
            handler_idx: WINED3DSIH_UTOF,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_XOR,
            handler_idx: WINED3DSIH_XOR,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_RESOURCE,
            handler_idx: WINED3DSIH_DCL,
            dst_info: b"R\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_resource
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_CONSTANT_BUFFER,
            handler_idx: WINED3DSIH_DCL_CONSTANT_BUFFER,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_constant_buffer
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_SAMPLER,
            handler_idx: WINED3DSIH_DCL_SAMPLER,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_sampler
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INDEX_RANGE,
            handler_idx: WINED3DSIH_DCL_INDEX_RANGE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_index_range
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_OUTPUT_TOPOLOGY,
            handler_idx: WINED3DSIH_DCL_OUTPUT_TOPOLOGY,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_output_topology
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_PRIMITIVE,
            handler_idx: WINED3DSIH_DCL_INPUT_PRIMITIVE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_input_primitive
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_VERTICES_OUT,
            handler_idx: WINED3DSIH_DCL_VERTICES_OUT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT,
            handler_idx: WINED3DSIH_DCL_INPUT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_dst
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_SGV,
            handler_idx: WINED3DSIH_DCL_INPUT_SGV,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_register_semantic
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_SIV,
            handler_idx: WINED3DSIH_DCL_INPUT_SIV,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_register_semantic
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_PS,
            handler_idx: WINED3DSIH_DCL_INPUT_PS,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_input_ps
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_PS_SGV,
            handler_idx: WINED3DSIH_DCL_INPUT_PS_SGV,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_register_semantic
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INPUT_PS_SIV,
            handler_idx: WINED3DSIH_DCL_INPUT_PS_SIV,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_input_ps_siv
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_OUTPUT,
            handler_idx: WINED3DSIH_DCL_OUTPUT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_dst
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_OUTPUT_SIV,
            handler_idx: WINED3DSIH_DCL_OUTPUT_SIV,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_register_semantic
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_TEMPS,
            handler_idx: WINED3DSIH_DCL_TEMPS,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_INDEXABLE_TEMP,
            handler_idx: WINED3DSIH_DCL_INDEXABLE_TEMP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_indexable_temp
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_DCL_GLOBAL_FLAGS,
            handler_idx: WINED3DSIH_DCL_GLOBAL_FLAGS,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_global_flags
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_LOD,
            handler_idx: WINED3DSIH_LOD,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRS\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_GATHER4,
            handler_idx: WINED3DSIH_GATHER4,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRS\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_POS,
            handler_idx: WINED3DSIH_SAMPLE_POS,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"Ru\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM4_OP_SAMPLE_INFO,
            handler_idx: WINED3DSIH_SAMPLE_INFO,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"R\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_HS_DECLS,
            handler_idx: WINED3DSIH_HS_DECLS,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_HS_CONTROL_POINT_PHASE,
            handler_idx: WINED3DSIH_HS_CONTROL_POINT_PHASE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_HS_FORK_PHASE,
            handler_idx: WINED3DSIH_HS_FORK_PHASE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_HS_JOIN_PHASE,
            handler_idx: WINED3DSIH_HS_JOIN_PHASE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_EMIT_STREAM,
            handler_idx: WINED3DSIH_EMIT_STREAM,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_CUT_STREAM,
            handler_idx: WINED3DSIH_CUT_STREAM,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_FCALL,
            handler_idx: WINED3DSIH_FCALL,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"O\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_fcall
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_BUFINFO,
            handler_idx: WINED3DSIH_BUFINFO,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DERIV_RTX_COARSE,
            handler_idx: WINED3DSIH_DSX_COARSE,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DERIV_RTX_FINE,
            handler_idx: WINED3DSIH_DSX_FINE,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DERIV_RTY_COARSE,
            handler_idx: WINED3DSIH_DSY_COARSE,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DERIV_RTY_FINE,
            handler_idx: WINED3DSIH_DSY_FINE,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_GATHER4_C,
            handler_idx: WINED3DSIH_GATHER4_C,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_GATHER4_PO,
            handler_idx: WINED3DSIH_GATHER4_PO,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fiRS\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_GATHER4_PO_C,
            handler_idx: WINED3DSIH_GATHER4_PO_C,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fiRSf\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_RCP,
            handler_idx: WINED3DSIH_RCP,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_F32TOF16,
            handler_idx: WINED3DSIH_F32TOF16,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_F16TOF32,
            handler_idx: WINED3DSIH_F16TOF32,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_COUNTBITS,
            handler_idx: WINED3DSIH_COUNTBITS,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_FIRSTBIT_HI,
            handler_idx: WINED3DSIH_FIRSTBIT_HI,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_FIRSTBIT_LO,
            handler_idx: WINED3DSIH_FIRSTBIT_LO,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_FIRSTBIT_SHI,
            handler_idx: WINED3DSIH_FIRSTBIT_SHI,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_UBFE,
            handler_idx: WINED3DSIH_UBFE,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iiu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IBFE,
            handler_idx: WINED3DSIH_IBFE,
            dst_info: b"i\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_BFI,
            handler_idx: WINED3DSIH_BFI,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iiuu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_BFREV,
            handler_idx: WINED3DSIH_BFREV,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_SWAPC,
            handler_idx: WINED3DSIH_SWAPC,
            dst_info: b"ff\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"uff\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_STREAM,
            handler_idx: WINED3DSIH_DCL_STREAM,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"O\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_FUNCTION_BODY,
            handler_idx: WINED3DSIH_DCL_FUNCTION_BODY,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_function_body
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_FUNCTION_TABLE,
            handler_idx: WINED3DSIH_DCL_FUNCTION_TABLE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_function_table
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_INTERFACE,
            handler_idx: WINED3DSIH_DCL_INTERFACE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_interface
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_INPUT_CONTROL_POINT_COUNT,
            handler_idx: WINED3DSIH_DCL_INPUT_CONTROL_POINT_COUNT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_control_point_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_OUTPUT_CONTROL_POINT_COUNT,
            handler_idx: WINED3DSIH_DCL_OUTPUT_CONTROL_POINT_COUNT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_control_point_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_TESSELLATOR_DOMAIN,
            handler_idx: WINED3DSIH_DCL_TESSELLATOR_DOMAIN,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_tessellator_domain
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_TESSELLATOR_PARTITIONING,
            handler_idx: WINED3DSIH_DCL_TESSELLATOR_PARTITIONING,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_tessellator_partitioning
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_TESSELLATOR_OUTPUT_PRIMITIVE,
            handler_idx: WINED3DSIH_DCL_TESSELLATOR_OUTPUT_PRIMITIVE,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_tessellator_output_primitive
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_HS_MAX_TESSFACTOR,
            handler_idx: WINED3DSIH_DCL_HS_MAX_TESSFACTOR,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_hs_max_tessfactor
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_HS_FORK_PHASE_INSTANCE_COUNT,
            handler_idx: WINED3DSIH_DCL_HS_FORK_PHASE_INSTANCE_COUNT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_HS_JOIN_PHASE_INSTANCE_COUNT,
            handler_idx: WINED3DSIH_DCL_HS_JOIN_PHASE_INSTANCE_COUNT,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_THREAD_GROUP,
            handler_idx: WINED3DSIH_DCL_THREAD_GROUP,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_thread_group
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_UAV_TYPED,
            handler_idx: WINED3DSIH_DCL_UAV_TYPED,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_dcl_resource
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_UAV_RAW,
            handler_idx: WINED3DSIH_DCL_UAV_RAW,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_uav_raw
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_UAV_STRUCTURED,
            handler_idx: WINED3DSIH_DCL_UAV_STRUCTURED,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_uav_structured
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_TGSM_RAW,
            handler_idx: WINED3DSIH_DCL_TGSM_RAW,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_tgsm_raw
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_TGSM_STRUCTURED,
            handler_idx: WINED3DSIH_DCL_TGSM_STRUCTURED,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_tgsm_structured
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_RESOURCE_RAW,
            handler_idx: WINED3DSIH_DCL_RESOURCE_RAW,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_resource_raw
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_RESOURCE_STRUCTURED,
            handler_idx: WINED3DSIH_DCL_RESOURCE_STRUCTURED,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_dcl_resource_structured
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_LD_UAV_TYPED,
            handler_idx: WINED3DSIH_LD_UAV_TYPED,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iU\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_STORE_UAV_TYPED,
            handler_idx: WINED3DSIH_STORE_UAV_TYPED,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_LD_RAW,
            handler_idx: WINED3DSIH_LD_RAW,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iU\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_STORE_RAW,
            handler_idx: WINED3DSIH_STORE_RAW,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_LD_STRUCTURED,
            handler_idx: WINED3DSIH_LD_STRUCTURED,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iiR\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_STORE_STRUCTURED,
            handler_idx: WINED3DSIH_STORE_STRUCTURED,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iiu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_AND,
            handler_idx: WINED3DSIH_ATOMIC_AND,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_OR,
            handler_idx: WINED3DSIH_ATOMIC_OR,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_XOR,
            handler_idx: WINED3DSIH_ATOMIC_XOR,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_CMP_STORE,
            handler_idx: WINED3DSIH_ATOMIC_CMP_STORE,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iuu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_IADD,
            handler_idx: WINED3DSIH_ATOMIC_IADD,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_IMAX,
            handler_idx: WINED3DSIH_ATOMIC_IMAX,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_IMIN,
            handler_idx: WINED3DSIH_ATOMIC_IMIN,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_UMAX,
            handler_idx: WINED3DSIH_ATOMIC_UMAX,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_ATOMIC_UMIN,
            handler_idx: WINED3DSIH_ATOMIC_UMIN,
            dst_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_ALLOC,
            handler_idx: WINED3DSIH_IMM_ATOMIC_ALLOC,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_CONSUME,
            handler_idx: WINED3DSIH_IMM_ATOMIC_CONSUME,
            dst_info: b"u\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"U\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_IADD,
            handler_idx: WINED3DSIH_IMM_ATOMIC_IADD,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_AND,
            handler_idx: WINED3DSIH_IMM_ATOMIC_AND,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_OR,
            handler_idx: WINED3DSIH_IMM_ATOMIC_OR,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_XOR,
            handler_idx: WINED3DSIH_IMM_ATOMIC_XOR,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_EXCH,
            handler_idx: WINED3DSIH_IMM_ATOMIC_EXCH,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_CMP_EXCH,
            handler_idx: WINED3DSIH_IMM_ATOMIC_CMP_EXCH,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iuu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_IMAX,
            handler_idx: WINED3DSIH_IMM_ATOMIC_IMAX,
            dst_info: b"iU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_IMIN,
            handler_idx: WINED3DSIH_IMM_ATOMIC_IMIN,
            dst_info: b"iU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"ii\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_UMAX,
            handler_idx: WINED3DSIH_IMM_ATOMIC_UMAX,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_IMM_ATOMIC_UMIN,
            handler_idx: WINED3DSIH_IMM_ATOMIC_UMIN,
            dst_info: b"uU\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"iu\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_SYNC,
            handler_idx: WINED3DSIH_SYNC,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm5_read_sync
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_EVAL_SAMPLE_INDEX,
            handler_idx: WINED3DSIH_EVAL_SAMPLE_INDEX,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"fi\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_EVAL_CENTROID,
            handler_idx: WINED3DSIH_EVAL_CENTROID,
            dst_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"f\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: None,
        },
        wined3d_sm4_opcode_info {
            opcode: WINED3D_SM5_OP_DCL_GS_INSTANCES,
            handler_idx: WINED3DSIH_DCL_GS_INSTANCES,
            dst_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            src_info: b"\0" as *const u8 as *const ::core::ffi::c_char,
            read_opcode_func: Some(
                shader_sm4_read_declaration_count
                    as unsafe extern "C" fn(
                        *mut wined3d_shader_instruction,
                        DWORD,
                        DWORD,
                        *const DWORD,
                        ::core::ffi::c_uint,
                        *mut wined3d_sm4_data,
                    ) -> (),
            ),
        },
    ]
};

static mut register_type_table: [wined3d_shader_register_type; 42] = [
    WINED3DSPR_TEMP,
    WINED3DSPR_INPUT,
    WINED3DSPR_OUTPUT,
    WINED3DSPR_IDXTEMP,
    WINED3DSPR_IMMCONST,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_SAMPLER,
    WINED3DSPR_RESOURCE,
    WINED3DSPR_CONSTBUFFER,
    WINED3DSPR_IMMCONSTBUFFER,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_PRIMID,
    WINED3DSPR_DEPTHOUT,
    WINED3DSPR_NULL,
    WINED3DSPR_RASTERIZER,
    WINED3DSPR_SAMPLEMASK,
    WINED3DSPR_STREAM,
    WINED3DSPR_FUNCTIONBODY,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_FUNCTIONPOINTER,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_OUTPOINTID,
    WINED3DSPR_FORKINSTID,
    WINED3DSPR_JOININSTID,
    WINED3DSPR_INCONTROLPOINT,
    WINED3DSPR_OUTCONTROLPOINT,
    WINED3DSPR_PATCHCONST,
    WINED3DSPR_TESSCOORD,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_UAV,
    WINED3DSPR_GROUPSHAREDMEM,
    WINED3DSPR_THREADID,
    WINED3DSPR_THREADGROUPID,
    WINED3DSPR_LOCALTHREADID,
    WINED3DSPR_COVERAGE,
    WINED3DSPR_LOCALTHREADINDEX,
    WINED3DSPR_GSINSTID,
    WINED3DSPR_DEPTHOUTGE,
    WINED3DSPR_DEPTHOUTLE,
    !(0 as ::core::ffi::c_uint) as wined3d_shader_register_type,
    WINED3DSPR_STENCILREF,
];

#[no_mangle]
pub static mut sm4_shader_frontend: wined3d_shader_frontend = unsafe {
    wined3d_shader_frontend {
        shader_init: Some(
            shader_sm4_init
                as unsafe extern "C" fn(
                    *const DWORD,
                    size_t,
                    *const wined3d_shader_signature,
                ) -> *mut ::core::ffi::c_void,
        ),
        shader_free: Some(shader_sm4_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        shader_read_header: Some(
            shader_sm4_read_header
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut *const DWORD,
                    *mut wined3d_shader_version,
                ) -> (),
        ),
        shader_read_instruction: Some(
            shader_sm4_read_instruction
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut *const DWORD,
                    *mut wined3d_shader_instruction,
                ) -> (),
        ),
        shader_is_end: Some(
            shader_sm4_is_end
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut *const DWORD) -> BOOL,
        ),
    }
};

// LIBVKD3D_SHADER_SOURCE: empty-valued marker define, no Rust equivalent needed.
// WINE_DEFAULT_DEBUG_CHANNEL / WINE_DECLARE_DEBUG_CHANNEL: debug-channel registration, not emitted.

pub const WINED3D_SM4_INSTRUCTION_MODIFIER: u32 = 0x80000000;
pub const WINED3D_SM4_MODIFIER_MASK: u32 = 0x3f;
pub const WINED3D_SM5_MODIFIER_DATA_TYPE_SHIFT: u32 = 6;
pub const WINED3D_SM5_MODIFIER_DATA_TYPE_MASK: u32 = 0x3fffc0;
pub const WINED3D_SM5_MODIFIER_RESOURCE_TYPE_SHIFT: u32 = 6;
pub const WINED3D_SM5_MODIFIER_RESOURCE_TYPE_MASK: u32 = 0x3c0;
pub const WINED3D_SM4_AOFFIMMI_U_SHIFT: u32 = 9;
pub const WINED3D_SM4_AOFFIMMI_U_MASK: u32 = 0x1e00;
pub const WINED3D_SM4_AOFFIMMI_V_SHIFT: u32 = 13;
pub const WINED3D_SM4_AOFFIMMI_V_MASK: u32 = 0x1e000;
pub const WINED3D_SM4_AOFFIMMI_W_SHIFT: u32 = 17;
pub const WINED3D_SM4_AOFFIMMI_W_MASK: u32 = 0x1e0000;
pub const WINED3D_SM4_INSTRUCTION_LENGTH_SHIFT: u32 = 24;
pub const WINED3D_SM4_INSTRUCTION_LENGTH_MASK: u32 = 0x1f000000;
pub const WINED3D_SM4_INSTRUCTION_FLAGS_SHIFT: u32 = 11;
pub const WINED3D_SM4_INSTRUCTION_FLAGS_MASK: u32 = 0x3800;
pub const WINED3D_SM4_RESOURCE_TYPE_SHIFT: u32 = 11;
pub const WINED3D_SM4_RESOURCE_TYPE_MASK: u32 = 0x7800;
pub const WINED3D_SM4_RESOURCE_SAMPLE_COUNT_SHIFT: u32 = 16;
pub const WINED3D_SM4_RESOURCE_SAMPLE_COUNT_MASK: u32 = 0xf0000;
pub const WINED3D_SM4_PRIMITIVE_TYPE_SHIFT: u32 = 11;
pub const WINED3D_SM4_PRIMITIVE_TYPE_MASK: u32 = 0x1f800;
pub const WINED3D_SM4_INDEX_TYPE_SHIFT: u32 = 11;
pub const WINED3D_SM4_INDEX_TYPE_MASK: u32 = 0x800;
pub const WINED3D_SM4_SAMPLER_MODE_SHIFT: u32 = 11;
pub const WINED3D_SM4_SAMPLER_MODE_MASK: u32 = 0x7800;
pub const WINED3D_SM4_SHADER_DATA_TYPE_SHIFT: u32 = 11;
pub const WINED3D_SM4_SHADER_DATA_TYPE_MASK: u32 = 0x7800;
pub const WINED3D_SM4_INTERPOLATION_MODE_SHIFT: u32 = 11;
pub const WINED3D_SM4_INTERPOLATION_MODE_MASK: u32 = 0x7800;
pub const WINED3D_SM4_GLOBAL_FLAGS_SHIFT: u32 = 11;
pub const WINED3D_SM4_GLOBAL_FLAGS_MASK: u32 = 0x7f800;
pub const WINED3D_SM5_PRECISE_SHIFT: u32 = 19;
pub const WINED3D_SM5_PRECISE_MASK: u32 = 0x780000;
pub const WINED3D_SM5_CONTROL_POINT_COUNT_SHIFT: u32 = 11;
pub const WINED3D_SM5_CONTROL_POINT_COUNT_MASK: u32 = 0x7f800;
pub const WINED3D_SM5_FP_ARRAY_SIZE_SHIFT: u32 = 16;
pub const WINED3D_SM5_FP_TABLE_COUNT_MASK: u32 = 0xffff;
pub const WINED3D_SM5_UAV_FLAGS_SHIFT: u32 = 15;
pub const WINED3D_SM5_UAV_FLAGS_MASK: u32 = 0xff8000;
pub const WINED3D_SM5_SYNC_FLAGS_SHIFT: u32 = 11;
pub const WINED3D_SM5_SYNC_FLAGS_MASK: u32 = 0x7f800;
pub const WINED3D_SM5_TESSELLATOR_SHIFT: u32 = 11;
pub const WINED3D_SM5_TESSELLATOR_MASK: u32 = 0x7800;
pub const WINED3D_SM4_OPCODE_MASK: u32 = 0xff;
pub const WINED3D_SM4_REGISTER_MODIFIER: u32 = 0x80000000;
pub const WINED3D_SM4_ADDRESSING_SHIFT1: u32 = 25;
pub const WINED3D_SM4_ADDRESSING_MASK1: u32 = 0x6000000;
pub const WINED3D_SM4_ADDRESSING_SHIFT0: u32 = 22;
pub const WINED3D_SM4_ADDRESSING_MASK0: u32 = 0xc00000;
pub const WINED3D_SM4_REGISTER_ORDER_SHIFT: u32 = 20;
pub const WINED3D_SM4_REGISTER_ORDER_MASK: u32 = 0x300000;
pub const WINED3D_SM4_REGISTER_TYPE_SHIFT: u32 = 12;
pub const WINED3D_SM4_REGISTER_TYPE_MASK: u32 = 0xff000;
pub const WINED3D_SM4_SWIZZLE_TYPE_SHIFT: u32 = 2;
pub const WINED3D_SM4_SWIZZLE_TYPE_MASK: u32 = 0xc;
pub const WINED3D_SM4_DIMENSION_SHIFT: u32 = 0;
pub const WINED3D_SM4_DIMENSION_MASK: u32 = 0x3;
pub const WINED3D_SM4_WRITEMASK_SHIFT: u32 = 4;
pub const WINED3D_SM4_WRITEMASK_MASK: u32 = 0xf0;
pub const WINED3D_SM4_SWIZZLE_SHIFT: u32 = 4;
pub const WINED3D_SM4_SWIZZLE_MASK: u32 = 0xff0;
// WINED3D_SM4_VERSION_MAJOR / WINED3D_SM4_VERSION_MINOR: function-like macros, skipped.
pub const WINED3D_SM4_ADDRESSING_RELATIVE: u32 = 0x2;
pub const WINED3D_SM4_ADDRESSING_OFFSET: u32 = 0x1;
pub const WINED3D_SM4_INSTRUCTION_FLAG_SATURATE: u32 = 0x4;
pub const WINED3D_SM4_CONDITIONAL_NZ: u32 = 0x40000;

pub const WINED3D_SM4_OP_ADD: u32 = 0x00;
pub const WINED3D_SM4_OP_AND: u32 = 0x01;
pub const WINED3D_SM4_OP_BREAK: u32 = 0x02;
pub const WINED3D_SM4_OP_BREAKC: u32 = 0x03;
pub const WINED3D_SM4_OP_CASE: u32 = 0x06;
pub const WINED3D_SM4_OP_CONTINUE: u32 = 0x07;
pub const WINED3D_SM4_OP_CONTINUEC: u32 = 0x08;
pub const WINED3D_SM4_OP_CUT: u32 = 0x09;
pub const WINED3D_SM4_OP_DEFAULT: u32 = 0x0a;
pub const WINED3D_SM4_OP_DERIV_RTX: u32 = 0x0b;
pub const WINED3D_SM4_OP_DERIV_RTY: u32 = 0x0c;
pub const WINED3D_SM4_OP_DISCARD: u32 = 0x0d;
pub const WINED3D_SM4_OP_DIV: u32 = 0x0e;
pub const WINED3D_SM4_OP_DP2: u32 = 0x0f;
pub const WINED3D_SM4_OP_DP3: u32 = 0x10;
pub const WINED3D_SM4_OP_DP4: u32 = 0x11;
pub const WINED3D_SM4_OP_ELSE: u32 = 0x12;
pub const WINED3D_SM4_OP_EMIT: u32 = 0x13;
pub const WINED3D_SM4_OP_ENDIF: u32 = 0x15;
pub const WINED3D_SM4_OP_ENDLOOP: u32 = 0x16;
pub const WINED3D_SM4_OP_ENDSWITCH: u32 = 0x17;
pub const WINED3D_SM4_OP_EQ: u32 = 0x18;
pub const WINED3D_SM4_OP_EXP: u32 = 0x19;
pub const WINED3D_SM4_OP_FRC: u32 = 0x1a;
pub const WINED3D_SM4_OP_FTOI: u32 = 0x1b;
pub const WINED3D_SM4_OP_FTOU: u32 = 0x1c;
pub const WINED3D_SM4_OP_GE: u32 = 0x1d;
pub const WINED3D_SM4_OP_IADD: u32 = 0x1e;
pub const WINED3D_SM4_OP_IF: u32 = 0x1f;
pub const WINED3D_SM4_OP_IEQ: u32 = 0x20;
pub const WINED3D_SM4_OP_IGE: u32 = 0x21;
pub const WINED3D_SM4_OP_ILT: u32 = 0x22;
pub const WINED3D_SM4_OP_IMAD: u32 = 0x23;
pub const WINED3D_SM4_OP_IMAX: u32 = 0x24;
pub const WINED3D_SM4_OP_IMIN: u32 = 0x25;
pub const WINED3D_SM4_OP_IMUL: u32 = 0x26;
pub const WINED3D_SM4_OP_INE: u32 = 0x27;
pub const WINED3D_SM4_OP_INEG: u32 = 0x28;
pub const WINED3D_SM4_OP_ISHL: u32 = 0x29;
pub const WINED3D_SM4_OP_ISHR: u32 = 0x2a;
pub const WINED3D_SM4_OP_ITOF: u32 = 0x2b;
pub const WINED3D_SM4_OP_LABEL: u32 = 0x2c;
pub const WINED3D_SM4_OP_LD: u32 = 0x2d;
pub const WINED3D_SM4_OP_LD2DMS: u32 = 0x2e;
pub const WINED3D_SM4_OP_LOG: u32 = 0x2f;
pub const WINED3D_SM4_OP_LOOP: u32 = 0x30;
pub const WINED3D_SM4_OP_LT: u32 = 0x31;
pub const WINED3D_SM4_OP_MAD: u32 = 0x32;
pub const WINED3D_SM4_OP_MIN: u32 = 0x33;
pub const WINED3D_SM4_OP_MAX: u32 = 0x34;
pub const WINED3D_SM4_OP_SHADER_DATA: u32 = 0x35;
pub const WINED3D_SM4_OP_MOV: u32 = 0x36;
pub const WINED3D_SM4_OP_MOVC: u32 = 0x37;
pub const WINED3D_SM4_OP_MUL: u32 = 0x38;
pub const WINED3D_SM4_OP_NE: u32 = 0x39;
pub const WINED3D_SM4_OP_NOP: u32 = 0x3a;
pub const WINED3D_SM4_OP_NOT: u32 = 0x3b;
pub const WINED3D_SM4_OP_OR: u32 = 0x3c;
pub const WINED3D_SM4_OP_RESINFO: u32 = 0x3d;
pub const WINED3D_SM4_OP_RET: u32 = 0x3e;
pub const WINED3D_SM4_OP_RETC: u32 = 0x3f;
pub const WINED3D_SM4_OP_ROUND_NE: u32 = 0x40;
pub const WINED3D_SM4_OP_ROUND_NI: u32 = 0x41;
pub const WINED3D_SM4_OP_ROUND_PI: u32 = 0x42;
pub const WINED3D_SM4_OP_ROUND_Z: u32 = 0x43;
pub const WINED3D_SM4_OP_RSQ: u32 = 0x44;
pub const WINED3D_SM4_OP_SAMPLE: u32 = 0x45;
pub const WINED3D_SM4_OP_SAMPLE_C: u32 = 0x46;
pub const WINED3D_SM4_OP_SAMPLE_C_LZ: u32 = 0x47;
pub const WINED3D_SM4_OP_SAMPLE_LOD: u32 = 0x48;
pub const WINED3D_SM4_OP_SAMPLE_GRAD: u32 = 0x49;
pub const WINED3D_SM4_OP_SAMPLE_B: u32 = 0x4a;
pub const WINED3D_SM4_OP_SQRT: u32 = 0x4b;
pub const WINED3D_SM4_OP_SWITCH: u32 = 0x4c;
pub const WINED3D_SM4_OP_SINCOS: u32 = 0x4d;
pub const WINED3D_SM4_OP_UDIV: u32 = 0x4e;
pub const WINED3D_SM4_OP_ULT: u32 = 0x4f;
pub const WINED3D_SM4_OP_UGE: u32 = 0x50;
pub const WINED3D_SM4_OP_UMUL: u32 = 0x51;
pub const WINED3D_SM4_OP_UMAX: u32 = 0x53;
pub const WINED3D_SM4_OP_UMIN: u32 = 0x54;
pub const WINED3D_SM4_OP_USHR: u32 = 0x55;
pub const WINED3D_SM4_OP_UTOF: u32 = 0x56;
pub const WINED3D_SM4_OP_XOR: u32 = 0x57;
pub const WINED3D_SM4_OP_DCL_RESOURCE: u32 = 0x58;
pub const WINED3D_SM4_OP_DCL_CONSTANT_BUFFER: u32 = 0x59;
pub const WINED3D_SM4_OP_DCL_SAMPLER: u32 = 0x5a;
pub const WINED3D_SM4_OP_DCL_INDEX_RANGE: u32 = 0x5b;
pub const WINED3D_SM4_OP_DCL_OUTPUT_TOPOLOGY: u32 = 0x5c;
pub const WINED3D_SM4_OP_DCL_INPUT_PRIMITIVE: u32 = 0x5d;
pub const WINED3D_SM4_OP_DCL_VERTICES_OUT: u32 = 0x5e;
pub const WINED3D_SM4_OP_DCL_INPUT: u32 = 0x5f;
pub const WINED3D_SM4_OP_DCL_INPUT_SGV: u32 = 0x60;
pub const WINED3D_SM4_OP_DCL_INPUT_SIV: u32 = 0x61;
pub const WINED3D_SM4_OP_DCL_INPUT_PS: u32 = 0x62;
pub const WINED3D_SM4_OP_DCL_INPUT_PS_SGV: u32 = 0x63;
pub const WINED3D_SM4_OP_DCL_INPUT_PS_SIV: u32 = 0x64;
pub const WINED3D_SM4_OP_DCL_OUTPUT: u32 = 0x65;
pub const WINED3D_SM4_OP_DCL_OUTPUT_SIV: u32 = 0x67;
pub const WINED3D_SM4_OP_DCL_TEMPS: u32 = 0x68;
pub const WINED3D_SM4_OP_DCL_INDEXABLE_TEMP: u32 = 0x69;
pub const WINED3D_SM4_OP_DCL_GLOBAL_FLAGS: u32 = 0x6a;
pub const WINED3D_SM4_OP_LOD: u32 = 0x6c;
pub const WINED3D_SM4_OP_GATHER4: u32 = 0x6d;
pub const WINED3D_SM4_OP_SAMPLE_POS: u32 = 0x6e;
pub const WINED3D_SM4_OP_SAMPLE_INFO: u32 = 0x6f;
pub const WINED3D_SM5_OP_HS_DECLS: u32 = 0x71;
pub const WINED3D_SM5_OP_HS_CONTROL_POINT_PHASE: u32 = 0x72;
pub const WINED3D_SM5_OP_HS_FORK_PHASE: u32 = 0x73;
pub const WINED3D_SM5_OP_HS_JOIN_PHASE: u32 = 0x74;
pub const WINED3D_SM5_OP_EMIT_STREAM: u32 = 0x75;
pub const WINED3D_SM5_OP_CUT_STREAM: u32 = 0x76;
pub const WINED3D_SM5_OP_FCALL: u32 = 0x78;
pub const WINED3D_SM5_OP_BUFINFO: u32 = 0x79;
pub const WINED3D_SM5_OP_DERIV_RTX_COARSE: u32 = 0x7a;
pub const WINED3D_SM5_OP_DERIV_RTX_FINE: u32 = 0x7b;
pub const WINED3D_SM5_OP_DERIV_RTY_COARSE: u32 = 0x7c;
pub const WINED3D_SM5_OP_DERIV_RTY_FINE: u32 = 0x7d;
pub const WINED3D_SM5_OP_GATHER4_C: u32 = 0x7e;
pub const WINED3D_SM5_OP_GATHER4_PO: u32 = 0x7f;
pub const WINED3D_SM5_OP_GATHER4_PO_C: u32 = 0x80;
pub const WINED3D_SM5_OP_RCP: u32 = 0x81;
pub const WINED3D_SM5_OP_F32TOF16: u32 = 0x82;
pub const WINED3D_SM5_OP_F16TOF32: u32 = 0x83;
pub const WINED3D_SM5_OP_COUNTBITS: u32 = 0x86;
pub const WINED3D_SM5_OP_FIRSTBIT_HI: u32 = 0x87;
pub const WINED3D_SM5_OP_FIRSTBIT_LO: u32 = 0x88;
pub const WINED3D_SM5_OP_FIRSTBIT_SHI: u32 = 0x89;
pub const WINED3D_SM5_OP_UBFE: u32 = 0x8a;
pub const WINED3D_SM5_OP_IBFE: u32 = 0x8b;
pub const WINED3D_SM5_OP_BFI: u32 = 0x8c;
pub const WINED3D_SM5_OP_BFREV: u32 = 0x8d;
pub const WINED3D_SM5_OP_SWAPC: u32 = 0x8e;
pub const WINED3D_SM5_OP_DCL_STREAM: u32 = 0x8f;
pub const WINED3D_SM5_OP_DCL_FUNCTION_BODY: u32 = 0x90;
pub const WINED3D_SM5_OP_DCL_FUNCTION_TABLE: u32 = 0x91;
pub const WINED3D_SM5_OP_DCL_INTERFACE: u32 = 0x92;
pub const WINED3D_SM5_OP_DCL_INPUT_CONTROL_POINT_COUNT: u32 = 0x93;
pub const WINED3D_SM5_OP_DCL_OUTPUT_CONTROL_POINT_COUNT: u32 = 0x94;
pub const WINED3D_SM5_OP_DCL_TESSELLATOR_DOMAIN: u32 = 0x95;
pub const WINED3D_SM5_OP_DCL_TESSELLATOR_PARTITIONING: u32 = 0x96;
pub const WINED3D_SM5_OP_DCL_TESSELLATOR_OUTPUT_PRIMITIVE: u32 = 0x97;
pub const WINED3D_SM5_OP_DCL_HS_MAX_TESSFACTOR: u32 = 0x98;
pub const WINED3D_SM5_OP_DCL_HS_FORK_PHASE_INSTANCE_COUNT: u32 = 0x99;
pub const WINED3D_SM5_OP_DCL_HS_JOIN_PHASE_INSTANCE_COUNT: u32 = 0x9a;
pub const WINED3D_SM5_OP_DCL_THREAD_GROUP: u32 = 0x9b;
pub const WINED3D_SM5_OP_DCL_UAV_TYPED: u32 = 0x9c;
pub const WINED3D_SM5_OP_DCL_UAV_RAW: u32 = 0x9d;
pub const WINED3D_SM5_OP_DCL_UAV_STRUCTURED: u32 = 0x9e;
pub const WINED3D_SM5_OP_DCL_TGSM_RAW: u32 = 0x9f;
pub const WINED3D_SM5_OP_DCL_TGSM_STRUCTURED: u32 = 0xa0;
pub const WINED3D_SM5_OP_DCL_RESOURCE_RAW: u32 = 0xa1;
pub const WINED3D_SM5_OP_DCL_RESOURCE_STRUCTURED: u32 = 0xa2;
pub const WINED3D_SM5_OP_LD_UAV_TYPED: u32 = 0xa3;
pub const WINED3D_SM5_OP_STORE_UAV_TYPED: u32 = 0xa4;
pub const WINED3D_SM5_OP_LD_RAW: u32 = 0xa5;
pub const WINED3D_SM5_OP_STORE_RAW: u32 = 0xa6;
pub const WINED3D_SM5_OP_LD_STRUCTURED: u32 = 0xa7;
pub const WINED3D_SM5_OP_STORE_STRUCTURED: u32 = 0xa8;
pub const WINED3D_SM5_OP_ATOMIC_AND: u32 = 0xa9;
pub const WINED3D_SM5_OP_ATOMIC_OR: u32 = 0xaa;
pub const WINED3D_SM5_OP_ATOMIC_XOR: u32 = 0xab;
pub const WINED3D_SM5_OP_ATOMIC_CMP_STORE: u32 = 0xac;
pub const WINED3D_SM5_OP_ATOMIC_IADD: u32 = 0xad;
pub const WINED3D_SM5_OP_ATOMIC_IMAX: u32 = 0xae;
pub const WINED3D_SM5_OP_ATOMIC_IMIN: u32 = 0xaf;
pub const WINED3D_SM5_OP_ATOMIC_UMAX: u32 = 0xb0;
pub const WINED3D_SM5_OP_ATOMIC_UMIN: u32 = 0xb1;
pub const WINED3D_SM5_OP_IMM_ATOMIC_ALLOC: u32 = 0xb2;
pub const WINED3D_SM5_OP_IMM_ATOMIC_CONSUME: u32 = 0xb3;
pub const WINED3D_SM5_OP_IMM_ATOMIC_IADD: u32 = 0xb4;
pub const WINED3D_SM5_OP_IMM_ATOMIC_AND: u32 = 0xb5;
pub const WINED3D_SM5_OP_IMM_ATOMIC_OR: u32 = 0xb6;
pub const WINED3D_SM5_OP_IMM_ATOMIC_XOR: u32 = 0xb7;
pub const WINED3D_SM5_OP_IMM_ATOMIC_EXCH: u32 = 0xb8;
pub const WINED3D_SM5_OP_IMM_ATOMIC_CMP_EXCH: u32 = 0xb9;
pub const WINED3D_SM5_OP_IMM_ATOMIC_IMAX: u32 = 0xba;
pub const WINED3D_SM5_OP_IMM_ATOMIC_IMIN: u32 = 0xbb;
pub const WINED3D_SM5_OP_IMM_ATOMIC_UMAX: u32 = 0xbc;
pub const WINED3D_SM5_OP_IMM_ATOMIC_UMIN: u32 = 0xbd;
pub const WINED3D_SM5_OP_SYNC: u32 = 0xbe;
pub const WINED3D_SM5_OP_EVAL_SAMPLE_INDEX: u32 = 0xcc;
pub const WINED3D_SM5_OP_EVAL_CENTROID: u32 = 0xcd;
pub const WINED3D_SM5_OP_DCL_GS_INSTANCES: u32 = 0xce;

pub const WINED3D_SM4_MODIFIER_AOFFIMMI: u32 = 0x1;
pub const WINED3D_SM5_MODIFIER_RESOURCE_TYPE: u32 = 0x2;
pub const WINED3D_SM5_MODIFIER_DATA_TYPE: u32 = 0x3;

pub const WINED3D_SM4_RT_TEMP: u32 = 0x00;
pub const WINED3D_SM4_RT_INPUT: u32 = 0x01;
pub const WINED3D_SM4_RT_OUTPUT: u32 = 0x02;
pub const WINED3D_SM4_RT_INDEXABLE_TEMP: u32 = 0x03;
pub const WINED3D_SM4_RT_IMMCONST: u32 = 0x04;
pub const WINED3D_SM4_RT_SAMPLER: u32 = 0x06;
pub const WINED3D_SM4_RT_RESOURCE: u32 = 0x07;
pub const WINED3D_SM4_RT_CONSTBUFFER: u32 = 0x08;
pub const WINED3D_SM4_RT_IMMCONSTBUFFER: u32 = 0x09;
pub const WINED3D_SM4_RT_PRIMID: u32 = 0x0b;
pub const WINED3D_SM4_RT_DEPTHOUT: u32 = 0x0c;
pub const WINED3D_SM4_RT_NULL: u32 = 0x0d;
pub const WINED3D_SM4_RT_RASTERIZER: u32 = 0x0e;
pub const WINED3D_SM4_RT_OMASK: u32 = 0x0f;
pub const WINED3D_SM5_RT_STREAM: u32 = 0x10;
pub const WINED3D_SM5_RT_FUNCTION_BODY: u32 = 0x11;
pub const WINED3D_SM5_RT_FUNCTION_POINTER: u32 = 0x13;
pub const WINED3D_SM5_RT_OUTPUT_CONTROL_POINT_ID: u32 = 0x16;
pub const WINED3D_SM5_RT_FORK_INSTANCE_ID: u32 = 0x17;
pub const WINED3D_SM5_RT_JOIN_INSTANCE_ID: u32 = 0x18;
pub const WINED3D_SM5_RT_INPUT_CONTROL_POINT: u32 = 0x19;
pub const WINED3D_SM5_RT_OUTPUT_CONTROL_POINT: u32 = 0x1a;
pub const WINED3D_SM5_RT_PATCH_CONSTANT_DATA: u32 = 0x1b;
pub const WINED3D_SM5_RT_DOMAIN_LOCATION: u32 = 0x1c;
pub const WINED3D_SM5_RT_UAV: u32 = 0x1e;
pub const WINED3D_SM5_RT_SHARED_MEMORY: u32 = 0x1f;
pub const WINED3D_SM5_RT_THREAD_ID: u32 = 0x20;
pub const WINED3D_SM5_RT_THREAD_GROUP_ID: u32 = 0x21;
pub const WINED3D_SM5_RT_LOCAL_THREAD_ID: u32 = 0x22;
pub const WINED3D_SM5_RT_COVERAGE: u32 = 0x23;
pub const WINED3D_SM5_RT_LOCAL_THREAD_INDEX: u32 = 0x24;
pub const WINED3D_SM5_RT_GS_INSTANCE_ID: u32 = 0x25;
pub const WINED3D_SM5_RT_DEPTHOUT_GREATER_EQUAL: u32 = 0x26;
pub const WINED3D_SM5_RT_DEPTHOUT_LESS_EQUAL: u32 = 0x27;
pub const WINED3D_SM5_RT_OUTPUT_STENCIL_REF: u32 = 0x29;

pub const WINED3D_SM4_OUTPUT_PT_POINTLIST: u32 = 0x1;
pub const WINED3D_SM4_OUTPUT_PT_LINESTRIP: u32 = 0x3;
pub const WINED3D_SM4_OUTPUT_PT_TRIANGLESTRIP: u32 = 0x5;

pub const WINED3D_SM4_INPUT_PT_POINT: u32 = 0x01;
pub const WINED3D_SM4_INPUT_PT_LINE: u32 = 0x02;
pub const WINED3D_SM4_INPUT_PT_TRIANGLE: u32 = 0x03;
pub const WINED3D_SM4_INPUT_PT_LINEADJ: u32 = 0x06;
pub const WINED3D_SM4_INPUT_PT_TRIANGLEADJ: u32 = 0x07;
pub const WINED3D_SM5_INPUT_PT_PATCH1: u32 = 0x08;
pub const WINED3D_SM5_INPUT_PT_PATCH2: u32 = 0x09;
pub const WINED3D_SM5_INPUT_PT_PATCH3: u32 = 0x0a;
pub const WINED3D_SM5_INPUT_PT_PATCH4: u32 = 0x0b;
pub const WINED3D_SM5_INPUT_PT_PATCH5: u32 = 0x0c;
pub const WINED3D_SM5_INPUT_PT_PATCH6: u32 = 0x0d;
pub const WINED3D_SM5_INPUT_PT_PATCH7: u32 = 0x0e;
pub const WINED3D_SM5_INPUT_PT_PATCH8: u32 = 0x0f;
pub const WINED3D_SM5_INPUT_PT_PATCH9: u32 = 0x10;
pub const WINED3D_SM5_INPUT_PT_PATCH10: u32 = 0x11;
pub const WINED3D_SM5_INPUT_PT_PATCH11: u32 = 0x12;
pub const WINED3D_SM5_INPUT_PT_PATCH12: u32 = 0x13;
pub const WINED3D_SM5_INPUT_PT_PATCH13: u32 = 0x14;
pub const WINED3D_SM5_INPUT_PT_PATCH14: u32 = 0x15;
pub const WINED3D_SM5_INPUT_PT_PATCH15: u32 = 0x16;
pub const WINED3D_SM5_INPUT_PT_PATCH16: u32 = 0x17;
pub const WINED3D_SM5_INPUT_PT_PATCH17: u32 = 0x18;
pub const WINED3D_SM5_INPUT_PT_PATCH18: u32 = 0x19;
pub const WINED3D_SM5_INPUT_PT_PATCH19: u32 = 0x1a;
pub const WINED3D_SM5_INPUT_PT_PATCH20: u32 = 0x1b;
pub const WINED3D_SM5_INPUT_PT_PATCH21: u32 = 0x1c;
pub const WINED3D_SM5_INPUT_PT_PATCH22: u32 = 0x1d;
pub const WINED3D_SM5_INPUT_PT_PATCH23: u32 = 0x1e;
pub const WINED3D_SM5_INPUT_PT_PATCH24: u32 = 0x1f;
pub const WINED3D_SM5_INPUT_PT_PATCH25: u32 = 0x20;
pub const WINED3D_SM5_INPUT_PT_PATCH26: u32 = 0x21;
pub const WINED3D_SM5_INPUT_PT_PATCH27: u32 = 0x22;
pub const WINED3D_SM5_INPUT_PT_PATCH28: u32 = 0x23;
pub const WINED3D_SM5_INPUT_PT_PATCH29: u32 = 0x24;
pub const WINED3D_SM5_INPUT_PT_PATCH30: u32 = 0x25;
pub const WINED3D_SM5_INPUT_PT_PATCH31: u32 = 0x26;
pub const WINED3D_SM5_INPUT_PT_PATCH32: u32 = 0x27;

pub const WINED3D_SM4_SWIZZLE_NONE: u32 = 0x0;
pub const WINED3D_SM4_SWIZZLE_VEC4: u32 = 0x1;
pub const WINED3D_SM4_SWIZZLE_SCALAR: u32 = 0x2;

pub const WINED3D_SM4_DIMENSION_SCALAR: u32 = 0x1;
pub const WINED3D_SM4_DIMENSION_VEC4: u32 = 0x2;

pub const WINED3D_SM4_RESOURCE_BUFFER: u32 = 0x1;
pub const WINED3D_SM4_RESOURCE_TEXTURE_1D: u32 = 0x2;
pub const WINED3D_SM4_RESOURCE_TEXTURE_2D: u32 = 0x3;
pub const WINED3D_SM4_RESOURCE_TEXTURE_2DMS: u32 = 0x4;
pub const WINED3D_SM4_RESOURCE_TEXTURE_3D: u32 = 0x5;
pub const WINED3D_SM4_RESOURCE_TEXTURE_CUBE: u32 = 0x6;
pub const WINED3D_SM4_RESOURCE_TEXTURE_1DARRAY: u32 = 0x7;
pub const WINED3D_SM4_RESOURCE_TEXTURE_2DARRAY: u32 = 0x8;
pub const WINED3D_SM4_RESOURCE_TEXTURE_2DMSARRAY: u32 = 0x9;
pub const WINED3D_SM4_RESOURCE_TEXTURE_CUBEARRAY: u32 = 0xa;

pub const WINED3D_SM4_DATA_UNORM: u32 = 0x1;
pub const WINED3D_SM4_DATA_SNORM: u32 = 0x2;
pub const WINED3D_SM4_DATA_INT: u32 = 0x3;
pub const WINED3D_SM4_DATA_UINT: u32 = 0x4;
pub const WINED3D_SM4_DATA_FLOAT: u32 = 0x5;

pub const WINED3D_SM4_SAMPLER_DEFAULT: u32 = 0x0;
pub const WINED3D_SM4_SAMPLER_COMPARISON: u32 = 0x1;

pub const WINED3D_SM4_SHADER_DATA_IMMEDIATE_CONSTANT_BUFFER: u32 = 0x3;
pub const WINED3D_SM4_SHADER_DATA_MESSAGE: u32 = 0x4;


#[repr(C)]
pub struct wined3d_shader_src_param_entry {
    pub entry: list,
    pub param: wined3d_shader_src_param,
}
pub type Pwined3d_shader_src_param_entry = *mut wined3d_shader_src_param_entry;

#[repr(C)]
pub struct wined3d_sm4_data {
    pub shader_version: wined3d_shader_version,
    pub start: *const u32,
    pub end: *const u32,
    pub output_map: [u32; (MAX_REG_OUTPUT as _)],
    pub src_param: [wined3d_shader_src_param; 5],
    pub dst_param: [wined3d_shader_dst_param; 2],
    pub src_free: list,
    pub src: list,
    pub icb: wined3d_shader_immediate_constant_buffer,
}
pub type Pwined3d_sm4_data = *mut wined3d_sm4_data;

#[repr(C)]
pub struct wined3d_sm4_opcode_info {
    pub opcode: u32,
    pub handler_idx: WINED3D_SHADER_INSTRUCTION_HANDLER,
    pub dst_info: *const i8,
    pub src_info: *const i8,
    pub read_opcode_func: Option<
        unsafe extern "C" fn(
            ins: *mut wined3d_shader_instruction,
            opcode: u32,
            opcode_token: u32,
            tokens: *const u32,
            token_count: u32,
            priv_: *mut wined3d_sm4_data,
        ),
    >,
}
pub type Pwined3d_sm4_opcode_info = *mut wined3d_sm4_opcode_info;

// output_primitive_type_table, input_primitive_type_table, resource_type_table,
// data_type_table, opcode_table, register_type_table, sm4_shader_frontend:
// table definitions skipped per instructions.
// shader_sm4_read_src_param / shader_sm4_read_dst_param: prototypes, skipped.

pub const TAG_AON9: u32 = 0x396e6f41;
pub const TAG_ISG1: u32 = 0x31475349;
pub const TAG_ISGN: u32 = 0x4e475349;
pub const TAG_OSG1: u32 = 0x3147534f;
pub const TAG_OSG5: u32 = 0x3547534f;
pub const TAG_OSGN: u32 = 0x4e47534f;
pub const TAG_PCSG: u32 = 0x47534350;
pub const TAG_PSG1: u32 = 0x31475350;
pub const TAG_SHDR: u32 = 0x52444853;
pub const TAG_SHEX: u32 = 0x58454853;

#[repr(C)]
pub struct aon9_header {
    pub chunk_size: u32,
    pub shader_version: u32,
    pub unknown: u32,
    pub byte_code_offset: u32,
}
pub type Paon9_header = *mut aon9_header;

unsafe extern "C" fn shader_sm4_read_conditional_op(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_src_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_UINT,
        core::ptr::addr_of_mut!((*priv_).src_param[0]),
    );
    (*ins).flags = (if opcode_token & WINED3D_SM4_CONDITIONAL_NZ != 0 {
        WINED3D_SHADER_CONDITIONAL_OP_NZ
    } else {
        WINED3D_SHADER_CONDITIONAL_OP_Z
    }) as uint32_t;
}

unsafe extern "C" fn shader_sm4_read_shader_data(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    let mut type_: wined3d_sm4_shader_data_type = 0 as wined3d_sm4_shader_data_type;
    let mut icb_size: ::core::ffi::c_uint = 0;
    let mut tokens = tokens;
    type_ = ((opcode_token & WINED3D_SM4_SHADER_DATA_TYPE_MASK) >> WINED3D_SM4_SHADER_DATA_TYPE_SHIFT) as wined3d_sm4_shader_data_type;
    if type_ as ::core::ffi::c_uint != WINED3D_SM4_SHADER_DATA_IMMEDIATE_CONSTANT_BUFFER {
        FIXME!("Ignoring shader data type %#x.\n", type_ as ::core::ffi::c_uint);
        (*ins).handler_idx = WINED3DSIH_NOP;
        return;
    }

    tokens = tokens.offset(1);
    icb_size = token_count.wrapping_sub(1);
    if icb_size.wrapping_rem(4) != 0 || icb_size > MAX_IMMEDIATE_CONSTANT_BUFFER_SIZE {
        FIXME!("Unexpected immediate constant buffer size %u.\n", icb_size);
        (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
        return;
    }

    (*priv_).icb.vec4_count = icb_size.wrapping_div(4);
    memcpy(
        core::ptr::addr_of_mut!((*priv_).icb.data) as *mut ::core::ffi::c_void,
        tokens as *const ::core::ffi::c_void,
        (::core::mem::size_of::<DWORD>() as usize).wrapping_mul(icb_size as usize),
    );
    (*ins).declaration.icb = core::ptr::addr_of_mut!((*priv_).icb);
}

unsafe extern "C" fn shader_sm4_read_dcl_resource(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    let mut resource_type: wined3d_sm4_resource_type;
    let mut data_type: wined3d_sm4_data_type;
    let mut reg_data_type: wined3d_data_type;
    let mut components: uint32_t;

    resource_type = ((opcode_token & WINED3D_SM4_RESOURCE_TYPE_MASK) >> WINED3D_SM4_RESOURCE_TYPE_SHIFT) as wined3d_sm4_resource_type;
    if resource_type as u64 == 0 || resource_type as usize >= ARRAY_SIZE!(resource_type_table) as usize {
        FIXME!("Unhandled resource type %#x.\n", resource_type as ::core::ffi::c_uint);
        (*ins).declaration.semantic.resource_type = WINED3D_SHADER_RESOURCE_NONE;
    } else {
        (*ins).declaration.semantic.resource_type = resource_type_table[resource_type as usize];
    }

    if (*ins).declaration.semantic.resource_type == WINED3D_SHADER_RESOURCE_TEXTURE_2DMS
        || (*ins).declaration.semantic.resource_type == WINED3D_SHADER_RESOURCE_TEXTURE_2DMSARRAY
    {
        (*ins).declaration.semantic.sample_count =
            (opcode_token & WINED3D_SM4_RESOURCE_SAMPLE_COUNT_MASK) >> WINED3D_SM4_RESOURCE_SAMPLE_COUNT_SHIFT;
    }

    reg_data_type = (if opcode == WINED3D_SM4_OP_DCL_RESOURCE {
        WINED3D_DATA_RESOURCE
    } else {
        WINED3D_DATA_UAV
    }) as wined3d_data_type;
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        reg_data_type,
        core::ptr::addr_of_mut!((*ins).declaration.semantic.reg),
    );

    let fresh0 = tokens;
    tokens = tokens.add(1);
    components = *fresh0;
    if (components & 0xfff0) != (components & 0xf) * 0x1110 {
        FIXME!("Components (%#x) have different data types.\n", components);
    }
    data_type = (components & 0xf) as wined3d_sm4_data_type;

    if data_type as u64 == 0 || data_type as usize >= ARRAY_SIZE!(data_type_table) as usize {
        FIXME!("Unhandled data type %#x.\n", data_type as ::core::ffi::c_uint);
        (*ins).declaration.semantic.resource_data_type = WINED3D_DATA_FLOAT;
    } else {
        (*ins).declaration.semantic.resource_data_type = data_type_table[data_type as usize];
    }

    if reg_data_type == WINED3D_DATA_UAV {
        (*ins).flags = (opcode_token & WINED3D_SM5_UAV_FLAGS_MASK) >> WINED3D_SM5_UAV_FLAGS_SHIFT;
    }
}

unsafe extern "C" fn shader_sm4_read_dcl_constant_buffer(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    shader_sm4_read_src_param(
        priv_0,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize) as *const DWORD,
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.src),
    );
    if (opcode_token as ::core::ffi::c_uint & WINED3D_SM4_INDEX_TYPE_MASK) != 0 {
        (*ins).flags |= WINED3DSI_INDEXED_DYNAMIC as uint32_t;
    }
}

unsafe extern "C" fn shader_sm4_read_dcl_sampler(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    (*ins).flags = (((opcode_token as u32) & (WINED3D_SM4_SAMPLER_MODE_MASK as u32))
        >> (WINED3D_SM4_SAMPLER_MODE_SHIFT as u32)) as u32;
    if (*ins).flags & !(WINED3D_SM4_SAMPLER_COMPARISON as u32) != 0 {
        FIXME!("Unhandled sampler mode %#x.\n", (*ins).flags);
    }
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_SAMPLER,
        core::ptr::addr_of_mut!((*ins).declaration.dst),
    );
}

unsafe extern "C" fn shader_sm4_read_dcl_index_range(mut ins: *mut wined3d_shader_instruction, mut opcode: DWORD, mut opcode_token: DWORD, mut tokens: *const DWORD, mut token_count: ::core::ffi::c_uint, mut priv_0: *mut wined3d_sm4_data) {
    shader_sm4_read_dst_param(priv_0, core::ptr::addr_of_mut!(tokens), tokens.add(token_count as usize) as *const DWORD, WINED3D_DATA_OPAQUE, core::ptr::addr_of_mut!((*ins).declaration.index_range.first_register));
    (*ins).declaration.index_range.last_register = *tokens;
}

unsafe extern "C" fn shader_sm4_read_dcl_output_topology(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    let mut primitive_type: u32 = 0;
    primitive_type = (opcode_token & WINED3D_SM4_PRIMITIVE_TYPE_MASK) >> WINED3D_SM4_PRIMITIVE_TYPE_SHIFT;
    if primitive_type as usize >= ARRAY_SIZE!(output_primitive_type_table) as usize {
        (*ins).declaration.primitive_type.type_ = WINED3D_PT_UNDEFINED;
    } else {
        (*ins).declaration.primitive_type.type_ =
            output_primitive_type_table[primitive_type as usize];
    }

    if (*ins).declaration.primitive_type.type_ == WINED3D_PT_UNDEFINED {
        FIXME!("Unhandled output primitive type %#x.\n", primitive_type);
    }
}

unsafe extern "C" fn shader_sm4_read_dcl_input_primitive(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    let mut primitive_type: u32;

    primitive_type = (opcode_token & WINED3D_SM4_PRIMITIVE_TYPE_MASK) >> WINED3D_SM4_PRIMITIVE_TYPE_SHIFT;
    if WINED3D_SM5_INPUT_PT_PATCH1 <= primitive_type && primitive_type <= WINED3D_SM5_INPUT_PT_PATCH32 {
        (*ins).declaration.primitive_type.type_ = WINED3D_PT_PATCH;
        (*ins).declaration.primitive_type.patch_vertex_count = primitive_type
            .wrapping_sub(WINED3D_SM5_INPUT_PT_PATCH1)
            .wrapping_add(1);
    } else if primitive_type as usize >= ARRAY_SIZE!(input_primitive_type_table) as usize {
        (*ins).declaration.primitive_type.type_ = WINED3D_PT_UNDEFINED;
    } else {
        (*ins).declaration.primitive_type.type_ =
            input_primitive_type_table[primitive_type as usize];
    }

    if (*ins).declaration.primitive_type.type_ == WINED3D_PT_UNDEFINED {
        FIXME!("Unhandled input primitive type %#x.\n", primitive_type);
    }
}

unsafe extern "C" fn shader_sm4_read_declaration_count(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    (*ins).declaration.count = *tokens;
}

unsafe extern "C" fn shader_sm4_read_declaration_dst(
    mut ins: *mut wined3d_shader_instruction,
    mut opcode: DWORD,
    mut opcode_token: DWORD,
    mut tokens: *const DWORD,
    mut token_count: ::core::ffi::c_uint,
    mut priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.dst),
    );
}

unsafe extern "C" fn shader_sm4_read_declaration_register_semantic(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_0,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.register_semantic.reg),
    );
    (*ins).declaration.register_semantic.sysval_semantic =
        *tokens as wined3d_shader_input_sysval_semantic;
}

unsafe extern "C" fn shader_sm4_read_dcl_input_ps(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    let mut tokens = tokens;
    (*ins).flags = ((opcode_token as ::core::ffi::c_uint & WINED3D_SM4_INTERPOLATION_MODE_MASK)
        >> WINED3D_SM4_INTERPOLATION_MODE_SHIFT) as uint32_t;
    shader_sm4_read_dst_param(
        priv_0,
        core::ptr::addr_of_mut!(tokens),
        tokens.offset(token_count as isize) as *const DWORD,
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.dst),
    );
}

unsafe extern "C" fn shader_sm4_read_dcl_input_ps_siv(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    let mut tokens = tokens;
    (*ins).flags = ((opcode_token & WINED3D_SM4_INTERPOLATION_MODE_MASK) >> WINED3D_SM4_INTERPOLATION_MODE_SHIFT) as _;
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.register_semantic.reg),
    );
    (*ins).declaration.register_semantic.sysval_semantic = *tokens as wined3d_shader_input_sysval_semantic;
}

unsafe extern "C" fn shader_sm4_read_dcl_indexable_temp(
    mut ins: *mut wined3d_shader_instruction,
    mut opcode: DWORD,
    mut opcode_token: DWORD,
    mut tokens: *const DWORD,
    mut token_count: ::core::ffi::c_uint,
    mut priv_0: *mut wined3d_sm4_data,
) {
    let fresh31 = tokens;
    tokens = tokens.offset(1);
    (*ins).declaration.indexable_temp.register_idx = *fresh31 as ::core::ffi::c_uint;
    let fresh32 = tokens;
    tokens = tokens.offset(1);
    (*ins).declaration.indexable_temp.register_size = *fresh32 as ::core::ffi::c_uint;
    (*ins).declaration.indexable_temp.component_count = *tokens as ::core::ffi::c_uint;
}

unsafe extern "C" fn shader_sm4_read_dcl_global_flags(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    (*ins).flags = ((opcode_token & WINED3D_SM4_GLOBAL_FLAGS_MASK) >> WINED3D_SM4_GLOBAL_FLAGS_SHIFT) as DWORD;
}

unsafe extern "C" fn shader_sm5_read_fcall(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    let fresh30 = tokens;
    tokens = tokens.add(1);
    (*priv_).src_param[0 as usize].reg.u.fp_body_idx = *fresh30;
    shader_sm4_read_src_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_OPAQUE,
        core::ptr::addr_of_mut!((*priv_).src_param[0 as usize]),
    );
}

unsafe extern "C" fn shader_sm5_read_dcl_function_body(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    (*ins).declaration.index = *tokens;
}

unsafe extern "C" fn shader_sm5_read_dcl_function_table(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    let fresh29 = tokens;
    tokens = tokens.add(1);
    (*ins).declaration.index = *fresh29;
    FIXME!("Ignoring set of function bodies (count %lu).\n", *tokens);
}

unsafe extern "C" fn shader_sm5_read_dcl_interface(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    let fresh26 = tokens;
    tokens = tokens.add(1);
    (*ins).declaration.fp.index = *fresh26 as ::core::ffi::c_uint;
    let fresh27 = tokens;
    tokens = tokens.add(1);
    (*ins).declaration.fp.body_count = *fresh27 as ::core::ffi::c_uint;
    (*ins).declaration.fp.array_size = (*tokens >> WINED3D_SM5_FP_ARRAY_SIZE_SHIFT) as ::core::ffi::c_uint;
    let fresh28 = tokens;
    tokens = tokens.add(1);
    (*ins).declaration.fp.table_count = *fresh28 & WINED3D_SM5_FP_TABLE_COUNT_MASK;
    FIXME!(
        "Ignoring set of function tables (count %u).\n",
        (*ins).declaration.fp.table_count
    );
}

unsafe extern "C" fn shader_sm5_read_control_point_count(
    mut ins: *mut wined3d_shader_instruction,
    mut opcode: DWORD,
    mut opcode_token: DWORD,
    mut tokens: *const DWORD,
    mut token_count: ::core::ffi::c_uint,
    mut priv_0: *mut wined3d_sm4_data,
) {
    (*ins).declaration.count = ((opcode_token & WINED3D_SM5_CONTROL_POINT_COUNT_MASK)
        >> WINED3D_SM5_CONTROL_POINT_COUNT_SHIFT) as _;
}

unsafe extern "C" fn shader_sm5_read_dcl_tessellator_domain(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    (*ins).declaration.tessellator_domain = ((opcode_token & WINED3D_SM5_TESSELLATOR_MASK)
        >> WINED3D_SM5_TESSELLATOR_SHIFT) as wined3d_tessellator_domain;
}

unsafe extern "C" fn shader_sm5_read_dcl_tessellator_partitioning(
    mut ins: *mut wined3d_shader_instruction,
    mut opcode: DWORD,
    mut opcode_token: DWORD,
    mut tokens: *const DWORD,
    mut token_count: ::core::ffi::c_uint,
    mut priv_0: *mut wined3d_sm4_data,
) {
    (*ins).declaration.tessellator_partitioning =
        ((opcode_token & WINED3D_SM5_TESSELLATOR_MASK)
            >> WINED3D_SM5_TESSELLATOR_SHIFT) as wined3d_tessellator_partitioning;
}

unsafe extern "C" fn shader_sm5_read_dcl_tessellator_output_primitive(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: core::ffi::c_uint,
    priv_0: *mut wined3d_sm4_data,
) {
    (*ins).declaration.tessellator_output_primitive = ((opcode_token & WINED3D_SM5_TESSELLATOR_MASK)
        >> WINED3D_SM5_TESSELLATOR_SHIFT) as wined3d_tessellator_output_primitive;
}

unsafe extern "C" fn shader_sm5_read_dcl_hs_max_tessfactor(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    (*ins).declaration.max_tessellation_factor = *(tokens as *const f32);
}

unsafe extern "C" fn shader_sm5_read_dcl_thread_group(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    let mut tokens = tokens;
    (*ins).declaration.thread_group_size.x = *tokens;
    tokens = tokens.offset(1);
    (*ins).declaration.thread_group_size.y = *tokens;
    tokens = tokens.offset(1);
    (*ins).declaration.thread_group_size.z = *tokens;
    tokens = tokens.offset(1);
}

unsafe extern "C" fn shader_sm5_read_dcl_uav_raw(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize) as *const DWORD,
        WINED3D_DATA_UAV,
        core::ptr::addr_of_mut!((*ins).declaration.dst),
    );
    (*ins).flags = (opcode_token & WINED3D_SM5_UAV_FLAGS_MASK) >> WINED3D_SM5_UAV_FLAGS_SHIFT;
}

unsafe extern "C" fn shader_sm5_read_dcl_uav_structured(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: u32,
    priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_UAV,
        core::ptr::addr_of_mut!((*ins).declaration.structured_resource.reg),
    );
    (*ins).flags = (opcode_token & WINED3D_SM5_UAV_FLAGS_MASK) >> WINED3D_SM5_UAV_FLAGS_SHIFT;
    (*ins).declaration.structured_resource.byte_stride = *tokens;
    if (*ins).declaration.structured_resource.byte_stride % 4 != 0 {
        FIXME!(
            "Byte stride %u is not multiple of 4.\n",
            (*ins).declaration.structured_resource.byte_stride
        );
    }
}

unsafe extern "C" fn shader_sm5_read_dcl_tgsm_raw(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.tgsm_raw.reg),
    );
    (*ins).declaration.tgsm_raw.byte_count = *tokens;
    if (*ins).declaration.tgsm_raw.byte_count % 4 != 0 {
        FIXME!(
            "Byte count %u is not multiple of 4.\n",
            (*ins).declaration.tgsm_raw.byte_count
        );
    }
}

unsafe extern "C" fn shader_sm5_read_dcl_tgsm_structured(
    ins: *mut wined3d_shader_instruction,
    opcode: DWORD,
    opcode_token: DWORD,
    mut tokens: *const DWORD,
    token_count: ::core::ffi::c_uint,
    priv_: *mut wined3d_sm4_data,
) {
    shader_sm4_read_dst_param(
        priv_,
        core::ptr::addr_of_mut!(tokens),
        tokens.add(token_count as usize),
        WINED3D_DATA_FLOAT,
        core::ptr::addr_of_mut!((*ins).declaration.tgsm_structured.reg),
    );
    let fresh21 = tokens;
    tokens = tokens.add(1);
    (*ins).declaration.tgsm_structured.byte_stride = *fresh21;
    (*ins).declaration.tgsm_structured.structure_count = *tokens;
    if (*ins).declaration.tgsm_structured.byte_stride % 4 != 0 {
        FIXME!(
            "Byte stride %u is not multiple of 4.\n",
            (*ins).declaration.tgsm_structured.byte_stride
        );
    }
}

unsafe extern "C" fn shader_sm5_read_dcl_resource_structured(ins: *mut wined3d_shader_instruction, opcode: DWORD, opcode_token: DWORD, mut tokens: *const DWORD, token_count: u32, priv_: *mut wined3d_sm4_data) {
    shader_sm4_read_dst_param(priv_, core::ptr::addr_of_mut!(tokens), tokens.add(token_count as usize), WINED3D_DATA_RESOURCE, core::ptr::addr_of_mut!((*ins).declaration.structured_resource.reg));
    (*ins).declaration.structured_resource.byte_stride = *tokens;
    if (*ins).declaration.structured_resource.byte_stride % 4 != 0 {
        FIXME!("Byte stride %u is not multiple of 4.\n", (*ins).declaration.structured_resource.byte_stride);
    }
}

unsafe extern "C" fn shader_sm5_read_dcl_resource_raw(mut ins: *mut wined3d_shader_instruction, opcode: DWORD, opcode_token: DWORD, mut tokens: *const DWORD, token_count: ::core::ffi::c_uint, priv_0: *mut wined3d_sm4_data) {
    shader_sm4_read_dst_param(priv_0, core::ptr::addr_of_mut!(tokens), tokens.add(token_count as usize) as *const DWORD, WINED3D_DATA_RESOURCE, core::ptr::addr_of_mut!((*ins).declaration.dst));
}

unsafe extern "C" fn shader_sm5_read_sync(ins: *mut wined3d_shader_instruction, opcode: DWORD, opcode_token: DWORD, tokens: *const DWORD, token_count: ::core::ffi::c_uint, priv_: *mut wined3d_sm4_data) {
    (*ins).flags = (opcode_token & WINED3D_SM5_SYNC_FLAGS_MASK) >> WINED3D_SM5_SYNC_FLAGS_SHIFT;
}

unsafe extern "C" fn get_opcode_info(opcode: wined3d_sm4_opcode) -> *const wined3d_sm4_opcode_info {
    let mut i: ::core::ffi::c_uint = 0;
    while (i as usize) < ARRAY_SIZE!(opcode_table) as usize {
        if opcode as ::core::ffi::c_uint == opcode_table[i as usize].opcode as ::core::ffi::c_uint {
            return (core::ptr::addr_of!(opcode_table) as *const wined3d_sm4_opcode_info)
                .offset(i as isize);
        }
        i = i.wrapping_add(1);
    }

    return core::ptr::null();
}

unsafe extern "C" fn map_register(
    priv_0: *const wined3d_sm4_data,
    reg: *mut wined3d_shader_register,
) {
    match (*priv_0).shader_version.type_ {
        WINED3D_SHADER_TYPE_PIXEL => {
            if (*reg).type_ == WINED3DSPR_OUTPUT {
                let mut reg_idx: u32 = (*reg).idx[0 as usize].offset;

                if reg_idx as usize >= ARRAY_SIZE!((*priv_0).output_map) as usize {
                    ERR!("Invalid output index %u.\n", reg_idx);
                } else {
                    (*reg).type_ = WINED3DSPR_COLOROUT;
                    (*reg).idx[0 as usize].offset =
                        (*priv_0).output_map[reg_idx as usize];
                }
            }
        }
        _ => {}
    }
}

unsafe extern "C" fn map_data_type(t: ::core::ffi::c_char) -> wined3d_data_type {
    match t as ::core::ffi::c_int {
        102 => {
            return WINED3D_DATA_FLOAT;
        }
        105 => {
            return WINED3D_DATA_INT;
        }
        117 => {
            return WINED3D_DATA_UINT;
        }
        79 => {
            return WINED3D_DATA_OPAQUE;
        }
        82 => {
            return WINED3D_DATA_RESOURCE;
        }
        83 => {
            return WINED3D_DATA_SAMPLER;
        }
        85 => {
            return WINED3D_DATA_UAV;
        }
        _ => {
            ERR!("Invalid data type '%c'.\n", t as core::ffi::c_int);
            return WINED3D_DATA_FLOAT;
        }
    }
}

unsafe extern "C" fn wined3d_get_sm4_shader_type(byte_code: *const DWORD, byte_code_size: size_t) -> wined3d_shader_type {
    let mut shader_type: u32;

    if (byte_code_size / core::mem::size_of::<DWORD>()) < 1 {
        WARN!("Invalid byte code size %lu.\n", byte_code_size as core::ffi::c_long);
        return WINED3D_SHADER_TYPE_INVALID;
    }

    shader_type = *byte_code.add(0 as usize) >> 16;
    if shader_type == WINED3D_SM4_PS {
        return WINED3D_SHADER_TYPE_PIXEL;
    }
    if shader_type == WINED3D_SM4_VS {
        return WINED3D_SHADER_TYPE_VERTEX;
    }
    if shader_type == WINED3D_SM4_GS {
        return WINED3D_SHADER_TYPE_GEOMETRY;
    }
    if shader_type == WINED3D_SM5_HS {
        return WINED3D_SHADER_TYPE_HULL;
    }
    if shader_type == WINED3D_SM5_DS {
        return WINED3D_SHADER_TYPE_DOMAIN;
    }
    if shader_type == WINED3D_SM5_CS {
        return WINED3D_SHADER_TYPE_COMPUTE;
    }
    FIXME!("Unrecognised shader type %#x.\n", shader_type);
    return WINED3D_SHADER_TYPE_INVALID;
}

unsafe extern "C" fn shader_sm4_init(
    byte_code: *const DWORD,
    byte_code_size: size_t,
    output_signature: *const wined3d_shader_signature,
) -> *mut core::ffi::c_void {
    let mut version_token: u32;
    let mut token_count: u32;
    let mut priv_: *mut wined3d_sm4_data;
    let mut i: u32;

    if byte_code_size / core::mem::size_of::<DWORD>() < 2 {
        WARN!("Invalid byte code size %lu.\n", byte_code_size as core::ffi::c_long);
        return core::ptr::null_mut();
    }

    version_token = *byte_code.add(0);

    token_count = *byte_code.add(1);

    if token_count < 2 || byte_code_size / core::mem::size_of::<DWORD>() < token_count as usize {
        WARN!("Invalid token count %u.\n", token_count);
        return core::ptr::null_mut();
    }

    priv_ = malloc(core::mem::size_of::<wined3d_sm4_data>() as size_t) as *mut wined3d_sm4_data;
    if priv_.is_null() {
        ERR!("Failed to allocate private data\n");
        return core::ptr::null_mut();
    }

    (*priv_).start = byte_code.add(2);
    (*priv_).end = byte_code.add(token_count as usize);

    (*priv_).shader_version.type_ = wined3d_get_sm4_shader_type(byte_code, byte_code_size);
    if (*priv_).shader_version.type_ as u32 == WINED3D_SHADER_TYPE_INVALID {
        free(priv_ as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    (*priv_).shader_version.major = WINED3D_SM4_VERSION_MAJOR!(version_token) as BYTE;
    (*priv_).shader_version.minor = WINED3D_SM4_VERSION_MINOR!(version_token) as BYTE;

    memset(
        core::ptr::addr_of_mut!((*priv_).output_map) as *mut core::ffi::c_void,
        0xff,
        core::mem::size_of::<[core::ffi::c_uint; 32]>() as size_t,
    );
    i = 0;
    while i < (*output_signature).element_count {
        let mut e: *mut wined3d_shader_signature_element =
            (*output_signature).elements.add(i as usize) as *mut wined3d_shader_signature_element;

        if !((*priv_).shader_version.type_ as u32 == WINED3D_SHADER_TYPE_PIXEL
            && stricmp(
                (*e).semantic_name,
                b"SV_TARGET\0".as_ptr() as *const core::ffi::c_char,
            ) != 0)
        {
            if (*e).register_idx as usize
                >= ARRAY_SIZE!((*priv_).output_map) as usize
            {
                WARN!("Invalid output index %u.\n", (*e).register_idx);
            } else {
                (*priv_).output_map[(*e).register_idx as usize] = (*e).semantic_idx;
            }
        }
        i = i.wrapping_add(1);
    }

    list_init(core::ptr::addr_of_mut!((*priv_).src_free));
    list_init(core::ptr::addr_of_mut!((*priv_).src));

    return priv_ as *mut core::ffi::c_void;
}

unsafe extern "C" fn shader_sm4_free(data: *mut core::ffi::c_void) {
    let mut e1: *mut wined3d_shader_src_param_entry = core::ptr::null_mut();
    let mut e2: *mut wined3d_shader_src_param_entry = core::ptr::null_mut();
    let priv_: *mut wined3d_sm4_data = data as *mut wined3d_sm4_data;
    list_move_head(
        core::ptr::addr_of_mut!((*priv_).src_free),
        core::ptr::addr_of_mut!((*priv_).src),
    );
    e1 = ((*priv_).src_free.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_shader_src_param_entry, entry))
        as *mut wined3d_shader_src_param_entry;
    e2 = ((*e1).entry.next as *mut u8)
        .sub(core::mem::offset_of!(wined3d_shader_src_param_entry, entry))
        as *mut wined3d_shader_src_param_entry;
    while core::ptr::addr_of_mut!((*e1).entry) != core::ptr::addr_of_mut!((*priv_).src_free) {
        free(e1 as *mut core::ffi::c_void);
        e1 = e2;
        e2 = ((*e1).entry.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_shader_src_param_entry, entry))
            as *mut wined3d_shader_src_param_entry;
    }
    free(priv_ as *mut core::ffi::c_void);
}

unsafe extern "C" fn get_src_param(priv_: *mut wined3d_sm4_data) -> *mut wined3d_shader_src_param {
    let mut e: *mut wined3d_shader_src_param_entry = core::ptr::null_mut();
    let mut elem: *mut list = core::ptr::null_mut();
    if list_empty(core::ptr::addr_of_mut!((*priv_).src_free)) == 0 {
        elem = list_head(core::ptr::addr_of_mut!((*priv_).src_free));
        list_remove(elem);
    } else {
        e = malloc(core::mem::size_of::<wined3d_shader_src_param_entry>())
            as *mut wined3d_shader_src_param_entry;
        if e.is_null() {
            return core::ptr::null_mut::<wined3d_shader_src_param>();
        }
        elem = core::ptr::addr_of_mut!((*e).entry);
    }
    list_add_tail(core::ptr::addr_of_mut!((*priv_).src), elem);
    e = (elem as *mut u8)
        .sub(core::mem::offset_of!(wined3d_shader_src_param_entry, entry))
        as *mut wined3d_shader_src_param_entry;
    core::ptr::addr_of_mut!((*e).param)
}

unsafe extern "C" fn shader_sm4_read_header(data: *mut core::ffi::c_void, ptr: *mut *const DWORD, shader_version: *mut wined3d_shader_version) {
    let priv_: *mut wined3d_sm4_data = data as *mut wined3d_sm4_data;
    *ptr = (*priv_).start;
    *shader_version = (*priv_).shader_version;
}

unsafe extern "C" fn shader_sm4_read_reg_idx(
    priv_0: *mut wined3d_sm4_data,
    ptr: *mut *const DWORD,
    end: *const DWORD,
    addressing: DWORD,
    reg_idx: *mut wined3d_shader_register_index,
) -> BOOL {
    if addressing & WINED3D_SM4_ADDRESSING_RELATIVE != 0 {
        let rel_addr: *mut wined3d_shader_src_param = get_src_param(priv_0);

        (*reg_idx).rel_addr = rel_addr;
        if (*reg_idx).rel_addr.is_null() {
            ERR!("Failed to get src param for relative addressing.\n");
            return (FALSE) as BOOL;
        }

        if addressing & WINED3D_SM4_ADDRESSING_OFFSET != 0 {
            let fresh19 = *ptr;
            *ptr = (*ptr).add(1);
            (*reg_idx).offset = *fresh19;
        } else {
            (*reg_idx).offset = 0;
        }
        shader_sm4_read_src_param(priv_0, ptr, end, WINED3D_DATA_INT, rel_addr);
    } else {
        (*reg_idx).rel_addr = core::ptr::null_mut();
        let fresh20 = *ptr;
        *ptr = (*ptr).add(1);
        (*reg_idx).offset = *fresh20;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn shader_sm4_read_param(
    priv_0: *mut wined3d_sm4_data,
    ptr: *mut *const DWORD,
    end: *const DWORD,
    data_type: wined3d_data_type,
    param: *mut wined3d_shader_register,
    modifier: *mut wined3d_shader_src_modifier,
) -> BOOL {
    let mut register_type: wined3d_sm4_register_type = WINED3D_SM4_RT_TEMP;
    let mut token: uint32_t = 0;
    let mut order: uint32_t = 0;

    if *ptr >= end {
        WARN!("Invalid ptr %p >= end %p.\n", *ptr, end);
        return (FALSE) as BOOL;
    }
    token = *(*ptr);
    *ptr = (*ptr).offset(1);

    register_type = ((token & WINED3D_SM4_REGISTER_TYPE_MASK) >> WINED3D_SM4_REGISTER_TYPE_SHIFT) as wined3d_sm4_register_type;
    if (register_type as usize) >= ARRAY_SIZE!(register_type_table) as usize
        || register_type_table[register_type as usize] as ::core::ffi::c_uint
            == !(0 as ::core::ffi::c_uint)
    {
        (*param).type_ = WINED3DSPR_TEMP;
    } else {
        (*param).type_ = register_type_table[register_type as usize];
    }
    (*param).data_type = data_type;

    if (token & WINED3D_SM4_REGISTER_MODIFIER) != 0 {
        let mut m: ::core::ffi::c_uint = 0;

        if *ptr >= end {
            WARN!("Invalid ptr %p >= end %p.\n", *ptr, end);
            return (FALSE) as BOOL;
        }
        m = *(*ptr);
        *ptr = (*ptr).offset(1);

        match m {
            0x41 => {
                *modifier = WINED3DSPSM_NEG;
            }
            0x81 => {
                *modifier = WINED3DSPSM_ABS;
            }
            0xc1 => {
                *modifier = WINED3DSPSM_ABSNEG;
            }
            0x01 => {
                *modifier = WINED3DSPSM_NONE;
            }
            _ => {
                FIXME!("Skipping modifier 0x%08x.\n", m);
                *modifier = WINED3DSPSM_NONE;
            }
        }
    } else {
        *modifier = WINED3DSPSM_NONE;
    }

    order = (token & WINED3D_SM4_REGISTER_ORDER_MASK) >> WINED3D_SM4_REGISTER_ORDER_SHIFT;

    if order < 1 {
        (*param).idx[0 as usize].offset = !(0 as ::core::ffi::c_uint);
    } else {
        let addressing: DWORD = (token & WINED3D_SM4_ADDRESSING_MASK0) >> WINED3D_SM4_ADDRESSING_SHIFT0;
        if shader_sm4_read_reg_idx(
            priv_0,
            ptr,
            end,
            addressing,
            core::ptr::addr_of_mut!((*param).idx[0 as usize]),
        ) == 0
        {
            ERR!("Failed to read register index.\n");
            return (FALSE) as BOOL;
        }
    }

    if order < 2 {
        (*param).idx[1 as usize].offset = !(0 as ::core::ffi::c_uint);
    } else {
        let addressing: DWORD = (token & WINED3D_SM4_ADDRESSING_MASK1) >> WINED3D_SM4_ADDRESSING_SHIFT1;
        if shader_sm4_read_reg_idx(
            priv_0,
            ptr,
            end,
            addressing,
            core::ptr::addr_of_mut!((*param).idx[1 as usize]),
        ) == 0
        {
            ERR!("Failed to read register index.\n");
            return (FALSE) as BOOL;
        }
    }

    if order > 2 {
        FIXME!("Unhandled order %u.\n", order);
    }

    if register_type as ::core::ffi::c_uint == (WINED3D_SM4_RT_IMMCONST as ::core::ffi::c_uint) {
        let mut dimension: wined3d_sm4_dimension =
            ((token & WINED3D_SM4_DIMENSION_MASK) >> WINED3D_SM4_DIMENSION_SHIFT) as wined3d_sm4_dimension;

        match dimension as ::core::ffi::c_uint {
            WINED3D_SM4_DIMENSION_SCALAR => {
                (*param).immconst_type = WINED3D_IMMCONST_SCALAR;
                if ((end.offset_from(*ptr)) as ::core::ffi::c_longlong)
                    < (1 as ::core::ffi::c_longlong)
                {
                    WARN!("Invalid ptr %p, end %p.\n", *ptr, end);
                    return (FALSE) as BOOL;
                }
                memcpy(
                    core::ptr::addr_of_mut!((*param).u.immconst_data) as *mut ::core::ffi::c_void,
                    *ptr as *const ::core::ffi::c_void,
                    (1 as usize).wrapping_mul(core::mem::size_of::<DWORD>()),
                );
                *ptr = (*ptr).offset(1 as isize);
            }

            WINED3D_SM4_DIMENSION_VEC4 => {
                (*param).immconst_type = WINED3D_IMMCONST_VEC4;
                if ((end.offset_from(*ptr)) as ::core::ffi::c_longlong)
                    < (4 as ::core::ffi::c_longlong)
                {
                    WARN!("Invalid ptr %p, end %p.\n", *ptr, end);
                    return (FALSE) as BOOL;
                }
                memcpy(
                    core::ptr::addr_of_mut!((*param).u.immconst_data) as *mut ::core::ffi::c_void,
                    *ptr as *const ::core::ffi::c_void,
                    (4 as usize).wrapping_mul(core::mem::size_of::<DWORD>()),
                );
                *ptr = (*ptr).offset(4 as isize);
            }

            _ => {
                FIXME!("Unhandled dimension %#x.\n", dimension as ::core::ffi::c_uint);
            }
        }
    }

    map_register(priv_0, param);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn shader_sm4_read_src_param(
    priv_: *mut wined3d_sm4_data,
    ptr: *mut *const DWORD,
    end: *const DWORD,
    data_type: wined3d_data_type,
    src_param: *mut wined3d_shader_src_param,
) -> BOOL {
    let mut token: DWORD = 0;
    if *ptr >= end {
        WARN!("Invalid ptr %p >= end %p.\n", *ptr, end);
        return (FALSE) as BOOL;
    }
    token = **ptr;

    if shader_sm4_read_param(
        priv_,
        ptr,
        end,
        data_type,
        core::ptr::addr_of_mut!((*src_param).reg),
        core::ptr::addr_of_mut!((*src_param).modifiers),
    ) == 0
    {
        ERR!("Failed to read parameter.\n");
        return (FALSE) as BOOL;
    }

    if (*src_param).reg.type_ as u32 == WINED3DSPR_IMMCONST as u32 {
        (*src_param).swizzle = WINED3DSP_NOSWIZZLE as DWORD;
    } else {
        let mut swizzle_type: wined3d_sm4_swizzle_type =
            ((token & WINED3D_SM4_SWIZZLE_TYPE_MASK) >> WINED3D_SM4_SWIZZLE_TYPE_SHIFT) as wined3d_sm4_swizzle_type;

        match swizzle_type as u32 {
            WINED3D_SM4_SWIZZLE_NONE => {
                (*src_param).swizzle = WINED3DSP_NOSWIZZLE as DWORD;
            }

            WINED3D_SM4_SWIZZLE_SCALAR => {
                (*src_param).swizzle = (token & WINED3D_SM4_SWIZZLE_MASK) >> WINED3D_SM4_SWIZZLE_SHIFT;
                (*src_param).swizzle = ((*src_param).swizzle & 0x3)
                    .wrapping_mul(0x55);
            }

            WINED3D_SM4_SWIZZLE_VEC4 => {
                (*src_param).swizzle = (token & WINED3D_SM4_SWIZZLE_MASK) >> WINED3D_SM4_SWIZZLE_SHIFT;
            }

            _ => {
                FIXME!("Unhandled swizzle type %#x.\n", swizzle_type as u32);
            }
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn shader_sm4_read_dst_param(priv_: *mut wined3d_sm4_data, ptr: *mut *const DWORD, end: *const DWORD, data_type: wined3d_data_type, dst_param: *mut wined3d_shader_dst_param) -> BOOL {
    let mut modifier: wined3d_shader_src_modifier = Default::default();
    let mut token: DWORD = 0;

    if *ptr >= end {
        WARN!("Invalid ptr %p >= end %p.\n", *ptr, end);
        return (FALSE) as BOOL;
    }
    token = **ptr;

    if shader_sm4_read_param(priv_, ptr, end, data_type, core::ptr::addr_of_mut!((*dst_param).reg), core::ptr::addr_of_mut!(modifier)) == 0 {
        ERR!("Failed to read parameter.\n");
        return (FALSE) as BOOL;
    }

    if modifier != WINED3DSPSM_NONE {
        ERR!("Invalid source modifier %#x on destination register.\n", modifier);
        return (FALSE) as BOOL;
    }

    (*dst_param).write_mask = ((token & WINED3D_SM4_WRITEMASK_MASK) >> WINED3D_SM4_WRITEMASK_SHIFT) as uint32_t;
    (*dst_param).modifiers = 0 as uint32_t;
    (*dst_param).shift = 0 as u32;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn shader_sm4_read_instruction_modifier(
    mut modifier: uint32_t,
    mut ins: *mut wined3d_shader_instruction,
) {
    let mut modifier_type: wined3d_sm4_instruction_modifier =
        (modifier & WINED3D_SM4_MODIFIER_MASK as uint32_t) as wined3d_sm4_instruction_modifier;
    match modifier_type as ::core::ffi::c_uint {
        WINED3D_SM4_MODIFIER_AOFFIMMI => {
            static recognized_bits: DWORD = WINED3D_SM4_INSTRUCTION_MODIFIER
                | WINED3D_SM4_MODIFIER_MASK
                | WINED3D_SM4_AOFFIMMI_U_MASK
                | WINED3D_SM4_AOFFIMMI_V_MASK
                | WINED3D_SM4_AOFFIMMI_W_MASK;

            /* Bit fields are used for sign extension. */
            /* struct { int u : W_U; int v : W_V; int w : W_W; } aoffimmi; */
            let mut aoffimmi_u: ::core::ffi::c_int = 0;
            let mut aoffimmi_v: ::core::ffi::c_int = 0;
            let mut aoffimmi_w: ::core::ffi::c_int = 0;

            if modifier & !(recognized_bits as uint32_t) != 0 {
                WINE_FIXME!("Unhandled instruction modifier %#x.\n", modifier);
            }
            aoffimmi_u = ((((modifier & WINED3D_SM4_AOFFIMMI_U_MASK as uint32_t)
                >> WINED3D_SM4_AOFFIMMI_U_SHIFT) as ::core::ffi::c_int)
                << (32 - 4))
                >> (32 - 4);
            aoffimmi_v = ((((modifier & WINED3D_SM4_AOFFIMMI_V_MASK as uint32_t)
                >> WINED3D_SM4_AOFFIMMI_V_SHIFT) as ::core::ffi::c_int)
                << (32 - 4))
                >> (32 - 4);
            aoffimmi_w = ((((modifier & WINED3D_SM4_AOFFIMMI_W_MASK as uint32_t)
                >> WINED3D_SM4_AOFFIMMI_W_SHIFT) as ::core::ffi::c_int)
                << (32 - 4))
                >> (32 - 4);
            (*ins).texel_offset.u = aoffimmi_u as ::core::ffi::c_schar;
            (*ins).texel_offset.v = aoffimmi_v as ::core::ffi::c_schar;
            (*ins).texel_offset.w = aoffimmi_w as ::core::ffi::c_schar;
        }
        WINED3D_SM5_MODIFIER_DATA_TYPE => {
            let mut components: uint32_t = (modifier
                & WINED3D_SM5_MODIFIER_DATA_TYPE_MASK as uint32_t)
                >> WINED3D_SM5_MODIFIER_DATA_TYPE_SHIFT;
            let mut data_type: wined3d_sm4_data_type =
                (components & 0xf as uint32_t) as wined3d_sm4_data_type;
            if components & 0xfff0 as uint32_t
                != (components & 0xf as uint32_t)
                    .wrapping_mul(0x1110 as uint32_t)
            {
                WINE_FIXME!("Components (%#x) have different data types.\n", components);
            }
            (*ins).resource_data_type = data_type_table[data_type as usize];
        }
        WINED3D_SM5_MODIFIER_RESOURCE_TYPE => {
            let mut resource_type: wined3d_sm4_resource_type = ((modifier
                & WINED3D_SM5_MODIFIER_RESOURCE_TYPE_MASK as uint32_t)
                >> WINED3D_SM5_MODIFIER_RESOURCE_TYPE_SHIFT)
                as wined3d_sm4_resource_type;
            (*ins).resource_type = resource_type_table[resource_type as usize];
        }
        _ => {
            WINE_FIXME!("Unhandled instruction modifier %#x.\n", modifier);
        }
    };
}

unsafe extern "C" fn shader_sm4_read_instruction(
    data: *mut ::core::ffi::c_void,
    ptr: *mut *const DWORD,
    ins: *mut wined3d_shader_instruction,
) {
    let mut opcode_info: *const wined3d_sm4_opcode_info = ::core::ptr::null();
    let mut opcode_token: uint32_t = 0;
    let mut previous_token: uint32_t = 0;
    let priv_: *mut wined3d_sm4_data = data as *mut wined3d_sm4_data;
    let mut opcode: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut len: ::core::ffi::c_uint = 0;
    let mut remaining: SIZE_T = 0;
    let mut p: *const DWORD = ::core::ptr::null();
    let mut precise: DWORD = 0;

    list_move_head(&raw mut (*priv_).src_free, &raw mut (*priv_).src);

    'fail: {
        if *ptr >= (*priv_).end {
            WARN!("End of byte-code, failed to read opcode.\n");
            break 'fail;
        }
        remaining = (*priv_).end.offset_from(*ptr) as SIZE_T;

        opcode_token = **ptr;
        *ptr = (*ptr).add(1);
        opcode = (opcode_token & WINED3D_SM4_OPCODE_MASK) as ::core::ffi::c_uint;

        len = ((opcode_token & WINED3D_SM4_INSTRUCTION_LENGTH_MASK) >> WINED3D_SM4_INSTRUCTION_LENGTH_SHIFT) as ::core::ffi::c_uint;
        if len == 0 {
            if remaining < 2 {
                WARN!("End of byte-code, failed to read length token.\n");
                break 'fail;
            }
            len = **ptr;
        }
        if len == 0 || remaining < len as SIZE_T {
            WARN!("Read invalid length %u (remaining %Iu).\n", len, remaining);
            break 'fail;
        }
        len -= 1;

        if ((TRACE_ON!(__wine_dbch_d3d_bytecode)) != 0) {
            WINE_TRACE_!(__wine_dbch_d3d_bytecode, "[ %08x ", opcode_token);
            i = 0;
            while i < len {
                WINE_TRACE_!(__wine_dbch_d3d_bytecode, "%08lx ", *(*ptr).add(i as usize));
                i = i.wrapping_add(1);
            }
            WINE_TRACE_!(__wine_dbch_d3d_bytecode, "]\n");
        }

        opcode_info = get_opcode_info(opcode as wined3d_sm4_opcode);
        if opcode_info.is_null() {
            FIXME!("Unrecognized opcode %#x, opcode_token 0x%08x.\n", opcode, opcode_token);
            (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
            *ptr = (*ptr).add(len as usize);
            return;
        }

        (*ins).handler_idx = (*opcode_info).handler_idx;
        (*ins).flags = 0;
        (*ins).coissue = 0 as BOOL;
        (*ins).predicate = ::core::ptr::null();
        (*ins).dst_count = strlen((*opcode_info).dst_info) as ::core::ffi::c_uint;
        (*ins).dst = &raw mut (*priv_).dst_param as *mut wined3d_shader_dst_param;
        (*ins).src_count = strlen((*opcode_info).src_info) as ::core::ffi::c_uint;
        (*ins).src = &raw mut (*priv_).src_param as *mut wined3d_shader_src_param;
        (*ins).resource_type = WINED3D_SHADER_RESOURCE_NONE;
        (*ins).resource_data_type = WINED3D_DATA_FLOAT;
        memset(
            &raw mut (*ins).texel_offset as *mut ::core::ffi::c_void,
            0,
            ::core::mem::size_of::<wined3d_shader_texel_offset>() as size_t,
        );

        p = *ptr;
        *ptr = (*ptr).add(len as usize);

        if let Some(read_opcode_func) = (*opcode_info).read_opcode_func {
            read_opcode_func(
                ins,
                opcode as DWORD,
                opcode_token as DWORD,
                p,
                len,
                priv_,
            );
        } else {
            let mut instruction_dst_modifier: wined3d_shader_dst_modifier =
                WINED3DSPDM_NONE;

            previous_token = opcode_token;
            while (previous_token & WINED3D_SM4_INSTRUCTION_MODIFIER) != 0 && p != *ptr {
                previous_token = *p;
                p = p.add(1);
                shader_sm4_read_instruction_modifier(previous_token, ins);
            }

            (*ins).flags = (opcode_token & WINED3D_SM4_INSTRUCTION_FLAGS_MASK) >> WINED3D_SM4_INSTRUCTION_FLAGS_SHIFT;
            if ((*ins).flags & WINED3D_SM4_INSTRUCTION_FLAG_SATURATE) != 0 {
                (*ins).flags &= !(WINED3D_SM4_INSTRUCTION_FLAG_SATURATE);
                instruction_dst_modifier = WINED3DSPDM_SATURATE;
            }
            precise = ((opcode_token & WINED3D_SM5_PRECISE_MASK) >> WINED3D_SM5_PRECISE_SHIFT) as DWORD;
            (*ins).flags |= precise << WINED3DSI_PRECISE_SHIFT;

            i = 0;
            while i < (*ins).dst_count {
                if shader_sm4_read_dst_param(
                    priv_,
                    &raw mut p,
                    *ptr,
                    map_data_type(*(*opcode_info).dst_info.add(i as usize)),
                    (&raw mut (*priv_).dst_param as *mut wined3d_shader_dst_param)
                        .add(i as usize) as *mut wined3d_shader_dst_param,
                ) == 0
                {
                    (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
                    return;
                }
                (*priv_).dst_param[i as usize].modifiers |= instruction_dst_modifier as uint32_t;
                i = i.wrapping_add(1);
            }

            i = 0;
            while i < (*ins).src_count {
                if shader_sm4_read_src_param(
                    priv_,
                    &raw mut p,
                    *ptr,
                    map_data_type(*(*opcode_info).src_info.add(i as usize)),
                    (&raw mut (*priv_).src_param as *mut wined3d_shader_src_param)
                        .add(i as usize) as *mut wined3d_shader_src_param,
                ) == 0
                {
                    (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
                    return;
                }
                i = i.wrapping_add(1);
            }
        }

        return;
    }

    *ptr = (*priv_).end;
    (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
}

unsafe extern "C" fn shader_sm4_is_end(data: *mut core::ffi::c_void, ptr: *mut *const DWORD) -> BOOL {
    let priv_0: *mut wined3d_sm4_data = data as *mut wined3d_sm4_data;
    return (*ptr == (*priv_0).end) as BOOL;
}

unsafe extern "C" fn read_dword(ptr: *mut *const i8) -> u32 {
    let mut ret: u32 = 0;
    memcpy(
        core::ptr::addr_of_mut!(ret) as *mut core::ffi::c_void,
        *ptr as *const core::ffi::c_void,
        core::mem::size_of::<u32>(),
    );
    *ptr = (*ptr).add(core::mem::size_of::<u32>()) as *const i8;
    return ret;
}

unsafe extern "C" fn require_space(offset: usize, count: usize, size: usize, data_size: usize) -> i32 {
    return (count == 0 || data_size.wrapping_sub(offset).wrapping_div(count) >= size) as i32;
}

unsafe extern "C" fn skip_dword_unknown(ptr: *mut *const i8, count: u32) {
    let mut i: u32;
    let mut d: u32;

    WARN!("Skipping %u unknown DWORDs:\n", count);
    i = 0;
    while i < count {
        d = read_dword(ptr);
        WARN!("\t0x%08x\n", d);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn shader_get_string(data: *const i8, data_size: usize, offset: u32) -> *const i8 {
    if offset as usize >= data_size {
        WARN!("Invalid offset %#x (data size %#lx).\n", offset, data_size as core::ffi::c_long);
        return core::ptr::null();
    }

    if memchr(
        data.add(offset as usize) as *const core::ffi::c_void,
        0,
        data_size - offset as usize,
    )
    .is_null()
    {
        return core::ptr::null();
    }

    return data.add(offset as usize);
}

unsafe extern "C" fn shader_parse_signature(
    mut tag: DWORD,
    mut data: *const core::ffi::c_char,
    mut data_size: core::ffi::c_uint,
    mut s: *mut wined3d_shader_signature,
) -> HRESULT {
    let mut e: *mut wined3d_shader_signature_element = core::ptr::null_mut();
    let mut has_stream_index: bool;
    let mut has_min_precision: bool;
    let mut ptr: *const core::ffi::c_char = data;
    let mut i: core::ffi::c_uint;
    let mut count: core::ffi::c_uint;

    if require_space(
        0,
        2,
        core::mem::size_of::<DWORD>() as size_t,
        data_size as size_t,
    ) == 0
    {
        WARN!("Invalid data size %#x.\n", data_size);
        return E_INVALIDARG;
    }

    count = read_dword(core::ptr::addr_of_mut!(ptr));

    skip_dword_unknown(core::ptr::addr_of_mut!(ptr), 1);

    if require_space(
        ptr.offset_from(data) as size_t,
        count as size_t,
        6 * (core::mem::size_of::<DWORD>() as size_t),
        data_size as size_t,
    ) == 0
    {
        WARN!("Invalid count %#x (data size %#x).\n", count, data_size);
        return E_INVALIDARG;
    }

    e = calloc(
        (count as size_t) as _,
        (core::mem::size_of::<wined3d_shader_signature_element>() as size_t) as _,
    ) as *mut wined3d_shader_signature_element;
    if e.is_null() {
        ERR!("Failed to allocate input signature memory.\n");
        return E_OUTOFMEMORY;
    }

    has_min_precision = tag == TAG_OSG1
        || tag == TAG_PSG1
        || tag == TAG_ISG1;
    has_stream_index = tag == TAG_OSG5 || has_min_precision;

    i = 0;
    while i < count {
        let mut name_offset: core::ffi::c_uint = 0;

        if has_stream_index {
            (*e.offset(i as isize)).stream_idx = read_dword(core::ptr::addr_of_mut!(ptr));
        } else {
            (*e.offset(i as isize)).stream_idx = 0;
        }
        name_offset = read_dword(core::ptr::addr_of_mut!(ptr));
        (*e.offset(i as isize)).semantic_name =
            shader_get_string(data, data_size as size_t, name_offset);
        if (*e.offset(i as isize)).semantic_name.is_null() {
            WARN!(
                "Invalid name offset %#x (data size %#x).\n",
                name_offset,
                data_size
            );
            free(e as *mut core::ffi::c_void);
            return E_INVALIDARG;
        }
        (*e.offset(i as isize)).semantic_idx = read_dword(core::ptr::addr_of_mut!(ptr));
        (*e.offset(i as isize)).sysval_semantic =
            read_dword(core::ptr::addr_of_mut!(ptr)) as wined3d_sysval_semantic;
        (*e.offset(i as isize)).component_type =
            read_dword(core::ptr::addr_of_mut!(ptr)) as wined3d_component_type;
        (*e.offset(i as isize)).register_idx = read_dword(core::ptr::addr_of_mut!(ptr));
        (*e.offset(i as isize)).mask =
            read_dword(core::ptr::addr_of_mut!(ptr)) as uint32_t;

        if has_min_precision {
            (*e.offset(i as isize)).min_precision = read_dword(core::ptr::addr_of_mut!(ptr));
        } else {
            (*e.offset(i as isize)).min_precision = 0;
        }

        TRACE!(
            "Stream: %u, semantic: %s, semantic idx: %u, sysval_semantic %#x, type %u, register idx: %u, use_mask %#x, input_mask %#x, min_precision %u.\n",
            (*e.offset(i as isize)).stream_idx,
            debugstr_a((*e.offset(i as isize)).semantic_name),
            (*e.offset(i as isize)).semantic_idx,
            (*e.offset(i as isize)).sysval_semantic as core::ffi::c_uint,
            (*e.offset(i as isize)).component_type as core::ffi::c_uint,
            (*e.offset(i as isize)).register_idx,
            ((*e.offset(i as isize)).mask >> 8) & (0xff as uint32_t),
            (*e.offset(i as isize)).mask & (0xff as uint32_t),
            (*e.offset(i as isize)).min_precision,
        );

        i = i.wrapping_add(1);
    }

    (*s).elements = e;
    (*s).element_count = count;

    return S_OK;
}

unsafe extern "C" fn shader_dxbc_process_section(shader: *mut wined3d_shader, max_version: ::core::ffi::c_uint, source_type: *mut vkd3d_shader_source_type, section: *const vkd3d_shader_dxbc_section_desc) -> HRESULT {
    let mut data_size: ::core::ffi::c_uint = (*section).data.size as ::core::ffi::c_uint;
    let mut data: *const ::core::ffi::c_void = (*section).data.code;
    let mut tag: uint32_t = (*section).tag;
    let mut hr: HRESULT = 0;
    match tag {
        TAG_ISGN | TAG_ISG1 => {
            if max_version < 4 as ::core::ffi::c_uint {
                TRACE!("Skipping shader input signature.\n");
            } else if !(*shader).input_signature.elements.is_null() {
                FIXME!("Multiple input signatures.\n");
            } else {
                hr = shader_parse_signature(
                    tag as DWORD,
                    data as *const ::core::ffi::c_char,
                    data_size,
                    core::ptr::addr_of_mut!((*shader).input_signature),
                );
                if hr < 0 {
                    return hr;
                }
            }
        }
        TAG_OSGN | TAG_OSG1 | TAG_OSG5 => {
            if max_version < 4 as ::core::ffi::c_uint {
                TRACE!("Skipping shader output signature.\n");
            } else if !(*shader).output_signature.elements.is_null() {
                FIXME!("Multiple output signatures.\n");
            } else {
                hr = shader_parse_signature(
                    tag as DWORD,
                    data as *const ::core::ffi::c_char,
                    data_size,
                    core::ptr::addr_of_mut!((*shader).output_signature),
                );
                if hr < 0 {
                    return hr;
                }
            }
        }
        TAG_PCSG | TAG_PSG1 => {
            if !(*shader).patch_constant_signature.elements.is_null() {
                FIXME!("Multiple patch constant signatures.\n");
            } else {
                hr = shader_parse_signature(
                    tag as DWORD,
                    data as *const ::core::ffi::c_char,
                    data_size,
                    core::ptr::addr_of_mut!((*shader).patch_constant_signature),
                );
                if hr < 0 {
                    return hr;
                }
            }
        }
        TAG_SHDR | TAG_SHEX => {
            if max_version < 4 as ::core::ffi::c_uint {
                TRACE!("Skipping SM4+ shader.\n");
            } else {
                if !(*shader).function.is_null() {
                    FIXME!("Multiple shader code chunks.\n");
                }
                (*shader).function = data as *const DWORD;
                (*shader).functionLength = data_size;
                *source_type = VKD3D_SHADER_SOURCE_DXBC_TPF;
            }
        }
        TAG_AON9 => {
            if max_version < 4 as ::core::ffi::c_uint {
                let mut header: *const aon9_header = data as *const aon9_header;
                let mut unknown_dword_count: ::core::ffi::c_uint = 0;
                let mut byte_code: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                if (data_size as usize) < core::mem::size_of::<aon9_header>() as usize {
                    WARN!("Invalid Aon9 data size %#x.\n", data_size);
                    return E_FAIL;
                }
                byte_code = data as *const ::core::ffi::c_char;
                byte_code = byte_code.offset((*header).byte_code_offset as isize);
                unknown_dword_count = ((*header).byte_code_offset as usize)
                    .wrapping_sub(core::mem::size_of::<aon9_header>() as usize)
                    .wrapping_div(core::mem::size_of::<DWORD>() as usize)
                    as ::core::ffi::c_uint;
                if (data_size as usize).wrapping_sub(
                    (2 as usize).wrapping_mul(core::mem::size_of::<DWORD>() as usize),
                ) < (*header).byte_code_offset as usize
                {
                    WARN!(
                        "Invalid byte code offset %#x (size %#x).\n",
                        (*header).byte_code_offset,
                        data_size
                    );
                    return E_FAIL;
                }
                FIXME!("Skipping %u unknown DWORDs.\n", unknown_dword_count);
                if !(*shader).function.is_null() {
                    FIXME!("Multiple shader code chunks.\n");
                }
                (*shader).function = byte_code as *const DWORD;
                (*shader).functionLength = data_size.wrapping_sub((*header).byte_code_offset);
                *source_type = VKD3D_SHADER_SOURCE_D3D_BYTECODE;
                TRACE!(
                    "Feature level 9 shader version 0%08x, 0%08lx.\n",
                    (*header).shader_version,
                    *(*shader).function
                );
            } else {
                TRACE!("Skipping feature level 9 shader code.\n");
            }
        }
        _ => {
            TRACE!("Skipping chunk %s.\n", debugstr_fourcc(tag as ::core::ffi::c_uint));
        }
    }
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_shader_extract_from_dxbc(shader: *mut wined3d_shader, max_shader_version: ::core::ffi::c_uint, source_type: *mut vkd3d_shader_source_type) -> HRESULT {
    let dxbc: vkd3d_shader_code = vkd3d_shader_code {
        code: (*shader).byte_code,
        size: (*shader).byte_code_size as size_t,
    };
    let mut dxbc_desc: vkd3d_shader_dxbc_desc = vkd3d_shader_dxbc_desc {
        tag: 0,
        checksum: [0; 4],
        version: 0,
        size: 0,
        section_count: 0,
        sections: core::ptr::null_mut(),
    };
    let mut hr: HRESULT = WINED3D_OK;
    let mut i: ::core::ffi::c_uint;
    let mut ret: ::core::ffi::c_int;

    ret = vkd3d_shader_parse_dxbc(
        core::ptr::addr_of!(dxbc),
        0,
        core::ptr::addr_of_mut!(dxbc_desc),
        core::ptr::null_mut(),
    );
    if ret < 0 {
        WARN!("Failed to parse DXBC, ret %d.\n", ret);
        return E_INVALIDARG;
    }

    i = 0;
    while (i as size_t) < dxbc_desc.section_count {
        hr = shader_dxbc_process_section(
            shader,
            max_shader_version,
            source_type,
            dxbc_desc.sections.offset(i as isize),
        );
        if hr < 0 {
            break;
        }
        i = i.wrapping_add(1);
    }
    vkd3d_shader_free_dxbc(core::ptr::addr_of_mut!(dxbc_desc));

    if (*shader).function.is_null() {
        hr = E_INVALIDARG;
    }

    if hr < 0 {
        WARN!("Failed to parse DXBC, hr %#lx.\n", hr);
    }

    return hr;
}
