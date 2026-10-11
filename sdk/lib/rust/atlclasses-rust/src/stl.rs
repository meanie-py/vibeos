//! The C++ standard library containers the ported C++ modules use, on the CRT heap.
//!
//! Same constraint as the ATL collections beside it: the crate is `no_std` with no `alloc`, so
//! storage comes from `CCRTAllocator` (what the C++ `operator new` behind libstdc++/STLport
//! uses too). Measured surface only -- drwtsn32 (`std::string`, `std::wstring`,
//! `std::vector`, `std::map`, `std::sort`) and ksproxy (`std::vector`).
//!
//! Each container holds raw pointers rather than a `Vec`, so a stray `core::mem::zeroed()`
//! (the translation's default for a C++ member) is merely EMPTY, never UB.

use super::{CAllocator, CCRTAllocator};
use core::ffi::c_void;

// -------------------------------------------------------------------------------------------
// std::vector
// -------------------------------------------------------------------------------------------

/// `std::vector<T>`. `operator[]` is [`StdVector::at`] and returns a POINTER (call sites take
/// `&v[i]` as often as `v[i]`); iteration and `std::sort` go through [`StdVector::as_mut_slice`].
pub struct StdVector<T> {
    data: *mut T,
    len: usize,
    cap: usize,
}

impl<T> StdVector<T> {
    pub const fn new() -> Self {
        Self { data: core::ptr::null_mut(), len: 0, cap: 0 }
    }

    /// Grow to hold at least `n`. libstdc++ doubles; an allocation failure is `std::bad_alloc`,
    /// which nothing in the callers catches, so it terminates -- as here.
    unsafe fn reserve_for(&mut self, n: usize) {
        if n <= self.cap {
            return;
        }
        let cap = core::cmp::max(n, self.cap.saturating_mul(2)).max(1);
        let bytes = cap.checked_mul(core::mem::size_of::<T>().max(1)).expect("vector too long");
        let p = CCRTAllocator::Reallocate(self.data as *mut c_void, bytes) as *mut T;
        if p.is_null() {
            panic!("std::bad_alloc");
        }
        self.data = p;
        self.cap = cap;
    }

    pub unsafe fn push_back(&mut self, t: T) {
        self.reserve_for(self.len + 1);
        core::ptr::write(self.data.add(self.len), t);
        self.len += 1;
    }

    pub fn size(&self) -> usize {
        self.len
    }

    pub fn empty(&self) -> bool {
        self.len == 0
    }

    pub fn data(&self) -> *mut T {
        self.data
    }

    /// `v[i]` / `&v[i]`. Out of range is UB in C++; here it is a null pointer that faults at
    /// the site rather than reading past the buffer.
    pub fn at(&self, i: usize) -> *mut T {
        if i >= self.len {
            return core::ptr::null_mut();
        }
        unsafe { self.data.add(i) }
    }

    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            return &[];
        }
        unsafe { core::slice::from_raw_parts(self.data, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            return &mut [];
        }
        unsafe { core::slice::from_raw_parts_mut(self.data, self.len) }
    }

    /// `std::sort(v.begin(), v.end(), less)`: `less(a, b)` is the C++ strict-weak-order
    /// predicate. std::sort is not stable, and neither is this.
    pub fn sort_by_less(&mut self, mut less: impl FnMut(&T, &T) -> bool) {
        self.as_mut_slice().sort_unstable_by(|a, b| {
            if less(a, b) {
                core::cmp::Ordering::Less
            } else if less(b, a) {
                core::cmp::Ordering::Greater
            } else {
                core::cmp::Ordering::Equal
            }
        });
    }

    pub fn clear(&mut self) {
        unsafe {
            for i in 0..self.len {
                core::ptr::drop_in_place(self.data.add(i));
            }
        }
        self.len = 0;
    }
}

impl<T> Default for StdVector<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for StdVector<T> {
    fn drop(&mut self) {
        self.clear();
        if !self.data.is_null() {
            unsafe { CCRTAllocator::Free(self.data as *mut c_void) };
        }
        self.data = core::ptr::null_mut();
        self.cap = 0;
    }
}

impl<T: Clone> Clone for StdVector<T> {
    fn clone(&self) -> Self {
        let mut v = Self::new();
        for t in self.as_slice() {
            unsafe { v.push_back(t.clone()) };
        }
        v
    }
}

// -------------------------------------------------------------------------------------------
// std::basic_string
// -------------------------------------------------------------------------------------------

/// A character `std::basic_string` stores: `u8` for `std::string`, `u16` for `std::wstring`.
pub trait StdChar: Copy + PartialEq + Default {}
impl StdChar for u8 {}
impl StdChar for u16 {}

/// `std::basic_string<C>`. Always NUL-terminated in its buffer, so [`StdBasicString::c_str`]
/// is just the data pointer, as in libstdc++.
pub struct StdBasicString<C: StdChar> {
    buf: StdVector<C>, // holds len + 1 chars once non-empty; the last is NUL
}

pub type StdString = StdBasicString<u8>;
pub type StdWString = StdBasicString<u16>;

/// `std::string::npos` (lower-case, as C++ spells it at every call site).
#[allow(non_upper_case_globals)]
pub const npos: usize = usize::MAX;

impl<C: StdChar> StdBasicString<C> {
    pub const fn new() -> Self {
        Self { buf: StdVector::new() }
    }

    /// `std::basic_string(const C*)`: copy up to the NUL. A null pointer is UB in C++; it is
    /// the empty string here.
    pub unsafe fn from_ptr(s: *const C) -> Self {
        let mut out = Self::new();
        out.assign(s);
        out
    }

    /// `std::basic_string(const C*, n)`.
    pub unsafe fn from_ptr_len(s: *const C, n: usize) -> Self {
        let mut out = Self::new();
        out.append_n(s, n);
        out
    }

    pub fn size(&self) -> usize {
        self.buf.size().saturating_sub(1)
    }

    pub fn length(&self) -> usize {
        self.size()
    }

    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    /// `c_str()` / `data()`: a NUL-terminated pointer, valid until the next mutation. An
    /// empty string has no buffer, so it points at a static NUL instead.
    pub fn c_str(&self) -> *const C {
        if self.buf.empty() {
            return empty_nul::<C>();
        }
        self.buf.data()
    }

    pub fn chars(&self) -> &[C] {
        let s = self.buf.as_slice();
        &s[..s.len().saturating_sub(1)]
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub unsafe fn assign(&mut self, s: *const C) {
        self.clear();
        if !s.is_null() {
            self.append_n(s, c_len(s));
        }
    }

    pub unsafe fn append(&mut self, s: *const C) {
        if !s.is_null() {
            self.append_n(s, c_len(s));
        }
    }

    pub unsafe fn append_n(&mut self, s: *const C, n: usize) {
        if n == 0 {
            return;
        }
        if !self.buf.empty() {
            self.buf.len -= 1; // drop the NUL; re-added below
        }
        self.buf.reserve_for(self.buf.size() + n + 1);
        for i in 0..n {
            self.buf.push_back(*s.add(i));
        }
        self.buf.push_back(C::default());
    }

    /// `find_last_of(set)`: the last index whose char is in the NUL-terminated `set`, or `npos`.
    pub unsafe fn find_last_of(&self, set: *const C) -> usize {
        let n = c_len(set);
        let set = core::slice::from_raw_parts(set, n);
        self.chars().iter().rposition(|c| set.contains(c)).unwrap_or(npos)
    }

    /// `find_first_of(set)`.
    pub unsafe fn find_first_of(&self, set: *const C) -> usize {
        let n = c_len(set);
        let set = core::slice::from_raw_parts(set, n);
        self.chars().iter().position(|c| set.contains(c)).unwrap_or(npos)
    }

    /// `substr(pos, n)`; `n` may be `npos`. `pos > size()` throws `std::out_of_range` in C++,
    /// uncaught in every caller -- it terminates here too.
    pub fn substr(&self, pos: usize, n: usize) -> Self {
        let s = self.chars();
        if pos > s.len() {
            panic!("std::out_of_range: basic_string::substr");
        }
        let end = pos.saturating_add(n).min(s.len());
        unsafe { Self::from_ptr_len(s.as_ptr().add(pos), end - pos) }
    }
}

impl<C: StdChar> StdBasicString<C> {
    /// `resize(n)`: truncate, or pad with NUL chars. The buffer always keeps one more NUL past
    /// `size()`, so the `&s[0]`-as-an-out-buffer idiom (drwtsn32: `resize(MAX_PATH)`, then
    /// `GetModuleFileNameExA(.., &s[0], s.size())`, then `resize(len)`) has `n + 1` writable
    /// chars, exactly as libstdc++'s.
    pub unsafe fn resize(&mut self, n: usize) {
        let cur = self.size();
        if n == cur {
            return;
        }
        if n < cur {
            for i in n..self.buf.len {
                core::ptr::drop_in_place(self.buf.data.add(i)); // Copy chars: a no-op
            }
            self.buf.len = n;
            self.buf.push_back(C::default());
            return;
        }
        if !self.buf.empty() {
            self.buf.len -= 1;
        }
        self.buf.reserve_for(n + 1);
        while self.buf.len < n + 1 {
            self.buf.push_back(C::default());
        }
    }

    /// `&s[i]` (also `s.at(i)`'s address): a pointer into the buffer. `i == size()` is the
    /// terminator, as in C++; past it, null.
    pub fn at(&self, i: usize) -> *mut C {
        if self.buf.empty() {
            return if i == 0 { empty_nul::<C>() as *mut C } else { core::ptr::null_mut() };
        }
        self.buf.at(i)
    }
}

/// `s[i]`. Out of range panics (C++ UB); `s[size()]` is the terminator, as in C++.
impl<C: StdChar> core::ops::Index<usize> for StdBasicString<C> {
    type Output = C;
    fn index(&self, i: usize) -> &C {
        let p = self.at(i);
        assert!(!p.is_null(), "basic_string index out of range");
        unsafe { &*p }
    }
}

impl<C: StdChar> core::ops::IndexMut<usize> for StdBasicString<C> {
    fn index_mut(&mut self, i: usize) -> &mut C {
        assert!(i < self.size(), "basic_string index out of range");
        unsafe { &mut *self.buf.at(i) }
    }
}

/// `s += psz` / `s += t`.
impl<C: StdChar> core::ops::AddAssign<*const C> for StdBasicString<C> {
    fn add_assign(&mut self, s: *const C) {
        unsafe { self.append(s) }
    }
}

impl<C: StdChar> core::ops::AddAssign<*mut C> for StdBasicString<C> {
    fn add_assign(&mut self, s: *mut C) {
        unsafe { self.append(s) }
    }
}

impl<C: StdChar> core::ops::AddAssign<&StdBasicString<C>> for StdBasicString<C> {
    fn add_assign(&mut self, t: &StdBasicString<C>) {
        let c = t.chars();
        unsafe { self.append_n(c.as_ptr(), c.len()) }
    }
}

impl<C: StdChar> Default for StdBasicString<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: StdChar> Clone for StdBasicString<C> {
    fn clone(&self) -> Self {
        Self { buf: self.buf.clone() }
    }
}

impl<C: StdChar> PartialEq for StdBasicString<C> {
    fn eq(&self, o: &Self) -> bool {
        self.chars() == o.chars()
    }
}

unsafe fn c_len<C: StdChar>(s: *const C) -> usize {
    let mut n = 0;
    while *s.add(n) != C::default() {
        n += 1;
    }
    n
}

fn empty_nul<C: StdChar>() -> *const C {
    // Both char widths fit in a u16 zero, and the pointer is only ever read up to its NUL.
    static NUL: u16 = 0;
    &NUL as *const u16 as *const C
}

// -------------------------------------------------------------------------------------------
// std::map
// -------------------------------------------------------------------------------------------

/// One `std::map` element: `it->first` / `it->second`.
pub struct StdPair<K, V> {
    pub first: K,
    pub second: V,
}

/// `std::map<K, V>`: a SORTED array of pairs. Iteration is in key order, as a red-black tree
/// gives; element addresses are NOT stable across inserts (they are in C++) -- no caller in
/// the ported modules holds one across an insert.
pub struct StdMap<K: Ord, V> {
    items: StdVector<StdPair<K, V>>,
}

impl<K: Ord, V> StdMap<K, V> {
    pub const fn new() -> Self {
        Self { items: StdVector::new() }
    }

    fn search(&self, k: &K) -> Result<usize, usize> {
        self.items.as_slice().binary_search_by(|p| p.first.cmp(k))
    }

    pub fn size(&self) -> usize {
        self.items.size()
    }

    pub fn empty(&self) -> bool {
        self.items.empty()
    }

    /// `find(k)`: the element's value, or null for `end()`.
    pub fn find(&self, k: &K) -> *mut V {
        match self.search(k) {
            Ok(i) => unsafe { &mut (*self.items.at(i)).second as *mut V },
            Err(_) => core::ptr::null_mut(),
        }
    }

    /// `insert({k, v})`: keeps an existing value (C++ returns `{it, false}`); true if inserted.
    pub unsafe fn insert(&mut self, k: K, v: V) -> bool {
        match self.search(&k) {
            Ok(_) => false,
            Err(i) => {
                self.insert_at(i, StdPair { first: k, second: v });
                true
            }
        }
    }

    unsafe fn insert_at(&mut self, i: usize, p: StdPair<K, V>) {
        self.items.push_back(p);
        // rotate the new last element down into place
        self.items.as_mut_slice()[i..].rotate_right(1);
    }

    /// `m[k] = v` where V's default constructor has no effect beyond zeroing (drwtsn32's
    /// `Threads[id] = ThreadData(h)`): C++ default-constructs then assigns; this stores `v`.
    /// C++17's `insert_or_assign`, which is what that statement means.
    pub unsafe fn insert_or_assign(&mut self, k: K, v: V) {
        match self.search(&k) {
            Ok(i) => (*self.items.at(i)).second = v,
            Err(i) => self.insert_at(i, StdPair { first: k, second: v }),
        }
    }

    /// `erase(k)`: the number of elements removed (0 or 1).
    pub fn erase(&mut self, k: &K) -> usize {
        match self.search(k) {
            Ok(i) => {
                let s = self.items.as_mut_slice();
                s[i..].rotate_left(1);
                let last = self.items.len - 1;
                unsafe { core::ptr::drop_in_place(self.items.data.add(last)) };
                self.items.len = last;
                1
            }
            Err(_) => 0,
        }
    }

    pub fn as_slice(&self) -> &[StdPair<K, V>] {
        self.items.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [StdPair<K, V>] {
        self.items.as_mut_slice()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<K: Ord, V: Default> StdMap<K, V> {
    /// `m[k]`: the value, default-constructed and inserted first if absent.
    pub unsafe fn index(&mut self, k: K) -> *mut V {
        let i = match self.search(&k) {
            Ok(i) => i,
            Err(i) => {
                self.insert_at(i, StdPair { first: k, second: V::default() });
                i
            }
        };
        &mut (*self.items.at(i)).second as *mut V
    }
}

impl<K: Ord, V> Default for StdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(s: &str) -> std::vec::Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    #[test]
    fn wstring_find_last_of_and_substr_split_a_path() {
        // drwtsn32: ProcessName = ProcessPath.substr(ProcessPath.find_last_of(L"\\/") + 1)
        unsafe {
            let p = StdWString::from_ptr(w("C:\\ReactOS/system32\\notepad.exe").as_ptr());
            let pos = p.find_last_of(w("\\/").as_ptr());
            assert_eq!(pos, 19);
            let name = p.substr(pos + 1, npos);
            assert_eq!(name.chars(), &w("notepad.exe")[..11]);
            assert_eq!(*name.c_str().add(11), 0);
            assert_eq!(p.find_last_of(w("|").as_ptr()), npos);
        }
    }

    #[test]
    fn an_empty_string_has_a_nul_c_str() {
        let s = StdString::new();
        assert!(s.empty());
        assert_eq!(unsafe { *s.c_str() }, 0);
        let z: StdString = unsafe { core::mem::zeroed() };
        assert_eq!(z.size(), 0);
        assert_eq!(unsafe { *z.c_str() }, 0);
    }

    #[test]
    fn resize_gives_an_out_buffer_then_trims_to_the_written_length() {
        // drwtsn32's ModuleData::Update: resize(MAX_PATH), write via &s[0], resize(len)
        unsafe {
            let mut s = StdString::new();
            s.resize(8);
            assert_eq!(s.size(), 8);
            let p = s.at(0);
            for (i, b) in b"abc\0".iter().enumerate() {
                *p.add(i) = *b;
            }
            s.resize(3);
            assert_eq!(core::slice::from_raw_parts(s.c_str(), 4), b"abc\0");
            assert_eq!(s[1], b'b');
            assert_eq!(s[3], 0); // s[size()] is the terminator
            s += b"de\0".as_ptr();
            assert_eq!(s.chars(), b"abcde");
        }
    }

    #[test]
    fn append_keeps_one_terminator() {
        unsafe {
            let mut s = StdString::from_ptr(b"ab\0".as_ptr());
            s.append(b"cd\0".as_ptr());
            assert_eq!(s.size(), 4);
            assert_eq!(core::slice::from_raw_parts(s.c_str(), 5), b"abcd\0");
        }
    }

    #[test]
    fn vector_sort_by_less_orders_like_std_sort() {
        let mut v = StdVector::new();
        for x in [5u32, 1, 4, 1, 3] {
            unsafe { v.push_back(x) };
        }
        v.sort_by_less(|a, b| a < b);
        assert_eq!(v.as_slice(), &[1, 1, 3, 4, 5]);
        assert!(v.at(5).is_null());
        assert_eq!(unsafe { *v.at(2) }, 3);
    }

    #[test]
    fn map_iterates_in_key_order_and_index_inserts_default() {
        let mut m: StdMap<u32, u32> = StdMap::new();
        unsafe {
            *m.index(30) = 3;
            assert!(m.insert(10, 1));
            assert!(!m.insert(10, 99)); // existing key keeps its value
            *m.index(20) += 2;
        }
        let keys: std::vec::Vec<_> = m.as_slice().iter().map(|p| (p.first, p.second)).collect();
        assert_eq!(keys, [(10, 1), (20, 2), (30, 3)]);
        assert_eq!(unsafe { *m.find(&20) }, 2);
        assert!(m.find(&25).is_null());
        assert_eq!(m.erase(&20), 1);
        assert_eq!(m.erase(&20), 0);
        assert_eq!(m.size(), 2);
    }
}
