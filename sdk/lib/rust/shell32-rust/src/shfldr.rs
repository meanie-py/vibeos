//! The C++ struct that lives in `dll/win32/shell32/shfldr.h`, alongside that header's
//! `EXTERN_C` key-array functions.
//!
//! `CRegKeyHandleArray` is `#ifdef __cplusplus`-only: it is a header-inline class, so no
//! translated TU carries it and every TU that declares one has to re-derive it. Two already
//! did (`cfs_folder.rs`, `desktop_folder.rs`) and both re-derived it WITHOUT the destructor —
//! the same defect twice, which is the signal that says "one definition, not N".
//!
//! A re-declared `extern` fn is cheap to repeat and the crate does it freely; a `Drop` is not.
//! Getting a `Drop` wrong is a leak that nothing in the build can see, so this type has exactly
//! one definition and the leak can only be reintroduced in one place.
//!
//! `crate::types` world (`HKEY = HANDLE = *mut c_void`), matching every consumer. The island's
//! `HKEY` is `*mut HKEY__` — a different Rust type of the same width — so a consumer written
//! against `crate::island` must not use this struct without saying so deliberately.

use crate::types::{HKEY, UINT};

extern "C" {
    /// `shfldr.h:184` `EXTERN_C void CloseRegKeyArray(HKEY*, UINT)` — implemented in
    /// `shell32/shlfolder.rs`. DECLARED, not imported: that module is written against the
    /// island's `HKEY` while this one uses `crate::types`, and two aliases of the same width
    /// are still two different Rust types.
    fn CloseRegKeyArray(array: *mut HKEY, cKeys: UINT);
}

/// `shfldr.h:191` — `struct CRegKeyHandleArray { HKEY hKeys[16]; UINT cKeys; }`.
///
/// The C++ conversion operators have no Rust equivalent and are spelled out at each call site:
/// `operator HKEY*()` → `.hKeys.as_mut_ptr()`, `operator UINT*()` → `&mut .cKeys`,
/// `operator UINT()` → [`len`](Self::len). Writing `f(keys, keys)` in C++ passes two DIFFERENT
/// pointer types from one object; in Rust the two arguments must be spelled differently.
#[repr(C)]
pub struct CRegKeyHandleArray {
    pub hKeys: [HKEY; 16],
    pub cKeys: UINT,
}

impl CRegKeyHandleArray {
    #[inline]
    pub fn len(&self) -> usize {
        self.cKeys as usize
    }

    #[inline]
    pub fn as_ptr(&self) -> *const HKEY {
        self.hKeys.as_ptr()
    }
}

/// C++ `CRegKeyHandleArray() : cKeys(0) {}`. The array itself is left uninitialised there and
/// is only ever read up to `cKeys`, so zeroing the whole thing changes no path.
impl Default for CRegKeyHandleArray {
    #[inline]
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// The C++ DESTRUCTOR (`shfldr.h:198`), which both earlier translations dropped along with the
/// rest of the class. `AddClassKeyToArray` OPENS each key, so without this every `IContextMenu`
/// creation leaked up to 16 HKEYs — once per right-click, on a file or on the desktop.
///
/// Drop order matches C++ scope exit: `keys` is declared before the `DEFCONTEXTMENU` that
/// borrows it, so it is released after `SHCreateDefaultContextMenu` returns.
impl Drop for CRegKeyHandleArray {
    fn drop(&mut self) {
        unsafe { CloseRegKeyArray(self.hKeys.as_mut_ptr(), self.cKeys) }
    }
}
