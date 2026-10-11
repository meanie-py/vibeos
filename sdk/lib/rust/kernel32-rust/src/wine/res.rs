use winedebug_rust::{ERR, TRACE};
use ntdll_sys::nt::*;
use core::ffi::c_void;

use crate::string::nls_subst::{HeapAlloc, HeapFree, HeapReAlloc};

// ---- logging macros (no-op; established sibling convention from comm.rs) ----
// debugstr_a/_w only ever appear inside the (no-op) TRACE/ERR/WARN args, so
// they can be no-op macros too — faithful to the C, evaluates nothing.
macro_rules! debugstr_a { ($($t:tt)*) => {{ () }} }
macro_rules! debugstr_w { ($($t:tt)*) => {{ () }} }

// ---- win32 macro helpers (faithful to winnt.h / winbase.h) ----
// IS_INTRESOURCE: returns BOOL (i32) to match the c2rust `== 0` / `!= 0`
// integer-comparison call sites. Accepts any pointer/integer via `as usize`.
macro_rules! IS_INTRESOURCE {
    ($x:expr) => {
        (((($x) as usize) >> 16) == 0) as BOOL
    };
}

macro_rules! LOWORD {
    ($x:expr) => { ((($x) as usize) & 0xffff) as WORD };
}
macro_rules! HIWORD {
    ($x:expr) => { (((($x) as usize) >> 16) & 0xffff) as WORD };
}
macro_rules! ULongToPtr {
    ($x:expr) => { (($x) as usize) as *mut c_void };
}
macro_rules! UIntToPtr {
    ($x:expr) => { (($x) as usize) as *mut c_void };
}
macro_rules! MAKELANGID {
    ($p:expr, $s:expr) => { (((($s) as WORD) << 10) | (($p) as WORD)) as WORD };
}

const CP_ACP: UINT = 0;
const INVALID_HANDLE_VALUE: isize = -1;

// wine/list.h: the shared port (its LIST_FOR_EACH_ENTRY runs its step on `continue`, which
// this file's own copy had learned from UpdateResource hanging on its second resource).
use winelist_rust::{
    list, list_add_before, list_add_tail, list_head, list_init, list_remove, LIST_ENTRY,
    LIST_FOR_EACH_ENTRY,
};

extern "system" {
    fn GetModuleHandleA(lpModuleName: *const u8) -> HINSTANCE;
    fn SetLastError(dwErrCode: u32);
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
}

// ---- IMAGE_RESOURCE_DIRECTORY_ENTRY bitfield/union accessors ----------------
// winnt.h declares NameIsString/NameOffset/OffsetToDirectory/DataIsDirectory as
// :31/:1 bitfields and Name/Id/OffsetToData as union members; bindgen lowers
// these into nested anonymous unions + accessor methods. These thin helpers map
// the flat C field syntax used by res.rs onto the bindgen layout (faithful, no
// behavior change).
#[inline]
unsafe fn rde_name_is_string(e: *const IMAGE_RESOURCE_DIRECTORY_ENTRY) -> DWORD {
    (*e).__bindgen_anon_1.__bindgen_anon_1.NameIsString()
}
#[inline]
unsafe fn rde_set_name_is_string(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: DWORD) {
    (*e).__bindgen_anon_1.__bindgen_anon_1.set_NameIsString(v)
}
#[inline]
unsafe fn rde_name_offset(e: *const IMAGE_RESOURCE_DIRECTORY_ENTRY) -> DWORD {
    (*e).__bindgen_anon_1.__bindgen_anon_1.NameOffset()
}
#[inline]
unsafe fn rde_set_name_offset(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: DWORD) {
    (*e).__bindgen_anon_1.__bindgen_anon_1.set_NameOffset(v)
}
#[inline]
unsafe fn rde_id(e: *const IMAGE_RESOURCE_DIRECTORY_ENTRY) -> WORD {
    (*e).__bindgen_anon_1.Id
}
#[inline]
unsafe fn rde_set_id(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: WORD) {
    (*e).__bindgen_anon_1.Id = v;
}
#[inline]
unsafe fn rde_offset_to_directory(e: *const IMAGE_RESOURCE_DIRECTORY_ENTRY) -> DWORD {
    (*e).__bindgen_anon_2.__bindgen_anon_1.OffsetToDirectory()
}
#[inline]
unsafe fn rde_set_offset_to_directory(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: DWORD) {
    (*e).__bindgen_anon_2.__bindgen_anon_1.set_OffsetToDirectory(v)
}
#[inline]
unsafe fn rde_set_data_is_directory(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: DWORD) {
    (*e).__bindgen_anon_2.__bindgen_anon_1.set_DataIsDirectory(v)
}
#[inline]
unsafe fn rde_offset_to_data(e: *const IMAGE_RESOURCE_DIRECTORY_ENTRY) -> DWORD {
    (*e).__bindgen_anon_2.OffsetToData
}
#[inline]
unsafe fn rde_set_offset_to_data(e: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY, v: DWORD) {
    (*e).__bindgen_anon_2.OffsetToData = v;
}

// ---- kernel32 / win32 API exports used by res.rs (faithful Win32 sigs) ----
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn lstrlenW(lpString: LPCWSTR) -> INT;
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    fn lstrcmpW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT,
        lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut BOOL,
    ) -> INT;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn CreateFileW(
        lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void, dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn CreateFileMappingW(
        hFile: HANDLE, lpFileMappingAttributes: *mut c_void, flProtect: DWORD,
        dwMaximumSizeHigh: DWORD, dwMaximumSizeLow: DWORD, lpName: LPCWSTR,
    ) -> HANDLE;
    fn MapViewOfFile(
        hFileMappingObject: HANDLE, dwDesiredAccess: DWORD,
        dwFileOffsetHigh: DWORD, dwFileOffsetLow: DWORD, dwNumberOfBytesToMap: SIZE_T,
    ) -> LPVOID;
    fn UnmapViewOfFile(lpBaseAddress: *const c_void) -> BOOL;
    fn CopyFileW(lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL) -> BOOL;
    fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: *mut DWORD) -> DWORD;
    fn SetFilePointer(hFile: HANDLE, lDistanceToMove: i32, lpDistanceToMoveHigh: *mut i32, dwMoveMethod: DWORD) -> DWORD;
    fn SetEndOfFile(hFile: HANDLE) -> BOOL;
    fn GetTempPathW(nBufferLength: DWORD, lpBuffer: LPWSTR) -> DWORD;
    fn GetTempFileNameW(lpPathName: LPCWSTR, lpPrefixString: LPCWSTR, uUnique: UINT, lpTempFileName: LPWSTR) -> UINT;
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
}

// ---- CRT functions (cdecl — MUST be extern "C" for i386 link) ----
extern "C" {
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> INT;
    fn memset(s: *mut c_void, c: INT, n: usize) -> *mut c_void;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct QUEUEDUPDATES {
    pub pFileName: LPWSTR,
    pub bDeleteExistingResources: BOOL,
    pub root: list,
}

#[repr(C)]
pub struct resource_dir_entry {
    pub entry: list,
    pub id: LPWSTR,
    pub children: list,
}

#[repr(C)]
pub struct resource_data {
    pub entry: list,
    pub lang: LANGID,
    pub codepage: DWORD,
    pub cbData: DWORD,
    pub lpData: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct resource_size_info {
    pub types_ofs: DWORD,
    pub names_ofs: DWORD,
    pub langs_ofs: DWORD,
    pub data_entry_ofs: DWORD,
    pub strings_ofs: DWORD,
    pub data_ofs: DWORD,
    pub total_size: DWORD,
}

#[repr(C)]
pub struct mapping_info {
    pub file: HANDLE,
    pub base: *mut core::ffi::c_void,
    pub size: DWORD,
    pub read_write: BOOL,
}
unsafe fn get_res_nameA(name: LPCSTR, str: *mut _UNICODE_STRING) -> NTSTATUS {
    if IS_INTRESOURCE!(name) != 0 {
        (*str).Buffer = ULongToPtr!(LOWORD!(name)) as LPWSTR;
        return STATUS_SUCCESS;
    }
    if *name.offset(0) as u8 == b'#' {
        let mut value: ULONG = 0;
        if RtlCharToInteger(name.offset(1), 10, &mut value) != STATUS_SUCCESS || HIWORD!(value) != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        (*str).Buffer = ULongToPtr!(value) as LPWSTR;
        return STATUS_SUCCESS;
    }
    RtlCreateUnicodeStringFromAsciiz(str, name);
    RtlUpcaseUnicodeString(str, str, 0);
    return STATUS_SUCCESS;
}
unsafe fn get_res_nameW( name: LPCWSTR, str: *mut _UNICODE_STRING ) -> NTSTATUS {
    if IS_INTRESOURCE!(name) != 0 {
        (*str).Buffer = ULongToPtr!(LOWORD!(name)) as LPWSTR;
        return STATUS_SUCCESS;
    }
    if *name.offset(0) as u16 == '#' as u16 {
        let mut value: ULONG = 0;
        RtlInitUnicodeString(str, name.offset(1));
        if RtlUnicodeStringToInteger(str, 10, &mut value) != STATUS_SUCCESS || HIWORD!(value) != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        (*str).Buffer = ULongToPtr!(value) as LPWSTR;
        return STATUS_SUCCESS;
    }
    RtlCreateUnicodeString(str, name);
    RtlUpcaseUnicodeString(str, str, 0);
    return STATUS_SUCCESS;
}
unsafe fn find_resourceA(
    hModule: HMODULE,
    type_: LPCSTR,
    name: LPCSTR,
    lang: WORD,
) -> HRSRC {
    let mut status: NTSTATUS;
    let mut nameW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut entry: *mut IMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();

    nameW.Buffer = core::ptr::null_mut();
    typeW.Buffer = core::ptr::null_mut();

    // __TRY block (DUMMY_PSEH: runs unconditionally, __EXCEPT is dead code)
    'done: {
        status = get_res_nameA(name, &mut nameW);
        if status != STATUS_SUCCESS {
            break 'done;
        }
        status = get_res_nameA(type_, &mut typeW);
        if status != STATUS_SUCCESS {
            break 'done;
        }
        info.Type = typeW.Buffer as ULONG_PTR;
        info.Name = nameW.Buffer as ULONG_PTR;
        info.Language = lang as ULONG_PTR;
        status = LdrFindResource_U(hModule as PVOID, &mut info, 3, &mut entry);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }

    // __EXCEPT_PAGE_FAULT handler
    // In Rust we cannot have SEH, so we wrap the try block in a way that
    // catches access violations. For a faithful transliteration, we keep
    // the structure but note that actual SEH is not available.
    // The page fault handler would set ERROR_INVALID_PARAMETER.
    // Since we cannot catch page faults in Rust, we just proceed.
    // The original code would catch STATUS_ACCESS_VIOLATION and set
    // ERROR_INVALID_PARAMETER. We simulate this by checking if entry is null
    // after the call, but that's not exactly the same. For now, we just
    // keep the structure as a comment.
    // __EXCEPT_PAGE_FAULT:
    // SetLastError(ERROR_INVALID_PARAMETER);

    // get_res_nameA allocated a string name (RtlCreateUnicodeStringFromAsciiz);
    // an integer one is the id itself.
    if IS_INTRESOURCE!(nameW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, nameW.Buffer as *mut c_void);
    }
    if IS_INTRESOURCE!(typeW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut c_void);
    }
    return entry as HRSRC;
}
unsafe fn find_resourceW(
    hModule: HMODULE,
    r#type: LPCWSTR,
    name: LPCWSTR,
    lang: WORD,
) -> HRSRC {
    let mut status: NTSTATUS;
    let mut nameW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut entry: *mut IMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();

    nameW.Buffer = core::ptr::null_mut();
    typeW.Buffer = core::ptr::null_mut();

    // __TRY block (DUMMY_PSEH: runs unconditionally, __EXCEPT is dead code)
    'done: {
        status = get_res_nameW(name, &mut nameW);
        if status != STATUS_SUCCESS {
            break 'done;
        }
        status = get_res_nameW(r#type, &mut typeW);
        if status != STATUS_SUCCESS {
            break 'done;
        }
        info.Type = typeW.Buffer as ULONG_PTR;
        info.Name = nameW.Buffer as ULONG_PTR;
        info.Language = lang as ULONG_PTR;
        status = LdrFindResource_U(hModule as PVOID, &mut info, 3, &mut entry);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }

    if IS_INTRESOURCE!(nameW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, nameW.Buffer as *mut c_void);
    }
    if IS_INTRESOURCE!(typeW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut c_void);
    }
    entry as HRSRC
}

#[no_mangle]
pub unsafe extern "system" fn FindResourceExA(
    hModule: HMODULE,
    type_: LPCSTR,
    name: LPCSTR,
    lang: WORD,
) -> HRSRC {

    TRACE!(
        b"%p %s %s %04x\0".as_ptr() as *const u8,
        hModule,
        debugstr_a!(type_),
        debugstr_a!(name),
        lang,
    );

    let mut hModule = hModule;
    if hModule.is_null() {
        hModule = GetModuleHandleW(core::ptr::null_mut());
    }
    find_resourceA(hModule, type_, name, lang)
}

#[no_mangle]
pub unsafe extern "system" fn FindResourceA(
    hModule: HMODULE,
    name: LPCSTR,
    type_: LPCSTR,
) -> HRSRC {
    return FindResourceExA(
        hModule,
        type_,
        name,
        MAKELANGID!(LANG_NEUTRAL as WORD, SUBLANG_NEUTRAL as WORD),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FindResourceExW(
    hModule: HMODULE,
    r#type: LPCWSTR,
    name: LPCWSTR,
    lang: WORD,
) -> HRSRC {

    TRACE!(
        b"%p %s %s %04x\0".as_ptr() as *const u8,
        hModule,
        debugstr_w!(r#type),
        debugstr_w!(name),
        lang,
    );

    let mut hModule = hModule;
    if hModule.is_null() {
        hModule = GetModuleHandleW(core::ptr::null());
    }
    find_resourceW(hModule, r#type, name, lang)
}

#[no_mangle]
pub unsafe extern "system" fn FindResourceW(
    hModule: HINSTANCE,
    name: LPCWSTR,
    r#type: LPCWSTR,
) -> HRSRC {
    return FindResourceExW(
        hModule,
        r#type,
        name,
        MAKELANGID!(LANG_NEUTRAL as WORD, SUBLANG_NEUTRAL as WORD),
    );
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceTypesA(
    hmod: HMODULE,
    lpfun: ENUMRESTYPEPROCA,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32;
    let mut ret: BOOL = FALSE as BOOL;
    let mut type_: LPSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut newlen: DWORD;
    let mut status: NTSTATUS;
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut str_: *const IMAGE_RESOURCE_DIR_STRING_U;

    TRACE!("%p %p %lx\n", hmod, lpfun, lparam);

    let mut hmod = hmod;
    if hmod.is_null() {
        hmod = GetModuleHandleA(core::ptr::null_mut());
    }

    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut resdir);
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
        return FALSE as BOOL;
    }
    et = resdir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    i = 0;
    while i < ((*resdir).NumberOfNamedEntries as i32 + (*resdir).NumberOfIdEntries as i32) {
        if rde_name_is_string(et.add(i as usize)) != 0 {
            str_ = (resdir as *const u8)
                .offset(rde_name_offset(et.add(i as usize)) as isize)
                as *const IMAGE_RESOURCE_DIR_STRING_U;
            newlen = WideCharToMultiByte(
                CP_ACP,
                0,
                (*str_).NameString.as_ptr(),
                (*str_).Length as i32,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            if newlen + 1 > len {
                len = newlen + 1;
                HeapFree(GetProcessHeap(), 0, type_ as *mut _);
                type_ = HeapAlloc(GetProcessHeap(), 0, len as usize) as LPSTR;
                if type_.is_null() {
                    return FALSE as BOOL;
                }
            }
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*str_).NameString.as_ptr(),
                (*str_).Length as i32,
                type_,
                len as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            *type_.add(newlen as usize) = 0;
            ret = lpfun.unwrap()(hmod, type_, lparam);
        } else {
            ret = lpfun.unwrap()(hmod, UIntToPtr!(rde_id(et.add(i as usize))) as LPSTR, lparam);
        }
        if ret == 0 {
            break;
        }
        i += 1;
    }
    HeapFree(GetProcessHeap(), 0, type_ as *mut _);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceTypesW(
    hmod: HMODULE,
    lpfun: ENUMRESTYPEPROCW,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32 = 0;
    let mut len: i32 = 0;
    let mut ret: BOOL = 0;
    let mut r#type: LPWSTR = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut str_: *const IMAGE_RESOURCE_DIR_STRING_U;

    TRACE!(b"%p %p %lx\0".as_ptr() as *const i8, hmod, lpfun, lparam);

    let mut hmod = hmod;
    if hmod.is_null() {
        hmod = GetModuleHandleW(core::ptr::null_mut());
    }

    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut resdir);
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
        return 0;
    }
    et = resdir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    i = 0;
    while i < ((*resdir).NumberOfNamedEntries as i32 + (*resdir).NumberOfIdEntries as i32) {
        if rde_name_is_string(et.offset(i as isize)) != 0 {
            str_ = (resdir as *const u8)
                .offset(rde_name_offset(et.offset(i as isize)) as isize)
                as *const IMAGE_RESOURCE_DIR_STRING_U;
            if (*str_).Length as i32 + 1 > len {
                len = (*str_).Length as i32 + 1;
                HeapFree(GetProcessHeap(), 0, r#type as *mut core::ffi::c_void);
                r#type = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    (len as u32 * core::mem::size_of::<WCHAR>() as u32) as usize,
                ) as LPWSTR;
                if r#type.is_null() {
                    return 0;
                }
            }
            memcpy(
                r#type as *mut core::ffi::c_void,
                (*str_).NameString.as_ptr() as *const core::ffi::c_void,
                (*str_).Length as usize * core::mem::size_of::<WCHAR>(),
            );
            *r#type.offset((*str_).Length as isize) = 0;
            ret = lpfun.unwrap()(hmod, r#type, lparam);
        } else {
            ret = lpfun.unwrap()(
                hmod,
                UIntToPtr!(rde_id(et.offset(i as isize))) as LPWSTR,
                lparam,
            );
        }
        if ret == 0 {
            break;
        }
        i += 1;
    }
    HeapFree(GetProcessHeap(), 0, r#type as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceNamesA(
    hmod: HMODULE,
    type_: LPCSTR,
    lpfun: ENUMRESNAMEPROCA,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32;
    let mut ret: BOOL = 0;
    let mut len: DWORD = 0;
    let mut newlen: DWORD;
    let mut name: LPSTR = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut basedir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut str_: *const IMAGE_RESOURCE_DIR_STRING_U;

    TRACE!(
        "%p %s %p %lx\n",
        hmod,
        debugstr_a!(type_),
        lpfun,
        lparam,
    );

    let hmod = if hmod.is_null() {
        GetModuleHandleA(core::ptr::null())
    } else {
        hmod
    };
    typeW.Buffer = core::ptr::null_mut();
    'done: {
    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut basedir);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameA(type_, &mut typeW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    info.Type = typeW.Buffer as ULONG_PTR;
    status = LdrFindResourceDirectory_U(hmod as PVOID, &mut info, 1, &mut resdir);
    if status != STATUS_SUCCESS {
        break 'done;
    }

    et = resdir.add(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    // DUMMY_PSEH: the _SEH2_TRY body runs unconditionally; the _SEH2_EXCEPT
    // handler (ret=0; status=ACCESS_VIOLATION) is dead code under USE_DUMMY_PSEH.
    {
            for i in 0..((*resdir).NumberOfNamedEntries + (*resdir).NumberOfIdEntries) as i32 {
                if rde_name_is_string(et.add(i as usize)) != 0 {
                    str_ = ((basedir as *const u8).add(rde_name_offset(et.add(i as usize)) as usize))
                        as *const IMAGE_RESOURCE_DIR_STRING_U;
                    newlen = WideCharToMultiByte(
                        CP_ACP,
                        0,
                        (*str_).NameString.as_ptr(),
                        (*str_).Length as i32,
                        core::ptr::null_mut(),
                        0,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    ) as DWORD;
                    if newlen + 1 > len {
                        len = newlen + 1;
                        HeapFree(GetProcessHeap(), 0, name as *mut _);
                        name = HeapAlloc(GetProcessHeap(), 0, (len + 1) as usize) as LPSTR;
                        if name.is_null() {
                            ret = 0;
                            break 'done;
                        }
                    }
                    WideCharToMultiByte(
                        CP_ACP,
                        0,
                        (*str_).NameString.as_ptr(),
                        (*str_).Length as i32,
                        name,
                        len as i32,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    *name.add(newlen as usize) = 0;
                    ret = lpfun.unwrap()(hmod, type_, name, lparam);
                } else {
                    ret = lpfun.unwrap()(
                        hmod,
                        type_,
                        UIntToPtr!(rde_id(et.add(i as usize))) as LPSTR,
                        lparam,
                    );
                }
                if ret == 0 {
                    break;
                }
            }
    }
    } // 'done

    if !typeW.Buffer.is_null() && IS_INTRESOURCE!(typeW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut _);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceNamesW(
    hmod: HMODULE,
    r#type: LPCWSTR,
    lpfun: ENUMRESNAMEPROCW,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32 = 0;
    let mut len: i32 = 0;
    let mut ret: BOOL = 0;
    let mut name: LPWSTR = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut basedir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut str: *const IMAGE_RESOURCE_DIR_STRING_U;

    TRACE!(
        "%p %s %p %lx\0".as_ptr() as *const i8,
        hmod,
        debugstr_w!(r#type),
        lpfun,
        lparam,
    );

    let mut hmod = hmod;
    if hmod.is_null() {
        hmod = GetModuleHandleW(core::ptr::null());
    }
    typeW.Buffer = core::ptr::null_mut();
    'done: {
    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut basedir);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameW(r#type, &mut typeW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    info.Type = typeW.Buffer as ULONG_PTR;
    status = LdrFindResourceDirectory_U(hmod as PVOID, &mut info, 1, &mut resdir);
    if status != STATUS_SUCCESS {
        break 'done;
    }

    et = (resdir.offset(1)) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    // DUMMY_PSEH: __TRY runs unconditionally, __EXCEPT is dead code.
    {
        {
            i = 0;
            while i < ((*resdir).NumberOfNamedEntries + (*resdir).NumberOfIdEntries) as i32 {
                if rde_name_is_string(et.offset(i as isize)) != 0 {
                    str = ((basedir as *const u8)
                        .offset(rde_name_offset((et.offset(i as isize))) as isize))
                        as *const IMAGE_RESOURCE_DIR_STRING_U;
                    if ((*str).Length as i32 + 1) > len {
                        len = (*str).Length as i32 + 1;
                        HeapFree(GetProcessHeap(), 0, name as *mut core::ffi::c_void);
                        name = HeapAlloc(
                            GetProcessHeap(),
                            0,
                            (len as u32 * core::mem::size_of::<WCHAR>() as u32) as usize,
                        ) as LPWSTR;
                        if name.is_null() {
                            ret = 0;
                            break;
                        }
                    }
                    memcpy(
                        name as *mut core::ffi::c_void,
                        (*str).NameString.as_ptr() as *const core::ffi::c_void,
                        ((*str).Length as u32 * core::mem::size_of::<WCHAR>() as u32) as usize,
                    );
                    *name.offset((*str).Length as isize) = 0;
                    ret = lpfun.unwrap()(hmod, r#type, name, lparam);
                } else {
                    ret = lpfun.unwrap()(
                        hmod,
                        r#type,
                        UIntToPtr!(rde_id(et.offset(i as isize))) as LPWSTR,
                        lparam,
                    );
                }
                if ret == 0 {
                    break;
                }
                i += 1;
            }
        }
    }
    } // 'done
    // __EXCEPT_PAGE_FAULT handler is dead code under USE_DUMMY_PSEH.
    if !(typeW.Buffer as usize >> 16 == 0) {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut core::ffi::c_void);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceLanguagesA(
    hmod: HMODULE,
    type_: LPCSTR,
    name: LPCSTR,
    lpfun: ENUMRESLANGPROCA,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32;
    let mut ret: BOOL = 0;
    let mut status: NTSTATUS;
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut nameW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut basedir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;

    TRACE!(
        "%p %s %s %p %lx\0".as_ptr() as *const i8,
        hmod,
        debugstr_a!(type_),
        debugstr_a!(name),
        lpfun,
        lparam,
    );

    let hmod = if hmod.is_null() {
        GetModuleHandleA(core::ptr::null())
    } else {
        hmod
    };
    typeW.Buffer = core::ptr::null_mut();
    nameW.Buffer = core::ptr::null_mut();
    'done: {
    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut basedir);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameA(type_, &mut typeW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameA(name, &mut nameW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    info.Type = typeW.Buffer as ULONG_PTR;
    info.Name = nameW.Buffer as ULONG_PTR;
    status = LdrFindResourceDirectory_U(hmod as PVOID, &mut info, 2, &mut resdir);
    if status != STATUS_SUCCESS {
        break 'done;
    }

    et = (resdir.offset(1)) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    // __TRY (DUMMY_PSEH: runs unconditionally, __EXCEPT is dead code)
    {
        i = 0;
        while i < ((*resdir).NumberOfNamedEntries + (*resdir).NumberOfIdEntries) as i32 {
            ret = lpfun.unwrap()(
                hmod,
                type_,
                name,
                rde_id(et.offset(i as isize)),
                lparam,
            );
            if ret == 0 {
                break;
            }
            i += 1;
        }
    }
    } // 'done

    if IS_INTRESOURCE!(nameW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, nameW.Buffer as *mut _);
    }
    if IS_INTRESOURCE!(typeW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut _);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn EnumResourceLanguagesW(
    hmod: HMODULE,
    type_: LPCWSTR,
    name: LPCWSTR,
    lpfun: ENUMRESLANGPROCW,
    lparam: LONG_PTR,
) -> BOOL {
    let mut i: i32;
    let mut ret: BOOL = 0;
    let mut status: NTSTATUS;
    let mut typeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut nameW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut info: LDR_RESOURCE_INFO = unsafe { core::mem::zeroed() };
    let mut basedir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let mut resdir: *mut IMAGE_RESOURCE_DIRECTORY = core::ptr::null_mut();
    let et: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;

    TRACE!(
        "%p %s %s %p %lx\0".as_ptr() as *const i8,
        hmod,
        debugstr_w!(type_),
        debugstr_w!(name),
        lpfun,
        lparam,
    );

    let mut hmod = hmod;
    if hmod.is_null() {
        hmod = GetModuleHandleW(core::ptr::null_mut());
    }
    typeW.Buffer = core::ptr::null_mut();
    nameW.Buffer = core::ptr::null_mut();
    'done: {
    status = LdrFindResourceDirectory_U(hmod as PVOID, core::ptr::null_mut(), 0, &mut basedir);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameW(type_, &mut typeW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    status = get_res_nameW(name, &mut nameW);
    if status != STATUS_SUCCESS {
        break 'done;
    }
    info.Type = typeW.Buffer as ULONG_PTR;
    info.Name = nameW.Buffer as ULONG_PTR;
    status = LdrFindResourceDirectory_U(hmod as PVOID, &mut info, 2, &mut resdir);
    if status != STATUS_SUCCESS {
        break 'done;
    }

    et = (resdir.offset(1)) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    // __TRY (DUMMY_PSEH: runs unconditionally, __EXCEPT is dead code)
    {
        i = 0;
        while i < ((*resdir).NumberOfNamedEntries + (*resdir).NumberOfIdEntries) as i32 {
            ret = lpfun.unwrap()(
                hmod,
                type_,
                name,
                rde_id(et.offset(i as isize)),
                lparam,
            );
            if ret == 0 {
                break;
            }
            i += 1;
        }
    }
    } // 'done

    if IS_INTRESOURCE!(typeW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, typeW.Buffer as *mut _);
    }
    if IS_INTRESOURCE!(nameW.Buffer) == 0 {
        HeapFree(GetProcessHeap(), 0, nameW.Buffer as *mut _);
    }
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn LoadResource(
    hModule: HINSTANCE,
    hRsrc: HRSRC,
) -> HGLOBAL {
    let status: NTSTATUS;
    let mut ret: *mut core::ffi::c_void = core::ptr::null_mut();

    // TRACE!( "%p %p\n", hModule, hRsrc );

    if hRsrc.is_null() {
        return core::ptr::null_mut();
    }
    let hModule = if hModule.is_null() {
        GetModuleHandleA(core::ptr::null())
    } else {
        hModule
    };
    status = LdrAccessResource(
        hModule as PVOID,
        hRsrc as *mut IMAGE_RESOURCE_DATA_ENTRY,
        &mut ret,
        core::ptr::null_mut(),
    );
    if status != STATUS_SUCCESS {
        SetLastError(RtlNtStatusToDosError(status));
    }
    ret as HGLOBAL
}

#[no_mangle]
pub unsafe extern "system" fn LockResource(handle: HGLOBAL) -> LPVOID {
    return handle;
}

#[no_mangle]
pub unsafe extern "system" fn FreeResource(handle: HGLOBAL) -> BOOL {
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn SizeofResource(
    hModule: HINSTANCE,
    hRsrc: HRSRC,
) -> DWORD {
    if hRsrc.is_null() {
        return 0;
    }
    return (*(hRsrc as *const IMAGE_RESOURCE_DATA_ENTRY)).Size;
}
unsafe fn resource_strcmp(a: LPCWSTR, b: LPCWSTR) -> i32 {
    if a == b {
        return 0;
    }
    if IS_INTRESOURCE!(a) == 0 && IS_INTRESOURCE!(b) == 0 {
        return lstrcmpW(a, b);
    }
    /* strings come before ids */
    if IS_INTRESOURCE!(a) == 0 && IS_INTRESOURCE!(b) != 0 {
        return -1;
    }
    if IS_INTRESOURCE!(b) == 0 && IS_INTRESOURCE!(a) != 0 {
        return 1;
    }
    return if a < b { -1 } else { 1 };
}
unsafe fn find_resource_dir_entry(
    dir: *mut list,
    id: LPCWSTR,
) -> *mut resource_dir_entry {
    let mut ent: *mut resource_dir_entry;

    LIST_FOR_EACH_ENTRY!(ent, dir, resource_dir_entry, entry, {
        if resource_strcmp(id, (*ent).id) == 0 {
            return ent;
        }
    });

    return core::ptr::null_mut();
}

unsafe fn find_resource_data(dir: *mut list, lang: LANGID) -> *mut resource_data {
    let mut res_data: *mut resource_data;

    /* match only languages here */
    LIST_FOR_EACH_ENTRY!(res_data, dir, resource_data, entry, {
        if lang == (*res_data).lang {
            return res_data;
        }
    });

    return core::ptr::null_mut();
}
unsafe fn add_resource_dir_entry(dir: *mut list, resdir: *mut resource_dir_entry) {
    let mut ent: *mut resource_dir_entry;
    LIST_FOR_EACH_ENTRY!(ent, dir, resource_dir_entry, entry, {
        if 0 > resource_strcmp((*ent).id, (*resdir).id) {
            continue;
        }
        list_add_before(&mut (*ent).entry, &mut (*resdir).entry);
        return;
    });
    list_add_tail(dir, &mut (*resdir).entry);
}
unsafe fn add_resource_data_entry(dir: *mut list, resdata: *mut resource_data) {
    let mut ent: *mut resource_data;
    LIST_FOR_EACH_ENTRY!(ent, dir, resource_data, entry, {
        if (*ent).lang < (*resdata).lang {
            continue;
        }
        list_add_before(&mut (*ent).entry, &mut (*resdata).entry);
        return;
    });
    list_add_tail(dir, &mut (*resdata).entry);
}
unsafe fn res_strdupW(str: LPCWSTR) -> LPWSTR {
    let ret: LPWSTR;
    let len: UINT;

    if IS_INTRESOURCE!(str) != 0 {
        return LOWORD!(str) as UINT_PTR as LPWSTR;
    }
    len = (lstrlenW(str) + 1) as UINT * core::mem::size_of::<WCHAR>() as UINT;
    ret = HeapAlloc(GetProcessHeap(), 0, len as usize) as LPWSTR;
    memcpy(ret as *mut core::ffi::c_void, str as *const core::ffi::c_void, len as usize);
    return ret;
}
unsafe fn res_free_str(str: LPWSTR) {
    if IS_INTRESOURCE!(str) == 0 {
        HeapFree(GetProcessHeap(), 0, str as *mut _);
    }
}
unsafe fn update_add_resource(
    updates: *mut QUEUEDUPDATES,
    Type: LPCWSTR,
    Name: LPCWSTR,
    Lang: LANGID,
    resdata: *mut resource_data,
    overwrite_existing: BOOL,
) -> BOOL {
    let mut restype: *mut resource_dir_entry;
    let mut resname: *mut resource_dir_entry;
    let existing: *mut resource_data;

    TRACE!(
        "%p %s %s %p %d\n",
        updates,
        debugstr_w!(Type),
        debugstr_w!(Name),
        resdata,
        overwrite_existing,
    );

    restype = find_resource_dir_entry(&mut (*updates).root, Type);
    if restype.is_null() {
        restype = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<resource_dir_entry>()) as *mut resource_dir_entry;
        (*restype).id = res_strdupW(Type);
        list_init(&mut (*restype).children);
        add_resource_dir_entry(&mut (*updates).root, restype);
    }

    resname = find_resource_dir_entry(&mut (*restype).children, Name);
    if resname.is_null() {
        resname = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<resource_dir_entry>()) as *mut resource_dir_entry;
        (*resname).id = res_strdupW(Name);
        list_init(&mut (*resname).children);
        add_resource_dir_entry(&mut (*restype).children, resname);
    }

    existing = find_resource_data(&mut (*resname).children, Lang);
    if !existing.is_null() {
        if overwrite_existing == 0 {
            return 0;
        }
        list_remove(&mut (*existing).entry);
        HeapFree(GetProcessHeap(), 0, existing as *mut core::ffi::c_void);
    }

    if !resdata.is_null() {
        add_resource_data_entry(&mut (*resname).children, resdata);
    }

    return 1;
}
unsafe fn allocate_resource_data(
    Language: WORD,
    codepage: DWORD,
    lpData: LPVOID,
    cbData: DWORD,
    copy_data: BOOL,
) -> *mut resource_data {
    let resdata: *mut resource_data;

    if lpData.is_null() || cbData == 0 {
        return core::ptr::null_mut();
    }

    resdata = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<resource_data>() + if copy_data != 0 { cbData as usize } else { 0 },
    ) as *mut resource_data;
    if !resdata.is_null() {
        (*resdata).lang = Language;
        (*resdata).codepage = codepage;
        (*resdata).cbData = cbData;
        if copy_data != 0 {
            (*resdata).lpData = resdata.add(1) as LPVOID;
            memcpy((*resdata).lpData, lpData, cbData as usize);
        } else {
            (*resdata).lpData = lpData;
        }
    }

    return resdata;
}
unsafe fn free_resource_directory(head: *mut list, level: i32) {
    let mut ptr: *mut list = core::ptr::null_mut();

    loop {
        ptr = list_head(head);
        if ptr.is_null() {
            break;
        }
        list_remove(ptr);
        if level != 0 {
            let ent: *mut resource_dir_entry = LIST_ENTRY!(ptr, resource_dir_entry, entry);
            res_free_str((*ent).id);
            free_resource_directory(&mut (*ent).children, level - 1);
            HeapFree(GetProcessHeap(), 0, ent as *mut _);
        } else {
            let data: *mut resource_data = LIST_ENTRY!(ptr, resource_data, entry);
            HeapFree(GetProcessHeap(), 0, data as *mut _);
        }
    }
}
unsafe fn get_nt_header(base: *mut core::ffi::c_void, mapping_size: DWORD) -> *mut IMAGE_NT_HEADERS {
    let nt: *mut IMAGE_NT_HEADERS;
    let dos: *mut IMAGE_DOS_HEADER;

    if mapping_size < core::mem::size_of::<IMAGE_DOS_HEADER>() as DWORD {
        return core::ptr::null_mut();
    }

    dos = base as *mut IMAGE_DOS_HEADER;
    if (*dos).e_magic != IMAGE_DOS_SIGNATURE as u16 {
        return core::ptr::null_mut();
    }

    if ((*dos).e_lfanew as DWORD + core::mem::size_of::<IMAGE_NT_HEADERS>() as DWORD) > mapping_size {
        return core::ptr::null_mut();
    }

    nt = (base as *mut u8).offset((*dos).e_lfanew as isize) as *mut IMAGE_NT_HEADERS;

    if (*nt).Signature != IMAGE_NT_SIGNATURE {
        return core::ptr::null_mut();
    }

    return nt;
}
unsafe fn get_section_header(
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
    num_sections: *mut DWORD,
) -> *mut IMAGE_SECTION_HEADER {
    let nt: *mut IMAGE_NT_HEADERS;
    let section_ofs: DWORD;

    nt = get_nt_header(base, mapping_size);
    if nt.is_null() {
        return core::ptr::null_mut();
    }

    /* check that we don't go over the end of the file accessing the sections */
    section_ofs = (core::mem::offset_of!(IMAGE_NT_HEADERS, OptionalHeader) as DWORD)
        + (*nt).FileHeader.SizeOfOptionalHeader as DWORD;
    if ((*nt).FileHeader.NumberOfSections as DWORD)
        .wrapping_mul(core::mem::size_of::<IMAGE_SECTION_HEADER>() as DWORD)
        .wrapping_add(section_ofs)
        > mapping_size
    {
        return core::ptr::null_mut();
    }

    if !num_sections.is_null() {
        *num_sections = (*nt).FileHeader.NumberOfSections as DWORD;
    }

    /* from here we have a valid PE exe to update */
    return (nt as *mut u8).offset(section_ofs as isize) as *mut IMAGE_SECTION_HEADER;
}
unsafe fn check_pe_exe(file: HANDLE, updates: *mut QUEUEDUPDATES) -> BOOL {
    let nt: *const IMAGE_NT_HEADERS32;
    let nt64: *const IMAGE_NT_HEADERS64;
    let sec: *const IMAGE_SECTION_HEADER;
    let mut dd: *const IMAGE_DATA_DIRECTORY;
    let mut ret: BOOL = 0;
    let mapping: HANDLE;
    let mapping_size: DWORD;
    let mut num_sections: DWORD = 0;
    let mut base: *mut core::ffi::c_void = core::ptr::null_mut();

    mapping_size = GetFileSize(file, core::ptr::null_mut());

    'done: {
    mapping = CreateFileMappingW(file, core::ptr::null_mut(), PAGE_READONLY, 0, 0, core::ptr::null_mut());
    if mapping.is_null() {
        break 'done;
    }

    base = MapViewOfFile(mapping, FILE_MAP_READ, 0, 0, mapping_size as SIZE_T);
    if base.is_null() {
        break 'done;
    }

    nt = get_nt_header(base, mapping_size) as *const IMAGE_NT_HEADERS32;
    if nt.is_null() {
        break 'done;
    }

    nt64 = nt as *const IMAGE_NT_HEADERS64;
    dd = &(*nt).OptionalHeader.DataDirectory[0];
    if (*nt).OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16 {
        dd = &(*nt64).OptionalHeader.DataDirectory[0];
    }

    TRACE!(
        b"resources: %08x %08x\n\0".as_ptr() as *const i8,
        (*dd.offset(IMAGE_DIRECTORY_ENTRY_RESOURCE as isize)).VirtualAddress,
        (*dd.offset(IMAGE_DIRECTORY_ENTRY_RESOURCE as isize)).Size,
    );

    sec = get_section_header(base, mapping_size, &mut num_sections);
    if sec.is_null() {
        break 'done;
    }

    ret = 1;
    }

    if !base.is_null() {
        UnmapViewOfFile(base);
    }
    if !mapping.is_null() {
        CloseHandle(mapping);
    }

    return ret;
}
unsafe fn section_from_rva(
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
    rva: DWORD,
) -> *const IMAGE_SECTION_HEADER {
    let sec: *const IMAGE_SECTION_HEADER;
    let mut num_sections: DWORD = 0;
    let mut i: i32;

    sec = get_section_header(base, mapping_size, &mut num_sections);
    if sec.is_null() {
        return core::ptr::null();
    }

    i = (num_sections as i32) - 1;
    while i >= 0 {
        if (*sec.offset(i as isize)).VirtualAddress <= rva
            && rva <= (*sec.offset(i as isize)).VirtualAddress + (*sec.offset(i as isize)).SizeOfRawData
        {
            return &*sec.offset(i as isize);
        }
        i -= 1;
    }

    core::ptr::null()
}
unsafe fn address_from_rva(
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
    rva: DWORD,
    len: DWORD,
) -> *mut core::ffi::c_void {
    let sec: *const IMAGE_SECTION_HEADER;

    sec = section_from_rva(base, mapping_size, rva);
    if sec.is_null() {
        return core::ptr::null_mut();
    }

    if rva + len <= (*sec).VirtualAddress as DWORD + (*sec).SizeOfRawData {
        return (base as LPBYTE)
            .offset(((*sec).PointerToRawData + rva - (*sec).VirtualAddress) as isize)
            as *mut core::ffi::c_void;
    }

    return core::ptr::null_mut();
}
unsafe fn resource_dup_string(
    root: *const IMAGE_RESOURCE_DIRECTORY,
    entry: *const IMAGE_RESOURCE_DIRECTORY_ENTRY,
) -> LPWSTR {
    let string: *const IMAGE_RESOURCE_DIR_STRING_U;
    let s: LPWSTR;

    if rde_name_is_string(entry) == 0 {
        return UIntToPtr!(rde_id(entry) as usize) as LPWSTR;
    }

    string = (root as *const u8).offset(rde_name_offset(entry) as isize) as *const IMAGE_RESOURCE_DIR_STRING_U;
    s = HeapAlloc(GetProcessHeap(), 0, ((*string).Length as usize + 1) * core::mem::size_of::<u16>()) as LPWSTR;
    memcpy(
        s as *mut core::ffi::c_void,
        (*string).NameString.as_ptr() as *const core::ffi::c_void,
        ((*string).Length as usize + 1) * core::mem::size_of::<u16>(),
    );
    *s.offset((*string).Length as isize) = 0;

    return s;
}
unsafe fn enumerate_mapped_resources(
    updates: *mut QUEUEDUPDATES,
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
    root: *const IMAGE_RESOURCE_DIRECTORY,
) -> BOOL {
    let mut namedir: *const IMAGE_RESOURCE_DIRECTORY;
    let mut langdir: *const IMAGE_RESOURCE_DIRECTORY;
    let mut e1: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut e2: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut e3: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut data: *const IMAGE_RESOURCE_DATA_ENTRY;
    let mut i: DWORD;
    let mut j: DWORD;
    let mut k: DWORD;

    TRACE!(
        "version (%d.%d) %d named %d id entries\n",
        (*root).MajorVersion,
        (*root).MinorVersion,
        (*root).NumberOfNamedEntries,
        (*root).NumberOfIdEntries,
    );

    i = 0;
    while i < ((*root).NumberOfNamedEntries as u32 + (*root).NumberOfIdEntries as u32) {
        let Type: LPWSTR;

        e1 = (root.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY).offset(i as isize);

        Type = resource_dup_string(root, e1);

        namedir = (root as *const u8)
            .offset(rde_offset_to_directory(e1) as isize) as *const IMAGE_RESOURCE_DIRECTORY;
        j = 0;
        while j < ((*namedir).NumberOfNamedEntries as u32 + (*namedir).NumberOfIdEntries as u32) {
            let Name: LPWSTR;

            e2 = (namedir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY).offset(j as isize);

            Name = resource_dup_string(root, e2);

            langdir = (root as *const u8)
                .offset(rde_offset_to_directory(e2) as isize) as *const IMAGE_RESOURCE_DIRECTORY;
            k = 0;
            while k < ((*langdir).NumberOfNamedEntries as u32 + (*langdir).NumberOfIdEntries as u32) {
                let Lang: LANGID;
                let p: *mut core::ffi::c_void;
                let resdata: *mut resource_data;

                e3 = (langdir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY)
                    .offset(k as isize);

                Lang = rde_id(e3);

                data = (root as *const u8)
                    .offset(rde_offset_to_data(e3) as isize) as *const IMAGE_RESOURCE_DATA_ENTRY;

                p = address_from_rva(base, mapping_size, (*data).OffsetToData, (*data).Size);

                resdata = allocate_resource_data(Lang, (*data).CodePage, p, (*data).Size, 0);
                if !resdata.is_null() {
                    if update_add_resource(updates, Type, Name, Lang, resdata, 0) == 0 {
                        HeapFree(GetProcessHeap(), 0, resdata as *mut core::ffi::c_void);
                    }
                }
                k += 1;
            }
            res_free_str(Name);
            j += 1;
        }
        res_free_str(Type);
        i += 1;
    }

    return 1;
}
unsafe fn read_mapped_resources(
    updates: *mut QUEUEDUPDATES,
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
) -> BOOL {
    let root: *const IMAGE_RESOURCE_DIRECTORY;
    let nt: *const IMAGE_NT_HEADERS;
    let sec: *const IMAGE_SECTION_HEADER;
    let mut num_sections: DWORD = 0;
    let mut i: DWORD;

    nt = get_nt_header(base, mapping_size);
    if nt.is_null() {
        return FALSE as BOOL;
    }

    sec = get_section_header(base, mapping_size, &mut num_sections);
    if sec.is_null() {
        return FALSE as BOOL;
    }

    i = 0;
    while i < num_sections {
        if memcmp(
            (*sec.offset(i as isize)).Name.as_ptr() as *const core::ffi::c_void,
            b".rsrc\0".as_ptr() as *const core::ffi::c_void,
            6,
        ) == 0
        {
            break;
        }
        i += 1;
    }

    if i == num_sections {
        return TRUE as BOOL;
    }

    /* check the resource data is inside the mapping */
    if (*sec.offset(i as isize)).PointerToRawData > mapping_size
        || ((*sec.offset(i as isize)).PointerToRawData + (*sec.offset(i as isize)).SizeOfRawData)
            > mapping_size
    {
        return TRUE as BOOL;
    }

    TRACE!(
        b"found .rsrc at %08x, size %08x\0".as_ptr() as *const i8,
        (*sec.offset(i as isize)).PointerToRawData,
        (*sec.offset(i as isize)).SizeOfRawData,
    );

    if (*sec.offset(i as isize)).PointerToRawData == 0
        || (*sec.offset(i as isize)).SizeOfRawData
            < core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD
    {
        return TRUE as BOOL;
    }

    root = (base as *mut u8).offset((*sec.offset(i as isize)).PointerToRawData as isize)
        as *const IMAGE_RESOURCE_DIRECTORY;
    enumerate_mapped_resources(updates, base, mapping_size, root);

    return TRUE as BOOL;
}
unsafe fn map_file_into_memory(mi: *mut mapping_info) -> BOOL {
    let page_attr: DWORD;
    let perm: DWORD;
    let mapping: HANDLE;

    if (*mi).read_write != 0 {
        page_attr = PAGE_READWRITE;
        perm = FILE_MAP_WRITE | FILE_MAP_READ;
    } else {
        page_attr = PAGE_READONLY;
        perm = FILE_MAP_READ;
    }

    mapping = CreateFileMappingW((*mi).file, core::ptr::null_mut(), page_attr, 0, 0, core::ptr::null_mut());
    if mapping.is_null() {
        return 0;
    }

    (*mi).base = MapViewOfFile(mapping, perm, 0, 0, (*mi).size as SIZE_T);
    CloseHandle(mapping);

    return if !(*mi).base.is_null() { 1 } else { 0 };
}
unsafe fn unmap_file_from_memory(mi: *mut mapping_info) -> BOOL {
    if !(*mi).base.is_null() {
        UnmapViewOfFile((*mi).base);
    }
    (*mi).base = core::ptr::null_mut();
    return 1;
}
unsafe fn destroy_mapping(mi: *mut mapping_info) {
    if mi.is_null() {
        return;
    }
    unmap_file_from_memory(mi);
    if !(*mi).file.is_null() {
        CloseHandle((*mi).file);
    }
    HeapFree(GetProcessHeap(), 0, mi as *mut _);
}
unsafe fn create_mapping(name: LPCWSTR, rw: BOOL) -> *mut mapping_info {
    let mi: *mut mapping_info = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<mapping_info>()) as *mut mapping_info;
    if mi.is_null() {
        return core::ptr::null_mut();
    }

    (*mi).read_write = rw;

    (*mi).file = CreateFileW(
        name,
        GENERIC_READ | if rw != 0 { GENERIC_WRITE } else { 0 },
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );

    if (*mi).file != INVALID_HANDLE_VALUE as HANDLE {
        (*mi).size = GetFileSize((*mi).file, core::ptr::null_mut());

        if map_file_into_memory(mi) != 0 {
            return mi;
        }
    }
    destroy_mapping(mi);
    core::ptr::null_mut()
}
unsafe fn resize_mapping(mi: *mut mapping_info, new_size: DWORD) -> BOOL {
    if unmap_file_from_memory(mi) == 0 {
        return 0i32;
    }
    SetFilePointer((*mi).file, new_size as i32, core::ptr::null_mut(), FILE_BEGIN);
    if SetEndOfFile((*mi).file) == 0 {
        ERR!("failed to set file size to %08x\n", new_size);
        return 0i32;
    }
    (*mi).size = new_size;
    return map_file_into_memory(mi);
}
unsafe fn get_resource_sizes(updates: *mut QUEUEDUPDATES, si: *mut resource_size_info) {
    let mut types: *mut resource_dir_entry = core::ptr::null_mut();
    let mut names: *mut resource_dir_entry = core::ptr::null_mut();
    let mut data: *mut resource_data = core::ptr::null_mut();
    let mut num_types: DWORD = 0;
    let mut num_names: DWORD = 0;
    let mut num_langs: DWORD = 0;
    let mut strings_size: DWORD = 0;
    let mut data_size: DWORD = 0;

    memset(si as *mut core::ffi::c_void, 0, core::mem::size_of::<resource_size_info>());

    LIST_FOR_EACH_ENTRY!(types, &raw mut (*updates).root, resource_dir_entry, entry, {
        num_types += 1;
        if IS_INTRESOURCE!((*types).id) == 0 {
            strings_size += core::mem::size_of::<WORD>() as DWORD + lstrlenW((*types).id) as DWORD * core::mem::size_of::<WCHAR>() as DWORD;
        }

        LIST_FOR_EACH_ENTRY!(names, &raw mut (*types).children, resource_dir_entry, entry, {
            num_names += 1;

            if IS_INTRESOURCE!((*names).id) == 0 {
                strings_size += core::mem::size_of::<WORD>() as DWORD + lstrlenW((*names).id) as DWORD * core::mem::size_of::<WCHAR>() as DWORD;
            }

            LIST_FOR_EACH_ENTRY!(data, &raw mut (*names).children, resource_data, entry, {
                num_langs += 1;
                data_size += ((*data).cbData + 3) & !3;
            });
        });
    });

    /* names are at the end of the types */
    (*si).names_ofs = core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD +
            num_types * core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

    /* language directories are at the end of the names */
    (*si).langs_ofs = (*si).names_ofs +
            num_types * core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD +
            num_names * core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

    (*si).data_entry_ofs = (*si).langs_ofs +
            num_names * core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD +
            num_langs * core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

    (*si).strings_ofs = (*si).data_entry_ofs +
            num_langs * core::mem::size_of::<IMAGE_RESOURCE_DATA_ENTRY>() as DWORD;

    (*si).data_ofs = (*si).strings_ofs + ((strings_size + 3) & !3);

    (*si).total_size = (*si).data_ofs + data_size;

    TRACE!("names %08x langs %08x data entries %08x strings %08x data %08x total %08x\n",
          (*si).names_ofs, (*si).langs_ofs, (*si).data_entry_ofs,
          (*si).strings_ofs, (*si).data_ofs, (*si).total_size);
}
unsafe fn res_write_padding(res_base: *mut u8, size: u32) {
    static PAD: [u8; 16] = [
        b'P', b'A', b'D', b'D', b'I', b'N', b'G', b'X',
        b'X', b'P', b'A', b'D', b'D', b'I', b'N', b'G',
    ];
    let mut i: u32 = 0;
    i = 0;
    while i < size / core::mem::size_of_val(&PAD) as u32 {
        core::ptr::copy_nonoverlapping( // nolint: silent-failure (byte-array pointee — count==bytes)
            PAD.as_ptr(),
            res_base.offset((i * core::mem::size_of_val(&PAD) as u32) as isize),
            core::mem::size_of_val(&PAD),
        );
        i += 1;
    }
    core::ptr::copy_nonoverlapping( // nolint: silent-failure (byte-array pointee — count==bytes)
        PAD.as_ptr(),
        res_base.offset((i * core::mem::size_of_val(&PAD) as u32) as isize),
        (size % core::mem::size_of_val(&PAD) as u32) as usize,
    );
}
unsafe fn write_resources(
    updates: *mut QUEUEDUPDATES,
    base: LPBYTE,
    si: *mut resource_size_info,
    rva: DWORD,
) -> BOOL {
    let mut types: *mut resource_dir_entry;
    let mut names: *mut resource_dir_entry;
    let mut data: *mut resource_data;
    let root: *mut IMAGE_RESOURCE_DIRECTORY;

    TRACE!("%p %p %p %08x\n", updates, base, si, rva);

    core::ptr::write_bytes(base, 0, (*si).total_size as usize);

    root = base as *mut IMAGE_RESOURCE_DIRECTORY;
    core::ptr::write_bytes(root as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>());
    (*root).MajorVersion = 4;
    (*si).types_ofs = core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD;
    LIST_FOR_EACH_ENTRY!(
        types,
        &mut (*updates).root,
        resource_dir_entry,
        entry,
    {
        let e1: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
        let namedir: *mut IMAGE_RESOURCE_DIRECTORY;

        e1 = base.offset((*si).types_ofs as isize) as *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
        core::ptr::write_bytes(e1 as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>());
        if IS_INTRESOURCE!((*types).id) == 0 {
            let strings: *mut WCHAR;
            let len: DWORD;

            (*root).NumberOfNamedEntries += 1;
            rde_set_name_is_string(e1, (1) as DWORD);
            rde_set_name_offset(e1, ((*si).strings_ofs) as DWORD);

            strings = base.offset((*si).strings_ofs as isize) as *mut WCHAR;
            len = lstrlenW((*types).id) as DWORD;
            *strings = len as WORD;
            core::ptr::copy_nonoverlapping(
                (*types).id as *const WCHAR,
                strings.offset(1),
                len as usize,
            );
            (*si).strings_ofs += (len + 1) * core::mem::size_of::<WCHAR>() as DWORD;
        } else {
            (*root).NumberOfIdEntries += 1;
            rde_set_id(e1, (LOWORD!((*types).id)) as WORD);
        }
        rde_set_offset_to_directory(e1, ((*si).names_ofs) as DWORD);
        rde_set_data_is_directory(e1, (1) as DWORD);
        (*si).types_ofs += core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

        namedir = base.offset((*si).names_ofs as isize) as *mut IMAGE_RESOURCE_DIRECTORY;
        core::ptr::write_bytes(namedir as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>());
        (*namedir).MajorVersion = 4;
        (*si).names_ofs += core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD;

        LIST_FOR_EACH_ENTRY!(
            names,
            &mut (*types).children,
            resource_dir_entry,
            entry,
        {
            let e2: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
            let langdir: *mut IMAGE_RESOURCE_DIRECTORY;

            e2 = base.offset((*si).names_ofs as isize) as *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
            core::ptr::write_bytes(e2 as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>());
            if IS_INTRESOURCE!((*names).id) == 0 {
                let strings: *mut WCHAR;
                let len: DWORD;

                (*namedir).NumberOfNamedEntries += 1;
                rde_set_name_is_string(e2, (1) as DWORD);
                rde_set_name_offset(e2, ((*si).strings_ofs) as DWORD);

                strings = base.offset((*si).strings_ofs as isize) as *mut WCHAR;
                len = lstrlenW((*names).id) as DWORD;
                *strings = len as WORD;
                core::ptr::copy_nonoverlapping(
                    (*names).id as *const WCHAR,
                    strings.offset(1),
                    len as usize,
                );
                (*si).strings_ofs += (len + 1) * core::mem::size_of::<WCHAR>() as DWORD;
            } else {
                (*namedir).NumberOfIdEntries += 1;
                rde_set_id(e2, (LOWORD!((*names).id)) as WORD);
            }
            rde_set_offset_to_directory(e2, ((*si).langs_ofs) as DWORD);
            rde_set_data_is_directory(e2, (1) as DWORD);
            (*si).names_ofs += core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

            langdir = base.offset((*si).langs_ofs as isize) as *mut IMAGE_RESOURCE_DIRECTORY;
            core::ptr::write_bytes(langdir as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>());
            (*langdir).MajorVersion = 4;
            (*si).langs_ofs += core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY>() as DWORD;

            LIST_FOR_EACH_ENTRY!(
                data,
                &mut (*names).children,
                resource_data,
                entry,
            {
                let e3: *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
                let de: *mut IMAGE_RESOURCE_DATA_ENTRY;
                let pad_size: i32;

                e3 = base.offset((*si).langs_ofs as isize) as *mut IMAGE_RESOURCE_DIRECTORY_ENTRY;
                core::ptr::write_bytes(e3 as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>());
                (*langdir).NumberOfIdEntries += 1;
                rde_set_id(e3, (LOWORD!((*data).lang)) as WORD);
                rde_set_offset_to_data(e3, ((*si).data_entry_ofs) as DWORD);

                (*si).langs_ofs += core::mem::size_of::<IMAGE_RESOURCE_DIRECTORY_ENTRY>() as DWORD;

                de = base.offset((*si).data_entry_ofs as isize) as *mut IMAGE_RESOURCE_DATA_ENTRY;
                core::ptr::write_bytes(de as *mut u8, 0, core::mem::size_of::<IMAGE_RESOURCE_DATA_ENTRY>());
                (*de).OffsetToData = (*si).data_ofs + rva;
                (*de).Size = (*data).cbData;
                (*de).CodePage = (*data).codepage;
                (*si).data_entry_ofs += core::mem::size_of::<IMAGE_RESOURCE_DATA_ENTRY>() as DWORD;

                core::ptr::copy_nonoverlapping(
                    (*data).lpData as *const u8,
                    base.offset((*si).data_ofs as isize),
                    (*data).cbData as usize,
                );
                (*si).data_ofs += (*data).cbData;

                pad_size = (-((*si).data_ofs as i32)) & 3;
                res_write_padding(base.offset((*si).data_ofs as isize), pad_size as u32);
                (*si).data_ofs += pad_size as DWORD;
            });
        });
    });

    1
}
unsafe fn get_resource_section(
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
) -> *mut IMAGE_SECTION_HEADER {
    let mut sec: *mut IMAGE_SECTION_HEADER;
    let nt: *mut IMAGE_NT_HEADERS;
    let mut i: DWORD;
    let mut num_sections: DWORD = 0;

    nt = get_nt_header(base, mapping_size);
    if nt.is_null() {
        return core::ptr::null_mut();
    }

    sec = get_section_header(base, mapping_size, &mut num_sections);
    if sec.is_null() {
        return core::ptr::null_mut();
    }

    /* find the resources section */
    i = 0;
    while i < num_sections {
        if memcmp(
            (*sec.offset(i as isize)).Name.as_ptr() as *const core::ffi::c_void,
            b".rsrc\0".as_ptr() as *const core::ffi::c_void,
            6,
        ) == 0
        {
            break;
        }
        i += 1;
    }

    if i == num_sections {
        return core::ptr::null_mut();
    }

    &mut *sec.offset(i as isize)
}
unsafe fn get_init_data_size(
    base: *mut core::ffi::c_void,
    mapping_size: DWORD,
) -> DWORD {
    let mut i: DWORD = 0;
    let mut sz: DWORD = 0;
    let mut num_sections: DWORD = 0;
    let s: *mut IMAGE_SECTION_HEADER;

    s = get_section_header(base, mapping_size, &mut num_sections);

    i = 0;
    while i < num_sections {
        if (*s.add(i as usize)).Characteristics & IMAGE_SCN_CNT_INITIALIZED_DATA != 0 {
            sz += (*s.add(i as usize)).SizeOfRawData;
        }
        i += 1;
    }

    TRACE!("size = %08x\n", sz);

    return sz;
}
unsafe fn write_raw_resources(updates: *mut QUEUEDUPDATES) -> BOOL {
    static prefix: [WCHAR; 5] = [b'r' as WCHAR, b'e' as WCHAR, b's' as WCHAR, b'u' as WCHAR, 0];
    let mut tempdir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut tempfile: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut i: DWORD;
    let mut section_size: DWORD;
    let mut ret: BOOL = 0;
    let mut sec: *mut IMAGE_SECTION_HEADER;
    let mut nt: *mut IMAGE_NT_HEADERS32;
    let mut nt64: *mut IMAGE_NT_HEADERS64;
    let mut res_size: resource_size_info = unsafe { core::mem::zeroed() };
    let mut res_base: *mut u8;
    let mut read_map: *mut mapping_info = core::ptr::null_mut();
    let mut write_map: *mut mapping_info = core::ptr::null_mut();
    let mut PeSectionAlignment: DWORD;
    let mut PeFileAlignment: DWORD;
    let mut PeSizeOfImage: DWORD;

    tempdir[0] = 0;
    if GetTempPathW(MAX_PATH, tempdir.as_mut_ptr()) == 0 {
        return ret;
    }

    if GetTempFileNameW(tempdir.as_mut_ptr(), prefix.as_ptr(), 0, tempfile.as_mut_ptr()) == 0 {
        return ret;
    }

    'done: {
    if CopyFileW((*updates).pFileName, tempfile.as_mut_ptr(), 0) == 0 {
        break 'done;
    }

    TRACE!(b"tempfile %s\n\0".as_ptr() as *const i8, debugstr_w!(tempfile.as_mut_ptr()));

    if (*updates).bDeleteExistingResources == 0 {
        read_map = create_mapping((*updates).pFileName, 0);
        if read_map.is_null() {
            break 'done;
        }

        ret = read_mapped_resources(updates, (*read_map).base, (*read_map).size);
        if ret == 0 {
            ERR!("failed to read existing resources\n");
            break 'done;
        }
    }

    write_map = create_mapping(tempfile.as_mut_ptr(), 1);
    if write_map.is_null() {
        break 'done;
    }

    nt = get_nt_header((*write_map).base, (*write_map).size) as *mut IMAGE_NT_HEADERS32;
    if nt.is_null() {
        break 'done;
    }

    nt64 = nt as *mut IMAGE_NT_HEADERS64;
    if (*nt).OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16 {
        PeSectionAlignment = (*nt64).OptionalHeader.SectionAlignment;
        PeFileAlignment = (*nt64).OptionalHeader.FileAlignment;
        PeSizeOfImage = (*nt64).OptionalHeader.SizeOfImage;
    } else {
        PeSectionAlignment = (*nt).OptionalHeader.SectionAlignment;
        PeFileAlignment = (*nt).OptionalHeader.FileAlignment;
        PeSizeOfImage = (*nt).OptionalHeader.SizeOfImage;
    }

    if (PeSectionAlignment as LONG) <= 0 {
        ERR!("invalid section alignment %08x\n", PeSectionAlignment);
        break 'done;
    }

    if (PeFileAlignment as LONG) <= 0 {
        ERR!("invalid file alignment %08x\n", PeFileAlignment);
        break 'done;
    }

    sec = get_resource_section((*write_map).base, (*write_map).size);
    if sec.is_null() {
        let mut num_sections: DWORD = 0;

        sec = get_section_header((*write_map).base, (*write_map).size, &mut num_sections);
        if sec.is_null() {
            break 'done;
        }

        sec = sec.offset(num_sections as isize);
        (*nt).FileHeader.NumberOfSections += 1;

        core::ptr::write_bytes(sec, 0, 1);
        core::ptr::copy_nonoverlapping(b".rsrc\0".as_ptr(), (*sec).Name.as_mut_ptr() as *mut u8, 5);
        (*sec).Characteristics = IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ;
        (*sec).VirtualAddress = PeSizeOfImage;
    }

    if (*sec).PointerToRawData == 0 {
        (*sec).PointerToRawData = (*write_map).size + (-((*write_map).size as i32) as DWORD) % PeFileAlignment;
        (*sec).SizeOfRawData = 0;
    }

    TRACE!(
        b"before .rsrc at %08x, size %08x\n\0".as_ptr() as *const i8,
        (*sec).PointerToRawData,
        (*sec).SizeOfRawData,
    );

    get_resource_sizes(updates, &mut res_size);

    section_size = res_size.total_size;
    section_size += (-(section_size as i32) as DWORD) % PeFileAlignment;

    TRACE!(
        b"requires %08x (%08x) bytes\n\0".as_ptr() as *const i8,
        res_size.total_size,
        section_size,
    );

    if section_size != (*sec).SizeOfRawData {
        let old_size: DWORD = (*write_map).size;
        let virtual_section_size: DWORD =
            res_size.total_size + (-(res_size.total_size as i32) as DWORD) % PeSectionAlignment;
        let delta: i32 = (section_size as i32)
            - (((*sec).SizeOfRawData + (-((*sec).SizeOfRawData as i32) as DWORD) % PeFileAlignment) as i32);
        let rva_delta: i32 = (virtual_section_size as i32)
            - (((*sec).Misc.VirtualSize + (-((*sec).Misc.VirtualSize as i32) as DWORD) % PeSectionAlignment) as i32);
        let rsrc_is_last: BOOL =
            if (*sec).PointerToRawData + (*sec).SizeOfRawData >= old_size { 1 } else { 0 };
        let mapping_size: DWORD = if rsrc_is_last != 0 {
            (*sec).PointerToRawData + section_size
        } else {
            old_size + delta as DWORD
        };
        let resize_after: BOOL = if mapping_size < old_size && rsrc_is_last == 0 { 1 } else { 0 };

        TRACE!(b"file size %08x -> %08x\n\0".as_ptr() as *const i8, old_size, mapping_size);

        if resize_after == 0 {
            ret = resize_mapping(write_map, mapping_size);

            nt = get_nt_header((*write_map).base, mapping_size) as *mut IMAGE_NT_HEADERS32;
            if nt.is_null() {
                ERR!("couldn't get NT header\n");
                break 'done;
            }
            nt64 = nt as *mut IMAGE_NT_HEADERS64;

            sec = get_resource_section((*write_map).base, mapping_size);
            if sec.is_null() {
                break 'done;
            }
        }

        if rsrc_is_last == 0 {
            let mut s: *mut IMAGE_SECTION_HEADER;
            let tail_start: DWORD = (*sec).PointerToRawData + (*sec).SizeOfRawData;
            let mut num_sections: DWORD = 0;

            core::ptr::copy(
                ((*write_map).base as *mut u8).offset(tail_start as isize),
                ((*write_map).base as *mut u8).offset((tail_start + delta as DWORD) as isize),
                (old_size - tail_start) as usize,
            );

            s = get_section_header((*write_map).base, mapping_size, &mut num_sections);

            i = 0;
            while i < num_sections {
                if (*s.offset(i as isize)).PointerToRawData > (*sec).PointerToRawData {
                    (*s.offset(i as isize)).PointerToRawData =
                        ((*s.offset(i as isize)).PointerToRawData as i32 + delta) as DWORD;
                    (*s.offset(i as isize)).VirtualAddress =
                        ((*s.offset(i as isize)).VirtualAddress as i32 + rva_delta) as DWORD;
                }
                i += 1;
            }
        }

        if resize_after != 0 {
            ret = resize_mapping(write_map, mapping_size);

            nt = get_nt_header((*write_map).base, mapping_size) as *mut IMAGE_NT_HEADERS32;
            if nt.is_null() {
                ERR!("couldn't get NT header\n");
                break 'done;
            }
            nt64 = nt as *mut IMAGE_NT_HEADERS64;

            sec = get_resource_section((*write_map).base, mapping_size);
            if sec.is_null() {
                break 'done;
            }
        }

        (*sec).SizeOfRawData = section_size;
        (*sec).Misc.VirtualSize = virtual_section_size;
        if (*nt).OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16 {
            (*nt64).OptionalHeader.SizeOfImage = ((*nt64).OptionalHeader.SizeOfImage as i32 + rva_delta) as DWORD;
            (*nt64)
                .OptionalHeader
                .DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE as usize]
                .VirtualAddress = (*sec).VirtualAddress;
            (*nt64)
                .OptionalHeader
                .DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE as usize]
                .Size = res_size.total_size;
            (*nt64).OptionalHeader.SizeOfInitializedData =
                get_init_data_size((*write_map).base, mapping_size);

            i = 0;
            while i < (*nt64).OptionalHeader.NumberOfRvaAndSizes {
                if (*nt64).OptionalHeader.DataDirectory[i as usize].VirtualAddress > (*sec).VirtualAddress {
                    (*nt64).OptionalHeader.DataDirectory[i as usize].VirtualAddress =
                        ((*nt64).OptionalHeader.DataDirectory[i as usize].VirtualAddress as i32 + rva_delta) as DWORD;
                }
                i += 1;
            }
        } else {
            (*nt).OptionalHeader.SizeOfImage = ((*nt).OptionalHeader.SizeOfImage as i32 + rva_delta) as DWORD;
            (*nt)
                .OptionalHeader
                .DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE as usize]
                .VirtualAddress = (*sec).VirtualAddress;
            (*nt)
                .OptionalHeader
                .DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE as usize]
                .Size = res_size.total_size;
            (*nt).OptionalHeader.SizeOfInitializedData =
                get_init_data_size((*write_map).base, mapping_size);

            i = 0;
            while i < (*nt).OptionalHeader.NumberOfRvaAndSizes {
                if (*nt).OptionalHeader.DataDirectory[i as usize].VirtualAddress > (*sec).VirtualAddress {
                    (*nt).OptionalHeader.DataDirectory[i as usize].VirtualAddress =
                        ((*nt).OptionalHeader.DataDirectory[i as usize].VirtualAddress as i32 + rva_delta) as DWORD;
                }
                i += 1;
            }
        }
    }

    res_base = ((*write_map).base as *mut u8).offset((*sec).PointerToRawData as isize);

    TRACE!(
        b"base = %p offset = %08x\n\0".as_ptr() as *const i8,
        (*write_map).base,
        (*sec).PointerToRawData,
    );

    ret = write_resources(updates, res_base, &mut res_size, (*sec).VirtualAddress);

    res_write_padding(
        res_base.offset(res_size.total_size as isize),
        section_size - res_size.total_size,
    );

    TRACE!(
        b"after  .rsrc at %08x, size %08x\n\0".as_ptr() as *const i8,
        (*sec).PointerToRawData,
        (*sec).SizeOfRawData,
    );

    } // 'done

    destroy_mapping(read_map);
    destroy_mapping(write_map);

    if ret != 0 {
        ret = CopyFileW(tempfile.as_mut_ptr(), (*updates).pFileName, 0);
    }

    DeleteFileW(tempfile.as_mut_ptr());

    ret
}

#[no_mangle]
pub unsafe extern "system" fn BeginUpdateResourceW(
    pFileName: LPCWSTR,
    bDeleteExistingResources: BOOL,
) -> HANDLE {
    let mut updates: *mut QUEUEDUPDATES = core::ptr::null_mut();
    let mut hUpdate: HANDLE;
    let mut file: HANDLE;
    let mut ret: HANDLE = core::ptr::null_mut();

    TRACE!(
        b"%s, %d\0".as_ptr() as *const i8,
        debugstr_w!(pFileName),
        bDeleteExistingResources,
    );

    hUpdate = GlobalAlloc(GHND, core::mem::size_of::<QUEUEDUPDATES>() as SIZE_T);
    if hUpdate.is_null() {
        return ret;
    }

    updates = GlobalLock(hUpdate) as *mut QUEUEDUPDATES;
    if !updates.is_null() {
        list_init(&mut (*updates).root);
        (*updates).bDeleteExistingResources = bDeleteExistingResources;
        (*updates).pFileName = HeapAlloc(
            GetProcessHeap(),
            0,
            ((lstrlenW(pFileName) + 1) as usize * core::mem::size_of::<u16>()) as usize,
        ) as LPWSTR;
        if !(*updates).pFileName.is_null() {
            lstrcpyW((*updates).pFileName, pFileName);

            file = CreateFileW(
                pFileName,
                GENERIC_READ | GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );

            /* if resources are deleted, only the file's presence is checked */
            if file != INVALID_HANDLE_VALUE as HANDLE
                && (bDeleteExistingResources != 0 || check_pe_exe(file, updates) != 0)
            {
                ret = hUpdate;
            } else {
                HeapFree(GetProcessHeap(), 0, (*updates).pFileName as *mut core::ffi::c_void);
            }

            CloseHandle(file);
        }
        GlobalUnlock(hUpdate as *mut core::ffi::c_void);
    }

    if ret.is_null() {
        GlobalFree(hUpdate as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn BeginUpdateResourceA(
    pFileName: LPCSTR,
    bDeleteExistingResources: BOOL,
) -> HANDLE {
    let mut FileNameW: _UNICODE_STRING = core::mem::zeroed();
    RtlCreateUnicodeStringFromAsciiz(&mut FileNameW, pFileName);
    let ret: HANDLE = BeginUpdateResourceW(FileNameW.Buffer, bDeleteExistingResources);
    RtlFreeUnicodeString(&mut FileNameW);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn EndUpdateResourceW(
    hUpdate: HANDLE,
    fDiscard: BOOL,
) -> BOOL {
    let updates: *mut QUEUEDUPDATES;
    let ret: BOOL;

    TRACE!("%p %d\n", hUpdate, fDiscard);

    updates = GlobalLock(hUpdate) as *mut QUEUEDUPDATES;
    if updates.is_null() {
        return 0;
    }

    ret = if fDiscard != 0 { 1 } else { write_raw_resources(updates) };

    free_resource_directory(&mut (*updates).root, 2);

    HeapFree(GetProcessHeap(), 0, (*updates).pFileName as *mut core::ffi::c_void);
    GlobalUnlock(hUpdate);
    GlobalFree(hUpdate);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn EndUpdateResourceA(
    hUpdate: HANDLE,
    fDiscard: BOOL,
) -> BOOL {
    return EndUpdateResourceW(hUpdate, fDiscard);
}

#[no_mangle]
pub unsafe extern "system" fn UpdateResourceW(
    hUpdate: HANDLE,
    lpType: LPCWSTR,
    lpName: LPCWSTR,
    wLanguage: WORD,
    lpData: LPVOID,
    cbData: DWORD,
) -> BOOL {
    let updates: *mut QUEUEDUPDATES;
    let mut ret: BOOL = 0;

    TRACE!(
        "%p %s %s %08x %p %d\0".as_ptr() as *const i8,
        hUpdate,
        debugstr_w!(lpType),
        debugstr_w!(lpName),
        wLanguage,
        lpData,
        cbData,
    );

    updates = GlobalLock(hUpdate) as *mut QUEUEDUPDATES;
    if !updates.is_null() {
        if lpData.is_null() && cbData == 0 {
            /* remove resource */
            ret = update_add_resource(updates, lpType, lpName, wLanguage, core::ptr::null_mut(), 1);
        } else {
            let data: *mut resource_data;
            data = allocate_resource_data(wLanguage, 0, lpData, cbData, 1);
            if !data.is_null() {
                ret = update_add_resource(updates, lpType, lpName, wLanguage, data, 1);
            }
        }
        GlobalUnlock(hUpdate);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn UpdateResourceA(
    hUpdate: HANDLE,
    lpType: LPCSTR,
    lpName: LPCSTR,
    wLanguage: WORD,
    lpData: LPVOID,
    cbData: DWORD,
) -> BOOL {
    let ret: BOOL;
    let mut TypeW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut NameW: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    if IS_INTRESOURCE!(lpType) != 0 {
        TypeW.Buffer = ULongToPtr!(LOWORD!(lpType)) as LPWSTR;
    } else {
        RtlCreateUnicodeStringFromAsciiz(&mut TypeW, lpType);
    }
    if IS_INTRESOURCE!(lpName) != 0 {
        NameW.Buffer = ULongToPtr!(LOWORD!(lpName)) as LPWSTR;
    } else {
        RtlCreateUnicodeStringFromAsciiz(&mut NameW, lpName);
    }
    ret = UpdateResourceW(hUpdate, TypeW.Buffer, NameW.Buffer, wLanguage, lpData, cbData);
    if IS_INTRESOURCE!(lpType) == 0 {
        RtlFreeUnicodeString(&mut TypeW);
    }
    if IS_INTRESOURCE!(lpName) == 0 {
        RtlFreeUnicodeString(&mut NameW);
    }
    return ret;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Resources
//  *
//  * Copyright 1993 Robert J. Amstadt
//  * Copyright 1995, 2003 Alexandre Julliard
//  * Copyright 2006 Mike McCormack
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <k32.h>
//
// #include <wine/list.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(resource);
//
// /* retrieve the resource name to pass to the ntdll functions */
// static NTSTATUS get_res_nameA( LPCSTR name, UNICODE_STRING *str )
// {
//     if (IS_INTRESOURCE!(name))
//     {
//         str->Buffer = ULongToPtr!(LOWORD!(name));
//         return STATUS_SUCCESS;
//     }
//     if (name[0] == '#')
//     {
//         ULONG value;
//         if (RtlCharToInteger( name + 1, 10, &value ) != STATUS_SUCCESS || HIWORD!(value))
//             return STATUS_INVALID_PARAMETER;
//         str->Buffer = ULongToPtr!(value);
//         return STATUS_SUCCESS;
//     }
//     RtlCreateUnicodeStringFromAsciiz( str, name );
//     RtlUpcaseUnicodeString( str, str, FALSE );
//     return STATUS_SUCCESS;
// }
//
// /* retrieve the resource name to pass to the ntdll functions */
// static NTSTATUS get_res_nameW( LPCWSTR name, UNICODE_STRING *str )
// {
//     if (IS_INTRESOURCE!(name))
//     {
//         str->Buffer = ULongToPtr!(LOWORD!(name));
//         return STATUS_SUCCESS;
//     }
//     if (name[0] == '#')
//     {
//         ULONG value;
//         RtlInitUnicodeString( str, name + 1 );
//         if (RtlUnicodeStringToInteger( str, 10, &value ) != STATUS_SUCCESS || HIWORD!(value))
//             return STATUS_INVALID_PARAMETER;
//         str->Buffer = ULongToPtr!(value);
//         return STATUS_SUCCESS;
//     }
//     RtlCreateUnicodeString( str, name );
//     RtlUpcaseUnicodeString( str, str, FALSE );
//     return STATUS_SUCCESS;
// }
//
// /* implementation of FindResourceExA */
// static HRSRC find_resourceA( HMODULE hModule, LPCSTR type, LPCSTR name, WORD lang )
// {
//     NTSTATUS status;
//     UNICODE_STRING nameW, typeW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DATA_ENTRY *entry = NULL;
//
//     nameW.Buffer = NULL;
//     typeW.Buffer = NULL;
//
//     __TRY
//     {
//         if ((status = get_res_nameA( name, &nameW )) != STATUS_SUCCESS) goto done;
//         if ((status = get_res_nameA( type, &typeW )) != STATUS_SUCCESS) goto done;
//         info.Type = (ULONG_PTR)typeW.Buffer;
//         info.Name = (ULONG_PTR)nameW.Buffer;
//         info.Language = lang;
//         status = LdrFindResource_U( hModule, &mut info, 3, &entry );
//     done:
//         if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//     }
//     __ENDTRY
//
//     if (!IS_INTRESOURCE!(nameW.Buffer)) HeapFree( GetProcessHeap(), 0, nameW.Buffer );
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     return (HRSRC)entry;
// }
//
//
// /* implementation of FindResourceExW */
// static HRSRC find_resourceW( HMODULE hModule, LPCWSTR type, LPCWSTR name, WORD lang )
// {
//     NTSTATUS status;
//     UNICODE_STRING nameW, typeW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DATA_ENTRY *entry = NULL;
//
//     nameW.Buffer = typeW.Buffer = NULL;
//
//     __TRY
//     {
//         if ((status = get_res_nameW( name, &nameW )) != STATUS_SUCCESS) goto done;
//         if ((status = get_res_nameW( type, &typeW )) != STATUS_SUCCESS) goto done;
//         info.Type = (ULONG_PTR)typeW.Buffer;
//         info.Name = (ULONG_PTR)nameW.Buffer;
//         info.Language = lang;
//         status = LdrFindResource_U( hModule, &mut info, 3, &entry );
//     done:
//         if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         SetLastError( ERROR_INVALID_PARAMETER );
//     }
//     __ENDTRY
//
//     if (!IS_INTRESOURCE!(nameW.Buffer)) HeapFree( GetProcessHeap(), 0, nameW.Buffer );
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     return (HRSRC)entry;
// }
//
// /**********************************************************************
//  *	    FindResourceExA  (KERNEL32.@)
//  */
// HRSRC WINAPI FindResourceExA( HMODULE hModule, LPCSTR type, LPCSTR name, WORD lang )
// {
//     TRACE!( "%p %s %s %04x\n", hModule, debugstr_a!(type), debugstr_a!(name), lang );
//
//     if (!hModule) hModule = GetModuleHandleW(0);
//     return find_resourceA( hModule, type, name, lang );
// }
//
//
// /**********************************************************************
//  *	    FindResourceA    (KERNEL32.@)
//  */
// HRSRC WINAPI FindResourceA( HMODULE hModule, LPCSTR name, LPCSTR type )
// {
//     return FindResourceExA( hModule, type, name, MAKELANGID!( LANG_NEUTRAL, SUBLANG_NEUTRAL ) );
// }
//
//
// /**********************************************************************
//  *	    FindResourceExW  (KERNEL32.@)
//  */
// HRSRC WINAPI FindResourceExW( HMODULE hModule, LPCWSTR type, LPCWSTR name, WORD lang )
// {
//     TRACE!( "%p %s %s %04x\n", hModule, debugstr_w!(type), debugstr_w!(name), lang );
//
//     if (!hModule) hModule = GetModuleHandleW(0);
//     return find_resourceW( hModule, type, name, lang );
// }
//
//
// /**********************************************************************
//  *	    FindResourceW    (KERNEL32.@)
//  */
// HRSRC WINAPI FindResourceW( HINSTANCE hModule, LPCWSTR name, LPCWSTR type )
// {
//     return FindResourceExW( hModule, type, name, MAKELANGID!( LANG_NEUTRAL, SUBLANG_NEUTRAL ) );
// }
//
//
// /**********************************************************************
//  *	EnumResourceTypesA	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceTypesA( HMODULE hmod, ENUMRESTYPEPROCA lpfun, LONG_PTR lparam )
// {
//     int i;
//     BOOL ret = FALSE;
//     LPSTR type = NULL;
//     DWORD len = 0, newlen;
//     NTSTATUS status;
//     IMAGE_RESOURCE_DIRECTORY *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//     const IMAGE_RESOURCE_DIR_STRING_U *str;
//
//     TRACE!( "%p %p %lx\n", hmod, lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleA( NULL );
//
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &resdir )) != STATUS_SUCCESS)
//     {
//         SetLastError( RtlNtStatusToDosError(status) );
//         return FALSE;
//     }
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     for (i = 0; i < resdir->NumberOfNamedEntries+resdir->NumberOfIdEntries; i++)
//     {
//         if (et[i].NameIsString)
//         {
//             str = (const IMAGE_RESOURCE_DIR_STRING_U *)((const BYTE *)resdir + et[i].NameOffset);
//             newlen = WideCharToMultiByte( CP_ACP, 0, str->NameString, str->Length, NULL, 0, NULL, NULL);
//             if (newlen + 1 > len)
//             {
//                 len = newlen + 1;
//                 HeapFree( GetProcessHeap(), 0, type );
//                 if (!(type = HeapAlloc( GetProcessHeap(), 0, len ))) return FALSE;
//             }
//             WideCharToMultiByte( CP_ACP, 0, str->NameString, str->Length, type, len, NULL, NULL);
//             type[newlen] = 0;
//             ret = lpfun.unwrap()(hmod,type,lparam);
//         }
//         else
//         {
//             ret = lpfun.unwrap()( hmod, UIntToPtr!(et[i].Id), lparam );
//         }
//         if (!ret) break;
//     }
//     HeapFree( GetProcessHeap(), 0, type );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	EnumResourceTypesW	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceTypesW( HMODULE hmod, ENUMRESTYPEPROCW lpfun, LONG_PTR lparam )
// {
//     int i, len = 0;
//     BOOL ret = FALSE;
//     LPWSTR type = NULL;
//     NTSTATUS status;
//     IMAGE_RESOURCE_DIRECTORY *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//     const IMAGE_RESOURCE_DIR_STRING_U *str;
//
//     TRACE!( "%p %p %lx\n", hmod, lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleW( NULL );
//
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &resdir )) != STATUS_SUCCESS)
//     {
//         SetLastError( RtlNtStatusToDosError(status) );
//         return FALSE;
//     }
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     for (i = 0; i < resdir->NumberOfNamedEntries + resdir->NumberOfIdEntries; i++)
//     {
//         if (et[i].NameIsString)
//         {
//             str = (const IMAGE_RESOURCE_DIR_STRING_U *)((const BYTE *)resdir + et[i].NameOffset);
//             if (str->Length + 1 > len)
//             {
//                 len = str->Length + 1;
//                 HeapFree( GetProcessHeap(), 0, type );
//                 if (!(type = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) ))) return FALSE;
//             }
//             memcpy(type, str->NameString, str->Length * sizeof (WCHAR));
//             type[str->Length] = 0;
//             ret = lpfun.unwrap()(hmod,type,lparam);
//         }
//         else
//         {
//             ret = lpfun.unwrap()( hmod, UIntToPtr!(et[i].Id), lparam );
//         }
//         if (!ret) break;
//     }
//     HeapFree( GetProcessHeap(), 0, type );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	EnumResourceNamesA	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceNamesA( HMODULE hmod, LPCSTR type, ENUMRESNAMEPROCA lpfun, LONG_PTR lparam )
// {
//     int i;
//     BOOL ret = FALSE;
//     DWORD len = 0, newlen;
//     LPSTR name = NULL;
//     NTSTATUS status;
//     UNICODE_STRING typeW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DIRECTORY *basedir, *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//     const IMAGE_RESOURCE_DIR_STRING_U *str;
//
//     TRACE!( "%p %s %p %lx\n", hmod, debugstr_a!(type), lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleA( NULL );
//     typeW.Buffer = NULL;
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &basedir )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameA( type, &typeW )) != STATUS_SUCCESS)
//         goto done;
//     info.Type = (ULONG_PTR)typeW.Buffer;
//     if ((status = LdrFindResourceDirectory_U( hmod, &mut info, 1, &resdir )) != STATUS_SUCCESS)
//         goto done;
//
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     __TRY
//     {
//         for (i = 0; i < resdir->NumberOfNamedEntries+resdir->NumberOfIdEntries; i++)
//         {
//             if (et[i].NameIsString)
//             {
//                 str = (const IMAGE_RESOURCE_DIR_STRING_U *)((const BYTE *)basedir + et[i].NameOffset);
//                 newlen = WideCharToMultiByte(CP_ACP, 0, str->NameString, str->Length, NULL, 0, NULL, NULL);
//                 if (newlen + 1 > len)
//                 {
//                     len = newlen + 1;
//                     HeapFree( GetProcessHeap(), 0, name );
//                     if (!(name = HeapAlloc(GetProcessHeap(), 0, len + 1 )))
//                     {
//                         ret = FALSE;
//                         break;
//                     }
//                 }
//                 WideCharToMultiByte( CP_ACP, 0, str->NameString, str->Length, name, len, NULL, NULL );
//                 name[newlen] = 0;
//                 ret = lpfun.unwrap()(hmod,type,name,lparam);
//             }
//             else
//             {
//                 ret = lpfun.unwrap()( hmod, type, UIntToPtr!(et[i].Id), lparam );
//             }
//             if (!ret) break;
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         ret = FALSE;
//         status = STATUS_ACCESS_VIOLATION;
//     }
//     __ENDTRY;
//
// done:
//     HeapFree( GetProcessHeap(), 0, name );
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	EnumResourceNamesW	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceNamesW( HMODULE hmod, LPCWSTR type, ENUMRESNAMEPROCW lpfun, LONG_PTR lparam )
// {
//     int i, len = 0;
//     BOOL ret = FALSE;
//     LPWSTR name = NULL;
//     NTSTATUS status;
//     UNICODE_STRING typeW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DIRECTORY *basedir, *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//     const IMAGE_RESOURCE_DIR_STRING_U *str;
//
//     TRACE!( "%p %s %p %lx\n", hmod, debugstr_w!(type), lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleW( NULL );
//     typeW.Buffer = NULL;
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &basedir )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameW( type, &typeW )) != STATUS_SUCCESS)
//         goto done;
//     info.Type = (ULONG_PTR)typeW.Buffer;
//     if ((status = LdrFindResourceDirectory_U( hmod, &mut info, 1, &resdir )) != STATUS_SUCCESS)
//         goto done;
//
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     __TRY
//     {
//         for (i = 0; i < resdir->NumberOfNamedEntries+resdir->NumberOfIdEntries; i++)
//         {
//             if (et[i].NameIsString)
//             {
//                 str = (const IMAGE_RESOURCE_DIR_STRING_U *)((const BYTE *)basedir + et[i].NameOffset);
//                 if (str->Length + 1 > len)
//                 {
//                     len = str->Length + 1;
//                     HeapFree( GetProcessHeap(), 0, name );
//                     if (!(name = HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) )))
//                     {
//                         ret = FALSE;
//                         break;
//                     }
//                 }
//                 memcpy(name, str->NameString, str->Length * sizeof (WCHAR));
//                 name[str->Length] = 0;
//                 ret = lpfun.unwrap()(hmod,type,name,lparam);
//             }
//             else
//             {
//                 ret = lpfun.unwrap()( hmod, type, UIntToPtr!(et[i].Id), lparam );
//             }
//             if (!ret) break;
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         ret = FALSE;
//         status = STATUS_ACCESS_VIOLATION;
//     }
//     __ENDTRY
// done:
//     HeapFree( GetProcessHeap(), 0, name );
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	EnumResourceLanguagesA	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceLanguagesA( HMODULE hmod, LPCSTR type, LPCSTR name,
//                                     ENUMRESLANGPROCA lpfun, LONG_PTR lparam )
// {
//     int i;
//     BOOL ret = FALSE;
//     NTSTATUS status;
//     UNICODE_STRING typeW, nameW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DIRECTORY *basedir, *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//
//     TRACE!( "%p %s %s %p %lx\n", hmod, debugstr_a!(type), debugstr_a!(name), lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleA( NULL );
//     typeW.Buffer = nameW.Buffer = NULL;
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &basedir )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameA( type, &typeW )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameA( name, &nameW )) != STATUS_SUCCESS)
//         goto done;
//     info.Type = (ULONG_PTR)typeW.Buffer;
//     info.Name = (ULONG_PTR)nameW.Buffer;
//     if ((status = LdrFindResourceDirectory_U( hmod, &mut info, 2, &resdir )) != STATUS_SUCCESS)
//         goto done;
//
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     __TRY
//     {
//         for (i = 0; i < resdir->NumberOfNamedEntries + resdir->NumberOfIdEntries; i++)
//         {
//             ret = lpfun.unwrap()( hmod, type, name, et[i].Id, lparam );
//             if (!ret) break;
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         ret = FALSE;
//         status = STATUS_ACCESS_VIOLATION;
//     }
//     __ENDTRY
// done:
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     if (!IS_INTRESOURCE!(nameW.Buffer)) HeapFree( GetProcessHeap(), 0, nameW.Buffer );
//     if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	EnumResourceLanguagesW	(KERNEL32.@)
//  */
// BOOL WINAPI EnumResourceLanguagesW( HMODULE hmod, LPCWSTR type, LPCWSTR name,
//                                     ENUMRESLANGPROCW lpfun, LONG_PTR lparam )
// {
//     int i;
//     BOOL ret = FALSE;
//     NTSTATUS status;
//     UNICODE_STRING typeW, nameW;
//     LDR_RESOURCE_INFO info;
//     IMAGE_RESOURCE_DIRECTORY *basedir, *resdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *et;
//
//     TRACE!( "%p %s %s %p %lx\n", hmod, debugstr_w!(type), debugstr_w!(name), lpfun, lparam );
//
//     if (!hmod) hmod = GetModuleHandleW( NULL );
//     typeW.Buffer = nameW.Buffer = NULL;
//     if ((status = LdrFindResourceDirectory_U( hmod, NULL, 0, &basedir )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameW( type, &typeW )) != STATUS_SUCCESS)
//         goto done;
//     if ((status = get_res_nameW( name, &nameW )) != STATUS_SUCCESS)
//         goto done;
//     info.Type = (ULONG_PTR)typeW.Buffer;
//     info.Name = (ULONG_PTR)nameW.Buffer;
//     if ((status = LdrFindResourceDirectory_U( hmod, &mut info, 2, &resdir )) != STATUS_SUCCESS)
//         goto done;
//
//     et = (const IMAGE_RESOURCE_DIRECTORY_ENTRY *)(resdir + 1);
//     __TRY
//     {
//         for (i = 0; i < resdir->NumberOfNamedEntries + resdir->NumberOfIdEntries; i++)
//         {
//             ret = lpfun.unwrap()( hmod, type, name, et[i].Id, lparam );
//             if (!ret) break;
//         }
//     }
//     __EXCEPT_PAGE_FAULT
//     {
//         ret = FALSE;
//         status = STATUS_ACCESS_VIOLATION;
//     }
//     __ENDTRY
// done:
//     if (!IS_INTRESOURCE!(typeW.Buffer)) HeapFree( GetProcessHeap(), 0, typeW.Buffer );
//     if (!IS_INTRESOURCE!(nameW.Buffer)) HeapFree( GetProcessHeap(), 0, nameW.Buffer );
//     if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	    LoadResource     (KERNEL32.@)
//  */
// HGLOBAL WINAPI LoadResource( HINSTANCE hModule, HRSRC hRsrc )
// {
//     NTSTATUS status;
//     void *ret = NULL;
//
//     TRACE!( "%p %p\n", hModule, hRsrc );
//
//     if (!hRsrc) return 0;
//     if (!hModule) hModule = GetModuleHandleA( NULL );
//     status = LdrAccessResource( hModule, (IMAGE_RESOURCE_DATA_ENTRY *)hRsrc, &ret, NULL );
//     if (status != STATUS_SUCCESS) SetLastError( RtlNtStatusToDosError(status) );
//     return ret;
// }
//
//
// /**********************************************************************
//  *	    LockResource     (KERNEL32.@)
//  */
// LPVOID WINAPI LockResource( HGLOBAL handle )
// {
//     return handle;
// }
//
//
// /**********************************************************************
//  *	    FreeResource     (KERNEL32.@)
//  */
// BOOL WINAPI FreeResource( HGLOBAL handle )
// {
//     return FALSE;
// }
//
//
// /**********************************************************************
//  *	    SizeofResource   (KERNEL32.@)
//  */
// DWORD WINAPI SizeofResource( HINSTANCE hModule, HRSRC hRsrc )
// {
//     if (!hRsrc) return 0;
//     return ((PIMAGE_RESOURCE_DATA_ENTRY)hRsrc)->Size;
// }
//
// /*
//  *  Data structure for updating resources.
//  *  Type/Name/Language is a keyset for accessing resource data.
//  *
//  *  QUEUEDUPDATES (root) ->
//  *    list of struct resource_dir_entry    (Type) ->
//  *      list of struct resource_dir_entry  (Name)   ->
//  *         list of struct resource_data    Language + Data
//  */
//
// typedef struct
// {
//     LPWSTR pFileName;
//     BOOL bDeleteExistingResources;
//     struct list root;
// } QUEUEDUPDATES;
//
// /* this structure is shared for types and names */
// struct resource_dir_entry {
//     struct list entry;
//     LPWSTR id;
//     struct list children;
// };
//
// /* this structure is the leaf */
// struct resource_data {
//     struct list entry;
//     LANGID lang;
//     DWORD codepage;
//     DWORD cbData;
//     void *lpData;
// };
//
// static int resource_strcmp( LPCWSTR a, LPCWSTR b )
// {
//     if ( a == b )
//         return 0;
//     if (!IS_INTRESOURCE!( a ) && !IS_INTRESOURCE!( b ) )
//         return lstrcmpW( a, b );
//     /* strings come before ids */
//     if (!IS_INTRESOURCE!( a ) && IS_INTRESOURCE!( b ))
//         return -1;
//     if (!IS_INTRESOURCE!( b ) && IS_INTRESOURCE!( a ))
//         return 1;
//     return ( a < b ) ? -1 : 1;
// }
//
// static struct resource_dir_entry *find_resource_dir_entry( struct list *dir, LPCWSTR id )
// {
//     struct resource_dir_entry *ent;
//
//     /* match either IDs or strings */
//     LIST_FOR_EACH_ENTRY( ent, dir, struct resource_dir_entry, entry )
//         if (!resource_strcmp( id, ent->id ))
//             return ent;
//
//     return NULL;
// }
//
// static struct resource_data *find_resource_data( struct list *dir, LANGID lang )
// {
//     struct resource_data *res_data;
//
//     /* match only languages here */
//     LIST_FOR_EACH_ENTRY( res_data, dir, struct resource_data, entry )
//         if ( lang == res_data->lang )
//              return res_data;
//
//     return NULL;
// }
//
// static void add_resource_dir_entry( struct list *dir, struct resource_dir_entry *resdir )
// {
//     struct resource_dir_entry *ent;
//
//     LIST_FOR_EACH_ENTRY( ent, dir, struct resource_dir_entry, entry )
//     {
//         if (0>resource_strcmp( ent->id, resdir->id ))
//             continue;
//
//         list_add_before( &ent->entry, &resdir->entry );
//         return;
//     }
//     list_add_tail( dir, &resdir->entry );
// }
//
// static void add_resource_data_entry( struct list *dir, struct resource_data *resdata )
// {
//     struct resource_data *ent;
//
//     LIST_FOR_EACH_ENTRY( ent, dir, struct resource_data, entry )
//     {
//         if (ent->lang < resdata->lang)
//             continue;
//
//         list_add_before( &ent->entry, &resdata->entry );
//         return;
//     }
//     list_add_tail( dir, &resdata->entry );
// }
//
// static LPWSTR res_strdupW( LPCWSTR str )
// {
//     LPWSTR ret;
//     UINT len;
//
//     if (IS_INTRESOURCE!(str))
//         return (LPWSTR) (UINT_PTR) LOWORD!(str);
//     len = (lstrlenW( str ) + 1) * sizeof (WCHAR);
//     ret = HeapAlloc( GetProcessHeap(), 0, len );
//     memcpy( ret, str, len );
//     return ret;
// }
//
// static void res_free_str( LPWSTR str )
// {
//     if (!IS_INTRESOURCE!(str))
//         HeapFree( GetProcessHeap(), 0, str );
// }
//
// static BOOL update_add_resource( QUEUEDUPDATES *updates, LPCWSTR Type, LPCWSTR Name,
//                                  LANGID Lang, struct resource_data *resdata,
//                                  BOOL overwrite_existing )
// {
//     struct resource_dir_entry *restype, *resname;
//     struct resource_data *existing;
//
//     TRACE!("%p %s %s %p %d\n", updates,
//           debugstr_w!(Type), debugstr_w!(Name), resdata, overwrite_existing );
//
//     restype = find_resource_dir_entry( &updates->root, Type );
//     if (!restype)
//     {
//         restype = HeapAlloc( GetProcessHeap(), 0, sizeof *restype );
//         restype->id = res_strdupW( Type );
//         list_init( &restype->children );
//         add_resource_dir_entry( &updates->root, restype );
//     }
//
//     resname = find_resource_dir_entry( &restype->children, Name );
//     if (!resname)
//     {
//         resname = HeapAlloc( GetProcessHeap(), 0, sizeof *resname );
//         resname->id = res_strdupW( Name );
//         list_init( &resname->children );
//         add_resource_dir_entry( &restype->children, resname );
//     }
//
//     /*
//      * If there's an existing resource entry with matching (Type,Name,Language)
//      *  it needs to be removed before adding the new data.
//      */
//     existing = find_resource_data( &resname->children, Lang );
//     if (existing)
//     {
//         if (!overwrite_existing)
//             return FALSE;
//         list_remove( &existing->entry );
//         HeapFree( GetProcessHeap(), 0, existing );
//     }
//
//     if (resdata)
//         add_resource_data_entry( &resname->children, resdata );
//
//     return TRUE;
// }
//
// static struct resource_data *allocate_resource_data( WORD Language, DWORD codepage,
//                                                      LPVOID lpData, DWORD cbData, BOOL copy_data )
// {
//     struct resource_data *resdata;
//
//     if (!lpData || !cbData)
//         return NULL;
//
//     resdata = HeapAlloc( GetProcessHeap(), 0, sizeof *resdata + (copy_data ? cbData : 0) );
//     if (resdata)
//     {
//         resdata->lang = Language;
//         resdata->codepage = codepage;
//         resdata->cbData = cbData;
//         if (copy_data)
//         {
//             resdata->lpData = &resdata[1];
//             memcpy( resdata->lpData, lpData, cbData );
//         }
//         else
//             resdata->lpData = lpData;
//     }
//
//     return resdata;
// }
//
// static void free_resource_directory( struct list *head, int level )
// {
//     struct list *ptr = NULL;
//
//     while ((ptr = list_head( head )))
//     {
//         list_remove( ptr );
//         if (level)
//         {
//             struct resource_dir_entry *ent;
//
//             ent = LIST_ENTRY!( ptr, struct resource_dir_entry, entry );
//             res_free_str( ent->id );
//             free_resource_directory( &ent->children, level - 1 );
//             HeapFree(GetProcessHeap(), 0, ent);
//         }
//         else
//         {
//             struct resource_data *data;
//
//             data = LIST_ENTRY!( ptr, struct resource_data, entry );
//             HeapFree( GetProcessHeap(), 0, data );
//         }
//     }
// }
//
// static IMAGE_NT_HEADERS *get_nt_header( void *base, DWORD mapping_size )
// {
//     IMAGE_NT_HEADERS *nt;
//     IMAGE_DOS_HEADER *dos;
//
//     if (mapping_size<sizeof (*dos))
//         return NULL;
//
//     dos = base;
//     if (dos->e_magic != IMAGE_DOS_SIGNATURE)
//         return NULL;
//
//     if ((dos->e_lfanew + sizeof (*nt)) > mapping_size)
//         return NULL;
//
//     nt = (void*) ((BYTE*)base + dos->e_lfanew);
//
//     if (nt->Signature != IMAGE_NT_SIGNATURE)
//         return NULL;
//
//     return nt;
// }
//
// static IMAGE_SECTION_HEADER *get_section_header( void *base, DWORD mapping_size, DWORD *num_sections )
// {
//     IMAGE_NT_HEADERS *nt;
//     DWORD section_ofs;
//
//     nt = get_nt_header( base, mapping_size );
//     if (!nt)
//         return NULL;
//
//     /* check that we don't go over the end of the file accessing the sections */
//     section_ofs = FIELD_OFFSET(IMAGE_NT_HEADERS, OptionalHeader) + nt->FileHeader.SizeOfOptionalHeader;
//     if ((nt->FileHeader.NumberOfSections * sizeof (IMAGE_SECTION_HEADER) + section_ofs) > mapping_size)
//         return NULL;
//
//     if (num_sections)
//         *num_sections = nt->FileHeader.NumberOfSections;
//
//     /* from here we have a valid PE exe to update */
//     return (void*) ((BYTE*)nt + section_ofs);
// }
//
// static BOOL check_pe_exe( HANDLE file, QUEUEDUPDATES *updates )
// {
//     const IMAGE_NT_HEADERS32 *nt;
//     const IMAGE_NT_HEADERS64 *nt64;
//     const IMAGE_SECTION_HEADER *sec;
//     const IMAGE_DATA_DIRECTORY *dd;
//     BOOL ret = FALSE;
//     HANDLE mapping;
//     DWORD mapping_size, num_sections = 0;
//     void *base = NULL;
//
//     mapping_size = GetFileSize( file, NULL );
//
//     mapping = CreateFileMappingW( file, NULL, PAGE_READONLY, 0, 0, NULL );
//     if (!mapping)
//         goto done;
//
//     base = MapViewOfFile( mapping, FILE_MAP_READ, 0, 0, mapping_size );
//     if (!base)
//         goto done;
//
//     nt = (IMAGE_NT_HEADERS32 *)get_nt_header( base, mapping_size );
//     if (!nt)
//         goto done;
//
//     nt64 = (IMAGE_NT_HEADERS64*)nt;
//     dd = &nt->OptionalHeader.DataDirectory[0];
//     if (nt->OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16)
//         dd = &nt64->OptionalHeader.DataDirectory[0];
//
//     TRACE!("resources: %08x %08x\n",
//           dd[IMAGE_DIRECTORY_ENTRY_RESOURCE].VirtualAddress,
//           dd[IMAGE_DIRECTORY_ENTRY_RESOURCE].Size);
//
//     sec = get_section_header( base, mapping_size, &num_sections );
//     if (!sec)
//         goto done;
//
//     ret = TRUE;
//
// done:
//     if (base)
//         UnmapViewOfFile( base );
//     if (mapping)
//         CloseHandle( mapping );
//
//     return ret;
// }
//
// struct resource_size_info {
//     DWORD types_ofs;
//     DWORD names_ofs;
//     DWORD langs_ofs;
//     DWORD data_entry_ofs;
//     DWORD strings_ofs;
//     DWORD data_ofs;
//     DWORD total_size;
// };
//
// struct mapping_info {
//     HANDLE file;
//     void *base;
//     DWORD size;
//     BOOL read_write;
// };
//
// static const IMAGE_SECTION_HEADER *section_from_rva( void *base, DWORD mapping_size, DWORD rva )
// {
//     const IMAGE_SECTION_HEADER *sec;
//     DWORD num_sections = 0;
//     int i;
//
//     sec = get_section_header( base, mapping_size, &num_sections );
//     if (!sec)
//         return NULL;
//
//     for (i=num_sections-1; i>=0; i--)
//     {
//         if (sec[i].VirtualAddress <= rva &&
//             rva <= (DWORD)sec[i].VirtualAddress + sec[i].SizeOfRawData)
//         {
//             return &sec[i];
//         }
//     }
//
//     return NULL;
// }
//
// static void *address_from_rva( void *base, DWORD mapping_size, DWORD rva, DWORD len )
// {
//     const IMAGE_SECTION_HEADER *sec;
//
//     sec = section_from_rva( base, mapping_size, rva );
//     if (!sec)
//         return NULL;
//
//     if (rva + len <= (DWORD)sec->VirtualAddress + sec->SizeOfRawData)
//         return (void*)((LPBYTE) base + (sec->PointerToRawData + rva - sec->VirtualAddress));
//
//     return NULL;
// }
//
// static LPWSTR resource_dup_string( const IMAGE_RESOURCE_DIRECTORY *root, const IMAGE_RESOURCE_DIRECTORY_ENTRY *entry )
// {
//     const IMAGE_RESOURCE_DIR_STRING_U* string;
//     LPWSTR s;
//
//     if (!entry->NameIsString)
//         return UIntToPtr!(entry->Id);
//
//     string = (const IMAGE_RESOURCE_DIR_STRING_U*) (((const char *)root) + entry->NameOffset);
//     s = HeapAlloc(GetProcessHeap(), 0, (string->Length + 1)*sizeof (WCHAR) );
//     memcpy( s, string->NameString, (string->Length + 1)*sizeof (WCHAR) );
//     s[string->Length] = 0;
//
//     return s;
// }
//
// /* this function is based on the code in winedump's pe.c */
// static BOOL enumerate_mapped_resources( QUEUEDUPDATES *updates,
//                              void *base, DWORD mapping_size,
//                              const IMAGE_RESOURCE_DIRECTORY *root )
// {
//     const IMAGE_RESOURCE_DIRECTORY *namedir, *langdir;
//     const IMAGE_RESOURCE_DIRECTORY_ENTRY *e1, *e2, *e3;
//     const IMAGE_RESOURCE_DATA_ENTRY *data;
//     DWORD i, j, k;
//
//     TRACE!("version (%d.%d) %d named %d id entries\n",
//           root->MajorVersion, root->MinorVersion, root->NumberOfNamedEntries, root->NumberOfIdEntries);
//
//     for (i = 0; i< root->NumberOfNamedEntries + root->NumberOfIdEntries; i++)
//     {
//         LPWSTR Type;
//
//         e1 = (const IMAGE_RESOURCE_DIRECTORY_ENTRY*)(root + 1) + i;
//
//         Type = resource_dup_string( root, e1 );
//
//         namedir = (const IMAGE_RESOURCE_DIRECTORY *)((const char *)root + e1->OffsetToDirectory);
//         for (j = 0; j < namedir->NumberOfNamedEntries + namedir->NumberOfIdEntries; j++)
//         {
//             LPWSTR Name;
//
//             e2 = (const IMAGE_RESOURCE_DIRECTORY_ENTRY*)(namedir + 1) + j;
//
//             Name = resource_dup_string( root, e2 );
//
//             langdir = (const IMAGE_RESOURCE_DIRECTORY *)((const char *)root + e2->OffsetToDirectory);
//             for (k = 0; k < langdir->NumberOfNamedEntries + langdir->NumberOfIdEntries; k++)
//             {
//                 LANGID Lang;
//                 void *p;
//                 struct resource_data *resdata;
//
//                 e3 = (const IMAGE_RESOURCE_DIRECTORY_ENTRY*)(langdir + 1) + k;
//
//                 Lang = e3->Id;
//
//                 data = (const IMAGE_RESOURCE_DATA_ENTRY *)((const char *)root + e3->OffsetToData);
//
//                 p = address_from_rva( base, mapping_size, data->OffsetToData, data->Size );
//
//                 resdata = allocate_resource_data( Lang, data->CodePage, p, data->Size, FALSE );
//                 if (resdata)
//                 {
//                     if (!update_add_resource( updates, Type, Name, Lang, resdata, FALSE ))
//                         HeapFree( GetProcessHeap(), 0, resdata );
//                 }
//             }
//             res_free_str( Name );
//         }
//         res_free_str( Type );
//     }
//
//     return TRUE;
// }
//
// static BOOL read_mapped_resources( QUEUEDUPDATES *updates, void *base, DWORD mapping_size )
// {
//     const IMAGE_RESOURCE_DIRECTORY *root;
//     const IMAGE_NT_HEADERS *nt;
//     const IMAGE_SECTION_HEADER *sec;
//     DWORD num_sections = 0, i;
//
//     nt = get_nt_header( base, mapping_size );
//     if (!nt)
//         return FALSE;
//
//     sec = get_section_header( base, mapping_size, &num_sections );
//     if (!sec)
//         return FALSE;
//
//     for (i=0; i<num_sections; i++)
//         if (!memcmp(sec[i].Name, ".rsrc", 6))
//             break;
//
//     if (i == num_sections)
//         return TRUE;
//
//     /* check the resource data is inside the mapping */
//     if (sec[i].PointerToRawData > mapping_size ||
//         (sec[i].PointerToRawData + sec[i].SizeOfRawData) > mapping_size)
//         return TRUE;
//
//     TRACE!("found .rsrc at %08x, size %08x\n", sec[i].PointerToRawData, sec[i].SizeOfRawData);
//
//     if (!sec[i].PointerToRawData || sec[i].SizeOfRawData < sizeof(IMAGE_RESOURCE_DIRECTORY))
//         return TRUE;
//
//     root = (void*) ((BYTE*)base + sec[i].PointerToRawData);
//     enumerate_mapped_resources( updates, base, mapping_size, root );
//
//     return TRUE;
// }
//
// static BOOL map_file_into_memory( struct mapping_info *mi )
// {
//     DWORD page_attr, perm;
//     HANDLE mapping;
//
//     if (mi->read_write)
//     {
//         page_attr = PAGE_READWRITE;
//         perm = FILE_MAP_WRITE | FILE_MAP_READ;
//     }
//     else
//     {
//         page_attr = PAGE_READONLY;
//         perm = FILE_MAP_READ;
//     }
//
//     mapping = CreateFileMappingW( mi->file, NULL, page_attr, 0, 0, NULL );
//     if (!mapping) return FALSE;
//
//     mi->base = MapViewOfFile( mapping, perm, 0, 0, mi->size );
//     CloseHandle( mapping );
//
//     return mi->base != NULL;
// }
//
// static BOOL unmap_file_from_memory( struct mapping_info *mi )
// {
//     if (mi->base)
//         UnmapViewOfFile( mi->base );
//     mi->base = NULL;
//     return TRUE;
// }
//
// static void destroy_mapping( struct mapping_info *mi )
// {
//     if (!mi)
//         return;
//     unmap_file_from_memory( mi );
//     if (mi->file)
//         CloseHandle( mi->file );
//     HeapFree( GetProcessHeap(), 0, mi );
// }
//
// static struct mapping_info *create_mapping( LPCWSTR name, BOOL rw )
// {
//     struct mapping_info *mi;
//
//     mi = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof *mi );
//     if (!mi)
//         return NULL;
//
//     mi->read_write = rw;
//
//     mi->file = CreateFileW( name, GENERIC_READ | (rw ? GENERIC_WRITE : 0),
//                             0, NULL, OPEN_EXISTING, 0, 0 );
//
//     if (mi->file != INVALID_HANDLE_VALUE as HANDLE)
//     {
//         mi->size = GetFileSize( mi->file, NULL );
//
//         if (map_file_into_memory( mi ))
//             return mi;
//     }
//     destroy_mapping( mi );
//     return NULL;
// }
//
// static BOOL resize_mapping( struct mapping_info *mi, DWORD new_size )
// {
//     if (!unmap_file_from_memory( mi ))
//         return FALSE;
//
//     /* change the file size */
//     SetFilePointer( mi->file, new_size, NULL, FILE_BEGIN );
//     if (!SetEndOfFile( mi->file ))
//     {
//         ERR!("failed to set file size to %08x\n", new_size );
//         return FALSE;
//     }
//
//     mi->size = new_size;
//
//     return map_file_into_memory( mi );
// }
//
// static void get_resource_sizes( QUEUEDUPDATES *updates, struct resource_size_info *si )
// {
//     struct resource_dir_entry *types, *names;
//     struct resource_data *data;
//     DWORD num_types = 0, num_names = 0, num_langs = 0, strings_size = 0, data_size = 0;
//
//     memset( si, 0, sizeof *si );
//
//     LIST_FOR_EACH_ENTRY( types, &updates->root, struct resource_dir_entry, entry )
//     {
//         num_types++;
//         if (!IS_INTRESOURCE!( types->id ))
//             strings_size += sizeof (WORD) + lstrlenW( types->id )*sizeof (WCHAR);
//
//         LIST_FOR_EACH_ENTRY( names, &types->children, struct resource_dir_entry, entry )
//         {
//             num_names++;
//
//             if (!IS_INTRESOURCE!( names->id ))
//                 strings_size += sizeof (WORD) + lstrlenW( names->id )*sizeof (WCHAR);
//
//             LIST_FOR_EACH_ENTRY( data, &names->children, struct resource_data, entry )
//             {
//                 num_langs++;
//                 data_size += (data->cbData + 3) & ~3;
//             }
//         }
//     }
//
//     /* names are at the end of the types */
//     si->names_ofs = sizeof (IMAGE_RESOURCE_DIRECTORY) +
//             num_types * sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//     /* language directories are at the end of the names */
//     si->langs_ofs = si->names_ofs +
//             num_types * sizeof (IMAGE_RESOURCE_DIRECTORY) +
//             num_names * sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//     si->data_entry_ofs = si->langs_ofs +
//             num_names * sizeof (IMAGE_RESOURCE_DIRECTORY) +
//             num_langs * sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//     si->strings_ofs = si->data_entry_ofs +
//             num_langs * sizeof (IMAGE_RESOURCE_DATA_ENTRY);
//
//     si->data_ofs = si->strings_ofs + ((strings_size + 3) & ~3);
//
//     si->total_size = si->data_ofs + data_size;
//
//     TRACE!("names %08x langs %08x data entries %08x strings %08x data %08x total %08x\n",
//           si->names_ofs, si->langs_ofs, si->data_entry_ofs,
//           si->strings_ofs, si->data_ofs, si->total_size);
// }
//
// static void res_write_padding( BYTE *res_base, DWORD size )
// {
//     static const BYTE pad[] = {
//         'P','A','D','D','I','N','G','X','X','P','A','D','D','I','N','G' };
//     DWORD i;
//
//     for ( i = 0; i < size / sizeof pad; i++ )
//         memcpy( &res_base[i*sizeof pad], pad, sizeof pad );
//     memcpy( &res_base[i*sizeof pad], pad, size%sizeof pad );
// }
//
// static BOOL write_resources( QUEUEDUPDATES *updates, LPBYTE base, struct resource_size_info *si, DWORD rva )
// {
//     struct resource_dir_entry *types, *names;
//     struct resource_data *data;
//     IMAGE_RESOURCE_DIRECTORY *root;
//
//     TRACE!("%p %p %p %08x\n", updates, base, si, rva );
//
//     memset( base, 0, si->total_size );
//
//     /* the root entry always exists */
//     root = (IMAGE_RESOURCE_DIRECTORY*) base;
//     memset( root, 0, sizeof *root );
//     root->MajorVersion = 4;
//     si->types_ofs = sizeof *root;
//     LIST_FOR_EACH_ENTRY( types, &updates->root, struct resource_dir_entry, entry )
//     {
//         IMAGE_RESOURCE_DIRECTORY_ENTRY *e1;
//         IMAGE_RESOURCE_DIRECTORY *namedir;
//
//         e1 = (IMAGE_RESOURCE_DIRECTORY_ENTRY*) &base[si->types_ofs];
//         memset( e1, 0, sizeof *e1 );
//         if (!IS_INTRESOURCE!( types->id ))
//         {
//             WCHAR *strings;
//             DWORD len;
//
//             root->NumberOfNamedEntries++;
//             e1->NameIsString = 1;
//             e1->NameOffset = si->strings_ofs;
//
//             strings = (WCHAR*) &base[si->strings_ofs];
//             len = lstrlenW( types->id );
//             strings[0] = len;
//             memcpy( &strings[1], types->id, len * sizeof (WCHAR) );
//             si->strings_ofs += (len + 1) * sizeof (WCHAR);
//         }
//         else
//         {
//             root->NumberOfIdEntries++;
//             e1->Id = LOWORD!( types->id );
//         }
//         e1->OffsetToDirectory = si->names_ofs;
//         e1->DataIsDirectory = TRUE;
//         si->types_ofs += sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//         namedir = (IMAGE_RESOURCE_DIRECTORY*) &base[si->names_ofs];
//         memset( namedir, 0, sizeof *namedir );
//         namedir->MajorVersion = 4;
//         si->names_ofs += sizeof (IMAGE_RESOURCE_DIRECTORY);
//
//         LIST_FOR_EACH_ENTRY( names, &types->children, struct resource_dir_entry, entry )
//         {
//             IMAGE_RESOURCE_DIRECTORY_ENTRY *e2;
//             IMAGE_RESOURCE_DIRECTORY *langdir;
//
//             e2 = (IMAGE_RESOURCE_DIRECTORY_ENTRY*) &base[si->names_ofs];
//             memset( e2, 0, sizeof *e2 );
//             if (!IS_INTRESOURCE!( names->id ))
//             {
//                 WCHAR *strings;
//                 DWORD len;
//
//                 namedir->NumberOfNamedEntries++;
//                 e2->NameIsString = 1;
//                 e2->NameOffset = si->strings_ofs;
//
//                 strings = (WCHAR*) &base[si->strings_ofs];
//                 len = lstrlenW( names->id );
//                 strings[0] = len;
//                 memcpy( &strings[1], names->id, len * sizeof (WCHAR) );
//                 si->strings_ofs += (len + 1) * sizeof (WCHAR);
//             }
//             else
//             {
//                 namedir->NumberOfIdEntries++;
//                 e2->Id = LOWORD!( names->id );
//             }
//             e2->OffsetToDirectory = si->langs_ofs;
//             e2->DataIsDirectory = TRUE;
//             si->names_ofs += sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//             langdir = (IMAGE_RESOURCE_DIRECTORY*) &base[si->langs_ofs];
//             memset( langdir, 0, sizeof *langdir );
//             langdir->MajorVersion = 4;
//             si->langs_ofs += sizeof (IMAGE_RESOURCE_DIRECTORY);
//
//             LIST_FOR_EACH_ENTRY( data, &names->children, struct resource_data, entry )
//             {
//                 IMAGE_RESOURCE_DIRECTORY_ENTRY *e3;
//                 IMAGE_RESOURCE_DATA_ENTRY *de;
//                 int pad_size;
//
//                 e3 = (IMAGE_RESOURCE_DIRECTORY_ENTRY*) &base[si->langs_ofs];
//                 memset( e3, 0, sizeof *e3 );
//                 langdir->NumberOfIdEntries++;
//                 e3->Id = LOWORD!( data->lang );
//                 e3->OffsetToData = si->data_entry_ofs;
//
//                 si->langs_ofs += sizeof (IMAGE_RESOURCE_DIRECTORY_ENTRY);
//
//                 /* write out all the data entries */
//                 de = (IMAGE_RESOURCE_DATA_ENTRY*) &base[si->data_entry_ofs];
//                 memset( de, 0, sizeof *de );
//                 de->OffsetToData = si->data_ofs + rva;
//                 de->Size = data->cbData;
//                 de->CodePage = data->codepage;
//                 si->data_entry_ofs += sizeof (IMAGE_RESOURCE_DATA_ENTRY);
//
//                 /* write out the resource data */
//                 memcpy( &base[si->data_ofs], data->lpData, data->cbData );
//                 si->data_ofs += data->cbData;
//
//                 pad_size = (-si->data_ofs)&3;
//                 res_write_padding( &base[si->data_ofs], pad_size );
//                 si->data_ofs += pad_size;
//             }
//         }
//     }
//
//     return TRUE;
// }
//
// /*
//  *  FIXME:
//  *  Assumes that the resources are in .rsrc
//  *   and .rsrc is the last section in the file.
//  *  Not sure whether updating resources will other cases on Windows.
//  *  If the resources lie in a section containing other data,
//  *   resizing that section could possibly cause trouble.
//  *  If the section with the resources isn't last, the remaining
//  *   sections need to be moved down in the file, and the section header
//  *   would need to be adjusted.
//  *  If we needed to add a section, what would we name it?
//  *  If we needed to add a section and there wasn't space in the file
//  *   header, how would that work?
//  *  Seems that at least some of these cases can't be handled properly.
//  */
// static IMAGE_SECTION_HEADER *get_resource_section( void *base, DWORD mapping_size )
// {
//     IMAGE_SECTION_HEADER *sec;
//     IMAGE_NT_HEADERS *nt;
//     DWORD i, num_sections = 0;
//
//     nt = get_nt_header( base, mapping_size );
//     if (!nt)
//         return NULL;
//
//     sec = get_section_header( base, mapping_size, &num_sections );
//     if (!sec)
//         return NULL;
//
//     /* find the resources section */
//     for (i=0; i<num_sections; i++)
//         if (!memcmp(sec[i].Name, ".rsrc", 6))
//             break;
//
//     if (i == num_sections)
//         return NULL;
//
//     return &sec[i];
// }
//
// static DWORD get_init_data_size( void *base, DWORD mapping_size )
// {
//     DWORD i, sz = 0, num_sections = 0;
//     IMAGE_SECTION_HEADER *s;
//
//     s = get_section_header( base, mapping_size, &num_sections );
//
//     for (i=0; i<num_sections; i++)
//         if (s[i].Characteristics & IMAGE_SCN_CNT_INITIALIZED_DATA)
//             sz += s[i].SizeOfRawData;
//
//     TRACE!("size = %08x\n", sz);
//
//     return sz;
// }
//
// static BOOL write_raw_resources( QUEUEDUPDATES *updates )
// {
//     static const WCHAR prefix[] = { 'r','e','s','u',0 };
//     WCHAR tempdir[MAX_PATH], tempfile[MAX_PATH];
//     DWORD i, section_size;
//     BOOL ret = FALSE;
//     IMAGE_SECTION_HEADER *sec;
//     IMAGE_NT_HEADERS32 *nt;
//     IMAGE_NT_HEADERS64 *nt64;
//     struct resource_size_info res_size;
//     BYTE *res_base;
//     struct mapping_info *read_map = NULL, *write_map = NULL;
//     DWORD PeSectionAlignment, PeFileAlignment, PeSizeOfImage;
//
//     /* copy the exe to a temp file then update the temp file... */
//     tempdir[0] = 0;
//     if (!GetTempPathW( MAX_PATH, tempdir ))
//         return ret;
//
//     if (!GetTempFileNameW( tempdir, prefix, 0, tempfile ))
//         return ret;
//
//     if (!CopyFileW( updates->pFileName, tempfile, FALSE ))
//         goto done;
//
//     TRACE!("tempfile %s\n", debugstr_w!(tempfile));
//
//     if (!updates->bDeleteExistingResources)
//     {
//         read_map = create_mapping( updates->pFileName, FALSE );
//         if (!read_map)
//             goto done;
//
//         ret = read_mapped_resources( updates, read_map->base, read_map->size );
//         if (!ret)
//         {
//             ERR!("failed to read existing resources\n");
//             goto done;
//         }
//     }
//
//     write_map = create_mapping( tempfile, TRUE );
//     if (!write_map)
//         goto done;
//
//     nt = (IMAGE_NT_HEADERS32*)get_nt_header( write_map->base, write_map->size );
//     if (!nt)
//         goto done;
//
//     nt64 = (IMAGE_NT_HEADERS64*)nt;
//     if (nt->OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16) {
//         PeSectionAlignment = nt64->OptionalHeader.SectionAlignment;
//         PeFileAlignment = nt64->OptionalHeader.FileAlignment;
//         PeSizeOfImage = nt64->OptionalHeader.SizeOfImage;
//     } else {
//         PeSectionAlignment = nt->OptionalHeader.SectionAlignment;
//         PeFileAlignment = nt->OptionalHeader.FileAlignment;
//         PeSizeOfImage = nt->OptionalHeader.SizeOfImage;
//     }
//
//     if ((LONG)PeSectionAlignment <= 0)
//     {
//         ERR!("invalid section alignment %08x\n", PeSectionAlignment);
//         goto done;
//     }
//
//     if ((LONG)PeFileAlignment <= 0)
//     {
//         ERR!("invalid file alignment %08x\n", PeFileAlignment);
//         goto done;
//     }
//
//     sec = get_resource_section( write_map->base, write_map->size );
//     if (!sec) /* no section, add one */
//     {
//         DWORD num_sections;
//
//         sec = get_section_header( write_map->base, write_map->size, &num_sections );
//         if (!sec)
//             goto done;
//
//         sec += num_sections;
//         nt->FileHeader.NumberOfSections++;
//
//         memset( sec, 0, sizeof *sec );
//         memcpy( sec->Name, ".rsrc", 5 );
//         sec->Characteristics = IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ;
//         sec->VirtualAddress = PeSizeOfImage;
//     }
//
//     if (!sec->PointerToRawData)  /* empty section */
//     {
//         sec->PointerToRawData = write_map->size + (-write_map->size) % PeFileAlignment;
//         sec->SizeOfRawData = 0;
//     }
//
//     TRACE!("before .rsrc at %08x, size %08x\n", sec->PointerToRawData, sec->SizeOfRawData);
//
//     get_resource_sizes( updates, &res_size );
//
//     /* round up the section size */
//     section_size = res_size.total_size;
//     section_size += (-section_size) % PeFileAlignment;
//
//     TRACE!("requires %08x (%08x) bytes\n", res_size.total_size, section_size );
//
//     /* check if the file size needs to be changed */
//     if (section_size != sec->SizeOfRawData)
//     {
//         DWORD old_size = write_map->size;
//         DWORD virtual_section_size = res_size.total_size + (-res_size.total_size) % PeSectionAlignment;
//         int delta = section_size - (sec->SizeOfRawData + (-sec->SizeOfRawData) % PeFileAlignment);
//         int rva_delta = virtual_section_size -
//             (sec->Misc.VirtualSize + (-sec->Misc.VirtualSize) % PeSectionAlignment);
//         /* when new section is added it could end past current mapping size */
//         BOOL rsrc_is_last = sec->PointerToRawData + sec->SizeOfRawData >= old_size;
// 	/* align .rsrc size when possible */
//         DWORD mapping_size = rsrc_is_last ? sec->PointerToRawData + section_size : old_size + delta;
//
//         /* postpone file truncation if there are some data to be moved down from file end */
//         BOOL resize_after = mapping_size < old_size && !rsrc_is_last;
//
//         TRACE!("file size %08x -> %08x\n", old_size, mapping_size);
//
//         if (!resize_after)
//         {
//             /* unmap the file before changing the file size */
//             ret = resize_mapping( write_map, mapping_size );
//
//             /* get the pointers again - they might be different after remapping */
//             nt = (IMAGE_NT_HEADERS32*)get_nt_header( write_map->base, mapping_size );
//             if (!nt)
//             {
//                 ERR!("couldn't get NT header\n");
//                 goto done;
//             }
//             nt64 = (IMAGE_NT_HEADERS64*)nt;
//
//             sec = get_resource_section( write_map->base, mapping_size );
//             if (!sec)
//                  goto done;
//         }
//
//         if (!rsrc_is_last) /* not last section, relocate trailing sections */
//         {
//             IMAGE_SECTION_HEADER *s;
//             DWORD tail_start = sec->PointerToRawData + sec->SizeOfRawData;
//             DWORD i, num_sections = 0;
//
//             memmove( (char*)write_map->base + tail_start + delta, (char*)write_map->base + tail_start, old_size - tail_start );
//
//             s = get_section_header( write_map->base, mapping_size, &num_sections );
//
//             for (i=0; i<num_sections; i++)
//             {
//                 if (s[i].PointerToRawData > sec->PointerToRawData)
//                 {
//                     s[i].PointerToRawData += delta;
//                     s[i].VirtualAddress += rva_delta;
//                 }
//             }
//         }
//
//         if (resize_after)
//         {
//             ret = resize_mapping( write_map, mapping_size );
//
//             nt = (IMAGE_NT_HEADERS32*)get_nt_header( write_map->base, mapping_size );
//             if (!nt)
//             {
//                 ERR!("couldn't get NT header\n");
//                 goto done;
//             }
//             nt64 = (IMAGE_NT_HEADERS64*)nt;
//
//             sec = get_resource_section( write_map->base, mapping_size );
//             if (!sec)
//                  goto done;
//         }
//
//         /* adjust the PE header information */
//         sec->SizeOfRawData = section_size;
//         sec->Misc.VirtualSize = virtual_section_size;
//         if (nt->OptionalHeader.Magic == IMAGE_NT_OPTIONAL_HDR64_MAGIC as u16) {
//             nt64->OptionalHeader.SizeOfImage += rva_delta;
//             nt64->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE].VirtualAddress = sec->VirtualAddress;
//             nt64->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE].Size = res_size.total_size;
//             nt64->OptionalHeader.SizeOfInitializedData = get_init_data_size( write_map->base, mapping_size );
//
//             for (i=0; i<nt64->OptionalHeader.NumberOfRvaAndSizes; i++)
//                 if (nt64->OptionalHeader.DataDirectory[i].VirtualAddress > sec->VirtualAddress)
//                     nt64->OptionalHeader.DataDirectory[i].VirtualAddress += rva_delta;
//         } else {
//             nt->OptionalHeader.SizeOfImage += rva_delta;
//             nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE].VirtualAddress = sec->VirtualAddress;
//             nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_RESOURCE].Size = res_size.total_size;
//             nt->OptionalHeader.SizeOfInitializedData = get_init_data_size( write_map->base, mapping_size );
//
//             for (i=0; i<nt->OptionalHeader.NumberOfRvaAndSizes; i++)
//                 if (nt->OptionalHeader.DataDirectory[i].VirtualAddress > sec->VirtualAddress)
//                     nt->OptionalHeader.DataDirectory[i].VirtualAddress += rva_delta;
//         }
//     }
//
//     res_base = (LPBYTE) write_map->base + sec->PointerToRawData;
//
//     TRACE!("base = %p offset = %08x\n", write_map->base, sec->PointerToRawData);
//
//     ret = write_resources( updates, res_base, &res_size, sec->VirtualAddress );
//
//     res_write_padding( res_base + res_size.total_size, section_size - res_size.total_size );
//
//     TRACE!("after  .rsrc at %08x, size %08x\n", sec->PointerToRawData, sec->SizeOfRawData);
//
// done:
//     destroy_mapping( read_map );
//     destroy_mapping( write_map );
//
//     if (ret)
//         ret = CopyFileW( tempfile, updates->pFileName, FALSE );
//
//     DeleteFileW( tempfile );
//
//     return ret;
// }
//
// /***********************************************************************
//  *          BeginUpdateResourceW                 (KERNEL32.@)
//  */
// HANDLE WINAPI BeginUpdateResourceW( LPCWSTR pFileName, BOOL bDeleteExistingResources )
// {
//     QUEUEDUPDATES *updates = NULL;
//     HANDLE hUpdate, file, ret = NULL;
//
//     TRACE!("%s, %d\n", debugstr_w!(pFileName), bDeleteExistingResources);
//
//     hUpdate = GlobalAlloc(GHND, sizeof(QUEUEDUPDATES));
//     if (!hUpdate)
//         return ret;
//
//     updates = GlobalLock(hUpdate);
//     if (updates)
//     {
//         list_init( &updates->root );
//         updates->bDeleteExistingResources = bDeleteExistingResources;
//         updates->pFileName = HeapAlloc(GetProcessHeap(), 0, (lstrlenW(pFileName)+1)*sizeof(WCHAR));
//         if (updates->pFileName)
//         {
//             lstrcpyW(updates->pFileName, pFileName);
//
//             file = CreateFileW( pFileName, GENERIC_READ | GENERIC_WRITE,
//                                 0, NULL, OPEN_EXISTING, 0, 0 );
//
//             /* if resources are deleted, only the file's presence is checked */
//             if (file != INVALID_HANDLE_VALUE as HANDLE &&
//                 (bDeleteExistingResources || check_pe_exe( file, updates )))
//                 ret = hUpdate;
//             else
//                 HeapFree( GetProcessHeap(), 0, updates->pFileName );
//
//             CloseHandle( file );
//         }
//         GlobalUnlock(hUpdate);
//     }
//
//     if (!ret)
//         GlobalFree(hUpdate);
//
//     return ret;
// }
//
//
// /***********************************************************************
//  *          BeginUpdateResourceA                 (KERNEL32.@)
//  */
// HANDLE WINAPI BeginUpdateResourceA( LPCSTR pFileName, BOOL bDeleteExistingResources )
// {
//     UNICODE_STRING FileNameW;
//     HANDLE ret;
//     RtlCreateUnicodeStringFromAsciiz(&FileNameW, pFileName);
//     ret = BeginUpdateResourceW(FileNameW.Buffer, bDeleteExistingResources);
//     RtlFreeUnicodeString(&FileNameW);
//     return ret;
// }
//
//
// /***********************************************************************
//  *          EndUpdateResourceW                 (KERNEL32.@)
//  */
// BOOL WINAPI EndUpdateResourceW( HANDLE hUpdate, BOOL fDiscard )
// {
//     QUEUEDUPDATES *updates;
//     BOOL ret;
//
//     TRACE!("%p %d\n", hUpdate, fDiscard);
//
//     updates = GlobalLock(hUpdate);
//     if (!updates)
//         return FALSE;
//
//     ret = fDiscard || write_raw_resources( updates );
//
//     free_resource_directory( &updates->root, 2 );
//
//     HeapFree( GetProcessHeap(), 0, updates->pFileName );
//     GlobalUnlock( hUpdate );
//     GlobalFree( hUpdate );
//
//     return ret;
// }
//
//
// /***********************************************************************
//  *          EndUpdateResourceA                 (KERNEL32.@)
//  */
// BOOL WINAPI EndUpdateResourceA( HANDLE hUpdate, BOOL fDiscard )
// {
//     return EndUpdateResourceW(hUpdate, fDiscard);
// }
//
//
// /***********************************************************************
//  *           UpdateResourceW                 (KERNEL32.@)
//  */
// BOOL WINAPI UpdateResourceW( HANDLE hUpdate, LPCWSTR lpType, LPCWSTR lpName,
//                              WORD wLanguage, LPVOID lpData, DWORD cbData)
// {
//     QUEUEDUPDATES *updates;
//     BOOL ret = FALSE;
//
//     TRACE!("%p %s %s %08x %p %d\n", hUpdate,
//           debugstr_w!(lpType), debugstr_w!(lpName), wLanguage, lpData, cbData);
//
//     updates = GlobalLock(hUpdate);
//     if (updates)
//     {
//         if (lpData == NULL && cbData == 0)  /* remove resource */
//         {
//             ret = update_add_resource( updates, lpType, lpName, wLanguage, NULL, TRUE );
//         }
//         else
//         {
//             struct resource_data *data;
//             data = allocate_resource_data( wLanguage, 0, lpData, cbData, TRUE );
//             if (data)
//                 ret = update_add_resource( updates, lpType, lpName, wLanguage, data, TRUE );
//         }
//         GlobalUnlock(hUpdate);
//     }
//     return ret;
// }
//
//
// /***********************************************************************
//  *           UpdateResourceA                 (KERNEL32.@)
//  */
// BOOL WINAPI UpdateResourceA( HANDLE hUpdate, LPCSTR lpType, LPCSTR lpName,
//                              WORD wLanguage, LPVOID lpData, DWORD cbData)
// {
//     BOOL ret;
//     UNICODE_STRING TypeW;
//     UNICODE_STRING NameW;
//     if(IS_INTRESOURCE!(lpType))
//         TypeW.Buffer = ULongToPtr!(LOWORD!(lpType));
//     else
//         RtlCreateUnicodeStringFromAsciiz(&TypeW, lpType);
//     if(IS_INTRESOURCE!(lpName))
//         NameW.Buffer = ULongToPtr!(LOWORD!(lpName));
//     else
//         RtlCreateUnicodeStringFromAsciiz(&NameW, lpName);
//     ret = UpdateResourceW(hUpdate, TypeW.Buffer, NameW.Buffer, wLanguage, lpData, cbData);
//     if(!IS_INTRESOURCE!(lpType)) RtlFreeUnicodeString(&TypeW);
//     if(!IS_INTRESOURCE!(lpName)) RtlFreeUnicodeString(&NameW);
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
