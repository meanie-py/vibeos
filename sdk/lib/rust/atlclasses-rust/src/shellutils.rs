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

#[cfg(test)]
mod tests {
    #[no_mangle]
    extern "system" fn RegisterClipboardFormatW(_f: *const u16) -> u32 {
        0xC0DE
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
