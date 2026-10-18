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
