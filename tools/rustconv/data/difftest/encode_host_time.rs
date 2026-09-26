//! Deterministic NtQuerySystemTime, mirroring encode_host_time.h exactly.
use core::sync::atomic::{AtomicU32, Ordering};

const TIMES: [i64; 4] = [0x1200, 0xAB0000, 0xFF, 0x7FFF_FFFF_FFFF_FFFF];
const STATUS: [i32; 4] = [0, 0, 0, 0xC000_0001u32 as i32];
static CALLS: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
pub unsafe extern "system" fn NtQuerySystemTime(time: *mut i64) -> i32 {
    let i = (CALLS.fetch_add(1, Ordering::Relaxed) & 3) as usize;
    unsafe { *time = TIMES[i] };
    STATUS[i]
}
