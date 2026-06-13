//! wstring/mbrtowc.c + wstring/wcrtomb.c — the two mbcs conversion leaves
//! (msvcrt @ DLL_EXPORT_VERSION < 0x600). mbstate_t is `int`.

extern "C" {
    fn ___mb_cur_max_func() -> i32;
    fn isleadbyte(c: i32) -> i32;
    fn ___lc_codepage_func() -> u32;
    fn wctomb(dst: *mut u8, ch: u16) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: *const u8,
        cbMultiByte: i32,
        lpWideCharStr: *mut u16,
        cchWideChar: i32,
    ) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    dst: *mut u16,
    str_: *const u8,
    n: usize,
    state: *mut i32,
) -> usize {
    let mut s: i32 = if !state.is_null() { *state } else { 0 };
    let mut tmpstr = [0u8; 2];
    let mut len: i32 = 0;

    if !dst.is_null() {
        *dst = 0;
    }
    if n == 0 || str_.is_null() || *str_ == 0 {
        return 0;
    }

    if ___mb_cur_max_func() == 1 {
        tmpstr[len as usize] = *str_;
        len += 1;
    } else if s == 0 && isleadbyte(*str_ as i32) != 0 {
        if n == 1 {
            s = *str_ as i32;
            len = -2;
        } else {
            tmpstr[0] = *str_;
            tmpstr[1] = *str_.add(1);
            len = 2;
        }
    } else if s == 0 {
        tmpstr[len as usize] = *str_;
        len += 1;
    } else {
        tmpstr[0] = s as u8;
        tmpstr[1] = *str_;
        len = 2;
        s = 0;
    }

    if len > 0 {
        let cch = if dst.is_null() { 0 } else { 1 };
        if MultiByteToWideChar(___lc_codepage_func(), 0, tmpstr.as_ptr(), len, dst, cch) == 0 {
            len = -1;
        }
    }

    if !state.is_null() {
        *state = s;
    }
    len as usize
}

#[no_mangle]
pub unsafe extern "C" fn wcrtomb(dst: *mut u8, ch: u16, s: *mut i32) -> usize {
    if !s.is_null() {
        *s = 0;
    }
    wctomb(dst, ch) as usize
}
