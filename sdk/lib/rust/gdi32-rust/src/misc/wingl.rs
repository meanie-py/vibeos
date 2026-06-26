//! misc/wingl.c → Rust. OpenGL (wgl*) client entry points.
//!
//! Thin OpenGL forwarders: lazily LoadLibraryW("OPENGL32.DLL") and resolve the
//! wgl* pixel-format / swap-buffers entry points via GetProcAddress, then forward
//! ChoosePixelFormat / DescribePixelFormat / GetPixelFormat / SetPixelFormat /
//! SwapBuffers into them.
//!
//! NEEDS-IMPORT: kernel32 exports GetProcAddress / LoadLibraryW / FreeLibrary are
//!   declared `extern "system"` (WINAPI = stdcall on i386) below; they resolve
//!   against the kernel32 importlib at link.
//! NEEDS-CHECK: `hOpenGL` is a NON-static (exported) HINSTANCE global in the C
//!   (the OpenGL module handle); kept as `#[no_mangle] pub static mut`.
//! NEEDS-CHECK: the lazy function-pointer globals (glChoosePixelFormat, …) are
//!   file-private `static`s in C → kept private (no `#[no_mangle]`).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// BOOL-typed TRUE/FALSE shadows (win32k_sys FALSE/TRUE are u32; BOOL is i32).
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

// ── OpenGL forwarder function-pointer typedefs (all WINAPI = stdcall) ──────────
type CHOOSEPIXELFMT = Option<unsafe extern "system" fn(HDC, *const PIXELFORMATDESCRIPTOR) -> INT>;
type SETPIXELFMT = Option<unsafe extern "system" fn(HDC, INT, *const PIXELFORMATDESCRIPTOR) -> BOOL>;
type SWAPBUFFERS = Option<unsafe extern "system" fn(HDC) -> BOOL>;
type DESCRIBEPIXELFMT = Option<unsafe extern "system" fn(HDC, INT, UINT, LPPIXELFORMATDESCRIPTOR) -> INT>;
type GETPIXELFMT = Option<unsafe extern "system" fn(HDC) -> INT>;

// ── Lazily-resolved OpenGL entry points (file-private `static` in C) ───────────
static mut glChoosePixelFormat: CHOOSEPIXELFMT = None;
static mut glSetPixelFormat: SETPIXELFMT = None;
static mut glSwapBuffers: SWAPBUFFERS = None;
static mut glDescribePixelFormat: DESCRIBEPIXELFMT = None;
static mut glGetPixelFormat: GETPIXELFMT = None;

/* OpenGL Handle. */
#[no_mangle]
pub static mut hOpenGL: HINSTANCE = core::ptr::null_mut();

// L"OPENGL32.DLL"
const OPENGL32_DLL: [WCHAR; 13] = [
    b'O' as WCHAR, b'P' as WCHAR, b'E' as WCHAR, b'N' as WCHAR, b'G' as WCHAR,
    b'L' as WCHAR, b'3' as WCHAR, b'2' as WCHAR, b'.' as WCHAR, b'D' as WCHAR,
    b'L' as WCHAR, b'L' as WCHAR, 0,
];

// ── kernel32 exports (WINAPI = stdcall on i386) ───────────────────────────────
extern "system" {
    fn GetProcAddress(hModule: HMODULE, lpProcName: PCSTR) -> FARPROC;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
}

// ── Interlocked* pointer ops (i386 = kernel32 import, amd64/arm64 = intrinsic
//    with no link symbol → portable core-atomic impl, per the cohab contract) ──
#[inline]
unsafe fn InterlockedCompareExchangePointer(dest: *mut PVOID, exchange: PVOID, comparand: PVOID) -> PVOID {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}

unsafe fn OpenGLInitFunction(name: PCSTR, funcptr: *mut FARPROC) -> BOOL {
    let func: PVOID = core::mem::transmute::<FARPROC, PVOID>(GetProcAddress(hOpenGL, name));
    if !func.is_null() {
        InterlockedCompareExchangePointer(funcptr as *mut PVOID, func, core::ptr::null_mut());
        return TRUE;
    }

    FALSE
}

unsafe fn OpenGLEnable() -> BOOL {
    let mut ret: BOOL = TRUE;

    let hModOpengl32: HMODULE = LoadLibraryW(OPENGL32_DLL.as_ptr());
    if hModOpengl32.is_null() {
        return FALSE;
    }

    if !InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!(hOpenGL) as *mut PVOID,
        hModOpengl32 as PVOID,
        core::ptr::null_mut(),
    )
    .is_null()
    {
        FreeLibrary(hModOpengl32);

        /* NOTE: Even though another thread was faster loading the
                 library we can't just bail out here. We really need
                 to *try* to locate every function. This is slow but
                 thread-safe */
    }

    /* The cast is required on x64, because FARPROC has INT_PTR sized return */
    if OpenGLInitFunction(
        b"wglChoosePixelFormat\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(glChoosePixelFormat) as *mut FARPROC,
    ) == FALSE
    {
        ret = FALSE;
    }

    if OpenGLInitFunction(
        b"wglSetPixelFormat\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(glSetPixelFormat) as *mut FARPROC,
    ) == FALSE
    {
        ret = FALSE;
    }

    if OpenGLInitFunction(
        b"wglSwapBuffers\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(glSwapBuffers) as *mut FARPROC,
    ) == FALSE
    {
        ret = FALSE;
    }

    if OpenGLInitFunction(
        b"wglDescribePixelFormat\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(glDescribePixelFormat) as *mut FARPROC,
    ) == FALSE
    {
        ret = FALSE;
    }

    if OpenGLInitFunction(
        b"wglGetPixelFormat\0".as_ptr() as PCSTR,
        core::ptr::addr_of_mut!(glGetPixelFormat) as *mut FARPROC,
    ) == FALSE
    {
        ret = FALSE;
    }

    ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ChoosePixelFormat(hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR) -> INT {
    if glChoosePixelFormat.is_none() && OpenGLEnable() == FALSE {
        return 0;
    }

    (glChoosePixelFormat.unwrap())(hdc, ppfd)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DescribePixelFormat(
    hdc: HDC,
    iPixelFormat: INT,
    nBytes: UINT,
    ppfd: LPPIXELFORMATDESCRIPTOR,
) -> INT {
    if glDescribePixelFormat.is_none() && OpenGLEnable() == FALSE {
        return 0;
    }

    (glDescribePixelFormat.unwrap())(hdc, iPixelFormat, nBytes, ppfd)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetPixelFormat(hdc: HDC) -> INT {
    if glGetPixelFormat.is_none() && OpenGLEnable() == FALSE {
        return 0;
    }

    (glGetPixelFormat.unwrap())(hdc)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetPixelFormat(
    hdc: HDC,
    iPixelFormat: INT,
    ppfd: *const PIXELFORMATDESCRIPTOR,
) -> BOOL {
    /* Can only be set once */
    let current: INT = GetPixelFormat(hdc);
    if current != 0 {
        return (current == iPixelFormat) as BOOL;
    }

    if glSetPixelFormat.is_none() && OpenGLEnable() == FALSE {
        return 0;
    }

    (glSetPixelFormat.unwrap())(hdc, iPixelFormat, ppfd)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SwapBuffers(hdc: HDC) -> BOOL {
    if glSwapBuffers.is_none() && OpenGLEnable() == FALSE {
        return 0;
    }

    (glSwapBuffers.unwrap())(hdc)
}

/*
    Do this here for now.
*/

/* EOF */
