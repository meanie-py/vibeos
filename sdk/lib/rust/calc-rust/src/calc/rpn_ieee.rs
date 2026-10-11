use crate::calc::fun_ieee::apply_int_mask;
use crate::calc::fun_ieee::logic_dbl2int;
use crate::calc::fun_ieee::logic_int2dbl;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut stack: *mut stack_node_t = ::core::ptr::null::<stack_node_t>() as *mut stack_node_t;

static mut temp: calc_node_t = calc_node_t {
    number: calc_number_t { f: 0. },
    operation: 0,
    base: 0,
};

static mut percent_mode: BOOL = 0;

static mut operator_list: [calc_operator_t; 15] = unsafe {
    [
        calc_operator_t {
            prec: 0 as ::core::ffi::c_uint,
            op_f: None,
            op_i: None,
            op_p: None,
        },
        calc_operator_t {
            prec: 0 as ::core::ffi::c_uint,
            op_f: None,
            op_i: None,
            op_p: None,
        },
        calc_operator_t {
            prec: 0 as ::core::ffi::c_uint,
            op_f: None,
            op_i: None,
            op_p: None,
        },
        calc_operator_t {
            prec: 1 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_or_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_or_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 2 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_xor_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_xor_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 3 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_and_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_and_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 4 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_shl_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_shl_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 4 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_shr_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_shr_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 5 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_add_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_add_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: Some(
                rpn_add_p
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
        },
        calc_operator_t {
            prec: 5 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_sub_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_sub_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: Some(
                rpn_sub_p
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
        },
        calc_operator_t {
            prec: 6 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_mul_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_mul_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: Some(
                rpn_mul_p
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
        },
        calc_operator_t {
            prec: 6 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_div_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_div_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: Some(
                rpn_div_p
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
        },
        calc_operator_t {
            prec: 6 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_mod_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: Some(
                rpn_mod_i
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_p: None,
        },
        calc_operator_t {
            prec: 7 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_pow_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: None,
            op_p: None,
        },
        calc_operator_t {
            prec: 7 as ::core::ffi::c_uint,
            op_f: Some(
                rpn_sqr_f
                    as unsafe extern "C" fn(
                        *mut calc_number_t,
                        *mut calc_number_t,
                        *mut calc_number_t,
                    ) -> (),
            ),
            op_i: None,
            op_p: None,
        },
    ]
};

#[repr(C)]
pub struct stack_node_t {
    pub node: calc_node_t,
    pub next: PVOID,
}

pub type operator_call = unsafe extern "C" fn(*mut calc_number_t, *mut calc_number_t, *mut calc_number_t);

#[repr(C)]
pub struct calc_operator_t {
    pub prec: u32,
    pub op_f: Option<operator_call>,
    pub op_i: Option<operator_call>,
    pub op_p: Option<operator_call>,
}

unsafe extern "C" fn pop() -> *mut calc_node_t {
    let mut next: *mut core::ffi::c_void = core::ptr::null_mut();
    if stack.is_null() {
        return core::ptr::null_mut();
    }
    temp = (*stack).node;
    next = (*stack).next;
    free(stack as *mut core::ffi::c_void);
    stack = next as *mut stack_node_t;
    return core::ptr::addr_of_mut!(temp);
}

unsafe extern "C" fn is_stack_empty() -> i32 {
    return (stack == core::ptr::null_mut()) as i32;
}

unsafe extern "C" fn push(op: *mut calc_node_t) {
    let z: *mut stack_node_t =
        malloc(::core::mem::size_of::<stack_node_t>() as size_t) as *mut stack_node_t;

    (*z).node = *op;
    (*z).next = stack as *mut ::core::ffi::c_void;
    stack = z;
}

unsafe extern "C" fn rpn_add_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f + (*b).f;
}

unsafe extern "C" fn rpn_sub_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f - (*b).f;
}

unsafe extern "C" fn rpn_mul_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f * (*b).f;
}

unsafe extern "C" fn rpn_div_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    if (*b).f == 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*r).f = (*a).f / (*b).f;
    }
}

unsafe extern "C" fn rpn_mod_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut t: f64 = 0.0;
    if (*b).f == 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        modf((*a).f / (*b).f, core::ptr::addr_of_mut!(t));
        (*r).f = (*a).f - (t * (*b).f);
    }
}

unsafe extern "C" fn rpn_and_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut ai: calc_number_t = core::mem::zeroed();
    let mut bi: calc_number_t = core::mem::zeroed();
    ai.i = logic_dbl2int(a);
    bi.i = logic_dbl2int(b);
    (*r).f = (ai.i & bi.i) as f64;
}

unsafe extern "C" fn rpn_or_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut ai: calc_number_t = core::mem::zeroed();
    let mut bi: calc_number_t = core::mem::zeroed();

    ai.i = logic_dbl2int(a);
    bi.i = logic_dbl2int(b);

    (*r).f = (ai.i | bi.i) as f64 as _;
}

unsafe extern "C" fn rpn_xor_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut ai: calc_number_t = core::mem::zeroed();
    let mut bi: calc_number_t = core::mem::zeroed();

    ai.i = logic_dbl2int(a);
    bi.i = logic_dbl2int(b);

    (*r).f = (ai.i ^ bi.i) as f64;
}

unsafe extern "C" fn rpn_shl_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut n: calc_number_t = calc_number_t { f: 0. };

    modf((*b).f, core::ptr::addr_of_mut!(n.f));

    (*r).f = (*a).f * pow(2., n.f);
}

unsafe extern "C" fn rpn_shr_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    let mut n: calc_number_t = core::mem::zeroed();

    modf((*b).f, core::ptr::addr_of_mut!(n.f));

    (*r).f = (*a).f / pow(2.0, n.f);
}

unsafe extern "C" fn rpn_pow_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = pow((*a).f, (*b).f);
    if _finite((*r).f) == 0 || _isnan((*r).f) != 0 {
        calc.is_nan = (TRUE) as i32;
    }
}

unsafe extern "C" fn rpn_sqr_f(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    if (*b).f == 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*r).f = pow((*a).f, 1.0 / (*b).f);
        if _finite((*r).f) == 0 || _isnan((*r).f) != 0 {
            calc.is_nan = (TRUE) as i32;
        }
    }
}

unsafe extern "C" fn rpn_add_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i + (*b).i;
}

unsafe extern "C" fn rpn_sub_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i - (*b).i;
}

unsafe extern "C" fn rpn_mul_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i * (*b).i;
}

unsafe extern "C" fn rpn_div_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    if (*b).i == 0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*r).i = (*a).i / (*b).i;
    }
}

unsafe extern "C" fn rpn_mod_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    if (*b).i == 0 as INT64 {
        calc.is_nan = TRUE as BOOL;
    } else {
        (*r).i = (*a).i % (*b).i;
    }
}

unsafe extern "C" fn rpn_and_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i & (*b).i;
}

unsafe extern "C" fn rpn_or_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i | (*b).i;
}

unsafe extern "C" fn rpn_xor_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i ^ (*b).i;
}

unsafe extern "C" fn rpn_shl_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i << (*b).i;
}

unsafe extern "C" fn rpn_shr_i(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).i = (*a).i >> (*b).i;
}

unsafe extern "C" fn rpn_add_p(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f * (1. + (*b).f / 100.);
}

unsafe extern "C" fn rpn_sub_p(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f * (1. - (*b).f / 100.);
}

unsafe extern "C" fn rpn_mul_p(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    (*r).f = (*a).f * (*b).f / 100.;
}

unsafe extern "C" fn rpn_div_p(r: *mut calc_number_t, a: *mut calc_number_t, b: *mut calc_number_t) {
    if (*b).f == 0.0 {
        calc.is_nan = (TRUE) as i32;
    } else {
        (*r).f = (*a).f * 100. / (*b).f;
    }
}
#[no_mangle]
pub unsafe extern "C" fn run_operator(result: *mut calc_node_t, a: *mut calc_node_t, b: *mut calc_node_t, operation: ::core::ffi::c_uint) {
    let mut da: calc_number_t = core::mem::zeroed();
    let mut db: calc_number_t = core::mem::zeroed();
    let mut dc: calc_number_t = core::mem::zeroed();
    let mut base: DWORD = calc.base;

    da = (*a).number;
    db = (*b).number;
    if (*a).base == IDC_RADIO_DEC && (*b).base != IDC_RADIO_DEC {
        db.f = logic_int2dbl(core::ptr::addr_of_mut!((*b).number));
        base = IDC_RADIO_DEC;
    } else if (*a).base != IDC_RADIO_DEC && (*b).base == IDC_RADIO_DEC {
        da.f = logic_int2dbl(core::ptr::addr_of_mut!((*a).number));
        base = IDC_RADIO_DEC;
    }

    if base == IDC_RADIO_DEC {
        if percent_mode != 0 {
            percent_mode = (FALSE) as i32;
            (operator_list[operation as usize]
                .op_p
                .unwrap_unchecked())(
                core::ptr::addr_of_mut!(dc),
                core::ptr::addr_of_mut!(da),
                core::ptr::addr_of_mut!(db),
            );
        } else {
            (operator_list[operation as usize]
                .op_f
                .unwrap_unchecked())(
                core::ptr::addr_of_mut!(dc),
                core::ptr::addr_of_mut!(da),
                core::ptr::addr_of_mut!(db),
            );
        }
        if _finite(dc.f) == 0 {
            calc.is_nan = (TRUE) as i32;
        }
    } else {
        (operator_list[operation as usize]
            .op_i
            .unwrap_unchecked())(
            core::ptr::addr_of_mut!(dc),
            core::ptr::addr_of_mut!(da),
            core::ptr::addr_of_mut!(db),
        );
        /* apply final limiter to result */
        apply_int_mask(core::ptr::addr_of_mut!(dc));
    }

    if (*a).base == IDC_RADIO_DEC && (*b).base != IDC_RADIO_DEC {
        (*result).number.i = logic_dbl2int(core::ptr::addr_of_mut!(dc)) as INT64;
        apply_int_mask(core::ptr::addr_of_mut!((*result).number));
    } else if (*a).base != IDC_RADIO_DEC && (*b).base == IDC_RADIO_DEC {
        (*result).number.f = dc.f;
    } else {
        (*result).number = dc;
    }
}

unsafe extern "C" fn evalStack(mut number: *mut calc_number_t) {
    let mut op: *mut calc_node_t = ::core::ptr::null_mut::<calc_node_t>();
    let mut ip: calc_node_t = calc_node_t {
        number: calc_number_t { f: 0. },
        operation: 0,
        base: 0,
    };
    let mut prec: ::core::ffi::c_uint = 0;
    op = pop();
    ip = *op;
    prec = operator_list[ip.operation as usize].prec;
    while is_stack_empty() == 0 {
        op = pop();
        if prec <= operator_list[(*op).operation as usize].prec {
            if (*op).operation == RPN_OPERATOR_PARENT as ::core::ffi::c_uint {
                continue;
            }
            calc.prev = ip.number;
            run_operator(&raw mut ip, op, &raw mut ip, (*op).operation);
            if calc.is_nan != 0 {
                flush_postfix();
                return;
            }
        } else {
            push(op);
            break;
        }
    }
    if ip.operation != RPN_OPERATOR_EQUAL as ::core::ffi::c_uint
        && ip.operation != RPN_OPERATOR_PERCENT as ::core::ffi::c_uint
    {
        push(&raw mut ip);
    }
    calc.prev_operator = (*op).operation;
    *number = ip.number;
}

#[no_mangle]
pub unsafe extern "C" fn exec_infix2postfix(number: *mut calc_number_t, func: ::core::ffi::c_uint) -> ::core::ffi::c_int {
    let mut tmp: calc_node_t = core::mem::zeroed();

    if is_stack_empty() != 0 && func == RPN_OPERATOR_EQUAL {
        /* if a number has been entered with exponential */
        /* notation, I may update it with normal mode */
        if calc.sci_in != 0 {
            return 1;
        }
        return 0;
    }

    if func == RPN_OPERATOR_PERCENT {
        percent_mode = (TRUE) as i32;
    }

    tmp.number = *number;
    tmp.base = calc.base;
    tmp.operation = func;

    push(core::ptr::addr_of_mut!(tmp));

    if func == RPN_OPERATOR_NONE {
        return 0;
    }

    if func != RPN_OPERATOR_PARENT {
        calc.last_operator = func;
        evalStack(number);
    }
    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn exec_change_infix() {
    let mut op: *mut stack_node_t = stack;
    if op.is_null() {
        return;
    }
    if (*op).node.operation == RPN_OPERATOR_PARENT
        || (*op).node.operation == RPN_OPERATOR_PERCENT
        || (*op).node.operation == RPN_OPERATOR_EQUAL
    {
        return;
    }
    pop();
}
#[no_mangle]
pub unsafe extern "C" fn exec_closeparent(number: *mut calc_number_t) {
    let mut op: *mut calc_node_t = core::ptr::null_mut();
    let mut ip: calc_node_t = core::mem::zeroed();

    ip.number = *number;
    ip.base = calc.base;
    while is_stack_empty() == 0 {
        op = pop();

        if (*op).operation == RPN_OPERATOR_PARENT {
            break;
        }

        run_operator(core::ptr::addr_of_mut!(ip), op, core::ptr::addr_of_mut!(ip), (*op).operation);
        if calc.is_nan != 0 {
            flush_postfix();
            return;
        }
    }
    *number = ip.number;
}
#[no_mangle]
pub unsafe extern "C" fn eval_parent_count() -> ::core::ffi::c_int {
    let mut s: *mut stack_node_t = stack;
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !s.is_null() {
        if (*s).node.operation == RPN_OPERATOR_PARENT as ::core::ffi::c_int as ::core::ffi::c_uint {
            n += 1;
        }
        s = (*s).next as *mut stack_node_t;
    }
    return n;
}
#[no_mangle]
pub unsafe extern "C" fn flush_postfix() {
    while is_stack_empty() == 0 {
        pop();
    }
    /* clear prev and last typed operators */
    calc.last_operator = 0;
    calc.prev_operator = 0;
}
#[no_mangle]
pub unsafe extern "C" fn start_rpn_engine() {
    stack = core::ptr::null_mut::<stack_node_t>();
}

#[no_mangle]
pub unsafe extern "C" fn stop_rpn_engine() {}
