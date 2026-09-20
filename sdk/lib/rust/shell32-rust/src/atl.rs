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
    CoTaskMemAlloc, CoTaskMemFree, CoTaskMemRealloc, LocalAlloc, LocalFree, LocalReAlloc,
    MultiByteToWideChar, RegCloseKey, RegOpenKeyExW, BOOL, BSTR, CP_THREAD_ACP, HKEY, HLOCAL,
    HRESULT, INT, LONG, LPCSTR, LPCWSTR, POSITION, REGSAM, SIZE_T, UINT, VARIANT, WCHAR,
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

impl<K, V> Default for CSimpleMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}


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
        let n = crate::win32::LoadStringW(
            crate::win32::shell32_hInstance,
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
    unsafe fn format_begin(&mut self, need: INT) -> *mut WCHAR {
        if need < 0 {
            self.SetString(EMPTY_WSTR.as_ptr(), 0);
            return core::ptr::null_mut();
        }
        self.GetBuffer(need)
    }

    /// `Format(PCXSTR pszFormat, <one %s>)`.
    pub unsafe fn FormatStr(&mut self, pszFormat: *const WCHAR, a: *const WCHAR) {
        let need = _scwprintf(pszFormat, a);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        crate::prelude::swprintf(buf, pszFormat, a);
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
        crate::prelude::swprintf(buf, pszFormat, a, b);
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

    /// `Format(PCXSTR pszFormat, <one %d>)`.
    pub unsafe fn FormatInt(&mut self, pszFormat: *const WCHAR, a: INT) {
        let need = _scwprintf(pszFormat, a);
        let buf = self.format_begin(need);
        if buf.is_null() {
            return;
        }
        crate::prelude::swprintf(buf, pszFormat, a);
        self.ReleaseBuffer();
    }
}

extern "C" {
    /// msvcrt `@ varargs _scwprintf(wstr)` — the character count a `swprintf` of the same
    /// arguments would produce, excluding the terminator. Negative on a bad format.
    fn _scwprintf(format: *const WCHAR, ...) -> INT;
    /// msvcrt — case-insensitive wide compare, what ATL's W string traits use for
    /// `CompareNoCase`.
    fn _wcsicmp(left: *const WCHAR, right: *const WCHAR) -> INT;
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
                <CLocalAllocator as CAllocator>::Free(self.m_pszData as *mut c_void);
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// CComVariant
// -------------------------------------------------------------------------------------------

extern "system" {
    /// oleaut32 `VariantClear` — releases whatever the variant owns and sets it VT_EMPTY.
    fn VariantClear(pvarg: *mut VARIANT) -> HRESULT;
}

/// ATL's owning VARIANT (atlcomcli.h). ATL DERIVES it from `VARIANT` and adds a destructor,
/// so it IS a VARIANT at the ABI — hence `repr(transparent)`, which is what makes the
/// `&mut var as *mut CComVariant as *mut VARIANT` casts the translation emits sound.
///
/// The default ctor leaves it `VT_EMPTY`, and VT_EMPTY is 0, so an all-zero variant is the
/// correct starting state — `core::mem::zeroed()` in a translated body is right, not lazy.
#[repr(transparent)]
pub struct CComVariant(pub VARIANT);

impl CComVariant {
    /// `CComVariant()` — VT_EMPTY.
    pub unsafe fn new() -> Self {
        Self(core::mem::zeroed())
    }

    /// The `VARIANT*` every oleaut32 entry point wants.
    pub fn as_mut_ptr(&mut self) -> *mut VARIANT {
        core::ptr::addr_of_mut!(self.0)
    }
}

impl Drop for CComVariant {
    /// `~CComVariant()` — `VariantClear`. Dropping this is what releases a BSTR or an
    /// interface the variant is holding; lowering the local to a bare `VARIANT` instead
    /// compiles and leaks whatever it owned.
    fn drop(&mut self) {
        unsafe {
            VariantClear(core::ptr::addr_of_mut!(self.0));
        }
    }
}

// -------------------------------------------------------------------------------------------
// CComBSTR
// -------------------------------------------------------------------------------------------

extern "system" {
    /// oleaut32 `SysFreeString`. Documented to ignore a NULL argument, which is why
    /// `CComBSTR::drop` below does not pre-check — ATL's own `~CComBSTR` does not either.
    fn SysFreeString(bstrString: BSTR);
}

/// ATL's owning BSTR wrapper (atlbase.h).
///
/// ONE member, exactly as the bindgen island already renders it
/// (`ATL_CComBSTR { m_str: BSTR }`), so a class embedding this by value gets the C++ layout
/// for free and needs no shim.
///
/// `m_str` is PUBLIC on purpose: the C++ reaches straight into it. `CFolderItemVerb::Init`
/// is `m_name.m_str = name;` — a raw ADOPT of a BSTR the caller already allocated, not a
/// copy — so it has to lower to that same single store or the ownership transfer changes
/// and the string is either leaked or freed twice.
#[repr(C)]
pub struct CComBSTR {
    pub m_str: BSTR,
}

impl CComBSTR {
    /// `CComBSTR()` — the default constructor leaves the string null.
    pub const fn new() -> Self {
        Self { m_str: core::ptr::null_mut() }
    }
}

impl Default for CComBSTR {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CComBSTR {
    /// `~CComBSTR()` — `SysFreeString(m_str)`. This IS the member destructor a translation
    /// would otherwise silently drop, which is the whole reason the type exists rather than
    /// the field being lowered to a bare `BSTR`.
    fn drop(&mut self) {
        unsafe {
            SysFreeString(self.m_str);
        }
    }
}

// -------------------------------------------------------------------------------------------
// CRegKey
// -------------------------------------------------------------------------------------------

/// `CRegKey` — an owning `HKEY`, ported from `sdk/lib/atl/atlbase.h`.
///
/// Only the surface the translated shell32 C++ actually uses: `Open` / `Close` / `Detach`,
/// plus the destructor. `Create`, the value getters and the transaction-manager overloads
/// are deliberately absent — this module records measured usage, not ATL's full published
/// surface, and an unused port is an untested one.
///
/// Two details of ATL's `Open` are load-bearing and easy to "improve" away:
///
///  * it closes the previously-held key **only when the new open succeeds**, so a failed
///    `Open` leaves the old key intact and still owned; and
///  * `Close` on an empty key is `ERROR_SUCCESS`, not an error.
///
/// `m_hKey` is `pub` because the C++ passes a `CRegKey` where an `HKEY` is expected, relying
/// on ATL's `operator HKEY()`. Rust has no implicit conversion, so call sites spell the field.
#[repr(transparent)]
pub struct CRegKey {
    pub m_hKey: HKEY,
}

impl CRegKey {
    pub const fn new() -> Self {
        Self { m_hKey: core::ptr::null_mut() }
    }

    /// ATL: `explicit CRegKey(HKEY hKey)` — takes ownership of an already-open key.
    pub const fn from_raw(hKey: HKEY) -> Self {
        Self { m_hKey: hKey }
    }

    /// Release ownership without closing.
    pub fn Detach(&mut self) -> HKEY {
        let hKey = self.m_hKey;
        self.m_hKey = core::ptr::null_mut();
        hKey
    }

    pub fn Attach(&mut self, hKey: HKEY) {
        self.m_hKey = hKey;
    }

    pub unsafe fn Close(&mut self) -> LONG {
        if self.m_hKey.is_null() {
            return 0; // ERROR_SUCCESS
        }
        RegCloseKey(self.Detach()) as LONG
    }

    /// ATL has a `samDesired = KEY_READ | KEY_WRITE` default argument; Rust has no default
    /// arguments, and every translated call site passes the access mask explicitly anyway.
    pub unsafe fn Open(&mut self, hKeyParent: HKEY, lpszKeyName: LPCWSTR, samDesired: REGSAM) -> LONG {
        let mut hKey: HKEY = core::ptr::null_mut();
        let lRes = RegOpenKeyExW(hKeyParent, lpszKeyName, 0, samDesired, &mut hKey) as LONG;
        if lRes == 0 {
            // ERROR_SUCCESS: adopt the new key, closing whatever we held. On FAILURE the old
            // key is kept — dropping it here would be a use-after-close for the caller.
            self.Close();
            self.m_hKey = hKey;
        }
        lRes
    }
}

impl Default for CRegKey {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CRegKey {
    fn drop(&mut self) {
        unsafe {
            self.Close();
        }
    }
}
