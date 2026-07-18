use crate::prelude::*;
use core::ffi::c_void;


// ---- strlcpy ----
pub unsafe fn strlcpy(d: *mut u8, s: *const u8, bufsize: usize) -> usize {
    let len = strlen((s) as *const i8);
    let ret = len;
    if bufsize > 0 {
        let mut copy_len = len;
        if len >= bufsize {
            copy_len = bufsize - 1;
        }
        core::ptr::copy_nonoverlapping(s, d, copy_len);
        *d.add(copy_len) = 0;
    }
    ret
}

// ---- arc4random ----
pub unsafe fn arc4random() -> u32 {
    static mut did_srand: i32 = 0;
    if did_srand == 0 {
        srand(0u32);
        did_srand = 1;
    }
    let ret: u32 = ((rand() as u32) << 10) ^ (rand() as u32);
    ret
}

// ---- inet_aton ----
#[no_mangle]
pub unsafe fn inet_aton(mut cp: *const u8, inp: *mut in_addr) -> i32 {
    let mut Octets: [u32; 4] = [0; 4];
    let mut i: u32 = 0;

    if cp.is_null() {
        return 0;
    }

    loop {
        let c = *cp;
        if c == 0 {
            break;
        }
        cp = cp.add(1);

        if c == b'.' {
            i += 1;
            if i >= 4 {
                return 0;
            }
            continue;
        }

        if c < b'0' || c > b'9' {
            return 0;
        }

        let idx = i as usize;
        Octets[idx] = Octets[idx].wrapping_mul(10);
        Octets[idx] = Octets[idx].wrapping_add((c - b'0') as u32);

        if Octets[idx] > 255 {
            return 0;
        }
    }

    (*inp).S_un.S_addr = (Octets[3] << 24) | (Octets[2] << 16) | (Octets[1] << 8) | Octets[0];
    1
}
