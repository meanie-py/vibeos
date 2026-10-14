//! HSTRING (combase/string.c) — the WinRT string API, hosted in ole32 because this
//! tree keeps ole32 monolithic (no combase.dll; api-ms-win-core-winrt-string-* route
//! here). Semantics follow Wine's combase/string.c exactly: a NULL HSTRING is the
//! empty string, caller-stack "reference" strings are never freed and are deep-copied
//! by Duplicate, heap strings are refcounted.
//!
//! Self-contained FFI (kernel32 heap only); no allocator or std.
use core::ffi::c_void;
use core::mem::{offset_of, size_of};
use core::ptr;
use core::sync::atomic::{AtomicI32, Ordering};

type HRESULT = i32;
type BOOL = i32;
type HSTRING = *mut c_void;
type HSTRING_BUFFER = *mut c_void;

const S_OK: HRESULT = 0;
const E_BOUNDS: HRESULT = 0x8000000Bu32 as i32;
const E_POINTER: HRESULT = 0x80004003u32 as i32;
const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as i32;

const HSTRING_REFERENCE_FLAG: u32 = 1;

mod k32 {
    use core::ffi::c_void;
    extern "system" {
        pub fn GetProcessHeap() -> *mut c_void;
        pub fn HeapAlloc(heap: *mut c_void, flags: u32, bytes: usize) -> *mut c_void;
        pub fn HeapFree(heap: *mut c_void, flags: u32, mem: *mut c_void) -> i32;
    }
}

/// Wine's `struct hstring_header`; lives inside the SDK's opaque HSTRING_HEADER.
#[repr(C)]
struct HStringHeader {
    flags: u32,
    length: u32,
    padding1: u32,
    padding2: u32,
    str_: *const u16,
}

/// Windows SDK HSTRING_HEADER: union { PVOID Reserved1; char Reserved2[24 | 20]; }.
#[repr(C)]
pub struct HSTRING_HEADER {
    reserved: [usize; 0],
    bytes: [u8; HSTRING_HEADER_BYTES],
}
#[cfg(target_pointer_width = "64")]
const HSTRING_HEADER_BYTES: usize = 24;
#[cfg(target_pointer_width = "32")]
const HSTRING_HEADER_BYTES: usize = 20;
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<HSTRING_HEADER>() == 24);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<HSTRING_HEADER>() == 20);
const _: () = assert!(size_of::<HStringHeader>() <= size_of::<HSTRING_HEADER>());
const _: () = assert!(core::mem::align_of::<HStringHeader>() <= core::mem::align_of::<HSTRING_HEADER>());

#[repr(C)]
struct HStringPrivate {
    header: HStringHeader,
    refcount: i32,
    buffer: [u16; 1],
}

static EMPTY: [u16; 1] = [0];

#[inline]
fn empty_ptr() -> *mut u16 {
    EMPTY.as_ptr() as *mut u16
}

#[inline]
unsafe fn from_buffer(buf: HSTRING_BUFFER) -> *mut HStringPrivate {
    (buf as *mut u8).sub(offset_of!(HStringPrivate, buffer)) as *mut HStringPrivate
}

unsafe fn alloc_string(len: u32) -> *mut HStringPrivate {
    let bytes = match (len as usize)
        .checked_add(1)
        .and_then(|n| n.checked_mul(2))
        .and_then(|n| n.checked_add(offset_of!(HStringPrivate, buffer)))
    {
        Some(b) => b,
        None => return ptr::null_mut(),
    };
    let p = k32::HeapAlloc(k32::GetProcessHeap(), 0, bytes) as *mut HStringPrivate;
    if p.is_null() {
        return p;
    }
    let buf = ptr::addr_of_mut!((*p).buffer) as *mut u16;
    (*p).header.flags = 0;
    (*p).header.length = len;
    (*p).header.padding1 = 0;
    (*p).header.padding2 = 0;
    (*p).header.str_ = buf;
    (*p).refcount = 1;
    *buf.add(len as usize) = 0;
    p
}

#[inline]
unsafe fn refcount<'a>(p: *mut HStringPrivate) -> &'a AtomicI32 {
    AtomicI32::from_ptr(ptr::addr_of_mut!((*p).refcount))
}

#[inline]
unsafe fn hs(s: HSTRING) -> *mut HStringPrivate {
    s as *mut HStringPrivate
}

#[inline]
unsafe fn slice<'a>(p: *mut HStringPrivate) -> &'a [u16] {
    core::slice::from_raw_parts((*p).header.str_, (*p).header.length as usize)
}

#[no_mangle]
pub unsafe extern "system" fn WindowsCreateString(ptr_: *const u16, len: u32, out: *mut HSTRING) -> HRESULT {
    if out.is_null() {
        return E_INVALIDARG;
    }
    if len == 0 {
        *out = ptr::null_mut();
        return S_OK;
    }
    if ptr_.is_null() {
        return E_POINTER;
    }
    let p = alloc_string(len);
    if p.is_null() {
        // alloc_string already set nothing; Wine leaves *out unspecified here.
        return E_OUTOFMEMORY;
    }
    ptr::copy_nonoverlapping(ptr_, ptr::addr_of_mut!((*p).buffer) as *mut u16, len as usize);
    *out = p as HSTRING;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsCreateStringReference(
    ptr_: *const u16,
    len: u32,
    header: *mut HSTRING_HEADER,
    out: *mut HSTRING,
) -> HRESULT {
    if out.is_null() || header.is_null() {
        return E_INVALIDARG;
    }
    if !ptr_.is_null() && *ptr_.add(len as usize) != 0 {
        return E_INVALIDARG;
    }
    if len == 0 {
        *out = ptr::null_mut();
        return S_OK;
    }
    if ptr_.is_null() {
        return E_POINTER;
    }
    let h = header as *mut HStringHeader;
    (*h).str_ = ptr_;
    (*h).length = len;
    (*h).flags = HSTRING_REFERENCE_FLAG;
    (*h).padding1 = 0;
    (*h).padding2 = 0;
    *out = header as HSTRING;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsDeleteString(s: HSTRING) -> HRESULT {
    if s.is_null() {
        return S_OK;
    }
    let p = hs(s);
    if (*p).header.flags & HSTRING_REFERENCE_FLAG != 0 {
        return S_OK;
    }
    if refcount(p).fetch_sub(1, Ordering::AcqRel) == 1 {
        k32::HeapFree(k32::GetProcessHeap(), 0, p as *mut c_void);
    }
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsDuplicateString(s: HSTRING, out: *mut HSTRING) -> HRESULT {
    if out.is_null() {
        return E_INVALIDARG;
    }
    if s.is_null() {
        *out = ptr::null_mut();
        return S_OK;
    }
    let p = hs(s);
    if (*p).header.flags & HSTRING_REFERENCE_FLAG != 0 {
        return WindowsCreateString((*p).header.str_, (*p).header.length, out);
    }
    refcount(p).fetch_add(1, Ordering::AcqRel);
    *out = s;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsPreallocateStringBuffer(
    len: u32,
    outptr: *mut *mut u16,
    out: *mut HSTRING_BUFFER,
) -> HRESULT {
    if outptr.is_null() || out.is_null() {
        return E_POINTER;
    }
    if len == 0 {
        *outptr = empty_ptr();
        *out = ptr::null_mut();
        return S_OK;
    }
    let p = alloc_string(len);
    if p.is_null() {
        return E_OUTOFMEMORY;
    }
    let buf = ptr::addr_of_mut!((*p).buffer) as *mut u16;
    *outptr = buf;
    *out = buf as HSTRING_BUFFER;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsDeleteStringBuffer(buf: HSTRING_BUFFER) -> HRESULT {
    let p = if buf.is_null() { ptr::null_mut() } else { from_buffer(buf) };
    WindowsDeleteString(p as HSTRING)
}

#[no_mangle]
pub unsafe extern "system" fn WindowsPromoteStringBuffer(buf: HSTRING_BUFFER, out: *mut HSTRING) -> HRESULT {
    if out.is_null() {
        return E_POINTER;
    }
    if buf.is_null() {
        *out = ptr::null_mut();
        return S_OK;
    }
    let p = from_buffer(buf);
    let b = ptr::addr_of!((*p).buffer) as *const u16;
    if *b.add((*p).header.length as usize) != 0
        || (*p).header.flags & HSTRING_REFERENCE_FLAG != 0
        || (*p).refcount != 1
    {
        return E_INVALIDARG;
    }
    *out = p as HSTRING;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsGetStringLen(s: HSTRING) -> u32 {
    if s.is_null() {
        return 0;
    }
    (*hs(s)).header.length
}

#[no_mangle]
pub unsafe extern "system" fn WindowsGetStringRawBuffer(s: HSTRING, len: *mut u32) -> *const u16 {
    if s.is_null() {
        if !len.is_null() {
            *len = 0;
        }
        return EMPTY.as_ptr();
    }
    let p = hs(s);
    if !len.is_null() {
        *len = (*p).header.length;
    }
    (*p).header.str_
}

#[no_mangle]
pub unsafe extern "system" fn WindowsIsStringEmpty(s: HSTRING) -> BOOL {
    if s.is_null() {
        return 1;
    }
    ((*hs(s)).header.length == 0) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn WindowsStringHasEmbeddedNull(s: HSTRING, out: *mut BOOL) -> HRESULT {
    if out.is_null() {
        return E_INVALIDARG;
    }
    if s.is_null() {
        *out = 0;
        return S_OK;
    }
    *out = slice(hs(s)).iter().any(|&c| c == 0) as BOOL;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsSubstring(s: HSTRING, start: u32, out: *mut HSTRING) -> HRESULT {
    let len = WindowsGetStringLen(s);
    if out.is_null() {
        return E_INVALIDARG;
    }
    if start > len {
        return E_BOUNDS;
    }
    if start == len {
        *out = ptr::null_mut();
        return S_OK;
    }
    WindowsCreateString((*hs(s)).header.str_.add(start as usize), len - start, out)
}

#[no_mangle]
pub unsafe extern "system" fn WindowsSubstringWithSpecifiedLength(
    s: HSTRING,
    start: u32,
    len: u32,
    out: *mut HSTRING,
) -> HRESULT {
    if out.is_null() {
        return E_INVALIDARG;
    }
    let end = match start.checked_add(len) {
        Some(e) if e <= WindowsGetStringLen(s) => e,
        _ => return E_BOUNDS,
    };
    let _ = end;
    if len == 0 {
        *out = ptr::null_mut();
        return S_OK;
    }
    WindowsCreateString((*hs(s)).header.str_.add(start as usize), len, out)
}

#[no_mangle]
pub unsafe extern "system" fn WindowsConcatString(s1: HSTRING, s2: HSTRING, out: *mut HSTRING) -> HRESULT {
    if out.is_null() {
        return E_INVALIDARG;
    }
    if s1.is_null() {
        return WindowsDuplicateString(s2, out);
    }
    if s2.is_null() {
        return WindowsDuplicateString(s1, out);
    }
    let (p1, p2) = (hs(s1), hs(s2));
    let (l1, l2) = ((*p1).header.length, (*p2).header.length);
    if l1 == 0 && l2 == 0 {
        *out = ptr::null_mut();
        return S_OK;
    }
    let total = match l1.checked_add(l2) {
        Some(t) => t,
        None => return E_OUTOFMEMORY,
    };
    let p = alloc_string(total);
    if p.is_null() {
        return E_OUTOFMEMORY;
    }
    let buf = ptr::addr_of_mut!((*p).buffer) as *mut u16;
    ptr::copy_nonoverlapping((*p1).header.str_, buf, l1 as usize);
    ptr::copy_nonoverlapping((*p2).header.str_, buf.add(l1 as usize), l2 as usize);
    *out = p as HSTRING;
    S_OK
}

/// Wine routes this through CompareStringOrdinal(..., FALSE) - CSTR_EQUAL, i.e. an
/// ordinal UTF-16 code-unit compare yielding -1/0/1; done directly here.
#[no_mangle]
pub unsafe extern "system" fn WindowsCompareStringOrdinal(s1: HSTRING, s2: HSTRING, res: *mut i32) -> HRESULT {
    if res.is_null() {
        return E_INVALIDARG;
    }
    if s1 == s2 {
        *res = 0;
        return S_OK;
    }
    let a: &[u16] = if s1.is_null() { &[] } else { slice(hs(s1)) };
    let b: &[u16] = if s2.is_null() { &[] } else { slice(hs(s2)) };
    *res = match a.cmp(b) {
        core::cmp::Ordering::Less => -1,
        core::cmp::Ordering::Equal => 0,
        core::cmp::Ordering::Greater => 1,
    };
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn WindowsTrimStringStart(s1: HSTRING, s2: HSTRING, out: *mut HSTRING) -> HRESULT {
    if out.is_null() || s2.is_null() || (*hs(s2)).header.length == 0 {
        return E_INVALIDARG;
    }
    if s1.is_null() {
        *out = ptr::null_mut();
        return S_OK;
    }
    let (a, set) = (slice(hs(s1)), slice(hs(s2)));
    let start = a.iter().position(|c| !set.contains(c)).unwrap_or(a.len());
    if start != 0 {
        WindowsCreateString((*hs(s1)).header.str_.add(start), (a.len() - start) as u32, out)
    } else {
        WindowsDuplicateString(s1, out)
    }
}

#[no_mangle]
pub unsafe extern "system" fn WindowsTrimStringEnd(s1: HSTRING, s2: HSTRING, out: *mut HSTRING) -> HRESULT {
    if out.is_null() || s2.is_null() || (*hs(s2)).header.length == 0 {
        return E_INVALIDARG;
    }
    if s1.is_null() {
        *out = ptr::null_mut();
        return S_OK;
    }
    let (a, set) = (slice(hs(s1)), slice(hs(s2)));
    let len = a.iter().rposition(|c| !set.contains(c)).map_or(0, |i| i + 1);
    if len < a.len() {
        WindowsCreateString((*hs(s1)).header.str_, len as u32, out)
    } else {
        WindowsDuplicateString(s1, out)
    }
}
