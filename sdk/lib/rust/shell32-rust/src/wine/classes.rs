//! File-type / class mapping over HKEY_CLASSES_ROOT — translation of
//! dll/win32/shell32/wine/classes.c (657 LOC).
//!
//! All 13 exports are plain cdecl (`EXTERN_C`/no-WINAPI) internal helpers consumed by the
//! shell32 C++ side, so they are `#[no_mangle] pub unsafe extern "C"` → i386 symbols
//! `_HCR_GetClassNameA` … `_HLM_GetIconW` (leading `_` = cdecl decoration, NO `@N`).
//! Verified against the pre-flip obj's `nm -T`.
//!
//! Build config (dll/win32/shell32/CMakeLists.txt): `-D__REACTOS__`, `-DUNICODE`,
//! `-D_WIN32_WINNT=0x600`, `-D_SHELL32_`. Only the `__REACTOS__`-active branches are
//! translated (verified against the compiled obj's string table). TRACE/WARN/ERR dropped.

use crate::types::*;
use core::ffi::c_void;

// ── file-local types not (yet) in crate::types — see ADDED-TO-TYPES note at EOF ───────────
pub type ULONG = u32;
pub type PCWSTR = *const WCHAR;
pub type PWSTR = *mut WCHAR;
pub type LPOLESTR = *mut WCHAR;
pub type PINT = *mut INT;
/// PIDL — opaque; only ever passed by pointer here.
pub type LPCITEMIDLIST = *const c_void;

/// GUID/IID (guiddef.h): { Data1: DWORD, Data2: WORD, Data3: WORD, Data4: [BYTE;8] }.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GUID {
    pub Data1: DWORD,
    pub Data2: WORD,
    pub Data3: WORD,
    pub Data4: [BYTE; 8],
}
pub type REFIID = *const GUID;

/// IShellFolder vtable (shobjidl.h). Only Release(2) / BindToObject(5) / GetAttributesOf(9)
/// are called; the other slots are kept as raw pointers to preserve the layout/ordering.
#[repr(C)]
pub struct IShellFolder {
    pub lpVtbl: *const IShellFolderVtbl,
}
pub type LPSHELLFOLDER = *mut IShellFolder;

#[repr(C)]
pub struct IShellFolderVtbl {
    pub QueryInterface: PVOID,
    pub AddRef: PVOID,
    pub Release: unsafe extern "system" fn(this: LPSHELLFOLDER) -> ULONG,
    pub ParseDisplayName: PVOID,
    pub EnumObjects: PVOID,
    pub BindToObject: unsafe extern "system" fn(
        this: LPSHELLFOLDER,
        pidl: LPCITEMIDLIST,
        pbc: PVOID,
        riid: REFIID,
        ppv: *mut LPVOID,
    ) -> HRESULT,
    pub BindToStorage: PVOID,
    pub CompareIDs: PVOID,
    pub CreateViewObject: PVOID,
    pub GetAttributesOf: unsafe extern "system" fn(
        this: LPSHELLFOLDER,
        cidl: UINT,
        apidl: *mut LPCITEMIDLIST,
        rgfInOut: *mut ULONG,
    ) -> HRESULT,
    pub GetUIObjectOf: PVOID,
    pub GetDisplayNameOf: PVOID,
    pub SetNameOf: PVOID,
}

// ── constants pinned from headers ─────────────────────────────────────────────────────────
const MAX_PATH: usize = 260; // minwindef.h
const MAX_EXTENSION_LENGTH: usize = 20; // classes.c:47 (#define)
// HKEY predefined handles (winreg.h: (HKEY)(LONG_PTR)(LONG)0x8000000X).
const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as i32 as isize as usize as HKEY;
const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as i32 as isize as usize as HKEY;
const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as i32 as isize as usize as HKEY;
// KEY_READ (winnt/cmtypes.h): (STANDARD_RIGHTS_READ|KEY_QUERY_VALUE|KEY_ENUMERATE_SUB_KEYS|
//   KEY_NOTIFY) & ~SYNCHRONIZE = (0x20000|0x1|0x8|0x10) & ~0x100000 = 0x20019.
const KEY_READ: REGSAM = 0x20019;
const RRF_RT_REG_SZ: DWORD = 1 << 1; // winreg.h
const REG_EXPAND_SZ: DWORD = 2; // winreg.h
const S_FALSE: HRESULT = 1; // winerror.h
const UNICODE_NULL: WCHAR = 0; // winnt.h
// STRSAFE_E_INSUFFICIENT_BUFFER (strsafe.h): 0x8007007A.
const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;
// Resource IDs (shresdef.h).
const IDS_DESKTOP: UINT = 20;
const IDS_CONTROLPANEL: UINT = 148;
const IDS_RECYCLEBIN_FOLDER_NAME: UINT = 8964;
const IDS_ADMINISTRATIVETOOLS: UINT = 22982;
const IDS_MYCOMPUTER: UINT = 9216;
const IDS_PERSONAL: UINT = 9227;

// ── wide-string statics (UTF-16, NUL-terminated) ─────────────────────────────────────────
macro_rules! wlit {
    ($name:ident, $b:literal) => {
        static $name: [WCHAR; { $b.len() + 1 }] = {
            let s: &[u8] = $b;
            let mut a = [0u16; { $b.len() + 1 }];
            let mut i = 0;
            while i < s.len() {
                a[i] = s[i] as u16;
                i += 1;
            }
            a
        };
    };
}
wlit!(W_SHELL, b"shell");
wlit!(W_SHELL_OPEN_COMMAND, b"shell\\open\\command");
wlit!(W_OPEN, b"open");
wlit!(W_SHELL_BS, b"shell\\");
wlit!(W_BS_COMMAND, b"\\command");
wlit!(W_DEFAULTICON, b"\\DefaultIcon");
wlit!(W_LOCALIZEDSTRING, b"LocalizedString");
wlit!(W_EMPTY, b"");
wlit!(W_CALLFORATTRIBUTES, b"CallForAttributes");
wlit!(W_ATTRIBUTES, b"Attributes");
wlit!(W_CLSID_PREFIX, b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CLSID\\");
wlit!(W_SHELL_ICONS, b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Icons");
wlit!(W_SHELLFOLDER_KEY, b"CLSID\\{00021400-0000-0000-C000-000000000046}\\ShellFolder");
// length of W_SHELLFOLDER_KEY incl. NUL (6 + 38 + 12 + 1) — can't use STATIC.len() in const pos.
const SHELLFOLDER_KEY_LEN: usize = 57;

// ── imports (resolved via shell32's existing import libs at link) ─────────────────────────
extern "system" {
    // advapi32
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: PHKEY) -> LONG; // @20
    fn RegOpenKeyExA(hKey: HKEY, lpSubKey: LPCSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: PHKEY) -> LONG; // @20
    fn RegCloseKey(hKey: HKEY) -> LONG; // @4
    fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LONG; // @16
    fn RegQueryValueW(hKey: HKEY, lpSubKey: LPCWSTR, lpData: LPWSTR, lpcbData: *mut LONG) -> LONG; // @16
    fn RegQueryValueA(hKey: HKEY, lpSubKey: LPCSTR, lpData: LPSTR, lpcbData: *mut LONG) -> LONG; // @16
    fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG; // @24
    fn RegQueryValueExA(hKey: HKEY, lpValueName: LPCSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG; // @24
    fn RegGetValueW(hkey: HKEY, lpSubKey: LPCWSTR, lpValue: LPCWSTR, dwFlags: DWORD, pdwType: LPDWORD, pvData: PVOID, pcbData: LPDWORD) -> LONG; // @28
    fn RegGetValueA(hkey: HKEY, lpSubKey: LPCSTR, lpValue: LPCSTR, dwFlags: DWORD, pdwType: LPDWORD, pvData: PVOID, pcbData: LPDWORD) -> LONG; // @28
    fn RegLoadMUIStringW(hKey: HKEY, pszValue: LPCWSTR, pszOutBuf: LPWSTR, cbBuffer: DWORD, pcbData: LPDWORD, Flags: DWORD, pszDirectory: LPCWSTR) -> LONG; // @28
    fn RegLoadMUIStringA(hKey: HKEY, pszValue: LPCSTR, pszOutBuf: LPSTR, cbBuffer: DWORD, pcbData: LPDWORD, Flags: DWORD, pszDirectory: LPCSTR) -> LONG; // @28
    // kernel32
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD; // @12
    fn ExpandEnvironmentStringsA(lpSrc: LPCSTR, lpDst: LPSTR, nSize: DWORD) -> DWORD; // @12
    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: INT) -> INT; // @16
    fn LoadStringA(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPSTR, cchBufferMax: INT) -> INT; // @16
    fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // @8
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // @8
    fn lstrcpynW(lpString1: LPWSTR, lpString2: LPCWSTR, iMaxLength: INT) -> LPWSTR; // @12
    fn lstrcpynA(lpString1: LPSTR, lpString2: LPCSTR, iMaxLength: INT) -> LPSTR; // @12
    // shlwapi
    fn ParseFieldW(src: LPCWSTR, nField: DWORD, dst: LPWSTR, len: DWORD) -> DWORD; // @16 (shell32 internal)
    fn ParseFieldA(src: LPCSTR, nField: DWORD, dst: LPSTR, len: DWORD) -> DWORD; // @16
    fn PathUnquoteSpacesW(lpsz: LPWSTR) -> BOOL; // @4
    fn PathUnquoteSpacesA(lpsz: LPSTR) -> BOOL; // @4
    fn SHGetValueW(hkey: HKEY, pszSubKey: LPCWSTR, pszValue: LPCWSTR, pdwType: LPDWORD, pvData: PVOID, pcbData: LPDWORD) -> DWORD; // @24
    // shell32 (self)
    fn SHGetDesktopFolder(ppshf: *mut LPSHELLFOLDER) -> HRESULT; // @4
    // ole32
    fn StringFromCLSID(rclsid: REFIID, lplpsz: *mut LPOLESTR) -> HRESULT; // @8
    fn CoTaskMemFree(pv: LPVOID); // @4
}

extern "C" {
    // shell32 internal cdecl helpers/data (no @N)
    fn SHELL32_EnumDefaultVerbList(List: LPCWSTR, Index: UINT, Verb: LPWSTR, cchMax: usize) -> HRESULT;
    fn _ILIsPidlSimple(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILIsDesktop(pidl: LPCITEMIDLIST) -> BOOL;
    fn _ILGetGUIDPointer(pidl: LPCITEMIDLIST) -> *mut GUID;
    static shell32_hInstance: HINSTANCE;
    static CLSID_ShellDesktop: GUID;
    static CLSID_MyComputer: GUID;
    static CLSID_MyDocuments: GUID;
    static CLSID_RecycleBin: GUID;
    static CLSID_ControlPanel: GUID;
    static CLSID_AdminFolderShortcut: GUID;
    static IID_IShellFolder: GUID;
}

// ── tiny faithful helpers (CRT / strsafe machinery the obj inlined) ───────────────────────
#[inline]
unsafe fn wv<T>(p: *mut T, v: T) {
    p.write_volatile(v);
}
#[inline]
unsafe fn rv<T: Copy>(p: *const T) -> T {
    p.read_volatile()
}

/// IsEqualIID/IsEqualGUID (guiddef.h) — byte-exact GUID compare.
unsafe fn is_equal_iid(a: REFIID, b: *const GUID) -> bool {
    let x = &*a;
    let y = &*b;
    x.Data1 == y.Data1 && x.Data2 == y.Data2 && x.Data3 == y.Data3 && x.Data4 == y.Data4
}

/// `_wtoi`/`atoiW`: parse a NUL-terminated wide string to i32 (skip ws, optional sign).
unsafe fn atoi_w(mut p: *const WCHAR) -> INT {
    while matches!(*p, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D) {
        p = p.add(1);
    }
    let mut neg = false;
    match *p {
        c if c == b'+' as WCHAR => p = p.add(1),
        c if c == b'-' as WCHAR => {
            neg = true;
            p = p.add(1);
        }
        _ => {}
    }
    let mut val: INT = 0;
    loop {
        let d = *p;
        if d < b'0' as WCHAR || d > b'9' as WCHAR {
            break;
        }
        val = val.wrapping_mul(10).wrapping_add((d - b'0' as WCHAR) as INT);
        p = p.add(1);
    }
    if neg {
        -val
    } else {
        val
    }
}

/// `atoi` (narrow). Same semantics as atoi_w.
unsafe fn atoi_a(mut p: *const CHAR) -> INT {
    while matches!(*p as u8, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D) {
        p = p.add(1);
    }
    let mut neg = false;
    match *p as u8 {
        b'+' => p = p.add(1),
        b'-' => {
            neg = true;
            p = p.add(1);
        }
        _ => {}
    }
    let mut val: INT = 0;
    loop {
        let d = *p as u8;
        if !d.is_ascii_digit() {
            break;
        }
        val = val.wrapping_mul(10).wrapping_add((d - b'0') as INT);
        p = p.add(1);
    }
    if neg {
        -val
    } else {
        val
    }
}

// --- bounded wide builder (StringCchPrintfW substitute) ---
struct WBuf {
    p: *mut WCHAR,
    cap: usize,
    len: usize,
    ok: bool,
}
impl WBuf {
    unsafe fn new(p: *mut WCHAR, cap: usize) -> Self {
        WBuf { p, cap, len: 0, ok: true }
    }
    /// Append a NUL-terminated wide literal slice (stops at its NUL).
    unsafe fn cat(&mut self, lit: &[WCHAR]) {
        let mut i = 0;
        while i < lit.len() && lit[i] != 0 {
            if self.len + 1 >= self.cap {
                self.ok = false;
                return;
            }
            wv(self.p.add(self.len), lit[i]);
            self.len += 1;
            i += 1;
        }
    }
    /// Append a NUL-terminated wide C-string.
    unsafe fn cat_ptr(&mut self, mut s: *const WCHAR) {
        while *s != 0 {
            if self.len + 1 >= self.cap {
                self.ok = false;
                return;
            }
            wv(self.p.add(self.len), *s);
            self.len += 1;
            s = s.add(1);
        }
    }
    /// Append a signed decimal integer ("%d").
    unsafe fn cat_int(&mut self, v: INT) {
        let mut tmp = [0u16; 12];
        let mut n = 0usize;
        let neg = v < 0;
        let mut u: u32 = if neg { (v as i64).unsigned_abs() as u32 } else { v as u32 };
        if u == 0 {
            tmp[n] = b'0' as WCHAR;
            n += 1;
        } else {
            while u > 0 {
                tmp[n] = b'0' as WCHAR + (u % 10) as WCHAR;
                n += 1;
                u /= 10;
            }
        }
        if neg {
            if self.len + 1 >= self.cap {
                self.ok = false;
                return;
            }
            wv(self.p.add(self.len), b'-' as WCHAR);
            self.len += 1;
        }
        while n > 0 {
            n -= 1;
            if self.len + 1 >= self.cap {
                self.ok = false;
                return;
            }
            wv(self.p.add(self.len), tmp[n]);
            self.len += 1;
        }
    }
    /// NUL-terminate; return S_OK or STRSAFE_E_INSUFFICIENT_BUFFER.
    unsafe fn finish(&mut self) -> HRESULT {
        if self.len < self.cap {
            wv(self.p.add(self.len), 0);
        } else if self.cap > 0 {
            wv(self.p.add(self.cap - 1), 0);
        }
        if self.ok {
            S_OK
        } else {
            STRSAFE_E_INSUFFICIENT_BUFFER
        }
    }
}

// --- bounded narrow builder (sprintf substitute) ---
struct ABuf {
    p: *mut u8,
    cap: usize,
    len: usize,
    ok: bool,
}
impl ABuf {
    unsafe fn new(p: *mut CHAR, cap: usize) -> Self {
        ABuf { p: p as *mut u8, cap, len: 0, ok: true }
    }
    unsafe fn put(&mut self, b: u8) {
        if self.len + 1 >= self.cap {
            self.ok = false;
            return;
        }
        wv(self.p.add(self.len), b);
        self.len += 1;
    }
    /// Append a NUL-terminated byte literal.
    unsafe fn cat_bytes(&mut self, lit: &[u8]) {
        for &b in lit {
            if b == 0 {
                break;
            }
            self.put(b);
        }
    }
    /// Append a NUL-terminated narrow C-string ("%s").
    unsafe fn cat_ptr(&mut self, mut s: *const CHAR) {
        while *s != 0 {
            self.put(*s as u8);
            s = s.add(1);
        }
    }
    /// Append a NUL-terminated wide C-string truncated to its low byte ("%S").
    unsafe fn cat_wide_as_narrow(&mut self, mut s: *const WCHAR) {
        while *s != 0 {
            self.put((*s & 0xFF) as u8);
            s = s.add(1);
        }
    }
    /// Append `val` as `width` lowercase hex digits, zero-padded ("%0<width>x").
    unsafe fn cat_hex(&mut self, val: u32, width: usize) {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut tmp = [0u8; 8];
        let mut n = 0usize;
        let mut u = val;
        if u == 0 {
            tmp[n] = b'0';
            n += 1;
        } else {
            while u > 0 {
                tmp[n] = HEX[(u & 0xF) as usize];
                n += 1;
                u >>= 4;
            }
        }
        while n < width {
            tmp[n] = b'0';
            n += 1;
        }
        while n > 0 {
            n -= 1;
            self.put(tmp[n]);
        }
    }
    unsafe fn finish(&mut self) {
        if self.len < self.cap {
            wv(self.p.add(self.len), 0u8);
        } else if self.cap > 0 {
            wv(self.p.add(self.cap - 1), 0u8);
        }
    }
}

#[inline]
fn failed(hr: HRESULT) -> bool {
    hr < 0
}

/// GetRegString (classes.c:49) — RegGetValueW(RRF_RT_REG_SZ) into a WCHAR buffer.
unsafe fn get_reg_string(hkey: HKEY, sub_key: PCWSTR, name: PCWSTR, buffer: PWSTR, cch_buf: UINT) -> LONG {
    let mut cb: DWORD = (core::mem::size_of::<WCHAR>() as DWORD) * cch_buf;
    RegGetValueW(hkey, sub_key, name, RRF_RT_REG_SZ, core::ptr::null_mut(), buffer as PVOID, &mut cb)
}

// ─────────────────────────────────────────────────────────────────────────────────────────
// Exports
// ─────────────────────────────────────────────────────────────────────────────────────────

/// HCR_GetProgIdKeyOfExtension (classes.c:55)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetProgIdKeyOfExtension(
    mut szExtension: PCWSTR,
    phKey: PHKEY,
    AllowFallback: BOOL,
) -> HRESULT {
    let mut err: LONG;
    // WCHAR ext[max(1 + MAX_EXTENSION_LENGTH + 1, MAX_PATH)]
    const EXT_LEN: usize = if 1 + MAX_EXTENSION_LENGTH + 1 > MAX_PATH {
        1 + MAX_EXTENSION_LENGTH + 1
    } else {
        MAX_PATH
    };
    let mut ext = [0u16; EXT_LEN];
    let mut progid = [0u16; MAX_PATH];

    if *szExtension != b'.' as WCHAR {
        ext[0] = b'.' as WCHAR;
        lstrcpynW(ext.as_mut_ptr().add(1), szExtension, (EXT_LEN - 1) as INT);
        szExtension = ext.as_ptr();
    }
    err = get_reg_string(
        HKEY_CLASSES_ROOT,
        szExtension,
        core::ptr::null(),
        progid.as_mut_ptr(),
        progid.len() as UINT,
    );
    if err == 0 && progid[0] != UNICODE_NULL {
        err = RegOpenKeyExW(HKEY_CLASSES_ROOT, progid.as_ptr(), 0, KEY_READ, phKey);
        if err == 0 {
            return err; // A real ProgId key, return S_OK
        }
    }
    if AllowFallback != FALSE {
        err = RegOpenKeyExW(HKEY_CLASSES_ROOT, szExtension, 0, KEY_READ, phKey);
        if err == 0 {
            return S_FALSE;
        }
    }
    HRESULT_FROM_WIN32(err)
}

/// HCR_MapTypeToValueW (classes.c:82)
#[no_mangle]
pub unsafe extern "C" fn HCR_MapTypeToValueW(
    szExtension: LPCWSTR,
    szFileType: LPWSTR,
    mut len: LONG,
    mut bPrependDot: BOOL,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut szTemp = [0u16; MAX_EXTENSION_LENGTH + 2];

    // do not want double dots
    if *szExtension == b'.' as WCHAR {
        bPrependDot = FALSE;
    }
    if bPrependDot != FALSE {
        szTemp[0] = b'.' as WCHAR;
    }
    let off = if bPrependDot != FALSE { 1 } else { 0 };
    lstrcpynW(szTemp.as_mut_ptr().add(off), szExtension, MAX_EXTENSION_LENGTH as INT);

    if RegOpenKeyExW(HKEY_CLASSES_ROOT, szTemp.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
        return FALSE;
    }
    if RegQueryValueW(hkey, core::ptr::null(), szFileType, &mut len) != 0 {
        RegCloseKey(hkey);
        return FALSE;
    }
    RegCloseKey(hkey);
    TRUE
}

/// HCR_MapTypeToValueA (classes.c:116)
#[no_mangle]
pub unsafe extern "C" fn HCR_MapTypeToValueA(
    szExtension: LPCSTR,
    szFileType: LPSTR,
    mut len: LONG,
    mut bPrependDot: BOOL,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut szTemp = [0i8; MAX_EXTENSION_LENGTH + 2];

    if *szExtension == b'.' as CHAR {
        bPrependDot = FALSE;
    }
    if bPrependDot != FALSE {
        szTemp[0] = b'.' as CHAR;
    }
    let off = if bPrependDot != FALSE { 1 } else { 0 };
    lstrcpynA(szTemp.as_mut_ptr().add(off), szExtension, MAX_EXTENSION_LENGTH as INT);

    if RegOpenKeyExA(HKEY_CLASSES_ROOT, szTemp.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
        return FALSE;
    }
    if RegQueryValueA(hkey, core::ptr::null(), szFileType, &mut len) != 0 {
        RegCloseKey(hkey);
        return FALSE;
    }
    RegCloseKey(hkey);
    TRUE
}

/// HCR_GetDefaultVerbW (classes.c:152)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetDefaultVerbW(
    hkeyClass: HKEY,
    szVerb: LPCWSTR,
    szDest: LPWSTR,
    len: DWORD,
) -> BOOL {
    let mut sTemp = [0u16; MAX_PATH];
    let mut verbs = [0u16; MAX_PATH];
    let mut hkey: HKEY = core::ptr::null_mut();

    if !szVerb.is_null() && *szVerb != 0 {
        lstrcpynW(szDest, szVerb, len as INT);
        return TRUE;
    }

    // MSDN says to first try the default verb
    let mut size: LONG = verbs.len() as LONG;
    if RegQueryValueW(hkeyClass, W_SHELL.as_ptr(), verbs.as_mut_ptr(), &mut size) == 0 && verbs[0] != 0 {
        let mut i: UINT = 0;
        loop {
            if failed(SHELL32_EnumDefaultVerbList(verbs.as_ptr(), i, szDest, len as usize)) {
                break;
            }
            // StringCchPrintfW(sTemp, _countof(sTemp), L"shell\\%s\\command", szDest)
            let mut b = WBuf::new(sTemp.as_mut_ptr(), sTemp.len());
            b.cat(&W_SHELL_BS);
            b.cat_ptr(szDest);
            b.cat(&W_BS_COMMAND);
            if failed(b.finish()) {
                break;
            }
            if RegOpenKeyExW(hkeyClass, sTemp.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
                RegCloseKey(hkey);
                return TRUE;
            }
            i = i.wrapping_add(1);
        }
    }
    wv(szDest, UNICODE_NULL);

    // then fallback to 'open'
    if RegOpenKeyExW(hkeyClass, W_SHELL_OPEN_COMMAND.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        RegCloseKey(hkey);
        lstrcpynW(szDest, W_OPEN.as_ptr(), len as INT);
        return TRUE;
    }

    // and then just use the first verb (__REACTOS__)
    if RegOpenKeyExW(hkeyClass, W_SHELL.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        if RegEnumKeyW(hkey, 0, szDest, len) == 0 && rv(szDest) != 0 {
            RegCloseKey(hkey);
            return TRUE;
        }
        RegCloseKey(hkey);
    }

    FALSE
}

/// HCR_GetExecuteCommandW (classes.c:220)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetExecuteCommandW(
    mut hkeyClass: HKEY,
    szClass: LPCWSTR,
    szVerb: LPCWSTR,
    szDest: LPWSTR,
    mut len: DWORD,
) -> BOOL {
    let mut sTempVerb = [0u16; MAX_PATH];
    let mut ret: BOOL = FALSE;

    if !szClass.is_null() {
        RegOpenKeyExW(HKEY_CLASSES_ROOT, szClass, 0, KEY_READ, &mut hkeyClass);
    }
    if hkeyClass.is_null() {
        return FALSE;
    }

    if HCR_GetDefaultVerbW(hkeyClass, szVerb, sTempVerb.as_mut_ptr(), sTempVerb.len() as DWORD) != FALSE {
        let mut sTemp = [0u16; MAX_PATH];
        lstrcpyW(sTemp.as_mut_ptr(), W_SHELL_BS.as_ptr());
        lstrcatW(sTemp.as_mut_ptr(), sTempVerb.as_ptr());
        lstrcatW(sTemp.as_mut_ptr(), W_BS_COMMAND.as_ptr());
        ret = (SHGetValueW(hkeyClass, sTemp.as_ptr(), core::ptr::null(), core::ptr::null_mut(), szDest as PVOID, &mut len)
            == ERROR_SUCCESS as DWORD) as BOOL;
    }
    if !szClass.is_null() {
        RegCloseKey(hkeyClass);
    }
    ret
}

/// HCR_RegOpenClassIDKey (classes.c:253)
#[no_mangle]
pub unsafe extern "C" fn HCR_RegOpenClassIDKey(riid: REFIID, hkey: *mut HKEY) -> BOOL {
    let mut xriid = [0i8; 50];
    // sprintf(xriid, "CLSID\\{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}", ...)
    let r = &*riid;
    let mut b = ABuf::new(xriid.as_mut_ptr(), xriid.len());
    b.cat_bytes(b"CLSID\\{");
    b.cat_hex(r.Data1, 8);
    b.put(b'-');
    b.cat_hex(r.Data2 as u32, 4);
    b.put(b'-');
    b.cat_hex(r.Data3 as u32, 4);
    b.put(b'-');
    b.cat_hex(r.Data4[0] as u32, 2);
    b.cat_hex(r.Data4[1] as u32, 2);
    b.put(b'-');
    b.cat_hex(r.Data4[2] as u32, 2);
    b.cat_hex(r.Data4[3] as u32, 2);
    b.cat_hex(r.Data4[4] as u32, 2);
    b.cat_hex(r.Data4[5] as u32, 2);
    b.cat_hex(r.Data4[6] as u32, 2);
    b.cat_hex(r.Data4[7] as u32, 2);
    b.put(b'}');
    b.finish();

    (RegOpenKeyExA(HKEY_CLASSES_ROOT, xriid.as_ptr(), 0, KEY_READ, hkey) == 0) as BOOL
}

/// HCR_RegGetIconW (classes.c:266) — internal static.
unsafe fn hcr_reg_get_icon_w(hkey: HKEY, szDest: LPWSTR, szName: LPCWSTR, len: DWORD, picon_idx: PINT) -> BOOL {
    let mut dw_type: DWORD = 0;
    let mut size: DWORD = len * core::mem::size_of::<WCHAR>() as DWORD;
    let mut sTemp = [0u16; MAX_PATH];
    let mut sNum = [0u16; 5];

    if RegQueryValueExW(hkey, szName, core::ptr::null_mut(), &mut dw_type, szDest as LPBYTE, &mut size) == 0 {
        if dw_type == REG_EXPAND_SZ {
            ExpandEnvironmentStringsW(szDest, sTemp.as_mut_ptr(), MAX_PATH as DWORD);
            lstrcpynW(szDest, sTemp.as_ptr(), len as INT);
        }
        if ParseFieldW(szDest, 2, sNum.as_mut_ptr(), sNum.len() as DWORD) != 0 {
            wv(picon_idx, atoi_w(sNum.as_ptr()));
        } else {
            wv(picon_idx, 0); // sometimes the icon number is missing
        }
        ParseFieldW(szDest, 1, szDest, len);
        PathUnquoteSpacesW(szDest);
        return TRUE;
    }
    FALSE
}

/// HCR_RegGetIconA (classes.c:290) — internal static.
unsafe fn hcr_reg_get_icon_a(hkey: HKEY, szDest: LPSTR, szName: LPCSTR, mut len: DWORD, picon_idx: PINT) -> BOOL {
    let mut dw_type: DWORD = 0;
    let mut sTemp = [0i8; MAX_PATH];
    let mut sNum = [0i8; 5];

    if RegQueryValueExA(hkey, szName, core::ptr::null_mut(), &mut dw_type, szDest as LPBYTE, &mut len) == 0 {
        if dw_type == REG_EXPAND_SZ {
            ExpandEnvironmentStringsA(szDest, sTemp.as_mut_ptr(), MAX_PATH as DWORD);
            lstrcpynA(szDest, sTemp.as_ptr(), len as INT);
        }
        if ParseFieldA(szDest, 2, sNum.as_mut_ptr(), 5) != 0 {
            wv(picon_idx, atoi_a(sNum.as_ptr()));
        } else {
            wv(picon_idx, 0);
        }
        ParseFieldA(szDest, 1, szDest, len);
        PathUnquoteSpacesA(szDest);
        return TRUE;
    }
    FALSE
}

/// HCR_GetIconW (classes.c:314)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetIconW(
    szClass: LPCWSTR,
    szDest: LPWSTR,
    szName: LPCWSTR,
    len: DWORD,
    picon_idx: PINT,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut sTemp = [0u16; MAX_PATH];
    let mut ret: BOOL = FALSE;

    lstrcpynW(sTemp.as_mut_ptr(), szClass, MAX_PATH as INT);
    lstrcatW(sTemp.as_mut_ptr(), W_DEFAULTICON.as_ptr());

    if RegOpenKeyExW(HKEY_CLASSES_ROOT, sTemp.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        ret = hcr_reg_get_icon_w(hkey, szDest, szName, len, picon_idx);
        RegCloseKey(hkey);
    }
    ret
}

/// HCR_GetIconA (classes.c:339)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetIconA(
    szClass: LPCSTR,
    szDest: LPSTR,
    szName: LPCSTR,
    len: DWORD,
    picon_idx: PINT,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut sTemp = [0i8; MAX_PATH];
    let mut ret: BOOL = FALSE;

    // sprintf(sTemp, "%s\\DefaultIcon", szClass)
    let mut b = ABuf::new(sTemp.as_mut_ptr(), sTemp.len());
    b.cat_ptr(szClass);
    b.cat_bytes(b"\\DefaultIcon");
    b.finish();

    if RegOpenKeyExA(HKEY_CLASSES_ROOT, sTemp.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        ret = hcr_reg_get_icon_a(hkey, szDest, szName, len, picon_idx);
        RegCloseKey(hkey);
    }
    ret
}

/// HCU_GetIconW (classes.c:364, __REACTOS__)
#[no_mangle]
pub unsafe extern "C" fn HCU_GetIconW(
    szClass: LPCWSTR,
    szDest: LPWSTR,
    szName: LPCWSTR,
    len: DWORD,
    picon_idx: PINT,
) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut sTemp = [0u16; MAX_PATH];
    let mut ret: BOOL = FALSE;

    // StringCchPrintfW(sTemp, _countof(sTemp), L"%s\\DefaultIcon", szClass)
    let mut b = WBuf::new(sTemp.as_mut_ptr(), sTemp.len());
    b.cat_ptr(szClass);
    b.cat(&W_DEFAULTICON);
    b.finish();

    if RegOpenKeyExW(HKEY_CURRENT_USER, sTemp.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        ret = hcr_reg_get_icon_w(hkey, szDest, szName, len, picon_idx);
        RegCloseKey(hkey);
    }
    ret
}

/// HLM_GetIconW (classes.c:388, __REACTOS__)
#[no_mangle]
pub unsafe extern "C" fn HLM_GetIconW(reg_idx: INT, szDest: LPWSTR, len: DWORD, picon_idx: PINT) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut sTemp = [0u16; 5];
    let mut ret: BOOL = FALSE;

    // StringCchPrintfW(sTemp, _countof(sTemp), L"%d", reg_idx)
    let mut b = WBuf::new(sTemp.as_mut_ptr(), sTemp.len());
    b.cat_int(reg_idx);
    b.finish();

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE, W_SHELL_ICONS.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
        ret = hcr_reg_get_icon_w(hkey, szDest, sTemp.as_ptr(), len, picon_idx);
        RegCloseKey(hkey);
    }
    ret
}

/// HCR_GetClassNameW (classes.c:422)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetClassNameW(riid: REFIID, szDest: LPWSTR, mut len: DWORD) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut ret: BOOL = FALSE;
    let buflen: DWORD = len;
    let mut szName = [0u16; 100];
    let mut pStr: LPOLESTR = core::ptr::null_mut();

    wv(szDest, 0);

    if StringFromCLSID(riid, &mut pStr) == S_OK {
        let mut dw_len: DWORD = buflen * core::mem::size_of::<WCHAR>() as DWORD;
        // swprintf(szName, L"Software\\...\\CLSID\\%s", pStr)
        let mut b = WBuf::new(szName.as_mut_ptr(), szName.len());
        b.cat(&W_CLSID_PREFIX);
        b.cat_ptr(pStr);
        b.finish();
        if RegGetValueW(
            HKEY_CURRENT_USER,
            szName.as_ptr(),
            core::ptr::null(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            szDest as PVOID,
            &mut dw_len,
        ) == 0
        {
            ret = TRUE;
        }
        CoTaskMemFree(pStr as LPVOID);
    }
    if ret == FALSE && HCR_RegOpenClassIDKey(riid, &mut hkey) != FALSE {
        if RegLoadMUIStringW(hkey, W_LOCALIZEDSTRING.as_ptr(), szDest, len, core::ptr::null_mut(), 0, core::ptr::null())
            == 0
            || RegQueryValueExW(hkey, W_EMPTY.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), szDest as LPBYTE, &mut len)
                == 0
        {
            ret = TRUE;
        }
        RegCloseKey(hkey);
    }

    if ret == FALSE || rv(szDest) == 0 {
        if is_equal_iid(riid, &CLSID_ShellDesktop) {
            if LoadStringW(shell32_hInstance, IDS_DESKTOP, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_MyComputer) {
            if LoadStringW(shell32_hInstance, IDS_MYCOMPUTER, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_MyDocuments) {
            if LoadStringW(shell32_hInstance, IDS_PERSONAL, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_RecycleBin) {
            if LoadStringW(shell32_hInstance, IDS_RECYCLEBIN_FOLDER_NAME, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_ControlPanel) {
            if LoadStringW(shell32_hInstance, IDS_CONTROLPANEL, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_AdminFolderShortcut)
            && LoadStringW(shell32_hInstance, IDS_ADMINISTRATIVETOOLS, szDest, buflen as INT) != 0
        {
            ret = TRUE;
        }
    }
    ret
}

/// HCR_GetClassNameA (classes.c:497)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetClassNameA(riid: REFIID, szDest: LPSTR, mut len: DWORD) -> BOOL {
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut ret: BOOL = FALSE;
    let buflen: DWORD = len;
    let mut szName = [0i8; 100];
    let mut pStr: LPOLESTR = core::ptr::null_mut();

    wv(szDest, 0);

    if StringFromCLSID(riid, &mut pStr) == S_OK {
        let mut dw_len: DWORD = buflen * core::mem::size_of::<CHAR>() as DWORD;
        // sprintf(szName, "Software\\...\\CLSID\\%S", pStr)
        let mut b = ABuf::new(szName.as_mut_ptr(), szName.len());
        b.cat_bytes(b"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\CLSID\\");
        b.cat_wide_as_narrow(pStr);
        b.finish();
        if RegGetValueA(
            HKEY_CURRENT_USER,
            szName.as_ptr(),
            core::ptr::null(),
            RRF_RT_REG_SZ,
            core::ptr::null_mut(),
            szDest as PVOID,
            &mut dw_len,
        ) == 0
        {
            ret = TRUE;
        }
        CoTaskMemFree(pStr as LPVOID);
    }
    if ret == FALSE && HCR_RegOpenClassIDKey(riid, &mut hkey) != FALSE {
        if RegLoadMUIStringA(hkey, b"LocalizedString\0".as_ptr() as LPCSTR, szDest, len, core::ptr::null_mut(), 0, core::ptr::null())
            == 0
            || RegQueryValueExA(hkey, b"\0".as_ptr() as LPCSTR, core::ptr::null_mut(), core::ptr::null_mut(), szDest as LPBYTE, &mut len)
                == 0
        {
            ret = TRUE;
        }
        RegCloseKey(hkey);
    }

    if ret == FALSE || rv(szDest) == 0 {
        if is_equal_iid(riid, &CLSID_ShellDesktop) {
            if LoadStringA(shell32_hInstance, IDS_DESKTOP, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_MyComputer) {
            if LoadStringA(shell32_hInstance, IDS_MYCOMPUTER, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_MyDocuments) {
            if LoadStringA(shell32_hInstance, IDS_PERSONAL, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_RecycleBin) {
            if LoadStringA(shell32_hInstance, IDS_RECYCLEBIN_FOLDER_NAME, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_ControlPanel) {
            if LoadStringA(shell32_hInstance, IDS_CONTROLPANEL, szDest, buflen as INT) != 0 {
                ret = TRUE;
            }
        } else if is_equal_iid(riid, &CLSID_AdminFolderShortcut)
            && LoadStringA(shell32_hInstance, IDS_ADMINISTRATIVETOOLS, szDest, buflen as INT) != 0
        {
            ret = TRUE;
        }
    }
    ret
}

/// HCR_GetFolderAttributes (classes.c:590)
#[no_mangle]
pub unsafe extern "C" fn HCR_GetFolderAttributes(pidlFolder: LPCITEMIDLIST, pdwAttributes: LPDWORD) -> BOOL {
    let mut hSFKey: HKEY = core::ptr::null_mut();
    let mut pwszCLSID: LPOLESTR = core::ptr::null_mut();
    let mut lResult: LONG;
    let mut dwTemp: DWORD = 0;
    let mut dwLen: DWORD;
    // WCHAR wszShellFolderKey[] = L"CLSID\\{00021400-...046}\\ShellFolder"
    let mut wszShellFolderKey: [WCHAR; SHELLFOLDER_KEY_LEN] = W_SHELLFOLDER_KEY;

    if _ILIsPidlSimple(pidlFolder) == FALSE {
        // static firstHit ERR-once — observable behavior is just "return FALSE".
        return FALSE;
    }

    if _ILIsDesktop(pidlFolder) == FALSE {
        if failed(StringFromCLSID(_ILGetGUIDPointer(pidlFolder), &mut pwszCLSID)) {
            return FALSE;
        }
        // memcpy(&wszShellFolderKey[6], pwszCLSID, 38 * sizeof(WCHAR))
        core::ptr::copy_nonoverlapping(pwszCLSID, wszShellFolderKey.as_mut_ptr().add(6), 38);
        CoTaskMemFree(pwszCLSID as LPVOID);
    }

    lResult = RegOpenKeyExW(HKEY_CLASSES_ROOT, wszShellFolderKey.as_ptr(), 0, KEY_READ, &mut hSFKey);
    if lResult != ERROR_SUCCESS {
        return FALSE; // __REACTOS__: ERR then return FALSE
    }

    dwLen = core::mem::size_of::<DWORD>() as DWORD;
    lResult = RegQueryValueExW(hSFKey, W_CALLFORATTRIBUTES.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), &mut dwTemp as *mut DWORD as LPBYTE, &mut dwLen);
    if lResult == ERROR_SUCCESS && (dwTemp & rv(pdwAttributes)) != 0 {
        let mut psfDesktop: LPSHELLFOLDER = core::ptr::null_mut();
        let mut psfFolder: LPSHELLFOLDER = core::ptr::null_mut();
        let mut hr: HRESULT;

        RegCloseKey(hSFKey);
        hr = SHGetDesktopFolder(&mut psfDesktop);
        if !failed(hr) {
            hr = ((*(*psfDesktop).lpVtbl).BindToObject)(
                psfDesktop,
                pidlFolder,
                core::ptr::null_mut(),
                &IID_IShellFolder,
                &mut psfFolder as *mut LPSHELLFOLDER as *mut LPVOID,
            );
            if !failed(hr) {
                hr = ((*(*psfFolder).lpVtbl).GetAttributesOf)(psfFolder, 0, core::ptr::null_mut(), pdwAttributes);
                ((*(*psfFolder).lpVtbl).Release)(psfFolder);
            }
            ((*(*psfDesktop).lpVtbl).Release)(psfDesktop);
        }
        if failed(hr) {
            return FALSE;
        }
    } else {
        lResult = RegQueryValueExW(hSFKey, W_ATTRIBUTES.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), &mut dwTemp as *mut DWORD as LPBYTE, &mut dwLen);
        RegCloseKey(hSFKey);
        if lResult == ERROR_SUCCESS {
            wv(pdwAttributes, rv(pdwAttributes) & dwTemp);
        } else {
            return FALSE;
        }
    }

    TRUE
}

// ADDED-TO-TYPES?: GUID/REFIID (guiddef.h), ULONG, PCWSTR/PWSTR/LPOLESTR/PINT, LPCITEMIDLIST,
//   IShellFolder + IShellFolderVtbl (shobjidl.h) — all are common shell32 types that future
//   PIDL/COM waves will reuse; recommend hoisting GUID/REFIID/ULONG/LPCITEMIDLIST and the
//   IShellFolder vtable into crate::types when the COM surface is shared.
