//! ATL's STATIC registrar — `ATL::CRegObject` from sdk/lib/atl/statreg.h — ported to Rust for
//! shell32's DllRegisterServer / DllUnregisterServer.
//!
//! WHY NOT atl100.dll's registrar: it is Wine's parser, and it has no `e` (REG_EXPAND_SZ) value
//! type — its `switch` knows only `s`, `d` and `b` and answers anything else with
//! DISP_E_EXCEPTION. shell32's .rgs scripts use `= e` 54 times, so registering through it failed
//! outright (0x80020009, measured by tools/payloads/regdiff). It also parses `d` in decimal only
//! and doubles `''` inside strings, both of which statreg.h does differently. statreg.h is what
//! shell32.cpp compiled in, so it is what this reproduces — including its quirks, which are
//! marked where they are kept on purpose.
//!
//! Heap and all: statreg.h grows its buffers with HeapAlloc/HeapReAlloc, and this crate has no
//! `alloc`, so the port does the same.
#![allow(non_snake_case)]

use crate::island::{
    FindResourceW, FreeLibrary, GetLastError, GetProcessHeap, HeapAlloc, HeapFree, HeapReAlloc,
    LoadLibraryExW, LoadResource, MultiByteToWideChar, RegCloseKey, RegCreateKeyW, RegDeleteKeyW,
    RegEnumKeyExW, RegOpenKeyExW, RegOpenKeyW, RegSetValueExW, SizeofResource, lstrcmpiW, lstrlenW,
    DWORD, HKEY, HRESULT, LPCWSTR, SIZE_T, UINT, WCHAR,
};
use core::ptr::{null, null_mut};

extern "C" {
    fn iswspace(c: i32) -> i32;
    fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    fn wcstoul(s: *const WCHAR, end: *mut *mut WCHAR, base: i32) -> u32;
    fn wcstol(s: *const WCHAR, end: *mut *mut WCHAR, base: i32) -> i32;
    fn _memicmp(a: *const core::ffi::c_void, b: *const core::ffi::c_void, n: usize) -> i32;
}

const S_OK: HRESULT = 0;
const E_OUTOFMEMORY: HRESULT = 0x8007_000E_u32 as HRESULT;
const DISP_E_EXCEPTION: HRESULT = 0x8002_0009_u32 as HRESULT;
const ERROR_SUCCESS: i32 = 0;
const REG_SZ: DWORD = 1;
const REG_EXPAND_SZ: DWORD = 2;
const REG_BINARY: DWORD = 3;
const REG_DWORD: DWORD = 4;
const KEY_READ: DWORD = 0x0002_0019;
const KEY_WRITE: DWORD = 0x0002_0006;
const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x0000_0002;
const CP_ACP: u32 = 0;
const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;

/// winreg.h's predefined keys: `(HKEY)(ULONG_PTR)((LONG)0x8000000N)` — sign-extended on 64-bit.
const fn predef(n: u32) -> HKEY {
    (n as i32 as isize) as HKEY
}

#[inline]
fn failed(hr: HRESULT) -> bool {
    hr < 0
}

#[inline]
fn hresult_from_win32(x: u32) -> HRESULT {
    if x as i32 <= 0 { x as HRESULT } else { ((x & 0xFFFF) | 0x8007_0000) as HRESULT }
}

const fn w<const N: usize>(s: &str) -> [WCHAR; N] {
    let b = s.as_bytes();
    let mut out = [0 as WCHAR; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as WCHAR;
        i += 1;
    }
    out
}

static W_NOREMOVE: [WCHAR; 9] = w("NoRemove");
static W_FORCEREMOVE: [WCHAR; 12] = w("ForceRemove");
static W_DELETE: [WCHAR; 7] = w("Delete");
static W_VAL: [WCHAR; 4] = w("val");
static W_PERCENT: [WCHAR; 2] = w("%");

static ROOT_KEYS: [(&[WCHAR], u32); 14] = [
    (&w::<18>("HKEY_CLASSES_ROOT"), 0x8000_0000),
    (&w::<18>("HKEY_CURRENT_USER"), 0x8000_0001),
    (&w::<19>("HKEY_LOCAL_MACHINE"), 0x8000_0002),
    (&w::<11>("HKEY_USERS"), 0x8000_0003),
    (&w::<22>("HKEY_PERFORMANCE_DATA"), 0x8000_0004),
    (&w::<14>("HKEY_DYN_DATA"), 0x8000_0006),
    (&w::<20>("HKEY_CURRENT_CONFIG"), 0x8000_0005),
    (&w::<5>("HKCR"), 0x8000_0000),
    (&w::<5>("HKCU"), 0x8000_0001),
    (&w::<5>("HKLM"), 0x8000_0002),
    (&w::<4>("HKU"), 0x8000_0003),
    (&w::<5>("HKPD"), 0x8000_0004),
    (&w::<5>("HKDD"), 0x8000_0006),
    (&w::<5>("HKCC"), 0x8000_0005),
];

/// statreg.h `strbuf`: a NUL-terminated, heap-grown UTF-16 buffer.
struct StrBuf {
    str_: *mut WCHAR,
    alloc: DWORD,
    len: DWORD,
}

impl StrBuf {
    unsafe fn init() -> Result<StrBuf, HRESULT> {
        let p = HeapAlloc(GetProcessHeap(), 0, 128 * 2) as *mut WCHAR;
        if p.is_null() {
            return Err(E_OUTOFMEMORY);
        }
        Ok(StrBuf { str_: p, alloc: 128, len: 0 })
    }

    unsafe fn write(&mut self, s: *const WCHAR, len: i32) -> HRESULT {
        let len = if len == -1 { lstrlenW(s) } else { len } as DWORD;
        if self.len + len + 1 >= self.alloc {
            self.alloc = (self.len + len) * 2;
            let p = HeapReAlloc(GetProcessHeap(), 0, self.str_.cast(), self.alloc as SIZE_T * 2);
            if p.is_null() {
                return E_OUTOFMEMORY;
            }
            self.str_ = p as *mut WCHAR;
        }
        core::ptr::copy_nonoverlapping(s, self.str_.add(self.len as usize), len as usize);
        self.len += len;
        *self.str_.add(self.len as usize) = 0;
        S_OK
    }

    #[inline]
    unsafe fn at(&self, i: usize) -> WCHAR {
        *self.str_.add(i)
    }
}

impl Drop for StrBuf {
    fn drop(&mut self) {
        unsafe { HeapFree(GetProcessHeap(), 0, self.str_.cast()) };
    }
}

/// `CRegObject` with its `rep_list`. statreg.h PREPENDS each replacement and searches from the
/// head, so the newest one wins; `find` walks this array backwards for the same effect. The
/// strings are borrowed, not copied: every caller's replacements outlive the registrar.
pub struct RegObject {
    reps: [(LPCWSTR, LPCWSTR); 4],
    count: usize,
}

impl RegObject {
    pub const fn new() -> Self {
        RegObject { reps: [(null(), null()); 4], count: 0 }
    }

    /// CRegObject::AddReplacement. Four slots cover ATL's APPID + Module + Module_Raw; a fifth
    /// is E_OUTOFMEMORY, which is what statreg.h returns when it cannot store one.
    pub fn add_replacement(&mut self, key: LPCWSTR, item: LPCWSTR) -> HRESULT {
        if self.count == self.reps.len() {
            return E_OUTOFMEMORY;
        }
        self.reps[self.count] = (key, item);
        self.count += 1;
        S_OK
    }

    unsafe fn find(&self, name: *const WCHAR, len: usize) -> Option<LPCWSTR> {
        for &(key, item) in self.reps[..self.count].iter().rev() {
            if lstrlenW(key) as usize == len && _memicmp(name.cast(), key.cast(), len * 2) == 0 {
                return Some(item);
            }
        }
        None
    }

    /// CRegObject::ResourceRegister / ResourceUnregister (`MAKEINTRESOURCEW(nID)`).
    pub unsafe fn resource_register(
        &self, res_file: LPCWSTR, id: UINT, res_type: LPCWSTR, do_register: bool,
    ) -> HRESULT {
        let hins = LoadLibraryExW(res_file, null_mut(), LOAD_LIBRARY_AS_DATAFILE);
        if hins.is_null() {
            return hresult_from_win32(GetLastError());
        }
        let hr;
        let src = FindResourceW(hins, id as usize as LPCWSTR, res_type);
        if !src.is_null() {
            let regstra = LoadResource(hins, src);
            let reslen = SizeofResource(hins, src);
            if !regstra.is_null() {
                let len = MultiByteToWideChar(CP_ACP, 0, regstra as *const _, reslen as i32, null_mut(), 0) + 1;
                let regstrw =
                    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len as SIZE_T * 2) as *mut WCHAR;
                if regstrw.is_null() {
                    // statreg.h returns here without FreeLibrary; kept.
                    return E_OUTOFMEMORY;
                }
                MultiByteToWideChar(CP_ACP, 0, regstra as *const _, reslen as i32, regstrw, len);
                *regstrw.add(len as usize - 1) = 0;
                hr = self.string_register(regstrw, do_register);
                HeapFree(GetProcessHeap(), 0, regstrw.cast());
            } else {
                hr = hresult_from_win32(GetLastError());
            }
        } else {
            hr = hresult_from_win32(GetLastError());
        }
        FreeLibrary(hins);
        hr
    }

    unsafe fn string_register(&self, data: LPCWSTR, do_register: bool) -> HRESULT {
        let mut buf = match StrBuf::init() {
            Ok(b) => b,
            Err(hr) => return hr,
        };
        let mut hr = self.do_preprocess(data, &mut buf);
        if !failed(hr) {
            hr = do_process_root_key(buf.str_, do_register);
            // A failed register is rolled back by unregistering the same script.
            if failed(hr) && do_register {
                do_process_root_key(buf.str_, false);
            }
        }
        hr
    }

    /// Expands `%name%` from the replacement list; `%%` is a literal percent.
    unsafe fn do_preprocess(&self, data: LPCWSTR, buf: &mut StrBuf) -> HRESULT {
        let mut iter2 = data;
        let mut iter = wcschr(data, b'%' as WCHAR) as *const WCHAR;
        while !iter.is_null() {
            let hr = buf.write(iter2, iter.offset_from(iter2) as i32);
            if failed(hr) {
                return hr;
            }
            iter = iter.add(1);
            iter2 = iter;
            if *iter2 == 0 {
                return DISP_E_EXCEPTION;
            }
            iter = wcschr(iter2, b'%' as WCHAR);
            if iter.is_null() {
                return DISP_E_EXCEPTION;
            }
            let hr = if iter == iter2 {
                buf.write(W_PERCENT.as_ptr(), 1)
            } else {
                match self.find(iter2, iter.offset_from(iter2) as usize) {
                    Some(item) => buf.write(item, -1),
                    None => return DISP_E_EXCEPTION,
                }
            };
            if failed(hr) {
                return hr;
            }
            iter = iter.add(1);
            iter2 = iter;
            iter = wcschr(iter, b'%' as WCHAR);
        }
        buf.write(iter2, -1)
    }
}

#[inline]
unsafe fn is_space(c: WCHAR) -> bool {
    iswspace(c as i32) != 0
}

/// One token: `}` or `=` alone, a `'quoted'` string (no escape for `'` — statreg.h has none,
/// unlike Wine's `''`), or a run of non-space. Skips the whitespace after it.
unsafe fn get_word(str_: &mut *const WCHAR, buf: &mut StrBuf) -> HRESULT {
    let mut iter2 = *str_;
    buf.len = 0;
    *buf.str_ = 0;

    while is_space(*iter2) {
        iter2 = iter2.add(1);
    }
    let mut iter = iter2;
    if *iter == 0 {
        *str_ = iter;
        return S_OK;
    }

    if *iter == b'}' as WCHAR || *iter == b'=' as WCHAR {
        let hr = buf.write(iter, 1);
        if failed(hr) {
            return hr;
        }
        iter = iter.add(1);
    } else if *iter == b'\'' as WCHAR {
        iter = iter.add(1);
        iter2 = iter;
        iter = wcschr(iter, b'\'' as WCHAR);
        if iter.is_null() {
            *str_ = iter;
            return DISP_E_EXCEPTION;
        }
        let hr = buf.write(iter2, iter.offset_from(iter2) as i32);
        if failed(hr) {
            return hr;
        }
        iter = iter.add(1);
    } else {
        while *iter != 0 && !is_space(*iter) {
            iter = iter.add(1);
        }
        let hr = buf.write(iter2, iter.offset_from(iter2) as i32);
        if failed(hr) {
            return hr;
        }
    }

    while is_space(*iter) {
        iter = iter.add(1);
    }
    *str_ = iter;
    S_OK
}

fn hex_to_bin(a: WCHAR) -> u32 {
    let a = a as u32;
    match a {
        0x30..=0x39 => a - 0x30,
        0x41..=0x46 => a - 0x41 + 10,
        0x61..=0x66 => a - 0x61 + 10,
        _ => 0, // ATLASSERT(false) in a checked build; release returns 0
    }
}

/// CRegObject::RegDeleteTreeX, KEEPING ITS DEFECT: the loop enumerates `parent_key`, not the
/// opened child, so for any key that exists it looks for the parent's first subkey name UNDER
/// the child, normally fails, and returns without deleting. ForceRemove and Delete therefore
/// never removed an existing key under shell32.cpp; "fixing" it here would start deleting keys
/// that the setup INFs populate before DllRegisterServer runs. The child handle the C leaked on
/// that path is closed — that changes nothing a registry reader can see.
unsafe fn reg_delete_tree_x(parent_key: HKEY, sub_key: LPCWSTR) -> i32 {
    let mut child: HKEY = null_mut();
    let lres = RegOpenKeyExW(parent_key, sub_key, 0, KEY_READ | KEY_WRITE, &mut child);
    if lres != ERROR_SUCCESS {
        return lres;
    }
    let mut name = [0 as WCHAR; 256];
    let mut size: DWORD = 256;
    let mut time = core::mem::zeroed();
    while RegEnumKeyExW(parent_key, 0, name.as_mut_ptr(), &mut size, null_mut(), null_mut(), null_mut(), &mut time)
        == ERROR_SUCCESS
    {
        let lres = reg_delete_tree_x(child, name.as_ptr());
        if lres != ERROR_SUCCESS {
            RegCloseKey(child);
            return lres;
        }
        size = 256;
    }
    RegCloseKey(child);
    RegDeleteKeyW(parent_key, sub_key)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyType {
    Normal,
    NoRemove,
    IsVal,
    ForceRemove,
    DoDelete,
}

unsafe fn do_process_key(
    pstr: &mut *const WCHAR, parent_key: HKEY, buf: &mut StrBuf, do_register: bool,
) -> HRESULT {
    let mut iter = *pstr;
    let mut hkey: HKEY = null_mut();
    let mut key_type = KeyType::Normal;

    let mut hres = get_word(&mut iter, buf);
    if failed(hres) {
        return hres;
    }
    let mut name = match StrBuf::init() {
        Ok(b) => b,
        Err(hr) => return hr,
    };

    while buf.at(1) != 0 || buf.at(0) != b'}' as WCHAR {
        key_type = if lstrcmpiW(buf.str_, W_NOREMOVE.as_ptr()) == 0 {
            KeyType::NoRemove
        } else if lstrcmpiW(buf.str_, W_FORCEREMOVE.as_ptr()) == 0 {
            KeyType::ForceRemove
        } else if lstrcmpiW(buf.str_, W_VAL.as_ptr()) == 0 {
            KeyType::IsVal
        } else if lstrcmpiW(buf.str_, W_DELETE.as_ptr()) == 0 {
            KeyType::DoDelete
        } else {
            KeyType::Normal
        };

        if key_type != KeyType::Normal {
            hres = get_word(&mut iter, buf);
            if failed(hres) {
                break;
            }
        }

        if do_register {
            if key_type == KeyType::IsVal {
                hkey = parent_key;
                hres = name.write(buf.str_, -1);
                if failed(hres) {
                    return hres;
                }
            } else if key_type == KeyType::DoDelete {
                reg_delete_tree_x(parent_key, buf.str_);
            } else {
                if key_type == KeyType::ForceRemove {
                    reg_delete_tree_x(parent_key, buf.str_);
                }
                let lres = RegCreateKeyW(parent_key, buf.str_, &mut hkey);
                if lres != ERROR_SUCCESS {
                    hres = hresult_from_win32(lres as u32);
                    break;
                }
            }
        } else if key_type != KeyType::IsVal && key_type != KeyType::DoDelete {
            hres = name.write(buf.str_, -1);
            if failed(hres) {
                return hres;
            }
            // A missing key is not an error on unregister; hkey stays null.
            RegOpenKeyW(parent_key, buf.str_, &mut hkey);
        }

        if key_type != KeyType::DoDelete && *iter == b'=' as WCHAR {
            iter = iter.add(1);
            hres = get_word(&mut iter, buf);
            if failed(hres) {
                break;
            }
            if buf.len != 1 {
                hres = DISP_E_EXCEPTION;
                break;
            }
            if do_register {
                let value_name = if name.len != 0 { name.str_ as LPCWSTR } else { null() };
                let kind = buf.at(0);
                if kind == b's' as WCHAR || kind == b'e' as WCHAR {
                    hres = get_word(&mut iter, buf);
                    if !failed(hres) {
                        let ty = if kind == b's' as WCHAR { REG_SZ } else { REG_EXPAND_SZ };
                        let lres = RegSetValueExW(
                            hkey, value_name, 0, ty, buf.str_ as *const u8,
                            ((lstrlenW(buf.str_) + 1) * 2) as DWORD,
                        );
                        if lres != ERROR_SUCCESS {
                            hres = hresult_from_win32(lres as u32);
                        }
                    }
                } else if kind == b'd' as WCHAR {
                    hres = get_word(&mut iter, buf);
                    if !failed(hres) {
                        let mut end: *mut WCHAR = null_mut();
                        let dw: DWORD = if (buf.at(0) == b'0' as WCHAR && buf.at(1) == b'x' as WCHAR)
                            || (buf.at(0) == b'&' as WCHAR && buf.at(1) == b'H' as WCHAR)
                        {
                            wcstoul(buf.str_.add(2), &mut end, 16)
                        } else {
                            wcstol(buf.str_, &mut end, 10) as DWORD
                        };
                        let lres = RegSetValueExW(
                            hkey, value_name, 0, REG_DWORD, &dw as *const DWORD as *const u8, 4,
                        );
                        if lres != ERROR_SUCCESS {
                            hres = hresult_from_win32(lres as u32);
                        }
                    }
                } else if kind == b'b' as WCHAR {
                    hres = get_word(&mut iter, buf);
                    if !failed(hres) {
                        let count = buf.len;
                        if count & 1 != 0 {
                            return DISP_E_EXCEPTION;
                        }
                        let count = count / 2;
                        // Decoded in place over the hex text, like the C: byte i is written
                        // after reading WCHARs 2i and 2i+1, which sit past it.
                        let bytes = buf.str_ as *mut u8;
                        for i in 0..count as usize {
                            *bytes.add(i) =
                                ((hex_to_bin(buf.at(i * 2)) << 4) | hex_to_bin(buf.at(i * 2 + 1))) as u8;
                        }
                        let lres = RegSetValueExW(hkey, value_name, 0, REG_BINARY, bytes, count);
                        if lres != ERROR_SUCCESS {
                            hres = hresult_from_win32(lres as u32);
                        }
                    }
                } else {
                    hres = DISP_E_EXCEPTION;
                }
                if failed(hres) {
                    break;
                }
            } else {
                if *iter == b'-' as WCHAR {
                    iter = iter.add(1);
                }
                hres = get_word(&mut iter, buf);
                if failed(hres) {
                    break;
                }
            }
        } else if key_type == KeyType::IsVal {
            hres = DISP_E_EXCEPTION;
            break;
        }

        if key_type != KeyType::IsVal
            && key_type != KeyType::DoDelete
            && *iter == b'{' as WCHAR
            && is_space(*iter.add(1))
        {
            hres = get_word(&mut iter, buf);
            if failed(hres) {
                break;
            }
            hres = do_process_key(&mut iter, hkey, buf, do_register);
            if failed(hres) {
                break;
            }
        }

        if !do_register && (key_type == KeyType::Normal || key_type == KeyType::ForceRemove) {
            RegDeleteKeyW(parent_key, name.str_);
        }

        if !hkey.is_null() && key_type != KeyType::IsVal {
            RegCloseKey(hkey);
        }
        hkey = null_mut();
        name.len = 0;

        hres = get_word(&mut iter, buf);
        if failed(hres) {
            break;
        }
    }

    if !hkey.is_null() && key_type != KeyType::IsVal {
        RegCloseKey(hkey);
    }
    *pstr = iter;
    hres
}

unsafe fn do_process_root_key(data: LPCWSTR, do_register: bool) -> HRESULT {
    let mut iter = data;
    let mut buf = match StrBuf::init() {
        Ok(b) => b,
        Err(hr) => return hr,
    };
    let mut hr = get_word(&mut iter, &mut buf);
    if failed(hr) {
        return hr;
    }

    while *iter != 0 {
        if buf.len == 0 {
            hr = DISP_E_EXCEPTION;
            break;
        }
        let Some(&(_, root)) = ROOT_KEYS.iter().find(|(n, _)| lstrcmpiW(buf.str_, n.as_ptr()) == 0)
        else {
            hr = DISP_E_EXCEPTION;
            break;
        };
        hr = get_word(&mut iter, &mut buf);
        if failed(hr) {
            break;
        }
        if buf.at(1) != 0 || buf.at(0) != b'{' as WCHAR {
            hr = DISP_E_EXCEPTION;
            break;
        }
        hr = do_process_key(&mut iter, predef(root), &mut buf, do_register);
        if failed(hr) {
            break;
        }
        hr = get_word(&mut iter, &mut buf);
        if failed(hr) {
            break;
        }
    }
    hr
}
