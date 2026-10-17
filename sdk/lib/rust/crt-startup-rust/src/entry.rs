//! startup/crtexe.c (+ wcrtexe.c via WPRFLAG, crt0_c.c/crt0_w.c) — the exe
//! entry points and `__tmainCRTStartup`.
//!
//! `WIDE` selects the ANSI (`crtexe`) vs wide (`wcrtexe`) personality — the
//! two are mutually-exclusive features, so exactly one set of entry symbols
//! + one `__getmainargs`/`__wgetmainargs` init runs, exactly as the C
//! archive's lazy crtexe.o/wcrtexe.o pull guaranteed.
//!
//! Simplifications vs the C (off the boot path, behaviour-preserving):
//!   - the i386 `_alloca(32)` + `andl $-16,%esp` frame realign is dropped
//!     (Rust keeps the stack aligned; i386 here is no-SSE anyway).
//!   - the amd64/arm64 inline `.seh_handler` frame scope is dropped; the
//!     process-wide `SetUnhandledExceptionFilter(_gnu_exception_handler)`
//!     (installed below) is the same handler and the live catch path.

use crate::cinit::{__xc_a, __xc_z, __xi_a, __xi_z};
use crate::globals::*;
use crate::state::*;
use crate::winapi::*;
use core::ffi::c_void;

const WIDE: bool = cfg!(feature = "wcrtexe");
const SPACECHAR: u32 = b' ' as u32;
const DQUOTECHAR: u32 = b'"' as u32;

extern "C" {
    #[cfg_attr(target_arch = "x86", link_name = "_image_base__")]
    #[cfg_attr(not(target_arch = "x86"), link_name = "__image_base__")]
    static __ImageBase: IMAGE_DOS_HEADER;
}

#[no_mangle]
pub static mut __mingw_winmain_hInstance: HINSTANCE = core::ptr::null_mut();
#[no_mangle]
pub static mut __mingw_winmain_lpCmdLine: *mut u8 = core::ptr::null_mut(); // _TCHAR*
#[no_mangle]
pub static mut __mingw_winmain_nShowCmd: DWORD = 0;

// crtexe.c file-scope statics.
static mut ARGC: i32 = 0;
static mut ARGV: *mut *mut c_void = core::ptr::null_mut();
static mut ENVP: *mut *mut c_void = core::ptr::null_mut();
static mut MAINRET: i32 = 0;
static mut MANAGEDAPP: i32 = 0;
static mut HAS_CCTOR: i32 = 0;
static mut STARTINFO: StartupInfo = StartupInfo { newmode: 0 };

extern "C" {
    fn _fpreset();
    fn _cexit();
}

// ---- .CRT$X* init entries -------------------------------------------
unsafe extern "C" fn pre_c_init() -> i32 {
    MANAGEDAPP = check_managed_app();
    if mingw_app_type != 0 {
        __set_app_type(_GUI_APP);
    } else {
        __set_app_type(_CONSOLE_APP);
    }
    __onexit_init_sentinel();

    if WIDE {
        crate::argv::_wsetargv();
    } else {
        crate::argv::_setargv();
    }
    if _MINGW_INSTALL_DEBUG_MATHERR == 1 {
        __setusermatherr(crate::misc::_matherr as *mut c_void);
    }
    0
}

#[inline]
unsafe fn __onexit_init_sentinel() {
    // exe: onexit table is the CRT's — sentinel -1.
    let m1 = usize::MAX as *mut Pvfv;
    crate::atexit::__onexitbegin = m1;
    crate::atexit::__onexitend = m1;
}

unsafe extern "C" fn pre_cpp_init() {
    STARTINFO.newmode = _newmode;
    if WIDE {
        __wgetmainargs(
            &raw mut ARGC,
            &raw mut ARGV as *mut _ as *mut *mut *mut u16,
            &raw mut ENVP as *mut _ as *mut *mut *mut u16,
            _dowildcard,
            &raw mut STARTINFO,
        );
    } else {
        __getmainargs(
            &raw mut ARGC,
            &raw mut ARGV as *mut _ as *mut *mut *mut u8,
            &raw mut ENVP as *mut _ as *mut *mut *mut u8,
            _dowildcard,
            &raw mut STARTINFO,
        );
    }
}

#[no_mangle]
#[link_section = ".CRT$XIAA"]
#[used]
pub static mingw_pcinit: Pifv = Some(pre_c_init);
#[no_mangle]
#[link_section = ".CRT$XCAA"]
#[used]
pub static mingw_pcppinit: Pvfv = Some(pre_cpp_init);

// ---- entry points ----------------------------------------------------
#[cfg(feature = "crtexe")]
#[no_mangle]
pub unsafe extern "C" fn mainCRTStartup() -> i32 {
    // _DLL is defined for msvcrtex → no crt_process_init.
    mingw_app_type = 0;
    crate::gs::__security_init_cookie();
    __tmainCRTStartup()
}

#[cfg(feature = "crtexe")]
#[no_mangle]
pub unsafe extern "C" fn WinMainCRTStartup() -> i32 {
    mingw_app_type = 1;
    crate::gs::__security_init_cookie();
    __tmainCRTStartup()
}

#[cfg(feature = "wcrtexe")]
#[no_mangle]
pub unsafe extern "C" fn wmainCRTStartup() -> i32 {
    mingw_app_type = 0;
    crate::gs::__security_init_cookie();
    __tmainCRTStartup()
}

#[cfg(feature = "wcrtexe")]
#[no_mangle]
pub unsafe extern "C" fn wWinMainCRTStartup() -> i32 {
    mingw_app_type = 1;
    crate::gs::__security_init_cookie();
    __tmainCRTStartup()
}

unsafe fn __tmainCRTStartup() -> i32 {
    let mut startup_info: STARTUPINFOA = core::mem::zeroed();

    if mingw_app_type != 0 {
        GetStartupInfoA(&mut startup_info);
    }

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
    } else {
        HAS_CCTOR = 1;
    }

    if __native_startup_state == NATIVE_INITIALIZING {
        _initterm(__xc_a.as_ptr(), __xc_z.as_ptr());
        __native_startup_state = NATIVE_INITIALIZED;
    }

    if !nested {
        lock_xchg(core::ptr::null_mut());
    }

    if let Some(cb) = crate::tls::__dyn_tls_init_callback {
        cb(core::ptr::null_mut(), DLL_THREAD_ATTACH, core::ptr::null_mut());
    }

    crate::pseudo_reloc::_pei386_runtime_relocator();
    crate::excpt::__mingw_oldexcpt_handler = SetUnhandledExceptionFilter(Some(
        crate::excpt::_gnu_exception_handler,
    ));
    #[cfg(not(target_arch = "x86"))]
    crate::excpt::__mingw_init_ehandler();
    __mingw_prepare_except_for_msvcr80_and_higher();

    _fpreset();

    if mingw_app_type != 0 {
        let cmdline = compute_winmain_cmdline();
        __mingw_winmain_hInstance = core::ptr::addr_of!(__ImageBase) as HINSTANCE;
        __mingw_winmain_lpCmdLine = cmdline;
        __mingw_winmain_nShowCmd = if startup_info.dwFlags & STARTF_USESHOWWINDOW != 0 {
            startup_info.wShowWindow as DWORD
        } else {
            SW_SHOWDEFAULT
        };
    }

    duplicate_ppstrings(ARGC, &raw mut ARGV);
    crate::gccmain::__main();

    // __initenv / __winitenv aren't present on arm/arm64.
    #[cfg(not(target_arch = "aarch64"))]
    set_initenv();

    if WIDE {
        MAINRET = wmain(ARGC, ARGV as *mut *mut u16, ENVP as *mut *mut u16);
    } else {
        MAINRET = main(ARGC, ARGV as *mut *mut u8, ENVP as *mut *mut u8);
    }

    crate::gccmain::__do_global_dtors();

    if MANAGEDAPP == 0 {
        exit(MAINRET);
    }
    if HAS_CCTOR == 0 {
        _cexit();
    }
    MAINRET
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn set_initenv() {
    // write the msvcrt __initenv/__winitenv globals through their __imp_ slot.
    if WIDE {
        *IMP_WINITENV = ENVP as *mut *mut u16;
    } else {
        *IMP_INITENV = ENVP as *mut *mut u8;
    }
}

/// The GUI command-line: skip the program name (respecting quotes), then
/// leading spaces. Returns a pointer into _acmdln/_wcmdln (stored raw).
unsafe fn compute_winmain_cmdline() -> *mut u8 {
    if WIDE {
        let mut p = *IMP_WCMDLN;
        let mut in_dq = false;
        loop {
            let c = *p as u32;
            if !(c > SPACECHAR || (c != 0 && in_dq)) {
                break;
            }
            if c == DQUOTECHAR {
                in_dq = !in_dq;
            }
            p = p.add(1);
        }
        loop {
            let c = *p as u32;
            if !(c != 0 && c <= SPACECHAR) {
                break;
            }
            p = p.add(1);
        }
        p as *mut u8
    } else {
        let mut p = *IMP_ACMDLN;
        let mut in_dq = false;
        loop {
            let c = *p as u32;
            if !(c > SPACECHAR || (c != 0 && in_dq)) {
                break;
            }
            if c == DQUOTECHAR {
                in_dq = !in_dq;
            }
            p = p.add(1);
        }
        loop {
            let c = *p as u32;
            if !(c != 0 && c <= SPACECHAR) {
                break;
            }
            p = p.add(1);
        }
        p
    }
}

unsafe fn duplicate_ppstrings(ac: i32, av: *mut *mut *mut c_void) {
    if WIDE {
        let avl = *av as *mut *mut u16;
        let n = malloc(core::mem::size_of::<*mut u16>() * (ac as usize + 1)) as *mut *mut u16;
        let mut i = 0isize;
        while i < ac as isize {
            let src = *avl.offset(i);
            let l = wbytelen(src);
            let dst = malloc(l) as *mut u16;
            memcpy(dst as *mut c_void, src as *const c_void, l);
            *n.offset(i) = dst;
            i += 1;
        }
        *n.offset(ac as isize) = core::ptr::null_mut();
        *av = n as *mut *mut c_void;
    } else {
        let avl = *av as *mut *mut u8;
        let n = malloc(core::mem::size_of::<*mut u8>() * (ac as usize + 1)) as *mut *mut u8;
        let mut i = 0isize;
        while i < ac as isize {
            let src = *avl.offset(i);
            let l = strlen(src) + 1;
            let dst = malloc(l) as *mut u8;
            memcpy(dst as *mut c_void, src as *const c_void, l);
            *n.offset(i) = dst;
            i += 1;
        }
        *n.offset(ac as isize) = core::ptr::null_mut();
        *av = n as *mut *mut c_void;
    }
}

/// wide string length in BYTES, including the terminator.
unsafe fn wbytelen(p: *const u16) -> usize {
    let mut ret = 1usize;
    let mut q = p;
    while *q != 0 {
        ret += 1;
        q = q.add(1);
    }
    ret * 2
}

unsafe fn check_managed_app() -> i32 {
    // Force the lts/charmax helpers to be linked.
    mingw_initltsdrot_force = 1;
    mingw_initltsdyn_force = 1;
    mingw_initltssuo_force = 1;
    mingw_initcharmax = 1;

    let base = core::ptr::addr_of!(__ImageBase) as *const u8;
    let dos = base as *const IMAGE_DOS_HEADER;
    if (*dos).e_magic != IMAGE_DOS_SIGNATURE {
        return 0;
    }
    let nt = base.offset((*dos).e_lfanew as isize);
    let sig = *(nt as *const DWORD);
    if sig != IMAGE_NT_SIGNATURE {
        return 0;
    }
    let opt32 = nt.add(core::mem::size_of::<DWORD>() + core::mem::size_of::<IMAGE_FILE_HEADER>())
        as *const IMAGE_OPTIONAL_HEADER32;
    match (*opt32).Magic {
        IMAGE_NT_OPTIONAL_HDR32_MAGIC => {
            if (*opt32).NumberOfRvaAndSizes as usize <= IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR {
                return 0;
            }
            (((*opt32).DataDirectory[IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR].VirtualAddress) != 0)
                as i32
        }
        IMAGE_NT_OPTIONAL_HDR64_MAGIC => {
            let opt64 = opt32 as *const IMAGE_OPTIONAL_HEADER64;
            if (*opt64).NumberOfRvaAndSizes as usize <= IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR {
                return 0;
            }
            (((*opt64).DataDirectory[IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR].VirtualAddress) != 0)
                as i32
        }
        _ => 0,
    }
}

// __mingw_invalidParameterHandler — no-op in our config.
unsafe extern "C" fn __mingw_invalidParameterHandler(
    _expr: *const u16,
    _func: *const u16,
    _file: *const u16,
    _line: u32,
    _reserved: usize,
) {
}

unsafe fn __mingw_prepare_except_for_msvcr80_and_higher() {
    type SetIph = unsafe extern "C" fn(
        crate::invalid_param::InvalidParamHandler,
    ) -> crate::invalid_param::InvalidParamHandler;
    let p = GetProcAddress(
        crate::pesect::__mingw_get_msvcrt_handle(),
        b"_set_invalid_parameter_handler\0".as_ptr(),
    );
    if let Some(f) = p {
        let f: SetIph = core::mem::transmute(f);
        f(Some(__mingw_invalidParameterHandler));
    }
}

// NOTE: crt0_c.c / crt0_w.c (the default `main`/`wmain` → WinMain bridge for
// GUI images that supply WinMain/wWinMain) are intentionally NOT defined here.
// In the C archive they were separate objects, lazy-pulled only when `main`
// was otherwise undefined; folding them into this single staticlib would leave
// every console exe (which has its own `main`, no `WinMain`) with an undefined
// `WinMain` reference. `__tmainCRTStartup` calls `main`/`wmain` via the extern
// decls (winapi.rs) → the consumer's. A GUI consumer that needs the bridge
// provides it itself (or via a dedicated crt0 shim) — see the wiring notes.
