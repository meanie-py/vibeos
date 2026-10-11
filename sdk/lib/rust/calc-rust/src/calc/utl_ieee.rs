use crate::calc::fun_ieee::apply_int_mask;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn prepare_rpn_result_2(rpn: *mut calc_number_t, buffer: *mut TCHAR, size: ::core::ffi::c_int, base: ::core::ffi::c_int) {
    let mut tmp: calc_number_t = *rpn;
    let mut width: i32 = 0;
    match (base) as u32 {
        IDC_RADIO_HEX => {
            StringCchPrintf!(buffer, size, (&[0x0025, 0x0049, 0x0036, 0x0034, 0x0058, 0x0000u16]).as_ptr() /* _T "%I64X" */, (*rpn).i);
        }
        IDC_RADIO_DEC => {
            width = if (*rpn).f == 0 as f64 {
                1
            } else {
                log10(fabs((*rpn).f)) as i32
                    + 1
            };
            if (calc.sci_out) as u32 == TRUE
                || width > MAX_LD_WIDTH
                || width < -MAX_LD_WIDTH
            {
                StringCchPrintf!(
                    buffer,
                    size,
                    (&[0x0025, 0x0023, 0x002e, 0x002a, 0x0065, 0x0000u16]).as_ptr() /* _T "%#.*e" */,
                    MAX_LD_WIDTH - 1,
                    (*rpn).f,
                );
            } else {
                let mut ptr: *mut TCHAR = core::ptr::null_mut();
                let mut dst: *mut TCHAR = core::ptr::null_mut();
                StringCchPrintfEx!(
                    buffer,
                    size,
                    core::ptr::addr_of_mut!(ptr),
                    core::ptr::null_mut(),
                    0,
                    (&[0x0025, 0x0023, 0x002a, 0x002e, 0x002a, 0x0066, 0x0000u16]).as_ptr() /* _T "%#*.*f" */,
                    width,
                    if (MAX_LD_WIDTH - width - 1)
                        >= 0
                    {
                        MAX_LD_WIDTH - width - 1
                    } else {
                        0
                    },
                    (*rpn).f,
                );
                /* format string ensures there is a '.': */
                dst = _tcschr(buffer, ('.' as u16));
                loop {
                    ptr = ptr.offset(-1);
                    if !(ptr > dst) {
                        break;
                    }
                    if *ptr != ('0' as u16) {
                        break;
                    }
                }

                /* put the string terminator for removing the final '0' (if any) */
                *ptr.offset(1 as isize) = ('\0' as u16);
                /* check if the number finishes with '.' */
                if ptr == dst {
                    /* remove the dot (it will be re-added later) */
                    *ptr.offset(0 as isize) = ('\0' as u16);
                }
            }
        }
        IDC_RADIO_OCT => {
            StringCchPrintf!(buffer, size, (&[0x0025, 0x0049, 0x0036, 0x0034, 0x006f, 0x0000u16]).as_ptr() /* _T "%I64o" */, (*rpn).i);
        }
        IDC_RADIO_BIN => {
            if (*rpn).i == 0 as i64 {
                *buffer.offset(0 as isize) = ('0' as u16);
                *buffer.offset(1 as isize) = ('\0' as u16);
            } else {
                tmp = *rpn;
                *buffer.offset(0 as isize) = ('\0' as u16);
                while tmp.u != 0 {
                    memmove(
                        buffer.offset(1 as isize) as *mut core::ffi::c_void,
                        buffer as *const core::ffi::c_void,
                        ((size - 1) as usize)
                            .wrapping_mul(core::mem::size_of::<TCHAR>() as usize),
                    );
                    if (tmp.u & 1 as u64) != 0 {
                        *buffer.offset(0 as isize) = ('1' as u16);
                    } else {
                        *buffer.offset(0 as isize) = ('0' as u16);
                    }
                    tmp.u >>= 1 as u32;
                }
            }
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn convert_text2number_2(a: *mut calc_number_t) {
    let mut ptr: *mut TCHAR = core::ptr::null_mut::<TCHAR>();
    match calc.base {
        IDC_RADIO_HEX => {
            let fmt: [u16; 6] = ['%' as u16, 'I' as u16, '6' as u16, '4' as u16, 'X' as u16, 0];
            _stscanf(
                core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR,
                fmt.as_ptr() as *const u16,
                core::ptr::addr_of_mut!((*a).i),
            );
        }
        IDC_RADIO_DEC => {
            let fmt: [u16; 4] = ['%' as u16, 'l' as u16, 'f' as u16, 0];
            _stscanf(
                core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR,
                fmt.as_ptr() as *const u16,
                core::ptr::addr_of_mut!((*a).f),
            );
        }
        IDC_RADIO_OCT => {
            let fmt: [u16; 6] = ['%' as u16, 'I' as u16, '6' as u16, '4' as u16, 'o' as u16, 0];
            _stscanf(
                core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR,
                fmt.as_ptr() as *const u16,
                core::ptr::addr_of_mut!((*a).i),
            );
        }
        IDC_RADIO_BIN => {
            ptr = core::ptr::addr_of_mut!(calc.buffer) as *mut TCHAR;
            (*a).i = 0 as INT64;
            while *ptr != 0u16 {
                (*a).i <<= 1;
                let fresh31 = ptr;
                ptr = ptr.offset(1);
                if *fresh31 == '1' as u16 {
                    (*a).i |= 1 as INT64;
                }
            }
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn convert_real_integer(base: ::core::ffi::c_uint) {
    match base {
        IDC_RADIO_DEC => {
            calc.code.f = calc.code.i as ::core::ffi::c_double;
        }
        IDC_RADIO_OCT | IDC_RADIO_BIN | IDC_RADIO_HEX => {
            if calc.base == IDC_RADIO_DEC as DWORD {
                calc.code.i = calc.code.f as ::core::ffi::c_longlong as INT64;
                apply_int_mask(core::ptr::addr_of_mut!(calc.code));
            }
        }
        _ => {}
    }
}
