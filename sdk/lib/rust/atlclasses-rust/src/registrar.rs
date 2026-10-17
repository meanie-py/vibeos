//! ATL's static registrar (statreg.h `CRegObject`): the `.rgs` script language behind
//! `DECLARE_REGISTRY_RESOURCEID` and `UpdateRegistryFromResource`.
//!
//! A port of statreg.h, step for step: `%KEY%` replacement (`do_preprocess`), the word
//! splitter (`get_word`), the key walker (`do_process_key`) and the root-key loop
//! (`do_process_root_key`), with a failed REGISTER rolled back by an unregister pass. The
//! registry sits behind [`Registry`] so the parser runs on the host against a fake; the DLL
//! build uses [`Win32Registry`] (feature `dll`, which is what imports advapi32).
//!
//! Where it departs from statreg.h, on purpose:
//! - `RegDeleteTreeX` (the C's `ForceRemove`/`Delete` helper) enumerates the PARENT's
//!   subkeys while it deletes under the CHILD, so a `ForceRemove` key that has subkeys is
//!   never removed (its first "child" lookup fails and the delete is abandoned). This walks
//!   the child, which is what the name and every caller mean.
//! - A key block that runs off the end of the script loops forever in the C (an empty word
//!   is neither `}` nor progress); here it is `DISP_E_EXCEPTION`, the C's error for every
//!   other malformed script.

use core::ffi::c_void;

pub type HRESULT = i32;
pub type HKEY = *mut c_void;

pub const S_OK: HRESULT = 0;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000E_u32 as i32;
pub const DISP_E_EXCEPTION: HRESULT = 0x8002_0009_u32 as i32;

pub const REG_SZ: u32 = 1;
pub const REG_EXPAND_SZ: u32 = 2;
pub const REG_BINARY: u32 = 3;
pub const REG_DWORD: u32 = 4;

/// winerror.h `HRESULT_FROM_WIN32`.
pub fn hresult_from_win32(e: i32) -> HRESULT {
    if e <= 0 {
        e
    } else {
        ((e as u32 & 0xFFFF) | 0x8007_0000) as i32
    }
}

/// `((HKEY)(ULONG_PTR)((LONG)0x8000000N))`: sign-extended on 64-bit, as winreg.h spells it.
const fn predefined(n: u32) -> HKEY {
    (n as i32 as isize) as HKEY
}

/// The calls statreg.h makes, by name. Every string is NUL-terminated UTF-16.
pub trait Registry {
    /// `RegCreateKeyW`.
    unsafe fn create_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32;
    /// `RegOpenKeyW`.
    unsafe fn open_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32;
    /// `RegSetValueExW` (`name` null for the default value).
    unsafe fn set_value(&mut self, key: HKEY, name: *const u16, ty: u32, data: *const u8, len: u32)
        -> i32;
    /// `RegDeleteKeyW`.
    unsafe fn delete_key(&mut self, parent: HKEY, sub: *const u16) -> i32;
    /// `RegDeleteTreeX`: the key and everything under it.
    unsafe fn delete_tree(&mut self, parent: HKEY, sub: *const u16) -> i32;
    /// `RegCloseKey`.
    unsafe fn close_key(&mut self, key: HKEY) -> i32;
}

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn realloc(p: *mut c_void, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

/// statreg.h's `strbuf`: a growable NUL-terminated wide string.
struct StrBuf {
    str: *mut u16,
    alloc: usize,
    len: usize,
}

impl StrBuf {
    /// `strbuf_init`: 128 characters.
    fn new() -> Option<Self> {
        let str = unsafe { malloc(128 * 2) } as *mut u16;
        if str.is_null() {
            return None;
        }
        unsafe { *str = 0 };
        Some(Self { str, alloc: 128, len: 0 })
    }

    /// `strbuf_write`.
    unsafe fn write(&mut self, s: *const u16, len: usize) -> HRESULT {
        if self.len + len + 1 >= self.alloc {
            let alloc = (self.len + len) * 2;
            let p = realloc(self.str as *mut c_void, alloc * 2) as *mut u16;
            if p.is_null() {
                return E_OUTOFMEMORY;
            }
            self.str = p;
            self.alloc = alloc;
        }
        core::ptr::copy_nonoverlapping(s, self.str.add(self.len), len);
        self.len += len;
        *self.str.add(self.len) = 0;
        S_OK
    }

    unsafe fn write_sz(&mut self, s: *const u16) -> HRESULT {
        self.write(s, wcslen(s))
    }

    fn clear(&mut self) {
        self.len = 0;
        unsafe { *self.str = 0 };
    }

    fn at(&self, i: usize) -> u16 {
        if i < self.len {
            unsafe { *self.str.add(i) }
        } else {
            0
        }
    }

    /// `buf->str[1] || buf->str[0] != c`: anything but exactly the one character.
    fn is(&self, c: u8) -> bool {
        self.len == 1 && self.at(0) == c as u16
    }

    fn eq_ignore_case(&self, word: &str) -> bool {
        self.len == word.len()
            && word
                .bytes()
                .enumerate()
                .all(|(i, b)| lower(self.at(i)) == b.to_ascii_lowercase() as u16)
    }
}

impl Drop for StrBuf {
    fn drop(&mut self) {
        unsafe { free(self.str as *mut c_void) }
    }
}

unsafe fn wcslen(s: *const u16) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

fn lower(c: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&c) {
        c + 32
    } else {
        c
    }
}

/// `iswspace` for the script's characters (msvcrt's C1_SPACE set below 0x100).
fn is_space(c: u16) -> bool {
    matches!(c, 0x09..=0x0D | 0x20)
}

/// A NUL-terminated wide string with a cursor: `*iter` reads 0 past the end.
struct Cursor {
    s: *const u16,
    i: usize,
}

impl Cursor {
    fn cur(&self) -> u16 {
        unsafe { *self.s.add(self.i) }
    }
    fn peek(&self, k: usize) -> u16 {
        // Never past the NUL: `iter[1]` is read only when `*iter` is not 0.
        if self.cur() == 0 {
            0
        } else {
            unsafe { *self.s.add(self.i + k) }
        }
    }
    fn ptr(&self) -> *const u16 {
        unsafe { self.s.add(self.i) }
    }
}

/// `CRegObject`: the replacement list, and the script walk over a [`Registry`].
pub struct RegObject {
    /// `rep_list`, as `key\0item\0` pairs in insertion order (looked up newest first, as
    /// the C's prepended list is).
    reps: StrBuf,
}

impl RegObject {
    pub fn new() -> Option<Self> {
        Some(Self { reps: StrBuf::new()? })
    }

    /// `AddReplacement(key, item)`.
    pub unsafe fn AddReplacement(&mut self, key: *const u16, item: *const u16) -> HRESULT {
        let nul = [0u16];
        let nul = nul.as_ptr();
        for (s, n) in [(key, wcslen(key)), (nul, 1), (item, wcslen(item)), (nul, 1)] {
            let hr = self.reps.write(s, n);
            if hr != S_OK {
                return hr;
            }
        }
        S_OK
    }

    /// `ClearReplacements()`.
    pub fn ClearReplacements(&mut self) -> HRESULT {
        self.reps.clear();
        S_OK
    }

    /// The item of the newest replacement whose key equals `key[..len]` under `_memicmp`
    /// (byte-wise, ASCII case-folded: both bytes of each UTF-16 unit).
    unsafe fn replacement(&self, key: *const u16, len: usize) -> Option<*const u16> {
        let mut found = None;
        let mut i = 0;
        while i < self.reps.len {
            let k = self.reps.str.add(i);
            let klen = wcslen(k);
            let item = k.add(klen + 1);
            let ilen = wcslen(item);
            if klen == len
                && (0..len).all(|j| {
                    let (a, b) = (*k.add(j), *key.add(j));
                    let fold = |c: u16| ((c >> 8) as u8).to_ascii_lowercase() as u16 * 256
                        + (c as u8).to_ascii_lowercase() as u16;
                    fold(a) == fold(b)
                })
            {
                found = Some(item as *const u16);
            }
            i += klen + 1 + ilen + 1;
        }
        found
    }

    /// `do_preprocess`: `%KEY%` -> its replacement, `%%` -> `%`.
    unsafe fn preprocess(&self, data: *const u16, buf: &mut StrBuf) -> HRESULT {
        let mut it = Cursor { s: data, i: 0 };
        let mut start = 0;
        loop {
            while it.cur() != 0 && it.cur() != b'%' as u16 {
                it.i += 1;
            }
            if it.cur() == 0 {
                break;
            }
            let hr = buf.write(data.add(start), it.i - start);
            if hr != S_OK {
                return hr;
            }
            it.i += 1;
            start = it.i;
            if it.cur() == 0 {
                return DISP_E_EXCEPTION;
            }
            while it.cur() != 0 && it.cur() != b'%' as u16 {
                it.i += 1;
            }
            if it.cur() == 0 {
                return DISP_E_EXCEPTION;
            }
            let hr = if it.i == start {
                buf.write([b'%' as u16].as_ptr(), 1)
            } else {
                match self.replacement(data.add(start), it.i - start) {
                    Some(item) => buf.write_sz(item),
                    None => return DISP_E_EXCEPTION,
                }
            };
            if hr != S_OK {
                return hr;
            }
            it.i += 1;
            start = it.i;
        }
        buf.write_sz(data.add(start))
    }

    /// `string_register`: preprocess, walk, and on a failed register undo what was done.
    pub unsafe fn string_register<R: Registry>(
        &self,
        reg: &mut R,
        data: *const u16,
        do_register: bool,
    ) -> HRESULT {
        let Some(mut buf) = StrBuf::new() else {
            return E_OUTOFMEMORY;
        };
        let mut hr = self.preprocess(data, &mut buf);
        if hr >= 0 {
            hr = process_root_key(reg, buf.str, do_register);
            if hr < 0 && do_register {
                process_root_key(reg, buf.str, false);
            }
        }
        hr
    }
}

/// `get_word`: the next word into `buf` -- `}` or `=` alone, a `'quoted string'`, or a run
/// of non-space -- and the cursor past it and the spaces after it.
unsafe fn get_word(it: &mut Cursor, buf: &mut StrBuf) -> HRESULT {
    buf.clear();
    while is_space(it.cur()) {
        it.i += 1;
    }
    if it.cur() == 0 {
        return S_OK;
    }
    let c = it.cur();
    if c == b'}' as u16 || c == b'=' as u16 {
        let hr = buf.write(it.ptr(), 1);
        if hr != S_OK {
            return hr;
        }
        it.i += 1;
    } else if c == b'\'' as u16 {
        it.i += 1;
        let start = it.i;
        while it.cur() != 0 && it.cur() != b'\'' as u16 {
            it.i += 1;
        }
        if it.cur() == 0 {
            return DISP_E_EXCEPTION;
        }
        let hr = buf.write(it.s.add(start), it.i - start);
        if hr != S_OK {
            return hr;
        }
        it.i += 1;
    } else {
        let start = it.i;
        while it.cur() != 0 && !is_space(it.cur()) {
            it.i += 1;
        }
        let hr = buf.write(it.s.add(start), it.i - start);
        if hr != S_OK {
            return hr;
        }
    }
    while is_space(it.cur()) {
        it.i += 1;
    }
    S_OK
}

fn hex_to_bin(a: u16) -> u8 {
    match a {
        0x30..=0x39 => (a - 0x30) as u8,
        0x41..=0x46 => (a - 0x41 + 10) as u8,
        0x61..=0x66 => (a - 0x61 + 10) as u8,
        _ => 0, // ATLASSERT(false)
    }
}

/// msvcrt `wcstoul(s, &end, 16)` / `wcstol(s, &end, 10)`: leading space, a sign, digits up
/// to the first that is not one; out of range saturates (ULONG_MAX, LONG_MAX/LONG_MIN).
unsafe fn wcsto(s: *const u16, radix: u32, signed: bool) -> u32 {
    let mut i = 0;
    while is_space(*s.add(i)) {
        i += 1;
    }
    let neg = *s.add(i) == b'-' as u16;
    if neg || *s.add(i) == b'+' as u16 {
        i += 1;
    }
    let mut v: u64 = 0;
    let mut over = false;
    while let Some(d) = char::from_u32(*s.add(i) as u32).and_then(|c| c.to_digit(radix)) {
        v = v * radix as u64 + d as u64;
        over |= v > u32::MAX as u64;
        i += 1;
    }
    if signed {
        let lim = if neg { 0x8000_0000u64 } else { 0x7FFF_FFFF };
        if over || v > lim {
            return if neg { 0x8000_0000 } else { 0x7FFF_FFFF };
        }
        if neg {
            (v as u32).wrapping_neg()
        } else {
            v as u32
        }
    } else if over {
        u32::MAX
    } else if neg {
        (v as u32).wrapping_neg()
    } else {
        v as u32
    }
}

#[derive(PartialEq, Clone, Copy)]
enum KeyType {
    Normal,
    NoRemove,
    IsVal,
    ForceRemove,
    DoDelete,
}

/// `do_process_key`: one `{ ... }` block under `parent`, up to and including its `}`.
unsafe fn process_key<R: Registry>(
    reg: &mut R,
    it: &mut Cursor,
    parent: HKEY,
    buf: &mut StrBuf,
    do_register: bool,
) -> HRESULT {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut key_type = KeyType::Normal;
    let mut hres = get_word(it, buf);
    if hres < 0 {
        return hres;
    }
    let Some(mut name) = StrBuf::new() else {
        return E_OUTOFMEMORY;
    };
    while !buf.is(b'}') {
        if buf.len == 0 {
            hres = DISP_E_EXCEPTION; // the script ended inside this block
            break;
        }
        key_type = if buf.eq_ignore_case("NoRemove") {
            KeyType::NoRemove
        } else if buf.eq_ignore_case("ForceRemove") {
            KeyType::ForceRemove
        } else if buf.eq_ignore_case("val") {
            KeyType::IsVal
        } else if buf.eq_ignore_case("Delete") {
            KeyType::DoDelete
        } else {
            KeyType::Normal
        };
        if key_type != KeyType::Normal {
            hres = get_word(it, buf);
            if hres < 0 {
                break;
            }
        }

        if do_register {
            match key_type {
                KeyType::IsVal => {
                    hkey = parent;
                    hres = name.write(buf.str, buf.len);
                    if hres < 0 {
                        return hres;
                    }
                }
                KeyType::DoDelete => {
                    reg.delete_tree(parent, buf.str);
                }
                _ => {
                    if key_type == KeyType::ForceRemove {
                        reg.delete_tree(parent, buf.str);
                    }
                    let lres = reg.create_key(parent, buf.str, &mut hkey);
                    if lres != 0 {
                        hres = hresult_from_win32(lres);
                        break;
                    }
                }
            }
        } else if key_type != KeyType::IsVal && key_type != KeyType::DoDelete {
            hres = name.write(buf.str, buf.len);
            if hres < 0 {
                return hres;
            }
            hkey = core::ptr::null_mut();
            if reg.open_key(parent, buf.str, &mut hkey) != 0 {
                hkey = core::ptr::null_mut();
            }
        }

        if key_type != KeyType::DoDelete && it.cur() == b'=' as u16 {
            it.i += 1;
            hres = get_word(it, buf);
            if hres < 0 {
                break;
            }
            if buf.len != 1 {
                hres = DISP_E_EXCEPTION;
                break;
            }
            if do_register {
                let vname = if name.len != 0 { name.str as *const u16 } else { core::ptr::null() };
                let ty = buf.at(0);
                hres = get_word(it, buf);
                if hres < 0 {
                    break;
                }
                let lres = match ty as u8 {
                    b's' | b'e' => reg.set_value(
                        hkey,
                        vname,
                        if ty == b's' as u16 { REG_SZ } else { REG_EXPAND_SZ },
                        buf.str as *const u8,
                        ((buf.len + 1) * 2) as u32,
                    ),
                    b'd' => {
                        let hex = (buf.at(0) == b'0' as u16 && buf.at(1) == b'x' as u16)
                            || (buf.at(0) == b'&' as u16 && buf.at(1) == b'H' as u16);
                        let dw = if hex {
                            wcsto(buf.str.add(2), 16, false)
                        } else {
                            wcsto(buf.str, 10, true)
                        };
                        reg.set_value(hkey, vname, REG_DWORD, &dw as *const u32 as *const u8, 4)
                    }
                    b'b' => {
                        let count = buf.len;
                        if count & 1 != 0 {
                            return DISP_E_EXCEPTION;
                        }
                        let bytes = buf.str as *mut u8;
                        // In place, as the C does: byte k is written after units 2k, 2k+1
                        // (which it overlaps at most) have been read.
                        for k in 0..count / 2 {
                            let v = (hex_to_bin(buf.at(k * 2)) << 4) | hex_to_bin(buf.at(k * 2 + 1));
                            *bytes.add(k) = v;
                        }
                        reg.set_value(hkey, vname, REG_BINARY, bytes, (count / 2) as u32)
                    }
                    _ => {
                        hres = DISP_E_EXCEPTION;
                        0
                    }
                };
                if lres != 0 {
                    hres = hresult_from_win32(lres);
                }
                // The C's `if (FAILED(hres)) break;` after its switch.
                if hres < 0 {
                    break;
                }
            } else {
                if it.cur() == b'-' as u16 {
                    it.i += 1;
                }
                hres = get_word(it, buf);
                if hres < 0 {
                    break;
                }
            }
        } else if key_type == KeyType::IsVal {
            hres = DISP_E_EXCEPTION;
            break;
        }

        if key_type != KeyType::IsVal
            && key_type != KeyType::DoDelete
            && it.cur() == b'{' as u16
            && is_space(it.peek(1))
        {
            hres = get_word(it, buf);
            if hres < 0 {
                break;
            }
            hres = process_key(reg, it, hkey, buf, do_register);
            if hres < 0 {
                break;
            }
        }

        if !do_register && (key_type == KeyType::Normal || key_type == KeyType::ForceRemove) {
            reg.delete_key(parent, name.str);
        }

        if !hkey.is_null() && key_type != KeyType::IsVal {
            reg.close_key(hkey);
        }
        hkey = core::ptr::null_mut();
        name.clear();

        hres = get_word(it, buf);
        if hres < 0 {
            break;
        }
    }
    if !hkey.is_null() && key_type != KeyType::IsVal {
        reg.close_key(hkey);
    }
    hres
}

const ROOT_KEYS: [(&str, u32); 14] = [
    ("HKEY_CLASSES_ROOT", 0x8000_0000),
    ("HKEY_CURRENT_USER", 0x8000_0001),
    ("HKEY_LOCAL_MACHINE", 0x8000_0002),
    ("HKEY_USERS", 0x8000_0003),
    ("HKEY_PERFORMANCE_DATA", 0x8000_0004),
    ("HKEY_DYN_DATA", 0x8000_0006),
    ("HKEY_CURRENT_CONFIG", 0x8000_0005),
    ("HKCR", 0x8000_0000),
    ("HKCU", 0x8000_0001),
    ("HKLM", 0x8000_0002),
    ("HKU", 0x8000_0003),
    ("HKPD", 0x8000_0004),
    ("HKDD", 0x8000_0006),
    ("HKCC", 0x8000_0005),
];

/// `do_process_root_key`: `HKCR { ... } HKLM { ... }`.
unsafe fn process_root_key<R: Registry>(reg: &mut R, data: *const u16, do_register: bool) -> HRESULT {
    let mut it = Cursor { s: data, i: 0 };
    let Some(mut buf) = StrBuf::new() else {
        return E_OUTOFMEMORY;
    };
    let mut hr = get_word(&mut it, &mut buf);
    if hr < 0 {
        return hr;
    }
    while it.cur() != 0 {
        if buf.len == 0 {
            hr = DISP_E_EXCEPTION;
            break;
        }
        let Some(&(_, key)) = ROOT_KEYS.iter().find(|(n, _)| buf.eq_ignore_case(n)) else {
            hr = DISP_E_EXCEPTION;
            break;
        };
        hr = get_word(&mut it, &mut buf);
        if hr < 0 {
            break;
        }
        if !buf.is(b'{') {
            hr = DISP_E_EXCEPTION;
            break;
        }
        hr = process_key(reg, &mut it, predefined(key), &mut buf, do_register);
        if hr < 0 {
            break;
        }
        hr = get_word(&mut it, &mut buf);
        if hr < 0 {
            break;
        }
    }
    hr
}

#[cfg(feature = "dll")]
pub use win32::*;

#[cfg(feature = "dll")]
mod win32 {
    use super::*;

    const KEY_READ: u32 = 0x2_0019;
    const KEY_WRITE: u32 = 0x2_0006;
    const LOAD_LIBRARY_AS_DATAFILE: u32 = 0x2;
    const CP_ACP: u32 = 0;

    extern "system" {
        fn RegCreateKeyW(key: HKEY, sub: *const u16, out: *mut HKEY) -> i32;
        fn RegOpenKeyW(key: HKEY, sub: *const u16, out: *mut HKEY) -> i32;
        fn RegOpenKeyExW(key: HKEY, sub: *const u16, opt: u32, sam: u32, out: *mut HKEY) -> i32;
        fn RegEnumKeyExW(
            key: HKEY,
            index: u32,
            name: *mut u16,
            cch: *mut u32,
            reserved: *mut u32,
            class: *mut u16,
            cch_class: *mut u32,
            time: *mut c_void,
        ) -> i32;
        fn RegSetValueExW(
            key: HKEY,
            name: *const u16,
            reserved: u32,
            ty: u32,
            data: *const u8,
            len: u32,
        ) -> i32;
        fn RegDeleteKeyW(key: HKEY, sub: *const u16) -> i32;
        fn RegCloseKey(key: HKEY) -> i32;
    }

    extern "system" {
        fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
        fn FreeLibrary(h: *mut c_void) -> i32;
        fn FindResourceW(h: *mut c_void, name: *const u16, ty: *const u16) -> *mut c_void;
        fn LoadResource(h: *mut c_void, res: *mut c_void) -> *mut c_void;
        fn SizeofResource(h: *mut c_void, res: *mut c_void) -> u32;
        // lib.rs declares it too: the same signature, or rustc warns of a clash.
        fn MultiByteToWideChar(cp: u32, flags: u32, s: *const i8, n: i32, w: *mut u16, cch: i32)
            -> i32;
        fn GetLastError() -> u32;
    }

    /// The process registry, through advapi32.
    pub struct Win32Registry;

    impl Registry for Win32Registry {
        unsafe fn create_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32 {
            RegCreateKeyW(parent, sub, out)
        }
        unsafe fn open_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32 {
            RegOpenKeyW(parent, sub, out)
        }
        unsafe fn set_value(
            &mut self,
            key: HKEY,
            name: *const u16,
            ty: u32,
            data: *const u8,
            len: u32,
        ) -> i32 {
            RegSetValueExW(key, name, 0, ty, data, len)
        }
        unsafe fn delete_key(&mut self, parent: HKEY, sub: *const u16) -> i32 {
            RegDeleteKeyW(parent, sub)
        }
        unsafe fn delete_tree(&mut self, parent: HKEY, sub: *const u16) -> i32 {
            // statreg.h's RegDeleteTreeX, enumerating the CHILD (see the module docs).
            let mut child: HKEY = core::ptr::null_mut();
            let lres = RegOpenKeyExW(parent, sub, 0, KEY_READ | KEY_WRITE, &mut child);
            if lres != 0 {
                return lres;
            }
            let mut name = [0u16; 256];
            loop {
                let mut cch = name.len() as u32;
                let null = core::ptr::null_mut();
                if RegEnumKeyExW(child, 0, name.as_mut_ptr(), &mut cch, null, null as _, null, null as _)
                    != 0
                {
                    break;
                }
                let lres = self.delete_tree(child, name.as_ptr());
                if lres != 0 {
                    RegCloseKey(child);
                    return lres;
                }
            }
            RegCloseKey(child);
            RegDeleteKeyW(parent, sub)
        }
        unsafe fn close_key(&mut self, key: HKEY) -> i32 {
            RegCloseKey(key)
        }
    }

    impl RegObject {
        /// `ResourceRegister(resFileName, nID, szType)` / `ResourceUnregister`
        /// (`resource_register`): the script is the ANSI resource `szType`/`nID` of the file.
        pub unsafe fn resource_register(
            &self,
            res_file: *const u16,
            id: u32,
            ty: *const u16,
            do_register: bool,
        ) -> HRESULT {
            let hins = LoadLibraryExW(res_file, core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
            if hins.is_null() {
                return hresult_from_win32(GetLastError() as i32);
            }
            // MAKEINTRESOURCEW(nID)
            let src = FindResourceW(hins, id as u16 as usize as *const u16, ty);
            let hr = if src.is_null() {
                hresult_from_win32(GetLastError() as i32)
            } else {
                let data = LoadResource(hins, src) as *const i8;
                let reslen = SizeofResource(hins, src) as i32;
                if data.is_null() {
                    hresult_from_win32(GetLastError() as i32)
                } else {
                    let len = MultiByteToWideChar(CP_ACP, 0, data, reslen, core::ptr::null_mut(), 0)
                        as usize
                        + 1;
                    let w = malloc(len * 2) as *mut u16;
                    if w.is_null() {
                        FreeLibrary(hins);
                        return E_OUTOFMEMORY;
                    }
                    core::ptr::write_bytes(w, 0, len);
                    MultiByteToWideChar(CP_ACP, 0, data, reslen, w, len as i32);
                    *w.add(len - 1) = 0;
                    let hr = self.string_register(&mut Win32Registry, w, do_register);
                    free(w as *mut c_void);
                    hr
                }
            };
            FreeLibrary(hins);
            hr
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }
    unsafe fn s(p: *const u16) -> String {
        String::from_utf16(core::slice::from_raw_parts(p, wcslen(p))).unwrap()
    }

    /// Keys as `ROOT\a\b` paths; a handle is an index into `open`.
    #[derive(Default)]
    struct Fake {
        keys: BTreeMap<String, BTreeMap<String, (u32, Vec<u8>)>>,
        open: Vec<String>,
        log: Vec<String>,
    }

    impl Fake {
        fn path(&self, parent: HKEY) -> String {
            let p = parent as isize;
            if p < 0 || p as u32 >= 0x8000_0000 {
                format!("R{:x}", p as u32 & 0xF)
            } else {
                self.open[p as usize - 1].clone()
            }
        }
        fn handle(&mut self, path: String) -> HKEY {
            self.open.push(path);
            self.open.len() as isize as HKEY
        }
    }

    impl Registry for Fake {
        unsafe fn create_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32 {
            let p = format!("{}\\{}", self.path(parent), s(sub));
            self.keys.entry(p.clone()).or_default();
            *out = self.handle(p);
            0
        }
        unsafe fn open_key(&mut self, parent: HKEY, sub: *const u16, out: *mut HKEY) -> i32 {
            let p = format!("{}\\{}", self.path(parent), s(sub));
            if !self.keys.contains_key(&p) {
                return 2;
            }
            *out = self.handle(p);
            0
        }
        unsafe fn set_value(&mut self, key: HKEY, name: *const u16, ty: u32, d: *const u8, n: u32) -> i32 {
            let name = if name.is_null() { "@".into() } else { s(name) };
            let p = self.path(key);
            let data = core::slice::from_raw_parts(d, n as usize).to_vec();
            self.keys.get_mut(&p).unwrap().insert(name, (ty, data));
            0
        }
        unsafe fn delete_key(&mut self, parent: HKEY, sub: *const u16) -> i32 {
            let p = format!("{}\\{}", self.path(parent), s(sub));
            self.log.push(format!("delete {p}"));
            if self.keys.keys().any(|k| k.starts_with(&format!("{p}\\"))) {
                return 5; // RegDeleteKeyW refuses a key with subkeys
            }
            if self.keys.remove(&p).is_some() { 0 } else { 2 }
        }
        unsafe fn delete_tree(&mut self, parent: HKEY, sub: *const u16) -> i32 {
            let p = format!("{}\\{}", self.path(parent), s(sub));
            let before = self.keys.len();
            self.keys.retain(|k, _| k != &p && !k.starts_with(&format!("{p}\\")));
            if self.keys.len() == before { 2 } else { 0 }
        }
        unsafe fn close_key(&mut self, _key: HKEY) -> i32 {
            0
        }
    }

    fn sz(v: &(u32, Vec<u8>)) -> String {
        assert_eq!(v.0, REG_SZ);
        let u: Vec<u16> = v.1.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16(&u[..u.len() - 1]).unwrap()
    }

    // mmcshext.rgs, verbatim.
    const MMCSHEXT: &str = "HKCR\n{\n    NoRemove CLSID\n    {\n        ForceRemove {7A80E4A8-8005-11D2-BCF8-00C04F72C717} = s 'MMC Shell Extension'\n        {\n            InprocServer32 = s '%MODULE%' { val ThreadingModel = s 'Apartment' }\n        }\n    }\n\n    NoRemove mscfile\n    {\n        DefaultIcon =  s '%%1'\n        NoRemove shellex\n        {\n            IconHandler = s '{7A80E4A8-8005-11D2-BCF8-00C04F72C717}'\n        }\n    }\n}\n\nHKLM\n{\n    NoRemove SOFTWARE\n    {\n        NoRemove Microsoft\n        {\n            NoRemove Windows\n            {\n                NoRemove CurrentVersion\n                {\n                    NoRemove 'Shell Extensions'\n                    {\n                        NoRemove Approved\n                        {\n                            val {7A80E4A8-8005-11D2-BCF8-00C04F72C717} = s 'MMC Icon Handler'\n                        }\n                    }\n                }\n            }\n        }\n    } \n}\n";
    const CLSID: &str = "{7A80E4A8-8005-11D2-BCF8-00C04F72C717}";

    fn registrar() -> RegObject {
        let mut r = RegObject::new().unwrap();
        unsafe {
            // CommonInitRegistrar's order: APPID, Module, Module_Raw.
            r.AddReplacement(w("APPID").as_ptr(), w("").as_ptr());
            r.AddReplacement(w("Module").as_ptr(), w("C:\\ReactOS\\system32\\mmcshext.dll").as_ptr());
            r.AddReplacement(w("Module_Raw").as_ptr(), w("C:\\ReactOS\\system32\\mmcshext.dll").as_ptr());
        }
        r
    }

    #[test]
    fn registers_mmcshext_rgs() {
        let mut reg = Fake::default();
        let hr = unsafe { registrar().string_register(&mut reg, w(MMCSHEXT).as_ptr(), true) };
        assert_eq!(hr, S_OK);
        let k = |p: &str| reg.keys.get(p).unwrap_or_else(|| panic!("no key {p}"));
        assert_eq!(sz(&k(&format!("R0\\CLSID\\{CLSID}"))["@"]), "MMC Shell Extension");
        let inproc = k(&format!("R0\\CLSID\\{CLSID}\\InprocServer32"));
        assert_eq!(sz(&inproc["@"]), "C:\\ReactOS\\system32\\mmcshext.dll");
        assert_eq!(sz(&inproc["ThreadingModel"]), "Apartment");
        assert_eq!(sz(&k("R0\\mscfile\\DefaultIcon")["@"]), "%1");
        assert_eq!(sz(&k("R0\\mscfile\\shellex\\IconHandler")["@"]), CLSID);
        let approved = k("R2\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Shell Extensions\\Approved");
        assert_eq!(sz(&approved[CLSID]), "MMC Icon Handler");
    }

    #[test]
    fn unregister_removes_only_what_is_not_noremove() {
        let mut reg = Fake::default();
        let r = registrar();
        unsafe {
            assert_eq!(r.string_register(&mut reg, w(MMCSHEXT).as_ptr(), true), S_OK);
            assert_eq!(r.string_register(&mut reg, w(MMCSHEXT).as_ptr(), false), S_OK);
        }
        // The CLSID key and its InprocServer32 are gone (child first), DefaultIcon and
        // IconHandler too; every NoRemove parent survives.
        assert!(!reg.keys.keys().any(|k| k.contains(CLSID)));
        assert!(!reg.keys.contains_key("R0\\mscfile\\DefaultIcon"));
        assert!(!reg.keys.contains_key("R0\\mscfile\\shellex\\IconHandler"));
        assert!(reg.keys.contains_key("R0\\CLSID"));
        assert!(reg.keys.contains_key("R0\\mscfile\\shellex"));
        assert!(reg.keys.contains_key("R2\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Shell Extensions\\Approved"));
    }

    #[test]
    fn force_remove_clears_a_stale_tree_first() {
        let mut reg = Fake::default();
        reg.keys.insert(format!("R0\\CLSID\\{CLSID}"), BTreeMap::new());
        reg.keys.insert(format!("R0\\CLSID\\{CLSID}\\Stale"), BTreeMap::new());
        let hr = unsafe { registrar().string_register(&mut reg, w(MMCSHEXT).as_ptr(), true) };
        assert_eq!(hr, S_OK);
        assert!(!reg.keys.contains_key(&format!("R0\\CLSID\\{CLSID}\\Stale")));
    }

    #[test]
    fn value_types_and_replacement_case() {
        let mut reg = Fake::default();
        let script = "HKCU { K { val a = d '0x10' val b = d '-2' val c = b 'DEADbe' val e = e '%module%' } }";
        let hr = unsafe { registrar().string_register(&mut reg, w(script).as_ptr(), true) };
        assert_eq!(hr, S_OK);
        let k = &reg.keys["R1\\K"];
        assert_eq!(k["a"], (REG_DWORD, 16u32.to_le_bytes().to_vec()));
        assert_eq!(k["b"], (REG_DWORD, (-2i32 as u32).to_le_bytes().to_vec()));
        assert_eq!(k["c"], (REG_BINARY, vec![0xDE, 0xAD, 0xBE]));
        assert_eq!(k["e"].0, REG_EXPAND_SZ);
    }

    #[test]
    fn malformed_scripts_fail_and_a_failed_register_is_undone() {
        let mut reg = Fake::default();
        let r = registrar();
        unsafe {
            for bad in ["HKCR { A = s 'x' ", "HKXX { }", "HKCR { A = s '%NOPE%' }", "HKCR { A = q 'x' }"] {
                assert_eq!(r.string_register(&mut reg, w(bad).as_ptr(), true), DISP_E_EXCEPTION, "{bad}");
            }
            // A = s 'x' was written, then `q` failed: the rollback removed A again.
            assert!(!reg.keys.contains_key("R0\\A"));
        }
    }
}
