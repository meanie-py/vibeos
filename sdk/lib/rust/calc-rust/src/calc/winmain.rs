use crate::calc::convert::ConvAdjust;
use crate::calc::convert::ConvExecute;
use crate::calc::convert::ConvInit;
use crate::calc::htmlhelp::HtmlHelp_Start;
use crate::calc::htmlhelp::HtmlHelp_Stop;
use crate::calc::theme::Theme_Start;
use crate::calc::theme::Theme_Stop;
use crate::calc::fun_ieee::apply_int_mask;
use crate::calc::utl_ieee::convert_real_integer;
use crate::calc::utl_ieee::convert_text2number_2;
use crate::calc::rpn_ieee::eval_parent_count;
use crate::calc::rpn_ieee::exec_change_infix;
use crate::calc::rpn_ieee::exec_closeparent;
use crate::calc::rpn_ieee::exec_infix2postfix;
use crate::calc::rpn_ieee::flush_postfix;
use crate::calc::utl_ieee::prepare_rpn_result_2;
use crate::calc::fun_ieee::rpn_2pi;
use crate::calc::fun_ieee::rpn_acos;
use crate::calc::fun_ieee::rpn_acosh;
use crate::calc::fun_ieee::rpn_alloc;
use crate::calc::fun_ieee::rpn_asin;
use crate::calc::fun_ieee::rpn_asinh;
use crate::calc::fun_ieee::rpn_atan;
use crate::calc::fun_ieee::rpn_atanh;
use crate::calc::fun_ieee::rpn_ave;
use crate::calc::fun_ieee::rpn_ave2;
use crate::calc::fun_ieee::rpn_cbrt;
use crate::calc::fun_ieee::rpn_copy;
use crate::calc::fun_ieee::rpn_cos;
use crate::calc::fun_ieee::rpn_cosh;
use crate::calc::fun_ieee::rpn_dec2dms;
use crate::calc::fun_ieee::rpn_dms2dec;
use crate::calc::fun_ieee::rpn_exp;
use crate::calc::fun_ieee::rpn_exp10;
use crate::calc::fun_ieee::rpn_exp2;
use crate::calc::fun_ieee::rpn_exp3;
use crate::calc::fun_ieee::rpn_fact;
use crate::calc::fun_ieee::rpn_frac;
use crate::calc::fun_ieee::rpn_free;
use crate::calc::fun_ieee::rpn_int;
use crate::calc::fun_ieee::rpn_is_zero;
use crate::calc::fun_ieee::rpn_ln;
use crate::calc::fun_ieee::rpn_log;
use crate::calc::fun_ieee::rpn_not;
use crate::calc::fun_ieee::rpn_pi;
use crate::calc::fun_ieee::rpn_reci;
use crate::calc::fun_ieee::rpn_s;
use crate::calc::fun_ieee::rpn_s_m1;
use crate::calc::fun_ieee::rpn_sign;
use crate::calc::fun_ieee::rpn_sin;
use crate::calc::fun_ieee::rpn_sinh;
use crate::calc::fun_ieee::rpn_sqrt;
use crate::calc::fun_ieee::rpn_sum;
use crate::calc::fun_ieee::rpn_sum2;
use crate::calc::fun_ieee::rpn_tan;
use crate::calc::fun_ieee::rpn_tanh;
use crate::calc::fun_ieee::rpn_zero;
use crate::calc::rpn_ieee::run_operator;
use crate::calc::rpn_ieee::start_rpn_engine;
use crate::calc::rpn_ieee::stop_rpn_engine;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_8D7C97326263: [u16; 7] = [0x0042u16, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut key2code: [key3code_t; 68] = [
    key3code_t {
        idc: IDC_BUTTON_STA as WORD,
        key: CTRL_S as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_AVE as WORD,
        key: CTRL_A as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_SUM as WORD,
        key: CTRL_T as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_S as WORD,
        key: CTRL_D as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_MS as WORD,
        key: CTRL_M as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_MR as WORD,
        key: CTRL_R as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_MP as WORD,
        key: CTRL_P as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_MC as WORD,
        key: CTRL_L as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_0 as WORD,
        key: '0' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_1 as WORD,
        key: '1' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_2 as WORD,
        key: '2' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_3 as WORD,
        key: '3' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_4 as WORD,
        key: '4' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_5 as WORD,
        key: '5' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_6 as WORD,
        key: '6' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_7 as WORD,
        key: '7' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_8 as WORD,
        key: '8' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_9 as WORD,
        key: '9' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_DOT as WORD,
        key: '.' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_DOT as WORD,
        key: ',' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_BUTTON_ADD as WORD,
        key: '+' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_SUB as WORD,
        key: '-' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_MULT as WORD,
        key: '*' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_DIV as WORD,
        key: '/' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_AND as WORD,
        key: '&' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_OR as WORD,
        key: '|' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_XOR as WORD,
        key: '^' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_LSH as WORD,
        key: '<' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_NOT as WORD,
        key: '~' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_INT as WORD,
        key: ';' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_EQU as WORD,
        key: '=' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_A as WORD,
        key: 'A' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_B as WORD,
        key: 'B' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_C as WORD,
        key: 'C' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_D as WORD,
        key: 'D' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_E as WORD,
        key: 'E' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_F as WORD,
        key: 'F' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_CHECK_HYP as WORD,
        key: 'H' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_CHECK_INV as WORD,
        key: 'I' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_BUTTON_LOG as WORD,
        key: 'L' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_DMS as WORD,
        key: 'M' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_LN as WORD,
        key: 'N' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_PI as WORD,
        key: 'P' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_RX as WORD,
        key: 'R' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_SIN as WORD,
        key: 'S' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_COS as WORD,
        key: 'O' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_TAN as WORD,
        key: 'T' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_FE as WORD,
        key: 'V' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_EXP as WORD,
        key: 'X' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_XeY as WORD,
        key: 'Y' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_SQRT as WORD,
        key: '@' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_Xe2 as WORD,
        key: '@' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_Xe3 as WORD,
        key: '#' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_NF as WORD,
        key: '!' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_LEFTPAR as WORD,
        key: '(' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_RIGHTPAR as WORD,
        key: ')' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_PURP,
    },
    key3code_t {
        idc: IDC_BUTTON_MOD as WORD,
        key: '%' as i32 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_PERCENT as WORD,
        key: '%' as i32 as CHAR,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_DAT as WORD,
        key: VK_INSERT as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
    key3code_t {
        idc: IDC_BUTTON_EQU as WORD,
        key: VK_RETURN as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_CANC as WORD,
        key: VK_ESCAPE as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_CE as WORD,
        key: VK_DELETE as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_BUTTON_BACK as WORD,
        key: VK_BACK as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_RED,
    },
    key3code_t {
        idc: IDC_RADIO_HEX as WORD,
        key: VK_F5 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_RADIO_DEC as WORD,
        key: VK_F6 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_RADIO_OCT as WORD,
        key: VK_F7 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_RADIO_BIN as WORD,
        key: VK_F8 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: -(1 as ::core::ffi::c_int) as COLORREF,
    },
    key3code_t {
        idc: IDC_BUTTON_SIGN as WORD,
        key: VK_F9 as CHAR,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 6 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int) as BYTE,
        col: CALC_CLR_BLUE,
    },
];

static mut key2code_base16: [key2code_t; 4] = [
    key2code_t {
        key: VK_F2 as CHAR,
        idc: IDC_RADIO_DWORD as WORD,
    },
    key2code_t {
        key: VK_F3 as CHAR,
        idc: IDC_RADIO_WORD as WORD,
    },
    key2code_t {
        key: VK_F4 as CHAR,
        idc: IDC_RADIO_BYTE as WORD,
    },
    key2code_t {
        key: VK_F12 as CHAR,
        idc: IDC_RADIO_QWORD as WORD,
    },
];

static mut key2code_base10: [key2code_t; 3] = [
    key2code_t {
        key: VK_F2 as CHAR,
        idc: IDC_RADIO_DEG as WORD,
    },
    key2code_t {
        key: VK_F3 as CHAR,
        idc: IDC_RADIO_RAD as WORD,
    },
    key2code_t {
        key: VK_F4 as CHAR,
        idc: IDC_RADIO_GRAD as WORD,
    },
];

static mut operator_codes: [WORD; 15] = [
    IDC_STATIC as WORD,
    IDC_BUTTON_PERCENT as WORD,
    IDC_BUTTON_EQU as WORD,
    IDC_BUTTON_OR as WORD,
    IDC_BUTTON_XOR as WORD,
    IDC_BUTTON_AND as WORD,
    IDC_BUTTON_LSH as WORD,
    IDC_BUTTON_RSH as WORD,
    IDC_BUTTON_ADD as WORD,
    IDC_BUTTON_SUB as WORD,
    IDC_BUTTON_MULT as WORD,
    IDC_BUTTON_DIV as WORD,
    IDC_BUTTON_MOD as WORD,
    IDC_BUTTON_XeY as WORD,
    IDC_BUTTON_XrY as WORD,
];

static mut function_table: [function_table_t; 24] = unsafe {
    [
        function_table_t {
            idc: IDC_BUTTON_SIN as WORD,
            range: (MODIFIER_INV | MODIFIER_HYP) as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_sin as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_asin as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: Some(rpn_sinh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inv_hyp: Some(rpn_asinh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
        },
        function_table_t {
            idc: IDC_BUTTON_COS as WORD,
            range: (MODIFIER_INV | MODIFIER_HYP) as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_cos as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_acos as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: Some(rpn_cosh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inv_hyp: Some(rpn_acosh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
        },
        function_table_t {
            idc: IDC_BUTTON_TAN as WORD,
            range: (MODIFIER_INV | MODIFIER_HYP) as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_tan as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_atan as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: Some(rpn_tanh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inv_hyp: Some(rpn_atanh as unsafe extern "C" fn(*mut calc_number_t) -> ()),
        },
        function_table_t {
            idc: IDC_BUTTON_INT as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_int as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_frac as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_RX as WORD,
            range: 0 as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_reci as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_NOT as WORD,
            range: 0 as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_not as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_PI as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(rpn_pi as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_2pi as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_Xe2 as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_exp2 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_sqrt as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_Xe3 as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_exp3 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_cbrt as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_LN as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_ln as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_exp as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_LOG as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_log as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_exp10 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_NF as WORD,
            range: 0 as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_fact as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_AVE as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(rpn_ave as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_ave2 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_SUM as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(rpn_sum as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_sum2 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_S as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(rpn_s_m1 as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_s as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_SQRT as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_sqrt as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_DMS as WORD,
            range: MODIFIER_INV as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(rpn_dec2dms as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(rpn_dms2dec as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_FE as WORD,
            range: 0 as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(run_fe as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_DAT as WORD,
            range: 0 as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(run_dat_sta as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_MP as WORD,
            range: (MODIFIER_INV | NO_CHAIN) as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(run_mp as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(run_mm as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_MS as WORD,
            range: (MODIFIER_INV | NO_CHAIN) as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(run_ms as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: Some(run_mw as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_CANC as WORD,
            range: NO_CHAIN as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(run_canc as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_RIGHTPAR as WORD,
            range: NO_CHAIN as BYTE,
            check_nan: 1 as BYTE,
            direct: Some(run_rpar as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
        function_table_t {
            idc: IDC_BUTTON_LEFTPAR as WORD,
            range: NO_CHAIN as BYTE,
            check_nan: 0 as BYTE,
            direct: Some(run_lpar as unsafe extern "C" fn(*mut calc_number_t) -> ()),
            inverse: None,
            hyperb: None,
            inv_hyp: None,
        },
    ]
};

#[no_mangle]
pub static mut calc: calc_t = calc_t {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    hWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hBgIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    hSmIcon: ::core::ptr::null::<HICON__>() as *mut HICON__,
    layout: 0,
    buffer: [0; 256],
    source: [0; 256],
    ptr: ::core::ptr::null::<TCHAR>() as *mut TCHAR,
    code: calc_number_t { f: 0. },
    prev: calc_number_t { f: 0. },
    memory: calc_node_t {
        number: calc_number_t { f: 0. },
        operation: 0,
        base: 0,
    },
    stat: ::core::ptr::null::<statistic_t>() as *mut statistic_t,
    is_memory: 0,
    is_nan: 0,
    sci_out: 0,
    sci_in: 0,
    usesep: 0,
    is_menu_on: 0,
    esp: 0,
    base: 0,
    size: 0,
    degr: 0,
    action: 0,
    hStatWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hConvWnd: ::core::ptr::null::<HWND__>() as *mut HWND__,
    Clipboard: sequence_t {
        data: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        ptr: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        wm_msg: 0,
    },
    Convert: [sequence_t {
        data: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        ptr: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
        wm_msg: 0,
    }; 2],
    last_operator: 0,
    prev_operator: 0,
    sDecimal: [0; 8],
    sThousand: [0; 8],
    sDecimal_len: 0,
    sThousand_len: 0,
    x_coord: 0,
    y_coord: 0,
};

#[no_mangle]
pub static mut BtnInfo: [BTNINFO; 255] = [const {
    BTNINFO {
        bHover: 0,
        oldProc: None,
    }
}; 255];

#[no_mangle]
pub static mut BtnCount: UINT = 0;

static mut upd: [_update_check_menus; 14] = unsafe { [
        _update_check_menus {
            sel: &raw mut calc.layout,
            idm: IDM_VIEW_STANDARD as WORD,
            idc: CALC_LAYOUT_STANDARD as ::core::ffi::c_int as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.layout,
            idm: IDM_VIEW_SCIENTIFIC as WORD,
            idc: CALC_LAYOUT_SCIENTIFIC as ::core::ffi::c_int as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.layout,
            idm: IDM_VIEW_CONVERSION as WORD,
            idc: CALC_LAYOUT_CONVERSION as ::core::ffi::c_int as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.base,
            idm: IDM_VIEW_HEX as WORD,
            idc: IDC_RADIO_HEX as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.base,
            idm: IDM_VIEW_DEC as WORD,
            idc: IDC_RADIO_DEC as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.base,
            idm: IDM_VIEW_OCT as WORD,
            idc: IDC_RADIO_OCT as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.base,
            idm: IDM_VIEW_BIN as WORD,
            idc: IDC_RADIO_BIN as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.degr,
            idm: IDM_VIEW_DEG as WORD,
            idc: IDC_RADIO_DEG as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.degr,
            idm: IDM_VIEW_RAD as WORD,
            idc: IDC_RADIO_RAD as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.degr,
            idm: IDM_VIEW_GRAD as WORD,
            idc: IDC_RADIO_GRAD as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.size,
            idm: IDM_VIEW_QWORD as WORD,
            idc: IDC_RADIO_QWORD as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.size,
            idm: IDM_VIEW_DWORD as WORD,
            idc: IDC_RADIO_DWORD as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.size,
            idm: IDM_VIEW_WORD as WORD,
            idc: IDC_RADIO_WORD as WORD,
        },
        _update_check_menus {
            sel: &raw mut calc.size,
            idm: IDM_VIEW_BYTE as WORD,
            idc: IDC_RADIO_BYTE as WORD,
        },
    ] };

static mut radio_setup: [radio_config_t; 7] = [
    radio_config_t {
        idc: IDC_RADIO_QWORD as WORD,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_DWORD as WORD,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_WORD as WORD,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_BYTE as WORD,
        mask: ((1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_DEG as WORD,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_RAD as WORD,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
    radio_config_t {
        idc: IDC_RADIO_GRAD as WORD,
        mask: ((0 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int
            | (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int
            | (0 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int) as WORD,
    },
];

pub const KEY_IS_UP: u32 = 0x8000_0000;

pub const KEY_WAS_DOWN: u32 = 0x4000_0000;

pub const BITMASK_IS_ASCII: u8 = 0x80;

pub const BITMASK_IS_STATS: u8 = 0x40;

pub const BITMASK_IS_CTRL: u8 = 0x20;

pub const BITMASK_HEX_MASK: u8 = 0x08;

pub const BITMASK_DEC_MASK: u8 = 0x04;

pub const BITMASK_OCT_MASK: u8 = 0x02;

pub const BITMASK_BIN_MASK: u8 = 0x01;

// RGB(0xFF, 0x00, 0x00)
pub const CALC_CLR_RED: COLORREF = 0x0000_00FF;

// RGB(0x00, 0x00, 0xFF)
pub const CALC_CLR_BLUE: COLORREF = 0x00FF_0000;

// RGB(0xFF, 0x00, 0xFF)
pub const CALC_CLR_PURP: COLORREF = 0x00FF_00FF;

#[repr(C)]
pub struct key2code_t {
    pub key: CHAR, // Virtual key identifier
    pub idc: WORD, // IDC for posting message
}

#[repr(C)]
pub struct key3code_t {
    pub idc: WORD,     // IDC for posting message
    pub key: CHAR,     // Virtual key identifier
    pub mask: BYTE,    // enable/disable into the various modes.
    pub col: COLORREF, // color used for drawing the text
}

pub const CTRL_FLAG: u32 = 0x100;

pub const ALT_FLAG: u32 = 0x200;

pub const CTRL_A: i8 = 1; // (0x0001+'A'-'A')

pub const CTRL_C: i8 = 3; // (0x0001+'C'-'A')

pub const CTRL_D: i8 = 4; // (0x0001+'D'-'A')

pub const CTRL_L: i8 = 12; // (0x0001+'L'-'A')

pub const CTRL_M: i8 = 13; // (0x0001+'M'-'A')

pub const CTRL_P: i8 = 16; // (0x0001+'P'-'A')

pub const CTRL_R: i8 = 18; // (0x0001+'R'-'A')

pub const CTRL_S: i8 = 19; // (0x0001+'S'-'A')

pub const CTRL_T: i8 = 20; // (0x0001+'T'-'A')

pub const CTRL_V: i8 = 22; // (0x0001+'V'-'A')

pub const CTRL_Z: i8 = 26; // (0x0001+'Z'-'A')

pub type rpn_callback1 = Option<unsafe extern "C" fn(*mut calc_number_t)>;

#[repr(C)]
pub struct function_table_t {
    pub idc: WORD,
    pub range: BYTE,
    pub check_nan: BYTE,
    pub direct: rpn_callback1,
    pub inverse: rpn_callback1,
    pub hyperb: rpn_callback1,
    pub inv_hyp: rpn_callback1,
}

#[repr(C)]
pub struct BTNINFO {
    pub bHover: BOOL,
    pub oldProc: WNDPROC,
}

pub type LPBTNINFO = *mut BTNINFO;

#[repr(C)]
pub struct _update_check_menus {
    pub sel: *mut DWORD,
    pub idm: WORD,
    pub idc: WORD,
}

unsafe impl Sync for _update_check_menus {}

#[repr(C)]
pub struct radio_config_t {
    pub idc: WORD,
    pub mask: WORD,
}

unsafe extern "C" fn UpdateNumberIntl() {
    /* Get current user defaults */
    if GetLocaleInfo(
        LOCALE_USER_DEFAULT,
        LOCALE_SDECIMAL,
        core::ptr::addr_of_mut!(calc.sDecimal) as LPWSTR,
        (core::mem::size_of::<[TCHAR; 8]>() as usize)
            .wrapping_div(core::mem::size_of::<TCHAR>() as usize) as ::core::ffi::c_int,
    ) == 0
    {
        let mut dot: [u16; 2] = ['.' as u16, 0];
        StringCbCopy(
            core::ptr::addr_of_mut!(calc.sDecimal) as STRSAFE_LPWSTR,
            core::mem::size_of::<[TCHAR; 8]>() as size_t,
            dot.as_ptr() as STRSAFE_LPCWSTR,
        );
    }

    if GetLocaleInfo(
        LOCALE_USER_DEFAULT,
        LOCALE_STHOUSAND,
        core::ptr::addr_of_mut!(calc.sThousand) as LPWSTR,
        (core::mem::size_of::<[TCHAR; 8]>() as usize)
            .wrapping_div(core::mem::size_of::<TCHAR>() as usize) as ::core::ffi::c_int,
    ) == 0
    {
        let mut comma: [u16; 2] = [',' as u16, 0];
        StringCbCopy(
            core::ptr::addr_of_mut!(calc.sThousand) as STRSAFE_LPWSTR,
            core::mem::size_of::<[TCHAR; 8]>() as size_t,
            comma.as_ptr() as STRSAFE_LPCWSTR,
        );
    }

    /* get the string lengths */
    calc.sDecimal_len = _tcslen(
        core::ptr::addr_of_mut!(calc.sDecimal) as *mut TCHAR,
    ) as ::core::ffi::c_uint;
    calc.sThousand_len = _tcslen(
        core::ptr::addr_of_mut!(calc.sThousand) as *mut TCHAR,
    ) as ::core::ffi::c_uint;
}

unsafe extern "C" fn LoadRegInt(lpszApp: LPCTSTR, lpszKey: LPCTSTR, mut iDefault: core::ffi::c_int) -> core::ffi::c_int {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut iValue: core::ffi::c_int = 0;
    let mut tmp: DWORD = 0;

    if (RegOpenKeyEx(HKEY_CURRENT_USER, lpszApp, 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
        /* Try to load integer value */
        tmp = core::mem::size_of::<core::ffi::c_int>() as DWORD;

        if (RegQueryValueEx(hKey, lpszKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(iValue) as LPBYTE, core::ptr::addr_of_mut!(tmp))) as u32 == ERROR_SUCCESS {
            iDefault = iValue;
        }

        /* close the key */
        RegCloseKey(hKey);
    }

    return iDefault;
}

unsafe extern "C" fn SaveRegInt(lpszApp: LPCTSTR, lpszKey: LPCTSTR, mut iValue: i32) {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegCreateKeyEx(
        HKEY_CURRENT_USER,
        lpszApp,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hKey),
        core::ptr::null_mut(),
    )) as u32 == ERROR_SUCCESS
    {
        RegSetValueEx(
            hKey,
            lpszKey,
            0,
            REG_DWORD,
            core::ptr::addr_of_mut!(iValue) as *const BYTE,
            core::mem::size_of::<i32>() as DWORD,
        );

        /* close the key */
        RegCloseKey(hKey);
    }
}

unsafe extern "C" fn load_config() {
    let mut osvi: OSVERSIONINFO = core::mem::zeroed();

    osvi.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as DWORD;
    GetVersionEx(core::ptr::addr_of_mut!(osvi));

    match osvi.dwPlatformId {
        VER_PLATFORM_WIN32s | VER_PLATFORM_WIN32_WINDOWS => {
            /* Try to load last selected layout */
            let sci_calc: [u16; 8] = ['S' as u16, 'c' as u16, 'i' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0];
            let layout_key: [u16; 7] = ['l' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 0];
            calc.layout = GetProfileInt(
                sci_calc.as_ptr(),
                layout_key.as_ptr(),
                (CALC_LAYOUT_STANDARD as i32),
            ) as DWORD;

            /* Try to load last selected formatting option */
            let use_sep_key: [u16; 7] = ['U' as u16, 's' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'p' as u16, 0];
            calc.usesep = if GetProfileInt(
                sci_calc.as_ptr(),
                use_sep_key.as_ptr(),
                FALSE as i32,
            ) != 0
            {
                TRUE
            } else {
                FALSE
            } as BOOL;
        }
        _ => {
            /* VER_PLATFORM_WIN32_NT */
            /* Try to load last selected layout */
            let calc_key: [u16; 24] = [
                'S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0,
            ];
            let layout_key: [u16; 7] = ['l' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 0];
            calc.layout = LoadRegInt(
                calc_key.as_ptr(),
                layout_key.as_ptr(),
                (CALC_LAYOUT_STANDARD as i32),
            ) as DWORD;

            /* Try to load last selected formatting option */
            let use_sep_key: [u16; 7] = ['U' as u16, 's' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'p' as u16, 0];
            calc.usesep = if LoadRegInt(
                calc_key.as_ptr(),
                use_sep_key.as_ptr(),
                FALSE as i32,
            ) != 0
            {
                TRUE
            } else {
                FALSE
            } as BOOL;
        }
    }

    /* memory is empty at startup */
    calc.is_memory = (FALSE) as i32;

    /* Get locale info for numbers */
    UpdateNumberIntl();
}

unsafe extern "C" fn save_config() {
    let mut buf: [TCHAR; 32] = [0; 32];
    let mut osvi: OSVERSIONINFO = core::mem::zeroed();

    osvi.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFO>() as DWORD;
    GetVersionEx(core::ptr::addr_of_mut!(osvi));

    match osvi.dwPlatformId {
        VER_PLATFORM_WIN32s | VER_PLATFORM_WIN32_WINDOWS => {
            StringCbPrintf!(
                core::ptr::addr_of_mut!(buf) as *mut TCHAR,
                core::mem::size_of::<[TCHAR; 32]>() as size_t,
                ['%' as u16, 'l' as u16, 'u' as u16, 0u16].as_ptr() as LPCTSTR,
                calc.layout,
            );
            WriteProfileString(
                ['S' as u16, 'c' as u16, 'i' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0u16].as_ptr() as LPCTSTR,
                ['l' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR,
                core::ptr::addr_of!(buf) as *const TCHAR as LPCTSTR,
            );
            WriteProfileString(
                ['S' as u16, 'c' as u16, 'i' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0u16].as_ptr() as LPCTSTR,
                ['U' as u16, 's' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'p' as u16, 0u16].as_ptr() as LPCTSTR,
                if (calc.usesep) as u32 == TRUE {
                    ['1' as u16, 0u16].as_ptr() as LPCTSTR
                } else {
                    ['0' as u16, 0u16].as_ptr() as LPCTSTR
                },
            );
            // break;
        }

        _ => {
            /* VER_PLATFORM_WIN32_NT */
            SaveRegInt(
                ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                 '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0u16].as_ptr() as LPCTSTR,
                ['l' as u16, 'a' as u16, 'y' as u16, 'o' as u16, 'u' as u16, 't' as u16, 0u16].as_ptr() as LPCTSTR,
                calc.layout as core::ffi::c_int,
            );
            SaveRegInt(
                ['S' as u16, 'O' as u16, 'F' as u16, 'T' as u16, 'W' as u16, 'A' as u16, 'R' as u16, 'E' as u16,
                 '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
                 'f' as u16, 't' as u16, '\\' as u16, 'C' as u16, 'a' as u16, 'l' as u16, 'c' as u16, 0u16].as_ptr() as LPCTSTR,
                ['U' as u16, 's' as u16, 'e' as u16, 'S' as u16, 'e' as u16, 'p' as u16, 0u16].as_ptr() as LPCTSTR,
                calc.usesep as core::ffi::c_int,
            );
            // break;
        }
    }
}

unsafe extern "C" fn post_key_press(mut lParam: LPARAM, mut idc: WORD) -> LRESULT {
    let mut hCtlWnd: HWND = GetDlgItem(calc.hWnd, idc as ::core::ffi::c_int);
    let mut ClassName: [TCHAR; 64] = [0; 64];

    /* check if the key is enabled! */
    if IsWindowEnabled(hCtlWnd) == 0 {
        return 1 as LRESULT;
    }

    if GetClassName(
        hCtlWnd,
        &raw mut ClassName as LPWSTR,
        SIZEOF!(ClassName) as ::core::ffi::c_int,
    ) == 0
    {
        return 1 as LRESULT;
    }

    if _tcscmp(&raw mut ClassName as *mut TCHAR, WC_BUTTON) == 0 {
        let mut dwStyle: DWORD = (GetWindowLongPtr(hCtlWnd, GWL_STYLE) & 0xF) as DWORD;

        /* Set states for press/release, but only for push buttons */
        if dwStyle == BS_PUSHBUTTON as DWORD
            || dwStyle == BS_DEFPUSHBUTTON as DWORD
            || dwStyle == BS_OWNERDRAW as DWORD
        {
            if lParam & KEY_WAS_DOWN as LPARAM == 0 {
                PostMessage(
                    hCtlWnd,
                    BM_SETSTATE as UINT,
                    1 as WPARAM,
                    0 as LPARAM,
                );
            } else if lParam & KEY_IS_UP as LPARAM != 0 {
                PostMessage(
                    hCtlWnd,
                    BM_SETSTATE as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
                PostMessage(
                    hCtlWnd,
                    BM_CLICK as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
            return 1 as LRESULT;
        }
    }
    /* default action: simple click event at key release */
    if lParam & KEY_IS_UP as LPARAM != 0 {
        PostMessage(
            hCtlWnd,
            BM_CLICK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
    return 1 as LRESULT;
}

unsafe extern "C" fn vk2ascii(mut vk: u32) -> i32 {
    let mut s: u16;
    let mut scan: i32;
    let mut state: [BYTE; 256] = [0; 256];
    let mut layout: HKL = GetKeyboardLayout(0 as DWORD);

    if GetKeyboardState(&raw mut state as PBYTE) == 0 {
        return 0 as i32;
    }

    scan = MapVirtualKeyEx(vk as UINT, 0 as UINT, layout) as i32;
    s = 0 as u16;
    if ToAsciiEx(
        vk as UINT,
        scan as UINT,
        &raw mut state as *mut BYTE,
        &raw mut s,
        0 as UINT,
        layout,
    ) > 0 as i32
    {
        /* convert to upper case */
        if s >= 'a' as u16 && s <= 'z' as u16 {
            s = (s as i32 - 'a' as i32 + 'A' as i32) as u16;
        }
        /* add check to CTRL key */
        if vk >= 'A' as u32
            && vk <= 'Z' as u32
            && s as i32 >= CTRL_A as i32
            && s as i32 <= CTRL_Z as i32
        {
            s = (s as i32 | CTRL_FLAG as i32) as u16;
        } else if (GetAsyncKeyState(VK_MENU as i32) as i32) < 0 as i32 {
            s = (s as i32 | ALT_FLAG as i32) as u16;
        }
        return s as i32;
    }
    return 0 as i32;
}

unsafe extern "C" fn process_vk_key(mut wParam: WPARAM, mut lParam: LPARAM) -> ::core::ffi::c_int {
    let mut k: *const key2code_t = ::core::ptr::null::<key2code_t>();
    let mut x: ::core::ffi::c_uint = 0;
    let mut ch: ::core::ffi::c_ushort = 0;
    ch = vk2ascii(LOWORD!(wParam) as ::core::ffi::c_uint) as ::core::ffi::c_ushort;
    if (lParam & (KEY_IS_UP as LPARAM)) != 0 {
        /* Test for "copy" to clipboard */
        if ch as ::core::ffi::c_int
            == (((CTRL_C as ::core::ffi::c_int) | (CTRL_FLAG as ::core::ffi::c_int)))
        {
            SendMessage(
                calc.hWnd,
                (WM_COMMAND) as UINT,
                (IDM_EDIT_COPY) as WPARAM,
                (0) as LPARAM,
            );
            return (1) as ::core::ffi::c_int;
        }
        /* Test for "paste" from clipboard */
        if ch as ::core::ffi::c_int
            == (((CTRL_V as ::core::ffi::c_int) | (CTRL_FLAG as ::core::ffi::c_int)))
        {
            SendMessage(
                calc.hWnd,
                (WM_COMMAND) as UINT,
                (IDM_EDIT_PASTE) as WPARAM,
                (0) as LPARAM,
            );
            return (1) as ::core::ffi::c_int;
        }
        /* Test of help menu */
        if (LOWORD!(wParam) as ::core::ffi::c_int) as u32 == (VK_F1) {
            SendMessage(
                calc.hWnd,
                (WM_COMMAND) as UINT,
                (IDM_HELP_HELP) as WPARAM,
                (0) as LPARAM,
            );
            return (1) as ::core::ffi::c_int;
        }
    }

    x = (0) as ::core::ffi::c_uint;
    while (x as usize)
        < (::core::mem::size_of::<[key3code_t; 68]>() as usize)
            .wrapping_div(::core::mem::size_of::<key3code_t>() as usize)
    {
        'body: {
            let mut key: ::core::ffi::c_int = key2code[x as usize].key as ::core::ffi::c_int;
            if (key2code[x as usize].mask as ::core::ffi::c_int & ((BITMASK_IS_CTRL)) as ::core::ffi::c_int) != 0 {
                key |= (CTRL_FLAG) as ::core::ffi::c_int;
            }
            if (key == ch as ::core::ffi::c_int
                && (key2code[x as usize].mask as ::core::ffi::c_int & ((BITMASK_IS_ASCII)) as ::core::ffi::c_int) != 0)
                || (key == LOWORD!(wParam) as ::core::ffi::c_int
                    && (key2code[x as usize].mask as ::core::ffi::c_int & ((BITMASK_IS_ASCII)) as ::core::ffi::c_int) == 0)
            {
                if GetDlgItem(
                    calc.hWnd,
                    key2code[x as usize].idc as ::core::ffi::c_int,
                )
                .is_null()
                {
                    break 'body;
                }
                return post_key_press(lParam, key2code[x as usize].idc) as ::core::ffi::c_int;
            }
        }
        x = x.wrapping_add(1);
    }

    if calc.layout == (CALC_LAYOUT_SCIENTIFIC) as DWORD {
        if calc.base == (IDC_RADIO_DEC) as DWORD {
            k = &raw const key2code_base10 as *const key2code_t;
            x = (::core::mem::size_of::<[key2code_t; 3]>() as usize)
                .wrapping_div(::core::mem::size_of::<key2code_t>() as usize)
                as ::core::ffi::c_uint;
        } else {
            k = &raw const key2code_base16 as *const key2code_t;
            x = (::core::mem::size_of::<[key2code_t; 4]>() as usize)
                .wrapping_div(::core::mem::size_of::<key2code_t>() as usize)
                as ::core::ffi::c_uint;
        }
        loop {
            if (*k).key as ::core::ffi::c_int == LOWORD!(wParam) as ::core::ffi::c_int {
                return post_key_press(lParam, (*k).idc) as ::core::ffi::c_int;
            }
            k = k.offset(1);
            x = x.wrapping_sub(1);
            if !(x != 0) {
                break;
            }
        }
    }
    return (0) as ::core::ffi::c_int;
}

unsafe extern "C" fn update_lcd_display(hwnd: HWND) {
    /*
     * multiply size of calc.buffer by 2 because it may
     * happen that separator is used between each digit.
     * Also added little additional space for dot and '\0'.
     */
    let mut tmp: [TCHAR; (MAX_CALC_SIZE * 2 + 2) as _] =
        [0; (MAX_CALC_SIZE * 2 + 2) as _];
    let tmp_ptr: *mut TCHAR = core::ptr::addr_of_mut!(tmp) as *mut TCHAR;

    if calc.buffer[0] == ('\0' as u16) {
        StringCbCopy(
            tmp_ptr,
            core::mem::size_of::<[TCHAR; (MAX_CALC_SIZE * 2 + 2) as _]>(),
            (&[0x0030, 0x0000u16]).as_ptr() /* _T "0" */,
        );
    } else {
        StringCbCopy(
            tmp_ptr,
            core::mem::size_of::<[TCHAR; (MAX_CALC_SIZE * 2 + 2) as _]>(),
            core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR,
        );
    }

    /* Add final '.' in decimal mode (if it's missing), but
     * only if it's a result: no append if it prints "ERROR".
     */
    if calc.base == IDC_RADIO_DEC && calc.is_nan == 0 {
        if _tcschr(tmp_ptr, ('.' as u16)).is_null() {
            StringCbCat(
                tmp_ptr,
                core::mem::size_of::<[TCHAR; (MAX_CALC_SIZE * 2 + 2) as _]>(),
                (&[0x002e, 0x0000u16]).as_ptr() /* _T "." */,
            );
        }
    }
    /* if separator mode is on, let's add an additional space */
    if calc.usesep != 0 && calc.sci_in == 0 && calc.sci_out == 0 && calc.is_nan == 0 {
        /* go to the integer part of the string */
        let mut p: *mut TCHAR = _tcschr(tmp_ptr, ('.' as u16)) as *mut TCHAR;
        let mut e: *mut TCHAR = _tcschr(tmp_ptr, ('\0' as u16)) as *mut TCHAR;
        let mut n: i32 = 0;
        let mut t: i32;

        if p.is_null() {
            p = e;
        }
        match calc.base {
            IDC_RADIO_HEX | IDC_RADIO_BIN => {
                t = 4;
            }
            _ => {
                /* fall here for:
                     IDC_RADIO_DEC:
                     IDC_RADIO_OCT: */
                t = 3;
            }
        }
        loop {
            p = p.offset(-1);
            if !(p > tmp_ptr) {
                break;
            }
            n += 1;
            if n == t && *p.offset(-(1)) != ('-' as u16) {
                memmove(
                    p.offset(1) as *mut core::ffi::c_void,
                    p as *const core::ffi::c_void,
                    ((e.offset_from(p) + 1) as usize)
                        * core::mem::size_of::<TCHAR>(),
                );
                e = e.offset(1);
                *p = (' ' as u16);
                n = 0;
            }
        }
        /* if decimal mode, apply regional settings */
        if calc.base == IDC_RADIO_DEC {
            let mut p: *mut TCHAR = tmp_ptr;
            let mut e: *mut TCHAR = _tcschr(tmp_ptr, ('.' as u16)) as *mut TCHAR;

            /* searching for thousands default separator */
            while p < e {
                if *p == (' ' as u16) {
                    memmove(
                        p.offset(calc.sThousand_len as isize) as *mut core::ffi::c_void,
                        p.offset(1) as *const core::ffi::c_void,
                        (_tcslen(p) as usize)
                            * core::mem::size_of::<TCHAR>(),
                    );
                    memcpy(
                        p as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!(calc.sThousand) as *mut TCHAR as *const core::ffi::c_void,
                        (calc.sThousand_len as usize)
                            * core::mem::size_of::<TCHAR>(),
                    );
                    p = p.offset(calc.sThousand_len as isize);
                } else {
                    p = p.offset(1);
                }
            }
            /* update decimal point too. */
            memmove(
                p.offset(calc.sDecimal_len as isize) as *mut core::ffi::c_void,
                p.offset(1) as *const core::ffi::c_void,
                (_tcslen(p) as usize)
                    * core::mem::size_of::<TCHAR>(),
            );
            memcpy(
                p as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(calc.sDecimal) as *mut TCHAR as *const core::ffi::c_void,
                (calc.sDecimal_len as usize)
                    * core::mem::size_of::<TCHAR>(),
            );
        }
    } else {
        let mut p: *mut TCHAR = _tcschr(tmp_ptr, ('.' as u16)) as *mut TCHAR;

        /* update decimal point when usesep is false */
        if !p.is_null() {
            memmove(
                p.offset(calc.sDecimal_len as isize) as *mut core::ffi::c_void,
                p.offset(1) as *const core::ffi::c_void,
                (_tcslen(p) as usize)
                    * core::mem::size_of::<TCHAR>(),
            );
            memcpy(
                p as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(calc.sDecimal) as *mut TCHAR as *const core::ffi::c_void,
                (calc.sDecimal_len as usize)
                    * core::mem::size_of::<TCHAR>(),
            );
        }
    }
    SetDlgItemText(hwnd, (IDC_TEXT_OUTPUT as i32), tmp_ptr);
}

unsafe extern "C" fn update_parent_display(hWnd: HWND) {
    let mut str: [TCHAR; 8] = [0; 8];
    let mut n: ::core::ffi::c_int = eval_parent_count();

    if n == 0 {
        str[0] = ('\0' as u16);
    } else {
        let fmt: [u16; 5] = ['(' as u16, '=' as u16, '%' as u16, 'd' as u16, 0];
        StringCbPrintf!(
            core::ptr::addr_of_mut!(str) as *mut TCHAR as STRSAFE_LPWSTR,
            core::mem::size_of::<[TCHAR; 8]>() as size_t,
            core::ptr::addr_of!(fmt) as *const u16 as STRSAFE_LPCWSTR,
            n,
        );
    }
    SetDlgItemText(
        hWnd,
        (IDC_TEXT_PARENT as i32),
        core::ptr::addr_of_mut!(str) as *mut TCHAR as LPCWSTR,
    );
}

unsafe extern "C" fn build_operand(hwnd: HWND, idc: DWORD) {
    let mut i: ::core::ffi::c_uint = 0;
    let mut n: ::core::ffi::c_uint;
    let mut cbPtr: size_t;

    if idc == IDC_BUTTON_DOT {
        /* if dot is the first char, it's added automatically */
        if (&raw mut calc.buffer as *mut TCHAR) == calc.ptr {
            let fresh21 = calc.ptr;
            calc.ptr = calc.ptr.offset(1);
            *fresh21 = ('0' as u16);
            let fresh22 = calc.ptr;
            calc.ptr = calc.ptr.offset(1);
            *fresh22 = ('.' as u16);
            *calc.ptr = ('\0' as u16);
            update_lcd_display(hwnd);
            return;
        }
        /* if pressed dot and it's already in the string, then return */
        if !_tcschr(&raw mut calc.buffer as *mut TCHAR, ('.' as u16)).is_null() {
            return;
        }
    }
    if idc != (IDC_STATIC) as u32 {
        while idc != key2code[i as usize].idc as DWORD {
            i = i.wrapping_add(1);
        }
    }
    n = calc.ptr.offset_from(&raw mut calc.buffer as *mut TCHAR) as ::core::ffi::c_longlong
        as ::core::ffi::c_uint;
    if idc == IDC_BUTTON_0 && n == 0 {
        /* no need to put the dot because it's handled by update_lcd_display() */
        calc.buffer[0 as usize] = ('0' as u16);
        calc.buffer[1 as usize] = ('\0' as u16);
        update_lcd_display(hwnd);
        return;
    }
    match calc.base {
        IDC_RADIO_HEX => {
            if n >= 16 {
                return;
            }
        }
        IDC_RADIO_DEC => {
            if n as usize >= SIZEOF!(calc.buffer) - 1 {
                return;
            }
            if calc.sci_in != 0 {
                if idc != (IDC_STATIC) as u32 {
                    calc.esp = ((calc.esp * 10
                        + (key2code[i as usize].key as ::core::ffi::c_int - '0' as ::core::ffi::c_int))
                        as ::core::ffi::c_long
                        % LOCAL_EXP_SIZE as ::core::ffi::c_long)
                        as ::core::ffi::c_int;
                }
                if calc.ptr == (&raw mut calc.buffer as *mut TCHAR) {
                    StringCbPrintf!(
                        calc.ptr,
                        ::core::mem::size_of::<[TCHAR; 256]>() as size_t,
                        (&[0x0030, 0x002e, 0x0065, 0x0025, 0x002b, 0x0064, 0x0000u16]).as_ptr() /* _T "0.e%+d" */,
                        calc.esp,
                    );
                } else {
                    /* adds the dot at the end if the number has no decimal part */
                    if _tcschr(&raw mut calc.buffer as *mut TCHAR, ('.' as u16)).is_null() {
                        let fresh23 = calc.ptr;
                        calc.ptr = calc.ptr.offset(1);
                        *fresh23 = ('.' as u16);
                    }

                    cbPtr = (::core::mem::size_of::<[TCHAR; 256]>() as usize).wrapping_sub(
                        (calc.ptr as *mut BYTE)
                            .offset_from(&raw mut calc.buffer as *mut TCHAR as *mut BYTE)
                            as ::core::ffi::c_longlong as usize,
                    ) as size_t;
                    StringCbPrintf!(calc.ptr, cbPtr, (&[0x0065, 0x0025, 0x002b, 0x0064, 0x0000u16]).as_ptr() /* _T "e%+d" */, calc.esp);
                }
                update_lcd_display(hwnd);
                return;
            }
        }
        IDC_RADIO_OCT => {
            if n >= 22 {
                return;
            }
        }
        IDC_RADIO_BIN => {
            if n >= 64 {
                return;
            }
        }
        _ => {}
    }

    cbPtr = (::core::mem::size_of::<[TCHAR; 256]>() as usize).wrapping_sub(
        (calc.ptr as *mut BYTE).offset_from(&raw mut calc.buffer as *mut TCHAR as *mut BYTE)
            as ::core::ffi::c_longlong as usize,
    ) as size_t;
    StringCbPrintfEx!(
        calc.ptr,
        cbPtr,
        &raw mut calc.ptr,
        core::ptr::null_mut::<size_t>(),
        STRSAFE_FILL_ON_FAILURE as STRSAFE_DWORD,
        (&[0x0025, 0x0043, 0x0000u16]).as_ptr() /* _T "%C" */,
        key2code[i as usize].key as ::core::ffi::c_int,
    );

    update_lcd_display(hwnd);
}

unsafe extern "C" fn prepare_rpn_result(
    mut rpn: *mut calc_number_t,
    mut buffer: *mut TCHAR,
    mut size: ::core::ffi::c_int,
    mut base: ::core::ffi::c_int,
) {
    if calc.is_nan != 0 {
        rpn_zero(core::ptr::addr_of_mut!(calc.code));
        LoadString(calc.hInstance, IDS_MATH_ERROR as UINT, buffer as LPWSTR, size);
        return;
    }
    prepare_rpn_result_2(rpn, buffer, size, base);
}

unsafe extern "C" fn set_rpn_result(hwnd: HWND, rpn: *mut calc_number_t) {
    calc.sci_in = (FALSE) as i32;
    prepare_rpn_result(rpn, &raw mut calc.buffer as *mut TCHAR, SIZEOF!(calc.buffer) as i32, calc.base as i32);
    calc.ptr = (&raw mut calc.buffer as *mut TCHAR).offset(_tcslen(&raw mut calc.buffer as *mut TCHAR) as isize);
    update_lcd_display(hwnd);
    update_parent_display(hwnd);
}

unsafe extern "C" fn display_rpn_result(hwnd: HWND, rpn: *mut calc_number_t) {
    set_rpn_result(hwnd, rpn);
    calc.ptr = core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR;
}

unsafe extern "C" fn get_modifiers(hWnd: HWND) -> i32 {
    let mut modifiers: i32 = 0;
    if IsDlgButtonChecked(hWnd, (IDC_CHECK_INV as i32)) == BST_CHECKED {
        modifiers |= (MODIFIER_INV as i32);
    }
    if IsDlgButtonChecked(hWnd, (IDC_CHECK_HYP as i32)) == BST_CHECKED {
        modifiers |= (MODIFIER_HYP as i32);
    }

    return modifiers;
}

unsafe extern "C" fn convert_text2number(a: *mut calc_number_t) {
    /* if the screen output buffer is empty, then */
    /* the operand is taken from the last input */
    if (core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR) == calc.ptr {
        /* if pushed valued is ZERO then we should grab it */
        let s_dot: [u16; 3] = ['0' as u16, '.' as u16, 0];
        let s_zero: [u16; 2] = ['0' as u16, 0];
        if _tcscmp(core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR, s_dot.as_ptr()) == 0
            || _tcscmp(core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR, s_zero.as_ptr()) == 0
        {
            /* this zero is good for both integer and decimal */
            rpn_zero(a);
        } else {
            rpn_copy(a, core::ptr::addr_of_mut!(calc.code));
        }
        return;
    }
    /* ZERO is the default value for all numeric bases */
    rpn_zero(a);
    convert_text2number_2(a);
}

unsafe extern "C" fn update_menu(hWnd: HWND) {
    let hMenu: HMENU = GetSubMenu(GetMenu(hWnd), 1);
    let mut x: ::core::ffi::c_uint = 0;
    x = 0;
    while (x as usize) < SIZEOF!(upd) {
        if *(upd[x as usize].sel) != upd[x as usize].idc as DWORD {
            CheckMenuItem(
                hMenu,
                upd[x as usize].idm as UINT,
                (MF_BYCOMMAND | MF_UNCHECKED) as UINT,
            );
            CheckDlgButton(
                hWnd,
                upd[x as usize].idc as ::core::ffi::c_int,
                BST_UNCHECKED as UINT,
            );
        } else {
            CheckMenuItem(
                hMenu,
                upd[x as usize].idm as UINT,
                (MF_BYCOMMAND | MF_CHECKED) as UINT,
            );
            CheckDlgButton(
                hWnd,
                upd[x as usize].idc as ::core::ffi::c_int,
                BST_CHECKED as UINT,
            );
        }
        x = x.wrapping_add(1);
    }
    CheckMenuItem(
        hMenu,
        IDM_VIEW_GROUP as UINT,
        (MF_BYCOMMAND
            | (if calc.usesep != 0 {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            })) as UINT,
    );
}

unsafe extern "C" fn enable_allowed_controls(hwnd: HWND, base: DWORD) {
    let mut mask: BYTE = 0;
    let mut n: ::core::ffi::c_int;
    match base {
        IDC_RADIO_DEC => {
            mask = BITMASK_DEC_MASK as BYTE;
        }
        IDC_RADIO_HEX => {
            mask = BITMASK_HEX_MASK as BYTE;
        }
        IDC_RADIO_OCT => {
            mask = BITMASK_OCT_MASK as BYTE;
        }
        IDC_RADIO_BIN => {
            mask = BITMASK_BIN_MASK as BYTE;
        }
        _ => return,
    }
    n = 0 as ::core::ffi::c_int;
    while (n as usize) < (SIZEOF!(key2code) as usize) {
        if (key2code[n as usize].mask as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
            let hCtlWnd: HWND =
                GetDlgItem(hwnd, key2code[n as usize].idc as ::core::ffi::c_int);
            let mut current: BOOL;

            if (key2code[n as usize].mask as ::core::ffi::c_int & BITMASK_IS_STATS as ::core::ffi::c_int) != 0 {
                current = if IsWindow(calc.hStatWnd) != 0 {
                    TRUE
                } else {
                    FALSE
                } as BOOL;
            } else {
                current = if (key2code[n as usize].mask as ::core::ffi::c_int & mask as ::core::ffi::c_int) != 0 {
                    TRUE
                } else {
                    FALSE
                } as BOOL;
            }
            if IsWindowEnabled(hCtlWnd) != current {
                EnableWindow(hCtlWnd, current);
            }
        }
        n += 1;
    }
}

unsafe extern "C" fn update_radio(mut hwnd: HWND, mut base: ::core::ffi::c_uint) {
    let mut hMenu: HMENU = ::core::ptr::null_mut::<HMENU__>();
    let mut lpMenuId: LPCTSTR = ::core::ptr::null::<WCHAR>();
    let mut mask: WORD = 0;
    let mut n: ::core::ffi::c_int = 0;
    match base {
        IDC_RADIO_DEC => {
            lpMenuId = (IDR_MENU_SCIENTIFIC_1 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCTSTR;
            mask = BITMASK_DEC_MASK as WORD;
        }
        IDC_RADIO_HEX => {
            lpMenuId = (IDR_MENU_SCIENTIFIC_2 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCTSTR;
            mask = BITMASK_HEX_MASK as WORD;
        }
        IDC_RADIO_OCT => {
            lpMenuId = (IDR_MENU_SCIENTIFIC_2 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCTSTR;
            mask = BITMASK_OCT_MASK as WORD;
        }
        IDC_RADIO_BIN => {
            lpMenuId = (IDR_MENU_SCIENTIFIC_2 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as LPCTSTR;
            mask = BITMASK_BIN_MASK as WORD;
        }
        _ => return,
    }
    if calc.base != base as DWORD {
        convert_text2number(::core::ptr::addr_of_mut!(calc.code));
        convert_real_integer(base);
        calc.base = base as DWORD;
        display_rpn_result(hwnd, ::core::ptr::addr_of_mut!(calc.code));

        hMenu = GetMenu(hwnd);
        DestroyMenu(hMenu);
        hMenu = LoadMenu(calc.hInstance, lpMenuId);
        SetMenu(hwnd, hMenu);
        update_menu(hwnd);

        n = 0 as ::core::ffi::c_int;
        while (n as usize)
            < (::core::mem::size_of::<[radio_config_t; 7]>() as usize)
                .wrapping_div(::core::mem::size_of::<radio_config_t>() as usize)
        {
            ShowWindow(
                GetDlgItem(hwnd, radio_setup[n as usize].idc as ::core::ffi::c_int),
                if radio_setup[n as usize].mask as ::core::ffi::c_int & mask as ::core::ffi::c_int
                    != 0
                {
                    (SW_SHOW) as i32
                } else {
                    (SW_HIDE) as i32
                },
            );
            n += 1;
        }

        enable_allowed_controls(hwnd, base as DWORD);
    }

    CheckRadioButton(
        hwnd,
        (IDC_RADIO_HEX as i32),
        (IDC_RADIO_BIN as i32),
        calc.base as ::core::ffi::c_int,
    );

    if base == IDC_RADIO_DEC as ::core::ffi::c_uint {
        CheckRadioButton(
            hwnd,
            (IDC_RADIO_DEG as i32),
            (IDC_RADIO_GRAD as i32),
            calc.degr as ::core::ffi::c_int,
        );
    } else {
        CheckRadioButton(
            hwnd,
            (IDC_RADIO_QWORD as i32),
            (IDC_RADIO_BYTE as i32),
            calc.size as ::core::ffi::c_int,
        );
    };
}

unsafe extern "C" fn update_memory_flag(mut hWnd: HWND, mut mem_flag: BOOL) {
    calc.is_memory = mem_flag;
    SetDlgItemText(
        hWnd,
        (IDC_TEXT_MEMORY as i32),
        if mem_flag != 0 {
            (&[0x004d, 0x0000u16]).as_ptr() /* _T "M" */
        } else {
            (&[0x0000u16]).as_ptr() /* _T "" */
        },
    );
}

unsafe extern "C" fn update_n_stats_items(hWnd: HWND, buffer: *mut TCHAR, cbBuffer: size_t) {
    let n: ::core::ffi::c_uint = SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_GETCOUNT, 0, 0) as ::core::ffi::c_uint;

    let fmt: [u16; 5] = ['n' as u16, '=' as u16, '%' as u16, 'u' as u16, 0];
    StringCbPrintf!(buffer, cbBuffer, fmt.as_ptr() as *const TCHAR, n);
    SetDlgItemText(hWnd, (IDC_TEXT_NITEMS as i32), buffer);
}

unsafe extern "C" fn clean_stat_list() {
    let mut p: *mut statistic_t = calc.stat;
    while !p.is_null() {
        let mut s: *mut statistic_t = p;
        p = (*p).next as *mut statistic_t;
        rpn_free(core::ptr::addr_of_mut!((*s).num));
        free(s as *mut core::ffi::c_void);
    }
    calc.stat = p;
}

unsafe extern "C" fn delete_stat_item(mut n: ::core::ffi::c_int) {
    let mut p: *mut statistic_t = calc.stat;
    let mut s: *mut statistic_t = ::core::ptr::null_mut::<statistic_t>();
    if n == 0 as ::core::ffi::c_int {
        calc.stat = (*p).next as *mut statistic_t;
        rpn_free(&raw mut (*p).num);
        free(p as *mut ::core::ffi::c_void);
    } else {
        s = (*p).next as *mut statistic_t;
        loop {
            n -= 1;
            if !(n != 0) {
                break;
            }
            p = s;
            s = (*p).next as *mut statistic_t;
        }
        (*p).next = (*s).next;
        rpn_free(&raw mut (*s).num);
        free(s as *mut ::core::ffi::c_void);
    };
}

unsafe extern "C" fn ReadConversion(formula: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let len: size_t = strlen(formula);
    let str: *mut ::core::ffi::c_char =
        malloc(len.wrapping_add(3 as size_t)) as *mut ::core::ffi::c_char;

    if str.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }

    *str.offset(0 as isize) = '(' as i32 as ::core::ffi::c_char;
    memcpy(
        str.offset(1 as isize) as *mut ::core::ffi::c_void,
        formula as *const ::core::ffi::c_void,
        len,
    );
    *str.offset(len.wrapping_add(1 as size_t) as isize) = ')' as i32 as ::core::ffi::c_char;
    *str.offset(len.wrapping_add(2 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;

    let zero_str: [u16; 2] = ['0' as u16, 0];
    StringCbCopy(
        ::core::ptr::addr_of_mut!(calc.source) as STRSAFE_LPWSTR,
        ::core::mem::size_of::<[TCHAR; 256]>() as size_t,
        if *(::core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR) as ::core::ffi::c_int
            == '\0' as i32 as ::core::ffi::c_int
        {
            zero_str.as_ptr() as STRSAFE_LPCWSTR
        } else {
            ::core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR as STRSAFE_LPCWSTR
        },
    );

    /* clear display content before proceeding */
    calc.ptr = ::core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR;
    calc.buffer[0 as usize] = '\0' as TCHAR;

    return str;
}

unsafe extern "system" fn DlgStatProc(hWnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    let mut buffer: [TCHAR; 256] = [0; 256];
    let mut n: DWORD = 0;
    if msg == WM_INITDIALOG {
        return TRUE as INT_PTR;
    } else if msg == WM_COMMAND {
        if LOWORD!(wp) == (IDC_LIST_STAT as u16) {
            if (HIWORD!(wp)) as u32 == CBN_DBLCLK {
                SendMessage(hWnd, WM_COMMAND, IDC_BUTTON_LOAD as WPARAM, 0);
            }
            return TRUE as INT_PTR;
        } else if LOWORD!(wp) == (IDC_BUTTON_RET as u16) {
            SetFocus(GetDlgItem(GetParent(hWnd), (IDC_BUTTON_FOCUS as i32)));
            return TRUE as INT_PTR;
        } else if LOWORD!(wp) == (IDC_BUTTON_LOAD as u16) {
            n = SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_GETCURSEL, 0, 0) as DWORD;
            if n == (LB_ERR) as u32 {
                return TRUE as INT_PTR;
            }
            PostMessage(GetParent(hWnd), WM_LOAD_STAT, n as WPARAM, 0);
            return TRUE as INT_PTR;
        } else if LOWORD!(wp) == (IDC_BUTTON_CD as u16) {
            n = SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_GETCURSEL, 0, 0) as DWORD;
            if n == (LB_ERR) as u32 {
                return TRUE as INT_PTR;
            }
            SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_DELETESTRING, n as WPARAM, 0);
            update_n_stats_items(hWnd, core::ptr::addr_of_mut!(buffer) as *mut TCHAR, core::mem::size_of::<[TCHAR; 256]>() as size_t);
            delete_stat_item((n) as i32);
            return TRUE as INT_PTR;
        } else if LOWORD!(wp) == (IDC_BUTTON_CAD as u16) {
            SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_RESETCONTENT, 0, 0);
            clean_stat_list();
            update_n_stats_items(hWnd, core::ptr::addr_of_mut!(buffer) as *mut TCHAR, core::mem::size_of::<[TCHAR; 256]>() as size_t);
            return TRUE as INT_PTR;
        }
    } else if msg == WM_CLOSE {
        DestroyWindow(hWnd);
        return TRUE as INT_PTR;
    } else if msg == WM_DESTROY {
        clean_stat_list();
        PostMessage(GetParent(hWnd), WM_CLOSE_STATS, 0, 0);
        return TRUE as INT_PTR;
    } else if msg == WM_INSERT_STAT {
        prepare_rpn_result(
            core::ptr::addr_of_mut!((*(lp as *mut statistic_t)).num),
            core::ptr::addr_of_mut!(buffer) as *mut TCHAR,
            (SIZEOF!(buffer)) as i32,
            ((*(lp as *mut statistic_t)).base) as i32,
        );
        SendDlgItemMessage(hWnd, (IDC_LIST_STAT as i32), LB_ADDSTRING, 0, core::ptr::addr_of_mut!(buffer) as *mut TCHAR as LPARAM);
        update_n_stats_items(hWnd, core::ptr::addr_of_mut!(buffer) as *mut TCHAR, core::mem::size_of::<[TCHAR; 256]>() as size_t);
        return TRUE as INT_PTR;
    }
    return FALSE as INT_PTR;
}

unsafe extern "C" fn idm_2_idc(mut idm: ::core::ffi::c_int) -> WPARAM {
    let mut x: ::core::ffi::c_int = 0;
    x = 0 as ::core::ffi::c_int;
    while (x as usize) < SIZEOF!(upd) as usize {
        if upd[x as usize].idm as ::core::ffi::c_int == idm {
            break;
        }
        x += 1;
    }
    return upd[x as usize].idc as WPARAM;
}

unsafe extern "C" fn CopyMemToClipboard(ptr: *mut ::core::ffi::c_void) {
    if OpenClipboard(core::ptr::null_mut()) != 0 {
        let mut clipbuffer: HGLOBAL = core::ptr::null_mut();
        let mut buffer: *mut TCHAR = core::ptr::null_mut();
        let mut cbBuffer: size_t = 0;

        EmptyClipboard();
        cbBuffer = ((_tcslen(ptr as *const TCHAR) as usize)
            .wrapping_add(1 as usize))
            .wrapping_mul(core::mem::size_of::<TCHAR>()) as size_t;
        clipbuffer = GlobalAlloc(GMEM_DDESHARE as UINT, cbBuffer as SIZE_T);
        buffer = GlobalLock(clipbuffer) as *mut TCHAR;
        StringCbCopy(buffer, cbBuffer, ptr as *const TCHAR);
        GlobalUnlock(clipbuffer);

        // `#ifdef UNICODE` -> CF_UNICODETEXT only. Registering the same HGLOBAL as CF_TEXT too
        // handed ANSI readers UTF-16 (calc's own paste read one digit) and gave the clipboard
        // one block under two formats to free.
        SetClipboardData(CF_UNICODETEXT as UINT, clipbuffer as HANDLE);

        CloseClipboard();
    }
}

unsafe extern "C" fn handle_copy_command(mut hWnd: HWND) {
    let mut display: [TCHAR; (MAX_CALC_SIZE) as _] = [0; (MAX_CALC_SIZE) as _];
    let mut n: UINT = 0;

    // Read current text from output display
    n = GetDlgItemText(
        hWnd,
        (IDC_TEXT_OUTPUT as i32),
        core::ptr::addr_of_mut!(display) as *mut TCHAR,
        SIZEOF!(display) as ::core::ffi::c_int,
    );

    // Check if result is a true number
    if calc.is_nan == 0 {
        // Remove trailing decimal point if no decimal digits exist
        if calc.base == IDC_RADIO_DEC as DWORD
            && _tcschr(core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR, ('.' as u16))
                .is_null()
        {
            display[(n as ::core::ffi::c_uint).wrapping_sub(calc.sDecimal_len) as usize] =
                ('\0' as u16);
        }
    }

    CopyMemToClipboard(core::ptr::addr_of_mut!(display) as *mut TCHAR as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn ReadClipboard() -> *mut ::core::ffi::c_char {
    let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if OpenClipboard((::core::ptr::null_mut::<HWND>()).cast()) != 0 {
        let mut hData: HANDLE = GetClipboardData(CF_TEXT as UINT);
        let mut fromClipboard: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !hData.is_null() {
            fromClipboard = GlobalLock(hData as HGLOBAL) as *mut ::core::ffi::c_char;
            if *fromClipboard.offset(0 as isize) != 0 {
                buffer = (_strupr((_strdup(fromClipboard)).cast())).cast();
            }
            GlobalUnlock(hData as HGLOBAL);
        }
        CloseClipboard();
    }
    return buffer;
}

unsafe extern "C" fn handle_sequence_input(hwnd: HWND, seq: *mut sequence_t) -> *mut ::core::ffi::c_char {
    let mut ptr: *mut ::core::ffi::c_char = (*seq).ptr;
    let mut ch: ::core::ffi::c_int = 0;
    let mut x: ::core::ffi::c_int = 0;
    let fresh17 = ptr;
    ptr = ptr.add(1);
    ch = *fresh17 as ::core::ffi::c_int;
    if ch == '\\' as ::core::ffi::c_int {
        PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_DAT) as WPARAM, 0);
    } else if ch == ':' as ::core::ffi::c_int {
        ch = *ptr as ::core::ffi::c_int;
        if ch != '\0' as ::core::ffi::c_int {
            ptr = ptr.add(1);
        }
        match ch {
            67 => {
                // 'C'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_MC) as WPARAM, 0);
            }
            69 => {
                // 'E'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_EXP) as WPARAM, 0);
            }
            77 => {
                // 'M'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_MS) as WPARAM, 0);
            }
            80 => {
                // 'P'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_MP) as WPARAM, 0);
            }
            81 => {
                // 'Q'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_CANC) as WPARAM, 0);
            }
            82 => {
                // 'R'
                PostMessage(hwnd, WM_COMMAND, (IDC_BUTTON_MR) as WPARAM, 0);
            }
            _ => {}
        }
    } else if ch == '$' as ::core::ffi::c_int {
        StringCbCopyEx(
            core::ptr::addr_of_mut!(calc.buffer) as STRSAFE_LPWSTR,
            core::mem::size_of::<[TCHAR; 256]>() as size_t,
            core::ptr::addr_of_mut!(calc.source) as *mut TCHAR as STRSAFE_LPCWSTR,
            core::ptr::addr_of_mut!(calc.ptr),
            core::ptr::null_mut::<size_t>(),
            STRSAFE_FILL_ON_FAILURE as STRSAFE_DWORD,
        );
    } else {
        x = 0 as ::core::ffi::c_int;
        while (x as usize) < (SIZEOF!(key2code) as usize) {
            if !(((key2code[x as usize].mask as ::core::ffi::c_int) & (BITMASK_IS_ASCII as ::core::ffi::c_int)) == 0
                || ((key2code[x as usize].mask as ::core::ffi::c_int) & (BITMASK_IS_CTRL as ::core::ffi::c_int)) != 0)
            {
                if key2code[x as usize].key as ::core::ffi::c_int == ch {
                    PostMessage(
                        hwnd,
                        WM_COMMAND,
                        (key2code[x as usize].idc) as WPARAM,
                        0,
                    );
                    break;
                }
            }
            x += 1;
        }
    }

    if *ptr as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
        (*seq).ptr = ptr;
        PostMessage(hwnd, (*seq).wm_msg, 0, 0);
    } else {
        free((*seq).data as *mut ::core::ffi::c_void);
        ptr = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*seq).ptr = ptr;
        (*seq).data = (*seq).ptr;
    }
    return ptr;
}

unsafe extern "C" fn run_dat_sta(a: *mut calc_number_t) {
    let s: *mut statistic_t = malloc(core::mem::size_of::<statistic_t>() as size_t) as *mut statistic_t;
    let mut p: *mut statistic_t = calc.stat;

    rpn_alloc(core::ptr::addr_of_mut!((*s).num));
    rpn_copy(core::ptr::addr_of_mut!((*s).num), a);
    (*s).base = calc.base;
    (*s).next = core::ptr::null_mut();
    if p.is_null() {
        calc.stat = s;
    } else {
        while !(*p).next.is_null() {
            p = (*p).next as *mut statistic_t;
        }
        (*p).next = s as *mut core::ffi::c_void;
    }
    PostMessage(
        calc.hStatWnd,
        WM_INSERT_STAT as UINT,
        0 as WPARAM,
        s as LPARAM,
    );
}

unsafe extern "C" fn run_mp(c: *mut calc_number_t) {
    let mut cn: calc_node_t = core::mem::zeroed();
    cn.number = *c;
    cn.base = calc.base;
    run_operator(
        core::ptr::addr_of_mut!(calc.memory),
        core::ptr::addr_of_mut!(calc.memory),
        core::ptr::addr_of_mut!(cn),
        RPN_OPERATOR_ADD,
    );
    update_memory_flag(calc.hWnd, (TRUE) as BOOL);
}

unsafe extern "C" fn run_mm(c: *mut calc_number_t) {
    let mut cn: calc_node_t = calc_node_t {
        number: calc_number_t { f: 0. },
        operation: 0,
        base: 0,
    };
    cn.number = *c;
    cn.base = calc.base;
    run_operator(
        core::ptr::addr_of_mut!(calc.memory),
        core::ptr::addr_of_mut!(calc.memory),
        core::ptr::addr_of_mut!(cn),
        RPN_OPERATOR_SUB,
    );
    update_memory_flag(calc.hWnd, (TRUE) as BOOL);
}

unsafe extern "C" fn run_ms(c: *mut calc_number_t) {
    rpn_copy(core::ptr::addr_of_mut!(calc.memory.number), c);
    calc.memory.base = calc.base;
    update_memory_flag(
        calc.hWnd,
        if rpn_is_zero(core::ptr::addr_of_mut!(calc.memory.number)) != 0 {
            (FALSE) as i32
        } else {
            (TRUE) as i32
        },
    );
}

unsafe extern "C" fn run_mw(c: *mut calc_number_t) {
    let mut tmp: calc_number_t = calc_number_t { f: 0. };
    rpn_copy(core::ptr::addr_of_mut!(tmp), core::ptr::addr_of_mut!(calc.memory.number));
    rpn_copy(core::ptr::addr_of_mut!(calc.memory.number), c);
    calc.memory.base = calc.base;
    if calc.is_memory != 0 {
        rpn_copy(c, core::ptr::addr_of_mut!(tmp));
    }
    update_memory_flag(
        calc.hWnd,
        if rpn_is_zero(core::ptr::addr_of_mut!(calc.memory.number)) != 0 {
            (FALSE) as i32
        } else {
            (TRUE) as i32
        },
    );
}

unsafe extern "C" fn upload_stat_number(mut n: ::core::ffi::c_int) -> *mut statistic_t {
    let mut p: *mut statistic_t = calc.stat;
    if p.is_null() {
        return p;
    }
    loop {
        let fresh20 = n;
        n = n.wrapping_sub(1);
        if !(fresh20 != 0) {
            break;
        }
        p = (*p).next as *mut statistic_t;
        if p.is_null() {
            return p;
        }
    }
    if calc.base != (*p).base {
        if calc.base == IDC_RADIO_DEC as DWORD {
            calc.code.f = (*p).num.i as ::core::ffi::c_double;
        } else {
            calc.code.i = (*p).num.f as ::core::ffi::c_longlong as INT64;
            apply_int_mask(core::ptr::addr_of_mut!(calc.code));
        }
    } else {
        rpn_copy(core::ptr::addr_of_mut!(calc.code), core::ptr::addr_of_mut!((*p).num));
    }
    calc.is_nan = FALSE as BOOL;
    return p;
}

unsafe extern "C" fn run_fe(number: *mut calc_number_t) {
    calc.sci_out = if (calc.sci_out) as u32 != FALSE { (FALSE) as i32 } else { (TRUE) as i32 };
}

unsafe extern "C" fn handle_context_menu(hWnd: HWND, wp: WPARAM, lp: LPARAM) {
    let mut text: [TCHAR; 64] = [0; 64];
    let mut hMenu: HMENU = CreatePopupMenu();
    let mut idm: BOOL;

    LoadString(calc.hInstance, IDS_QUICKHELP as UINT, text.as_mut_ptr(), SIZEOF!(text) as ::core::ffi::c_int);
    AppendMenu(hMenu, (MF_STRING | MF_ENABLED) as UINT, IDM_HELP_HELP as UINT_PTR, text.as_ptr() as LPCWSTR);
    idm = TrackPopupMenu(
        hMenu,
        (TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RETURNCMD | TPM_RIGHTBUTTON) as UINT,
        LOWORD!(lp) as ::core::ffi::c_int,
        HIWORD!(lp) as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        hWnd,
        core::ptr::null_mut(),
    );
    DestroyMenu(hMenu);

    if idm != 0 {
        let mut popup: HH_POPUP = core::mem::zeroed();

        memset(core::ptr::addr_of_mut!(popup) as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, core::mem::size_of::<HH_POPUP>() as size_t);
        popup.cbStruct = core::mem::size_of::<HH_POPUP>() as ::core::ffi::c_int;
        popup.clrForeground = 1 as COLORREF;
        popup.clrBackground = (-(1 as ::core::ffi::c_int)) as COLORREF;
        popup.pt.x = LOWORD!(lp) as LONG;
        popup.pt.y = HIWORD!(lp) as LONG;
        popup.rcMargins.top = -(1 as LONG);
        popup.rcMargins.bottom = -(1 as LONG);
        popup.rcMargins.left = -(1 as LONG);
        popup.rcMargins.right = -(1 as LONG);
        popup.idString = GetWindowLongPtr(wp as HWND, GWL_ID as ::core::ffi::c_int) as UINT;
        calc_HtmlHelp!(
            wp as HWND,
            HTMLHELP_PATH!("/popups.txt") as LPCWSTR,
            HH_DISPLAY_TEXT_POPUP as UINT,
            core::ptr::addr_of_mut!(popup) as DWORD_PTR,
        );
    }

    let _ = idm;
}

unsafe extern "C" fn run_canc(c: *mut calc_number_t) {
    flush_postfix();
    rpn_zero(c);

    /* clear also scientific display modes */
    calc.sci_out = (FALSE) as i32;
    calc.sci_in = (FALSE) as i32;

    /* clear state of inv and hyp flags */
    CheckDlgButton(calc.hWnd, (IDC_CHECK_INV as i32), BST_UNCHECKED);
    CheckDlgButton(calc.hWnd, (IDC_CHECK_HYP as i32), BST_UNCHECKED);
}

unsafe extern "C" fn run_rpar(c: *mut calc_number_t) {
    exec_closeparent(c);
}

unsafe extern "C" fn run_lpar(c: *mut calc_number_t) {
    exec_infix2postfix(c, RPN_OPERATOR_PARENT);
}

unsafe extern "system" fn SubclassButtonProc(hWnd: HWND, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let dis: LPDRAWITEMSTRUCT = lp as LPDRAWITEMSTRUCT;
    let mut dwText: UINT = 0;
    let mut text: [TCHAR; 64] = [0; 64];
    let mut dx: ::core::ffi::c_int = 0;
    let mut dy: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut size: SIZE = SIZE { cx: 0, cy: 0 };
    let mut pt: POINT = POINT { x: 0, y: 0 };

    if (*dis).CtlType == ODT_BUTTON {
        let mut hTheme: HTHEME = core::ptr::null_mut();
        let mut lpBtnInfo: LPBTNINFO = core::ptr::null_mut();

        if (calc_IsAppThemed.unwrap_unchecked())() != 0 && (calc_IsThemeActive.unwrap_unchecked())() != 0 {
            hTheme = (calc_OpenThemeData.unwrap_unchecked())(hWnd, RUSTCONV_WSTR_8D7C97326263.as_ptr());
        }

        if !hTheme.is_null() {
            let mut iState: ::core::ffi::c_int = 0;

            if ((*dis).itemState & ODS_DISABLED) != 0 {
                iState |= (PBS_DISABLED) as i32;
            }
            if ((*dis).itemState & ODS_SELECTED) != 0 {
                iState |= (PBS_PRESSED) as i32;
            }

            lpBtnInfo = GetWindowLongPtr((*dis).hwndItem, GWLP_USERDATA) as LPBTNINFO;
            if !lpBtnInfo.is_null() {
                if (*lpBtnInfo).bHover != 0 {
                    iState |= (PBS_HOT) as i32;
                }
            }

            if (calc_IsThemeBackgroundPartiallyTransparent.unwrap_unchecked())(hTheme, (BP_PUSHBUTTON) as i32, iState) != 0 {
                (calc_DrawThemeParentBackground.unwrap_unchecked())(
                    (*dis).hwndItem,
                    (*dis).hDC,
                    core::ptr::addr_of_mut!((*dis).rcItem),
                );
            }

            // Draw the frame around the control
            (calc_DrawThemeBackground.unwrap_unchecked())(
                hTheme,
                (*dis).hDC,
                (BP_PUSHBUTTON) as i32,
                iState,
                core::ptr::addr_of_mut!((*dis).rcItem),
                core::ptr::null_mut(),
            );

            (calc_CloseThemeData.unwrap_unchecked())(hTheme);
        } else {
            /* default state: unpushed */
            let mut dwStyle: DWORD = 0;

            if ((*dis).itemState & ODS_SELECTED) != 0 {
                dwStyle = DFCS_PUSHED;
            }

            DrawFrameControl(
                (*dis).hDC,
                core::ptr::addr_of_mut!((*dis).rcItem),
                DFC_BUTTON,
                DFCS_BUTTONPUSH | dwStyle,
            );
        }

        /* button text to write */
        len = GetWindowText((*dis).hwndItem, text.as_mut_ptr(), (SIZEOF!(text)) as i32);

        /*
         * little exception: 1/x has different color
         * in standard and scientific modes
         */
        if (calc.layout == CALC_LAYOUT_STANDARD || calc.layout == CALC_LAYOUT_CONVERSION)
            && IDC_BUTTON_RX == (*dis).CtlID
        {
            SetTextColor((*dis).hDC, CALC_CLR_BLUE);
        } else {
            dx = 0;
            while (dx as usize) < SIZEOF!(key2code) {
                if key2code[dx as usize].idc as UINT == (*dis).CtlID {
                    SetTextColor((*dis).hDC, key2code[dx as usize].col);
                    break;
                }
                dx += 1;
            }
        }

        /* No background, to avoid corruption of the texture */
        SetBkMode((*dis).hDC, (TRANSPARENT as i32));

        /* Default state: enabled */
        dwText = 0;
        if ((*dis).itemState & ODS_DISABLED) != 0 {
            dwText = DSS_DISABLED;
        }

        /* Draw the text in the button */
        GetTextExtentPoint32((*dis).hDC, text.as_mut_ptr(), len, core::ptr::addr_of_mut!(size));
        dx = (((*dis).rcItem.right - (*dis).rcItem.left) - size.cx) >> 1;
        dy = (((*dis).rcItem.bottom - (*dis).rcItem.top) - size.cy) >> 1;
        if ((*dis).itemState & ODS_SELECTED) != 0 {
            dx += 1;
            dy += 1;
        }
        pt.x = (*dis).rcItem.left + dx;
        pt.y = (*dis).rcItem.top + dy;
        DrawState(
            (*dis).hDC,
            core::ptr::null_mut(),
            None,
            text.as_mut_ptr() as LPARAM,
            0,
            pt.x,
            pt.y,
            size.cx,
            size.cy,
            DST_TEXT | dwText,
        );
    }
    return 1;
}

unsafe extern "system" fn HotButtonProc(hWnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> INT_PTR {
    let lpBtnInfo: LPBTNINFO = GetWindowLongPtr(hWnd, GWLP_USERDATA) as LPBTNINFO;
    let mut mouse_event: TRACKMOUSEEVENT = TRACKMOUSEEVENT {
        cbSize: 0,
        dwFlags: 0,
        hwndTrack: core::ptr::null_mut::<HWND__>(),
        dwHoverTime: 0,
    };

    match msg {
        WM_MOUSEMOVE => {
            mouse_event.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
            mouse_event.dwFlags = TME_QUERY as DWORD;
            if TrackMouseEvent(core::ptr::addr_of_mut!(mouse_event)) == 0
                || (mouse_event.dwFlags & ((TME_HOVER | TME_LEAVE) as DWORD)) == 0
            {
                mouse_event.dwFlags = (TME_HOVER | TME_LEAVE) as DWORD;
                mouse_event.hwndTrack = hWnd;
                mouse_event.dwHoverTime = 1 as DWORD;
                TrackMouseEvent(core::ptr::addr_of_mut!(mouse_event));
            }
        }

        WM_MOUSEHOVER => {
            (*lpBtnInfo).bHover = TRUE as BOOL;
            InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE as BOOL);
        }

        WM_MOUSELEAVE => {
            (*lpBtnInfo).bHover = FALSE as BOOL;
            InvalidateRect(hWnd, core::ptr::null::<RECT>(), FALSE as BOOL);
        }

        _ => {}
    }

    return CallWindowProc((*lpBtnInfo).oldProc, hWnd, msg, wp, lp) as INT_PTR;
}

unsafe extern "system" fn EnumChildProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let mut szClass: [TCHAR; 64] = [0; 64];

    if GetClassName(hWnd, szClass.as_mut_ptr(), SIZEOF!(szClass) as core::ffi::c_int) == 0 {
        return (TRUE) as BOOL;
    }

    if _tcscmp(szClass.as_ptr(), WC_BUTTON) == 0 {
        let pnCtrls: *mut core::ffi::c_int = lParam as *mut core::ffi::c_int;
        let mut nCtrls: core::ffi::c_int = *pnCtrls;

        BtnInfo[nCtrls as usize].oldProc = core::mem::transmute::<LONG_PTR, WNDPROC>(
            GetWindowLongPtr(hWnd, GWLP_WNDPROC) as LONG_PTR,
        );
        BtnInfo[nCtrls as usize].bHover = (FALSE) as i32;

        SetWindowLongPtr(
            hWnd,
            GWLP_USERDATA,
            core::ptr::addr_of_mut!(BtnInfo[nCtrls as usize]) as LONG_PTR,
        );
        SetWindowLongPtr(
            hWnd,
            GWLP_WNDPROC,
            HotButtonProc as usize as LONG_PTR,
        );

        nCtrls += 1;
        *pnCtrls = nCtrls;
    }
    return (TRUE) as BOOL;
}

unsafe extern "system" fn OnSettingChange(hWnd: HWND, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    /* Check for user policy and area string valid */
    if wParam == 0 as WPARAM && lParam != 0 as LPARAM {
        let lpArea: LPTSTR = lParam as LPTSTR;

        /* Check if a parameter has been changed into the locale settings */
        let intl: [u16; 5] = ['i' as u16, 'n' as u16, 't' as u16, 'l' as u16, 0];
        if _tcsicmp(lpArea as *const u16, intl.as_ptr()) == 0 {
            /* Re-load locale parameters */
            UpdateNumberIntl();

            /* Update text for decimal button */
            SetDlgItemText(
                hWnd,
                IDC_BUTTON_DOT as i32,
                core::ptr::addr_of_mut!(calc.sDecimal) as *mut TCHAR as LPCWSTR,
            );

            /* Update text into the output display */
            update_lcd_display(hWnd);
        }
    }
    return 0 as INT_PTR;
}

unsafe extern "system" fn DlgMainProc(mut hWnd: HWND, mut msg: UINT, mut wp: WPARAM, mut lp: LPARAM) -> INT_PTR {
    let mut x: ::core::ffi::c_uint = 0;
    let mut rc: RECT = core::mem::zeroed();
    'sw: {
        match msg {
            m if m == WM_DRAWITEM => {
                return SubclassButtonProc(hWnd, wp, lp) as INT_PTR;
            }
            m if m == WM_INITDIALOG => {
                // The C's `#ifdef DISABLE_HTMLHELP_SUPPORT` menu-greying and
                // `#ifdef USE_KEYBOARD_HOOK` hook are dead: calc.h never defines either.
                calc.hWnd = hWnd;
                BtnCount = 0;
                EnumChildWindows(hWnd, Some(EnumChildProc), &raw mut BtnCount as LPARAM);
                rpn_zero(&raw mut calc.code);
                calc.sci_out = (FALSE) as i32;
                calc.base = IDC_RADIO_DEC;
                calc.size = IDC_RADIO_QWORD;
                calc.degr = IDC_RADIO_DEG;
                calc.ptr = &raw mut calc.buffer as *mut TCHAR;
                calc.is_nan = (FALSE) as i32;
                enable_allowed_controls(hWnd, IDC_RADIO_DEC);
                update_radio(hWnd, IDC_RADIO_DEC);
                update_menu(hWnd);
                display_rpn_result(hWnd, &raw mut calc.code);
                update_memory_flag(hWnd, calc.is_memory);
                SetFocus(GetDlgItem(hWnd, (IDC_BUTTON_FOCUS as i32)));
                SendMessage(hWnd, WM_SETICON, (ICON_BIG) as _, calc.hBgIcon as LPARAM);
                SendMessage(hWnd, WM_SETICON, (ICON_SMALL) as _, calc.hSmIcon as LPARAM);
                SetDlgItemText(hWnd, (IDC_BUTTON_DOT as i32), &raw mut calc.sDecimal as *mut TCHAR as LPCWSTR);
                if calc.layout == CALC_LAYOUT_CONVERSION {
                    ConvInit(hWnd);
                }
                if calc.x_coord >= 0 && calc.y_coord >= 0 {
                    let mut w: ::core::ffi::c_int = 0;
                    let mut h: ::core::ffi::c_int = 0;
                    let mut sw: ::core::ffi::c_int = 0;
                    let mut sh: ::core::ffi::c_int = 0;
                    GetWindowRect(hWnd, &raw mut rc);
                    w = rc.right - rc.left;
                    h = rc.bottom - rc.top;
                    sw = GetSystemMetrics((SM_CXSCREEN) as i32);
                    sh = GetSystemMetrics((SM_CYSCREEN) as i32);
                    if calc.x_coord + w > sw {
                        calc.x_coord = sw - w;
                    }
                    if calc.y_coord + h > sh {
                        calc.y_coord = sh - h;
                    }
                    MoveWindow(hWnd, calc.x_coord, calc.y_coord, w, h, (FALSE) as BOOL);
                }
                break 'sw;
            }
            m if m == WM_CTLCOLORSTATIC => {
                if lp as HWND == GetDlgItem(hWnd, (IDC_TEXT_OUTPUT as i32)) {
                    return GetStockObject((WHITE_BRUSH as i32)) as INT_PTR;
                }
                break 'sw;
            }
            m if m == WM_HANDLE_CLIPBOARD => {
                handle_sequence_input(hWnd, &raw mut calc.Clipboard);
                return (TRUE) as INT_PTR;
            }
            m if m == WM_COMMAND => {
                if wp == MAKEWPARAM!(IDC_COMBO_CATEGORY, CBN_SELCHANGE) as WPARAM {
                    ConvAdjust(hWnd, (SendDlgItemMessage(hWnd, (IDC_COMBO_CATEGORY as i32), CB_GETCURSEL, 0, 0)) as i32);
                    return (TRUE) as INT_PTR;
                }
                if (HIWORD!(wp)) as u32 != BN_CLICKED && (HIWORD!(wp)) as u32 != BN_DBLCLK {
                    break 'sw;
                }
                if GetFocus() != GetDlgItem(hWnd, (IDC_BUTTON_FOCUS as i32)) {
                    SetFocus(GetDlgItem(hWnd, (IDC_BUTTON_FOCUS as i32)));
                }
                match (LOWORD!(wp)) as u32 {
                    v if (v) as u32 == IDM_HELP_ABOUT => {
                        let mut infotitle: [TCHAR; 100] = [0; 100];
                        let mut infotext: [TCHAR; 200] = [0; 200];
                        LoadString(calc.hInstance, IDS_CALC_NAME, &raw mut infotitle as LPWSTR, SIZEOF!(infotitle) as ::core::ffi::c_int);
                        LoadString(calc.hInstance, IDS_AUTHOR, &raw mut infotext as LPWSTR, SIZEOF!(infotext) as ::core::ffi::c_int);
                        ShellAbout(hWnd, &raw const infotitle as *const TCHAR as LPCWSTR, &raw const infotext as *const TCHAR as LPCWSTR, calc.hBgIcon);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_HELP_HELP => {
                        let help_path: [u16; 53] = [
                            '%' as u16, 's' as u16, 'y' as u16, 's' as u16, 't' as u16, 'e' as u16,
                            'm' as u16, 'r' as u16, 'o' as u16, 'o' as u16, 't' as u16, '%' as u16,
                            '\\' as u16, 'H' as u16, 'e' as u16, 'l' as u16, 'p' as u16, '\\' as u16,
                            'c' as u16, 'a' as u16, 'l' as u16, 'c' as u16, '.' as u16, 'c' as u16,
                            'h' as u16, 'm' as u16, ':' as u16, ':' as u16,
                            '/' as u16, 'g' as u16, 'e' as u16, 'n' as u16, 'e' as u16, 'r' as u16,
                            'a' as u16, 'l' as u16, '_' as u16, 'i' as u16, 'n' as u16, 'f' as u16,
                            'o' as u16, 'r' as u16, 'm' as u16, 'a' as u16, 't' as u16, 'i' as u16,
                            'o' as u16, 'n' as u16, '.' as u16, 'h' as u16, 't' as u16, 'm' as u16,
                            0,
                        ];
                        calc_HtmlHelp!(hWnd, &raw const help_path as LPCWSTR, HH_DISPLAY_TOPIC, core::ptr::null_mut::<core::ffi::c_void>() as DWORD_PTR);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_VIEW_STANDARD => {
                        calc.layout = CALC_LAYOUT_STANDARD;
                        calc.action = IDM_VIEW_STANDARD;
                        DestroyWindow(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_VIEW_SCIENTIFIC => {
                        calc.layout = CALC_LAYOUT_SCIENTIFIC;
                        calc.action = IDM_VIEW_SCIENTIFIC;
                        DestroyWindow(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_VIEW_CONVERSION => {
                        calc.layout = CALC_LAYOUT_CONVERSION;
                        calc.action = IDM_VIEW_CONVERSION;
                        DestroyWindow(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_VIEW_HEX || (v) as u32 == IDM_VIEW_DEC || (v) as u32 == IDM_VIEW_OCT
                        || (v) as u32 == IDM_VIEW_BIN || (v) as u32 == IDM_VIEW_DEG || (v) as u32 == IDM_VIEW_RAD
                        || (v) as u32 == IDM_VIEW_GRAD || (v) as u32 == IDM_VIEW_QWORD || (v) as u32 == IDM_VIEW_DWORD
                        || (v) as u32 == IDM_VIEW_WORD || (v) as u32 == IDM_VIEW_BYTE => {
                        SendMessage(hWnd, WM_COMMAND, idm_2_idc((LOWORD!(wp)) as _), 0);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_EDIT_COPY => {
                        handle_copy_command(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_EDIT_PASTE => {
                        if !calc.Clipboard.data.is_null() {
                            break 'sw;
                        }
                        calc.Clipboard.data = ReadClipboard();
                        if !calc.Clipboard.data.is_null() {
                            PostMessage(hWnd, WM_COMMAND, (IDC_BUTTON_CE as WPARAM), 0);
                            calc.Clipboard.ptr = calc.Clipboard.data;
                            calc.Clipboard.wm_msg = WM_HANDLE_CLIPBOARD;
                            handle_sequence_input(hWnd, &raw mut calc.Clipboard);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDM_VIEW_GROUP => {
                        calc.usesep = if calc.usesep != 0 { (FALSE) as i32 } else { (TRUE) as i32 };
                        update_menu(hWnd);
                        update_lcd_display(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_CONVERT => {
                        ConvExecute(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_CE => {
                        let mut tmp: calc_number_t = core::mem::zeroed();
                        rpn_zero(&raw mut tmp);
                        display_rpn_result(hWnd, &raw mut tmp);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_RADIO_DEC || (v) as u32 == IDC_RADIO_HEX || (v) as u32 == IDC_RADIO_OCT
                        || (v) as u32 == IDC_RADIO_BIN => {
                        // winmain.c's `if (calc.X == LOWORD(wp)) break;` early-outs are under
                        // `#ifdef _MSC_VER`, which the clang/mingw build never defines: the C
                        // always forces the radio update (clears is_nan, re-masks the value).
                        calc.is_nan = (FALSE) as i32;
                        update_radio(hWnd, (LOWORD!(wp)) as _);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_RADIO_DEG || (v) as u32 == IDC_RADIO_RAD || (v) as u32 == IDC_RADIO_GRAD => {
                        calc.degr = LOWORD!(wp) as DWORD;
                        calc.is_nan = (FALSE) as i32;
                        update_menu(hWnd);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_RADIO_QWORD || (v) as u32 == IDC_RADIO_DWORD || (v) as u32 == IDC_RADIO_WORD
                        || (v) as u32 == IDC_RADIO_BYTE => {
                        calc.size = LOWORD!(wp) as DWORD;
                        calc.is_nan = (FALSE) as i32;
                        update_menu(hWnd);
                        convert_text2number(&raw mut calc.code);
                        apply_int_mask(&raw mut calc.code);
                        display_rpn_result(hWnd, &raw mut calc.code);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_1 || (v) as u32 == IDC_BUTTON_2 || (v) as u32 == IDC_BUTTON_3
                        || (v) as u32 == IDC_BUTTON_4 || (v) as u32 == IDC_BUTTON_5 || (v) as u32 == IDC_BUTTON_6
                        || (v) as u32 == IDC_BUTTON_7 || (v) as u32 == IDC_BUTTON_8 || (v) as u32 == IDC_BUTTON_9
                        || (v) as u32 == IDC_BUTTON_0 || (v) as u32 == IDC_BUTTON_DOT || (v) as u32 == IDC_BUTTON_A
                        || (v) as u32 == IDC_BUTTON_B || (v) as u32 == IDC_BUTTON_C || (v) as u32 == IDC_BUTTON_D
                        || (v) as u32 == IDC_BUTTON_E || (v) as u32 == IDC_BUTTON_F => {
                        if calc.is_nan != 0 {
                            break 'sw;
                        }
                        build_operand(hWnd, (LOWORD!(wp)) as DWORD);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_PERCENT || (v) as u32 == IDC_BUTTON_ADD || (v) as u32 == IDC_BUTTON_SUB
                        || (v) as u32 == IDC_BUTTON_MULT || (v) as u32 == IDC_BUTTON_DIV || (v) as u32 == IDC_BUTTON_MOD
                        || (v) as u32 == IDC_BUTTON_AND || (v) as u32 == IDC_BUTTON_OR || (v) as u32 == IDC_BUTTON_XOR
                        || (v) as u32 == IDC_BUTTON_LSH || (v) as u32 == IDC_BUTTON_RSH || (v) as u32 == IDC_BUTTON_EQU
                        || (v) as u32 == IDC_BUTTON_XeY || (v) as u32 == IDC_BUTTON_XrY => {
                        if calc.is_nan != 0 {
                            break 'sw;
                        }
                        if (get_modifiers(hWnd) & (MODIFIER_INV as i32)) != 0 {
                            let mut IdcSim: WPARAM = IDC_STATIC as WPARAM;
                            'sw3: {
                                match (LOWORD!(wp)) as u32 {
                                    v2 if (v2) as u32 == IDC_BUTTON_LSH => {
                                        IdcSim = MAKEWPARAM!(IDC_BUTTON_RSH, BN_CLICKED);
                                        break 'sw3;
                                    }
                                    v2 if (v2) as u32 == IDC_BUTTON_XeY => {
                                        IdcSim = MAKEWPARAM!(IDC_BUTTON_XrY, BN_CLICKED);
                                        break 'sw3;
                                    }
                                    _ => {}
                                }
                            }
                            if IdcSim != ((IDC_STATIC) as u32) as _ {
                                PostMessage(hWnd, WM_COMMAND, IdcSim, 0);
                                CheckDlgButton(hWnd, (IDC_CHECK_INV as i32), BST_UNCHECKED);
                                break 'sw;
                            }
                        }
                        x = 0;
                        while (x as usize) < SIZEOF!(operator_codes) {
                            if LOWORD!(wp) == operator_codes[x as usize] {
                                convert_text2number(&raw mut calc.code);
                                if calc.ptr == &raw mut calc.buffer as *mut TCHAR {
                                    if calc.last_operator != x {
                                        if x != RPN_OPERATOR_EQUAL {
                                            exec_change_infix();
                                        }
                                    } else if x == RPN_OPERATOR_EQUAL {
                                        exec_infix2postfix(&raw mut calc.code, calc.prev_operator);
                                        rpn_copy(&raw mut calc.code, &raw mut calc.prev);
                                    } else {
                                        break;
                                    }
                                }
                                if exec_infix2postfix(&raw mut calc.code, x) == 0 {
                                    break;
                                }
                                display_rpn_result(hWnd, &raw mut calc.code);
                                break;
                            }
                            x = x.wrapping_add(1);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_BACK => {
                        if calc.is_nan != 0 {
                            break 'sw;
                        }
                        if calc.sci_in != 0 {
                            if calc.esp == 0 {
                                let mut ptr: *mut TCHAR = core::ptr::null_mut();
                                calc.sci_in = (FALSE) as i32;
                                ptr = _tcschr(calc.ptr, 'e' as i32 as TCHAR);
                                if !ptr.is_null() {
                                    *ptr = '\0' as i32 as TCHAR;
                                }
                                update_lcd_display(hWnd);
                            } else {
                                calc.esp /= 10;
                                build_operand(hWnd, (IDC_STATIC) as DWORD);
                            }
                        } else if &raw mut calc.buffer as *mut TCHAR != calc.ptr {
                            calc.ptr = calc.ptr.offset(-1);
                            *calc.ptr = '\0' as i32 as TCHAR;
                            let dash: [u16; 2] = ['-' as u16, 0];
                            let dash0: [u16; 3] = ['-' as u16, '0' as u16, 0];
                            let zero: [u16; 2] = ['0' as u16, 0];
                            if _tcscmp(&raw mut calc.buffer as *mut TCHAR, &raw const dash as *const TCHAR) == 0
                                || _tcscmp(&raw mut calc.buffer as *mut TCHAR, &raw const dash0 as *const TCHAR) == 0
                                || _tcscmp(&raw mut calc.buffer as *mut TCHAR, &raw const zero as *const TCHAR) == 0
                            {
                                calc.ptr = &raw mut calc.buffer as *mut TCHAR;
                                calc.buffer[0] = '\0' as i32 as TCHAR;
                            }
                            update_lcd_display(hWnd);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_MC => {
                        rpn_zero(&raw mut calc.memory.number);
                        update_memory_flag(hWnd, (FALSE) as BOOL);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_MR => {
                        if calc.is_memory != 0 {
                            calc.is_nan = (FALSE) as i32;
                            rpn_copy(&raw mut calc.code, &raw mut calc.memory.number);
                            display_rpn_result(hWnd, &raw mut calc.code);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_EXP => {
                        if calc.sci_in != 0 || calc.is_nan != 0 || &raw mut calc.buffer as *mut TCHAR == calc.ptr {
                            break 'sw;
                        }
                        calc.sci_in = (TRUE) as i32;
                        calc.esp = 0;
                        build_operand(hWnd, (IDC_STATIC) as DWORD);
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_SIGN => {
                        if calc.sci_in != 0 {
                            calc.esp = 0 - calc.esp;
                            build_operand(hWnd, (IDC_STATIC) as DWORD);
                        } else {
                            if calc.is_nan != 0 || calc.buffer[0] == '\0' as i32 as TCHAR {
                                break 'sw;
                            }
                            if calc.buffer[0] == '-' as i32 as TCHAR {
                                memmove(
                                    (&raw mut calc.buffer as *mut TCHAR).cast(),
                                    (&raw mut calc.buffer as *mut TCHAR).offset(1 as isize).cast(),
                                    core::mem::size_of::<[TCHAR; 256]>() - 1
                                );
                                if &raw mut calc.buffer as *mut TCHAR != calc.ptr {
                                    calc.ptr = calc.ptr.offset(-1);
                                }
                            } else {
                                if calc.buffer[0] == '0' as i32 as TCHAR
                                    && calc.buffer[1] != '.' as i32 as TCHAR
                                {
                                    break 'sw;
                                }
                                memmove(
                                    ((&raw mut calc.buffer as *mut TCHAR).offset(1 as isize)).cast(),
                                    (&raw mut calc.buffer as *mut TCHAR).cast(),
                                    core::mem::size_of::<[TCHAR; 256]>() - 1
                                );
                                calc.buffer[0] = '-' as i32 as TCHAR;
                                if &raw mut calc.buffer as *mut TCHAR != calc.ptr {
                                    calc.ptr = calc.ptr.offset(1);
                                }
                            }
                            if &raw mut calc.buffer as *mut TCHAR == calc.ptr {
                                rpn_sign(&raw mut calc.code);
                            }
                            update_lcd_display(hWnd);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_RIGHTPAR || (v) as u32 == IDC_BUTTON_LEFTPAR || (v) as u32 == IDC_BUTTON_CANC
                        || (v) as u32 == IDC_BUTTON_MP || (v) as u32 == IDC_BUTTON_DAT || (v) as u32 == IDC_BUTTON_FE
                        || (v) as u32 == IDC_BUTTON_DMS || (v) as u32 == IDC_BUTTON_SQRT || (v) as u32 == IDC_BUTTON_S
                        || (v) as u32 == IDC_BUTTON_SUM || (v) as u32 == IDC_BUTTON_AVE || (v) as u32 == IDC_BUTTON_NF
                        || (v) as u32 == IDC_BUTTON_LN || (v) as u32 == IDC_BUTTON_LOG || (v) as u32 == IDC_BUTTON_Xe2
                        || (v) as u32 == IDC_BUTTON_Xe3 || (v) as u32 == IDC_BUTTON_PI || (v) as u32 == IDC_BUTTON_NOT
                        || (v) as u32 == IDC_BUTTON_RX || (v) as u32 == IDC_BUTTON_INT || (v) as u32 == IDC_BUTTON_SIN
                        || (v) as u32 == IDC_BUTTON_COS || (v) as u32 == IDC_BUTTON_TAN || (v) as u32 == IDC_BUTTON_MS => {
                        x = 0;
                        while (x as usize) < SIZEOF!(function_table) {
                            if LOWORD!(wp) == function_table[x as usize].idc {
                                let mut cb: rpn_callback1 = None;
                                if calc.is_nan != 0 && function_table[x as usize].check_nan != 0 {
                                    break;
                                }
                                calc.is_nan = (FALSE) as i32;
                                'sw4: {
                                    match (get_modifiers(hWnd) & function_table[x as usize].range as i32) as u32 {
                                        v2 if v2 == 0 => {
                                            cb = function_table[x as usize].direct;
                                            break 'sw4;
                                        }
                                        v2 if (v2) as u32 == MODIFIER_INV => {
                                            cb = function_table[x as usize].inverse;
                                            break 'sw4;
                                        }
                                        v2 if (v2) as u32 == MODIFIER_HYP => {
                                            cb = function_table[x as usize].hyperb;
                                            break 'sw4;
                                        }
                                        v2 if (v2) as u32 == (MODIFIER_INV | MODIFIER_HYP) => {
                                            cb = function_table[x as usize].inv_hyp;
                                            break 'sw4;
                                        }
                                        _ => {}
                                    }
                                }
                                if cb.is_some() {
                                    convert_text2number(&raw mut calc.code);
                                    (cb.unwrap_unchecked())(&raw mut calc.code);
                                    set_rpn_result(hWnd, &raw mut calc.code);
                                    if (function_table[x as usize].range as i32 & (NO_CHAIN as i32)) != 0 {
                                        calc.ptr = &raw mut calc.buffer as *mut TCHAR;
                                    }
                                    if (function_table[x as usize].range as i32 & (MODIFIER_INV as i32)) != 0 {
                                        CheckDlgButton(hWnd, (IDC_CHECK_INV as i32), BST_UNCHECKED);
                                    }
                                    if (function_table[x as usize].range as i32 & (MODIFIER_HYP as i32)) != 0 {
                                        CheckDlgButton(hWnd, (IDC_CHECK_HYP as i32), BST_UNCHECKED);
                                    }
                                }
                                break;
                            }
                            x = x.wrapping_add(1);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    v if (v) as u32 == IDC_BUTTON_STA => {
                        if IsWindow(calc.hStatWnd) != 0 {
                            break 'sw;
                        }
                        calc.hStatWnd = CreateDialog!(calc.hInstance, MAKEINTRESOURCE!(IDD_DIALOG_STAT), hWnd, Some(DlgStatProc));
                        if !calc.hStatWnd.is_null() {
                            enable_allowed_controls(hWnd, calc.base);
                            SendMessage(calc.hStatWnd, WM_SETFOCUS, 0, 0);
                        }
                        return (TRUE) as INT_PTR;
                    }
                    _ => {}
                }
                break 'sw;
            }
            m if m == WM_CLOSE_STATS => {
                calc.hStatWnd = core::ptr::null_mut();
                enable_allowed_controls(hWnd, calc.base);
                return (TRUE) as INT_PTR;
            }
            m if m == WM_LOAD_STAT => {
                if !upload_stat_number(LOWORD!(wp) as ::core::ffi::c_int).is_null() {
                    display_rpn_result(hWnd, &raw mut calc.code);
                }
                return (TRUE) as INT_PTR;
            }
            m if m == WM_START_CONV => {
                x = LOWORD!(lp) as ::core::ffi::c_uint;
                calc.Convert[x as usize].data = ReadConversion(calc.Convert[x as usize].data);
                if !calc.Convert[x as usize].data.is_null() {
                    calc.Convert[x as usize].ptr = calc.Convert[x as usize].data;
                    PostMessage(hWnd, (HIWORD!(lp)) as _, 0, 0);
                }
                return (TRUE) as INT_PTR;
            }
            m if m == WM_HANDLE_FROM => {
                if calc.is_nan != 0 {
                    break 'sw;
                }
                if handle_sequence_input(hWnd, &raw mut calc.Convert[0]).is_null() {
                    PostMessage(hWnd, WM_START_CONV, 0,
                        MAKELPARAM!(0x0001, WM_HANDLE_TO));
                }
                return (TRUE) as INT_PTR;
            }
            m if m == WM_HANDLE_TO => {
                if calc.is_nan == 0 {
                    handle_sequence_input(hWnd, &raw mut calc.Convert[1]);
                }
                return (TRUE) as INT_PTR;
            }
            m if m == WM_CLOSE => {
                calc.action = (IDC_STATIC) as u32;
                DestroyWindow(hWnd);
                return (TRUE) as INT_PTR;
            }
            m if m == WM_DESTROY => {
                GetWindowRect(hWnd, &raw mut rc);
                calc.x_coord = rc.left;
                calc.y_coord = rc.top;
                PostQuitMessage(0);
                return (TRUE) as INT_PTR;
            }
            m if m == WM_CONTEXTMENU => {
                if wp as HWND != hWnd {
                    handle_context_menu(hWnd, wp, lp);
                }
                return (TRUE) as INT_PTR;
            }
            m if m == WM_ENTERMENULOOP => {
                calc.is_menu_on = (TRUE) as i32;
                EnableMenuItem(GetSubMenu(GetMenu(hWnd), 0),
                    IDM_EDIT_PASTE,
                    MF_BYCOMMAND |
                    (if IsClipboardFormatAvailable(CF_TEXT) != 0 { MF_ENABLED } else { MF_GRAYED }));
                break 'sw;
            }
            m if m == WM_EXITMENULOOP => {
                calc.is_menu_on = (FALSE) as i32;
                break 'sw;
            }
            m if m == WM_SETTINGCHANGE => {
                return OnSettingChange(hWnd, wp, lp);
            }
            m if m == WM_THEMECHANGED => {
                InvalidateRect(hWnd, core::ptr::null_mut(), (FALSE) as BOOL);
                break 'sw;
            }
            _ => {}
        }
    }
    return (FALSE) as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn wWinMain(hInstance: HINSTANCE, hPrevInstance: HINSTANCE, lpCmdLine: LPTSTR, nShowCmd: i32) -> i32 {
    let mut msg: MSG = core::mem::zeroed();
    let mut dwLayout: DWORD = 0;

    /* Initialize controls for theming & manifest support */
    InitCommonControls();

    calc.hInstance = hInstance;

    calc.x_coord = -1;
    calc.y_coord = -1;

    load_config();
    start_rpn_engine();

    HtmlHelp_Start(hInstance);

    Theme_Start(hInstance);

    calc.hBgIcon = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_CALC),
        IMAGE_ICON,
        0,
        0,
        LR_DEFAULTSIZE | LR_SHARED,
    ) as HICON;

    calc.hSmIcon = LoadImage(
        hInstance,
        MAKEINTRESOURCE!(IDI_CALC),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        LR_SHARED,
    ) as HICON;

    loop {
        /* ignore hwnd: dialogs are already visible! */
        if calc.layout == CALC_LAYOUT_SCIENTIFIC {
            dwLayout = IDD_DIALOG_SCIENTIFIC;
        } else if calc.layout == CALC_LAYOUT_CONVERSION {
            dwLayout = IDD_DIALOG_CONVERSION;
        } else {
            dwLayout = IDD_DIALOG_STANDARD;
        }

        /* This call will always fail if UNICODE for Win9x */
        if core::ptr::null_mut() == CreateDialog!(hInstance, MAKEINTRESOURCE!(dwLayout), core::ptr::null_mut(), Some(DlgMainProc)) {
            break;
        }

        while GetMessage(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0,
            0,
        ) != 0
        {
            /* ignore hwnd: dialogs are already visible! */
            if (msg.message == WM_KEYUP ||
                msg.message == WM_KEYDOWN) &&
                calc.is_menu_on == 0
            {
                process_vk_key(msg.wParam, msg.lParam);
            }

            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }

        save_config();
        if !(calc.action != (IDC_STATIC) as u32) {
            break;
        }
    }

    stop_rpn_engine();

    Theme_Stop();
    HtmlHelp_Stop();

    return 0;
}
