//! startup/crtdll.c — the DLL personality: `_CRT_INIT`, `DllMainCRTStartup`,
//! `__DllMainCRTStartup`, and the dll `pre_c_init` (`.CRT$XIAA`).
//!
//! `DllMain` is called via an extern symbol so a consumer-defined `DllMain` wins; the default
//! is in `crt-dllmain-rust`. The default `DllEntryPoint` is here (see below for why).

use crate::atexit::{__onexitbegin, __onexitend};
use crate::cinit::{__xc_a, __xc_z, __xi_a, __xi_z};
use crate::globals::mingw_app_type;
use crate::misc::_decode_pointer;
use crate::state::*;
use crate::winapi::*;
use core::ffi::c_void;

extern "C" {
    fn _amsg_exit(ret: i32) -> !;
    fn _initterm(first: *const Pvfv, last: *const Pvfv);
    fn free(ptr: *mut c_void);
}

// Resolved at link to the consumer's DllMain/DllEntryPoint, else the
// crt-dllmain-rust defaults. "Else" is an lld `-alternatename` (used only when
// the name is still undefined at the end of the link), not link order: the
// defaults used to be plain `DllMain`/`DllEntryPoint` definitions, and a module
// defining only DllMain pulled the crt-dllmain object in for DllEntryPoint and
// got two DllMains, kept by order (mlang). This object is always linked (it is
// the DLL entry point), so its directive always applies.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".section .drectve,\"yni\"",
    ".ascii \" -alternatename:_DllMain@12=_vibeos_default_DllMain@12\"",
    ".ascii \" -alternatename:_DllEntryPoint@12=_vibeos_default_DllEntryPoint@12 \"",
    ".text",
);
#[cfg(not(target_arch = "x86"))]
core::arch::global_asm!(
    ".section .drectve,\"yni\"",
    ".ascii \" -alternatename:DllMain=vibeos_default_DllMain\"",
    ".ascii \" -alternatename:DllEntryPoint=vibeos_default_DllEntryPoint \"",
    ".text",
);

extern "system" {
    fn DllMain(h: HANDLE, reason: DWORD, lpreserved: LPVOID) -> WINBOOL;
    fn DllEntryPoint(h: HANDLE, reason: DWORD, lpreserved: LPVOID) -> WINBOOL;
}

/// startup/dllentry.c's default `DllEntryPoint` (returns TRUE), reached through the
/// `-alternatename` above. It lives HERE, beside its only caller, and not in
/// crt-dllmain-rust with the default `DllMain`: in the C archive the two were separate
/// members (dllentry.o, dllmain.o), but a Rust staticlib puts them in one object, and since
/// almost no module defines `DllEntryPoint`, every DLL's link pulled that object in for it
/// and got the default `DllMain` along with it. A module whose real `DllMain` sits in an
/// archive rather than an object (shell32: libshell32_rust.a) lost to the default -- shell32
/// never initialised and explorer panicked in CDefView on amd64 (2026-10-04). A module that
/// defines its own `DllEntryPoint` (dciman32, msvfw32) gets it with no duplicate, because
/// this default is only an alternate name.
#[no_mangle]
pub unsafe extern "system" fn vibeos_default_DllEntryPoint(
    _h: HANDLE,
    _reason: DWORD,
    _lpreserved: LPVOID,
) -> WINBOOL {
    1
}

#[no_mangle]
pub static mut __proc_attached: i32 = 0;

// .CRT$XIAA — dll pre_c_init: the onexit table is our own (NULL, not the
// exe's -1 sentinel).
unsafe extern "C" fn pre_c_init() -> i32 {
    __onexitend = core::ptr::null_mut();
    __onexitbegin = core::ptr::null_mut();
    0
}
#[no_mangle]
#[link_section = ".CRT$XIAA"]
#[used]
pub static pcinit: Pifv = Some(pre_c_init);

#[no_mangle]
pub unsafe extern "system" fn _CRT_INIT(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    lpreserved: LPVOID,
) -> WINBOOL {
    if dwReason == DLL_PROCESS_DETACH {
        if __proc_attached > 0 {
            __proc_attached -= 1;
        } else {
            return FALSE;
        }
    }

    if dwReason == DLL_PROCESS_ATTACH {
        let fiberid = current_fiber_id();
        let mut nested = false;
        loop {
            let lock_free = lock_cmpxchg(fiberid, core::ptr::null_mut());
            if lock_free.is_null() {
                break;
            }
            if lock_free == fiberid {
                nested = true;
                break;
            }
            Sleep(1000);
        }

        if __native_startup_state == NATIVE_INITIALIZING {
            _amsg_exit(31);
        } else if __native_startup_state == NATIVE_UNINITIALIZED {
            __native_startup_state = NATIVE_INITIALIZING;
            _initterm(__xi_a.as_ptr(), __xi_z.as_ptr());
        }
        if __native_startup_state == NATIVE_INITIALIZING {
            _initterm(__xc_a.as_ptr(), __xc_z.as_ptr());
            __native_startup_state = NATIVE_INITIALIZED;
        }
        if !nested {
            lock_xchg(core::ptr::null_mut());
        }
        if let Some(cb) = crate::tls::__dyn_tls_init_callback {
            cb(hDllHandle, DLL_THREAD_ATTACH, lpreserved);
        }
        __proc_attached += 1;
    } else if dwReason == DLL_PROCESS_DETACH {
        loop {
            let lock_free = lock_cmpxchg(1usize as PVOID, core::ptr::null_mut());
            if lock_free.is_null() {
                break;
            }
            Sleep(1000);
        }
        if __native_startup_state != NATIVE_INITIALIZED {
            _amsg_exit(31);
        } else {
            if !__onexitbegin.is_null() {
                let onexitbegin = _decode_pointer(__onexitbegin as *mut c_void) as *mut Pvfv;
                let mut onexitend = _decode_pointer(__onexitend as *mut c_void) as *mut Pvfv;
                loop {
                    onexitend = onexitend.sub(1);
                    if onexitend < onexitbegin {
                        break;
                    }
                    if let Some(f) = *onexitend {
                        f();
                    }
                }
                if lpreserved.is_null() {
                    free(onexitbegin as *mut c_void);
                }
                __onexitbegin = core::ptr::null_mut();
                __onexitend = core::ptr::null_mut();
            }
            __native_startup_state = NATIVE_UNINITIALIZED;
            lock_xchg(core::ptr::null_mut());
        }
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn DllMainCRTStartup(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    lpreserved: LPVOID,
) -> WINBOOL {
    mingw_app_type = 0;
    if dwReason == DLL_PROCESS_ATTACH {
        crate::gs::__security_init_cookie();
        #[cfg(not(target_arch = "x86"))]
        crate::excpt::__mingw_init_ehandler();
    }
    __DllMainCRTStartup(hDllHandle, dwReason, lpreserved)
}

// cdecl (C: `static WINBOOL __DllMainCRTStartup`, NOT WINAPI) — internal,
// called only by DllMainCRTStartup.
#[no_mangle]
pub unsafe extern "C" fn __DllMainCRTStartup(
    hDllHandle: HANDLE,
    dwReason: DWORD,
    lpreserved: LPVOID,
) -> WINBOOL {
    let mut retcode = TRUE;

    __native_dllmain_reason = dwReason;
    if dwReason == DLL_PROCESS_DETACH && __proc_attached == 0 {
        retcode = FALSE;
    } else {
        crate::pseudo_reloc::_pei386_runtime_relocator();

        let mut bailed = false;
        if dwReason == DLL_PROCESS_ATTACH || dwReason == DLL_THREAD_ATTACH {
            retcode = _CRT_INIT(hDllHandle, dwReason, lpreserved);
            if retcode == 0 {
                bailed = true;
            } else {
                retcode = DllEntryPoint(hDllHandle, dwReason, lpreserved);
                if retcode == 0 {
                    if dwReason == DLL_PROCESS_ATTACH {
                        _CRT_INIT(hDllHandle, DLL_PROCESS_DETACH, lpreserved);
                    }
                    bailed = true;
                }
            }
        }

        if !bailed {
            if dwReason == DLL_PROCESS_ATTACH {
                crate::gccmain::__main();
            }
            retcode = DllMain(hDllHandle, dwReason, lpreserved);
            if dwReason == DLL_PROCESS_ATTACH && retcode == 0 {
                DllMain(hDllHandle, DLL_PROCESS_DETACH, lpreserved);
                DllEntryPoint(hDllHandle, DLL_PROCESS_DETACH, lpreserved);
                _CRT_INIT(hDllHandle, DLL_PROCESS_DETACH, lpreserved);
            }
            if dwReason == DLL_PROCESS_DETACH || dwReason == DLL_THREAD_DETACH {
                retcode = DllEntryPoint(hDllHandle, dwReason, lpreserved);
                if _CRT_INIT(hDllHandle, dwReason, lpreserved) == FALSE {
                    retcode = FALSE;
                }
            }
        }
    }

    __native_dllmain_reason = u32::MAX;
    retcode
}
