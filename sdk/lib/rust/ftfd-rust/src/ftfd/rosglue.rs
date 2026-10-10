use crate::prelude::*;
use ntkrnl_sys::ddk::{FILE, va_list};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
/// Multi-character constant 'PYTF' (little-endian byte order as MSVC/GCC would encode it).
pub const TAG_FREETYPE: u32 = 0x5059_5446;

#[repr(C)]
pub struct _MALLOC_HEADER {
    pub Size: usize,
    pub Alignment: usize,
}

pub type MALLOC_HEADER = _MALLOC_HEADER;
pub type PMALLOC_HEADER = *mut _MALLOC_HEADER;
#[no_mangle]
pub unsafe extern "C" fn FT_Message_v(format: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    // va_start/va_end live in the C trampoline (rust_varargs_shim.c); `ap_list` is its `&ap`.
    let va: va_list = *(ap_list as *mut va_list);
    EngDebugPrint(b"FreeType: \0".as_ptr() as PCHAR, format as PCHAR, va);
}

#[no_mangle]
pub unsafe extern "C" fn FT_Panic_v(format: *const ::core::ffi::c_char, ap_list: *mut ::core::ffi::c_void) {
    // va_start/va_end live in the C trampoline (rust_varargs_shim.c); `ap_list` is its `&ap`.
    let va: va_list = *(ap_list as *mut va_list);
    EngDebugPrint(
        b"FreeType: \0".as_ptr() as *const ::core::ffi::c_char as PCHAR,
        format as PCHAR,
        va,
    );
    EngBugCheckEx(
        0xDEADBEEF,
        0,
        0,
        0,
        0,
    );
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `FT_Panic_v` above.
    pub fn FT_Panic(format: *const ::core::ffi::c_char, ...);
}

#[no_mangle]
pub unsafe extern "C" fn malloc(mut Size: size_t) -> *mut ::core::ffi::c_void {
    let mut Header: PMALLOC_HEADER = ::core::ptr::null_mut::<_MALLOC_HEADER>();
    Header = EngAllocMem(
        0 as ULONG,
        (::core::mem::size_of::<MALLOC_HEADER>()).wrapping_add(Size as usize) as ULONG,
        TAG_FREETYPE as ULONG,
    ) as PMALLOC_HEADER;
    if Header.is_null() {
        return core::ptr::null_mut();
    }
    (*Header).Size = (Size as SIZE_T) as _;
    (*Header).Alignment = ((-(1 as ::core::ffi::c_int)) as SIZE_T) as _;
    return Header.offset(1 as isize) as *mut ::core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "C" fn realloc(mut Object: *mut ::core::ffi::c_void, mut Size: size_t) -> *mut ::core::ffi::c_void {
    let mut NewObject: PVOID = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut OldHeader: PMALLOC_HEADER = ::core::ptr::null_mut::<_MALLOC_HEADER>();
    let mut CopySize: size_t = 0;

    NewObject = malloc(Size) as PVOID;
    if NewObject.is_null() {
        return core::ptr::null_mut();
    }

    if Object.is_null() {
        return NewObject as *mut ::core::ffi::c_void;
    }

    OldHeader = (Object as PMALLOC_HEADER).offset(-(1 as isize));
    CopySize = min((*OldHeader).Size, Size);
    memcpy(NewObject as *mut ::core::ffi::c_void, Object, CopySize);

    free(Object);

    return NewObject as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn free(Object: *mut core::ffi::c_void) {
    if !Object.is_null() {
        EngFreeMem((Object as PMALLOC_HEADER).offset(-(1 as isize)) as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn fopen(FileName: *const i8, Mode: *const i8) -> *mut FILE {
    FT_Message(b"Freetype tries to open file %s\n\0".as_ptr() as *const i8, FileName);
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn fseek(Stream: *mut FILE, Offset: core::ffi::c_long, Origin: core::ffi::c_int) -> core::ffi::c_int {
    FT_Message(b"Doubleplus ungood: freetype shouldn't fseek!\n\0".as_ptr() as *const core::ffi::c_char);
    return -(1 as core::ffi::c_int);
}

#[no_mangle]
pub unsafe extern "C" fn ftell(Stream: *mut FILE) -> ::core::ffi::c_long {
    FT_Message(b"Doubleplus ungood: freetype shouldn't ftell!\n\0".as_ptr() as *const i8);
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn fread(Buffer: *mut core::ffi::c_void, Size: size_t, Count: size_t, Stream: *mut FILE) -> size_t {
    FT_Message(b"Doubleplus ungood: freetype shouldn't fread!\n\0".as_ptr() as *const i8);
    return 0;
}

#[no_mangle]
pub unsafe extern "C" fn fclose(Stream: *mut FILE) -> i32 {
    let _ = Stream;
    FT_Message(b"Doubleplus ungood: freetype shouldn't fclose!\n\0".as_ptr() as *const i8);
    return EOF;
}
