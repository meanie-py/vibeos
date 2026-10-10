use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_Names: *mut *const ::core::ffi::c_char =
    ::core::ptr::null::<*const ::core::ffi::c_char>() as *mut *const ::core::ffi::c_char;

static mut g_NameCount: ::core::ffi::c_int = 0;

// WIN32_NO_STATUS: feature-guard define with no value, nothing to emit.

// SHIM_NS IgnoreFreeLibrary: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(IgnoreFreeLibrary);

pub type FREELIBRARYPROC = unsafe extern "system" fn(HMODULE) -> BOOL;

const MAX_PATH: usize = 260;

extern "system" {
    fn GetModuleFileNameA(hModule: HMODULE, lpFilename: *mut core::ffi::c_char, nSize: DWORD) -> DWORD;
}
extern "C" {
    fn _stricmp(a: *const core::ffi::c_char, b: *const core::ffi::c_char) -> core::ffi::c_int;
}

pub unsafe extern "system" fn FreeLibrary(hModule: HMODULE) -> BOOL {
    let mut Buffer: [core::ffi::c_char; MAX_PATH] = [0; MAX_PATH];
    let mut Ptr: *mut core::ffi::c_char = Buffer.as_mut_ptr();
    let mut len: DWORD;
    let mut wanted: DWORD = ARRAYSIZE!(Buffer) as DWORD;
    loop {
        len = GetModuleFileNameA(hModule, Ptr, wanted);
        if len < wanted {
            break;
        }

        wanted *= 2;
        if Ptr != Buffer.as_mut_ptr() {
            ShimLib_ShimFree(Ptr as PVOID);
        }

        Ptr = ShimLib_ShimMalloc(wanted as SIZE_T) as *mut core::ffi::c_char;
        if Ptr.is_null() {
            break;
        }
    }

    if !Ptr.is_null() && len != 0 {
        let mut ModuleName: *mut core::ffi::c_char = core::ptr::null_mut();
        let mut n: core::ffi::c_int;
        while len != 0 {
            ModuleName = Ptr.offset(len as isize);
            if *ModuleName.offset(-1) == b'/' as core::ffi::c_char
                || *ModuleName.offset(-1) == b'\\' as core::ffi::c_char
            {
                break;
            }
            len -= 1;
        }
        n = 0;
        while n < g_NameCount {
            if _stricmp(*g_Names.offset(n as isize), ModuleName) == 0 {
                SHIM_INFO!("Prevented unload of %s\n", ModuleName);
                if !Ptr.is_null() && Ptr != Buffer.as_mut_ptr() {
                    ShimLib_ShimFree(Ptr as PVOID);
                }
                return TRUE as BOOL;
            }
            n += 1;
        }
    }

    if !Ptr.is_null() && Ptr != Buffer.as_mut_ptr() {
        ShimLib_ShimFree(Ptr as PVOID);
    }

    return CALL_SHIM!(0, FREELIBRARYPROC)(hModule);
}

unsafe extern "C" fn InitIgnoreFreeLibrary(CommandLine: PCSTR) {
    let mut prev: PCSTR = core::ptr::null();
    let mut cur: PCSTR = core::ptr::null();
    let mut count: core::ffi::c_int = 1;
    let mut n: core::ffi::c_int = 0;
    let mut names: *mut *const core::ffi::c_char = core::ptr::null_mut();

    if CommandLine.is_null() || *CommandLine == 0 {
        return;
    }

    prev = CommandLine;
    loop {
        cur = strchr(prev as *const core::ffi::c_char, ';' as core::ffi::c_int) as PCSTR;
        if cur.is_null() {
            break;
        }
        count += 1;
        prev = cur.offset(1 as isize);
    }

    names = ShimLib_ShimMalloc(
        (core::mem::size_of::<*mut core::ffi::c_char>() as SIZE_T).wrapping_mul(count as SIZE_T),
    ) as *mut *const core::ffi::c_char;
    if names.is_null() {
        SHIM_WARN!(
            "Unable to allocate %u bytes\n",
            (core::mem::size_of::<*mut core::ffi::c_char>() as SIZE_T).wrapping_mul(count as SIZE_T)
        );
        return;
    }

    'fail: {
        prev = CommandLine;
        loop {
            cur = strchr(prev as *const core::ffi::c_char, ';' as core::ffi::c_int) as PCSTR;
            if cur.is_null() {
                break;
            }
            *names.offset(n as isize) = ShimLib_StringNDuplicateA(
                prev,
                (cur.offset_from(prev) as SIZE_T).wrapping_add(1 as SIZE_T),
            ) as *const core::ffi::c_char;
            if (*names.offset(n as isize)).is_null() {
                SHIM_WARN!(
                    "Unable to allocate %u bytes\n",
                    (cur.offset_from(prev) as SIZE_T).wrapping_add(2 as SIZE_T)
                );
                break 'fail;
            }
            n += 1;
            prev = cur.offset(1 as isize);
        }
        *names.offset(n as isize) = ShimLib_StringDuplicateA(prev) as *const core::ffi::c_char;
        if (*names.offset(n as isize)).is_null() {
            SHIM_WARN!("Unable to allocate last string\n");
            break 'fail;
        }

        g_Names = names;
        g_NameCount = count;
        return;
    }

    // fail:
    n -= 1;
    while n >= 0 as core::ffi::c_int {
        if !(*names.offset(n as isize)).is_null() {
            ShimLib_ShimFree(*names.offset(n as isize) as PVOID);
        }
        n -= 1;
    }
    ShimLib_ShimFree(names as PVOID);
}

pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    if fdwReason == SHIM_NOTIFY_ATTACH {
        SHIM_MSG!("IgnoreFreeLibrary(%s)\n", g_szCommandLine);
        InitIgnoreFreeLibrary(g_szCommandLine);
    }
    return TRUE as BOOL;
}

// SHIM_NUM_HOOKS 1, SHIM_NOTIFY_FN = SHIM_OBJ_NAME(Notify)
implement_shim!(IgnoreFreeLibrary, 1usize, Some(Notify), [
    (0, "KERNEL32.DLL", "FreeLibrary", FreeLibrary),
]);
