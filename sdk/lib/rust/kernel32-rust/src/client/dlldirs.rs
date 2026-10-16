//! Windows 8 DLL search directories: AddDllDirectory, RemoveDllDirectory,
//! SetDefaultDllDirectories, and the LOAD_LIBRARY_SEARCH_* search path that
//! LoadLibraryExW builds from them.
//!
//! Windows keeps these in ntdll (LdrAddDllDirectory & co.). Ours computes every DLL
//! search path in kernel32 (BaseComputeProcessDllPath), so the directory list lives
//! here beside it, under the lock that already guards SetDllDirectory's directory.
//! Semantics follow Wine's ntdll/loader.c and the Windows behaviour its tests pin:
//! AddDllDirectory takes an absolute path to something that exists (a relative path
//! is ERROR_INVALID_PARAMETER, a missing one ERROR_FILE_NOT_FOUND); the most recently
//! added directory is searched first; DEFAULT_DIRS means application dir, user dirs,
//! System32.
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use core::ptr::{copy_nonoverlapping, null_mut};

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const LOAD_WITH_ALTERED_SEARCH_PATH_: DWORD = 0x0000_0008;
const LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR: DWORD = 0x0000_0100;
const LOAD_LIBRARY_SEARCH_APPLICATION_DIR: DWORD = 0x0000_0200;
const LOAD_LIBRARY_SEARCH_USER_DIRS: DWORD = 0x0000_0400;
const LOAD_LIBRARY_SEARCH_SYSTEM32: DWORD = 0x0000_0800;
const LOAD_LIBRARY_SEARCH_DEFAULT_DIRS: DWORD = 0x0000_1000;
const SEARCH_FLAGS: DWORD = LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR
    | LOAD_LIBRARY_SEARCH_APPLICATION_DIR
    | LOAD_LIBRARY_SEARCH_USER_DIRS
    | LOAD_LIBRARY_SEARCH_SYSTEM32
    | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS;
// DLL_LOAD_DIR names the directory of the DLL being loaded: meaningless as a default.
const DEFAULT_FLAGS_ALLOWED: DWORD = SEARCH_FLAGS & !LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR;

// RTL_PATH_TYPE values (RtlDetermineDosPathNameType_U).
const RTL_PATH_TYPE_UNC_ABSOLUTE: u32 = 1;
const RTL_PATH_TYPE_DRIVE_ABSOLUTE: u32 = 2;
const RTL_PATH_TYPE_ROOTED: u32 = 4;
const RTL_PATH_TYPE_LOCAL_DEVICE: u32 = 6;

const STATUS_INVALID_PARAMETER_: NTSTATUS = 0xC000_000D_u32 as NTSTATUS;
const STATUS_NO_MEMORY_: NTSTATUS = 0xC000_0017_u32 as NTSTATUS;

extern "system" {
    fn RtlDosPathNameToNtPathName_U_WithStatus(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: *mut c_void,
    ) -> NTSTATUS;
}

extern "C" {
    static mut BaseDllDirectoryLock: RTL_CRITICAL_SECTION;
    static mut BaseDllDirectory: UNICODE_STRING;
    static mut BaseWindowsSystemDirectory: UNICODE_STRING;
}

/// One AddDllDirectory entry; its address is the DLL_DIRECTORY_COOKIE. The DOS path
/// (NUL-terminated) follows the header in the same allocation.
#[repr(C)]
struct DllDir {
    next: *mut DllDir,
    len: usize, // in WCHARs, without the NUL
}

impl DllDir {
    unsafe fn path(this: *mut DllDir) -> *mut WCHAR {
        (this as *mut u8).add(core::mem::size_of::<DllDir>()) as *mut WCHAR
    }
}

/// Newest first, guarded by BaseDllDirectoryLock.
static mut DLL_DIRS: *mut DllDir = null_mut();
/// SetDefaultDllDirectories flags; 0 = the legacy search order.
static mut DEFAULT_FLAGS: DWORD = 0;

unsafe fn wcslen(s: *const WCHAR) -> usize {
    let mut n = 0;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// Wine's get_module_path_end: the length of the directory part of `path`, keeping
/// the backslash of a drive root ("C:\a.dll" -> "C:\", "C:a.dll" -> "C:").
unsafe fn dir_len(path: *const WCHAR) -> usize {
    let n = wcslen(path);
    let mut end = 0;
    for i in 0..n {
        let c = *path.add(i);
        if c == b'\\' as u16 || c == b'/' as u16 {
            end = i;
        }
    }
    if end == 2 && *path.add(1) == b':' as u16 {
        end += 1;
    }
    if end == 0 && n >= 2 && *path.add(1) == b':' as u16 {
        end = 2;
    }
    end
}

unsafe fn append(p: *mut WCHAR, src: *const WCHAR, len: usize) -> *mut WCHAR {
    if len == 0 {
        return p;
    }
    copy_nonoverlapping(src, p, len);
    *p.add(len) = b';' as u16;
    p.add(len + 1)
}

/// The search path LoadLibraryExW should hand LdrLoadDll for `module` and `flags`,
/// when LOAD_LIBRARY_SEARCH_* flags (passed, or set as the process default) apply.
/// Ok(null) means none apply and the legacy path is used; Err is the NTSTATUS to fail
/// the load with. The buffer is from the process heap, freed by the caller.
pub(crate) unsafe fn search_flags_path(module: PCWSTR, flags: DWORD) -> Result<PWSTR, NTSTATUS> {
    let mut flags = flags;
    let default = DEFAULT_FLAGS;
    if flags & LOAD_WITH_ALTERED_SEARCH_PATH_ != 0 {
        if flags & SEARCH_FLAGS != 0 {
            return Err(STATUS_INVALID_PARAMETER_);
        }
        if default != 0 {
            flags |= default | LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR;
        }
    } else if flags & SEARCH_FLAGS == 0 {
        flags |= default;
    }
    if flags & SEARCH_FLAGS == 0 {
        return Ok(null_mut());
    }
    if flags & LOAD_LIBRARY_SEARCH_DEFAULT_DIRS != 0 {
        flags |= LOAD_LIBRARY_SEARCH_APPLICATION_DIR
            | LOAD_LIBRARY_SEARCH_USER_DIRS
            | LOAD_LIBRARY_SEARCH_SYSTEM32;
    }

    let mut module_len = 0;
    if flags & LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR != 0 {
        let ty = RtlDetermineDosPathNameType_U(module) as u32;
        if ty != RTL_PATH_TYPE_DRIVE_ABSOLUTE
            && ty != RTL_PATH_TYPE_ROOTED
            && ty != RTL_PATH_TYPE_LOCAL_DEVICE
            && ty != RTL_PATH_TYPE_UNC_ABSOLUTE
        {
            return Err(STATUS_INVALID_PARAMETER_);
        }
        module_len = dir_len(module);
    }
    let image = (*(*NtCurrentPeb()).ProcessParameters).ImagePathName.Buffer as *const WCHAR;
    let image_len = if flags & LOAD_LIBRARY_SEARCH_APPLICATION_DIR != 0 && !image.is_null() {
        dir_len(image)
    } else {
        0
    };
    let sys = &raw const BaseWindowsSystemDirectory;
    let sys_len = if flags & LOAD_LIBRARY_SEARCH_SYSTEM32 != 0 {
        (*sys).Length as usize / 2
    } else {
        0
    };

    RtlEnterCriticalSection(&raw mut BaseDllDirectoryLock);
    let user = flags & LOAD_LIBRARY_SEARCH_USER_DIRS != 0;
    let dll_dir = &raw const BaseDllDirectory;
    let mut len = 1 + module_len + 1 + image_len + 1 + sys_len;
    if user {
        let mut d = DLL_DIRS;
        while !d.is_null() {
            len += (*d).len + 1;
            d = (*d).next;
        }
        len += (*dll_dir).Length as usize / 2 + 1;
    }
    let buf = RtlAllocateHeap(RtlGetProcessHeap(), 0, (len * 2) as SIZE_T) as *mut WCHAR;
    if buf.is_null() {
        RtlLeaveCriticalSection(&raw mut BaseDllDirectoryLock);
        return Err(STATUS_NO_MEMORY_);
    }
    let mut p = buf;
    p = append(p, module, module_len);
    p = append(p, image, image_len);
    if user {
        let mut d = DLL_DIRS;
        while !d.is_null() {
            p = append(p, DllDir::path(d), (*d).len);
            d = (*d).next;
        }
        p = append(p, (*dll_dir).Buffer, (*dll_dir).Length as usize / 2);
    }
    RtlLeaveCriticalSection(&raw mut BaseDllDirectoryLock);
    if sys_len != 0 {
        copy_nonoverlapping((*sys).Buffer as *const WCHAR, p, sys_len);
        p = p.add(sys_len);
    } else if p > buf {
        p = p.sub(1); // drop the trailing ';'
    }
    *p = 0;
    Ok(buf)
}

#[no_mangle]
pub unsafe extern "system" fn AddDllDirectory(NewDirectory: PCWSTR) -> *mut c_void {
    if NewDirectory.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return null_mut();
    }
    let ty = RtlDetermineDosPathNameType_U(NewDirectory) as u32;
    if ty != RTL_PATH_TYPE_ROOTED
        && ty != RTL_PATH_TYPE_DRIVE_ABSOLUTE
        && ty != RTL_PATH_TYPE_UNC_ABSOLUTE
        && ty != RTL_PATH_TYPE_LOCAL_DEVICE
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return null_mut();
    }
    let mut nt_name: UNICODE_STRING = core::mem::zeroed();
    let status =
        RtlDosPathNameToNtPathName_U_WithStatus(NewDirectory, &mut nt_name, null_mut(), null_mut());
    if !NT_SUCCESS(status) {
        BaseSetLastNTError(status);
        return null_mut();
    }
    let mut info: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut attr: OBJECT_ATTRIBUTES = core::mem::zeroed();
    attr.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    attr.ObjectName = &mut nt_name;
    attr.Attributes = OBJ_CASE_INSENSITIVE;
    let status = NtQueryAttributesFile(&mut attr, &mut info);
    if !NT_SUCCESS(status) {
        RtlFreeUnicodeString(&mut nt_name);
        BaseSetLastNTError(status);
        return null_mut();
    }
    // The search path wants the DOS form: the NT name less its "\??\" prefix.
    let mut src = nt_name.Buffer as *const WCHAR;
    let mut n = nt_name.Length as usize / 2;
    if n >= 4
        && *src == b'\\' as u16
        && *src.add(1) == b'?' as u16
        && *src.add(2) == b'?' as u16
        && *src.add(3) == b'\\' as u16
    {
        src = src.add(4);
        n -= 4;
    }
    let size = core::mem::size_of::<DllDir>() + (n + 1) * 2;
    let node = RtlAllocateHeap(RtlGetProcessHeap(), 0, size as SIZE_T) as *mut DllDir;
    if node.is_null() {
        RtlFreeUnicodeString(&mut nt_name);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return null_mut();
    }
    (*node).len = n;
    copy_nonoverlapping(src, DllDir::path(node), n);
    *DllDir::path(node).add(n) = 0;
    RtlFreeUnicodeString(&mut nt_name);

    RtlEnterCriticalSection(&raw mut BaseDllDirectoryLock);
    (*node).next = DLL_DIRS;
    DLL_DIRS = node;
    RtlLeaveCriticalSection(&raw mut BaseDllDirectoryLock);
    node as *mut c_void
}

/// Unlike Wine, which frees whatever pointer it is given, a cookie that is not on the
/// list fails with ERROR_INVALID_PARAMETER instead of corrupting the heap.
#[no_mangle]
pub unsafe extern "system" fn RemoveDllDirectory(Cookie: *mut c_void) -> BOOL {
    let target = Cookie as *mut DllDir;
    RtlEnterCriticalSection(&raw mut BaseDllDirectoryLock);
    let mut link: *mut *mut DllDir = &raw mut DLL_DIRS;
    while !(*link).is_null() && *link != target {
        link = &raw mut (**link).next;
    }
    let found = !(*link).is_null();
    if found {
        *link = (*target).next;
    }
    RtlLeaveCriticalSection(&raw mut BaseDllDirectoryLock);
    if !found {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, target as PVOID);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SetDefaultDllDirectories(DirectoryFlags: DWORD) -> BOOL {
    if DirectoryFlags == 0 || DirectoryFlags & !DEFAULT_FLAGS_ALLOWED != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    DEFAULT_FLAGS = DirectoryFlags;
    TRUE
}
