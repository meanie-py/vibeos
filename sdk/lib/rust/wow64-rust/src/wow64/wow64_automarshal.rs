use crate::prelude::*;

/// C's `__attribute__((aligned(16)))` on a local byte buffer.
#[repr(C, align(16))]
struct Wow64Aligned16<const N: usize>([u8; N]);
use core::ffi::c_void;

unsafe extern "C" fn WowAmCpy(mut d: *mut u8, mut s: *const u8, mut n: u32) {
    while n != 0 {
        n = n.wrapping_sub(1);
        *d = *s;
        d = d.add(1);
        s = s.add(1);
    }
}

unsafe extern "C" fn WowAmZero(mut d: *mut ::core::ffi::c_uchar, mut n: ::core::ffi::c_uint) {
    loop {
        let fresh18 = n;
        n = n.wrapping_sub(1);
        if !(fresh18 != 0) {
            break;
        }
        let fresh19 = d;
        d = d.offset(1);
        *fresh19 = 0;
    }
}

unsafe extern "C" fn WowAmSx(d: *mut u8, s: *const u8) {
    let mut v: LONG = 0;
    let mut w: LONGLONG = 0;
    WowAmCpy(core::ptr::addr_of_mut!(v) as *mut u8, s, 4 as u32);
    w = v as LONGLONG;
    WowAmCpy(d, core::ptr::addr_of!(w) as *const u8, 8 as u32);
}

unsafe extern "C" fn WowAmNx(d: *mut u8, s: *const u8) {
    WowAmCpy(d, s, 4);
}

unsafe extern "C" fn WowAmSizeIn(
    d: *mut u8,
    s: *const u8,
    g32: ULONG,
    h64: ULONG,
) {
    let mut v: ULONG = 0;
    WowAmCpy(
        core::ptr::addr_of_mut!(v) as *mut u8,
        s,
        4,
    );
    if v == g32 {
        v = h64;
    }
    WowAmCpy(
        d,
        core::ptr::addr_of!(v) as *const u8,
        4,
    );
}

unsafe extern "C" fn WowAmSizeOut(d: *mut u8, s: *const u8, g32: ULONG, h64: ULONG) {
    let mut v: ULONG = 0;
    WowAmCpy(core::ptr::addr_of_mut!(v) as *mut u8, s, 4);
    if v == h64 {
        v = g32;
    }
    WowAmCpy(d, core::ptr::addr_of!(v) as *const u8, 4);
}

unsafe extern "C" fn WowAmStrEq(mut a: *const i8, mut b: *const i8) -> i32 {
    while *a as i32 != 0 && *a as i32 == *b as i32 {
        a = a.offset(1);
        b = b.offset(1);
    }
    return (*a as i32 == *b as i32) as i32;
}

unsafe extern "C" fn WowAmIn__UNICODE_STRING(s: *const u8, d: *mut u8) {
    WowAmZero(d, 16 as u32);
    WowAmCpy(d.offset(0 as isize), s.offset(0 as isize), 2 as u32); /* Length */
    WowAmCpy(d.offset(2 as isize), s.offset(2 as isize), 2 as u32); /* MaximumLength */
    WowAmSx(d.offset(8 as isize), s.offset(4 as isize)); /* Buffer */
}

unsafe extern "C" fn WowAmOut__UNICODE_STRING(s: *const u8, d: *mut u8) {
    WowAmCpy(d.add(0), s.add(0), 2); /* Length */
    WowAmCpy(d.add(2), s.add(2), 2); /* MaximumLength */
    WowAmNx(d.add(4), s.add(8)); /* Buffer */
}

unsafe extern "C" fn WowAmIn_tagCOMBOBOXINFO(s: *const u8, d: *mut u8) {
    WowAmZero(d, 64);
    WowAmSizeIn(d.add(0), s.add(0), 52, 64); /* cbSize */
    WowAmCpy(d.add(4), s.add(4), 16); /* rcItem */
    WowAmCpy(d.add(20), s.add(20), 16); /* rcButton */
    WowAmCpy(d.add(36), s.add(36), 4); /* stateButton */
    WowAmSx(d.add(40), s.add(40)); /* hwndCombo */
    WowAmSx(d.add(48), s.add(44)); /* hwndItem */
    WowAmSx(d.add(56), s.add(48)); /* hwndList */
}

unsafe extern "C" fn WowAmOut_tagCOMBOBOXINFO(s: *const u8, d: *mut u8) {
    WowAmSizeOut(d.add(0 as usize), s.add(0 as usize), 52 as ULONG, 64 as ULONG); // cbSize
    WowAmCpy(d.add(4 as usize), s.add(4 as usize), 16 as u32); // rcItem
    WowAmCpy(d.add(20 as usize), s.add(20 as usize), 16 as u32); // rcButton
    WowAmCpy(d.add(36 as usize), s.add(36 as usize), 4 as u32); // stateButton
    WowAmNx(d.add(40 as usize), s.add(40 as usize)); // hwndCombo
    WowAmNx(d.add(44 as usize), s.add(48 as usize)); // hwndItem
    WowAmNx(d.add(48 as usize), s.add(56 as usize)); // hwndList
}

unsafe extern "C" fn WowAmIn_tagCURSORINFO(s: *const u8, d: *mut u8) {
    WowAmZero(d, 24 as u32);
    WowAmSizeIn(d.add(0), s.add(0), 20 as u32, 24 as u32); /* cbSize */
    WowAmCpy(d.add(4), s.add(4), 4 as u32); /* flags */
    WowAmSx(d.add(8), s.add(8)); /* hCursor */
    WowAmCpy(d.add(16), s.add(12), 8 as u32); /* ptScreenPos */
}

unsafe extern "C" fn WowAmOut_tagCURSORINFO(s: *const ::core::ffi::c_uchar, d: *mut ::core::ffi::c_uchar) {
    WowAmSizeOut(d.offset(0 as isize), s.offset(0 as isize), 20 as ULONG, 24 as ULONG); /* cbSize */
    WowAmCpy(d.offset(4 as isize), s.offset(4 as isize), 4 as ::core::ffi::c_uint); /* flags */
    WowAmNx(d.offset(8 as isize), s.offset(8 as isize)); /* hCursor */
    WowAmCpy(d.offset(12 as isize), s.offset(16 as isize), 8 as ::core::ffi::c_uint); /* ptScreenPos */
}

unsafe extern "C" fn WowAmIn_tagGUITHREADINFO(s: *const u8, d: *mut u8) {
    WowAmZero(d, 72 as ::core::ffi::c_uint);
    WowAmSizeIn(
        d.add(0 as usize),
        s.add(0 as usize),
        48 as ULONG,
        72 as ULONG,
    ); /* cbSize */
    WowAmCpy(
        d.add(4 as usize),
        s.add(4 as usize),
        4 as ::core::ffi::c_uint,
    ); /* flags */
    WowAmSx(
        d.add(8 as usize),
        s.add(8 as usize),
    ); /* hwndActive */
    WowAmSx(
        d.add(16 as usize),
        s.add(12 as usize),
    ); /* hwndFocus */
    WowAmSx(
        d.add(24 as usize),
        s.add(16 as usize),
    ); /* hwndCapture */
    WowAmSx(
        d.add(32 as usize),
        s.add(20 as usize),
    ); /* hwndMenuOwner */
    WowAmSx(
        d.add(40 as usize),
        s.add(24 as usize),
    ); /* hwndMoveSize */
    WowAmSx(
        d.add(48 as usize),
        s.add(28 as usize),
    ); /* hwndCaret */
    WowAmCpy(
        d.add(56 as usize),
        s.add(32 as usize),
        16 as ::core::ffi::c_uint,
    ); /* rcCaret */
}

unsafe extern "C" fn WowAmOut_tagGUITHREADINFO(s: *const u8, d: *mut u8) {
    WowAmSizeOut(d.add(0), s.add(0), 48 as ULONG, 72 as ULONG); /* cbSize */
    WowAmCpy(d.add(4), s.add(4), 4 as u32); /* flags */
    WowAmNx(d.add(8), s.add(8)); /* hwndActive */
    WowAmNx(d.add(12), s.add(16)); /* hwndFocus */
    WowAmNx(d.add(16), s.add(24)); /* hwndCapture */
    WowAmNx(d.add(20), s.add(32)); /* hwndMenuOwner */
    WowAmNx(d.add(24), s.add(40)); /* hwndMoveSize */
    WowAmNx(d.add(28), s.add(48)); /* hwndCaret */
    WowAmCpy(d.add(32), s.add(56), 16 as u32); /* rcCaret */
}

unsafe extern "C" fn WowAmIn_tagTRACKMOUSEEVENT(s: *const u8, d: *mut u8) {
    WowAmZero(d, 24 as u32);
    WowAmSizeIn(d.add(0), s.add(0), 16 as ULONG, 24 as ULONG); /* cbSize */
    WowAmCpy(d.add(4), s.add(4), 4 as u32); /* dwFlags */
    WowAmSx(d.add(8), s.add(8)); /* hwndTrack */
    WowAmCpy(d.add(16), s.add(12), 4 as u32); /* dwHoverTime */
}

unsafe extern "C" fn WowAmOut_tagTRACKMOUSEEVENT(s: *const u8, d: *mut u8) {
    WowAmSizeOut(d.add(0 as usize), s.add(0 as usize), 16 as ULONG, 24 as ULONG); /* cbSize */
    WowAmCpy(d.add(4 as usize), s.add(4 as usize), 4 as u32); /* dwFlags */
    WowAmNx(d.add(8 as usize), s.add(8 as usize)); /* hwndTrack */
    WowAmCpy(d.add(12 as usize), s.add(16 as usize), 4 as u32); /* dwHoverTime */
}

unsafe extern "C" fn WowAmThunk_NtUserChangeDisplaySettings(a: *mut ULONG) -> NTSTATUS {
    // `unsigned char b0[16] __attribute__((aligned(16)))`: the array keeps its size,
    // the storage takes the alignment.
    let mut b0_storage = Wow64Aligned16([0u8; 16]);
    let b0: &mut [u8; 16] = &mut b0_storage.0;
    if false {
    }
    let mut g0: *const u8 = *a.offset(0 as isize) as ULONG_PTR as *const u8;
    let mut r: ULONGLONG = 0;
    let _ = a;
    if !g0.is_null() {
        WowAmIn__UNICODE_STRING(g0, b0.as_mut_ptr());
    }
    r = (NtUserChangeDisplaySettings(
        if !g0.is_null() {
            ((b0.as_mut_ptr() as ULONG_PTR as ULONGLONG) as *mut _UNICODE_STRING)
        } else {
            ((0 as ULONGLONG) as *mut _UNICODE_STRING)
        },
        ((*a.offset(1 as isize) as LONG as LONGLONG as ULONGLONG) as *mut _devicemodeW),
        (*a.offset(2 as isize) as LONG as LONGLONG as ULONGLONG) as u32,
        ((*a.offset(3 as isize) as LONG as LONGLONG as ULONGLONG) as *mut c_void),
    )) as u64;
    if !g0.is_null() {
        WowAmOut__UNICODE_STRING(
            b0.as_mut_ptr(),
            *a.offset(0 as isize) as ULONG_PTR as *mut u8,
        );
    }
    return r as ULONG as NTSTATUS;
}

unsafe extern "C" fn WowAmThunk_NtUserGetComboBoxInfo(a: *mut ULONG) -> NTSTATUS {
    // `unsigned char b1[64] __attribute__((aligned(16)))`: the array keeps its size,
    // the storage takes the alignment.
    let mut b1_storage = Wow64Aligned16([0u8; 64]);
    let b1: &mut [u8; 64] = &mut b1_storage.0;
    let g1: *const u8 = *a.offset(1 as isize) as ULONG_PTR as *const u8;
    let r: ULONGLONG;
    let _ = a;
    if !g1.is_null() {
        WowAmIn_tagCOMBOBOXINFO(g1, b1.as_mut_ptr());
    }
    r = (NtUserGetComboBoxInfo(
        ((*a.offset(0 as isize) as LONG as LONGLONG as ULONGLONG) as *mut HWND__),
        if !g1.is_null() {
            ((b1.as_mut_ptr() as ULONGLONG) as *mut tagCOMBOBOXINFO)
        } else {
            ((0 as ULONGLONG) as *mut tagCOMBOBOXINFO)
        },
    )) as u64;
    if !g1.is_null() {
        WowAmOut_tagCOMBOBOXINFO(
            b1.as_mut_ptr(),
            *a.offset(1 as isize) as ULONG_PTR as *mut u8,
        );
    }
    return r as ULONG as NTSTATUS;
}

unsafe extern "C" fn WowAmThunk_NtUserGetCursorInfo(a: *mut ULONG) -> NTSTATUS {
    // `unsigned char b0[24] __attribute__((aligned(16)))`: the array keeps its size,
    // the storage takes the alignment.
    let mut b0_storage = Wow64Aligned16([0u8; 24]);
    let b0: &mut [u8; 24] = &mut b0_storage.0;
    let mut g0: *const u8 = *a.add(0) as ULONG_PTR as *const u8;
    let mut r: ULONGLONG = 0;
    let _ = a;
    if !g0.is_null() {
        WowAmIn_tagCURSORINFO(g0, b0.as_mut_ptr());
    }
    r = (NtUserGetCursorInfo(if !g0.is_null() {
        ((b0.as_mut_ptr() as ULONG_PTR) as *mut tagCURSORINFO)
    } else {
        core::ptr::null_mut()
    })) as u64;
    if !g0.is_null() {
        WowAmOut_tagCURSORINFO(b0.as_mut_ptr(), *a.add(0) as ULONG_PTR as *mut u8);
    }
    return r as ULONG as NTSTATUS;
}

unsafe extern "C" fn WowAmThunk_NtUserGetGUIThreadInfo(a: *mut ULONG) -> NTSTATUS {
    // `unsigned char b1[72] __attribute__((aligned(16)))`: the array keeps its size,
    // the storage takes the alignment.
    let mut b1_storage = Wow64Aligned16([0u8; 72]);
    let b1: &mut [u8; 72] = &mut b1_storage.0;
    let g1: *const u8 = *a.offset(1 as isize) as ULONG_PTR as *const u8;
    let mut r: ULONGLONG = 0;
    if !g1.is_null() {
        WowAmIn_tagGUITHREADINFO(g1, b1.as_mut_ptr());
    }
    r = (NtUserGetGUIThreadInfo(
        (*a.offset(0 as isize) as LONG as LONGLONG as ULONGLONG) as u32,
        if !g1.is_null() {
            ((b1.as_mut_ptr() as ULONG_PTR as ULONGLONG) as *mut tagGUITHREADINFO)
        } else {
            ((0 as ULONGLONG) as *mut tagGUITHREADINFO)
        },
    )) as u64;
    if !g1.is_null() {
        WowAmOut_tagGUITHREADINFO(
            b1.as_mut_ptr(),
            *a.offset(1 as isize) as ULONG_PTR as *mut u8,
        );
    }
    return r as ULONG as NTSTATUS;
}

unsafe extern "C" fn WowAmThunk_NtUserTrackMouseEvent(a: *mut ULONG) -> NTSTATUS {
    // `unsigned char b0[24] __attribute__((aligned(16)))`: the array keeps its size,
    // the storage takes the alignment.
    let mut b0_storage = Wow64Aligned16([0u8; 24]);
    let b0: &mut [u8; 24] = &mut b0_storage.0;
    let mut g0: *const u8 = *a.offset(0 as isize) as ULONG_PTR as *const u8;
    let mut r: ULONGLONG = 0;
    let _ = a;
    if !g0.is_null() {
        WowAmIn_tagTRACKMOUSEEVENT(g0, b0.as_mut_ptr());
    }
    r = (NtUserTrackMouseEvent(if !g0.is_null() {
        ((b0.as_mut_ptr() as ULONG_PTR as ULONGLONG) as *mut tagTRACKMOUSEEVENT)
    } else {
        ((0 as ULONGLONG) as *mut tagTRACKMOUSEEVENT)
    })) as u64;
    if !g0.is_null() {
        WowAmOut_tagTRACKMOUSEEVENT(
            b0.as_mut_ptr(),
            *a.offset(0 as isize) as ULONG_PTR as *mut u8,
        );
    }
    return r as ULONG as NTSTATUS;
}
#[no_mangle]
pub unsafe extern "C" fn Wow64AutoMarshalDispatch(Name: *const i8, Arguments: *mut ULONG) -> NTSTATUS {
    if WowAmStrEq(Name, b"NtUserChangeDisplaySettings\0".as_ptr() as *const i8) != 0 {
        return WowAmThunk_NtUserChangeDisplaySettings(Arguments);
    }
    if WowAmStrEq(Name, b"NtUserGetComboBoxInfo\0".as_ptr() as *const i8) != 0 {
        return WowAmThunk_NtUserGetComboBoxInfo(Arguments);
    }
    if WowAmStrEq(Name, b"NtUserGetCursorInfo\0".as_ptr() as *const i8) != 0 {
        return WowAmThunk_NtUserGetCursorInfo(Arguments);
    }
    if WowAmStrEq(Name, b"NtUserGetGUIThreadInfo\0".as_ptr() as *const i8) != 0 {
        return WowAmThunk_NtUserGetGUIThreadInfo(Arguments);
    }
    if WowAmStrEq(Name, b"NtUserTrackMouseEvent\0".as_ptr() as *const i8) != 0 {
        return WowAmThunk_NtUserTrackMouseEvent(Arguments);
    }
    return WOW64_AUTOMARSHAL_UNIMPL;
}
