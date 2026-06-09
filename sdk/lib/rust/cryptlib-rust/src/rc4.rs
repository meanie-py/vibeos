//! rc4.rs — RC4 stream cipher (sdk/lib/cryptlib/rc4.c).
//! ABI: plain C functions (cdecl on all arches).
//!
//! FIXES applied:
//!   - `use core::ffi::c_void` removed (unused).
//!   - RC4_CONTEXT type pulled from crate::types.

use crate::types::*;

#[no_mangle]
pub unsafe extern "C" fn rc4_init(a4i: *mut RC4_CONTEXT, key: *const u8, keyLen: u32) {
    let mut keyIndex: u32 = 0;
    let mut stateIndex: u32 = 0;
    let mut i: u32;
    let mut a: u32;
    (*a4i).y = 0;
    (*a4i).x = (*a4i).y;
    i = 0;
    while i < 256 {
        (*a4i).state[i as usize] = i as u8;
        i = i.wrapping_add(1);
    }
    i = 0;
    while i < 256 {
        a = (*a4i).state[i as usize] as u32;
        stateIndex = stateIndex.wrapping_add((*key.offset(keyIndex as isize) as u32).wrapping_add(a));
        stateIndex &= 0xff;
        (*a4i).state[i as usize] = (*a4i).state[stateIndex as usize];
        (*a4i).state[stateIndex as usize] = a as u8;
        keyIndex = keyIndex.wrapping_add(1);
        if keyIndex >= keyLen {
            keyIndex = 0;
        }
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rc4_crypt(
    a4i: *mut RC4_CONTEXT,
    mut inoutString: *mut u8,
    length: u32,
) {
    let s = core::ptr::addr_of_mut!((*a4i).state) as *mut u8;
    let mut x = (*a4i).x as u32;
    let mut y = (*a4i).y as u32;
    let mut a: u32;
    let mut b: u32;

    let mut length = length;
    loop {
        let fresh0 = length;
        length = length.wrapping_sub(1);
        if fresh0 == 0 {
            break;
        }
        x = (x.wrapping_add(1)) & 0xff;
        a = *s.offset(x as isize) as u32;
        y = (y.wrapping_add(a)) & 0xff;
        b = *s.offset(y as isize) as u32;
        *s.offset(x as isize) = b as u8;
        *s.offset(y as isize) = a as u8;
        let fresh1 = inoutString;
        inoutString = inoutString.offset(1);
        *fresh1 ^= *s.offset((a.wrapping_add(b) & 0xff) as isize);
    }

    (*a4i).x = x as u8;
    (*a4i).y = y as u8;
}
