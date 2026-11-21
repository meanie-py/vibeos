use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut vs_opcode_table: [wined3d_sm1_opcode_info; 58] = [
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_NOP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MOV,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MOV,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MOVA,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MOVA,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ADD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ADD,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SUB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SUB,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MAD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MAD,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MUL,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MUL,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RCP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RCP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RSQ,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RSQ,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DP3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DP3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DP4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DP4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MIN,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MIN,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MAX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MAX,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SLT,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SLT,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SGE,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SGE,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ABS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ABS,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_EXP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_EXP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOG,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOG,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_EXPP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_EXPP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOGP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOGP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LIT,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LIT,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DST,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DST,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LRP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LRP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_FRC,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_FRC,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_POW,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_POW,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CRS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CRS,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SGN,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SGN,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SGN,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SGN,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NRM,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_NRM,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SINCOS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SINCOS,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SINCOS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SINCOS,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M4x4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M4x4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M4x3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M4x3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x2,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x2,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DCL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DCL,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEF,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 5 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEF,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEFB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEFB,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEFI,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 5 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEFI,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_REP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_REP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDREP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDREP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_IF,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_IF,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_IFC,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_IFC,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ELSE,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ELSE,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDIF,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDIF,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAK,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAK,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAKC,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAKC,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAKP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAKP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CALL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CALL,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CALLNZ,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CALLNZ,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOOP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RET,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RET,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDLOOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDLOOP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LABEL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LABEL,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SETP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SETP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXLDL,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXLDL,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TABLE_SIZE,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
];

static mut ps_opcode_table: [wined3d_sm1_opcode_info; 85] = [
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_NOP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MOV,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MOV,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ADD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ADD,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SUB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SUB,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MAD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MAD,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MUL,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MUL,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RCP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RCP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RSQ,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RSQ,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DP3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DP3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DP4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DP4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MIN,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MIN,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_MAX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_MAX,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SLT,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SLT,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SGE,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SGE,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ABS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ABS,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_EXP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_EXP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOG,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOG,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_EXPP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_EXPP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOGP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOGP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DST,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DST,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LRP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LRP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_FRC,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_FRC,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CND,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CND,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CMP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CMP,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as DWORD,
        max_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_POW,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_POW,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CRS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CRS,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NRM,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_NRM,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SINCOS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SINCOS,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SINCOS,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SINCOS,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DP2ADD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 4 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DP2ADD,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M4x4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M4x4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M4x3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M4x3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x4,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x4,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x3,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_M3x2,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_M3x2,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DCL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DCL,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_REP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_REP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDREP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDREP,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_IF,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_IF,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_IFC,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_IFC,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ELSE,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ELSE,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDIF,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDIF,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAK,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAK,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAKC,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAKC,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BREAKP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BREAKP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CALL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CALL,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_CALLNZ,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_CALLNZ,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LOOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LOOP,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_RET,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_RET,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_ENDLOOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_ENDLOOP,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_LABEL,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_LABEL,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEF,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 5 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEF,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEFB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEFB,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DEFI,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 5 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DEFI,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXCOORD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXCOORD,
        min_version: 0 as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXCOORD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXCOORD,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXKILL,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXKILL,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEX,
        min_version: 0 as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEX,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEX,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXBEM,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXBEM,
        min_version: 0 as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXBEML,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXBEML,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXREG2AR,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXREG2AR,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXREG2GB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXREG2GB,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXREG2RGB,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXREG2RGB,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x2PAD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x2PAD,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x2TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x2TEX,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3PAD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3PAD,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3DIFF,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3DIFF,
        min_version: ((0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((0 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3SPEC,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3SPEC,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3VSPEC,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3VSPEC,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3TEX,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXDP3TEX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXDP3TEX,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x2DEPTH,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x2DEPTH,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXDP3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXDP3,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXM3x3,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXM3x3,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 2 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 3 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXDEPTH,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 1 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXDEPTH,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_BEM,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_BEM,
        min_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
        max_version: ((1 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 4 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DSX,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DSX,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_DSY,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 2 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_DSY,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXLDD,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 5 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXLDD,
        min_version: ((2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 1 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_SETP,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_SETP,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_TEXLDL,
        dst_count: 1 as ::core::ffi::c_uint,
        param_count: 3 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TEXLDL,
        min_version: ((3 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int
            | 0 as ::core::ffi::c_int) as DWORD,
        max_version: -(1 as ::core::ffi::c_int) as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_PHASE,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_PHASE,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
    wined3d_sm1_opcode_info {
        opcode: WINED3D_SM1_OP_NOP,
        dst_count: 0 as ::core::ffi::c_uint,
        param_count: 0 as ::core::ffi::c_uint,
        handler_idx: WINED3DSIH_TABLE_SIZE,
        min_version: 0 as DWORD,
        max_version: 0 as DWORD,
    },
];

static mut resource_type_table: [wined3d_shader_resource_type; 5] = [
    WINED3D_SHADER_RESOURCE_NONE,
    WINED3D_SHADER_RESOURCE_TEXTURE_1D,
    WINED3D_SHADER_RESOURCE_TEXTURE_2D,
    WINED3D_SHADER_RESOURCE_TEXTURE_CUBE,
    WINED3D_SHADER_RESOURCE_TEXTURE_3D,
];

#[no_mangle]
pub static mut sm1_shader_frontend: wined3d_shader_frontend = unsafe {
    wined3d_shader_frontend {
        shader_init: Some(
            shader_sm1_init
                as unsafe extern "C" fn(
                    *const DWORD,
                    size_t,
                    *const wined3d_shader_signature,
                ) -> *mut ::core::ffi::c_void,
        ),
        shader_free: Some(shader_sm1_free as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        shader_read_header: Some(
            shader_sm1_read_header
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut *const DWORD,
                    *mut wined3d_shader_version,
                ) -> (),
        ),
        shader_read_instruction: Some(
            shader_sm1_read_instruction
                as unsafe extern "C" fn(
                    *mut ::core::ffi::c_void,
                    *mut *const DWORD,
                    *mut wined3d_shader_instruction,
                ) -> (),
        ),
        shader_is_end: Some(
            shader_sm1_is_end
                as unsafe extern "C" fn(*mut ::core::ffi::c_void, *mut *const DWORD) -> BOOL,
        ),
    }
};

pub const WINED3D_SM1_DCL_USAGE_SHIFT: u32 = 0;
pub const WINED3D_SM1_DCL_USAGE_MASK: u32 = 0xf;
pub const WINED3D_SM1_DCL_USAGE_INDEX_SHIFT: u32 = 16;
pub const WINED3D_SM1_DCL_USAGE_INDEX_MASK: u32 = 0xf0000;
pub const WINED3D_SM1_RESOURCE_TYPE_SHIFT: u32 = 27;
pub const WINED3D_SM1_RESOURCE_TYPE_MASK: u32 = 0x78000000;
pub const WINED3D_SM1_OPCODE_MASK: u32 = 0x0000ffff;
pub const WINED3D_SM1_INSTRUCTION_FLAGS_SHIFT: u32 = 16;
pub const WINED3D_SM1_INSTRUCTION_FLAGS_MASK: u32 = 0xff0000;
pub const WINED3D_SM1_INSTRUCTION_LENGTH_SHIFT: u32 = 24;
pub const WINED3D_SM1_INSTRUCTION_LENGTH_MASK: u32 = 0x0f000000;
pub const WINED3D_SM1_COISSUE: u32 = 0x40000000;
pub const WINED3D_SM1_COMMENT_SIZE_SHIFT: u32 = 16;
pub const WINED3D_SM1_COMMENT_SIZE_MASK: u32 = 0x7fff0000;
pub const WINED3D_SM1_INSTRUCTION_PREDICATED: u32 = 0x10000000;
pub const WINED3D_SM1_REGISTER_NUMBER_MASK: u32 = 0x000007ff;
pub const WINED3D_SM1_REGISTER_TYPE_SHIFT: u32 = 28;
pub const WINED3D_SM1_REGISTER_TYPE_MASK: u32 = 0x70000000;
pub const WINED3D_SM1_REGISTER_TYPE_SHIFT2: u32 = 8;
pub const WINED3D_SM1_REGISTER_TYPE_MASK2: u32 = 0x1800;
pub const WINED3D_SM1_ADDRESS_MODE_SHIFT: u32 = 13;
pub const WINED3D_SM1_ADDRESS_MODE_MASK: u32 = 0x2000;
pub const WINED3D_SM1_DST_MODIFIER_SHIFT: u32 = 20;
pub const WINED3D_SM1_DST_MODIFIER_MASK: u32 = 0xf00000;
pub const WINED3D_SM1_DSTSHIFT_SHIFT: u32 = 24;
pub const WINED3D_SM1_DSTSHIFT_MASK: u32 = 0x0f000000;
pub const WINED3D_SM1_WRITEMASK_SHIFT: u32 = 16;
pub const WINED3D_SM1_WRITEMASK_MASK: u32 = 0xf0000;
pub const WINED3D_SM1_SWIZZLE_SHIFT: u32 = 16;
pub const WINED3D_SM1_SWIZZLE_MASK: u32 = 0xff0000;
pub const WINED3D_SM1_SRC_MODIFIER_SHIFT: u32 = 24;
pub const WINED3D_SM1_SRC_MODIFIER_MASK: u32 = 0x0f000000;
pub const WINED3D_SM1_END: u32 = 0x0000ffff;

pub const WINED3D_SM1_ADDRESS_MODE_ABSOLUTE: u32 = 0;
pub const WINED3D_SM1_ADDRESS_MODE_RELATIVE: u32 = 1 << 13;

pub const WINED3D_SM1_RESOURCE_UNKNOWN: u32 = 0x0;
pub const WINED3D_SM1_RESOURCE_TEXTURE_1D: u32 = 0x1;
pub const WINED3D_SM1_RESOURCE_TEXTURE_2D: u32 = 0x2;
pub const WINED3D_SM1_RESOURCE_TEXTURE_CUBE: u32 = 0x3;
pub const WINED3D_SM1_RESOURCE_TEXTURE_3D: u32 = 0x4;

pub const WINED3D_SM1_OP_NOP: u32 = 0x00;
pub const WINED3D_SM1_OP_MOV: u32 = 0x01;
pub const WINED3D_SM1_OP_ADD: u32 = 0x02;
pub const WINED3D_SM1_OP_SUB: u32 = 0x03;
pub const WINED3D_SM1_OP_MAD: u32 = 0x04;
pub const WINED3D_SM1_OP_MUL: u32 = 0x05;
pub const WINED3D_SM1_OP_RCP: u32 = 0x06;
pub const WINED3D_SM1_OP_RSQ: u32 = 0x07;
pub const WINED3D_SM1_OP_DP3: u32 = 0x08;
pub const WINED3D_SM1_OP_DP4: u32 = 0x09;
pub const WINED3D_SM1_OP_MIN: u32 = 0x0a;
pub const WINED3D_SM1_OP_MAX: u32 = 0x0b;
pub const WINED3D_SM1_OP_SLT: u32 = 0x0c;
pub const WINED3D_SM1_OP_SGE: u32 = 0x0d;
pub const WINED3D_SM1_OP_EXP: u32 = 0x0e;
pub const WINED3D_SM1_OP_LOG: u32 = 0x0f;
pub const WINED3D_SM1_OP_LIT: u32 = 0x10;
pub const WINED3D_SM1_OP_DST: u32 = 0x11;
pub const WINED3D_SM1_OP_LRP: u32 = 0x12;
pub const WINED3D_SM1_OP_FRC: u32 = 0x13;
pub const WINED3D_SM1_OP_M4x4: u32 = 0x14;
pub const WINED3D_SM1_OP_M4x3: u32 = 0x15;
pub const WINED3D_SM1_OP_M3x4: u32 = 0x16;
pub const WINED3D_SM1_OP_M3x3: u32 = 0x17;
pub const WINED3D_SM1_OP_M3x2: u32 = 0x18;
pub const WINED3D_SM1_OP_CALL: u32 = 0x19;
pub const WINED3D_SM1_OP_CALLNZ: u32 = 0x1a;
pub const WINED3D_SM1_OP_LOOP: u32 = 0x1b;
pub const WINED3D_SM1_OP_RET: u32 = 0x1c;
pub const WINED3D_SM1_OP_ENDLOOP: u32 = 0x1d;
pub const WINED3D_SM1_OP_LABEL: u32 = 0x1e;
pub const WINED3D_SM1_OP_DCL: u32 = 0x1f;
pub const WINED3D_SM1_OP_POW: u32 = 0x20;
pub const WINED3D_SM1_OP_CRS: u32 = 0x21;
pub const WINED3D_SM1_OP_SGN: u32 = 0x22;
pub const WINED3D_SM1_OP_ABS: u32 = 0x23;
pub const WINED3D_SM1_OP_NRM: u32 = 0x24;
pub const WINED3D_SM1_OP_SINCOS: u32 = 0x25;
pub const WINED3D_SM1_OP_REP: u32 = 0x26;
pub const WINED3D_SM1_OP_ENDREP: u32 = 0x27;
pub const WINED3D_SM1_OP_IF: u32 = 0x28;
pub const WINED3D_SM1_OP_IFC: u32 = 0x29;
pub const WINED3D_SM1_OP_ELSE: u32 = 0x2a;
pub const WINED3D_SM1_OP_ENDIF: u32 = 0x2b;
pub const WINED3D_SM1_OP_BREAK: u32 = 0x2c;
pub const WINED3D_SM1_OP_BREAKC: u32 = 0x2d;
pub const WINED3D_SM1_OP_MOVA: u32 = 0x2e;
pub const WINED3D_SM1_OP_DEFB: u32 = 0x2f;
pub const WINED3D_SM1_OP_DEFI: u32 = 0x30;

pub const WINED3D_SM1_OP_TEXCOORD: u32 = 0x40;
pub const WINED3D_SM1_OP_TEXKILL: u32 = 0x41;
pub const WINED3D_SM1_OP_TEX: u32 = 0x42;
pub const WINED3D_SM1_OP_TEXBEM: u32 = 0x43;
pub const WINED3D_SM1_OP_TEXBEML: u32 = 0x44;
pub const WINED3D_SM1_OP_TEXREG2AR: u32 = 0x45;
pub const WINED3D_SM1_OP_TEXREG2GB: u32 = 0x46;
pub const WINED3D_SM1_OP_TEXM3x2PAD: u32 = 0x47;
pub const WINED3D_SM1_OP_TEXM3x2TEX: u32 = 0x48;
pub const WINED3D_SM1_OP_TEXM3x3PAD: u32 = 0x49;
pub const WINED3D_SM1_OP_TEXM3x3TEX: u32 = 0x4a;
pub const WINED3D_SM1_OP_TEXM3x3DIFF: u32 = 0x4b;
pub const WINED3D_SM1_OP_TEXM3x3SPEC: u32 = 0x4c;
pub const WINED3D_SM1_OP_TEXM3x3VSPEC: u32 = 0x4d;
pub const WINED3D_SM1_OP_EXPP: u32 = 0x4e;
pub const WINED3D_SM1_OP_LOGP: u32 = 0x4f;
pub const WINED3D_SM1_OP_CND: u32 = 0x50;
pub const WINED3D_SM1_OP_DEF: u32 = 0x51;
pub const WINED3D_SM1_OP_TEXREG2RGB: u32 = 0x52;
pub const WINED3D_SM1_OP_TEXDP3TEX: u32 = 0x53;
pub const WINED3D_SM1_OP_TEXM3x2DEPTH: u32 = 0x54;
pub const WINED3D_SM1_OP_TEXDP3: u32 = 0x55;
pub const WINED3D_SM1_OP_TEXM3x3: u32 = 0x56;
pub const WINED3D_SM1_OP_TEXDEPTH: u32 = 0x57;
pub const WINED3D_SM1_OP_CMP: u32 = 0x58;
pub const WINED3D_SM1_OP_BEM: u32 = 0x59;
pub const WINED3D_SM1_OP_DP2ADD: u32 = 0x5a;
pub const WINED3D_SM1_OP_DSX: u32 = 0x5b;
pub const WINED3D_SM1_OP_DSY: u32 = 0x5c;
pub const WINED3D_SM1_OP_TEXLDD: u32 = 0x5d;
pub const WINED3D_SM1_OP_SETP: u32 = 0x5e;
pub const WINED3D_SM1_OP_TEXLDL: u32 = 0x5f;
pub const WINED3D_SM1_OP_BREAKP: u32 = 0x60;

pub const WINED3D_SM1_OP_PHASE: u32 = 0xfffd;
pub const WINED3D_SM1_OP_COMMENT: u32 = 0xfffe;
pub const WINED3D_SM1_OP_END: u32 = 0xffff;

#[repr(C)]
pub struct wined3d_sm1_opcode_info {
    pub opcode: u32,
    pub dst_count: u32,
    pub param_count: u32,
    pub handler_idx: WINED3D_SHADER_INSTRUCTION_HANDLER,
    pub min_version: DWORD,
    pub max_version: DWORD,
}

#[repr(C)]
pub struct wined3d_sm1_data {
    pub shader_version: wined3d_shader_version,
    pub opcode_table: *const wined3d_sm1_opcode_info,
    pub start: *const DWORD,
    pub src_rel_addr: [wined3d_shader_src_param; 4],
    pub pred_rel_addr: wined3d_shader_src_param,
    pub dst_rel_addr: wined3d_shader_src_param,
    pub src_param: [wined3d_shader_src_param; 4],
    pub pred_param: wined3d_shader_src_param,
    pub dst_param: wined3d_shader_dst_param,
}

unsafe extern "C" fn shader_get_param(
    priv_: *const wined3d_sm1_data,
    ptr: *const DWORD,
    token: *mut ::core::ffi::c_uint,
    addr_token: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_uint {
    let mut count: ::core::ffi::c_uint = 1;

    *token = *ptr as ::core::ffi::c_uint;

    if (*ptr & WINED3D_SM1_ADDRESS_MODE_RELATIVE) != 0 {
        if ((*priv_).shader_version.major as ::core::ffi::c_int) < 2 {
            *addr_token = (1 as ::core::ffi::c_uint) << 31
                | (((WINED3DSPR_ADDR as ::core::ffi::c_uint) << WINED3D_SM1_REGISTER_TYPE_SHIFT2)
                    & WINED3D_SM1_REGISTER_TYPE_MASK2)
                | (((WINED3DSPR_ADDR as ::core::ffi::c_uint) << WINED3D_SM1_REGISTER_TYPE_SHIFT)
                    & WINED3D_SM1_REGISTER_TYPE_MASK)
                | (WINED3DSP_NOSWIZZLE as ::core::ffi::c_uint) << WINED3D_SM1_SWIZZLE_SHIFT;
        } else {
            *addr_token = *ptr.offset(1 as isize) as ::core::ffi::c_uint;
            count = count.wrapping_add(1);
        }
    }

    return count;
}

unsafe extern "C" fn shader_get_opcode(
    mut priv_0: *const wined3d_sm1_data,
    mut token: ::core::ffi::c_uint,
) -> *const wined3d_sm1_opcode_info {
    let mut shader_version: ::core::ffi::c_uint = WINED3D_SHADER_VERSION!(
        (*priv_0).shader_version.major as ::core::ffi::c_uint,
        (*priv_0).shader_version.minor as ::core::ffi::c_uint,
    );
    let mut opcode_table: *const wined3d_sm1_opcode_info = (*priv_0).opcode_table;
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while (*opcode_table.offset(i as isize)).handler_idx as ::core::ffi::c_uint
        != WINED3DSIH_TABLE_SIZE as ::core::ffi::c_uint
    {
        if token & WINED3D_SM1_OPCODE_MASK as ::core::ffi::c_uint
            == (*opcode_table.offset(i as isize)).opcode as ::core::ffi::c_uint
            && shader_version >= (*opcode_table.offset(i as isize)).min_version
            && ((*opcode_table.offset(i as isize)).max_version == 0
                || shader_version <= (*opcode_table.offset(i as isize)).max_version)
        {
            return opcode_table.offset(i as isize) as *const wined3d_sm1_opcode_info;
        }
        i = i.wrapping_add(1);
    }
    WINE_FIXME!("Unsupported opcode %#x, token 0x%08x, shader version %#x.\n", token & 0xffff as ::core::ffi::c_uint, token, shader_version);
    return ::core::ptr::null::<wined3d_sm1_opcode_info>();
}

unsafe extern "C" fn shader_skip_opcode(
    priv_0: *const wined3d_sm1_data,
    opcode_info: *const wined3d_sm1_opcode_info,
    opcode_token: DWORD,
) -> ::core::ffi::c_uint {
    let mut length: ::core::ffi::c_uint = 0;
    length = (opcode_token as ::core::ffi::c_uint & WINED3D_SM1_INSTRUCTION_LENGTH_MASK) >> WINED3D_SM1_INSTRUCTION_LENGTH_SHIFT;
    return if (*priv_0).shader_version.major as ::core::ffi::c_int >= 2 {
        length
    } else {
        (*opcode_info).param_count
    };
}

unsafe extern "C" fn shader_parse_src_param(
    mut param: DWORD,
    mut rel_addr: *const wined3d_shader_src_param,
    mut src: *mut wined3d_shader_src_param,
) {
    (*src).reg.type_ = (((param & WINED3D_SM1_REGISTER_TYPE_MASK) >> WINED3D_SM1_REGISTER_TYPE_SHIFT)
        | ((param & WINED3D_SM1_REGISTER_TYPE_MASK2) >> WINED3D_SM1_REGISTER_TYPE_SHIFT2))
        as wined3d_shader_register_type;
    (*src).reg.data_type = WINED3D_DATA_FLOAT;
    (*src).reg.idx[0 as usize].offset =
        (param & WINED3D_SM1_REGISTER_NUMBER_MASK) as ::core::ffi::c_uint;
    (*src).reg.idx[0 as usize].rel_addr =
        rel_addr as *const wined3d_shader_src_param;
    (*src).reg.idx[1 as usize].offset =
        !(0 as ::core::ffi::c_uint);
    (*src).reg.idx[1 as usize].rel_addr =
        ::core::ptr::null::<wined3d_shader_src_param>();
    (*src).swizzle = ((param & WINED3D_SM1_SWIZZLE_MASK) >> WINED3D_SM1_SWIZZLE_SHIFT) as DWORD;
    (*src).modifiers = ((param & WINED3D_SM1_SRC_MODIFIER_MASK) >> WINED3D_SM1_SRC_MODIFIER_SHIFT)
        as wined3d_shader_src_modifier;
}

unsafe extern "C" fn shader_parse_dst_param(
    mut param: DWORD,
    mut rel_addr: *const wined3d_shader_src_param,
    mut dst: *mut wined3d_shader_dst_param,
) {
    (*dst).reg.type_ = ((param as ::core::ffi::c_uint & WINED3D_SM1_REGISTER_TYPE_MASK)
        >> WINED3D_SM1_REGISTER_TYPE_SHIFT
        | (param as ::core::ffi::c_uint & WINED3D_SM1_REGISTER_TYPE_MASK2)
            >> WINED3D_SM1_REGISTER_TYPE_SHIFT2)
        as wined3d_shader_register_type;
    (*dst).reg.data_type = WINED3D_DATA_FLOAT;
    (*dst).reg.idx[0 as usize].offset =
        (param & WINED3D_SM1_REGISTER_NUMBER_MASK as DWORD) as ::core::ffi::c_uint;
    (*dst).reg.idx[0 as usize].rel_addr =
        rel_addr as *const wined3d_shader_src_param;
    (*dst).reg.idx[1 as usize].offset =
        !(0 as ::core::ffi::c_uint);
    (*dst).reg.idx[1 as usize].rel_addr =
        ::core::ptr::null::<wined3d_shader_src_param>();
    (*dst).write_mask = ((param as ::core::ffi::c_uint & WINED3D_SM1_WRITEMASK_MASK)
        >> WINED3D_SM1_WRITEMASK_SHIFT) as uint32_t;
    (*dst).modifiers = ((param as ::core::ffi::c_uint & WINED3D_SM1_DST_MODIFIER_MASK)
        >> WINED3D_SM1_DST_MODIFIER_SHIFT) as uint32_t;
    (*dst).shift =
        (param as ::core::ffi::c_uint & WINED3D_SM1_DSTSHIFT_MASK) >> WINED3D_SM1_DSTSHIFT_SHIFT;
}

unsafe extern "C" fn shader_skip_unrecognized(
    mut priv_0: *const wined3d_sm1_data,
    mut ptr: *const DWORD,
) -> ::core::ffi::c_uint {
    let mut tokens_read: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    /* TODO: Think of a good name for 0x80000000 and replace it with a constant */
    while (*ptr & 0x80000000) != 0 {
        let mut token: ::core::ffi::c_uint = 0;
        let mut addr_token: ::core::ffi::c_uint = 0;
        let mut rel_addr: wined3d_shader_src_param = ::core::mem::zeroed();
        tokens_read = tokens_read.wrapping_add(shader_get_param(
            priv_0,
            ptr,
            &raw mut token,
            &raw mut addr_token,
        ));
        ptr = ptr.add(tokens_read as usize);

        if (token & WINED3D_SM1_ADDRESS_MODE_RELATIVE) != 0 {
            shader_parse_src_param(
                addr_token as DWORD,
                ::core::ptr::null::<wined3d_shader_src_param>(),
                &raw mut rel_addr,
            );
        }
        if i == 0 {
            let mut dst: wined3d_shader_dst_param = ::core::mem::zeroed();
            shader_parse_dst_param(
                token as DWORD,
                if (token & WINED3D_SM1_ADDRESS_MODE_RELATIVE) != 0 {
                    &raw mut rel_addr as *const wined3d_shader_src_param
                } else {
                    ::core::ptr::null::<wined3d_shader_src_param>()
                },
                &raw mut dst,
            );
        } else {
            let mut src: wined3d_shader_src_param = ::core::mem::zeroed();
            shader_parse_src_param(
                token as DWORD,
                if (token & WINED3D_SM1_ADDRESS_MODE_RELATIVE) != 0 {
                    &raw mut rel_addr as *const wined3d_shader_src_param
                } else {
                    ::core::ptr::null::<wined3d_shader_src_param>()
                },
                &raw mut src,
            );
        }
        i = i.wrapping_add(1);
    }
    return tokens_read;
}

unsafe extern "C" fn shader_sm1_init(
    byte_code: *const DWORD,
    byte_code_size: size_t,
    output_signature: *const wined3d_shader_signature,
) -> *mut core::ffi::c_void {
    let mut priv_: *mut wined3d_sm1_data = core::ptr::null_mut();
    let mut major: BYTE;
    let mut minor: BYTE;

    major = WINED3D_SM1_VERSION_MAJOR!(*byte_code) as BYTE;
    minor = WINED3D_SM1_VERSION_MINOR!(*byte_code) as BYTE;
    if WINED3D_SHADER_VERSION!(major, minor) > WINED3D_SHADER_VERSION!(3, 0) {
        WARN!("Invalid shader version %u.%u (%#lx).\n", major as core::ffi::c_uint, minor as core::ffi::c_uint, *byte_code);
        return core::ptr::null_mut();
    }

    priv_ = malloc(core::mem::size_of::<wined3d_sm1_data>() as size_t) as *mut wined3d_sm1_data;
    if priv_.is_null() {
        return core::ptr::null_mut();
    }

    if (*output_signature).element_count != 0 {
        FIXME!("SM 1-3 shader shouldn't have output signatures.\n");
    }

    match *byte_code >> 16 {
        WINED3D_SM1_VS => {
            (*priv_).shader_version.type_ = WINED3D_SHADER_TYPE_VERTEX;
            (*priv_).opcode_table = &raw const vs_opcode_table as *const wined3d_sm1_opcode_info;
        }

        WINED3D_SM1_PS => {
            (*priv_).shader_version.type_ = WINED3D_SHADER_TYPE_PIXEL;
            (*priv_).opcode_table = &raw const ps_opcode_table as *const wined3d_sm1_opcode_info;
        }

        _ => {
            FIXME!("Unrecognized shader type %#lx.\n", *byte_code >> 16);
            free(priv_ as *mut core::ffi::c_void);
            return core::ptr::null_mut();
        }
    }
    (*priv_).shader_version.major = WINED3D_SM1_VERSION_MAJOR!(*byte_code) as BYTE;
    (*priv_).shader_version.minor = WINED3D_SM1_VERSION_MINOR!(*byte_code) as BYTE;

    (*priv_).start = byte_code.offset(1 as isize) as *const DWORD;

    return priv_ as *mut core::ffi::c_void;
}

unsafe extern "C" fn shader_sm1_free(data: *mut core::ffi::c_void) {
    free(data);
}

unsafe extern "C" fn shader_sm1_read_header(data: *mut core::ffi::c_void, ptr: *mut *const DWORD, shader_version: *mut wined3d_shader_version) {
    let priv_: *mut wined3d_sm1_data = data as *mut wined3d_sm1_data;

    *ptr = (*priv_).start;
    *shader_version = (*priv_).shader_version;
}

unsafe extern "C" fn shader_sm1_read_src_param(
    priv_0: *mut wined3d_sm1_data,
    ptr: *mut *const DWORD,
    src_param: *mut wined3d_shader_src_param,
    src_rel_addr: *mut wined3d_shader_src_param,
) {
    let mut token: ::core::ffi::c_uint = 0;
    let mut addr_token: ::core::ffi::c_uint = 0;
    *ptr = (*ptr).offset(
        shader_get_param(priv_0, *ptr, &mut token, &mut addr_token) as isize,
    );
    if token & WINED3D_SM1_ADDRESS_MODE_RELATIVE != 0 {
        shader_parse_src_param(addr_token as DWORD, core::ptr::null_mut(), src_rel_addr);
        shader_parse_src_param(token as DWORD, src_rel_addr, src_param);
    } else {
        shader_parse_src_param(token as DWORD, core::ptr::null_mut(), src_param);
    }
}

unsafe extern "C" fn shader_sm1_read_dst_param(
    priv_: *mut wined3d_sm1_data,
    ptr: *mut *const DWORD,
    dst_param: *mut wined3d_shader_dst_param,
    dst_rel_addr: *mut wined3d_shader_src_param,
) {
    let mut token: ::core::ffi::c_uint = 0;
    let mut addr_token: ::core::ffi::c_uint = 0;

    *ptr = (*ptr).offset(
        shader_get_param(priv_, *ptr, core::ptr::addr_of_mut!(token), core::ptr::addr_of_mut!(addr_token)) as isize,
    );
    if (token & WINED3D_SM1_ADDRESS_MODE_RELATIVE) != 0 {
        shader_parse_src_param(
            addr_token as DWORD,
            core::ptr::null_mut::<wined3d_shader_src_param>(),
            dst_rel_addr,
        );
        shader_parse_dst_param(token as DWORD, dst_rel_addr, dst_param);
    } else {
        shader_parse_dst_param(
            token as DWORD,
            core::ptr::null_mut::<wined3d_shader_src_param>(),
            dst_param,
        );
    }
}

unsafe extern "C" fn shader_sm1_read_semantic(ptr: *mut *const DWORD, semantic: *mut wined3d_shader_semantic) {
    let mut resource_type: DWORD;
    let usage_token: DWORD = *(*ptr);
    *ptr = (*ptr).add(1);
    let dst_token: DWORD = *(*ptr);
    *ptr = (*ptr).add(1);

    (*semantic).usage = ((usage_token & WINED3D_SM1_DCL_USAGE_MASK) >> WINED3D_SM1_DCL_USAGE_SHIFT) as wined3d_decl_usage;
    (*semantic).usage_idx = ((usage_token & WINED3D_SM1_DCL_USAGE_INDEX_MASK) >> WINED3D_SM1_DCL_USAGE_INDEX_SHIFT) as UINT;
    resource_type = (usage_token & WINED3D_SM1_RESOURCE_TYPE_MASK) >> WINED3D_SM1_RESOURCE_TYPE_SHIFT;
    if resource_type as usize >= ARRAY_SIZE!(resource_type_table) as usize {
        FIXME!("Unhandled resource type %#x.\n", resource_type);
        (*semantic).resource_type = WINED3D_SHADER_RESOURCE_NONE;
    } else {
        (*semantic).resource_type = resource_type_table[resource_type as usize];
    }
    (*semantic).resource_data_type = WINED3D_DATA_FLOAT;
    shader_parse_dst_param(dst_token, core::ptr::null(), core::ptr::addr_of_mut!((*semantic).reg));
}

unsafe extern "C" fn shader_sm1_read_immconst(
    ptr: *mut *const DWORD,
    src_param: *mut wined3d_shader_src_param,
    type_0: wined3d_immconst_type,
    data_type: wined3d_data_type,
) {
    let count: ::core::ffi::c_uint = (if type_0 as ::core::ffi::c_uint == WINED3D_IMMCONST_VEC4 {
        4
    } else {
        1
    }) as ::core::ffi::c_uint;
    (*src_param).reg.type_ = WINED3DSPR_IMMCONST;
    (*src_param).reg.data_type = data_type;
    (*src_param).reg.idx[0 as usize].offset = !0;
    (*src_param).reg.idx[0 as usize].rel_addr = core::ptr::null_mut();
    (*src_param).reg.idx[1 as usize].offset = !0;
    (*src_param).reg.idx[1 as usize].rel_addr = core::ptr::null_mut();
    (*src_param).reg.immconst_type = type_0;
    memcpy(
        core::ptr::addr_of_mut!((*src_param).reg.u.immconst_data) as *mut ::core::ffi::c_void,
        *ptr as *const ::core::ffi::c_void,
        (count as usize).wrapping_mul(::core::mem::size_of::<DWORD>()),
    );
    (*src_param).swizzle = WINED3DSP_NOSWIZZLE;
    (*src_param).modifiers = 0;

    *ptr = (*ptr).offset(count as isize);
}

unsafe extern "C" fn shader_sm1_read_comment(ptr: *mut *const DWORD) {
    let mut token: DWORD = **ptr;
    let mut comment: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut size: UINT = 0;
    while token & WINED3D_SM1_OPCODE_MASK == WINED3D_SM1_OP_COMMENT {
        size = ((token & WINED3D_SM1_COMMENT_SIZE_MASK) >> WINED3D_SM1_COMMENT_SIZE_SHIFT) as UINT;
        *ptr = (*ptr).offset(1);
        comment = *ptr as *const ::core::ffi::c_char;
        *ptr = (*ptr).offset(size as isize);

        if size > 1
            && *(comment as *const DWORD)
                == WINEMAKEFOURCC!('T' as i32, 'E' as i32, 'X' as i32, 'T' as i32)
        {
            let end: *const ::core::ffi::c_char = comment.offset(
                (size as usize).wrapping_mul(::core::mem::size_of::<DWORD>()) as isize,
            );
            let mut p: *const ::core::ffi::c_char =
                comment.offset(::core::mem::size_of::<DWORD>() as isize);
            let mut line: *const ::core::ffi::c_char = p;

            while p != end {
                if *p as ::core::ffi::c_int == '\n' as i32 {
                    let mut len: UINT = p.offset_from(line) as UINT;
                    if len != 0
                        && *p.offset(-(1 as isize)) as ::core::ffi::c_int
                            == '\r' as i32
                    {
                        len = len.wrapping_sub(1);
                    }
                    p = p.offset(1);
                    line = p;
                } else {
                    p = p.offset(1);
                }
            }
            if line != p {
                TRACE!(
                    "// %s\n",
                    debugstr_an(line, p.offset_from(line) as ::core::ffi::c_int)
                );
            }
        } else if size != 0 {
            TRACE!(
                "// %s\n",
                debugstr_an(
                    comment,
                    (size as usize).wrapping_mul(::core::mem::size_of::<DWORD>()) as ::core::ffi::c_int
                )
            );
        } else {
            break;
        }
        token = **ptr;
    }
}

unsafe extern "C" fn shader_sm1_validate_instruction(ins: *mut wined3d_shader_instruction) {
    if (*ins).handler_idx == WINED3DSIH_BREAKP || (*ins).handler_idx == WINED3DSIH_IF {
        if (*ins).flags != 0 {
            (*ins).flags = 0;
        }
    }
}

unsafe extern "C" fn shader_sm1_read_instruction(data: *mut ::core::ffi::c_void, ptr: *mut *const DWORD, ins: *mut wined3d_shader_instruction) {
    let mut opcode_info: *const wined3d_sm1_opcode_info = ::core::ptr::null::<wined3d_sm1_opcode_info>();
    let mut priv_: *mut wined3d_sm1_data = data as *mut wined3d_sm1_data;
    let mut opcode_token: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut p: *const DWORD = ::core::ptr::null::<DWORD>();

    shader_sm1_read_comment(ptr);

    let fresh15 = *ptr;
    *ptr = (*ptr).offset(1);
    opcode_token = *fresh15 as ::core::ffi::c_uint;

    opcode_info = shader_get_opcode(priv_, opcode_token);
    if opcode_info.is_null() {
        (*ins).handler_idx = WINED3DSIH_TABLE_SIZE;
        *ptr = (*ptr).offset(shader_skip_unrecognized(priv_, *ptr) as isize);
        return;
    }

    (*ins).handler_idx = (*opcode_info).handler_idx;
    (*ins).flags = ((opcode_token & WINED3D_SM1_INSTRUCTION_FLAGS_MASK) >> WINED3D_SM1_INSTRUCTION_FLAGS_SHIFT) as uint32_t;
    (*ins).coissue = (opcode_token & WINED3D_SM1_COISSUE) as BOOL;
    (*ins).predicate = if opcode_token & WINED3D_SM1_INSTRUCTION_PREDICATED != 0 {
        &raw mut (*priv_).pred_param
    } else {
        ::core::ptr::null_mut::<wined3d_shader_src_param>()
    };
    (*ins).dst_count = (if (*opcode_info).dst_count != 0 {
        1
    } else {
        0
    }) as ::core::ffi::c_uint;
    (*ins).dst = &raw mut (*priv_).dst_param;
    (*ins).src_count = (*opcode_info).param_count.wrapping_sub((*opcode_info).dst_count);
    (*ins).src = &raw mut (*priv_).src_param as *mut wined3d_shader_src_param;
    memset(
        &raw mut (*ins).texel_offset as *mut ::core::ffi::c_void,
        0,
        ::core::mem::size_of::<wined3d_shader_texel_offset>() as size_t,
    );

    p = *ptr;
    *ptr = (*ptr).offset(shader_skip_opcode(priv_, opcode_info, opcode_token as DWORD) as isize);

    if (*ins).handler_idx as ::core::ffi::c_uint == WINED3DSIH_DCL as ::core::ffi::c_uint {
        shader_sm1_read_semantic(&raw mut p, &raw mut (*ins).declaration.semantic);
    } else if (*ins).handler_idx as ::core::ffi::c_uint == WINED3DSIH_DEF as ::core::ffi::c_uint {
        shader_sm1_read_dst_param(
            priv_,
            &raw mut p,
            &raw mut (*priv_).dst_param,
            &raw mut (*priv_).dst_rel_addr,
        );
        shader_sm1_read_immconst(
            &raw mut p,
            (&raw mut (*priv_).src_param as *mut wined3d_shader_src_param)
                .offset(0 as isize) as *mut wined3d_shader_src_param,
            WINED3D_IMMCONST_VEC4,
            WINED3D_DATA_FLOAT,
        );
    } else if (*ins).handler_idx as ::core::ffi::c_uint == WINED3DSIH_DEFB as ::core::ffi::c_uint {
        shader_sm1_read_dst_param(
            priv_,
            &raw mut p,
            &raw mut (*priv_).dst_param,
            &raw mut (*priv_).dst_rel_addr,
        );
        shader_sm1_read_immconst(
            &raw mut p,
            (&raw mut (*priv_).src_param as *mut wined3d_shader_src_param)
                .offset(0 as isize) as *mut wined3d_shader_src_param,
            WINED3D_IMMCONST_SCALAR,
            WINED3D_DATA_UINT,
        );
    } else if (*ins).handler_idx as ::core::ffi::c_uint == WINED3DSIH_DEFI as ::core::ffi::c_uint {
        shader_sm1_read_dst_param(
            priv_,
            &raw mut p,
            &raw mut (*priv_).dst_param,
            &raw mut (*priv_).dst_rel_addr,
        );
        shader_sm1_read_immconst(
            &raw mut p,
            (&raw mut (*priv_).src_param as *mut wined3d_shader_src_param)
                .offset(0 as isize) as *mut wined3d_shader_src_param,
            WINED3D_IMMCONST_VEC4,
            WINED3D_DATA_INT,
        );
    } else {
        /* Destination token */
        if (*ins).dst_count != 0 {
            shader_sm1_read_dst_param(
                priv_,
                &raw mut p,
                &raw mut (*priv_).dst_param,
                &raw mut (*priv_).dst_rel_addr,
            );
        }

        /* Predication token */
        if !(*ins).predicate.is_null() {
            shader_sm1_read_src_param(
                priv_,
                &raw mut p,
                &raw mut (*priv_).pred_param,
                &raw mut (*priv_).pred_rel_addr,
            );
        }

        /* Other source tokens */
        i = 0 as ::core::ffi::c_uint;
        while i < (*ins).src_count {
            shader_sm1_read_src_param(
                priv_,
                &raw mut p,
                (&raw mut (*priv_).src_param as *mut wined3d_shader_src_param).offset(i as isize)
                    as *mut wined3d_shader_src_param,
                (&raw mut (*priv_).src_rel_addr as *mut wined3d_shader_src_param).offset(i as isize)
                    as *mut wined3d_shader_src_param,
            );
            i = i.wrapping_add(1);
        }
    }

    shader_sm1_validate_instruction(ins);
}

unsafe extern "C" fn shader_sm1_is_end(data: *mut ::core::ffi::c_void, ptr: *mut *const DWORD) -> BOOL {
    shader_sm1_read_comment(ptr);
    if **ptr == WINED3D_SM1_END as DWORD {
        *ptr = (*ptr).offset(1);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
