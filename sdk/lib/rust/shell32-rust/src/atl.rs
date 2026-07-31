//! ATL's smart pointers and containers, as shell32's C++ actually uses them.
//!
//! 25 of the 38 C++ TUs left in `dll/win32/shell32/` mention `CComPtr`, and 19 mention
//! `CComHeapPtr`. That makes this module the shared gate in front of essentially all
//! remaining shell32 translation, not a per-file tail — which is why it is worth building
//! properly rather than stubbing per TU.
//!
//! Everything here is derived from MEASURED usage in the translated output, not from ATL's
//! full published surface. The header of each type records what the call sites actually do.
//!
//! # Three invariants the translated code depends on, all of them load-bearing
//!
//! **1. These types must be zeroable.** The translation emits `core::mem::zeroed()` as a
//! smart-pointer initialiser (`let mut pEnum: CComPtr<IEnumIDList> = core::mem::zeroed()`),
//! because that is how it lowers a C++ default constructor it did not recognise. Every type
//! here is therefore a raw pointer plus zero-sized fields, and "all zero" is its valid empty
//! state. Do not add a `Vec`, a `NonNull`, or any other field with a niche: a zeroed `Vec`
//! violates its own non-null invariant and is instant UB.
//!
//! **2. The pointer types must be `#[repr(transparent)]`.** The translation reproduces ATL's
//! `operator&` by casting the address of the smart pointer:
//!
//! ```ignore
//! pEnum.Next(1, &mut pidlTemp as *mut _ as *mut *mut ITEMIDLIST_RELATIVE, ...)
//! ```
//!
//! That is only sound if the smart pointer has the layout of the raw pointer it wraps, which
//! `repr(transparent)` guarantees and a plain `struct` does not. Like ATL's `operator&`, this
//! overwrites the held pointer WITHOUT releasing it first; that matches the C++ we translated
//! from, so it stays.
//!
//! **3. `Drop` is required, and it is what makes the translation correct.** The translation
//! DROPPED the C++ destructors — it left at most a `// Destructor for CComPtr<IShellFolder>
//! ... call Release` comment and no code. So the released-at-scope-exit behaviour the C++
//! relied on exists only if these types implement `Drop`. Implementing it here restores the
//! RAII for every call site at once. The corollary: if a future translation ALSO emits an
//! explicit `.Release()`/`.Free()` on one of these, that is a double free — the explicit call
//! is the one to delete, never the `Drop`.
//!
//! # Why `CComPtr::drop` may call `Release` through `IUnknown`
//!
//! Every COM interface begins with a vtable pointer whose first three slots are
//! QueryInterface/AddRef/Release, because every interface derives from IUnknown. Casting
//! `*mut T` to `*mut IUnknown` to reach `Release` is therefore ABI-sound for any COM `T`, and
//! is exactly what ATL's own `CComPtrBase::Release` does. It is NOT sound for a non-COM `T`,
//! so `CComPtr` is only ever instantiated over interface types.

use core::ffi::c_void;
use core::marker::PhantomData;

use crate::island::{
    CoTaskMemAlloc, CoTaskMemFree, CoTaskMemRealloc, LocalAlloc, LocalFree, LocalReAlloc, BOOL,
    HLOCAL, SIZE_T, WCHAR,
};
use crate::shellcom::IUnknown;

/// `LMEM_FIXED` — LocalAlloc returns a pointer, not a movable handle. winbase.h.
const LMEM_FIXED: u32 = 0x0000;
/// `LMEM_MOVEABLE` — required by LocalReAlloc to grow a block in place. winbase.h.
const LMEM_MOVEABLE: u32 = 0x0002;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn realloc(p: *mut c_void, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

// -------------------------------------------------------------------------------------------
// Allocator policies
// -------------------------------------------------------------------------------------------

/// ATL's allocator policy: the third parameter of `CHeapPtr` and friends.
///
/// ATL spells these as static members, so they take no `self` and the policy type is never
/// instantiated. Kept that way here — the policy is a marker, and `CHeapPtr` holds it in a
/// `PhantomData` so the wrapper stays pointer-sized (see invariant 2 above).
pub trait CAllocator {
    /// Byte count, not element count — the element arithmetic lives in `CHeapPtr::Allocate`.
    unsafe fn Allocate(nBytes: usize) -> *mut c_void;
    unsafe fn Reallocate(p: *mut c_void, nBytes: usize) -> *mut c_void;
    unsafe fn Free(p: *mut c_void);
}

/// `CCRTAllocator` — malloc/realloc/free. ATL's default for `CHeapPtr`.
pub struct CCRTAllocator;

impl CAllocator for CCRTAllocator {
    unsafe fn Allocate(nBytes: usize) -> *mut c_void {
        malloc(nBytes)
    }
    unsafe fn Reallocate(p: *mut c_void, nBytes: usize) -> *mut c_void {
        realloc(p, nBytes)
    }
    unsafe fn Free(p: *mut c_void) {
        free(p)
    }
}

/// `CLocalAllocator` — LocalAlloc/LocalReAlloc/LocalFree.
///
/// shell32 uses this where the buffer is handed to (or received from) a Win32 API that
/// documents LocalFree as its release call — `FormatMessage`, `CommandLineToArgvW`.
pub struct CLocalAllocator;

impl CAllocator for CLocalAllocator {
    unsafe fn Allocate(nBytes: usize) -> *mut c_void {
        LocalAlloc(LMEM_FIXED, nBytes as SIZE_T) as *mut c_void
    }
    unsafe fn Reallocate(p: *mut c_void, nBytes: usize) -> *mut c_void {
        // LocalReAlloc refuses a null block; ATL's policy allocates instead.
        if p.is_null() {
            return Self::Allocate(nBytes);
        }
        LocalReAlloc(p as HLOCAL, nBytes as SIZE_T, LMEM_MOVEABLE) as *mut c_void
    }
    unsafe fn Free(p: *mut c_void) {
        LocalFree(p as HLOCAL);
    }
}

/// `CComAllocator` — CoTaskMemAlloc/Realloc/Free, the COM task allocator.
///
/// This is the one that matters most here: shell32's PIDLs are `SHAlloc`ed, and `SHAlloc` IS
/// the COM task allocator, so `CComHeapPtr<ITEMIDLIST>` frees a PIDL correctly.
pub struct CComAllocator;

impl CAllocator for CComAllocator {
    unsafe fn Allocate(nBytes: usize) -> *mut c_void {
        CoTaskMemAlloc(nBytes as SIZE_T)
    }
    unsafe fn Reallocate(p: *mut c_void, nBytes: usize) -> *mut c_void {
        CoTaskMemRealloc(p, nBytes as SIZE_T)
    }
    unsafe fn Free(p: *mut c_void) {
        CoTaskMemFree(p)
    }
}

// -------------------------------------------------------------------------------------------
// CHeapPtr / CComHeapPtr
// -------------------------------------------------------------------------------------------

/// `CHeapPtr<T, Allocator>` — an owning pointer to a heap block, freed by the policy.
///
/// Measured surface: `new()`, `Allocate(n)`, `Attach(p)`, `Detach()`, `Free()`, and the
/// `&ptr as *mut _ as *mut *mut T` address-of idiom (invariant 2).
#[repr(transparent)]
pub struct CHeapPtr<T, A: CAllocator = CCRTAllocator> {
    pub m_pData: *mut T,
    _alloc: PhantomData<A>,
}

/// `CComHeapPtr<T>` — `CHeapPtr` on the COM task allocator. In ATL this is a derived class;
/// an alias is the faithful spelling in Rust because it adds no members of its own.
pub type CComHeapPtr<T> = CHeapPtr<T, CComAllocator>;

impl<T, A: CAllocator> CHeapPtr<T, A> {
    /// The default constructor: an empty pointer. ATL's one-argument `CHeapPtr(T*)` attaches
    /// instead — that is `from_raw`, because Rust has no overloading.
    pub const fn new() -> Self {
        Self { m_pData: core::ptr::null_mut(), _alloc: PhantomData }
    }

    /// ATL's `CHeapPtr(T* pData)` — takes ownership of an already-allocated block.
    pub const fn from_raw(p: *mut T) -> Self {
        Self { m_pData: p, _alloc: PhantomData }
    }

    /// ATL's `Allocate(nElements = 1)`: allocates `nElements * sizeof(T)` bytes.
    ///
    /// Returns `BOOL`, as ATL does, and NOT `bool`. That is deliberate: a call site written
    /// `if !p.Allocate(n)` is a translation artifact rather than a faithful rendering of the
    /// C++, and returning `BOOL` makes it a loud type error instead of silently flipping the
    /// bits of a success code.
    ///
    /// Like ATL, this leaks any block already held — the C++ only calls it on a fresh pointer.
    pub unsafe fn Allocate(&mut self, nElements: usize) -> BOOL {
        let bytes = nElements.wrapping_mul(core::mem::size_of::<T>());
        self.m_pData = A::Allocate(bytes) as *mut T;
        (!self.m_pData.is_null()) as BOOL
    }

    /// ATL's `Reallocate(nElements)`. Preserves the old block if the allocator moves it.
    pub unsafe fn Reallocate(&mut self, nElements: usize) -> BOOL {
        let bytes = nElements.wrapping_mul(core::mem::size_of::<T>());
        let p = A::Reallocate(self.m_pData as *mut c_void, bytes) as *mut T;
        if p.is_null() {
            return 0;
        }
        self.m_pData = p;
        1
    }

    /// ATL's `Attach(T*)`: free whatever is held, then take ownership of `p`.
    pub unsafe fn Attach(&mut self, p: *mut T) {
        if !self.m_pData.is_null() {
            A::Free(self.m_pData as *mut c_void);
        }
        self.m_pData = p;
    }

    /// ATL's `Detach()`: give up ownership without freeing, and return the raw pointer.
    pub fn Detach(&mut self) -> *mut T {
        let p = self.m_pData;
        self.m_pData = core::ptr::null_mut();
        p
    }

    /// ATL's `Free()`: release the block and become empty. Safe to call more than once.
    pub unsafe fn Free(&mut self) {
        if !self.m_pData.is_null() {
            A::Free(self.m_pData as *mut c_void);
            self.m_pData = core::ptr::null_mut();
        }
    }

    pub fn is_null(&self) -> bool {
        self.m_pData.is_null()
    }

    /// The raw pointer, without giving up ownership — ATL's `operator T*`.
    pub fn as_ptr(&self) -> *mut T {
        self.m_pData
    }

    /// `CHeapPtrBase::operator&()` — `T**` for an out-parameter. Same debug-only ATL assert as
    /// `CComPtr::GetAddressOf`, and the same reason it is not a panic here.
    pub fn put(&mut self) -> *mut *mut T {
        core::ptr::addr_of_mut!(self.m_pData)
    }

    /// The translation emits all three spellings of ATL's two accessors; these are `as_ptr`
    /// and `put` under the names `CComPtr` uses, so a call site reads the same either way.
    pub fn Get(&self) -> *mut T {
        self.m_pData
    }
    pub fn GetAddressOf(&mut self) -> *mut *mut T {
        core::ptr::addr_of_mut!(self.m_pData)
    }
}

impl<T, A: CAllocator> core::ops::Deref for CHeapPtr<T, A> {
    type Target = *mut T;
    fn deref(&self) -> &*mut T {
        &self.m_pData
    }
}

impl<T, A: CAllocator> Default for CHeapPtr<T, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, A: CAllocator> Drop for CHeapPtr<T, A> {
    fn drop(&mut self) {
        if !self.m_pData.is_null() {
            unsafe { A::Free(self.m_pData as *mut c_void) };
            self.m_pData = core::ptr::null_mut();
        }
    }
}

// -------------------------------------------------------------------------------------------
// CComPtr
// -------------------------------------------------------------------------------------------

/// `CComPtr<T>` — an owning COM interface pointer; `Release`d on drop.
///
/// `T` must be a COM interface (first member a vtable pointer whose slot 2 is `Release`); see
/// the module header for why that makes the `IUnknown` cast sound.
///
/// The field is named `p` because ATL names it `p` and the translated code reads it directly.
#[repr(transparent)]
pub struct CComPtr<T> {
    pub p: *mut T,
}

impl<T> CComPtr<T> {
    /// ATL's default constructor — a null interface pointer.
    pub const fn new() -> Self {
        Self { p: core::ptr::null_mut() }
    }

    /// Spelled `null()` at some call sites; same thing, kept so both idioms compile.
    pub const fn null() -> Self {
        Self::new()
    }

    /// Takes ownership of an interface pointer that the caller already holds a reference to.
    /// This does NOT AddRef, matching ATL's `Attach`.
    pub const fn from_raw(p: *mut T) -> Self {
        Self { p }
    }

    /// ATL's `Attach(T*)`: release the current interface, then adopt `p` without AddRef'ing.
    pub unsafe fn Attach(&mut self, p: *mut T) {
        if !self.p.is_null() {
            let u = self.p as *mut IUnknown;
            ((*(*u).lpVtbl).Release)(u);
        }
        self.p = p;
    }

    /// ATL's `Detach()`: surrender the reference to the caller without releasing it.
    pub fn Detach(&mut self) -> *mut T {
        let p = self.p;
        self.p = core::ptr::null_mut();
        p
    }

    /// ATL's `Release()`: drop the reference and become null. Safe to call more than once.
    pub unsafe fn Release(&mut self) {
        if !self.p.is_null() {
            let u = self.p as *mut IUnknown;
            ((*(*u).lpVtbl).Release)(u);
            self.p = core::ptr::null_mut();
        }
    }

    pub fn is_null(&self) -> bool {
        self.p.is_null()
    }

    /// The raw interface pointer, ownership retained — ATL's `operator T*`.
    pub fn as_ptr(&self) -> *mut T {
        self.p
    }

    /// ATL's `operator T*` under its explicit name; the translation emits both spellings.
    pub fn Get(&self) -> *mut T {
        self.p
    }

    /// ATL's `operator&()` — `T**` for an out-parameter. ATL asserts the pointer is empty
    /// first (writing through it would leak the old reference); the assert is a debug-only
    /// check in C++ and is not reproduced as a panic in a shipping DLL.
    pub fn GetAddressOf(&mut self) -> *mut *mut T {
        core::ptr::addr_of_mut!(self.p)
    }
}

/// ATL's `operator->`. This yields the INTERFACE, not a method: the crate uses the C `lpVtbl`
/// shape, so a call still goes through the vtable — `((*(*p).lpVtbl).Method)(p, ..)`. A
/// translated `p.Method(..)` is therefore a call-site defect to rewrite, not a missing impl,
/// and giving `CComPtr` a `Deref` here is what makes that distinction a compile error rather
/// than a silent resolution to something else.
impl<T> core::ops::Deref for CComPtr<T> {
    type Target = *mut T;
    fn deref(&self) -> &*mut T {
        &self.p
    }
}

impl<T> Default for CComPtr<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for CComPtr<T> {
    fn drop(&mut self) {
        if !self.p.is_null() {
            unsafe {
                let u = self.p as *mut IUnknown;
                ((*(*u).lpVtbl).Release)(u);
            }
            self.p = core::ptr::null_mut();
        }
    }
}

// -------------------------------------------------------------------------------------------
// CSimpleArray
// -------------------------------------------------------------------------------------------

/// `CSimpleArray<T>` — ATL's growable array.
///
/// Measured surface: `new()`, `Add(t)`, `GetSize()`, `GetData()`. The crate is `no_std` with
/// no `alloc`, so the storage comes from the CRT allocator, which is also what ATL uses.
///
/// This is NOT zeroable-and-then-used: unlike the pointer types it is only ever built by
/// `new()`. It still holds a raw pointer rather than a `Vec` so that a stray
/// `core::mem::zeroed()` would be merely empty rather than UB.
pub struct CSimpleArray<T> {
    m_aT: *mut T,
    m_nSize: i32,
    m_nAllocSize: i32,
}

impl<T> CSimpleArray<T> {
    pub const fn new() -> Self {
        Self { m_aT: core::ptr::null_mut(), m_nSize: 0, m_nAllocSize: 0 }
    }

    /// ATL's `Add`: append, growing geometrically. Returns FALSE if the allocation failed —
    /// ATL's contract, and the call sites check it.
    pub unsafe fn Add(&mut self, t: T) -> BOOL {
        if self.m_nSize == self.m_nAllocSize {
            // ATL grows by max(alloc/2, 1); the first growth lands on 1.
            let grow = if self.m_nAllocSize / 2 > 1 { self.m_nAllocSize / 2 } else { 1 };
            let n = self.m_nAllocSize + grow;
            let bytes = (n as usize).wrapping_mul(core::mem::size_of::<T>());
            let p = CCRTAllocator::Reallocate(self.m_aT as *mut c_void, bytes) as *mut T;
            if p.is_null() {
                return 0;
            }
            self.m_aT = p;
            self.m_nAllocSize = n;
        }
        core::ptr::write(self.m_aT.add(self.m_nSize as usize), t);
        self.m_nSize += 1;
        1
    }

    pub fn GetSize(&self) -> i32 {
        self.m_nSize
    }

    pub fn GetData(&self) -> *mut T {
        self.m_aT
    }

    pub fn RemoveAll(&mut self) {
        unsafe {
            for i in 0..self.m_nSize {
                core::ptr::drop_in_place(self.m_aT.add(i as usize));
            }
            if !self.m_aT.is_null() {
                CCRTAllocator::Free(self.m_aT as *mut c_void);
            }
        }
        self.m_aT = core::ptr::null_mut();
        self.m_nSize = 0;
        self.m_nAllocSize = 0;
    }
}

impl<T> Default for CSimpleArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for CSimpleArray<T> {
    fn drop(&mut self) {
        self.RemoveAll();
    }
}

// -------------------------------------------------------------------------------------------
// Helpers shared with the string type (next wave)
// -------------------------------------------------------------------------------------------

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
