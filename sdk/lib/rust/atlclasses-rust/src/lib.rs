//! ATL's smart pointers, containers and `CStringW`, shared by every translated C++ module.
//!
//! # Why this crate exists
//!
//! shell32 grew the first copy of this surface (`shell32-rust/src/atl.rs`), measured against
//! its own call sites. The C++ apps cluster (mspaint, rapps, charmap_new, fltmc, msconfig_new,
//! cleanmgr, mmc_new, ...) uses the same ATL types, and a per-module copy of `CStringW` is the
//! drift `win32macros-rust` was written to end: N copies, each its own chance to get one wrong.
//! The bodies here are shell32's, with their provenance notes kept; shell32 still carries its
//! own copy and moving it onto this crate is owed.
//!
//! # The type basis is core primitives, deliberately
//!
//! A shared crate cannot depend on any consumer's bindgen island, and must not: `HWND`,
//! `HKEY`, `POSITION` and `HINSTANCE` are NOMINAL pointer types there (`*mut HKEY__`), one per
//! island, and `SIZE_T`/`LONG_PTR` are `c_ulong` on i386 but `c_ulonglong` on 64-bit, never
//! `usize`. So everything here is spelled in types identical in every island: `u16` for
//! `WCHAR`, `i32` for `INT`/`BOOL`, `u32` for `UINT`, raw pointers to those, and `c_void` for
//! handles this crate only ever passes back to its OWN imports. A consumer's `WCHAR` *is*
//! `u16` (a transparent alias), so `CStringW::as_ptr()` feeds an island `LPCWSTR` unchanged.
//!
//! `POSITION` is the one island name this crate DEFINES rather than borrows: a C++ crate's
//! prelude re-exports `atl_rust::POSITION`, which shadows the island's glob, so a `CAtlList`
//! position has one type. `CRegKey` (`HKEY`) is still island-bound and not here: it arrives as
//! an instantiate-in-place macro when a module first needs it.
//!
//! # Resource strings
//!
//! ATL loads `CString` resources from `_AtlBaseModule`'s resource instance, which for an EXE is
//! the process image. [`resource_instance`] answers the same: whatever a DLL consumer set with
//! [`set_resource_instance`] (its own `hInstance`, as shell32 needs), else
//! `GetModuleHandleW(NULL)`.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, clippy::missing_safety_doc)]

pub mod win;

use core::ffi::c_void;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicPtr, Ordering};

pub type WCHAR = u16;
pub type BOOL = i32;
pub type INT = i32;
pub type UINT = u32;
pub type LPCSTR = *const i8;
pub type LPCWSTR = *const u16;
type HLOCAL = *mut c_void;
type SIZE_T = usize;

/// winnls.h `CP_THREAD_ACP`.
const CP_THREAD_ACP: u32 = 3;

extern "system" {
    fn LocalAlloc(uFlags: u32, uBytes: SIZE_T) -> HLOCAL;
    fn LocalReAlloc(hMem: HLOCAL, uBytes: SIZE_T, uFlags: u32) -> HLOCAL;
    fn LocalFree(hMem: HLOCAL) -> HLOCAL;
    fn MultiByteToWideChar(
        CodePage: u32,
        dwFlags: u32,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: *mut WCHAR,
        cchWideChar: INT,
    ) -> INT;
    fn LoadStringW(hInstance: *mut c_void, uID: UINT, lpBuffer: *mut WCHAR, cchBufferMax: INT)
        -> INT;
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> *mut c_void;
    fn WideCharToMultiByte(
        CodePage: u32,
        dwFlags: u32,
        lpWideCharStr: *const WCHAR,
        cchWideChar: INT,
        lpMultiByteStr: *mut u8,
        cbMultiByte: INT,
        lpDefaultChar: *const u8,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
}

// ole32's COM task allocator, only for a module that enables the `com` feature: imported
// unconditionally it made fltmc (no COM at all) an undefined-symbol link on amd64/arm64.
#[cfg(feature = "com")]
extern "system" {
    fn CoTaskMemAlloc(cb: SIZE_T) -> *mut c_void;
    fn CoTaskMemRealloc(pv: *mut c_void, cb: SIZE_T) -> *mut c_void;
    fn CoTaskMemFree(pv: *mut c_void);
}

extern "C" {
    /// wchar.h: under `__cplusplus` (every TU this crate serves) `swprintf` is the
    /// NON-CONFORMING two-fixed-argument form, with no `count`. The C99 spelling would shift
    /// every variadic argument by one slot.
    #[doc(hidden)]
    pub fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
}

static RESOURCE_INSTANCE: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

/// `_AtlBaseModule.SetResourceInstance` — a DLL consumer passes its own `hInstance` (from
/// `DllMain`), so `CStringW::LoadStringW` reads the DLL's string table, not the host EXE's.
pub fn set_resource_instance(h: *mut c_void) {
    RESOURCE_INSTANCE.store(h, Ordering::Relaxed);
}

/// `_AtlBaseModule.GetResourceInstance()`: the instance set above, else the process image.
pub fn resource_instance() -> *mut c_void {
    let h = RESOURCE_INSTANCE.load(Ordering::Relaxed);
    if h.is_null() {
        unsafe { GetModuleHandleW(core::ptr::null()) }
    } else {
        h
    }
}

/// `IUnknown::Release` through the vtable. Every COM interface begins with a vtable pointer
/// whose slot 2 is `Release`, so this is ABI-sound for any interface pointer and is exactly
/// ATL's `CComPtrBase::Release`. It is NOT sound for a non-COM pointee.
unsafe fn com_release(p: *mut c_void) {
    type Release = unsafe extern "system" fn(*mut c_void) -> u32;
    let vtbl = *(p as *const *const Release);
    (*vtbl.add(2))(p);
}

/// `CStringT::Format(pszFormat, ...)` for any arity: `atl_format!(s, fmt, a, b, ..)`.
///
/// Each argument is evaluated exactly ONCE, into a local, before the sizing call
/// (`_scwprintf`) and the printing call (`swprintf`) both read it. The shell32 copy used
/// fixed-arity `FormatStr2`/`FormatStr4` methods instead of a macro for exactly that reason,
/// and they are kept; this is the same guarantee for an arity they do not cover. Under
/// `__cplusplus` `swprintf` takes no count, so the buffer is sized first.
#[macro_export]
macro_rules! atl_format {
    ($s:expr, $fmt:expr $(, $a:expr)* $(,)?) => {
        $crate::atl_format!(@bind $s, $fmt, [],
            [__a0 __a1 __a2 __a3 __a4 __a5 __a6 __a7 __a8 __a9 __a10 __a11 __a12 __a13 __a14
             __a15] $(, $a)*)
    };
    (@bind $s:expr, $fmt:expr, [$($done:ident)*], [$n:ident $($names:ident)*], $a:expr
        $(, $rest:expr)*) => {{
        let $n = $a;
        $crate::atl_format!(@bind $s, $fmt, [$($done)* $n], [$($names)*] $(, $rest)*)
    }};
    (@bind $s:expr, $fmt:expr, [$($done:ident)*], [$($names:ident)*]) => {{
        let __fmt: *const u16 = $fmt;
        let __s: &mut $crate::CStringW = &mut $s;
        let __need = $crate::_scwprintf(__fmt $(, $done)*);
        let __buf = __s.format_begin_pub(__need);
        if !__buf.is_null() {
            $crate::swprintf(__buf, __fmt $(, $done)*);
            __s.ReleaseBuffer();
        }
    }};
}

// -------------------------------------------------------------------------------------------
// Everything below is shell32-rust/src/atl.rs, with the island imports replaced by the
// primitive basis above.
// -------------------------------------------------------------------------------------------

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

/// `CComAllocator` — CoTaskMemAlloc/Realloc/Free, the COM task allocator (feature `com`).
///
/// This is the one that matters most here: shell32's PIDLs are `SHAlloc`ed, and `SHAlloc` IS
/// the COM task allocator, so `CComHeapPtr<ITEMIDLIST>` frees a PIDL correctly.
#[cfg(feature = "com")]
pub struct CComAllocator;

#[cfg(feature = "com")]
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
#[cfg(feature = "com")]
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
            com_release(self.p as *mut c_void);
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
            com_release(self.p as *mut c_void);
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

    /// `GetAddressOf` under the spelling the translation also emits, for the same reason
    /// `Get` sits beside `as_ptr` above: one ATL operator, two renderings.
    pub fn as_mut_ptr(&mut self) -> *mut *mut T {
        self.GetAddressOf()
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
                com_release(self.p as *mut c_void);
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

    /// ATL's `RemoveAt`: drop the element, then shift the tail down. The `drop_in_place`
    /// matters for an owning element (`CSimpleMap<CStringW, CStringW>`); ATL runs the
    /// destructor here too, and `copy` alone would leak the removed value's buffer.
    pub unsafe fn RemoveAt(&mut self, nIndex: i32) -> BOOL {
        if nIndex < 0 || nIndex >= self.m_nSize {
            return 0;
        }
        core::ptr::drop_in_place(self.m_aT.add(nIndex as usize));
        let remaining = (self.m_nSize - nIndex - 1) as usize;
        if remaining > 0 {
            core::ptr::copy(
                self.m_aT.add(nIndex as usize + 1),
                self.m_aT.add(nIndex as usize),
                remaining,
            );
        }
        self.m_nSize -= 1;
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

/// ATL's `operator[]`, as a POINTER rather than a reference.
///
/// The C++ call sites take the element's ADDRESS (`ReleaseStgMedium(&m_Storage[n])`), so a
/// raw pointer is what the translation actually wants; `&arr[n]` becomes `arr.GetAt(n)` and
/// `arr[n]` becomes `(*arr.GetAt(n))`. Out of range returns null — ATL asserts, which is a
/// no-op in a release build, and a null deref faults at the site instead of silently
/// reading whatever follows the allocation.
impl<T> CSimpleArray<T> {
    pub unsafe fn GetAt(&self, nIndex: i32) -> *mut T {
        if nIndex < 0 || nIndex >= self.m_nSize {
            return core::ptr::null_mut();
        }
        self.m_aT.add(nIndex as usize)
    }
}

// -------------------------------------------------------------------------------------------
// CSimpleMap
// -------------------------------------------------------------------------------------------

/// `CSimpleMap<K, V>` — ATL's association list (`atlsimpcoll.h`), a LINEAR array of pairs.
///
/// Not a hash map: `FindKey` scans, so every lookup is O(n) and insertion order is preserved.
/// Callers that sort through it (CDefView's `SelectionMoveCompareFunc` runs one `Lookup` per
/// comparison) inherit that cost, which is ATL's behaviour and not something to "fix" here.
///
/// Measured surface: `new()`, `Add(k, v)`, `Lookup(k)`, `FindKey(k)`, `GetSize()`.
///
/// `Lookup` returns `V::default()` for an absent key — ATL returns a value-initialized
/// `TVal()`, which for the integer instantiations the tree uses is 0. That is
/// indistinguishable from a stored zero BY DESIGN in ATL; check `FindKey` first when the
/// difference matters.
pub struct CSimpleMap<K, V> {
    m_Pairs: CSimpleArray<CSimpleMapPair<K, V>>,
}

/// One key/value cell. ATL keeps these in a `CSimpleArray<Pair>`; so do we, so that growth,
/// allocator, and failure behaviour are the array's and stay in one place.
pub struct CSimpleMapPair<K, V> {
    pub key: K,
    pub val: V,
}

impl<K, V> CSimpleMap<K, V> {
    pub const fn new() -> Self {
        Self { m_Pairs: CSimpleArray::new() }
    }

    pub fn GetSize(&self) -> i32 {
        self.m_Pairs.GetSize()
    }

    /// ATL's `Add`: append. Returns FALSE when the array's allocation failed.
    pub unsafe fn Add(&mut self, key: K, val: V) -> BOOL {
        self.m_Pairs.Add(CSimpleMapPair { key, val })
    }

    pub fn RemoveAll(&mut self) {
        self.m_Pairs.RemoveAll();
    }
}

impl<K: PartialEq, V> CSimpleMap<K, V> {
    /// Index of the first pair with this key, or -1. ATL's `CSimpleMapEqualHelper` compares
    /// with `operator==`, which for the tree's integer keys is ordinary equality.
    pub unsafe fn FindKey(&self, key: &K) -> i32 {
        let count = self.m_Pairs.GetSize();
        for index in 0..count {
            let pair = self.m_Pairs.GetAt(index);
            if !pair.is_null() && (*pair).key == *key {
                return index;
            }
        }
        -1
    }

    /// Pointer to the stored value for this key, or null when absent. This is the form that
    /// keeps the "missing" case distinguishable; `Lookup` is the ATL-faithful one.
    pub unsafe fn FindValue(&self, key: &K) -> *mut V {
        let index = self.FindKey(key);
        if index < 0 {
            return core::ptr::null_mut();
        }
        let pair = self.m_Pairs.GetAt(index);
        if pair.is_null() {
            return core::ptr::null_mut();
        }
        core::ptr::addr_of_mut!((*pair).val)
    }
}

impl<K: PartialEq, V: Copy + Default> CSimpleMap<K, V> {
    /// ATL's `Lookup`: the stored value, or a value-initialized one when the key is absent.
    pub unsafe fn Lookup(&self, key: K) -> V {
        let val = self.FindValue(&key);
        if val.is_null() {
            return V::default();
        }
        *val
    }

    /// ATL's `SetAt`: overwrite in place when present, append otherwise.
    pub unsafe fn SetAt(&mut self, key: K, val: V) -> BOOL {
        let existing = self.FindValue(&key);
        if existing.is_null() {
            return self.Add(key, val);
        }
        *existing = val;
        1
    }
}

/// The OWNING-value half of CSimpleMap. `Lookup`/`SetAt` above require `V: Copy`, which the
/// integer instantiations satisfy; `CSimpleMap<CStringW, CStringW>` (filetypes.cpp's verb →
/// command-line map) does not, because CStringW owns a buffer and has a Drop. Splitting the
/// impl rather than relaxing the original keeps the copy path allocation-free.
impl<K: PartialEq, V: Clone + Default> CSimpleMap<K, V> {
    /// `SetAt` for an owning value: overwrite in place when present, append otherwise. The
    /// overwrite DROPS the old value, which is what `operator=` does in ATL.
    pub unsafe fn SetAtOwned(&mut self, key: K, val: V) -> BOOL {
        let existing = self.FindValue(&key);
        if existing.is_null() {
            return self.Add(key, val);
        }
        *existing = val;
        1
    }

    /// `GetValueAt(index)` — ATL returns a value-initialized V when out of range.
    pub unsafe fn GetValueAt(&self, index: i32) -> V {
        let pair = self.m_Pairs.GetAt(index);
        if pair.is_null() { V::default() } else { (*pair).val.clone() }
    }
}

impl<K: Clone + Default + PartialEq, V> CSimpleMap<K, V> {
    /// `GetKeyAt(index)` — ATL returns a value-initialized K when out of range.
    pub unsafe fn GetKeyAt(&self, index: i32) -> K {
        let pair = self.m_Pairs.GetAt(index);
        if pair.is_null() { K::default() } else { (*pair).key.clone() }
    }

    /// `Remove(key)` — ATL removes the first pair with this key, returning FALSE when absent.
    pub unsafe fn Remove(&mut self, key: &K) -> BOOL {
        let index = self.FindKey(key);
        if index < 0 { return 0; }
        self.m_Pairs.RemoveAt(index)
    }
}

impl<K, V> Default for CSimpleMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}


// -------------------------------------------------------------------------------------------

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

/// atlcoll.h `typedef __POSITION* POSITION`: a list node's address, null past the end. Defined
/// HERE, not taken from a consumer's island (where it is a nominal `*mut __POSITION`); the
/// scaffold re-exports this one from each C++ crate's prelude, shadowing the island's glob, so
/// `let pos: POSITION = list.GetHeadPosition()` names one type everywhere.
pub type POSITION = *mut c_void;

// -------------------------------------------------------------------------------------------
// CAtlList
// -------------------------------------------------------------------------------------------

/// `CAtlList<T>` — ATL's doubly-linked list (atlcoll.h).
///
/// Measured surface, from every `CAtlList` call site in `dll/win32/shell32/`:
/// `GetHeadPosition` / `GetNext` (7 each), `IsEmpty` (5), `RemoveAll` (2), `GetAt` (2),
/// `FindIndex` (2), `AddTail` (2), `AddHead` (1). ATL's own class is far larger; the rest
/// is deliberately absent so an unported call site is a compile error rather than a guess.
///
/// A `POSITION` is a node address, exactly as in ATL, and is null past the end of the list —
/// which is what makes the C++ `while (it != NULL)` loop shape translate literally.
///
/// **Element destructors run.** `RemoveAll` drops each element in place, so a `T` holding a
/// `CComPtr` or a `CStringW` releases correctly. This is the whole reason a container field
/// keeps its type in the Rust owner rather than being flattened to a pointer.
///
/// **OOM returns a null POSITION instead of throwing.** ATL raises `CAtlException` here;
/// this crate is `no_std` with no unwinding, and the shell32 call sites discard the returned
/// POSITION anyway, so there is nothing to propagate to. The difference is real and worth
/// stating: under genuine allocation failure the entry is dropped rather than the operation
/// being abandoned. Check the return where that matters.
#[repr(C)]
struct CAtlListNode<T> {
    next: *mut CAtlListNode<T>,
    prev: *mut CAtlListNode<T>,
    element: T,
}

pub struct CAtlList<T> {
    head: *mut CAtlListNode<T>,
    tail: *mut CAtlListNode<T>,
    count: usize,
}

impl<T> CAtlList<T> {
    pub const fn new() -> Self {
        Self { head: core::ptr::null_mut(), tail: core::ptr::null_mut(), count: 0 }
    }

    unsafe fn alloc_node(element: T) -> *mut CAtlListNode<T> {
        let node =
            CCRTAllocator::Allocate(core::mem::size_of::<CAtlListNode<T>>()) as *mut CAtlListNode<T>;
        if node.is_null() {
            return node;
        }
        core::ptr::write(
            node,
            CAtlListNode { next: core::ptr::null_mut(), prev: core::ptr::null_mut(), element },
        );
        node
    }

    pub unsafe fn AddHead(&mut self, element: T) -> POSITION {
        let node = Self::alloc_node(element);
        if node.is_null() {
            return core::ptr::null_mut();
        }
        (*node).next = self.head;
        if self.head.is_null() {
            self.tail = node;
        } else {
            (*self.head).prev = node;
        }
        self.head = node;
        self.count += 1;
        node.cast()
    }

    pub unsafe fn AddTail(&mut self, element: T) -> POSITION {
        let node = Self::alloc_node(element);
        if node.is_null() {
            return core::ptr::null_mut();
        }
        (*node).prev = self.tail;
        if self.tail.is_null() {
            self.head = node;
        } else {
            (*self.tail).next = node;
        }
        self.tail = node;
        self.count += 1;
        node.cast()
    }

    pub fn IsEmpty(&self) -> bool {
        self.head.is_null()
    }

    pub fn GetCount(&self) -> usize {
        self.count
    }

    pub fn GetHeadPosition(&self) -> POSITION {
        self.head.cast()
    }

    /// ATL's `GetNext(POSITION&)`: returns the element AT the position, then advances the
    /// position to the following node. The order matters — a translation that advances
    /// first would skip the head and read one past the tail.
    pub unsafe fn GetNext(&self, pos: &mut POSITION) -> *mut T {
        let node = (*pos) as *mut CAtlListNode<T>;
        if node.is_null() {
            return core::ptr::null_mut();
        }
        *pos = (*node).next.cast();
        core::ptr::addr_of_mut!((*node).element)
    }

    pub unsafe fn GetAt(&self, pos: POSITION) -> *mut T {
        let node = pos as *mut CAtlListNode<T>;
        if node.is_null() {
            return core::ptr::null_mut();
        }
        core::ptr::addr_of_mut!((*node).element)
    }

    /// `Find(const E& element, POSITION posStartAfter = NULL)` — atlcoll.h. The C++ tests
    /// the returned `POSITION` for truth (`if (!list.Find(key))`), so this returns `bool`:
    /// handing back a raw node pointer would invite it to be dereferenced, and no caller
    /// in the tree wants the position itself.
    pub unsafe fn Find(&self, element: &T) -> bool
    where
        T: PartialEq,
    {
        let mut node = self.head;
        while !node.is_null() {
            if (*node).element == *element {
                return true;
            }
            node = (*node).next;
        }
        false
    }

    /// Null when the index is out of range, matching the `POSITION` the C++ then tests.
    pub unsafe fn FindIndex(&self, iElement: i32) -> POSITION {
        if iElement < 0 {
            return core::ptr::null_mut();
        }
        let mut node = self.head;
        let mut i = 0i32;
        while !node.is_null() {
            if i == iElement {
                return node.cast();
            }
            node = (*node).next;
            i += 1;
        }
        core::ptr::null_mut()
    }

    pub unsafe fn RemoveAll(&mut self) {
        let mut node = self.head;
        while !node.is_null() {
            let next = (*node).next;
            core::ptr::drop_in_place(core::ptr::addr_of_mut!((*node).element));
            CCRTAllocator::Free(node as *mut c_void);
            node = next;
        }
        self.head = core::ptr::null_mut();
        self.tail = core::ptr::null_mut();
        self.count = 0;
    }
}

impl<T> Default for CAtlList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for CAtlList<T> {
    fn drop(&mut self) {
        unsafe { self.RemoveAll() }
    }
}

// -------------------------------------------------------------------------------------------
// Helpers shared with the string type (next wave)
// -------------------------------------------------------------------------------------------

// -------------------------------------------------------------------------------------------
// CStringW — `CStringT<WCHAR, StrTraitATL<WCHAR, ChTraitsCRT<WCHAR>>>` (atlstr.h:130)
// -------------------------------------------------------------------------------------------
//
// SCOPE, and why this is not ATL's string. Every `CStringW` in the translated shell32 TUs is a
// FUNCTION LOCAL of a `static` C++ function — `shlfileop.cpp`'s `try_find_new_name` returns one
// by value, and `shlexec.cpp`'s `PathProcessCommandW` builds one — so no instance ever crosses
// back into C++, and nothing depends on ATL's layout: no `CStringData` header, no refcount, no
// shared-empty-string singleton, no `IAtlStringMgr`. Reproducing those would be work with no
// observable effect. What IS observable is the operation semantics, and those are ported one at
// a time from `sdk/lib/atl/cstringt.h` with the line noted.
//
// Storage is `CLocalAllocator` (LocalAlloc/LocalReAlloc/LocalFree) rather than the CRT malloc
// ATL's `CWin32Heap` uses, because this crate is `#![no_std]` with no allocator and LocalAlloc
// is already the module's own allocation policy for owned buffers.
//
// INVARIANT: `m_pszData` is either null (empty) or a NUL-terminated buffer of `m_nAlloc`
// WCHARs holding `m_nLength` characters. `GetString`/`as_ptr` never returns null — ATL's
// `PCXSTR` conversion yields `L""` for an empty string, and call sites pass it straight to
// `PathFileExistsW`/`StringCchCopyW`, which would fault on null.
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
    ///
    /// THE INTRESOURCE BRANCH IS NOT OPTIONAL. cstringt.h:470 opens with
    /// `if (IS_INTRESOURCE(pszSrc)) LoadString(LOWORD(pszSrc));`, so this constructor is
    /// ALSO how ATL loads a string resource — `CStringW str(MAKEINTRESOURCEW(IDS_X))` is
    /// idiomatic and filetypes.cpp alone does it a dozen times. Without the branch that
    /// argument is a small integer dereferenced as a pointer: not a compile error, and not a
    /// clean crash either, since a low address may well be mapped.
    pub unsafe fn from_ptr(pszSrc: *const WCHAR) -> Self {
        let mut s = Self::new();
        // IS_INTRESOURCE(p): the whole pointer fits in its low word.
        if (pszSrc as usize) >> 16 == 0 {
            s.LoadStringW((pszSrc as usize & 0xFFFF) as UINT);
        } else {
            s.SetString(pszSrc, wide_len(pszSrc) as i32);
        }
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

    /// `GetLength()` — in CHARACTERS, not bytes.
    pub fn GetLength(&self) -> i32 {
        self.m_nLength
    }

    /// `IsEmpty()`.
    pub fn IsEmpty(&self) -> bool {
        self.m_nLength == 0
    }

    /// `CompareNoCase(psz)` — cstringt.h:743, `StringTraits::CompareNoCase(GetString(), psz)`,
    /// which for the W traits is `_wcsicmp`. Returns the sign convention of the C call, so
    /// translated `== 0` tests keep meaning "equal" rather than "not equal".
    pub unsafe fn CompareNoCase(&self, psz: *const WCHAR) -> INT {
        _wcsicmp(self.as_ptr(), psz)
    }

    /// Grow the buffer to hold at least `nChars` characters plus the terminator. Never
    /// shrinks: `GetBuffer`/`ReleaseBuffer` hand out a raw pointer that must stay valid.
    unsafe fn Reserve(&mut self, nChars: i32) {
        if nChars < self.m_nAlloc {
            return;
        }
        let need = nChars + 1;
        let p = <CLocalAllocator as CAllocator>::Reallocate(
            self.m_pszData as *mut c_void,
            need as usize * core::mem::size_of::<WCHAR>(),
        ) as *mut WCHAR;
        if p.is_null() {
            return;
        }
        self.m_pszData = p;
        self.m_nAlloc = need;
        *self.m_pszData.add(self.m_nLength as usize) = 0;
    }

    /// `SetString(PCXSTR pszSrc, int nLength)` — cstringt.h. Copies `nLength` characters and
    /// terminates; the source need not be NUL-terminated at that point (`shlexec` calls this
    /// with a length measured to an embedded quote).
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

    /// `CStringT(const char *pszSrc)` — cstringt.h's CONVERTING constructor, the one behind
    /// `CStringW strPathW(pszPath)` where pszPath is an LPCSTR.
    ///
    /// This is a real code-page conversion, not a cast. The page is `_AtlGetConversionACP()`
    /// (cstringt.h:14), which is CP_THREAD_ACP unless `_CONVERSION_DONT_USE_THREAD_LOCALE` is
    /// defined — and nothing in this build defines it. Using CP_ACP instead would agree with
    /// CP_THREAD_ACP on ASCII and silently produce different characters for everything else,
    /// which is the kind of difference that shows up as a corrupted path much later.
    ///
    /// ATL sizes with `MultiByteToWideChar(acp, 0, src, -1, NULL, 0) - 1` (the `-1` drops the
    /// terminator from the COUNT, which SetString then re-adds), so the same shape is kept.
    pub unsafe fn from_ansi(pszSrc: LPCSTR) -> Self {
        let mut s = Self::new();
        if pszSrc.is_null() {
            return s;
        }
        let n = MultiByteToWideChar(CP_THREAD_ACP, 0, pszSrc, -1, core::ptr::null_mut(), 0) - 1;
        if n <= 0 {
            return s;
        }
        s.Reserve(n);
        if s.m_pszData.is_null() {
            return s;
        }
        MultiByteToWideChar(CP_THREAD_ACP, 0, pszSrc, -1, s.m_pszData, n + 1);
        *s.m_pszData.add(n as usize) = 0;
        s.m_nLength = n;
        s
    }

    /// `Empty()` — atlsimpstr.h:253.
    ///
    /// ATL takes one of two branches: SetLength(0) when the buffer is LOCKED, or release the
    /// CStringData and attach the manager's nil string. This port has no refcount, no lock and
    /// no string manager, and its `Reserve` never shrinks by design (GetBuffer hands out a raw
    /// pointer that must stay valid) -- so the locked branch is the one consistent with the
    /// model here: length to zero, terminator written, allocation kept.
    pub unsafe fn Empty(&mut self) {
        self.m_nLength = 0;
        if !self.m_pszData.is_null() {
            *self.m_pszData = 0;
        }
    }

    /// `GetBuffer(int nMinBufferLength)` — cstringt.h. Returns a writable buffer of at least
    /// that many characters. Must be paired with `ReleaseBuffer`.
    pub unsafe fn GetBuffer(&mut self, nMinBufferLength: i32) -> *mut WCHAR {
        self.Reserve(nMinBufferLength);
        self.m_pszData
    }

    /// `ReleaseBuffer(int nNewLength = -1)` — cstringt.h. -1 means "measure to the NUL",
    /// which is what every shell32 call site relies on after writing through `GetBuffer`.
    pub unsafe fn ReleaseBuffer(&mut self) {
        if self.m_pszData.is_null() {
            self.m_nLength = 0;
            return;
        }
        self.m_nLength = wide_len(self.m_pszData) as i32;
    }

    /// `Left(int nCount)` — cstringt.h:776. ATL clamps: negative yields empty, over-long
    /// yields the whole string.
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

    /// `operator+=(XCHAR ch)` — cstringt.h. Appends the character **even when it is NUL**,
    /// growing the length by one; `shlexec`'s `PathProcessCommandW` relies on exactly that
    /// (its C++ carries a "Remove excessive '\0'" comment at the matching ReleaseBuffer).
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

    /// `operator[](int iChar)` — cstringt.h. ATL indexes the buffer directly and asserts
    /// `iChar <= GetLength()`; `PathProcessCommandW` reads at an index that can sit past the
    /// current length but inside a `GetBuffer(MAX_PATH)` allocation, so read the allocation
    /// and yield NUL beyond it rather than running off the end.
    pub unsafe fn GetAt(&self, iChar: i32) -> WCHAR {
        if self.m_pszData.is_null() || iChar < 0 || iChar >= self.m_nAlloc {
            return 0;
        }
        *self.m_pszData.add(iChar as usize)
    }

    /// `operator+=(PCXSTR)` — cstringt.h. Appending to self is not a case the call sites
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

    /// `CStringT::LoadString(UINT nID)` — cstringt.h. Loads from **shell32's own** resources,
    /// not the process image: `GetModuleHandleW(NULL)` would read explorer's string table.
    /// ATL returns FALSE and leaves the string untouched when the ID is absent.
    pub unsafe fn LoadStringW(&mut self, nID: UINT) -> BOOL {
        let mut stack: [WCHAR; 512] = [0; 512];
        let n = LoadStringW(
            resource_instance(),
            nID,
            stack.as_mut_ptr(),
            stack.len() as INT,
        );
        if n <= 0 {
            return 0;
        }
        self.SetString(stack.as_ptr(), n);
        1
    }

    /// `CStringT(UINT nID)`-shaped helper for the `Format(UINT nFormatID, ...)` overload,
    /// which loads the format string from resources before formatting.
    pub unsafe fn from_resource(nID: UINT) -> Self {
        let mut s = Self::new();
        s.LoadStringW(nID);
        s
    }

    /// Front half of every `Format`: take the width `_scwprintf` computed and hand back a
    /// buffer sized to hold exactly that plus the terminator, or null if there is nothing to
    /// print. ATL grows dynamically, so a fixed cap would silently truncate.
    ///
    /// Split in two around the caller's `swprintf` rather than taking a closure: a closure
    /// body is its own body, so whether it inherits an `unsafe fn`'s implicit unsafe block is
    /// a rule not worth depending on. The pair also keeps the sizing call and the printing
    /// call reading the SAME already-evaluated locals, which is the point of these being
    /// concrete arities instead of one variadic `macro_rules!` — a macro would re-expand each
    /// argument expression at both sites and evaluate the caller's side effects twice.
    #[doc(hidden)]
    pub unsafe fn format_begin_pub(&mut self, need: INT) -> *mut WCHAR {
        self.format_begin(need)
    }

    unsafe fn format_begin(&mut self, need: INT) -> *mut WCHAR {
        if need < 0 {
            self.SetString(EMPTY_WSTR.as_ptr(), 0);
            return core::ptr::null_mut();
        }
        self.GetBuffer(need)
    }

    /// `Replace(XCHAR chOld, XCHAR chNew)` -- cstringt.h:925, line for line. FindChar is
    /// `wcschr`: it stops at the terminator, so a `|` -> NUL pass (mspaint's file filter
    /// strings) works left to right and the length stays nLength, embedded NULs included.
    pub unsafe fn Replace(&mut self, chOld: WCHAR, chNew: WCHAR) -> i32 {
        unsafe fn find_char(mut p: *mut WCHAR, ch: WCHAR) -> *mut WCHAR {
            loop {
                if *p == ch {
                    return p;
                }
                if *p == 0 {
                    return core::ptr::null_mut();
                }
                p = p.add(1);
            }
        }
        if self.m_pszData.is_null() {
            return 0; // GetString() is L"": nothing to find
        }
        let pszString = self.m_pszData;
        let mut pszFirst = find_char(pszString, chOld);
        if pszFirst.is_null() {
            return 0;
        }
        let nLength = self.m_nLength;
        let mut nCount = 0;
        let pszBuffer = self.GetBuffer(nLength);
        pszFirst = pszBuffer.offset(pszFirst.offset_from(pszString));
        loop {
            *pszFirst = chNew;
            nCount += 1;
            pszFirst = find_char(pszFirst.add(1), chOld);
            if pszFirst.is_null() {
                break;
            }
        }
        // ReleaseBufferSetLength(nLength)
        self.m_nLength = nLength;
        *self.m_pszData.add(nLength as usize) = 0;
        nCount
    }

    /// `FormatV(PCXSTR pszFormat, va_list args)` -- cstringt.h:846: the length from
    /// `_vscwprintf`, then `_vswprintf` into `GetBuffer(nLength)`, then
    /// `ReleaseBufferSetLength(nLength)`. `args` is the caller's `va_list` VALUE (a single
    /// pointer on all three Windows targets), as C passes it.
    pub unsafe fn FormatV(&mut self, pszFormat: *const WCHAR, args: *mut c_void) {
        let n_length = _vscwprintf(pszFormat, args);
        let buf = self.GetBuffer(n_length);
        _vswprintf(buf, pszFormat, args);
        // ReleaseBufferSetLength(nLength): the length is the count, not a re-measure.
        self.m_nLength = n_length.max(0);
        if !buf.is_null() {
            *buf.add(self.m_nLength as usize) = 0;
        }
    }

    /// `Format(PCXSTR pszFormat, <one %s>)`.
    pub unsafe fn FormatStr(&mut self, pszFormat: *const WCHAR, a: *const WCHAR) {
        let need = _scwprintf(pszFormat, a);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        swprintf(buf, pszFormat, a);
        self.ReleaseBuffer();
    }

    /// `Format(PCXSTR pszFormat, <two %s>)`.
    pub unsafe fn FormatStr2(
        &mut self,
        pszFormat: *const WCHAR,
        a: *const WCHAR,
        b: *const WCHAR,
    ) {
        let need = _scwprintf(pszFormat, a, b);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        swprintf(buf, pszFormat, a, b);
        self.ReleaseBuffer();
    }

    /// `Format(UINT nFormatID, <one %s>)` — the resource-ID overload, which loads the format
    /// string from shell32's string table first. A missing ID formats an empty pattern, which
    /// is what ATL does too (`Format` on an empty string yields an empty string).
    pub unsafe fn FormatResStr(&mut self, nFormatID: UINT, a: *const WCHAR) {
        let f = Self::from_resource(nFormatID);
        self.FormatStr(f.as_ptr(), a);
    }

    /// `Format(UINT nFormatID, <two %s>)`.
    pub unsafe fn FormatResStr2(&mut self, nFormatID: UINT, a: *const WCHAR, b: *const WCHAR) {
        let f = Self::from_resource(nFormatID);
        self.FormatStr2(f.as_ptr(), a, b);
    }

    /// `Format(PCXSTR pszFormat, <four %s>)`.
    pub unsafe fn FormatStr4(
        &mut self,
        pszFormat: *const WCHAR,
        a: *const WCHAR,
        b: *const WCHAR,
        c: *const WCHAR,
        d: *const WCHAR,
    ) {
        let need = _scwprintf(pszFormat, a, b, c, d);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        swprintf(buf, pszFormat, a, b, c, d);
        self.ReleaseBuffer();
    }

    /// `Format(UINT nFormatID, <four %s>)` — IDS_NEWEXT_ALREADY_ASSOC names its two
    /// substitutions twice each, which is why a four-argument overload is needed at all.
    pub unsafe fn FormatResStr4(
        &mut self,
        nFormatID: UINT,
        a: *const WCHAR,
        b: *const WCHAR,
        c: *const WCHAR,
        d: *const WCHAR,
    ) {
        let f = Self::from_resource(nFormatID);
        self.FormatStr4(f.as_ptr(), a, b, c, d);
    }

    /// `Format(UINT nFormatID, <three %s>)`.
    pub unsafe fn FormatResStr3(
        &mut self,
        nFormatID: UINT,
        a: *const WCHAR,
        b: *const WCHAR,
        c: *const WCHAR,
    ) {
        let f = Self::from_resource(nFormatID);
        let need = _scwprintf(f.as_ptr(), a, b, c);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        swprintf(buf, f.as_ptr(), a, b, c);
        self.ReleaseBuffer();
    }

    /// `Format(PCXSTR pszFormat, <one %d>)`.
    pub unsafe fn FormatInt(&mut self, pszFormat: *const WCHAR, a: INT) {
        let need = _scwprintf(pszFormat, a);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        swprintf(buf, pszFormat, a);
        self.ReleaseBuffer();
    }
}

extern "C" {
    /// msvcrt `@ varargs _scwprintf(wstr)` — the character count a `swprintf` of the same
    /// arguments would produce, excluding the terminator. Negative on a bad format.
    #[doc(hidden)]
    pub fn _scwprintf(format: *const WCHAR, ...) -> INT;
    /// msvcrt `_vscwprintf(format, va_list)` / `_vswprintf(dest, format, va_list)`: ATL's
    /// wide StringTraits::FormatV pair.
    fn _vscwprintf(format: *const WCHAR, args: *mut c_void) -> INT;
    fn _vswprintf(dest: *mut WCHAR, format: *const WCHAR, args: *mut c_void) -> INT;
    /// msvcrt — case-insensitive wide compare, what ATL's W string traits use for
    /// `CompareNoCase`.
    fn _wcsicmp(left: *const WCHAR, right: *const WCHAR) -> INT;
}

impl PartialEq for CStringW {
    /// ATL compares CString by VALUE (`operator==` → `Compare() == 0`), not by buffer
    /// identity — which is what makes it usable as a CSimpleMap key.
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let (a, b) = (self.as_ptr(), other.as_ptr());
            let mut i = 0usize;
            loop {
                let (x, y) = (*a.add(i), *b.add(i));
                if x != y { return false; }
                if x == 0 { return true; }
                i += 1;
            }
        }
    }
}

impl CStringW {
    /// `MakeLower()` — cstringt.h, in place, returning the string.
    pub unsafe fn MakeLower(&mut self) {
        let buf = self.GetBuffer(-1);
        if !buf.is_null() {
            let mut i = 0usize;
            while *buf.add(i) != 0 {
                let c = *buf.add(i);
                if (b'A' as WCHAR..=b'Z' as WCHAR).contains(&c) {
                    *buf.add(i) = c + 32;
                }
                i += 1;
            }
        }
        self.ReleaseBuffer();
    }

    /// `Find(XCHAR ch, int nStart = 0)` — cstringt.h. Index of the first occurrence at or
    /// after `nStart`, or -1. Rust has no default arguments, so every caller spells the
    /// start; ATL returns -1 rather than clamping when `nStart` is past the end.
    pub unsafe fn Find(&self, ch: WCHAR, nStart: i32) -> i32 {
        if nStart < 0 || nStart > self.GetLength() {
            return -1;
        }
        let p = self.as_ptr();
        let mut i = nStart as usize;
        while *p.add(i) != 0 {
            if *p.add(i) == ch { return i as i32; }
            i += 1;
        }
        -1
    }
}

impl core::ops::AddAssign<*mut WCHAR> for CStringW {
    /// The same `operator+=`, for the `LPWSTR` returns the C++ passes to it unchanged.
    fn add_assign(&mut self, rhs: *mut WCHAR) {
        unsafe { self.Append(rhs) }
    }
}

impl core::ops::AddAssign<*const WCHAR> for CStringW {
    /// `CStringT::operator+=(PCXSTR)` — cstringt.h, i.e. `Append`.
    fn add_assign(&mut self, rhs: *const WCHAR) {
        unsafe { self.Append(rhs) }
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

/// `CStringA` (`CStringT<char, ..>`), as far as modules use it: built from a WIDE string
/// (cstringt.h's ChTraitsCRT<char>::ConvertToBaseType: WideCharToMultiByte with
/// `_AtlGetConversionACP()`, CP_THREAD_ACP) and read back as an `LPCSTR`.
pub struct CStringA {
    m_pszData: *mut u8,
    m_nLength: i32,
}

impl CStringA {
    pub fn new() -> Self {
        CStringA { m_pszData: core::ptr::null_mut(), m_nLength: 0 }
    }

    /// `CStringA(LPCWSTR)`. NULL is the empty string, as in ATL.
    pub unsafe fn from_wide(pszSrc: *const WCHAR) -> Self {
        let mut s = Self::new();
        if pszSrc.is_null() {
            return s;
        }
        // GetBaseTypeLength(LPCWSTR): the converted length, terminator excluded.
        let n = WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            pszSrc,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) - 1;
        if n < 0 {
            return s;
        }
        let p = <CLocalAllocator as CAllocator>::Reallocate(core::ptr::null_mut(), n as usize + 1)
            as *mut u8;
        if p.is_null() {
            return s;
        }
        WideCharToMultiByte(
            CP_THREAD_ACP,
            0,
            pszSrc,
            wide_len(pszSrc) as INT + 1,
            p,
            n + 1,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
        *p.add(n as usize) = 0;
        s.m_pszData = p;
        s.m_nLength = n;
        s
    }

    /// `(LPCSTR)str` -- never NULL, like `CStringW::GetString`.
    pub fn GetString(&self) -> *const u8 {
        if self.m_pszData.is_null() {
            b"\0".as_ptr()
        } else {
            self.m_pszData
        }
    }

    pub fn GetLength(&self) -> i32 {
        self.m_nLength
    }
}

impl Default for CStringA {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CStringA {
    fn drop(&mut self) {
        unsafe {
            if !self.m_pszData.is_null() {
                <CLocalAllocator as CAllocator>::Free(self.m_pszData as *mut c_void);
            }
        }
    }
}

/// `CPath` (atlpath.h's `CPathT<CStringW>`), as far as modules use it.
#[cfg(feature = "path")]
pub struct CPath {
    pub m_strPath: CStringW,
}

#[cfg(feature = "path")]
extern "system" {
    fn PathCompactPathExW(pszOut: *mut WCHAR, pszSrc: *const WCHAR, cchMax: UINT, dwFlags: u32)
        -> BOOL;
}

#[cfg(feature = "path")]
impl CPath {
    /// `CPathT(PCXSTR pszPath)`.
    pub unsafe fn from_ptr(pszPath: *const WCHAR) -> Self {
        CPath { m_strPath: CStringW::from_ptr(pszPath) }
    }

    /// atlpath.h:180 `CompactPathEx(UINT nMaxChars, DWORD dwFlags = 0)`, line for line.
    pub unsafe fn CompactPathEx(&mut self, nMaxChars: UINT, dwFlags: u32) -> BOOL {
        let mut strTemp = CStringW::new();
        let str = strTemp.GetBuffer(nMaxChars as i32);
        let result = PathCompactPathExW(str, self.m_strPath.GetString(), nMaxChars, dwFlags);
        strTemp.ReleaseBuffer();
        self.m_strPath = strTemp;
        result
    }

    /// `operator PCXSTR()`.
    pub fn GetString(&self) -> *const WCHAR {
        self.m_strPath.GetString()
    }
}

impl Drop for CStringW {
    fn drop(&mut self) {
        unsafe {
            if !self.m_pszData.is_null() {
                <CLocalAllocator as CAllocator>::Free(self.m_pszData as *mut c_void);
            }
        }
    }
}