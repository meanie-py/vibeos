//! stdlib/_invalid_parameter.c — the CRT invalid-parameter hook.
//!
//! Our config (`_MSVCREX_`, msvcrt classic → `_MSVCR_VER == 0`) compiles
//! the no-handler branch to a no-op (no RaiseException, ERR is empty).

pub type InvalidParamHandler =
    Option<unsafe extern "C" fn(*const u16, *const u16, *const u16, u32, usize)>;

static mut INVALID_PARAMETER_HANDLER: InvalidParamHandler = None;

#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter(
    expr: *const u16,
    func: *const u16,
    file: *const u16,
    line: u32,
    arg: usize,
) {
    if let Some(h) = INVALID_PARAMETER_HANDLER {
        h(expr, func, file, line, arg);
    }
    // else: no-op (msvcrt classic, _MSVCREX_).
}

#[no_mangle]
pub unsafe extern "C" fn _get_invalid_parameter_handler() -> InvalidParamHandler {
    INVALID_PARAMETER_HANDLER
}

#[no_mangle]
pub unsafe extern "C" fn _set_invalid_parameter_handler(
    handler: InvalidParamHandler,
) -> InvalidParamHandler {
    let old = INVALID_PARAMETER_HANDLER;
    INVALID_PARAMETER_HANDLER = handler;
    old
}
