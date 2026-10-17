use winedebug_rust::{TRACE, WARN};
// The crate's own functions: the definition is the declaration.
use crate::client::handle::CloseHandle;
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
// HeapAlloc/HeapFree are pure ntdll forwarders in kernel32.def (no @N symbol in this
// module) — they come from the nls_subst.rs Rust wrappers over RtlAllocateHeap/
// RtlFreeHeap, not from a linkable export. GetProcessHeap IS a real @0 export (below).
use crate::string::nls_subst::{HeapAlloc, HeapFree};

extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: *const CHAR, lpUsedDefaultChar: *mut BOOL, ) -> i32;
    // ---- cohab glue: kernel32 Rust exports this module calls (resolved in-crate / via spec) ----
    fn _hread(hFile: HFILE, lpBuffer: LPVOID, lBytes: LONG) -> LONG;
    fn _hwrite(hFile: HFILE, lpBuffer: LPCSTR, lBytes: LONG) -> LONG;
    fn _llseek(hFile: HFILE, lOffset: LONG, iOrigin: c_int) -> LONG;
    fn _lclose(hFile: HFILE) -> HFILE;
    fn OpenFile(lpFileName: LPCSTR, lpReOpenBuff: LPOFSTRUCT, uStyle: UINT) -> HFILE;
    fn MultiByteToWideChar(CodePage: UINT, Flags: DWORD, MultiByteString: LPCSTR, MultiByteCount: INT, WideCharString: LPWSTR, WideCharCount: INT) -> INT;
    fn GetFileTime(hFile: HANDLE, lpCreationTime: LPFILETIME, lpLastAccessTime: LPFILETIME, lpLastWriteTime: LPFILETIME) -> BOOL;
    fn SetFileTime(hFile: HANDLE, lpCreationTime: *const FILETIME, lpLastAccessTime: *const FILETIME, lpLastWriteTime: *const FILETIME) -> BOOL;
}

// ---- cohab glue: Wine debug macros (no-ops, as in the other ported modules) ----

// ---- cohab glue: LZ error codes (sdk/include/psdk/lzexpand.h) ----
pub const LZERROR_BADINHANDLE:  INT = -1;
pub const LZERROR_BADOUTHANDLE: INT = -2;
pub const LZERROR_READ:         INT = -3;
pub const LZERROR_WRITE:        INT = -4;
pub const LZERROR_GLOBALLOC:    INT = -5;
pub const LZERROR_GLOBLOCK:     INT = -6;
pub const LZERROR_BADVALUE:     INT = -7;
pub const LZERROR_UNKNOWNALG:   INT = -8;

// ---- cohab glue: HandleToLong/LongToHandle FORCEINLINE intrinsics (winnt.h) ----
#[inline]
unsafe fn LongToHandle(l_long: i32) -> HANDLE {
    l_long as isize as HANDLE
}

// ---- cohab glue: lzexpand.c function-like macros, as inline fns ----
//   _lread/_lwrite are spec forwarders to _hread/_hwrite (no static symbol) — wrap them.
#[inline]
unsafe fn _lread(hFile: HFILE, lpBuffer: LPVOID, uBytes: c_int) -> i32 {
    _hread(hFile, lpBuffer, uBytes as LONG) as i32
}
#[inline]
unsafe fn _lwrite(hFile: HFILE, lpBuffer: *mut CHAR, uBytes: LONG) -> i32 {
    _hwrite(hFile, lpBuffer as LPCSTR, uBytes) as i32
}
#[inline]
unsafe fn IS_LZ_HANDLE(h: HFILE) -> bool {
    h >= LZ_MIN_HANDLE && h < LZ_MIN_HANDLE + MAX_LZSTATES as HFILE
}
#[inline]
unsafe fn GET_LZ_STATE(h: HFILE) -> *mut lzstate {
    if IS_LZ_HANDLE(h) {
        let arr = core::ptr::addr_of!(lzstates) as *const *mut lzstate;
        *arr.add((h - LZ_MIN_HANDLE) as usize)
    } else {
        core::ptr::null_mut()
    }
}
#[inline]
unsafe fn GET(lzs: *mut lzstate, b: *mut BYTE) -> c_int {
    _lzget(lzs, b)
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut gDebugChannel: ULONG = 200 as ULONG;

static mut LZMagic: [BYTE; 8] = [
    'S' as i32 as BYTE,
    'Z' as i32 as BYTE,
    'D' as i32 as BYTE,
    'D' as i32 as BYTE,
    0x88 as ::core::ffi::c_int as BYTE,
    0xf0 as ::core::ffi::c_int as BYTE,
    0x27 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
];

static mut lzstates: [*mut lzstate; 16] = [::core::ptr::null::<lzstate>() as *mut lzstate; 16];

#[repr(C)]
pub struct lzfileheader {
    pub magic: [BYTE; LZ_MAGIC_LEN],
    pub compressiontype: BYTE,
    pub lastchar: CHAR,
    pub reallength: DWORD,
}

#[repr(C)]
pub struct lzstate {
    pub realfd: HFILE,
    pub lastchar: CHAR,
    pub reallength: DWORD,
    pub realcurrent: DWORD,
    pub realwanted: DWORD,
    pub table: [BYTE; LZ_TABLE_SIZE],
    pub curtabent: UINT,
    pub stringlen: BYTE,
    pub stringpos: DWORD,
    pub bytetype: WORD,
    pub get: *mut BYTE,
    pub getcur: DWORD,
    pub getlen: DWORD,
}

pub const MAX_LZSTATES: usize = 16;

pub const LZ_MIN_HANDLE: HFILE = 0x400;
pub const LZ_MAGIC_LEN: usize = 8;
pub const LZ_HEADER_LEN: usize = 14;
pub const LZ_TABLE_SIZE: usize = 0x1000;
pub const GETLEN: usize = 2048;
pub const HFILE_ERROR: HFILE = -1i64 as HFILE;
unsafe fn _lzget(lzs: *mut lzstate, b: *mut BYTE) -> c_int {
    if (*lzs).getcur < (*lzs).getlen {
        let fresh3 = (*lzs).getcur;
        (*lzs).getcur = (*lzs).getcur.wrapping_add(1);
        *b = *(*lzs).get.offset(fresh3 as isize);
        return 1;
    } else {
        let mut ret: c_int = _lread(
            (*lzs).realfd,
            (*lzs).get as LPVOID,
            GETLEN as c_int,
        ) as c_int;
        if ret == HFILE_ERROR {
            return HFILE_ERROR;
        }
        if ret == 0 {
            return 0;
        }
        (*lzs).getlen = ret as DWORD;
        (*lzs).getcur = 1;
        *b = *(*lzs).get;
        return 1;
    }
}
unsafe fn read_header(fd: HFILE, head: *mut lzfileheader) -> INT {
    let mut buf: [BYTE; LZ_HEADER_LEN] = [0; LZ_HEADER_LEN];

    if _llseek(fd, 0, SEEK_SET as c_int) == -1 {
        return LZERROR_BADINHANDLE;
    }

    if _lread(fd, buf.as_mut_ptr() as LPVOID, LZ_HEADER_LEN as c_int) < LZ_HEADER_LEN as LONG {
        return 0;
    }

    memcpy(
        (*head).magic.as_mut_ptr() as *mut core::ffi::c_void,
        buf.as_ptr() as *const core::ffi::c_void,
        LZ_MAGIC_LEN,
    );
    memcpy(
        &raw mut (*head).compressiontype as *mut core::ffi::c_void,
        buf.as_ptr().offset(LZ_MAGIC_LEN as isize) as *const core::ffi::c_void,
        1,
    );
    memcpy(
        &raw mut (*head).lastchar as *mut core::ffi::c_void,
        buf.as_ptr().offset(LZ_MAGIC_LEN as isize).offset(1) as *const core::ffi::c_void,
        1,
    );
    memcpy(
        &raw mut (*head).reallength as *mut core::ffi::c_void,
        buf.as_ptr().offset(LZ_MAGIC_LEN as isize).offset(2) as *const core::ffi::c_void,
        4,
    );

    if memcmp(
        (*head).magic.as_ptr() as *const core::ffi::c_void,
        (*(&raw const LZMagic)).as_ptr() as *const core::ffi::c_void,
        LZ_MAGIC_LEN,
    ) != 0
    {
        return 0;
    }

    if (*head).compressiontype != b'A' as BYTE {
        return LZERROR_UNKNOWNALG;
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn LZStart() -> i32 {
    TRACE!("(void)\n");
    return 1;
}
#[no_mangle]
pub unsafe extern "system" fn LZInit(hfSrc: HFILE) -> HFILE {
    let mut head: lzfileheader = core::mem::zeroed();
    let mut lzs: *mut lzstate = core::ptr::null_mut();
    let mut i: i32 = 0;
    let mut ret: i32 = 0;

    TRACE!(b"(%d)\n\0".as_ptr() as LPCSTR, hfSrc);

    ret = read_header(hfSrc, &mut head as *mut lzfileheader) as i32;
    if ret <= 0 {
        _llseek(hfSrc, 0, SEEK_SET as c_int);
        return if ret != 0 { ret as HFILE } else { hfSrc };
    }

    i = 0;
    while (i as usize) < MAX_LZSTATES {
        if lzstates[i as usize].is_null() {
            break;
        }
        i += 1;
    }
    if i as usize == MAX_LZSTATES {
        return LZERROR_GLOBALLOC as HFILE;
    }

    lzs = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<lzstate>(),
    ) as *mut lzstate;
    lzstates[i as usize] = lzs;
    if lzs.is_null() {
        return LZERROR_GLOBALLOC as HFILE;
    }

    (*lzs).realfd = hfSrc;
    (*lzs).lastchar = head.lastchar;
    (*lzs).reallength = head.reallength;

    (*lzs).get = HeapAlloc(GetProcessHeap(), 0, GETLEN) as *mut BYTE;
    (*lzs).getlen = 0;
    (*lzs).getcur = 0;

    if (*lzs).get.is_null() {
        HeapFree(GetProcessHeap(), 0, lzs as LPVOID);
        lzstates[i as usize] = core::ptr::null_mut();
        return LZERROR_GLOBALLOC as HFILE;
    }

    memset(
        (*lzs).table.as_mut_ptr() as *mut core::ffi::c_void,
        ' ' as i32,
        LZ_TABLE_SIZE,
    );
    (*lzs).curtabent = 0xff0;

    return (LZ_MIN_HANDLE + i as HFILE) as HFILE;
}
#[no_mangle]
pub unsafe extern "system" fn LZDone() {
    TRACE!(b"(void)\n\0" as *const u8 as *const ::core::ffi::c_char);
}

#[no_mangle]
pub unsafe extern "system" fn GetExpandedNameA(in_: LPSTR, out: LPSTR) -> INT {
    let mut head: lzfileheader = core::mem::zeroed();
    let mut fd: HFILE = 0;
    let mut ofs: OFSTRUCT = core::mem::zeroed();
    let mut fnislowercased: INT = 0;
    let mut ret: INT = 0;
    let mut len: INT = 0;
    let mut s: LPSTR = core::ptr::null_mut();
    let mut t: LPSTR = core::ptr::null_mut();

    TRACE!(b"(%s)\n\0".as_ptr() as LPCSTR, in_);
    fd = OpenFile(in_ as LPCSTR, &raw mut ofs, OF_READ);
    if fd == HFILE_ERROR {
        return LZERROR_BADINHANDLE as INT16 as INT;
    }
    strcpy(out as *mut i8, in_ as *const i8);
    ret = read_header(fd, &raw mut head);
    if ret <= 0 {
        _lclose(fd);
        return 1;
    }

    s = out;
    loop {
        t = strpbrk(s as *const i8, b"/\\:\0".as_ptr() as *const i8) as LPSTR;
        if t.is_null() {
            break;
        }
        s = t.offset(1);
    }

    if *s == 0 {
        WARN!(b"Specified a directory or what? (%s)\n\0".as_ptr() as LPCSTR, in_);
        _lclose(fd);
        return 1;
    }

    fnislowercased = 1;
    t = s.offset(strlen(s as *const i8) as isize).offset(-1);
    while t >= out {
        if isalpha(*t as i32) == 0 {
            t = t.offset(-1);
        } else {
            fnislowercased = islower(*t as i32) as INT;
            break;
        }
    }

    if isalpha(head.lastchar as i32) != 0 {
        if fnislowercased != 0 {
            head.lastchar = tolower(head.lastchar as i32) as i8;
        } else {
            head.lastchar = toupper(head.lastchar as i32) as i8;
        }
    }

    t = strchr(s as *const i8, '.' as i32) as LPSTR;
    if !t.is_null() {
        if *t.offset(1) as i32 == '\0' as i32 {
            *t.offset(0) = '\0' as i32 as i8;
        } else {
            len = (strlen(t as *const i8) as u64).wrapping_sub(1) as INT;
            if *t.offset(len as isize) as i32 == '_' as i32 {
                *t.offset(len as isize) = head.lastchar;
            }
        }
    }

    _lclose(fd);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetExpandedNameW(in_: LPWSTR, out: LPWSTR) -> INT {
    let mut ret: INT = 0;
    let len: DWORD = WideCharToMultiByte(
        CP_ACP,
        0,
        in_,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null(),
        core::ptr::null_mut(),
    ) as DWORD;
    let xin: LPSTR = HeapAlloc(GetProcessHeap(), 0, len as usize) as LPSTR;
    let xout: LPSTR = HeapAlloc(GetProcessHeap(), 0, len.wrapping_add(3) as usize) as LPSTR;
    WideCharToMultiByte(CP_ACP, 0, in_, -1, xin, len as i32, core::ptr::null(), core::ptr::null_mut());
    ret = GetExpandedNameA(xin, xout);
    if ret > 0 {
        MultiByteToWideChar(
            CP_ACP,
            0,
            xout,
            -1,
            out,
            strlenW(in_).wrapping_add(4) as i32,
        );
    }
    HeapFree(GetProcessHeap(), 0, xin as LPVOID);
    HeapFree(GetProcessHeap(), 0, xout as LPVOID);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn LZRead(fd: HFILE, vbuf: LPSTR, toread: INT) -> INT {
    let mut howmuch: i32 = 0;
    let mut b: BYTE = 0;
    let mut buf: *mut BYTE = core::ptr::null_mut();
    let mut lzs: *mut lzstate = core::ptr::null_mut();

    buf = vbuf as LPBYTE;
    TRACE!("(%d,%p,%d)\n", fd, buf, toread);

    howmuch = toread;
    lzs = GET_LZ_STATE(fd);
    if lzs.is_null() {
        return _lread(fd, buf as LPVOID, toread as c_int);
    }

    if (*lzs).realcurrent != (*lzs).realwanted {
        if (*lzs).realcurrent > (*lzs).realwanted {
            _llseek((*lzs).realfd, LZ_HEADER_LEN as i32, SEEK_SET as c_int);
            (*lzs).getcur = (*lzs).getlen;
            (*lzs).realcurrent = 0;
            (*lzs).bytetype = 0;
            (*lzs).stringlen = 0;
            core::ptr::write_bytes((*lzs).table.as_mut_ptr(), b' ', LZ_TABLE_SIZE);
            (*lzs).curtabent = 0xFF0;
        }
        while (*lzs).realcurrent < (*lzs).realwanted {
            if (*lzs).stringlen != 0 {
                b = (*lzs).table[(*lzs).stringpos as usize];
                (*lzs).stringpos = ((*lzs).stringpos.wrapping_add(1)) & 0xFFF;
                (*lzs).stringlen = (*lzs).stringlen.wrapping_sub(1);
            } else {
                if ((*lzs).bytetype as i32 & 0x100) == 0 {
                    if 1 != GET(lzs, &mut b) {
                        return toread - howmuch;
                    }
                    (*lzs).bytetype = (b as i32 | 0xFF00) as u16;
                }
                if ((*lzs).bytetype as i32 & 1) != 0 {
                    if 1 != GET(lzs, &mut b) {
                        return toread - howmuch;
                    }
                } else {
                    let mut b1: BYTE = 0;
                    let mut b2: BYTE = 0;
                    if 1 != GET(lzs, &mut b1) {
                        return toread - howmuch;
                    }
                    if 1 != GET(lzs, &mut b2) {
                        return toread - howmuch;
                    }
                    (*lzs).stringpos = (b1 as i32 | ((b2 as i32 & 0xF0) << 4)) as u32;
                    (*lzs).stringlen = ((b2 as i32 & 0xF) + 2) as u8;
                    b = (*lzs).table[(*lzs).stringpos as usize];
                    (*lzs).stringpos = ((*lzs).stringpos.wrapping_add(1)) & 0xFFF;
                }
                (*lzs).bytetype = ((*lzs).bytetype as i32 >> 1) as u16;
            }
            let idx = (*lzs).curtabent;
            (*lzs).curtabent = (*lzs).curtabent.wrapping_add(1);
            (*lzs).table[idx as usize] = b;
            (*lzs).curtabent &= 0xFFF;
            (*lzs).realcurrent = (*lzs).realcurrent.wrapping_add(1);
        }
    }

    while howmuch != 0 {
        if (*lzs).stringlen != 0 {
            b = (*lzs).table[(*lzs).stringpos as usize];
            (*lzs).stringpos = ((*lzs).stringpos.wrapping_add(1)) & 0xFFF;
            (*lzs).stringlen = (*lzs).stringlen.wrapping_sub(1);
        } else {
            if ((*lzs).bytetype as i32 & 0x100) == 0 {
                if 1 != GET(lzs, &mut b) {
                    return toread - howmuch;
                }
                (*lzs).bytetype = (b as i32 | 0xFF00) as u16;
            }
            if ((*lzs).bytetype as i32 & 1) != 0 {
                if 1 != GET(lzs, &mut b) {
                    return toread - howmuch;
                }
            } else {
                let mut b1: BYTE = 0;
                let mut b2: BYTE = 0;
                if 1 != GET(lzs, &mut b1) {
                    return toread - howmuch;
                }
                if 1 != GET(lzs, &mut b2) {
                    return toread - howmuch;
                }
                (*lzs).stringpos = (b1 as i32 | ((b2 as i32 & 0xF0) << 4)) as u32;
                (*lzs).stringlen = ((b2 as i32 & 0xF) + 2) as u8;
                b = (*lzs).table[(*lzs).stringpos as usize];
                (*lzs).stringpos = ((*lzs).stringpos.wrapping_add(1)) & 0xFFF;
            }
            (*lzs).bytetype = ((*lzs).bytetype as i32 >> 1) as u16;
        }
        let idx = (*lzs).curtabent;
        (*lzs).curtabent = (*lzs).curtabent.wrapping_add(1);
        (*lzs).table[idx as usize] = b;
        (*lzs).curtabent &= 0xFFF;
        (*lzs).realcurrent = (*lzs).realcurrent.wrapping_add(1);
        (*lzs).realwanted = (*lzs).realwanted.wrapping_add(1);
        *buf = b;
        buf = buf.offset(1);
        howmuch -= 1;
    }

    toread
}
#[no_mangle]
pub unsafe extern "system" fn LZSeek(fd: HFILE, off: LONG, type_: INT) -> LONG {
    let lzs: *mut lzstate;
    let mut newwanted: LONG;

    TRACE!(b"(%d,%d,%d)\n\0".as_ptr() as LPCSTR, fd, off, type_);

    lzs = GET_LZ_STATE(fd);
    if lzs.is_null() {
        return _llseek(fd, off, type_ as i32);
    }

    newwanted = (*lzs).realwanted as LONG;
    match type_ {
        1 => {
            newwanted = newwanted.wrapping_add(off);
        }
        2 => {
            newwanted = ((*lzs).reallength as LONG).wrapping_sub(off);
        }
        _ => {
            newwanted = off;
        }
    }

    if (newwanted as DWORD) > (*lzs).reallength {
        return LZERROR_BADVALUE as LONG;
    }
    if newwanted < 0 {
        return LZERROR_BADVALUE as LONG;
    }

    (*lzs).realwanted = newwanted as DWORD;
    newwanted
}

#[no_mangle]
pub unsafe extern "system" fn LZCopy(mut src: HFILE, mut dest: HFILE) -> LONG {
    let mut usedlzinit: i32 = 0;
    let mut ret: i32 = 0;
    let mut wret: i32 = 0;
    let mut len: LONG = 0;
    let mut oldsrc: HFILE = src;
    let mut srcfd: HFILE = 0;
    let mut filetime: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut lzs: *mut lzstate = core::ptr::null_mut();
    let mut buf: [CHAR; 1000] = [0; 1000];
    let mut xread: Option<unsafe extern "system" fn(HFILE, LPVOID, UINT) -> UINT> = None;

    TRACE!(b"(%d,%d)\n\0" as *const u8 as LPCSTR, src, dest);

    if !IS_LZ_HANDLE(src) {
        src = LZInit(src);
        if (src as i32) <= 0 {
            return 0;
        }
        if src != oldsrc {
            usedlzinit = 1;
        }
    }

    if !IS_LZ_HANDLE(src) {
        xread = Some(core::mem::transmute::<
            unsafe extern "system" fn(HFILE, LPVOID, LONG) -> LONG,
            unsafe extern "system" fn(HFILE, LPVOID, UINT) -> UINT,
        >(_hread));
    } else {
        xread = Some(core::mem::transmute::<
            unsafe extern "system" fn(HFILE, LPSTR, INT) -> INT,
            unsafe extern "system" fn(HFILE, LPVOID, UINT) -> UINT,
        >(LZRead));
    }

    len = 0;
    loop {
        ret = xread.expect("non-null function pointer")(
            src,
            &mut buf as *mut [CHAR; 1000] as *mut CHAR as LPVOID,
            1000,
        ) as i32;
        if ret <= 0 {
            if ret == 0 {
                break;
            }
            if ret == -1 {
                return LZERROR_READ as LONG;
            }
            return ret as LONG;
        }
        len += ret as LONG;
        wret = _lwrite(dest, &mut buf as *mut [CHAR; 1000] as *mut CHAR, ret as LONG) as i32;
        if wret != ret {
            return LZERROR_WRITE as LONG;
        }
    }

    lzs = GET_LZ_STATE(src);
    srcfd = if lzs.is_null() { src } else { (*lzs).realfd };

    GetFileTime(
        LongToHandle(srcfd as LONG) as *mut core::ffi::c_void,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut filetime,
    );
    SetFileTime(
        LongToHandle(dest as LONG) as *mut core::ffi::c_void,
        core::ptr::null(),
        core::ptr::null(),
        &filetime,
    );

    if usedlzinit != 0 {
        LZClose(src);
    }

    len
}
unsafe fn LZEXPAND_MangleName(fn_0: LPCSTR) -> LPSTR {
    let mut p: *mut i8 = core::ptr::null_mut();
    let mut mfn: *mut i8 = HeapAlloc(
        GetProcessHeap(),
        0,
        (strlen(fn_0) as usize).wrapping_add(3),
    ) as *mut i8;
    if mfn.is_null() {
        return core::ptr::null_mut();
    }
    strcpy(mfn, fn_0);
    p = strrchr(mfn, '\\' as i32);
    if p.is_null() {
        p = mfn;
    }
    p = strchr(p, '.' as i32);
    if !p.is_null() {
        p = p.offset(1);
        if strlen(p) < 3 {
            strcat(p, b"_\0".as_ptr() as *const i8);
        } else {
            *p.offset((strlen(p) as usize).wrapping_sub(1) as isize) = '_' as i8;
        }
    } else {
        strcat(mfn, b"._\0".as_ptr() as *const i8);
    }
    mfn as LPSTR
}
#[no_mangle]
pub unsafe extern "system" fn LZOpenFileA(
    fn_0: LPSTR,
    ofs: LPOFSTRUCT,
    mode: WORD,
) -> HFILE {
    let mut fd: HFILE = 0;
    let mut cfd: HFILE = 0;
    let ofs_cBytes: BYTE = (*ofs).cBytes;

    TRACE!(
        b"(%s,%p,%d)\0".as_ptr() as LPCSTR,
        fn_0,
        ofs,
        mode as i32,
    );

    fd = OpenFile(fn_0 as LPCSTR, ofs, mode as UINT);
    if fd == HFILE_ERROR {
        let mfn: LPSTR = LZEXPAND_MangleName(fn_0 as LPCSTR);
        fd = OpenFile(mfn as LPCSTR, ofs, mode as UINT);
        HeapFree(GetProcessHeap(), 0, mfn as LPVOID);
    }
    if fd == HFILE_ERROR {
        (*ofs).cBytes = ofs_cBytes;
    }
    if (mode as i32 & !0x70) != OF_READ as i32 {
        return fd;
    }
    if fd == HFILE_ERROR {
        return HFILE_ERROR;
    }
    cfd = LZInit(fd);
    if (cfd as i32) <= 0 {
        return fd;
    }
    cfd
}
#[no_mangle]
pub unsafe extern "system" fn LZOpenFileW(
    fn_0: LPWSTR,
    ofs: LPOFSTRUCT,
    mode: WORD,
) -> HFILE {

    let mut ret: HFILE = 0;
    let len: DWORD = WideCharToMultiByte(
        0 as UINT, // CP_ACP
        0 as DWORD,
        fn_0 as LPCWSTR,
        -1i32,
        core::ptr::null_mut::<CHAR>(),
        0i32,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    let xfn: LPSTR = HeapAlloc(GetProcessHeap(), 0 as DWORD, len as usize) as LPSTR;
    WideCharToMultiByte(
        0 as UINT, // CP_ACP
        0 as DWORD,
        fn_0 as LPCWSTR,
        -1i32,
        xfn,
        len as i32,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    ret = LZOpenFileA(xfn, ofs, mode) as HFILE;
    HeapFree(GetProcessHeap(), 0 as DWORD, xfn as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn LZClose(fd: HFILE) {
    let lzs: *mut lzstate;

    TRACE!(b"(%d)\n\0".as_ptr() as LPCSTR, fd);

    lzs = GET_LZ_STATE(fd);
    if lzs.is_null() {
        _lclose(fd);
    } else {
        HeapFree(GetProcessHeap(), 0, (*lzs).get as LPVOID);
        CloseHandle(LongToHandle((*lzs).realfd as i32) as HANDLE);
        lzstates[(fd as i32 - LZ_MIN_HANDLE) as usize] = core::ptr::null_mut::<lzstate>();
        HeapFree(GetProcessHeap(), 0, lzs as LPVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn LZCloseFile(FileHandle: HFILE) {
    LZClose(FileHandle);
}

#[no_mangle]
pub unsafe extern "system" fn LZCreateFileW(
    FileName: LPCWSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    dwCreationDisposition: DWORD,
    lpString1: LPWSTR,
) -> ULONG {
    WARN!(b" LZCreateFileW Not implemented!\n\0" as *const u8 as LPCSTR);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return ERROR_CALL_NOT_IMPLEMENTED;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * LZ Decompression functions
//  *
//  * Copyright 1996 Marcus Meissner
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * NOTES
//  *
//  * The LZ (Lempel Ziv) decompression was used in win16 installation programs.
//  * It is a simple tabledriven decompression engine, the algorithm is not
//  * documented as far as I know. WINE does not contain a compressor for
//  * this format.
//  *
//  * The implementation is complete and there have been no reports of failures
//  * for some time.
//  *
//  * TODO:
//  *
//  *   o Check whether the return values are correct
//  *
//  */
//
// #ifdef __REACTOS__
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
// #define HFILE_ERROR ((HFILE)-1)
//
// #include "lzexpand.h"
//
// #define _lwrite(a, b, c) (long)(_hwrite(a, b, (long)c))
//
// #else /* __REACTOS__ */
//
// #include "config.h"
//
// #include <string.h>
// #include <ctype.h>
// #include <sys/types.h>
// #include <stdarg.h>
// #include <stdio.h>
// #ifdef HAVE_UNISTD_H
// # include <unistd.h>
// #endif
//
// #include "windef.h"
// #include "winbase.h"
// #include "lzexpand.h"
//
// #include "wine/unicode.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(file);
//
// #endif /* __REACTOS__ */
//
// /* The readahead length of the decompressor. Reading single bytes
//  * using _lread() would be SLOW.
//  */
// #define	GETLEN	2048
//
// #define LZ_MAGIC_LEN    8
// #define LZ_HEADER_LEN   14
//
// /* Format of first 14 byte of LZ compressed file */
// struct lzfileheader {
// 	BYTE	magic[LZ_MAGIC_LEN];
// 	BYTE	compressiontype;
// 	CHAR	lastchar;
// 	DWORD	reallength;
// };
// static const BYTE LZMagic[LZ_MAGIC_LEN]={'S','Z','D','D',0x88,0xf0,0x27,0x33};
//
// #define LZ_TABLE_SIZE    0x1000
//
// struct lzstate {
// 	HFILE	realfd;		/* the real filedescriptor */
// 	CHAR	lastchar;	/* the last char of the filename */
//
// 	DWORD	reallength;	/* the decompressed length of the file */
// 	DWORD	realcurrent;	/* the position the decompressor currently is */
// 	DWORD	realwanted;	/* the position the user wants to read from */
//
// 	BYTE	table[LZ_TABLE_SIZE];	/* the rotating LZ table */
// 	UINT	curtabent;	/* CURrent TABle ENTry */
//
// 	BYTE	stringlen;	/* length and position of current string */
// 	DWORD	stringpos;	/* from stringtable */
//
//
// 	WORD	bytetype;	/* bitmask within blocks */
//
// 	BYTE	*get;		/* GETLEN bytes */
// 	DWORD	getcur;		/* current read */
// 	DWORD	getlen;		/* length last got */
// };
//
// #define MAX_LZSTATES 16
// static struct lzstate *lzstates[MAX_LZSTATES];
//
// #define LZ_MIN_HANDLE  0x400
// #define IS_LZ_HANDLE(h) (((h) >= LZ_MIN_HANDLE) && ((h) < LZ_MIN_HANDLE+MAX_LZSTATES))
// #define GET_LZ_STATE(h) (IS_LZ_HANDLE(h) ? lzstates[(h)-LZ_MIN_HANDLE] : NULL)
//
// /* reads one compressed byte, including buffering */
// #define GET(lzs,b)	_lzget(lzs,&b)
// #define GET_FLUSH(lzs)	lzs->getcur=lzs->getlen;
//
// static int
// _lzget(struct lzstate *lzs,BYTE *b) {
// 	if (lzs->getcur<lzs->getlen) {
// 		*b		= lzs->get[lzs->getcur++];
// 		return		1;
// 	} else {
// 		int ret = _lread(lzs->realfd,lzs->get,GETLEN);
// 		if (ret==HFILE_ERROR)
// 			return HFILE_ERROR;
// 		if (ret==0)
// 			return 0;
// 		lzs->getlen	= ret;
// 		lzs->getcur	= 1;
// 		*b		= *(lzs->get);
// 		return 1;
// 	}
// }
// /* internal function, reads lzheader
//  * returns BADINHANDLE for non filedescriptors
//  * return 0 for file not compressed using LZ
//  * return UNKNOWNALG for unknown algorithm
//  * returns lzfileheader in *head
//  */
// static INT read_header(HFILE fd,struct lzfileheader *head)
// {
// 	BYTE	buf[LZ_HEADER_LEN];
//
// 	if (_llseek(fd,0,SEEK_SET)==-1)
// 		return LZERROR_BADINHANDLE;
//
// 	/* We can't directly read the lzfileheader struct due to
// 	 * structure element alignment
// 	 */
// 	if (_lread(fd,buf,LZ_HEADER_LEN)<LZ_HEADER_LEN)
// 		return 0;
// 	memcpy(head->magic,buf,LZ_MAGIC_LEN);
// 	memcpy(&(head->compressiontype),buf+LZ_MAGIC_LEN,1);
// 	memcpy(&(head->lastchar),buf+LZ_MAGIC_LEN+1,1);
//
// 	/* FIXME: consider endianness on non-intel architectures */
// 	memcpy(&(head->reallength),buf+LZ_MAGIC_LEN+2,4);
//
// 	if (memcmp(head->magic,LZMagic,LZ_MAGIC_LEN))
// 		return 0;
// 	if (head->compressiontype!='A')
// 		return LZERROR_UNKNOWNALG;
// 	return 1;
// }
//
//
// /***********************************************************************
//  *           LZStart   (KERNEL32.@)
//  */
// INT WINAPI LZStart(void)
// {
//     TRACE!("(void)\n");
//     return 1;
// }
//
//
// /***********************************************************************
//  *           LZInit   (KERNEL32.@)
//  *
//  * initializes internal decompression buffers, returns lzfiledescriptor.
//  * (return value the same as hfSrc, if hfSrc is not compressed)
//  * on failure, returns error code <0
//  * lzfiledescriptors range from 0x400 to 0x410 (only 16 open files per process)
//  *
//  * since _llseek uses the same types as libc.lseek, we just use the macros of
//  *  libc
//  */
// HFILE WINAPI LZInit( HFILE hfSrc )
// {
//
// 	struct	lzfileheader	head;
// 	struct	lzstate		*lzs;
// 	int	i, ret;
//
// 	TRACE!("(%d)\n",hfSrc);
// 	ret=read_header(hfSrc,&head);
// 	if (ret<=0) {
// 		_llseek(hfSrc,0,SEEK_SET);
// 		return ret?ret:hfSrc;
// 	}
//         for (i = 0; i < MAX_LZSTATES; i++) if (!lzstates[i]) break;
//         if (i == MAX_LZSTATES) return LZERROR_GLOBALLOC;
// 	lzstates[i] = lzs = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(*lzs) );
// 	if(lzs == NULL) return LZERROR_GLOBALLOC;
//
// 	lzs->realfd	= hfSrc;
// 	lzs->lastchar	= head.lastchar;
// 	lzs->reallength = head.reallength;
//
// 	lzs->get	= HeapAlloc( GetProcessHeap(), 0, GETLEN );
// 	lzs->getlen	= 0;
// 	lzs->getcur	= 0;
//
// 	if(lzs->get == NULL) {
// 		HeapFree(GetProcessHeap(), 0, lzs);
// 		lzstates[i] = NULL;
// 		return LZERROR_GLOBALLOC;
// 	}
//
// 	/* Yes, preinitialize with spaces */
// 	memset(lzs->table,' ',LZ_TABLE_SIZE);
// 	/* Yes, start 16 byte from the END of the table */
// 	lzs->curtabent	= 0xff0;
// 	return LZ_MIN_HANDLE + i;
// }
//
//
// /***********************************************************************
//  *           LZDone   (KERNEL32.@)
//  */
// void WINAPI LZDone(void)
// {
//     TRACE!("(void)\n");
// }
//
//
// /***********************************************************************
//  *           GetExpandedNameA   (KERNEL32.@)
//  *
//  * gets the full filename of the compressed file 'in' by opening it
//  * and reading the header
//  *
//  * "file." is being translated to "file"
//  * "file.bl_" (with lastchar 'a') is being translated to "file.bla"
//  * "FILE.BL_" (with lastchar 'a') is being translated to "FILE.BLA"
//  */
//
// INT WINAPI GetExpandedNameA( LPSTR in, LPSTR out )
// {
// 	struct lzfileheader	head;
// 	HFILE		fd;
// 	OFSTRUCT	ofs;
// 	INT		fnislowercased,ret,len;
// 	LPSTR		s,t;
//
// 	TRACE!("(%s)\n",in);
// 	fd=OpenFile(in,&ofs,OF_READ);
// 	if (fd==HFILE_ERROR)
// 		return (INT)(INT16)LZERROR_BADINHANDLE;
// 	strcpy(out,in);
// 	ret=read_header(fd,&head);
// 	if (ret<=0) {
// 		/* not a LZ compressed file, so the expanded name is the same
// 		 * as the input name */
// 		_lclose(fd);
// 		return 1;
// 	}
//
//
// 	/* look for directory prefix and skip it. */
// 	s=out;
// 	while (NULL!=(t=strpbrk(s,"/\\:")))
// 		s=t+1;
//
// 	/* now mangle the basename */
// 	if (!*s) {
// 		/* FIXME: hmm. shouldn't happen? */
// 		WARN!("Specified a directory or what? (%s)\n",in);
// 		_lclose(fd);
// 		return 1;
// 	}
// 	/* see if we should use lowercase or uppercase on the last char */
// 	fnislowercased=1;
// 	t=s+strlen(s)-1;
// 	while (t>=out) {
// 		if (!isalpha(*t)) {
// 			t--;
// 			continue;
// 		}
// 		fnislowercased=islower(*t);
// 		break;
// 	}
// 	if (isalpha(head.lastchar)) {
// 		if (fnislowercased)
// 			head.lastchar=tolower(head.lastchar);
// 		else
// 			head.lastchar=toupper(head.lastchar);
// 	}
//
// 	/* now look where to replace the last character */
// 	if (NULL!=(t=strchr(s,'.'))) {
// 		if (t[1]=='\0') {
// 			t[0]='\0';
// 		} else {
// 			len=strlen(t)-1;
// 			if (t[len]=='_')
// 				t[len]=head.lastchar;
// 		}
// 	} /* else no modification necessary */
// 	_lclose(fd);
// 	return 1;
// }
//
//
// /***********************************************************************
//  *           GetExpandedNameW   (KERNEL32.@)
//  */
// INT WINAPI GetExpandedNameW( LPWSTR in, LPWSTR out )
// {
//     INT ret;
//     DWORD len = WideCharToMultiByte( CP_ACP, 0, in, -1, NULL, 0, NULL, NULL );
//     char *xin = HeapAlloc( GetProcessHeap(), 0, len );
//     char *xout = HeapAlloc( GetProcessHeap(), 0, len+3 );
//     WideCharToMultiByte( CP_ACP, 0, in, -1, xin, len, NULL, NULL );
//     if ((ret = GetExpandedNameA( xin, xout )) > 0)
//         MultiByteToWideChar( CP_ACP, 0, xout, -1, out, strlenW(in)+4 );
//     HeapFree( GetProcessHeap(), 0, xin );
//     HeapFree( GetProcessHeap(), 0, xout );
//     return ret;
// }
//
//
// /***********************************************************************
//  *           LZRead   (KERNEL32.@)
//  */
// INT WINAPI LZRead( HFILE fd, LPSTR vbuf, INT toread )
// {
// 	int	howmuch;
// 	BYTE	b,*buf;
// 	struct	lzstate	*lzs;
//
// 	buf=(LPBYTE)vbuf;
// 	TRACE!("(%d,%p,%d)\n",fd,buf,toread);
// 	howmuch=toread;
// 	if (!(lzs = GET_LZ_STATE(fd))) return _lread(fd,buf,toread);
//
// /* The decompressor itself is in a define, cause we need it twice
//  * in this function. (the decompressed byte will be in b)
//  */
// #define DECOMPRESS_ONE_BYTE 						\
// 		if (lzs->stringlen) {					\
// 			b		= lzs->table[lzs->stringpos];	\
// 			lzs->stringpos	= (lzs->stringpos+1)&0xFFF;	\
// 			lzs->stringlen--;				\
// 		} else {						\
// 			if (!(lzs->bytetype&0x100)) {			\
// 				if (1!=GET(lzs,b)) 			\
// 					return toread-howmuch;		\
// 				lzs->bytetype = b|0xFF00;		\
// 			}						\
// 			if (lzs->bytetype & 1) {			\
// 				if (1!=GET(lzs,b))			\
// 					return toread-howmuch;		\
// 			} else {					\
// 				BYTE	b1,b2;				\
// 									\
// 				if (1!=GET(lzs,b1))			\
// 					return toread-howmuch;		\
// 				if (1!=GET(lzs,b2))			\
// 					return toread-howmuch;		\
// 				/* Format:				\
// 				 * b1 b2				\
// 				 * AB CD 				\
// 				 * where CAB is the stringoffset in the table\
// 				 * and D+3 is the len of the string	\
// 				 */					\
// 				lzs->stringpos	= b1|((b2&0xf0)<<4);	\
// 				lzs->stringlen	= (b2&0xf)+2; 		\
// 				/* 3, but we use a  byte already below ... */\
// 				b		= lzs->table[lzs->stringpos];\
// 				lzs->stringpos	= (lzs->stringpos+1)&0xFFF;\
// 			}						\
// 			lzs->bytetype>>=1;				\
// 		}							\
// 		/* store b in table */					\
// 		lzs->table[lzs->curtabent++]= b;			\
// 		lzs->curtabent	&= 0xFFF;				\
// 		lzs->realcurrent++;
//
// 	/* if someone has seeked, we have to bring the decompressor
// 	 * to that position
// 	 */
// 	if (lzs->realcurrent!=lzs->realwanted) {
// 		/* if the wanted position is before the current position
// 		 * I see no easy way to unroll ... We have to restart at
// 		 * the beginning. *sigh*
// 		 */
// 		if (lzs->realcurrent>lzs->realwanted) {
// 			/* flush decompressor state */
// 			_llseek(lzs->realfd,LZ_HEADER_LEN,SEEK_SET);
// 			GET_FLUSH(lzs);
// 			lzs->realcurrent= 0;
// 			lzs->bytetype	= 0;
// 			lzs->stringlen	= 0;
// 			memset(lzs->table,' ',LZ_TABLE_SIZE);
// 			lzs->curtabent	= 0xFF0;
// 		}
// 		while (lzs->realcurrent<lzs->realwanted) {
// 			DECOMPRESS_ONE_BYTE;
// 		}
// 	}
//
// 	while (howmuch) {
// 		DECOMPRESS_ONE_BYTE;
// 		lzs->realwanted++;
// 		*buf++		= b;
// 		howmuch--;
// 	}
// 	return 	toread;
// #undef DECOMPRESS_ONE_BYTE
// }
//
//
// /***********************************************************************
//  *           LZSeek   (KERNEL32.@)
//  */
// LONG WINAPI LZSeek( HFILE fd, LONG off, INT type )
// {
// 	struct	lzstate	*lzs;
// 	LONG	newwanted;
//
// 	TRACE!("(%d,%d,%d)\n",fd,off,type);
// 	/* not compressed? just use normal _llseek() */
//         if (!(lzs = GET_LZ_STATE(fd))) return _llseek(fd,off,type);
// 	newwanted = lzs->realwanted;
// 	switch (type) {
// 	case 1:	/* SEEK_CUR */
// 		newwanted      += off;
// 		break;
// 	case 2:	/* SEEK_END */
// 		newwanted	= lzs->reallength-off;
// 		break;
// 	default:/* SEEK_SET */
// 		newwanted	= off;
// 		break;
// 	}
// 	if (newwanted>lzs->reallength)
// 		return LZERROR_BADVALUE;
// 	if (newwanted<0)
// 		return LZERROR_BADVALUE;
// 	lzs->realwanted	= newwanted;
// 	return newwanted;
// }
//
//
// /***********************************************************************
//  *           LZCopy   (KERNEL32.@)
//  *
//  * Copies everything from src to dest
//  * if src is a LZ compressed file, it will be uncompressed.
//  * will return the number of bytes written to dest or errors.
//  */
// LONG WINAPI LZCopy( HFILE src, HFILE dest )
// {
// 	int	usedlzinit = 0, ret, wret;
// 	LONG	len;
// 	HFILE	oldsrc = src, srcfd;
// 	FILETIME filetime;
// 	struct	lzstate	*lzs;
// #define BUFLEN	1000
// 	CHAR	buf[BUFLEN];
// 	/* we need that weird typedef, for i can't seem to get function pointer
// 	 * casts right. (Or they probably just do not like WINAPI in general)
// 	 */
// 	typedef	UINT	(WINAPI *_readfun)(HFILE,LPVOID,UINT);
//
// 	_readfun	xread;
//
// 	TRACE!("(%d,%d)\n",src,dest);
// 	if (!IS_LZ_HANDLE(src)) {
// 		src = LZInit(src);
//                 if ((INT)src <= 0) return 0;
// 		if (src != oldsrc) usedlzinit=1;
// 	}
//
// 	/* not compressed? just copy */
//         if (!IS_LZ_HANDLE(src))
// #ifdef __REACTOS__
// 		xread=(_readfun)_hread; // ROSHACK
// #else
// 		xread=_lread;
// #endif
// 	else
// 		xread=(_readfun)LZRead;
// 	len=0;
// 	while (1) {
// 		ret=xread(src,buf,BUFLEN);
// 		if (ret<=0) {
// 			if (ret==0)
// 				break;
// 			if (ret==-1)
// 				return LZERROR_READ;
// 			return ret;
// 		}
// 		len    += ret;
// 		wret	= _lwrite(dest,buf,ret);
// 		if (wret!=ret)
// 			return LZERROR_WRITE;
// 	}
//
// 	/* Maintain the timestamp of source file to destination file */
// 	srcfd = (!(lzs = GET_LZ_STATE(src))) ? src : lzs->realfd;
// 	GetFileTime( LongToHandle(srcfd), NULL, NULL, &filetime );
// 	SetFileTime( LongToHandle(dest), NULL, NULL, &filetime );
//
// 	/* close handle */
// 	if (usedlzinit)
// 		LZClose(src);
// 	return len;
// #undef BUFLEN
// }
//
// /* reverses GetExpandedPathname */
// static LPSTR LZEXPAND_MangleName( LPCSTR fn )
// {
//     char *p;
//     char *mfn = HeapAlloc( GetProcessHeap(), 0, strlen(fn) + 3 ); /* "._" and \0 */
//     if(mfn == NULL) return NULL;
//     strcpy( mfn, fn );
//     if (!(p = strrchr( mfn, '\\' ))) p = mfn;
//     if ((p = strchr( p, '.' )))
//     {
//         p++;
//         if (strlen(p) < 3) strcat( p, "_" );  /* append '_' */
//         else p[strlen(p)-1] = '_';  /* replace last character */
//     }
//     else strcat( mfn, "._" );	/* append "._" */
//     return mfn;
// }
//
//
// /***********************************************************************
//  *           LZOpenFileA   (KERNEL32.@)
//  *
//  * Opens a file. If not compressed, open it as a normal file.
//  */
// HFILE WINAPI LZOpenFileA( LPSTR fn, LPOFSTRUCT ofs, WORD mode )
// {
// 	HFILE	fd,cfd;
// 	BYTE    ofs_cBytes = ofs->cBytes;
//
// 	TRACE!("(%s,%p,%d)\n",fn,ofs,mode);
// 	/* 0x70 represents all OF_SHARE_* flags, ignore them for the check */
// 	fd=OpenFile(fn,ofs,mode);
// 	if (fd==HFILE_ERROR)
//         {
//             LPSTR mfn = LZEXPAND_MangleName(fn);
//             fd = OpenFile(mfn,ofs,mode);
//             HeapFree( GetProcessHeap(), 0, mfn );
// 	}
// 	if (fd==HFILE_ERROR)
// 		ofs->cBytes = ofs_cBytes;
// 	if ((mode&~0x70)!=OF_READ)
// 		return fd;
// 	if (fd==HFILE_ERROR)
// 		return HFILE_ERROR;
// 	cfd=LZInit(fd);
// 	if ((INT)cfd <= 0) return fd;
// 	return cfd;
// }
//
//
// /***********************************************************************
//  *           LZOpenFileW   (KERNEL32.@)
//  */
// HFILE WINAPI LZOpenFileW( LPWSTR fn, LPOFSTRUCT ofs, WORD mode )
// {
//     HFILE ret;
//     DWORD len = WideCharToMultiByte( CP_ACP, 0, fn, -1, NULL, 0, NULL, NULL );
//     LPSTR xfn = HeapAlloc( GetProcessHeap(), 0, len );
//     WideCharToMultiByte( CP_ACP, 0, fn, -1, xfn, len, NULL, NULL );
//     ret = LZOpenFileA(xfn,ofs,mode);
//     HeapFree( GetProcessHeap(), 0, xfn );
//     return ret;
// }
//
//
// /***********************************************************************
//  *           LZClose   (KERNEL32.@)
//  */
// void WINAPI LZClose( HFILE fd )
// {
// 	struct lzstate *lzs;
//
// 	TRACE!("(%d)\n",fd);
//         if (!(lzs = GET_LZ_STATE(fd))) _lclose(fd);
//         else
//         {
//             HeapFree( GetProcessHeap(), 0, lzs->get );
//             CloseHandle( LongToHandle(lzs->realfd) );
//             lzstates[fd - LZ_MIN_HANDLE] = NULL;
//             HeapFree( GetProcessHeap(), 0, lzs );
//         }
// }
//
// #ifdef __REACTOS__
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// LZCloseFile(IN HFILE FileHandle)
// {
//     /* One function uses _lclose, the other CloseHandle -- same thing */
//     LZClose(FileHandle);
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// WINAPI
// LZCreateFileW(IN LPCWSTR FileName,
//               IN DWORD dwDesiredAccess,
//               IN DWORD dwShareMode,
//               IN DWORD dwCreationDisposition,
//               IN LPWSTR lpString1)
// {
//     WARN!(" LZCreateFileW Not implemented!\n");
//     SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
// #endif /* __REACTOS__ */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
