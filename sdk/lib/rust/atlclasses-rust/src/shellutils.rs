//! shellutils.h's lazily registered clipboard formats, shared by every C++ shell module.
//!
//! `g_cfHIDA` and `g_cfShellIdListOffsets` are DECLSPEC_SELECTANY: one per image. This is an
//! rlib linked into each DLL, so its statics are per image too. A clipboard format is a
//! `WORD` on every target, which is what lets it live here rather than in each island; the
//! `CDataObjectHIDA` lowering (tools/rustconv/data/local_raii) calls [`hida_clipformat`].

extern "system" {
    fn RegisterClipboardFormatW(lpszFormat: *const u16) -> u32;
}

/// Encode an ASCII literal as a NUL-terminated UTF-16 array at compile time.
const fn wide<const N: usize>(s: &str) -> [u16; N] {
    let b = s.as_bytes();
    let mut out = [0u16; N];
    let mut i = 0;
    while i < b.len() {
        out[i] = b[i] as u16;
        i += 1;
    }
    out
}

/// shlobj.h `CFSTR_SHELLIDLISTW`.
pub const CFSTR_SHELLIDLISTW: [u16; 19] = wide("Shell IDList Array");
/// shlobj.h `CFSTR_SHELLIDLISTOFFSETW`.
pub const CFSTR_SHELLIDLISTOFFSETW: [u16; 21] = wide("Shell Object Offsets");

static mut G_CF_HIDA: u16 = 0;
static mut G_CF_SHELLIDLIST_OFFSETS: u16 = 0;

/// `if (g_cfHIDA == NULL) g_cfHIDA = (CLIPFORMAT)RegisterClipboardFormatW(CFSTR_SHELLIDLISTW)`.
pub unsafe fn hida_clipformat() -> u16 {
    if G_CF_HIDA == 0 {
        G_CF_HIDA = RegisterClipboardFormatW(CFSTR_SHELLIDLISTW.as_ptr()) as u16;
    }
    G_CF_HIDA
}

/// The same for `g_cfShellIdListOffsets`.
pub unsafe fn shellidlist_offsets_clipformat() -> u16 {
    if G_CF_SHELLIDLIST_OFFSETS == 0 {
        G_CF_SHELLIDLIST_OFFSETS = RegisterClipboardFormatW(CFSTR_SHELLIDLISTOFFSETW.as_ptr()) as u16;
    }
    G_CF_SHELLIDLIST_OFFSETS
}

/// `((LPBYTE)pida) + pida->aoffset[n]`. A CIDA is `{ UINT cidl; UINT aoffset[]; }` on every
/// target, so `aoffset[n]` is the `u32` at byte `4 * (n + 1)`, read unaligned: the island
/// lays CIDA out packed. Generic over the caller's own island types (its `CIDA` and its
/// ITEMIDLIST), which are nominal per crate and cannot be named here.
unsafe fn cida_offset<C, R>(pida: *const C, n: usize) -> *const R {
    let base = pida as *const u8;
    let off = core::ptr::read_unaligned(base.add(4 * (n + 1)) as *const u32);
    base.add(off as usize) as *const R
}

/// shellutils.h `HIDA_GetPIDLFolder`: the parent folder's absolute PIDL.
pub unsafe fn HIDA_GetPIDLFolder<C, R>(pida: *const C) -> *const R {
    cida_offset(pida, 0)
}

/// shellutils.h `HIDA_GetPIDLItem`: child `i`, relative to the folder.
pub unsafe fn HIDA_GetPIDLItem<C, R, I: TryInto<usize>>(pida: *const C, i: I) -> *const R {
    // SIZE_T in the C; callers hand a UINT loop index. Every value a CIDA can index fits.
    cida_offset(pida, i.try_into().unwrap_or(usize::MAX) + 1)
}

// The header inlines below are generic or `#[inline]`: each is instantiated only in a module
// that calls it, so its shell32/user32 imports never reach one that links neither.
extern "system" {
    fn ILClone(pidl: *const core::ffi::c_void) -> *mut core::ffi::c_void;
    fn ILFindLastID(pidl: *const core::ffi::c_void) -> *mut core::ffi::c_void;
    fn IsWindowVisible(hwnd: *mut core::ffi::c_void) -> i32;
    fn FormatMessageW(
        flags: u32,
        source: *const core::ffi::c_void,
        id: u32,
        lang: u32,
        buf: *mut u16,
        size: u32,
        args: *mut core::ffi::c_void,
    ) -> u32;
    fn LoadLibraryW(name: *const u16) -> *mut core::ffi::c_void;
    fn LoadStringW(module: *mut core::ffi::c_void, id: u32, buf: *mut u16, max: i32) -> i32;
    fn MessageBoxW(hwnd: *mut core::ffi::c_void, text: *const u16, caption: *const u16, ty: u32)
        -> i32;
}
extern "C" {
    fn wsprintfW(buf: *mut u16, fmt: *const u16, ...) -> i32;
}

/// shellutils.h `HResultFromWin32`: `HRESULT_FROM_WIN32` with its argument evaluated once.
#[inline]
pub fn HResultFromWin32(x: u32) -> i32 {
    // winerror.h: `(HRESULT)(x) <= 0 ? (HRESULT)(x) : (x & 0xFFFF) | (FACILITY_WIN32 << 16) |
    // 0x80000000`.
    if x as i32 <= 0 { x as i32 } else { ((x & 0xFFFF) | (7 << 16) | 0x8000_0000) as i32 }
}

/// shlobj.h `ILIsEmpty`: NULL, or a list whose first item is the 0-length terminator
/// (`!pidl->mkid.cb`; `cb` is the list's first `USHORT`).
#[inline]
pub unsafe fn ILIsEmpty<P>(pidl: *const P) -> i32 {
    (pidl.is_null() || core::ptr::read_unaligned(pidl as *const u16) == 0) as i32
}

/// shellutils.h `ILIsSingle`: `pidl == ILFindLastID(pidl)`.
#[inline]
pub unsafe fn ILIsSingle<P>(pidl: *const P) -> i32 {
    (pidl as *const core::ffi::c_void == ILFindLastID(pidl as *const _) as *const _) as i32
}

/// shellutils.h `SHILClone`: `*ppOut = ILClone(pidl)`, S_OK or E_OUTOFMEMORY.
#[inline]
pub unsafe fn SHILClone<P, Q>(pidl: *const P, out: *mut *mut Q) -> i32 {
    let r = ILClone(pidl as *const _) as *mut Q;
    *out = r;
    if r.is_null() { 0x8007_000Eu32 as i32 } else { 0 }
}

/// shellutils.h `SHELL_ErrorBox(hwndOwner, Error)` (`SHELL_ErrorBoxHelper`): the system's
/// message for `Error` in a stop box, owned by `hwnd` only while that is visible. C++ defaults
/// `Error` to `GetLastError()`; a Rust caller passes it.
#[inline]
pub unsafe fn SHELL_ErrorBox<H>(hwnd: *mut H, error: u32) -> u32 {
    const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x1000;
    const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x200;
    const ERROR_INTERNAL_ERROR: u32 = 1359;
    const MB_ICONSTOP: u32 = 0x10;
    const USER32_IDS_ERROR: u32 = 2; // IDS_ERROR from user32 resource.h ("Error" string)
    const USER32: [u16; 7] = wide("USER32");
    const FMT: [u16; 13] = wide("\n\n%#x (%d)");
    let mut buf = [0u16; 400];
    let mut owner = hwnd as *mut core::ffi::c_void;
    if IsWindowVisible(owner) == 0 {
        owner = core::ptr::null_mut();
    }
    let error = if error == 0 { ERROR_INTERNAL_ERROR } else { error };
    let mut cch = FormatMessageW(
        FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        core::ptr::null(),
        error,
        0,
        buf.as_mut_ptr(),
        buf.len() as u32,
        core::ptr::null_mut(),
    );
    if cch == 0 {
        cch = LoadStringW(
            LoadLibraryW(USER32.as_ptr()),
            USER32_IDS_ERROR,
            buf.as_mut_ptr(),
            buf.len() as i32,
        ) as u32;
        wsprintfW(buf.as_mut_ptr().add(cch as usize), FMT.as_ptr(), error, error);
    }
    MessageBoxW(owner, buf.as_ptr(), core::ptr::null(), MB_ICONSTOP); // MB_OK is 0
    error
}

/// shellutils.h `CCoInitBase<InitFunc, UninitFunc>`: initializes on construction, and
/// uninitializes on destruction only when that succeeded. `CCoInit` is the CoInitialize
/// instance, `COleInit` the OleInitialize one. ole32: feature `com`.
#[cfg(feature = "com")]
pub struct CCoInitBase<const OLE: bool> {
    pub hr: i32,
}

#[cfg(feature = "com")]
extern "system" {
    fn CoInitialize(reserved: *mut core::ffi::c_void) -> i32;
    fn CoUninitialize();
    fn OleInitialize(reserved: *mut core::ffi::c_void) -> i32;
    fn OleUninitialize();
}

#[cfg(feature = "com")]
impl<const OLE: bool> CCoInitBase<OLE> {
    /// `CCoInitBase() : hr(InitFunc(NULL))`.
    pub unsafe fn new() -> Self {
        let p = core::ptr::null_mut();
        CCoInitBase { hr: if OLE { OleInitialize(p) } else { CoInitialize(p) } }
    }
}

#[cfg(feature = "com")]
impl<const OLE: bool> Drop for CCoInitBase<OLE> {
    /// `~CCoInitBase() { if (SUCCEEDED(hr)) UninitFunc(); }`.
    fn drop(&mut self) {
        if self.hr >= 0 {
            unsafe {
                if OLE {
                    OleUninitialize()
                } else {
                    CoUninitialize()
                }
            }
        }
    }
}

/// `typedef CCoInitBase<CoInitialize, CoUninitialize> CCoInit;`
#[cfg(feature = "com")]
pub type CCoInit = CCoInitBase<false>;
/// `typedef CCoInitBase<OleInitialize, OleUninitialize> COleInit;`
#[cfg(feature = "com")]
pub type COleInit = CCoInitBase<true>;

// ---- STRRET (shtypes.h) and IDataObject::SetData, layout-generic ------------------------
// STRRET is `{ UINT uType; union { LPWSTR pOleStr; UINT uOffset; char cStr[MAX_PATH]; } }` on
// every target: the union is pointer-aligned, so it starts at `size_of::<usize>()`. Each
// island's STRRET is its own nominal type, so these take a pointer to any of them.
extern "system" {
    fn CoTaskMemAlloc(cb: usize) -> *mut core::ffi::c_void;
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut core::ffi::c_void;
    fn GlobalLock(h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn GlobalUnlock(h: *mut core::ffi::c_void) -> i32;
    fn GlobalFree(h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
}

const STRRET_WSTR: u32 = 0;
const STRRET_CSTR: u32 = 2;
const E_OUTOFMEMORY: i32 = 0x8007000Eu32 as i32;
const E_FAIL: i32 = 0x80004005u32 as i32;
const E_UNEXPECTED: i32 = 0x8000FFFFu32 as i32;

#[doc(hidden)]
#[inline]
pub unsafe fn strret_set_wide<P>(p: *mut P, value: *const u16) -> i32 {
    let mut n = 0usize;
    while *value.add(n) != 0 {
        n += 1;
    }
    let buf = CoTaskMemAlloc((n + 1) * 2) as *mut u16;
    if buf.is_null() {
        return E_OUTOFMEMORY;
    }
    core::ptr::copy_nonoverlapping(value, buf, n + 1);
    *(p as *mut u32) = STRRET_WSTR;
    *((p as *mut u8).add(core::mem::size_of::<usize>()) as *mut *mut u16) = buf;
    0
}

#[doc(hidden)]
#[inline]
pub unsafe fn strret_set_ansi<P>(p: *mut P, value: *const u8) -> i32 {
    *(p as *mut u32) = STRRET_CSTR;
    let dst = (p as *mut u8).add(core::mem::size_of::<usize>());
    // strcpy, as the C: the source is a C string the caller vouches fits cStr[MAX_PATH].
    let mut i = 0usize;
    loop {
        let c = *value.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    0
}

/// The value half of `SHSetStrRet(pStrRet, value)`: a wide string is STRRET_WSTR in a
/// CoTaskMemAlloc copy, a narrow one is STRRET_CSTR copied into `cStr`.
pub trait StrRetValue {
    unsafe fn set_strret<P>(self, p: *mut P) -> i32;
}
impl StrRetValue for *const u16 {
    unsafe fn set_strret<P>(self, p: *mut P) -> i32 { strret_set_wide(p, self) }
}
impl StrRetValue for *mut u16 {
    unsafe fn set_strret<P>(self, p: *mut P) -> i32 { strret_set_wide(p, self) }
}
impl StrRetValue for *const i8 {
    unsafe fn set_strret<P>(self, p: *mut P) -> i32 { strret_set_ansi(p, self as *const u8) }
}
impl StrRetValue for *const u8 {
    unsafe fn set_strret<P>(self, p: *mut P) -> i32 { strret_set_ansi(p, self) }
}

/// shellutils.h `SHSetStrRet(pStrRet, hInstance, resId)`: the string resource, as a wide
/// STRRET; E_FAIL when it does not load.
#[doc(hidden)]
#[inline]
pub unsafe fn strret_set_resource<P, H>(p: *mut P, hinstance: *mut H, res_id: u32) -> i32 {
    let mut buf = [0u16; 260];
    if LoadStringW(hinstance as *mut core::ffi::c_void, res_id, buf.as_mut_ptr(), 260) == 0 {
        return E_FAIL;
    }
    strret_set_wide(p, buf.as_ptr())
}

/// shellutils.h's three `SHSetStrRet` overloads, told apart by arity as C++ tells them apart
/// by type: `SHSetStrRet!(p, wide_or_narrow)` and `SHSetStrRet!(p, hInstance, resId)`.
#[macro_export]
macro_rules! SHSetStrRet {
    ($p:expr, $v:expr $(,)?) => {
        $crate::shellutils::StrRetValue::set_strret($v, $p)
    };
    ($p:expr, $h:expr, $id:expr $(,)?) => {
        // `as *mut u8` pins a generic `GetResourceInstance::<T>()` (E0282 otherwise).
        $crate::shellutils::strret_set_resource($p, ($h) as *mut u8, ($id) as u32)
    };
}

/// shellutils.h `SHSetStrRetEmpty`: an empty STRRET_CSTR.
#[inline]
pub unsafe fn SHSetStrRetEmpty<P>(p: *mut P) -> i32 {
    *(p as *mut u32) = STRRET_CSTR;
    *(p as *mut u8).add(core::mem::size_of::<usize>()) = 0;
    0
}

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> u32;
}
extern "system" {
    fn ILGetNext(pidl: *const core::ffi::c_void) -> *mut core::ffi::c_void;
}

/// shellutils.h `DumpIdList`: each SHITEMID's bytes to the debugger, four per line, the
/// tail of a `cb` that is not a multiple of 4 padded with `--`. Debug output only.
pub unsafe fn DumpIdList<P>(pcidl: *const P) {
    DbgPrint(b"Begin IDList Dump\n\0".as_ptr());
    let mut p = pcidl as *const u8;
    while !p.is_null() {
        let mut cb = u16::from_le_bytes([*p, *p.add(1)]) as i32;
        if cb == 0 {
            break; // ITEMIDLISTs are terminated with a null SHITEMID.
        }
        DbgPrint(b"Begin SHITEMID (cb=%d)\n\0".as_ptr(), cb);
        if cb & 3 != 0 {
            DbgPrint(b" - WARNING: cb is not a multiple of 4\n\0".as_ptr());
        }
        let sh = |k: i32| *p.add(k as usize) as u32;
        let mut i = 0;
        while i + 4 <= cb {
            DbgPrint(
                b" - abID[%08x]: %02x %02x %02x %02x\n\0".as_ptr(),
                i, sh(i), sh(i + 1), sh(i + 2), sh(i + 3),
            );
            i += 4;
        }
        if i < cb {
            cb -= i;
            match cb {
                3 => DbgPrint(b" - abID[%08x]: %02x %02x %02x --\n\0".as_ptr(), i, sh(i), sh(i + 1), sh(i + 2)),
                2 => DbgPrint(b" - abID[%08x]: %02x %02x -- --\n\0".as_ptr(), i, sh(i), sh(i + 1)),
                _ => DbgPrint(b" - abID[%08x]: %02x -- -- --\n\0".as_ptr(), i, sh(i)),
            };
        }
        DbgPrint(b"End SHITEMID\n\0".as_ptr());
        p = ILGetNext(p as *const core::ffi::c_void) as *const u8;
    }
    DbgPrint(b"End IDList Dump.\n\0".as_ptr());
}

// ---- shellfolderutils.h -------------------------------------------------------------------
// C++ templates over compile-time column count / canonical column / compare kind; here those
// are arguments. IShellFolder2's GetDetailsOf is vtable slot 18 (IUnknown 3, IShellFolder 10,
// then GetDefaultSearchGUID, EnumSearches, GetDefaultColumn, GetDefaultColumnState,
// GetDetailsEx), and SHELLDETAILS is `{ int fmt; int cxChar; STRRET str; }` with STRRET at 8.
extern "system" {
    fn StrRetToStrW(psr: *mut core::ffi::c_void, pidl: *const core::ffi::c_void, ppsz: *mut *mut u16)
        -> i32;
    fn StrCmpLogicalW(a: *const u16, b: *const u16) -> i32;
    fn lstrcmpiW(a: *const u16, b: *const u16) -> i32;
    fn SHFree(pv: *mut core::ffi::c_void);
}

const S_EQUAL: i32 = 0;
const S_GREATERTHAN: i32 = 1;
const S_LESSTHAN: i32 = 0xFFFF;
const SHCIDS_COLUMNMASK: i64 = 0x0000_FFFF;
const SHCIDS_CANONICALONLY: i64 = 0x1000_0000;
const SHCIDS_ALLFIELDS: i64 = 0x8000_0000;

/// shellutils.h `MAKE_COMPARE_HRESULT`.
fn make_compare_hresult(x: i32) -> i32 {
    if x > 0 {
        S_GREATERTHAN
    } else if x < 0 {
        S_LESSTHAN
    } else {
        S_EQUAL
    }
}

#[repr(C, align(8))]
struct ShellDetails([u8; 8 + 272]);

unsafe fn details_of<F, P>(psf: *mut F, pidl: *const P, column: u32, sd: *mut ShellDetails) -> i32 {
    type GetDetailsOf =
        unsafe extern "system" fn(*mut core::ffi::c_void, *const core::ffi::c_void, u32, *mut u8) -> i32;
    let vtbl = *(psf as *const *const Option<GetDetailsOf>);
    match *vtbl.add(18) {
        Some(f) => f(psf as *mut _, pidl as *const _, column, sd as *mut u8),
        None => E_UNEXPECTED,
    }
}

/// shellfolderutils.h `ShellFolderImpl_CompareItemColumn<LOGICALCMP>`: the column's text for
/// both items, compared logically (StrCmpLogicalW) or case-insensitively (lstrcmpiW).
pub unsafe fn ShellFolderImpl_CompareItemColumn<F, P>(
    psf: *mut F,
    column: u32,
    pidl1: *const P,
    pidl2: *const P,
    logical: bool,
) -> i32 {
    let mut d1 = ShellDetails([0; 280]);
    let mut d2 = ShellDetails([0; 280]);
    let (mut s1, mut s2): (*mut u16, *mut u16) = (core::ptr::null_mut(), core::ptr::null_mut());
    let mut hr = details_of(psf, pidl1, column, &mut d1);
    if hr >= 0 {
        hr = StrRetToStrW(d1.0.as_mut_ptr().add(8).cast(), pidl1.cast(), &mut s1);
        if hr >= 0 {
            hr = details_of(psf, pidl2, column, &mut d2);
            if hr >= 0 {
                hr = StrRetToStrW(d2.0.as_mut_ptr().add(8).cast(), pidl2.cast(), &mut s2);
                if hr >= 0 {
                    let res = if logical { StrCmpLogicalW(s1, s2) } else { lstrcmpiW(s1, s2) };
                    hr = make_compare_hresult(res);
                    SHFree(s2.cast());
                }
            }
            SHFree(s1.cast());
        }
    }
    hr
}

/// shellfolderutils.h `ShellFolderImpl_CompareItemIDs<COLCOUNT, CANONICAL, LOGICALCMP>`.
/// **Exceeds the C:** with `colcount` 0 and SHCIDS_ALLFIELDS the C++ returns an
/// uninitialized HRESULT; this returns S_EQUAL (no column differs).
pub unsafe fn ShellFolderImpl_CompareItemIDs<F, P, L: TryInto<i64>>(
    psf: *mut F,
    lParam: L,
    pidl1: *const P,
    pidl2: *const P,
    colcount: u32,
    canonical: i32,
    logical: bool,
) -> i32 {
    let lparam: i64 = lParam.try_into().unwrap_or(0);
    let mut hr;
    if canonical >= 0 && lparam & SHCIDS_CANONICALONLY != 0 {
        hr = ShellFolderImpl_CompareItemColumn(psf, canonical as u32, pidl1, pidl2, logical);
        if hr == S_EQUAL || lparam & SHCIDS_ALLFIELDS == 0 || hr < 0 {
            return hr;
        }
    }
    if lparam & SHCIDS_ALLFIELDS != 0 {
        hr = S_EQUAL;
        for i in 0..colcount {
            hr = ShellFolderImpl_CompareItemColumn(psf, i, pidl1, pidl2, logical);
            if hr != 0 && hr >= 0 {
                break; // only stop on a successfully found difference
            }
        }
        return hr;
    }
    let column = (lparam & SHCIDS_COLUMNMASK) as u32;
    ShellFolderImpl_CompareItemColumn(psf, column, pidl1, pidl2, logical)
}

#[repr(C)]
struct FormatEtc {
    cf_format: u16,
    ptd: *mut core::ffi::c_void,
    dw_aspect: u32,
    lindex: i32,
    tymed: u32,
}

#[repr(C)]
struct StgMedium {
    tymed: u32,
    h_global: *mut core::ffi::c_void,
    p_unk_for_release: *mut core::ffi::c_void,
}

/// shellutils.h `DataObject_SetData`: hand `pBuffer` to the data object as an HGLOBAL
/// medium it takes ownership of (fRelease TRUE); the HGLOBAL is freed if SetData fails.
#[inline]
pub unsafe fn DataObject_SetData<D, B>(
    pDataObject: *mut D,
    clipformat: u16,
    pBuffer: *const B,
    dwBufferSize: usize,
) -> i32 {
    const GHND: u32 = 0x42;
    const TYMED_HGLOBAL: u32 = 1;
    const DVASPECT_CONTENT: u32 = 1;
    let mut medium = StgMedium {
        tymed: TYMED_HGLOBAL,
        h_global: GlobalAlloc(GHND, dwBufferSize),
        p_unk_for_release: core::ptr::null_mut(),
    };
    if medium.h_global.is_null() {
        return E_OUTOFMEMORY;
    }
    let mut hr = E_UNEXPECTED;
    let blob = GlobalLock(medium.h_global);
    if !blob.is_null() {
        core::ptr::copy_nonoverlapping(pBuffer as *const u8, blob as *mut u8, dwBufferSize);
        GlobalUnlock(medium.h_global);
        let mut etc = FormatEtc {
            cf_format: clipformat,
            ptd: core::ptr::null_mut(),
            dw_aspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        // IDataObject's vtable: IUnknown's three, GetData, GetDataHere, QueryGetData,
        // GetCanonicalFormatEtc, then SetData (slot 7).
        type SetData = unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *mut FormatEtc,
            *mut StgMedium,
            i32,
        ) -> i32;
        let vtbl = *(pDataObject as *const *const Option<SetData>);
        if let Some(set) = *vtbl.add(7) {
            hr = set(pDataObject as *mut core::ffi::c_void, &mut etc, &mut medium, 1);
        }
    }
    if hr < 0 {
        GlobalFree(medium.h_global);
    }
    hr
}

#[cfg(test)]
mod tests {
    #[no_mangle]
    extern "system" fn RegisterClipboardFormatW(_f: *const u16) -> u32 {
        0xC0DE
    }

    #[test]
    fn hresult_from_win32_matches_winerror_h() {
        assert_eq!(super::HResultFromWin32(0), 0);
        assert_eq!(super::HResultFromWin32(2) as u32, 0x8007_0002); // ERROR_FILE_NOT_FOUND
        assert_eq!(super::HResultFromWin32(0x8000_4005) as u32, 0x8000_4005); // already one
    }

    #[test]
    fn an_empty_pidl_is_null_or_a_bare_terminator() {
        let term = [0u8; 2];
        let item = [4u8, 0, 0xAB, 0xCD, 0, 0];
        unsafe {
            assert_eq!(super::ILIsEmpty(core::ptr::null::<u8>()), 1);
            assert_eq!(super::ILIsEmpty(term.as_ptr()), 1);
            assert_eq!(super::ILIsEmpty(item.as_ptr()), 0);
        }
    }

    #[test]
    fn the_format_is_registered_once_and_cached() {
        unsafe {
            assert_eq!(super::hida_clipformat(), 0xC0DE);
            assert_eq!(super::hida_clipformat(), 0xC0DE);
        }
    }

    #[test]
    fn hida_offsets_index_aoffset_from_the_cida_start() {
        // cidl=2, aoffset=[12, 14, 16], then three 2-byte "pidls".
        let mut blob = [0u8; 18];
        for (i, v) in [2u32, 12, 14, 16].iter().enumerate() {
            blob[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
        }
        let p = blob.as_ptr();
        unsafe {
            assert_eq!(super::HIDA_GetPIDLFolder::<u8, u8>(p), p.add(12));
            assert_eq!(super::HIDA_GetPIDLItem::<u8, u8, u32>(p, 0u32), p.add(14));
            assert_eq!(super::HIDA_GetPIDLItem::<u8, u8, u32>(p, 1u32), p.add(16));
        }
    }

    #[test]
    fn the_formats_are_nul_terminated_utf16() {
        assert_eq!(String::from_utf16(&super::CFSTR_SHELLIDLISTW[..18]).unwrap(), "Shell IDList Array");
        assert_eq!(super::CFSTR_SHELLIDLISTW[18], 0);
        assert_eq!(super::CFSTR_SHELLIDLISTOFFSETW[20], 0);
    }
}
