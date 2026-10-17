//! Clipboard helper functions — translation of dll/win32/shell32/wine/clipboard.c.
//!
//! Builds clipboard HGLOBAL blocks (CF_HDROP / "Shell IDList Array" / CF_FILENAME{A,W})
//! from PIDLs. All four exports are plain cdecl (no WINAPI in the C signatures) →
//! `extern "C"` → undecorated `RenderHDROP` etc. (NO `@N`), confirmed against the i386
//! object's symbol table. This is the ReactOS (`__REACTOS__`) branch of each function.

#![allow(non_snake_case)]

use crate::types::*;
// The crate's own functions: the definition is the declaration.
use crate::wine::pidl::ILCombine;
use crate::wine::pidl::ILFree;
use crate::wine::pidl::ILGetSize;
use crate::wine::pidl::SHGetPathFromIDListW;
// One declaration per external symbol: island.rs.
use crate::island::GlobalAlloc;
use crate::island::HeapAlloc;
use core::mem::size_of;
use core::ptr;

// ── file-local types not yet in types.rs ───────────────────────────────────────────────
// ADDED-TO-TYPES?: SIZE_T, HGLOBAL, ITEMIDLIST/LPITEMIDLIST, DROPFILES, POINT are all
// common shell/win32 types — candidates to hoist into types.rs later.

pub use ntdll_sys::nt::SIZE_T; // ULONG_PTR (basetsd.h): 4 bytes on i386, 8 on amd64/arm64
pub type HGLOBAL = HANDLE; // winnt.h

/// ITEMIDLIST (shtypes.idl) — only ever used through a pointer here, so kept opaque.
#[repr(C)]
pub struct ITEMIDLIST {
    _private: [u8; 0],
}
pub type LPITEMIDLIST = *mut ITEMIDLIST;
pub type LPCITEMIDLIST = *const ITEMIDLIST;

/// POINT (windef.h): { LONG x; LONG y; }
pub use crate::island::POINT;

/// DROPFILES (shlobj.h) for CF_HDROP: { DWORD pFiles; POINT pt; BOOL fNC; BOOL fWide; }
pub use crate::island::DROPFILES;

/// CIDA / *LPIDA (shlobj.h): { UINT cidl; UINT aoffset[1]; } — header + offset table.
/// Accessed via raw pointer arithmetic because of the flexible `aoffset[]` array.
#[repr(C)]
pub struct CIDA {
    pub cidl: UINT,
    pub aoffset: [UINT; 1],
}

// ── constants pinned from SDK headers ───────────────────────────────────────────────────
const MAX_PATH: usize = 260; // minwindef.h
const GHND: UINT = 66; // winbase.h: GMEM_MOVEABLE(2) | GMEM_ZEROINIT(64)
const GMEM_SHARE: UINT = 8192; // winbase.h
const GHND_SHARE: UINT = GHND | GMEM_SHARE; // 0x2042

// ── imports (resolved via shell32's own exports + kernel32 import lib at link) ──────────
extern "system" {
    // PIDL helpers — exported by shell32 itself (WINAPI on i386 → `_ILCombine@8` etc.)
// @8
// @4
// @4
    fn SHFree(pv: LPVOID); // @4
    fn SHGetPathFromIDListA(pidl: LPCITEMIDLIST, pszPath: LPSTR) -> BOOL; // @8
// @8
    // kernel32
// @8
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID; // @4
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL; // @4
    fn GetProcessHeap() -> HANDLE; // @0
// @12
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // @12
}

// ── trivial in-file reimplementations of the libc/unicode helpers (per contract) ────────
#[inline]
unsafe fn r_strlen(mut s: *const CHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}

#[inline]
unsafe fn r_wcslen(mut s: *const WCHAR) -> usize {
    let mut n = 0usize;
    while *s != 0 {
        s = s.add(1);
        n += 1;
    }
    n
}

#[inline]
unsafe fn r_wcscpy(dst: *mut WCHAR, mut src: *const WCHAR) -> *mut WCHAR {
    let mut d = dst;
    loop {
        let c = *src;
        *d = c;
        if c == 0 {
            break;
        }
        d = d.add(1);
        src = src.add(1);
    }
    dst
}

/// memcpy-equivalent forward byte copy (clipboard blocks never overlap their source).
#[inline]
unsafe fn r_memcpy(dst: *mut u8, src: *const u8, n: usize) {
    let mut i = 0usize;
    while i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
}

/// **************************************************************************
/// RenderHDROP — creates a CF_HDROP structure.
#[no_mangle]
pub unsafe extern "C" fn RenderHDROP(
    pidlRoot: LPITEMIDLIST,
    apidl: *mut LPITEMIDLIST,
    cidl: UINT,
) -> HGLOBAL {
    let mut wszFileName: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut hGlobal: HGLOBAL = ptr::null_mut();

    let pidls = HeapAlloc(
        GetProcessHeap(),
        0,
        (cidl as usize * size_of::<LPITEMIDLIST>()) as _,
    ) as *mut LPITEMIDLIST;
    if !pidls.is_null() {
        // get the size needed
        let mut size: i32 = size_of::<DROPFILES>() as i32;

        let mut i: UINT = 0;
        while i < cidl {
            let combined = ILCombine((pidlRoot as LPCITEMIDLIST).cast(), (*apidl.add(i as usize) as LPCITEMIDLIST).cast());
            *pidls.add(i as usize) = (combined).cast();
            SHGetPathFromIDListW((combined as LPCITEMIDLIST).cast(), wszFileName.as_mut_ptr());
            size += (r_wcslen(wszFileName.as_ptr()) as i32 + 1) * size_of::<WCHAR>() as i32;
            i += 1;
        }

        size += size_of::<WCHAR>() as i32;

        // Fill the structure
        hGlobal = GlobalAlloc(GHND_SHARE, size as SIZE_T);
        if !hGlobal.is_null() {
            let pDropFiles = GlobalLock(hGlobal) as *mut DROPFILES;
            let mut offset: i32 =
                ((size_of::<DROPFILES>() + size_of::<WCHAR>() - 1) / size_of::<WCHAR>()) as i32;
            (*pDropFiles).pFiles = (offset * size_of::<WCHAR>() as i32) as DWORD;
            (*pDropFiles).fWide = TRUE;

            let base = pDropFiles as *mut WCHAR;
            let mut j: UINT = 0;
            while j < cidl {
                let p = *pidls.add(j as usize);
                SHGetPathFromIDListW((p as LPCITEMIDLIST).cast(), wszFileName.as_mut_ptr());
                r_wcscpy(base.add(offset as usize), wszFileName.as_ptr());
                offset += r_wcslen(wszFileName.as_ptr()) as i32 + 1;
                ILFree((p).cast());
                j += 1;
            }

            *base.add(offset as usize) = 0;
            GlobalUnlock(hGlobal);
        }

        // cleanup
        HeapFree(GetProcessHeap(), 0, pidls as LPVOID);
    }

    hGlobal
}

/// **************************************************************************
/// RenderSHELLIDLIST — creates a "Shell IDList Array" (CIDA) block.
#[no_mangle]
pub unsafe extern "C" fn RenderSHELLIDLIST(
    pidlRoot: LPITEMIDLIST,
    apidl: *mut LPITEMIDLIST,
    cidl: UINT,
) -> HGLOBAL {
    // get the size needed
    // size = sizeof(CIDA) + sizeof(UINT)*cidl   (header)
    let mut size: i32 = size_of::<CIDA>() as i32 + size_of::<UINT>() as i32 * cidl as i32;
    size += ILGetSize((pidlRoot as LPCITEMIDLIST).cast()) as i32; // root pidl
    let mut i: UINT = 0;
    while i < cidl {
        size += ILGetSize((*apidl.add(i as usize) as LPCITEMIDLIST).cast()) as i32; // child pidls
        i += 1;
    }

    // fill the structure
    let hGlobal = GlobalAlloc(GHND_SHARE, size as SIZE_T);
    if hGlobal.is_null() {
        return hGlobal;
    }
    let pcida = GlobalLock(hGlobal) as *mut CIDA;
    (*pcida).cidl = cidl;

    let base = pcida as *mut u8;
    // aoffset[] table starts right after `cidl`, i.e. at offset 4 within CIDA.
    let aoffset = (base.add(size_of::<UINT>())) as *mut UINT;

    // root pidl
    let mut offset: i32 = size_of::<CIDA>() as i32 + size_of::<UINT>() as i32 * cidl as i32;
    *aoffset.add(0) = offset as UINT; // first element
    let mut sizePidl: i32 = ILGetSize((pidlRoot as LPCITEMIDLIST).cast()) as i32;
    r_memcpy(
        base.add(offset as usize),
        pidlRoot as *const u8,
        sizePidl as usize,
    );
    offset += sizePidl;

    let mut k: UINT = 0;
    while k < cidl {
        // child pidls
        *aoffset.add((k + 1) as usize) = offset as UINT;
        let child = *apidl.add(k as usize);
        sizePidl = ILGetSize((child as LPCITEMIDLIST).cast()) as i32;
        r_memcpy(base.add(offset as usize), child as *const u8, sizePidl as usize);
        offset += sizePidl;
        k += 1;
    }

    GlobalUnlock(hGlobal);
    hGlobal
}

/// **************************************************************************
/// RenderFILENAMEA — CF_FILENAMEA: ANSI path of the first combined PIDL.
#[no_mangle]
pub unsafe extern "C" fn RenderFILENAMEA(
    pidlRoot: LPITEMIDLIST,
    apidl: *mut LPITEMIDLIST,
    _cidl: UINT,
) -> HGLOBAL {
    let mut szTemp: [CHAR; MAX_PATH] = [0; MAX_PATH];

    // get path of combined pidl
    let pidl = ILCombine((pidlRoot as LPCITEMIDLIST).cast(), (*apidl as LPCITEMIDLIST).cast());
    if pidl.is_null() {
        return ptr::null_mut();
    }

    let bSuccess = SHGetPathFromIDListA(pidl as LPCITEMIDLIST, szTemp.as_mut_ptr());
    SHFree(pidl as LPVOID);
    if bSuccess == FALSE {
        return ptr::null_mut();
    }

    let size: i32 = r_strlen(szTemp.as_ptr()) as i32 + 1;

    // fill the structure
    let hGlobal = GlobalAlloc(GHND_SHARE, size as SIZE_T);
    if hGlobal.is_null() {
        return hGlobal;
    }
    let szFileName = GlobalLock(hGlobal) as *mut u8;
    r_memcpy(szFileName, szTemp.as_ptr() as *const u8, size as usize);
    GlobalUnlock(hGlobal);

    hGlobal
}

/// **************************************************************************
/// RenderFILENAMEW — CF_FILENAMEW: Unicode path of the first combined PIDL.
#[no_mangle]
pub unsafe extern "C" fn RenderFILENAMEW(
    pidlRoot: LPITEMIDLIST,
    apidl: *mut LPITEMIDLIST,
    _cidl: UINT,
) -> HGLOBAL {
    let mut szTemp: [WCHAR; MAX_PATH] = [0; MAX_PATH];

    // get path of combined pidl
    let pidl = ILCombine((pidlRoot as LPCITEMIDLIST).cast(), (*apidl as LPCITEMIDLIST).cast());
    if pidl.is_null() {
        return ptr::null_mut();
    }

    let bSuccess = SHGetPathFromIDListW((pidl as LPCITEMIDLIST).cast(), szTemp.as_mut_ptr());
    SHFree(pidl as LPVOID);
    if bSuccess == FALSE {
        return ptr::null_mut();
    }

    let size: i32 = (r_wcslen(szTemp.as_ptr()) as i32 + 1) * size_of::<WCHAR>() as i32;

    // fill the structure
    let hGlobal = GlobalAlloc(GHND_SHARE, size as SIZE_T);
    if hGlobal.is_null() {
        return hGlobal;
    }
    let szFileName = GlobalLock(hGlobal) as *mut u8;
    r_memcpy(szFileName, szTemp.as_ptr() as *const u8, size as usize);
    GlobalUnlock(hGlobal);

    hGlobal
}

// ADDED-TO-TYPES?: SIZE_T, HGLOBAL, ITEMIDLIST/LPITEMIDLIST/LPCITEMIDLIST, POINT,
// DROPFILES, CIDA — common shell/win32 layouts worth hoisting into types.rs in a later wave.
