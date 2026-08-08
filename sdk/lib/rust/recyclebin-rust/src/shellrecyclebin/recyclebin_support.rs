//! Hand-written substrate for the shellrecyclebin translation.
//!
//! Lives outside the generated halves on purpose: a re-translate overwrites those wholesale,
//! and everything here has to survive that.
//!
//! Two C++ types the subtree uses that have no bindgen spelling:
//!
//!   * `CStringW` — ATL's string (atlstr.h). The island is generated in C mode (see the
//!     `recyclebin)` profile in gen_driver_types.sh), so ATL is not in the closure at all.
//!     PORTED VERBATIM from shell32-rust/src/atl.rs rather than reimplemented, and that is a
//!     correctness decision, not a DRY one: these strings build the paths that
//!     `RecycleBin5::Delete` and `::Restore` hand to MoveFileW and SHFileOperationW, so a
//!     `Left()` that clamps differently deletes a different file. Second copy in the tree —
//!     extraction into a shared crate is tracked, and must not be done opportunistically
//!     because shell32's resource helpers are inherent methods on the type.
//!
//!   * `CZZWStr` — a file-local RAII guard in recyclebin_v5.cpp:16 that owns an SHAlloc'd
//!     DOUBLE-NUL-terminated string for SHFILEOPSTRUCTW. Sealed to the translator via
//!     `--local-raii` so the drop semantics never go through the model.

use crate::prelude::*;

// ================= allocator ==================================================================
// LocalAlloc/LocalReAlloc/LocalFree, matching shell32-rust's CLocalAllocator. NOT the CRT heap
// ATL's CWin32Heap uses: this crate is no_std with no allocator, and LocalAlloc is already the
// module's allocation policy for owned buffers.
extern "system" {
    fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> *mut c_void;
    fn LocalReAlloc(hMem: *mut c_void, uBytes: SIZE_T, uFlags: UINT) -> *mut c_void;
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;
}
const LMEM_FIXED: UINT = 0x0000;
const LMEM_MOVEABLE: UINT = 0x0002;
const LMEM_ZEROINIT: UINT = 0x0040;

unsafe fn local_realloc(p: *mut c_void, bytes: SIZE_T) -> *mut c_void {
    if p.is_null() {
        LocalAlloc(LMEM_FIXED | LMEM_ZEROINIT, bytes)
    } else {
        LocalReAlloc(p, bytes, LMEM_MOVEABLE | LMEM_ZEROINIT)
    }
}

/// Length of a NUL-terminated wide string, in characters. `wcslen` without the CRT import.
pub unsafe fn wide_len(p: *const WCHAR) -> usize {
    if p.is_null() {
        return 0;
    }
    let mut n = 0usize;
    while *p.add(n) != 0 {
        n += 1;
    }
    n
}

// ================= CStringW ===================================================================
// Only the surface this subtree uses: assignment, the three `operator+=` overloads, `operator[]`,
// GetLength, Left, and the implicit `operator PCXSTR` conversion. No CStringData header, no
// refcount, no shared-empty singleton — no instance ever crosses back into C++.
//
// INVARIANT: `m_pszData` is either null (empty) or a NUL-terminated buffer of `m_nAlloc` WCHARs
// holding `m_nLength` characters. `as_ptr` NEVER returns null: ATL's PCXSTR conversion yields
// `L""` for an empty string and these call sites pass it straight to CreateFileW /
// GetFileAttributesW / MoveFileW, which would fault on null.
pub struct CStringW {
    m_pszData: *mut WCHAR,
    m_nLength: i32,
    m_nAlloc: i32,
}

/// The `L""` every empty `CStringW` hands out, so `as_ptr()` is never null.
static EMPTY_WSTR: [WCHAR; 1] = [0];

impl CStringW {
    /// `CStringT()` — the default constructor.
    pub const fn new() -> Self {
        Self { m_pszData: core::ptr::null_mut(), m_nLength: 0, m_nAlloc: 0 }
    }

    /// `CStringT(PCXSTR pszSrc)` — cstringt.h:463. A null source yields the empty string,
    /// matching ATL's `StringLength(NULL) == 0`.
    pub unsafe fn from_ptr(pszSrc: *const WCHAR) -> Self {
        let mut s = Self::new();
        s.SetString(pszSrc, wide_len(pszSrc) as i32);
        s
    }

    /// `operator PCXSTR()` / `GetString()` — cstringt.h. NEVER null; see the invariant above.
    pub fn as_ptr(&self) -> *const WCHAR {
        if self.m_pszData.is_null() {
            EMPTY_WSTR.as_ptr()
        } else {
            self.m_pszData
        }
    }

    /// `GetString()` under ATL's own name, for call sites translated from `.GetString()`.
    pub fn GetString(&self) -> *const WCHAR {
        self.as_ptr()
    }

    /// `GetLength()` — in characters, excluding the terminator.
    pub fn GetLength(&self) -> i32 {
        self.m_nLength
    }

    /// `IsEmpty()`.
    pub fn IsEmpty(&self) -> bool {
        self.m_nLength == 0
    }

    /// Grow the buffer to hold at least `nChars` characters plus the terminator. Never shrinks.
    unsafe fn Reserve(&mut self, nChars: i32) {
        if nChars < self.m_nAlloc {
            return;
        }
        let need = nChars + 1;
        let p = local_realloc(
            self.m_pszData as *mut c_void,
            need as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
        ) as *mut WCHAR;
        if p.is_null() {
            return;
        }
        self.m_pszData = p;
        self.m_nAlloc = need;
        *self.m_pszData.add(self.m_nLength as usize) = 0;
    }

    /// `SetString(PCXSTR pszSrc, int nLength)` — cstringt.h. Copies `nLength` characters and
    /// terminates; the source need not be NUL-terminated at that point.
    pub unsafe fn SetString(&mut self, pszSrc: *const WCHAR, nLength: i32) {
        if pszSrc.is_null() || nLength <= 0 {
            self.m_nLength = 0;
            if !self.m_pszData.is_null() {
                *self.m_pszData = 0;
            }
            return;
        }
        self.Reserve(nLength);
        if self.m_pszData.is_null() {
            return;
        }
        core::ptr::copy_nonoverlapping(pszSrc, self.m_pszData, nLength as usize);
        *self.m_pszData.add(nLength as usize) = 0;
        self.m_nLength = nLength;
    }

    /// `Left(int nCount)` — cstringt.h:776. ATL clamps: negative yields empty, over-long yields
    /// the whole string. `RecycleBin5::Initialize` uses this to trim `\INFO2` back off
    /// `m_Folder`, so the clamp behaviour is load-bearing.
    pub unsafe fn Left(&self, nCount: i32) -> Self {
        let n = if nCount < 0 {
            0
        } else if nCount > self.m_nLength {
            self.m_nLength
        } else {
            nCount
        };
        let mut s = Self::new();
        s.SetString(self.as_ptr(), n);
        s
    }

    /// `operator+=(XCHAR ch)` — cstringt.h. Appends the character even when it is NUL.
    pub unsafe fn AppendChar(&mut self, ch: WCHAR) {
        let newLen = self.m_nLength + 1;
        self.Reserve(newLen);
        if self.m_pszData.is_null() {
            return;
        }
        *self.m_pszData.add(self.m_nLength as usize) = ch;
        *self.m_pszData.add(newLen as usize) = 0;
        self.m_nLength = newLen;
    }

    /// `operator[](int iChar)` — cstringt.h. `RecycleBin5::GetDirectory` reads `m_Folder[0]` to
    /// test for the empty string, which is inside the allocation whenever one exists.
    pub unsafe fn GetAt(&self, iChar: i32) -> WCHAR {
        if self.m_pszData.is_null() || iChar < 0 || iChar >= self.m_nAlloc {
            return 0;
        }
        *self.m_pszData.add(iChar as usize)
    }

    /// `operator+=(PCXSTR)` — cstringt.h. Appending to self is not a case these call sites
    /// produce, and the realloc would invalidate `pszSrc`, so it is not supported.
    pub unsafe fn Append(&mut self, pszSrc: *const WCHAR) {
        let add = wide_len(pszSrc) as i32;
        if add == 0 {
            return;
        }
        let newLen = self.m_nLength + add;
        self.Reserve(newLen);
        if self.m_pszData.is_null() {
            return;
        }
        core::ptr::copy_nonoverlapping(
            pszSrc,
            self.m_pszData.add(self.m_nLength as usize),
            add as usize,
        );
        *self.m_pszData.add(newLen as usize) = 0;
        self.m_nLength = newLen;
    }

    /// `operator=(const CStringT&)` / `operator+=(const CStringT&)` helpers, so a call site
    /// translated from `m_Folder = VolumePath;` does not have to reach for `as_ptr()` itself.
    pub unsafe fn AssignStr(&mut self, other: &CStringW) {
        self.SetString(other.as_ptr(), other.GetLength());
    }
    pub unsafe fn AppendStr(&mut self, other: &CStringW) {
        self.Append(other.as_ptr());
    }
}

impl Default for CStringW {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CStringW {
    fn clone(&self) -> Self {
        unsafe {
            let mut s = Self::new();
            s.SetString(self.as_ptr(), self.m_nLength);
            s
        }
    }
}

impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.m_pszData.is_null() {
                LocalFree(self.m_pszData as *mut c_void);
                self.m_pszData = core::ptr::null_mut();
                self.m_nLength = 0;
                self.m_nAlloc = 0;
            }
        }
    }
}

// ================= CZZWStr ====================================================================
// recyclebin_v5.cpp:16. Owns an SHAlloc'd string with a SECOND NUL after the terminator, which
// is what SHFILEOPSTRUCTW's pFrom/pTo require (they are double-NUL-terminated lists). The extra
// element is why Initialize allocates `cch + 1` and writes `m_sz[cch] = 0` on top of the copy's
// own terminator.
pub struct CZZWStr {
    m_sz: LPWSTR,
}

impl CZZWStr {
    pub const fn new() -> Self {
        Self { m_sz: core::ptr::null_mut() }
    }

    /// `bool Initialize(LPCWSTR Str)` — false on allocation failure, exactly as the C++.
    pub unsafe fn Initialize(&mut self, Str: LPCWSTR) -> bool {
        let cch = wide_len(Str) + 1;
        self.m_sz = SHAlloc(
            ((cch + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as LPWSTR;
        if self.m_sz.is_null() {
            return false;
        }
        core::ptr::copy_nonoverlapping(Str, self.m_sz, cch);
        *self.m_sz.add(cch) = 0; // double-NUL terminate
        true
    }

    /// `LPWSTR c_str()`.
    pub fn c_str(&self) -> LPWSTR {
        self.m_sz
    }
}

impl Default for CZZWStr {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CZZWStr {
    fn drop(&mut self) {
        unsafe {
            if !self.m_sz.is_null() {
                SHFree(self.m_sz as *mut c_void);
                self.m_sz = core::ptr::null_mut();
            }
        }
    }
}

// ================= wide literals ==============================================================
// The `L"..."` constants the two v5 TUs use. Hand-encoded UTF-16 with the intent comment above
// each one, which is the form `tools/scan_wide_literals.py` decodes and checks — a miscounted
// length is invisible to rustc (the array is well-formed at ANY length), and that is exactly how
// SERVICES_KEY once shipped one character short and broke every service registration.
//
// Defined HERE rather than in the generated modules because this file survives a re-translate.

/// L"*"
pub static W_STAR: [WCHAR; 2] = [0x002A, 0];

/// L"\\"
pub static W_BACKSLASH: [WCHAR; 2] = [0x005C, 0];

/// L"\\D"
pub static W_BACKSLASH_D: [WCHAR; 3] = [0x005C, 0x0044, 0];

/// L"desktop.ini"
pub static W_DESKTOP_INI: [WCHAR; 12] = [
    0x0064, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x002E, 0x0069, 0x006E, 0x0069, 0,
];

/// L"%lu"
pub static W_FMT_LU: [WCHAR; 4] = [0x0025, 0x006C, 0x0075, 0];

/// `L'\\'` — a wide CHARACTER literal, not a string.
pub const WC_BACKSLASH: WCHAR = 0x005C;

// The three `#define`s from recyclebin_private.h:26-28. Exposed as pointers under their C
// names so the generated call sites — `wcslen(RECYCLE_BIN_FILE_NAME)` — need no edit, and so a
// re-translate does not have to be re-patched.

/// L"INFO2"
pub static W_INFO2: [WCHAR; 6] = [0x0049, 0x004E, 0x0046, 0x004F, 0x0032, 0];
/// L"RECYCLER"
pub static W_RECYCLER: [WCHAR; 9] =
    [0x0052, 0x0045, 0x0043, 0x0059, 0x0043, 0x004C, 0x0045, 0x0052, 0];
/// L"RECYCLED"
pub static W_RECYCLED: [WCHAR; 9] =
    [0x0052, 0x0045, 0x0043, 0x0059, 0x0043, 0x004C, 0x0045, 0x0044, 0];

pub const RECYCLE_BIN_FILE_NAME: LPCWSTR = W_INFO2.as_ptr();
pub const RECYCLE_BIN_DIRECTORY_WITH_ACL: LPCWSTR = W_RECYCLER.as_ptr();
pub const RECYCLE_BIN_DIRECTORY_WITHOUT_ACL: LPCWSTR = W_RECYCLED.as_ptr();

// ---- snake_case spellings -------------------------------------------------------------------
// `--local-raii` seals this API into the prompt, but the model rendered the calls in Rust
// snake_case (`assign`/`append_str`/`left`) rather than ATL's own PascalCase. Forwarders rather
// than a rewrite of the generated call sites: they cost nothing, they keep BOTH spellings valid,
// and a re-translate that picks either one still compiles. The ATL names above stay the
// definition site so there is exactly one implementation of each operation.
impl CStringW {
    /// `operator=(PCXSTR)`
    pub unsafe fn assign(&mut self, pszSrc: *const WCHAR) {
        self.SetString(pszSrc, wide_len(pszSrc) as i32);
    }
    /// `operator=(const CStringT&)`
    pub unsafe fn assign_cstringw(&mut self, other: &CStringW) {
        self.AssignStr(other);
    }
    /// `operator+=(PCXSTR)`
    pub unsafe fn append_str(&mut self, pszSrc: *const WCHAR) {
        self.Append(pszSrc);
    }
    /// `operator+=(XCHAR)`
    pub unsafe fn append_char(&mut self, ch: WCHAR) {
        self.AppendChar(ch);
    }
    /// `Left(int)`
    pub unsafe fn left(&self, nCount: i32) -> Self {
        self.Left(nCount)
    }
    /// `GetLength()`
    pub fn len(&self) -> i32 {
        self.GetLength()
    }
}
