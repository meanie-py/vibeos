//! startup/pesect.c — PE image/section helpers, driven off the
//! linker-defined image base (`__image_base__`).

use crate::winapi::*;

extern "C" {
    // lld-defined image base (mingw `__ImageBase` == `__image_base__`).
    #[link_name = "__image_base__"]
    static __ImageBase: IMAGE_DOS_HEADER;
}

#[inline]
fn image_base() -> *mut u8 {
    core::ptr::addr_of!(__ImageBase) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn _ValidateImageBase(pImageBase: PBYTE) -> WINBOOL {
    let dos = pImageBase as *const IMAGE_DOS_HEADER;
    if (*dos).e_magic != IMAGE_DOS_SIGNATURE {
        return FALSE;
    }
    let nt = pImageBase.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    if (*nt).Signature != IMAGE_NT_SIGNATURE {
        return FALSE;
    }
    let opt = core::ptr::addr_of!((*nt).OptionalHeader);
    if (*opt).Magic != IMAGE_NT_OPTIONAL_HDR_MAGIC {
        return FALSE;
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "C" fn _FindPESection(
    pImageBase: PBYTE,
    rva: DWORD_PTR,
) -> *mut IMAGE_SECTION_HEADER {
    let dos = pImageBase as *const IMAGE_DOS_HEADER;
    let nt = pImageBase.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    let mut sec = image_first_section(nt);
    let n = (*nt).FileHeader.NumberOfSections;
    let mut i = 0u16;
    while i < n {
        if rva >= (*sec).VirtualAddress as usize
            && rva < ((*sec).VirtualAddress + (*sec).VirtualSize) as usize
        {
            return sec as *mut IMAGE_SECTION_HEADER;
        }
        sec = sec.add(1);
        i += 1;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _FindPESectionByName(pName: *const u8) -> *mut IMAGE_SECTION_HEADER {
    // Long names aren't supported.
    if strlen(pName) > IMAGE_SIZEOF_SHORT_NAME {
        return core::ptr::null_mut();
    }
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return core::ptr::null_mut();
    }
    let dos = base as *const IMAGE_DOS_HEADER;
    let nt = base.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    let mut sec = image_first_section(nt);
    let n = (*nt).FileHeader.NumberOfSections;
    let mut i = 0u16;
    while i < n {
        if strncmp(
            core::ptr::addr_of!((*sec).Name[0]),
            pName,
            IMAGE_SIZEOF_SHORT_NAME,
        ) == 0
        {
            return sec as *mut IMAGE_SECTION_HEADER;
        }
        sec = sec.add(1);
        i += 1;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn __mingw_GetSectionForAddress(p: LPVOID) -> *mut IMAGE_SECTION_HEADER {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return core::ptr::null_mut();
    }
    let rva = (p as *mut u8).offset_from(base) as DWORD_PTR;
    _FindPESection(base, rva)
}

#[no_mangle]
pub unsafe extern "C" fn __mingw_GetSectionCount() -> i32 {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return 0;
    }
    let dos = base as *const IMAGE_DOS_HEADER;
    let nt = base.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    (*nt).FileHeader.NumberOfSections as i32
}

#[no_mangle]
pub unsafe extern "C" fn _FindPESectionExec(eNo: usize) -> *mut IMAGE_SECTION_HEADER {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return core::ptr::null_mut();
    }
    let dos = base as *const IMAGE_DOS_HEADER;
    let nt = base.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    let mut sec = image_first_section(nt);
    let n = (*nt).FileHeader.NumberOfSections;
    let mut e = eNo;
    let mut i = 0u16;
    while i < n {
        if (*sec).Characteristics & IMAGE_SCN_MEM_EXECUTE != 0 {
            if e == 0 {
                return sec as *mut IMAGE_SECTION_HEADER;
            }
            e -= 1;
        }
        sec = sec.add(1);
        i += 1;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn _GetPEImageBase() -> PBYTE {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return core::ptr::null_mut();
    }
    base
}

#[no_mangle]
pub unsafe extern "C" fn _IsNonwritableInCurrentImage(pTarget: PBYTE) -> WINBOOL {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return FALSE;
    }
    let rva = pTarget.offset_from(base) as DWORD_PTR;
    let sec = _FindPESection(base, rva);
    if sec.is_null() {
        return FALSE;
    }
    ((*sec).Characteristics & IMAGE_SCN_MEM_WRITE == 0) as WINBOOL
}

#[no_mangle]
pub unsafe extern "C" fn __mingw_enum_import_library_names(mut i: i32) -> *const u8 {
    let base = image_base();
    if _ValidateImageBase(base) == FALSE {
        return core::ptr::null();
    }
    let dos = base as *const IMAGE_DOS_HEADER;
    let nt = base.offset((*dos).e_lfanew as isize) as *const IMAGE_NT_HEADERS;
    let imports_rva =
        (*nt).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT].VirtualAddress;
    if imports_rva == 0 {
        return core::ptr::null();
    }
    if _FindPESection(base, imports_rva as DWORD_PTR).is_null() {
        return core::ptr::null();
    }
    let mut desc = base.offset(imports_rva as isize) as *const IMAGE_IMPORT_DESCRIPTOR;
    loop {
        if (*desc).TimeDateStamp == 0 && (*desc).Name == 0 {
            break;
        }
        if i <= 0 {
            return base.offset((*desc).Name as isize) as *const u8;
        }
        i -= 1;
        desc = desc.add(1);
    }
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn __mingw_get_msvcrt_handle() -> HMODULE {
    static mut MSVCRT_HANDLE: HANDLE = core::ptr::null_mut();
    if MSVCRT_HANDLE.is_null() {
        let mut lib_name: *const u8 = core::ptr::null();
        let mut i = 0i32;
        loop {
            lib_name = __mingw_enum_import_library_names(i);
            i += 1;
            if lib_name.is_null() {
                break;
            }
            let b = |k: usize| *lib_name.add(k);
            let m = |c: u8, u: u8, l: u8| c == u || c == l;
            if m(b(0), b'm', b'M')
                && m(b(1), b's', b'S')
                && m(b(2), b'v', b'V')
                && m(b(3), b'c', b'C')
                && m(b(4), b'r', b'R')
                && (m(b(5), b't', b'T') || (b(5) >= b'0' && b(5) <= b'9'))
            {
                break;
            }
        }
        if !lib_name.is_null() {
            MSVCRT_HANDLE = GetModuleHandleA(lib_name);
        }
        if MSVCRT_HANDLE.is_null() {
            // L"msvcrt.dll"
            const NAME: [u16; 11] = [
                b'm' as u16,
                b's' as u16,
                b'v' as u16,
                b'c' as u16,
                b'r' as u16,
                b't' as u16,
                b'.' as u16,
                b'd' as u16,
                b'l' as u16,
                b'l' as u16,
                0,
            ];
            MSVCRT_HANDLE = LoadLibraryW(NAME.as_ptr());
        }
    }
    MSVCRT_HANDLE
}
