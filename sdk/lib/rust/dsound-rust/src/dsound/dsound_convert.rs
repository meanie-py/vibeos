use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut convertbpp: [[bitsconvertfunc; 4]; 5] = unsafe {
    [
        [
            Some(
                convert_8_to_8
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_8_to_16
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_8_to_24
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_8_to_32
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
        ],
        [
            Some(
                convert_16_to_8
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_16_to_16
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_16_to_24
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_16_to_32
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
        ],
        [
            Some(
                convert_24_to_8
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_24_to_16
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_24_to_24
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_24_to_32
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
        ],
        [
            Some(
                convert_32_to_8
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_32_to_16
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_32_to_24
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_32_to_32
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
        ],
        [
            Some(
                convert_ieee_32_to_8
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_ieee_32_to_16
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_ieee_32_to_24
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
            Some(
                convert_ieee_32_to_32
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *mut ::core::ffi::c_void,
                        UINT,
                        UINT,
                        INT,
                        UINT,
                        UINT,
                    ) -> (),
            ),
        ],
    ]
};

#[no_mangle]
pub static mut mixfunctions: [mixfunc; 4] = unsafe {
    [
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    *mut ::core::ffi::c_schar,
                    *mut INT,
                    ::core::ffi::c_uint,
                ) -> (),
            >,
            mixfunc,
        >(Some(
            mix8 as unsafe extern "C" fn(
                *mut ::core::ffi::c_schar,
                *mut INT,
                ::core::ffi::c_uint,
            ) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut SHORT, *mut INT, ::core::ffi::c_uint) -> ()>,
            mixfunc,
        >(Some(
            mix16 as unsafe extern "C" fn(*mut SHORT, *mut INT, ::core::ffi::c_uint) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut BYTE, *mut INT, ::core::ffi::c_uint) -> ()>,
            mixfunc,
        >(Some(
            mix24 as unsafe extern "C" fn(*mut BYTE, *mut INT, ::core::ffi::c_uint) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut INT, *mut LONGLONG, ::core::ffi::c_uint) -> ()>,
            mixfunc,
        >(Some(
            mix32 as unsafe extern "C" fn(*mut INT, *mut LONGLONG, ::core::ffi::c_uint) -> (),
        )),
    ]
};

#[no_mangle]
pub static mut normfunctions: [normfunc; 4] = unsafe {
    [
        ::core::mem::transmute::<
            Option<
                unsafe extern "C" fn(
                    *mut INT,
                    *mut ::core::ffi::c_schar,
                    ::core::ffi::c_uint,
                ) -> (),
            >,
            normfunc,
        >(Some(
            norm8
                as unsafe extern "C" fn(
                    *mut INT,
                    *mut ::core::ffi::c_schar,
                    ::core::ffi::c_uint,
                ) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut INT, *mut SHORT, ::core::ffi::c_uint) -> ()>,
            normfunc,
        >(Some(
            norm16 as unsafe extern "C" fn(*mut INT, *mut SHORT, ::core::ffi::c_uint) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut INT, *mut BYTE, ::core::ffi::c_uint) -> ()>,
            normfunc,
        >(Some(
            norm24 as unsafe extern "C" fn(*mut INT, *mut BYTE, ::core::ffi::c_uint) -> (),
        )),
        ::core::mem::transmute::<
            Option<unsafe extern "C" fn(*mut LONGLONG, *mut INT, ::core::ffi::c_uint) -> ()>,
            normfunc,
        >(Some(
            norm32 as unsafe extern "C" fn(*mut LONGLONG, *mut INT, ::core::ffi::c_uint) -> (),
        )),
    ]
};

unsafe extern "C" fn src_advance(src: *mut *const core::ffi::c_void, stride: UINT, count: *mut INT, freqAcc: *mut UINT, adj: UINT) {
    *freqAcc = (*freqAcc).wrapping_add(adj);
    if *freqAcc >= ((1 << DSOUND_FREQSHIFT) as UINT) {
        let adv: ULONG = (*freqAcc >> DSOUND_FREQSHIFT) as ULONG;
        *freqAcc &= ((1 << DSOUND_FREQSHIFT) - 1) as UINT;
        *(src as *mut *const core::ffi::c_char) = (*(src as *mut *const core::ffi::c_char))
            .offset(adv.wrapping_mul(stride as ULONG) as isize);
        *count = (*count as ULONG).wrapping_sub(adv) as INT;
    }
}

unsafe extern "C" fn convert_8_to_8(
    mut src: *const core::ffi::c_void,
    mut dst: *mut core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        *(dst as *mut BYTE) = *(src as *const BYTE);
        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_8_to_16(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        let mut dest: WORD = *(src as *const BYTE) as WORD;
        let mut dest16: *mut WORD = dst as *mut WORD;
        *dest16 = le16!(
            (dest as i32 * 257 - 32768) as WORD,
        );

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize)
            as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_8_to_24(src: *const core::ffi::c_void, dst: *mut core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut count = count;
    let mut freqAcc = freqAcc;
    while count > 0 {
        let dest: BYTE = *(src as *const BYTE);
        let dest24: *mut BYTE = dst as *mut BYTE;
        *dest24.offset(0 as isize) = dest;
        *dest24.offset(1 as isize) = dest;
        *dest24.offset(2 as isize) = (dest as INT - 0x80) as BYTE;

        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_8_to_32(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        let mut dest: DWORD = *(src as *const BYTE) as DWORD;
        let mut dest32: *mut DWORD = dst as *mut DWORD;
        *dest32 = le32!(dest.wrapping_mul(16843009).wrapping_sub(2147483648));

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_16_to_8(src: *const ::core::ffi::c_void, dst: *mut ::core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut src_stride = src_stride;
    let mut dst_stride = dst_stride;
    let mut count = count;
    let mut freqAcc = freqAcc;
    let mut adj = adj;
    while count > 0 {
        let mut dst8: *mut BYTE = dst as *mut BYTE;
        *dst8 = (le16!(*(src as *const WORD)) as ::core::ffi::c_int / 256) as BYTE;
        *dst8 = (*dst8 as ::core::ffi::c_int - 0x80) as BYTE;

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_16_to_16(src: *const ::core::ffi::c_void, dst: *mut ::core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut count = count;
    let mut freqAcc = freqAcc;
    while count > 0 {
        *(dst as *mut WORD) = *(src as *const WORD);
        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_16_to_24(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    src_stride: UINT,
    dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    adj: UINT,
) {
    while count > 0 {
        let dest: WORD = le16!(*(src as *const WORD));
        let dest24: *mut BYTE = dst as *mut BYTE;

        *dest24.offset(0 as isize) = (dest / 256) as BYTE;
        *dest24.offset(1 as isize) = dest as BYTE;
        *dest24.offset(2 as isize) = (dest / 256) as BYTE;

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_16_to_32(src: *const core::ffi::c_void, dst: *mut core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut count = count;
    let mut freqAcc = freqAcc;
    while count > 0 {
        let mut dest: DWORD = *(src as *const WORD) as DWORD;
        let mut dest32: *mut DWORD = dst as *mut DWORD;
        *dest32 = dest.wrapping_mul(65537 as DWORD);

        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_24_to_8(
    mut src: *const core::ffi::c_void,
    mut dst: *mut core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        let mut dst8: *mut BYTE = dst as *mut BYTE;
        *dst8 = *(src as *const BYTE).offset(2 as isize);

        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_24_to_16(mut src: *const ::core::ffi::c_void, mut dst: *mut ::core::ffi::c_void, src_stride: UINT, dst_stride: UINT, mut count: INT, mut freqAcc: UINT, adj: UINT) {
    while count > 0 as INT {
        let dest16: *mut WORD = dst as *mut WORD;
        let source: *const BYTE = src as *const BYTE;
        *dest16 = le16!(((*source.offset(2 as isize) as ::core::ffi::c_int * 256 as ::core::ffi::c_int
            + *source.offset(1 as isize) as ::core::ffi::c_int) as WORD));
        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_24_to_24(mut src: *const core::ffi::c_void, mut dst: *mut core::ffi::c_void, src_stride: UINT, dst_stride: UINT, mut count: INT, mut freqAcc: UINT, adj: UINT) {
    while count > 0 {
        let mut dest24: *mut BYTE = dst as *mut BYTE;
        let mut src24: *const BYTE = src as *const BYTE;

        *dest24.offset(0 as isize) = *src24.offset(0 as isize);
        *dest24.offset(1 as isize) = *src24.offset(1 as isize);
        *dest24.offset(2 as isize) = *src24.offset(2 as isize);

        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_24_to_32(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        let mut dest32: *mut DWORD = dst as *mut DWORD;
        let mut source: *const BYTE = src as *const BYTE;
        *dest32 = le32!(
            ((*source.add(2) as INT * 16777217
                + *source.add(1) as INT * 65536
                + *source.add(0) as INT * 256) as DWORD),
        );

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize)
            as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_32_to_8(mut src: *const c_void, mut dst: *mut c_void, mut src_stride: UINT, mut dst_stride: UINT, mut count: INT, mut freqAcc: UINT, mut adj: UINT) {
    while count > 0 {
        let mut dst8: *mut BYTE = dst as *mut BYTE;
        *dst8 = (le32!(*(src as *const DWORD)) / 16777216) as BYTE;
        *dst8 = (*dst8 as INT - 0x80 as INT) as BYTE;
        dst = (dst as *mut c_char).offset(dst_stride as isize) as *mut c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_32_to_16(src: *const ::core::ffi::c_void, dst: *mut ::core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut src_stride = src_stride;
    let mut dst_stride = dst_stride;
    let mut count = count;
    let mut freqAcc = freqAcc;
    let mut adj = adj;
    while count > 0 {
        let mut dest16: *mut WORD = dst as *mut WORD;
        *dest16 = le16!(le32!(*(src as *const DWORD)) / 65536) as WORD;

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_32_to_24(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 {
        let mut dest: DWORD = le32!(*(src as *const DWORD));
        let mut dest24: *mut BYTE = dst as *mut BYTE;

        *dest24.offset(0 as isize) = (dest / 256) as BYTE;
        *dest24.offset(1 as isize) = (dest / 65536) as BYTE;
        *dest24.offset(2 as isize) = (dest / 16777216) as BYTE;

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_32_to_32(src: *const core::ffi::c_void, dst: *mut core::ffi::c_void, src_stride: UINT, dst_stride: UINT, count: INT, freqAcc: UINT, adj: UINT) {
    let mut src = src;
    let mut dst = dst;
    let mut count = count;
    let mut freqAcc = freqAcc;
    while count > 0 {
        let dest: *mut DWORD = dst as *mut DWORD;
        *dest = *(src as *const DWORD);
        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_ieee_32_to_8(mut src: *const ::core::ffi::c_void, mut dst: *mut ::core::ffi::c_void, mut src_stride: UINT, mut dst_stride: UINT, mut count: INT, mut freqAcc: UINT, mut adj: UINT) {
    while count > 0 {
        let mut src_le: DWORD = le32!(*(src as *mut DWORD));
        let mut v: f32 = *(core::ptr::addr_of_mut!(src_le) as *mut f32);
        let mut d: INT8 = 0 as INT8;

        if v < -1.0f32 {
            d = -(128 as ::core::ffi::c_int) as INT8;
        } else if v > 1.0f32 {
            d = 127 as INT8;
        } else {
            d = (v * 127.5f32 - 0.5f32) as INT8;
        }

        *(dst as *mut BYTE) = ((d as ::core::ffi::c_int) ^ (0x80 as ::core::ffi::c_int)) as BYTE;

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(core::ptr::addr_of_mut!(src), src_stride, core::ptr::addr_of_mut!(count), core::ptr::addr_of_mut!(freqAcc), adj);
    }
}

unsafe extern "C" fn convert_ieee_32_to_16(src: *const core::ffi::c_void, mut dst: *mut core::ffi::c_void, src_stride: UINT, dst_stride: UINT, mut count: INT, mut freqAcc: UINT, adj: UINT) {
    let mut src = src;
    while count > 0 {
        let mut src_le: DWORD = le32!(*(src as *mut DWORD));
        let v: f32 = *(core::ptr::addr_of_mut!(src_le) as *mut f32);

        let d: *mut INT16 = dst as *mut INT16;

        if v < -1.0f32 {
            *d = -32768;
        } else if v > 1.0f32 {
            *d = 32767;
        } else {
            *d = (v * 32767.5f32 - 0.5f32) as INT16;
        }

        *d = le16!(*d);

        dst = (dst as *mut core::ffi::c_char).offset(dst_stride as isize) as *mut core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn convert_ieee_32_to_24(
    mut src: *const ::core::ffi::c_void,
    mut dst: *mut ::core::ffi::c_void,
    mut src_stride: UINT,
    mut dst_stride: UINT,
    mut count: INT,
    mut freqAcc: UINT,
    mut adj: UINT,
) {
    while count > 0 as INT {
        let mut src_le: DWORD = le32!(*(src as *const DWORD));
        let mut v: f32 = *((&raw mut src_le) as *mut f32);
        let mut dest24: *mut BYTE = dst as *mut BYTE;

        if v < -1.0f32 {
            *dest24.offset(0 as isize) = 0 as BYTE;
            *dest24.offset(1 as isize) = 0 as BYTE;
            *dest24.offset(2 as isize) = 0x80 as BYTE;
        } else if v > 1.0f32 {
            *dest24.offset(0 as isize) = 0xff as BYTE;
            *dest24.offset(1 as isize) = 0xff as BYTE;
            *dest24.offset(2 as isize) = 0x7f as BYTE;
        } else if v < 0.0f32 {
            *dest24.offset(0 as isize) = (v * 8388608.0f32) as BYTE;
            *dest24.offset(1 as isize) = (v * 32768.0f32) as BYTE;
            *dest24.offset(2 as isize) = (v * 128.0f32) as BYTE;
        } else if v >= 0.0f32 {
            *dest24.offset(0 as isize) = (v * 8388608.0f32) as BYTE;
            *dest24.offset(1 as isize) = (v * 32768.0f32) as BYTE;
            *dest24.offset(2 as isize) = (v * 127.0f32) as BYTE;
        }

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize)
            as *mut ::core::ffi::c_void;
        src_advance(
            &raw mut src,
            src_stride,
            &raw mut count,
            &raw mut freqAcc,
            adj,
        );
    }
}

unsafe extern "C" fn convert_ieee_32_to_32(mut src: *const ::core::ffi::c_void, mut dst: *mut ::core::ffi::c_void, src_stride: UINT, dst_stride: UINT, mut count: INT, mut freqAcc: UINT, adj: UINT) {
    while count > 0 {
        let src_le: DWORD = le32!(*(src as *mut DWORD));
        let v: f32 = *(core::ptr::addr_of!(src_le) as *const f32);
        let d: *mut INT32 = dst as *mut INT32;

        if v < -1.0f32 {
            *d = (-2147483647 - 1) as INT32; /* silence warning */
        } else if v > 1.0f32 {
            *d = 2147483647 as INT32;
        } else {
            *d = (v * 2147483647.5f32 - 0.5f32) as INT32;
        }

        *d = le32!(*d);

        dst = (dst as *mut ::core::ffi::c_char).offset(dst_stride as isize) as *mut ::core::ffi::c_void;
        src_advance(
            core::ptr::addr_of_mut!(src),
            src_stride,
            core::ptr::addr_of_mut!(count),
            core::ptr::addr_of_mut!(freqAcc),
            adj,
        );
    }
}

unsafe extern "C" fn mix8(mut src: *mut i8, mut dst: *mut INT, mut len: u32) {
    while len != 0 {
        len -= 1;
        let fresh23 = src;
        src = src.offset(1);
        let fresh24 = dst;
        dst = dst.offset(1);
        *fresh24 += ((*fresh23 as u8 as i32) - (0x80 as BYTE as i32)) as i8 as i32;
    }
}

unsafe extern "C" fn mix16(mut src: *mut SHORT, mut dst: *mut INT, mut len: ::core::ffi::c_uint) {
    len /= 2;
    loop {
        let fresh0 = len;
        len = len.wrapping_sub(1);
        if !(fresh0 != 0) {
            break;
        }
        *dst += le16!(*src) as INT;
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

unsafe extern "C" fn mix24(mut src: *mut BYTE, mut dst: *mut INT, mut len: ::core::ffi::c_uint) {
    TRACE!("%p - %p %d\n", src, dst, len);
    len = len.wrapping_div(3 as ::core::ffi::c_uint);
    loop {
        let fresh19 = len;
        len = len.wrapping_sub(1);
        if !(fresh19 != 0) {
            break;
        }
        let mut field: DWORD = 0;
        field = ((*src.offset(2 as isize) as DWORD)
            << 16)
            .wrapping_add(
                (*src.offset(1 as isize) as DWORD) << 8,
            )
            .wrapping_add(*src.offset(0 as isize) as DWORD);
        if *src.offset(2 as isize) as ::core::ffi::c_int
            & 0x80
            != 0
        {
            field |= 0xFF000000 as DWORD;
        }
        let fresh20 = dst;
        dst = dst.offset(1);
        *fresh20 = (*fresh20 as DWORD).wrapping_add(field) as INT as INT;
        src = src.offset(1);
    }
}

unsafe extern "C" fn mix32(mut src: *mut INT, mut dst: *mut LONGLONG, mut len: ::core::ffi::c_uint) {
    len /= 4;
    while len != 0 {
        len = len.wrapping_sub(1);
        let fresh_src = src;
        src = src.offset(1);
        let fresh_dst = dst;
        dst = dst.offset(1);
        *fresh_dst += le32!(*fresh_src) as LONGLONG;
    }
}

unsafe extern "C" fn norm8(mut src: *mut i32, mut dst: *mut i8, mut len: u32) {
    loop {
        let fresh28 = len;
        len = len.wrapping_sub(1);
        if !(fresh28 != 0) {
            break;
        }
        *dst = (*src + 0x80) as i8;
        if *src < -0x80 {
            *dst = 0 as i8;
        } else if *src > 0x7f {
            *dst = 0xffu8 as i8;
        }
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

unsafe extern "C" fn norm16(mut src: *mut INT, mut dst: *mut SHORT, mut len: ::core::ffi::c_uint) {
    len = len.wrapping_div(2);
    loop {
        let fresh27 = len;
        len = len.wrapping_sub(1);
        if !(fresh27 != 0) {
            break;
        }
        *dst = le16!(*src) as SHORT;
        if *src <= -0x8000 {
            *dst = le16!(0x8000u16 as SHORT) as SHORT;
        } else if *src > 0x7fff {
            *dst = le16!((0x7fff) as SHORT) as SHORT;
        }
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

unsafe extern "C" fn norm24(mut src: *mut INT, mut dst: *mut BYTE, mut len: ::core::ffi::c_uint) {
    WINE_TRACE!("%p - %p %d\n", src, dst, len);
    len = len / (3 as ::core::ffi::c_uint);
    while len != 0 {
        len = len.wrapping_sub(1);
        if *src <= -(0x800000 as INT) {
            *dst.offset(0 as isize) = 0 as BYTE;
            *dst.offset(1 as isize) = 0 as BYTE;
            *dst.offset(2 as isize) = 0x80 as BYTE;
        } else if *src > (0x7fffff as INT) {
            *dst.offset(0 as isize) = 0xff as BYTE;
            *dst.offset(1 as isize) = 0xff as BYTE;
            *dst.offset(2 as isize) = 0x7f as BYTE;
        } else {
            *dst.offset(0 as isize) = *src as BYTE;
            *dst.offset(1 as isize) =
                (*src >> 8) as BYTE;
            *dst.offset(2 as isize) =
                (*src >> 16) as BYTE;
        }
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

unsafe extern "C" fn norm32(mut src: *mut LONGLONG, mut dst: *mut INT, mut len: ::core::ffi::c_uint) {
    len /= 4 as ::core::ffi::c_uint;
    loop {
        let fresh25 = len;
        len = len.wrapping_sub(1);
        if !(fresh25 != 0) {
            break;
        }
        *dst = le32!(*src as INT);
        if *src <= -(0x80000000 as ::core::ffi::c_uint as LONGLONG) {
            *dst = le32!(0x80000000 as ::core::ffi::c_uint as INT);
        } else if *src > 0x7fffffff as ::core::ffi::c_int as LONGLONG {
            *dst = le32!(0x7fffffff as ::core::ffi::c_int as INT);
        }
        dst = dst.offset(1);
        src = src.offset(1);
    }
}
