use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{PsGetCurrentProcessWin32Process, ExFreePoolWithTag};

// KeRosDumpStackFrames(Frames, Count) — win32k gdidebug.h macro; a no-op in the
// non-DBG build (where it would otherwise call the ntoskrnl debug export).
#[inline(always)]
unsafe fn KeRosDumpStackFrames(_Frames: *mut ULONG_PTR, _Count: ULONG) {}
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
#[allow(unused_imports)]
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// L"..." literal → NUL-terminated UTF-16 static, yields *const u16. (ASCII source.)
macro_rules! wide_str {
    ($s:literal) => {{
        const N: usize = $s.len() + 1;
        static W: [u16; N] = {
            let b = $s.as_bytes();
            let mut w = [0u16; N];
            let mut i = 0;
            while i < b.len() { w[i] = b[i] as u16; i += 1; }
            w
        };
        W.as_ptr()
    }};
}

// tags.h: USERTAG_WINDOWLIST 'lwsU' (C multichar big-endian → reversed bytes).
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");

// SPI broadcast target (winuser.h HWND_BROADCAST = (HWND)0xffff).
const HWND_BROADCAST: HWND = 0xffff as HWND;

// PtrToUlong (basetsd.h): truncate pointer to 32-bit.
#[inline(always)]
fn PtrToUlong<T>(p: *const T) -> u32 { p as usize as u32 }

// RtlZeroMemory / RtlEqualMemory (winnt.h intrinsics).
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut core::ffi::c_void, n: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, n);
}
#[inline(always)]
unsafe fn RtlEqualMemory(s1: *const core::ffi::c_void, s2: *const core::ffi::c_void, n: usize) -> u8 {
    let a = core::slice::from_raw_parts(s1 as *const u8, n);
    let b = core::slice::from_raw_parts(s2 as *const u8, n);
    (a == b) as u8
}

// SURFACE_Share{Lock,Unlock}Surface (gdi inline helpers, mirror gdiobj.rs).
#[inline(always)]
unsafe fn SURFACE_ShareLockSurface(h: HBITMAP) -> PSURFACE {
    GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE
}
#[inline(always)]
unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) {
    GDIOBJ_vDereferenceObject(p as POBJ);
}

// gpsi->{ptCursor,dtCaretBlink,PUSIFlags,...} live in the anonymous PERUSERSERVERINFO
// tail of tagSERVERINFO, which bindgen made opaque (`__bindgen_padding_0`). Mirror
// caret.rs/display.rs: reinterpret that tail as _PERUSERSERVERINFO.
#[inline(always)]
unsafe fn pusi(p: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*p).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// RECTL_bIsEmptyRect (gdi inline, mirrors region.rs).
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    (((*prcl).left >= (*prcl).right) || ((*prcl).top >= (*prcl).bottom)) as BOOL
}

// cdecl (`_Name`) win32k/user internal exports referenced by sysparams.
extern "C" {
    fn wcslen(str: *const u16) -> usize;
    fn _wtoi(s: *const u16) -> i32;
    fn _itow(value: i32, str: *mut u16, radix: i32) -> *mut u16;
    fn GetW32ProcessInfo() -> PPROCESSINFO;
}

// NTAPI (`_Name@N` stdcall on i386) win32k/ntoskrnl exports.
extern "system" {
    fn RegReadUserSetting(pwszKeyName: PCWSTR, pwszValueName: PCWSTR, ulType: ULONG, pvData: *mut core::ffi::c_void, cbDataSize: ULONG) -> NTSTATUS;
    fn InitMetrics() -> BOOL;
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: usize, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForRead(Address: *const core::ffi::c_void, Length: usize, Alignment: u32) -> NTSTATUS;
    fn PsGetCurrentThreadWin32Thread() -> PTHREADINFO;
}

// fastcall (i386) / cdecl (other) NTAPI helpers.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}

// UserRefObjectCo / UserDerefObjectCo (object.h static __inline, no symbol).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    (*UserReferenceEntry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// ---- file-scope preamble (translated) ----
// File-owned globals: sysparams.c DEFINED these (the win32k-sys bindgen only
// declares them `extern`). Now that sysparams.c is replaced by sysparams.rs,
// this module provides the real #[no_mangle] symbols (gspv = SPIVALUES{0},
// g_PaintDesktopVersion = FALSE) that metric/painting/etc. import. The local
// defs shadow the glob-imported extern declarations of the same names.
#[no_mangle]
pub static mut gspv: SPIVALUES = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut g_PaintDesktopVersion: BOOL = FALSE as BOOL;
#[no_mangle]
pub static mut g_bWindowSnapEnabled: BOOL = TRUE as BOOL;

#[no_mangle]
pub static mut gbSpiInitialized: BOOL = FALSE as BOOL;

pub const dpi: i32 = 96;

pub static KEY_MOUSE: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x4d,0x6f,0x75,0x73,0x65,0x00];
pub static VAL_MOUSE1: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x54,0x68,0x72,0x65,0x73,0x68,0x6f,0x6c,0x64,0x31,0x00];
pub static VAL_MOUSE2: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x54,0x68,0x72,0x65,0x73,0x68,0x6f,0x6c,0x64,0x32,0x00];
pub static VAL_MOUSE3: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x53,0x70,0x65,0x65,0x64,0x00];
pub static VAL_MOUSETRAILS: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x54,0x72,0x61,0x69,0x6c,0x73,0x00];
pub static VAL_DBLCLKWIDTH: &[u16] = &[0x44,0x6f,0x75,0x62,0x6c,0x65,0x43,0x6c,0x69,0x63,0x6b,0x57,0x69,0x64,0x74,0x68,0x00];
pub static VAL_DBLCLKHEIGHT: &[u16] = &[0x44,0x6f,0x75,0x62,0x6c,0x65,0x43,0x6c,0x69,0x63,0x6b,0x48,0x65,0x69,0x67,0x68,0x74,0x00];
pub static VAL_DBLCLKTIME: &[u16] = &[0x44,0x6f,0x75,0x62,0x6c,0x65,0x43,0x6c,0x69,0x63,0x6b,0x53,0x70,0x65,0x65,0x64,0x00];
pub static VAL_SNAPDEFBTN: &[u16] = &[0x53,0x6e,0x61,0x70,0x54,0x6f,0x44,0x65,0x66,0x61,0x75,0x6c,0x74,0x42,0x75,0x74,0x74,0x6f,0x6e,0x00];
pub static VAL_SWAP: &[u16] = &[0x53,0x77,0x61,0x70,0x4d,0x6f,0x75,0x73,0x65,0x42,0x75,0x74,0x74,0x6f,0x6e,0x73,0x00];
pub static VAL_HOVERTIME: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x48,0x6f,0x76,0x65,0x72,0x54,0x69,0x6d,0x65,0x00];
pub static VAL_HOVERWIDTH: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x48,0x6f,0x76,0x65,0x72,0x57,0x69,0x64,0x74,0x68,0x00];
pub static VAL_HOVERHEIGHT: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x48,0x6f,0x76,0x65,0x72,0x48,0x65,0x69,0x67,0x68,0x74,0x00];
pub static VAL_SENSITIVITY: &[u16] = &[0x4d,0x6f,0x75,0x73,0x65,0x53,0x65,0x6e,0x73,0x69,0x74,0x69,0x76,0x69,0x74,0x79,0x00];
pub static KEY_DESKTOP: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x44,0x65,0x73,0x6b,0x74,0x6f,0x70,0x00];
pub static VAL_SCRTO: &[u16] = &[0x53,0x63,0x72,0x65,0x65,0x6e,0x53,0x61,0x76,0x65,0x54,0x69,0x6d,0x65,0x4f,0x75,0x74,0x00];
pub static VAL_SCRNSV: &[u16] = &[0x53,0x43,0x52,0x4e,0x53,0x41,0x56,0x45,0x2e,0x45,0x58,0x45,0x00];
pub static VAL_SCRACT: &[u16] = &[0x53,0x63,0x72,0x65,0x65,0x6e,0x53,0x61,0x76,0x65,0x41,0x63,0x74,0x69,0x76,0x65,0x00];
pub static VAL_GRID: &[u16] = &[0x47,0x72,0x69,0x64,0x47,0x72,0x61,0x6e,0x75,0x6c,0x61,0x72,0x69,0x74,0x79,0x00];
pub static VAL_DRAG: &[u16] = &[0x44,0x72,0x61,0x67,0x46,0x75,0x6c,0x6c,0x57,0x69,0x6e,0x64,0x6f,0x77,0x73,0x00];
pub static VAL_DRAGHEIGHT: &[u16] = &[0x44,0x72,0x61,0x67,0x48,0x65,0x69,0x67,0x68,0x74,0x00];
pub static VAL_DRAGWIDTH: &[u16] = &[0x44,0x72,0x61,0x67,0x57,0x69,0x64,0x74,0x68,0x00];
pub static VAL_FONTSMOOTHING: &[u16] = &[0x46,0x6f,0x6e,0x74,0x53,0x6d,0x6f,0x6f,0x74,0x68,0x69,0x6e,0x67,0x00];
pub static VAL_FONTSMOOTHINGTYPE: &[u16] = &[0x46,0x6f,0x6e,0x74,0x53,0x6d,0x6f,0x6f,0x74,0x68,0x69,0x6e,0x67,0x54,0x79,0x70,0x65,0x00];
pub static VAL_FONTSMOOTHINGCONTRAST: &[u16] = &[0x46,0x6f,0x6e,0x74,0x53,0x6d,0x6f,0x6f,0x74,0x68,0x69,0x6e,0x67,0x47,0x61,0x6d,0x6d,0x61,0x00];
pub static VAL_FONTSMOOTHINGORIENTATION: &[u16] = &[0x46,0x6f,0x6e,0x74,0x53,0x6d,0x6f,0x6f,0x74,0x68,0x69,0x6e,0x67,0x4f,0x72,0x69,0x65,0x6e,0x74,0x61,0x74,0x69,0x6f,0x6e,0x00];
pub static VAL_SCRLLLINES: &[u16] = &[0x57,0x68,0x65,0x65,0x6c,0x53,0x63,0x72,0x6f,0x6c,0x6c,0x4c,0x69,0x6e,0x65,0x73,0x00];
pub static VAL_CLICKLOCKTIME: &[u16] = &[0x43,0x6c,0x69,0x63,0x6b,0x4c,0x6f,0x63,0x6b,0x54,0x69,0x6d,0x65,0x00];
pub static VAL_PAINTDESKVER: &[u16] = &[0x50,0x61,0x69,0x6e,0x74,0x44,0x65,0x73,0x6b,0x74,0x6f,0x70,0x56,0x65,0x72,0x73,0x69,0x6f,0x6e,0x00];
pub static VAL_CARETRATE: &[u16] = &[0x43,0x75,0x72,0x73,0x6f,0x72,0x42,0x6c,0x69,0x6e,0x6b,0x52,0x61,0x74,0x65,0x00];
pub static VAL_CARETWIDTH: &[u16] = &[0x43,0x61,0x72,0x65,0x74,0x57,0x69,0x64,0x74,0x68,0x00];
pub static VAL_SCRLLCHARS: &[u16] = &[0x57,0x68,0x65,0x65,0x6c,0x53,0x63,0x72,0x6f,0x6c,0x6c,0x43,0x68,0x61,0x72,0x73,0x00];
pub static VAL_USERPREFMASK: &[u16] = &[0x55,0x73,0x65,0x72,0x50,0x72,0x65,0x66,0x65,0x72,0x65,0x6e,0x63,0x65,0x73,0x4d,0x61,0x73,0x6b,0x00];
pub static VAL_SNAP_ENABLED: &[u16] = &[0x57,0x69,0x6e,0x64,0x6f,0x77,0x41,0x72,0x72,0x61,0x6e,0x67,0x65,0x6d,0x65,0x6e,0x74,0x41,0x63,0x74,0x69,0x76,0x65,0x00];
pub static VAL_SNAP_DOCKMOVING: &[u16] = &[0x44,0x6f,0x63,0x6b,0x4d,0x6f,0x76,0x69,0x6e,0x67,0x00];
pub static KEY_MDALIGN: &[u16] = &[0x53,0x6f,0x66,0x74,0x77,0x61,0x72,0x65,0x5c,0x4d,0x69,0x63,0x72,0x6f,0x73,0x6f,0x66,0x74,0x5c,0x57,0x69,0x6e,0x64,0x6f,0x77,0x73,0x20,0x4e,0x54,0x5c,0x43,0x75,0x72,0x72,0x65,0x6e,0x74,0x56,0x65,0x72,0x73,0x69,0x6f,0x6e,0x5c,0x57,0x69,0x6e,0x64,0x6f,0x77,0x73,0x00];
pub static VAL_MDALIGN: &[u16] = &[0x4d,0x65,0x6e,0x75,0x44,0x72,0x6f,0x70,0x41,0x6c,0x69,0x67,0x6e,0x6d,0x65,0x6e,0x74,0x00];
pub static KEY_METRIC: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x44,0x65,0x73,0x6b,0x74,0x6f,0x70,0x5c,0x57,0x69,0x6e,0x64,0x6f,0x77,0x4d,0x65,0x74,0x72,0x69,0x63,0x73,0x00];
pub static VAL_BORDER: &[u16] = &[0x42,0x6f,0x72,0x64,0x65,0x72,0x57,0x69,0x64,0x74,0x68,0x00];
pub static VAL_ICONSPC: &[u16] = &[0x49,0x63,0x6f,0x6e,0x53,0x70,0x61,0x63,0x69,0x6e,0x67,0x00];
pub static VAL_ICONVSPC: &[u16] = &[0x49,0x63,0x6f,0x6e,0x56,0x65,0x72,0x74,0x69,0x63,0x61,0x6c,0x73,0x70,0x61,0x63,0x69,0x6e,0x67,0x00];
pub static VAL_ITWRAP: &[u16] = &[0x49,0x63,0x6f,0x6e,0x54,0x69,0x74,0x6c,0x65,0x57,0x72,0x61,0x70,0x00];
pub static KEY_SOUND: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x53,0x6f,0x75,0x6e,0x64,0x00];
pub static VAL_BEEP: &[u16] = &[0x42,0x65,0x65,0x70,0x00];
pub static KEY_KBD: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x4b,0x65,0x79,0x62,0x6f,0x61,0x72,0x64,0x00];
pub static VAL_KBDSPD: &[u16] = &[0x4b,0x65,0x79,0x62,0x6f,0x61,0x72,0x64,0x53,0x70,0x65,0x65,0x64,0x00];
pub static VAL_KBDDELAY: &[u16] = &[0x4b,0x65,0x79,0x62,0x6f,0x61,0x72,0x64,0x44,0x65,0x6c,0x61,0x79,0x00];
pub static KEY_SHOWSNDS: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x41,0x63,0x63,0x65,0x73,0x73,0x69,0x62,0x69,0x6c,0x69,0x74,0x79,0x5c,0x53,0x68,0x6f,0x77,0x53,0x6f,0x75,0x6e,0x64,0x73,0x00];
pub static KEY_KDBPREF: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x41,0x63,0x63,0x65,0x73,0x73,0x69,0x62,0x69,0x6c,0x69,0x74,0x79,0x5c,0x4b,0x65,0x79,0x62,0x6f,0x61,0x72,0x64,0x20,0x50,0x72,0x65,0x66,0x65,0x72,0x65,0x6e,0x63,0x65,0x00];
pub static KEY_SCRREAD: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x41,0x63,0x63,0x65,0x73,0x73,0x69,0x62,0x69,0x6c,0x69,0x74,0x79,0x5c,0x42,0x6c,0x69,0x6e,0x64,0x20,0x41,0x63,0x63,0x65,0x73,0x73,0x00];
pub static VAL_ON: &[u16] = &[0x4f,0x6e,0x00];
pub static KEY_MOUSEKEYS: &[u16] = &[0x43,0x6f,0x6e,0x74,0x72,0x6f,0x6c,0x20,0x50,0x61,0x6e,0x65,0x6c,0x5c,0x41,0x63,0x63,0x65,0x73,0x73,0x69,0x62,0x69,0x6c,0x69,0x74,0x79,0x5c,0x4d,0x6f,0x75,0x73,0x65,0x4b,0x65,0x79,0x73,0x00];
pub static VAL_MOUSEKEYS_FLAGS: &[u16] = &[0x46,0x6c,0x61,0x67,0x73,0x00];
pub static VAL_MOUSEKEYS_MAX: &[u16] = &[0x4d,0x61,0x78,0x69,0x6d,0x75,0x6d,0x53,0x70,0x65,0x65,0x64,0x00];
pub static VAL_MOUSEKEYS_TIMETOMAX: &[u16] = &[0x54,0x69,0x6d,0x65,0x54,0x6f,0x4d,0x61,0x78,0x69,0x6d,0x75,0x6d,0x53,0x70,0x65,0x65,0x64,0x00];


unsafe fn SpiLoadDWord(pwszKey: PCWSTR, pwszValue: PCWSTR, iValue: INT) -> INT {
    let mut Result: DWORD = 0;
    if RegReadUserSetting(
        pwszKey,
        pwszValue,
        REG_DWORD,
        core::ptr::addr_of_mut!(Result) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as u32,
    ) == 0
    {
        return iValue;
    }
    Result as INT
}

unsafe fn SpiLoadInt(pwszKey: PCWSTR, pwszValue: PCWSTR, iValue: INT) -> INT {
    let mut awcBuffer: [u16; 12] = [0; 12];
    let cbSize: ULONG = core::mem::size_of_val(&awcBuffer) as ULONG;
    if RegReadUserSetting(
        pwszKey,
        pwszValue,
        REG_SZ,
        awcBuffer.as_mut_ptr() as *mut core::ffi::c_void,
        cbSize,
    ) == 0
    {
        return iValue;
    }
    _wtoi(awcBuffer.as_ptr())
}

unsafe fn SpiLoadUserPrefMask(dValue: DWORD) -> DWORD {
    let mut Result: DWORD = 0;
    if RegReadUserSetting(
        KEY_DESKTOP.as_ptr(),
        VAL_USERPREFMASK.as_ptr(),
        REG_BINARY,
        core::ptr::addr_of_mut!(Result) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as u32,
    ) == 0
    {
        return dValue;
    }
    Result
}

unsafe fn SpiLoadTimeOut() -> DWORD {
    let mut szApplicationName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    RtlZeroMemory(
        szApplicationName.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&szApplicationName),
    );
    if RegReadUserSetting(
        KEY_DESKTOP.as_ptr(),
        VAL_SCRNSV.as_ptr(),
        REG_SZ,
        szApplicationName.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&szApplicationName) as u32,
    ) == 0
    {
        return 0;
    }
    if szApplicationName[0] == 0 {
        return 0;
    }
    SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_SCRTO.as_ptr(), 600) as DWORD
}

unsafe fn SpiLoadMouse(pwszValue: PCWSTR, iValue: INT) -> INT {
    SpiLoadInt(KEY_MOUSE.as_ptr(), pwszValue, iValue)
}

unsafe fn SpiLoadMetric(pwszValue: PCWSTR, iValue: INT) -> INT {
    let iRegVal: INT = SpiLoadInt(KEY_METRIC.as_ptr(), pwszValue, -(((iValue * 1440) - 0) / dpi));
    if iRegVal > 0 {
        iRegVal
    } else {
        (-(iRegVal) * dpi + 720) / 1440
    }
}

unsafe fn SpiLoadFont(plfOut: PLOGFONTW, pwszValueName: LPWSTR, plfDefault: PLOGFONTW) {
    let bResult = RegReadUserSetting(
        KEY_METRIC.as_ptr(),
        pwszValueName,
        REG_BINARY,
        plfOut as *mut core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>() as u32,
    );
    if bResult == 0 {
        *plfOut = *plfDefault;
    }
}

unsafe fn SpiFixupValues() {
    gspv.ncm.iCaptionWidth = core::cmp::max(gspv.ncm.iCaptionWidth, 8);
    gspv.ncm.iBorderWidth = core::cmp::max(gspv.ncm.iBorderWidth, 1);
    gspv.ncm.iScrollWidth = core::cmp::max(gspv.ncm.iScrollWidth, 8);
    gspv.ncm.iScrollHeight = core::cmp::max(gspv.ncm.iScrollHeight, 8);
    if gspv.iDblClickTime == 0 {
        gspv.iDblClickTime = 500;
    }

    gspv.tmMenuFont.tmHeight = 11;
    gspv.tmMenuFont.tmExternalLeading = 2;

    gspv.tmCaptionFont.tmHeight = 11;
    gspv.tmCaptionFont.tmExternalLeading = 2;
}

unsafe fn SpiUpdatePerUserSystemParameters() {
    static mut lf1: LOGFONTW = LOGFONTW {
        lfHeight: -11,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: FW_NORMAL as _,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: ANSI_CHARSET as _,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: DEFAULT_QUALITY as _,
        lfPitchAndFamily: (VARIABLE_PITCH | FF_SWISS) as _,
        lfFaceName: [
            b'M' as u16, b'S' as u16, b' ' as u16, b'S' as u16, b'a' as u16,
            b'n' as u16, b's' as u16, b' ' as u16, b'S' as u16, b'e' as u16,
            b'r' as u16, b'i' as u16, b'f' as u16, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    };
    static mut lf2: LOGFONTW = LOGFONTW {
        lfHeight: -11,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: FW_BOLD as _,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: ANSI_CHARSET as _,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: DEFAULT_QUALITY as _,
        lfPitchAndFamily: (VARIABLE_PITCH | FF_SWISS) as _,
        lfFaceName: [
            b'M' as u16, b'S' as u16, b' ' as u16, b'S' as u16, b'a' as u16,
            b'n' as u16, b's' as u16, b' ' as u16, b'S' as u16, b'e' as u16,
            b'r' as u16, b'i' as u16, b'f' as u16, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    };

    /* Clear the structure */
    core::ptr::write_bytes(core::ptr::addr_of_mut!(gspv), 0, 1);

    /* Load mouse settings */
    gspv.caiMouse.FirstThreshold = SpiLoadMouse(VAL_MOUSE1.as_ptr(), 6) as _;
    gspv.caiMouse.SecondThreshold = SpiLoadMouse(VAL_MOUSE2.as_ptr(), 10) as _;
    gspv.caiMouse.Acceleration = SpiLoadMouse(VAL_MOUSE3.as_ptr(), 1) as _;
    gspv.iMouseSpeed = SpiLoadMouse(VAL_SENSITIVITY.as_ptr(), 10);
    gspv.bMouseBtnSwap = SpiLoadMouse(VAL_SWAP.as_ptr(), 0);
    gspv.bSnapToDefBtn = SpiLoadMouse(VAL_SNAPDEFBTN.as_ptr(), 0);
    gspv.iMouseTrails = SpiLoadMouse(VAL_MOUSETRAILS.as_ptr(), 0);
    gspv.iDblClickTime = SpiLoadMouse(VAL_DBLCLKTIME.as_ptr(), 500);
    gspv.iDblClickWidth = SpiLoadMouse(VAL_DBLCLKWIDTH.as_ptr(), 4);
    gspv.iDblClickHeight = SpiLoadMouse(VAL_DBLCLKHEIGHT.as_ptr(), 4);
    gspv.iMouseHoverTime = SpiLoadMouse(VAL_HOVERTIME.as_ptr(), 400);
    gspv.iMouseHoverWidth = SpiLoadMouse(VAL_HOVERWIDTH.as_ptr(), 4);
    gspv.iMouseHoverHeight = SpiLoadMouse(VAL_HOVERHEIGHT.as_ptr(), 4);

    /* Load keyboard settings */
    gspv.dwKbdSpeed = SpiLoadInt(KEY_KBD.as_ptr(), VAL_KBDSPD.as_ptr(), 31) as _;
    gspv.iKbdDelay = SpiLoadInt(KEY_KBD.as_ptr(), VAL_KBDDELAY.as_ptr(), 1);

    /* Load NONCLIENTMETRICS */
    gspv.ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as _;
    gspv.ncm.iBorderWidth = SpiLoadMetric(VAL_BORDER.as_ptr(), 1);
    gspv.ncm.iScrollWidth = SpiLoadMetric(w!("ScrollWidth").as_ptr() as PCWSTR, 16);
    gspv.ncm.iScrollHeight = SpiLoadMetric(w!("ScrollHeight").as_ptr() as PCWSTR, 16);
    gspv.ncm.iCaptionWidth = SpiLoadMetric(w!("CaptionWidth").as_ptr() as PCWSTR, 19);
    gspv.ncm.iCaptionHeight = SpiLoadMetric(w!("CaptionHeight").as_ptr() as PCWSTR, 19);
    gspv.ncm.iSmCaptionWidth = SpiLoadMetric(w!("SmCaptionWidth").as_ptr() as PCWSTR, 12);
    gspv.ncm.iSmCaptionHeight = SpiLoadMetric(w!("SmCaptionHeight").as_ptr() as PCWSTR, 15);
    gspv.ncm.iMenuWidth = SpiLoadMetric(w!("MenuWidth").as_ptr() as PCWSTR, 18);
    gspv.ncm.iMenuHeight = SpiLoadMetric(w!("MenuHeight").as_ptr() as PCWSTR, 18);
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.ncm.lfCaptionFont), w!("CaptionFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf2));
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.ncm.lfSmCaptionFont), w!("SmCaptionFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf1));
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.ncm.lfMenuFont), w!("MenuFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf1));
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.ncm.lfStatusFont), w!("StatusFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf1));
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.ncm.lfMessageFont), w!("MessageFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf1));

    /* Load MINIMIZEDMETRICS */
    gspv.mm.cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as _;
    gspv.mm.iWidth = SpiLoadMetric(w!("MinWidth").as_ptr() as PCWSTR, 160);
    gspv.mm.iHorzGap = SpiLoadMetric(w!("MinHorzGap").as_ptr() as PCWSTR, 160);
    gspv.mm.iVertGap = SpiLoadMetric(w!("MinVertGap").as_ptr() as PCWSTR, 24);
    gspv.mm.iArrange = SpiLoadInt(KEY_METRIC.as_ptr(), w!("MinArrange").as_ptr() as PCWSTR, ARW_HIDE as _);

    /* Load ICONMETRICS */
    gspv.im.cbSize = core::mem::size_of::<ICONMETRICSW>() as _;
    gspv.im.iHorzSpacing = SpiLoadMetric(VAL_ICONSPC.as_ptr(), 64);
    gspv.im.iVertSpacing = SpiLoadMetric(VAL_ICONVSPC.as_ptr(), 64);
    gspv.im.iTitleWrap = SpiLoadMetric(VAL_ITWRAP.as_ptr(), 1);
    SpiLoadFont(core::ptr::addr_of_mut!(gspv.im.lfFont), w!("IconFont").as_ptr() as LPWSTR, core::ptr::addr_of_mut!(lf1));

    /* Load desktop settings */
    gspv.bDragFullWindows = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_DRAG.as_ptr(), 0);
    gspv.iWheelScrollLines = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_SCRLLLINES.as_ptr(), 3);
    gspv.dwMouseClickLockTime = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_CLICKLOCKTIME.as_ptr(), 1200) as _;
    (*pusi(gpsi)).dtCaretBlink = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_CARETRATE.as_ptr(), 530) as _;
    gspv.dwCaretWidth = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_CARETWIDTH.as_ptr(), 1) as _;
    gspv.dwUserPrefMask = SpiLoadUserPrefMask(UPM_DEFAULT);
    gspv.bMouseClickLock = ((gspv.dwUserPrefMask & UPM_CLICKLOCK) != 0) as _;
    gspv.bMouseCursorShadow = ((gspv.dwUserPrefMask & UPM_CURSORSHADOW) != 0) as _;
    gspv.bFontSmoothing = (SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_FONTSMOOTHING.as_ptr(), 0) == 2) as _;
    gspv.uiFontSmoothingType = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_FONTSMOOTHINGTYPE.as_ptr(), 1) as _;
    gspv.uiFontSmoothingContrast = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_FONTSMOOTHINGCONTRAST.as_ptr(), 1400) as _;
    gspv.uiFontSmoothingOrientation = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_FONTSMOOTHINGORIENTATION.as_ptr(), 1) as _;

    /* Some hardcoded values for now */

    gspv.tmCaptionFont.tmAveCharWidth = 6;
    gspv.bBeep = 1u8 as _;
    gspv.uiFocusBorderWidth = 1;
    gspv.uiFocusBorderHeight = 1;
    gspv.bMenuDropAlign = 0;
    gspv.dwMenuShowDelay = SpiLoadInt(KEY_DESKTOP.as_ptr(), w!("MenuShowDelay").as_ptr() as PCWSTR, 400) as _;
    gspv.dwForegroundFlashCount = 3;

    gspv.iScrSaverTimeout = SpiLoadTimeOut() as _;
    gspv.bScrSaverActive = 0u8 as _;
    gspv.bScrSaverRunning = 0u8 as _;

    gspv.bFastTaskSwitch = 1u8 as _;

    gspv.accesstimeout.cbSize = core::mem::size_of::<ACCESSTIMEOUT>() as _;
    gspv.filterkeys.cbSize = core::mem::size_of::<FILTERKEYS>() as _;
    gspv.togglekeys.cbSize = core::mem::size_of::<TOGGLEKEYS>() as _;
    gspv.mousekeys.cbSize = core::mem::size_of::<MOUSEKEYS>() as _;
    gspv.mousekeys.dwFlags = SpiLoadInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_FLAGS.as_ptr(), 62) as _;
    gspv.mousekeys.iMaxSpeed = SpiLoadInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_MAX.as_ptr(), 80) as _;
    gspv.mousekeys.iTimeToMaxSpeed = SpiLoadInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_TIMETOMAX.as_ptr(), 3000) as _;
    gspv.mousekeys.iCtrlSpeed = 8;
    gspv.stickykeys.cbSize = core::mem::size_of::<STICKYKEYS>() as _;
    gspv.serialkeys.cbSize = core::mem::size_of::<SERIALKEYS>() as _;
    gspv.soundsentry.cbSize = core::mem::size_of::<SOUNDSENTRYW>() as _;
    gspv.highcontrast.cbSize = core::mem::size_of::<HIGHCONTRASTW>() as _;
    gspv.animationinfo.cbSize = core::mem::size_of::<ANIMATIONINFO>() as _;

    g_bWindowSnapEnabled = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_SNAP_ENABLED.as_ptr(), 1);
    gspv.bDockMoving = SpiLoadInt(KEY_DESKTOP.as_ptr(), VAL_SNAP_DOCKMOVING.as_ptr(), 1);

    /* Make sure we don't use broken values */
    SpiFixupValues();

    /* Update SystemMetrics */
    InitMetrics();

    if gbSpiInitialized != 0 && !gpsi.is_null() {
        if gspv.bKbdPref != 0 {
            (*gpsi).dwSRVIFlags |= SRVINFO_KBDPREF;
        }
        if (gspv.dwUserPrefMask & UPM_KEYBOARDCUES) != 0 {
            (*pusi(gpsi)).PUSIFlags |= PUSIF_KEYBOARDCUES;
        }
        if (gspv.dwUserPrefMask & UPM_COMBOBOXANIMATION) != 0 {
            (*pusi(gpsi)).PUSIFlags |= PUSIF_COMBOBOXANIMATION;
        }
        if (gspv.dwUserPrefMask & UPM_LISTBOXSMOOTHSCROLLING) != 0 {
            (*pusi(gpsi)).PUSIFlags |= PUSIF_LISTBOXSMOOTHSCROLLING;
        }
    }
    gdwLanguageToggleKey = UserGetLanguageToggle(w!("Language Hotkey").as_ptr() as LPWSTR, 1);
    gdwLayoutToggleKey = UserGetLanguageToggle(w!("Layout Hotkey").as_ptr() as LPWSTR, 2);
}

// InitSysParams (sysparams.h: cdecl `BOOL InitSysParams()`) — exported for main.rs.
#[no_mangle]
unsafe extern "C" fn InitSysParams() -> BOOL {
    SpiUpdatePerUserSystemParameters();
    gbSpiInitialized = 1;
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUpdatePerUserSystemParameters(
    dwReserved: DWORD,
    bEnable: BOOL,
) -> BOOL {
    let bResult: BOOL;

    UserEnterExclusive();

    SpiUpdatePerUserSystemParameters();
    if bEnable != 0 {
        g_PaintDesktopVersion = SpiLoadDWord(KEY_DESKTOP.as_ptr(), VAL_PAINTDESKVER.as_ptr(), 0);
    } else {
        g_PaintDesktopVersion = 0;
    }
    bResult = 1;

    UserLeave();

    bResult
}

unsafe fn SpiStoreDWord(pwszKey: PCWSTR, pwszValue: PCWSTR, Value: DWORD) {
    let mut Value = Value;
    RegWriteUserSetting(
        pwszKey,
        pwszValue,
        REG_DWORD,
        core::ptr::addr_of_mut!(Value) as *mut core::ffi::c_void,
        core::mem::size_of::<DWORD>() as u32,
    );
}

unsafe fn SpiStoreSz(pwszKey: PCWSTR, pwszValue: PCWSTR, pwsz: PCWSTR) {

    RegWriteUserSetting(
        pwszKey,
        pwszValue,
        REG_SZ as u32,
        pwsz as *const core::ffi::c_void,
        ((wcslen(pwsz) + 1) * core::mem::size_of::<u16>()) as u32,
    );
}

unsafe fn SpiStoreSzInt(pwszKey: PCWSTR, pwszValue: PCWSTR, iValue: INT) {
    let mut awcBuffer: [u16; 15] = [0; 15];

    _itow(iValue, awcBuffer.as_mut_ptr(), 10);
    RegWriteUserSetting(
        pwszKey,
        pwszValue,
        REG_SZ as u32,
        awcBuffer.as_ptr() as *const core::ffi::c_void,
        ((wcslen(awcBuffer.as_ptr()) + 1) * core::mem::size_of::<u16>() as usize) as u32,
    );
}

unsafe fn SpiStoreMetric(pwszValue: LPCWSTR, iValue: INT) {
    SpiStoreSzInt(KEY_METRIC.as_ptr(), pwszValue, -(((iValue * 1440) - 0) / dpi));
}

unsafe fn SpiStoreFont(pwszValue: PCWSTR, plogfont: *mut LOGFONTW) {
    RegWriteUserSetting(
        KEY_METRIC.as_ptr(),
        pwszValue,
        REG_BINARY,
        plogfont as *mut core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>() as u32,
    );
}

unsafe fn SpiMemCopy(pvDst: PVOID, pvSrc: PVOID, cbSize: ULONG, bProtect: BOOL) -> UINT_PTR {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if bProtect != 0 {
        // sysparams.c's ONE _SEH2_EXCEPT (sysparams.c ~462-470) wraps ONLY
        // this RtlCopyMemory -- no ProbeForRead/Write at all, since pvDst/
        // pvSrc are handed to SpiMemCopy already validated or kernel-owned by
        // the caller (SpiGet/SpiSet pass the raw SPI buffer pointers through).
        // The handler assigns Status and FALLS THROUGH -- no goto, no return,
        // no cleanup -- straight into the `if (!NT_SUCCESS(Status))` below,
        // which is what `Status` already existing in this fn's scope, and the
        // unconditional fall-out to `NT_SUCCESS(Status) as UINT_PTR`, exist to
        // serve. The translation kept that shape but replaced the guarded
        // copy with a bare one, so a hostile SPI pointer bugchecked instead of
        // returning FALSE with an error set. `safe_copy_raw` is the direct
        // match for a bare SEH-wrapped RtlCopyMemory with no probe.
        if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(pvDst, pvSrc, cbSize as usize) {
            Status = st;
        }
    } else {
        core::ptr::copy_nonoverlapping(pvSrc as *const u8, pvDst as *mut u8, cbSize as usize);
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        ERR!("SpiMemCopy failed, pvDst=%p, pvSrc=%p, bProtect=%d\n", pvDst, pvSrc, bProtect);
    }

    NT_SUCCESS(Status) as UINT_PTR
}

unsafe fn SpiGet(pvParam: PVOID, pvData: PVOID, cbSize: ULONG, fl: FLONG) -> UINT_PTR {
    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }
    SpiMemCopy(pvParam, pvData, cbSize, (fl & SPIF_PROTECT) as BOOL)
}

unsafe fn SpiSet(pvData: PVOID, pvParam: PVOID, cbSize: ULONG, fl: FLONG) -> UINT_PTR {
    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }
    SpiMemCopy(pvData, pvParam, cbSize, (fl & SPIF_PROTECT) as BOOL)
}

unsafe fn SpiGetEx(pvParam: PVOID, pvData: PVOID, cbSize: ULONG, fl: FLONG) -> UINT_PTR {
    let mut cbBufSize: ULONG = 0;
    if SpiSet(core::ptr::addr_of_mut!(cbBufSize) as PVOID, pvParam, core::mem::size_of::<ULONG>() as ULONG, fl) == 0 {
        return 0;
    }
    if cbBufSize != cbSize {
        return 0;
    }
    SpiGet(pvParam, pvData, cbSize, fl)
}

unsafe fn SpiGetInt(pvParam: PVOID, piValue: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGet(pvParam, piValue, core::mem::size_of::<INT>() as ULONG, fl)
}

unsafe fn SpiSetYesNo(
    pbData: *mut BOOL,
    bValue: BOOL,
    pwszKey: PCWSTR,
    pwszValue: PCWSTR,
    fl: FLONG,
) -> UINT_PTR {
    if (*(PsGetCurrentProcessWin32Process() as PPROCESSINFO)).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }
    *pbData = if bValue != 0 { 1 } else { 0 };
    if fl & SPIF_UPDATEINIFILE != 0 {
        let pwsz = if bValue != 0 {
            wide_str!("Yes")
        } else {
            wide_str!("No")
        };
        SpiStoreSz(pwszKey, pwszValue, pwsz);
    }
    pwszKey as UINT_PTR
}

unsafe fn SpiSetBool(pbData: *mut BOOL, iValue: INT, pwszKey: PCWSTR, pwszValue: PCWSTR, fl: FLONG) -> UINT_PTR {
    if (*(PsGetCurrentProcessWin32Process() as PPROCESSINFO)).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }
    *pbData = if iValue != 0 { 1 } else { 0 };
    if (fl & SPIF_UPDATEINIFILE) != 0 {
        SpiStoreSzInt(pwszKey, pwszValue, iValue);
    }
    pwszKey as UINT_PTR
}

unsafe fn SpiSetDWord(pvData: PVOID, iValue: INT, pwszKey: PCWSTR, pwszValue: PCWSTR, fl: FLONG) -> UINT_PTR {
    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION as DWORD);
        return 0;
    }
    *(pvData as *mut INT) = iValue;
    if fl & SPIF_UPDATEINIFILE != 0 {
        SpiStoreDWord(pwszKey, pwszValue, iValue as DWORD);
    }
    pwszKey as UINT_PTR
}

unsafe fn SpiSetInt(pvData: *mut core::ffi::c_void, iValue: INT, pwszKey: PCWSTR, pwszValue: PCWSTR, fl: FLONG) -> UINT_PTR {
    if (*(PsGetCurrentProcessWin32Process() as PPROCESSINFO)).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }
    *(pvData as *mut INT) = iValue;
    if (fl & SPIF_UPDATEINIFILE) != 0 {
        SpiStoreSzInt(pwszKey, pwszValue, iValue);
    }
    pwszKey as UINT_PTR
}

unsafe fn SpiSetMetric(pvData: PVOID, iValue: INT, pwszValue: PCWSTR, fl: FLONG) -> UINT_PTR {
    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }
    *(pvData as *mut INT) = iValue;
    if fl & SPIF_UPDATEINIFILE != 0 {
        SpiStoreMetric(pwszValue, iValue);
    }
    KEY_METRIC.as_ptr() as UINT_PTR
}

unsafe fn SpiSetUserPref(dwMask: u32, pvValue: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    let mut dwRegMask: u32 = 0;
    let bValue: u8 = (PtrToUlong(pvValue) != 0) as u8;

    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }

    gspv.dwUserPrefMask = if bValue != 0 {
        gspv.dwUserPrefMask | dwMask
    } else {
        gspv.dwUserPrefMask & !dwMask
    };

    if (fl & SPIF_UPDATEINIFILE) != 0 {
        if RegReadUserSetting(
            KEY_DESKTOP.as_ptr(),
            VAL_USERPREFMASK.as_ptr(),
            REG_BINARY,
            core::ptr::addr_of_mut!(dwRegMask) as *mut core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
        ) == 0
        {
            dwRegMask = 0;
        }

        dwRegMask = if bValue != 0 {
            dwRegMask | dwMask
        } else {
            dwRegMask & !dwMask
        };

        RegWriteUserSetting(
            KEY_DESKTOP.as_ptr(),
            VAL_USERPREFMASK.as_ptr(),
            REG_BINARY,
            core::ptr::addr_of_mut!(dwRegMask) as *mut core::ffi::c_void,
            core::mem::size_of::<u32>() as u32,
        );
    }

    KEY_DESKTOP.as_ptr() as UINT_PTR
}

unsafe fn SpiGetUserPref(dwMask: DWORD, pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    let mut iValue: INT = if gspv.dwUserPrefMask & dwMask != 0 { 1 } else { 0 };
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(iValue) as PVOID, fl)
}

unsafe fn SpiSetWallpaper(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    let mut ustr: _UNICODE_STRING = core::mem::zeroed();
    let mut awc: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut bResult: BOOL;
    let hbmp: HBITMAP;
    let hOldBitmap: HBITMAP;
    let psurfBmp: *mut SURFACE;
    let ulTile: ULONG;
    let ulStyle: ULONG;

    if (*GetW32ProcessInfo()).prpwinsta != InputWindowStation {
        EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
        return 0;
    }

    if pvParam.is_null() {
        return KEY_DESKTOP.as_ptr() as UINT_PTR;
    }

    bResult = SpiMemCopy(
        core::ptr::addr_of_mut!(ustr) as PVOID,
        pvParam,
        core::mem::size_of::<_UNICODE_STRING>() as ULONG,
        (fl & SPIF_PROTECT) as BOOL,
    ) as BOOL;
    if bResult == 0 {
        return 0;
    }
    if ustr.Length as usize > MAX_PATH as usize * core::mem::size_of::<WCHAR>() {
        return 0;
    }

    bResult = SpiMemCopy(
        (*(&raw mut gspv)).awcWallpaper.as_mut_ptr() as PVOID,
        ustr.Buffer as PVOID,
        ustr.Length as ULONG,
        (fl & SPIF_PROTECT) as BOOL,
    ) as BOOL;
    if bResult == 0 {
        return 0;
    }

    gspv.ustrWallpaper.Buffer = (*(&raw mut gspv)).awcWallpaper.as_mut_ptr();
    gspv.ustrWallpaper.MaximumLength = (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as u16;
    gspv.ustrWallpaper.Length = ustr.Length;
    gspv.awcWallpaper[ustr.Length as usize / core::mem::size_of::<WCHAR>()] = 0;

    if fl & SPIF_UPDATEINIFILE != 0 {
        SpiStoreSz(KEY_DESKTOP.as_ptr(), wide_str!("Wallpaper"), (*(&raw const gspv)).awcWallpaper.as_ptr());
    }

    if gspv.awcWallpaper[0] != 0 {
        ustr.Buffer = awc.as_mut_ptr();
        ustr.MaximumLength = (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as u16;
        ustr.Length = 0;
        if W32kDosPathNameToNtPathName((*(&raw const gspv)).awcWallpaper.as_ptr(), core::ptr::addr_of_mut!(ustr)) == 0 {
            return 0;
        }

        hbmp = UserLoadImage(ustr.Buffer);
        if hbmp.is_null() {
            return 0;
        }

        psurfBmp = SURFACE_ShareLockSurface(hbmp);
        if psurfBmp.is_null() {
            GreDeleteObject(hbmp as HGDIOBJ);
            return 0;
        }

        gspv.cxWallpaper = (*psurfBmp).SurfObj.sizlBitmap.cx as _;
        gspv.cyWallpaper = (*psurfBmp).SurfObj.sizlBitmap.cy as _;
        gspv.WallpaperMode = wmCenter;

        SURFACE_ShareUnlockSurface(psurfBmp);

        GreSetObjectOwner(hbmp as HGDIOBJ, GDI_OBJ_HMGR_PUBLIC);

        ulTile = SpiLoadInt(KEY_DESKTOP.as_ptr(), wide_str!("TileWallpaper"), 0) as ULONG;
        ulStyle = SpiLoadInt(KEY_DESKTOP.as_ptr(), wide_str!("WallpaperStyle"), 0) as ULONG;

        if ulTile != 0 && ulStyle == 0 {
            gspv.WallpaperMode = wmTile;
        } else if ulTile == 0 && ulStyle != 0 {
            if ulStyle == 2 {
                gspv.WallpaperMode = wmStretch;
            } else if ulStyle == 6 {
                gspv.WallpaperMode = wmFit;
            } else if ulStyle == 10 {
                gspv.WallpaperMode = wmFill;
            }
        }
    } else {
        gspv.cxWallpaper = 0;
        gspv.cyWallpaper = 0;
        hbmp = core::ptr::null_mut();
    }

    hOldBitmap = gspv.hbmWallpaper as HBITMAP;
    if !hOldBitmap.is_null() {
        GreSetObjectOwner(hOldBitmap as HGDIOBJ, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject(hOldBitmap as HGDIOBJ);
    }

    gspv.hbmWallpaper = hbmp as HANDLE;

    NtUserRedrawWindow(UserGetShellWindow(), core::ptr::null_mut(), core::ptr::null_mut(), RDW_INVALIDATE | RDW_ERASE);

    KEY_DESKTOP.as_ptr() as UINT_PTR
}

unsafe fn SpiNotifyNCMetricsChanged() -> BOOL {
    let pwndDesktop: PWND;
    let mut pwndCurrent: PWND;
    let ahwnd: *mut HWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let mut i: i32;

    pwndDesktop = UserGetDesktopWindow();

    ahwnd = IntWinListChildren(pwndDesktop);
    if ahwnd.is_null() {
        return 0;
    }

    i = 0;
    while !(*ahwnd.offset(i as isize)).is_null() {
        pwndCurrent = UserGetWindowObject(*ahwnd.offset(i as isize));
        if pwndCurrent.is_null() {
            i += 1;
            continue;
        }

        UserRefObjectCo(pwndCurrent, &mut Ref);
        co_WinPosSetWindowPos(
            pwndCurrent,
            core::ptr::null_mut(),
            (*pwndCurrent).rcWindow.left,
            (*pwndCurrent).rcWindow.top,
            (*pwndCurrent).rcWindow.right - (*pwndCurrent).rcWindow.left,
            (*pwndCurrent).rcWindow.bottom - (*pwndCurrent).rcWindow.top,
            (SWP_FRAMECHANGED | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOMOVE | SWP_NOZORDER | SWP_NOREDRAW) as u32,
        );
        UserDerefObjectCo(pwndCurrent);

        i += 1;
    }

    ExFreePoolWithTag(ahwnd as *mut core::ffi::c_void, USERTAG_WINDOWLIST);

    1
}

unsafe fn SpiGetSet_SPI_GETBEEP(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bBeep) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETBEEP(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetYesNo(
        core::ptr::addr_of_mut!(gspv.bBeep),
        uiParam as BOOL,
        KEY_SOUND.as_ptr(),
        VAL_BEEP.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.caiMouse) as PVOID, (3 * core::mem::size_of::<INT>()) as ULONG, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if SpiSet(
        core::ptr::addr_of_mut!(gspv.caiMouse) as PVOID,
        pvParam,
        (3 * core::mem::size_of::<INT>()) as ULONG,
        fl,
    ) == 0
    {
        return 0;
    }
    if (fl & SPIF_UPDATEINIFILE) != 0 {
        SpiStoreSzInt(KEY_MOUSE.as_ptr(), VAL_MOUSE1.as_ptr(), gspv.caiMouse.FirstThreshold as INT);
        SpiStoreSzInt(KEY_MOUSE.as_ptr(), VAL_MOUSE2.as_ptr(), gspv.caiMouse.SecondThreshold as INT);
        SpiStoreSzInt(KEY_MOUSE.as_ptr(), VAL_MOUSE3.as_ptr(), gspv.caiMouse.Acceleration as INT);
    }
    KEY_MOUSE.as_ptr() as UINT_PTR
}

unsafe fn SpiGetSet_SPI_GETBORDER(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.ncm.iBorderWidth) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETBORDER(mut uiParam: UINT, fl: FLONG) -> UINT_PTR {
    uiParam = if uiParam > 1 { uiParam } else { 1 };
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.ncm.iBorderWidth) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_METRIC.as_ptr(),
        VAL_BORDER.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETKEYBOARDSPEED(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwKbdSpeed) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETKEYBOARDSPEED(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.dwKbdSpeed) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_KBD.as_ptr(),
        VAL_KBDSPD.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETSCREENSAVETIMEOUT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iScrSaverTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSCREENSAVETIMEOUT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(core::ptr::addr_of_mut!(gspv.iScrSaverTimeout) as *mut core::ffi::c_void, uiParam as INT, KEY_DESKTOP.as_ptr(), VAL_SCRTO.as_ptr(), fl)
}

unsafe fn SpiGetSet_SPI_GETSCREENSAVEACTIVE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bScrSaverActive) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSCREENSAVEACTIVE(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.bScrSaverActive) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_SCRACT.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETGRIDGRANULARITY(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiGridGranularity) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETGRIDGRANULARITY(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.uiGridGranularity) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_GRID.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETDESKWALLPAPER(mut uiParam: UINT, pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    let max = (gspv.ustrWallpaper.Length as ULONG).wrapping_add(1);
    if uiParam > max {
        uiParam = max;
    }
    SpiGet(pvParam, (*(&raw mut gspv)).awcWallpaper.as_mut_ptr() as PVOID, uiParam, fl)
}

unsafe fn SpiGetSet_SPI_SETDESKWALLPAPER(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiSetWallpaper(pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETKEYBOARDDELAY(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iKbdDelay) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETKEYBOARDDELAY(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iKbdDelay) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_KBD.as_ptr(),
        VAL_KBDDELAY.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_ICONHORIZONTALSPACING(mut uiParam: UINT, pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if !pvParam.is_null() {
        return SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.im.iHorzSpacing) as PVOID, fl);
    }
    if uiParam < 32 {
        uiParam = 32;
    }
    SpiSetMetric(core::ptr::addr_of_mut!(gspv.im.iHorzSpacing) as PVOID, uiParam as INT, VAL_ICONSPC.as_ptr(), fl)
}

unsafe fn SpiGetSet_SPI_ICONVERTICALSPACING(mut uiParam: UINT, pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if !pvParam.is_null() {
        return SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.im.iVertSpacing) as PVOID, fl);
    }
    if uiParam < 32 {
        uiParam = 32;
    }
    SpiSetMetric(core::ptr::addr_of_mut!(gspv.im.iVertSpacing) as PVOID, uiParam as INT, VAL_ICONVSPC.as_ptr(), fl)
}

unsafe fn SpiGetSet_SPI_GETICONTITLEWRAP(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.im.iTitleWrap) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETICONTITLEWRAP(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.im.iTitleWrap) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_METRIC.as_ptr(),
        VAL_ITWRAP.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMENUDROPALIGNMENT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bMenuDropAlign) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMENUDROPALIGNMENT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bMenuDropAlign),
        uiParam as INT,
        KEY_MDALIGN.as_ptr(),
        VAL_MDALIGN.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETDOUBLECLKWIDTH(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iDblClickWidth) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        VAL_DBLCLKWIDTH.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETDOUBLECLKHEIGHT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iDblClickHeight) as *mut core::ffi::c_void,
        uiParam as INT,
        VAL_DBLCLKHEIGHT.as_ptr(),
        VAL_DBLCLKHEIGHT.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETICONTITLELOGFONT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGet(
        pvParam,
        core::ptr::addr_of_mut!(gspv.im.lfFont) as PVOID,
        core::mem::size_of::<LOGFONTW>() as ULONG,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETICONTITLELOGFONT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if SpiSet(
        core::ptr::addr_of_mut!(gspv.im.lfFont) as PVOID,
        pvParam,
        core::mem::size_of::<LOGFONTW>() as ULONG,
        fl,
    ) == 0
    {
        return 0;
    }
    if fl & SPIF_UPDATEINIFILE != 0 {
        SpiStoreFont(
            wide_str!("IconFont"),
            core::ptr::addr_of_mut!(gspv.im.lfFont),
        );
    }
    KEY_METRIC.as_ptr() as UINT_PTR
}

unsafe fn SpiGetSet_SPI_SETDOUBLECLICKTIME(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iDblClickTime) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        VAL_DBLCLKTIME.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETMOUSEBUTTONSWAP(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.bMouseBtnSwap) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        VAL_SWAP.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFASTTASKSWITCH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bFastTaskSwitch) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFASTTASKSWITCH() -> UINT_PTR {
    1
}

unsafe fn SpiGetSet_SPI_GETDRAGFULLWINDOWS(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bDragFullWindows) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETDRAGFULLWINDOWS(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.bDragFullWindows) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_DRAG.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETSHOWSOUNDS(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bShowSounds) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSHOWSOUNDS(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bShowSounds),
        uiParam as INT,
        KEY_SHOWSNDS.as_ptr(),
        VAL_ON.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETKEYBOARDPREF(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bKbdPref) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETKEYBOARDPREF(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(core::ptr::addr_of_mut!(gspv.bKbdPref), uiParam as INT, KEY_KDBPREF.as_ptr(), VAL_ON.as_ptr(), fl)
}

unsafe fn SpiGetSet_SPI_GETSCREENREADER(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bScreenReader) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSCREENREADER(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bScreenReader),
        uiParam as INT,
        KEY_SCRREAD.as_ptr(),
        VAL_ON.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETANIMATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGet(
        pvParam,
        core::ptr::addr_of_mut!(gspv.animationinfo) as PVOID,
        core::mem::size_of::<ANIMATIONINFO>() as ULONG,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETANIMATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if SpiSet(
        core::ptr::addr_of_mut!(gspv.animationinfo) as PVOID,
        pvParam,
        core::mem::size_of::<ANIMATIONINFO>() as ULONG,
        fl,
    ) == 0
    {
        return 0;
    }
    if (fl & SPIF_UPDATEINIFILE) != 0 {
        // FIXME: What to do?
    }
    KEY_DESKTOP.as_ptr() as UINT_PTR
}

unsafe fn SpiGetSet_SPI_GETFONTSMOOTHING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bFontSmoothing) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFONTSMOOTHING(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    gspv.bFontSmoothing = if uiParam != 0 { 1 } else { 0 };
    if fl & SPIF_UPDATEINIFILE != 0 {
        SpiStoreSzInt(KEY_DESKTOP.as_ptr(), VAL_FONTSMOOTHING.as_ptr(), if gspv.bFontSmoothing != 0 { 2 } else { 0 });
    }
    KEY_DESKTOP.as_ptr() as UINT_PTR
}

unsafe fn SpiGetSet_SPI_SETDRAGWIDTH(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iDragWidth) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_DRAGWIDTH.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETDRAGHEIGHT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iDragHeight) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_DRAGHEIGHT.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETHANDHELD(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bHandHeld),
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        w!("HandHeld").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETLOWPOWERTIMEOUT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iLowPwrTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_GETPOWEROFFTIMEOUT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iPwrOffTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETLOWPOWERTIMEOUT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iLowPwrTimeout) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        w!("LowPowerTimeOut").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETPOWEROFFTIMEOUT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iPwrOffTimeout) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        w!("PowerOffTimeOut").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETLOWPOWERACTIVE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iPwrOffTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_GETPOWEROFFACTIVE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bPwrOffActive) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETLOWPOWERACTIVE(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bLowPwrActive),
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        ['L' as u16, 'o' as u16, 'w' as u16, 'P' as u16, 'o' as u16, 'w' as u16, 'e' as u16, 'r' as u16, 'A' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0u16].as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETPOWEROFFACTIVE(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bPwrOffActive),
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        w!("PowerOffActive").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETDEFAULTINPUTLANG(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    if gspklBaseLayout.is_null() {
        return 0;
    }

    SpiGet(
        pvParam,
        core::ptr::addr_of_mut!((*gspklBaseLayout).hkl) as PVOID,
        core::mem::size_of::<HKL>() as ULONG,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_SETLANGTOGGLE() -> UINT_PTR {
    gdwLayoutToggleKey = UserGetLanguageToggle(
        w!("Layout Hotkey").as_ptr() as LPWSTR,
        2,
    );
    gdwLanguageToggleKey = UserGetLanguageToggle(
        w!("Language Hotkey").as_ptr() as LPWSTR,
        1,
    );
    gdwLanguageToggleKey as UINT_PTR
}

unsafe fn SpiGetSet_SPI_GETMOUSETRAILS(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iMouseTrails) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSETRAILS(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iMouseTrails) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        VAL_MOUSETRAILS.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETSNAPTODEFBUTTON(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bSnapToDefBtn) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSNAPTODEFBUTTON(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(core::ptr::addr_of_mut!(gspv.bSnapToDefBtn), uiParam as INT, KEY_MOUSE.as_ptr(), VAL_SNAPDEFBTN.as_ptr(), fl)
}

unsafe fn SpiGetSet_SPI_GETMOUSEHOVERWIDTH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iMouseHoverWidth) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSEHOVERWIDTH(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iMouseHoverWidth) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        VAL_HOVERWIDTH.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSEHOVERHEIGHT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iMouseHoverHeight) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSEHOVERHEIGHT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iMouseHoverHeight) as *mut core::ffi::c_void,
        uiParam as INT,
        VAL_HOVERHEIGHT.as_ptr(),
        VAL_HOVERHEIGHT.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSEHOVERTIME(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iMouseHoverTime) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSEHOVERTIME(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iMouseHoverTime) as *mut core::ffi::c_void,
        uiParam as INT,
        VAL_HOVERTIME.as_ptr(),
        VAL_HOVERTIME.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETWHEELSCROLLLINES(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iWheelScrollLines) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETWHEELSCROLLLINES(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.iWheelScrollLines) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_SCRLLLINES.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMENUSHOWDELAY(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwMenuShowDelay) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMENUSHOWDELAY(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.dwMenuShowDelay) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        wide_str!("MenuShowDelay"),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETSHOWIMEUI(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bShowImeUi) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSHOWIMEUI(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bShowImeUi),
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSESPEED(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.iMouseSpeed) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_GETSCREENSAVERRUNNING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bScrSaverRunning) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETSCREENSAVERRUNNING(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bScrSaverRunning),
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETACTIVEWINDOWTRACKING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_ACTIVEWINDOWTRACKING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETACTIVEWINDOWTRACKING(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_ACTIVEWINDOWTRACKING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETMENUANIMATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_MENUANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETMENUANIMATION(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_MENUANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETCOMBOBOXANIMATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_COMBOBOXANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETCOMBOBOXANIMATION(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_COMBOBOXANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETLISTBOXSMOOTHSCROLLING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_LISTBOXSMOOTHSCROLLING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETLISTBOXSMOOTHSCROLLING(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_LISTBOXSMOOTHSCROLLING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETGRADIENTCAPTIONS(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_GRADIENTCAPTIONS, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETKEYBOARDCUES(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_KEYBOARDCUES, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETKEYBOARDCUES(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_KEYBOARDCUES, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETACTIVEWNDTRKZORDER(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_ACTIVEWNDTRKZORDER, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETACTIVEWNDTRKZORDER(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_ACTIVEWNDTRKZORDER, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETHOTTRACKING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_HOTTRACKING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETHOTTRACKING(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_HOTTRACKING, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETMENUFADE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_MENUFADE as DWORD, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETMENUFADE(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_MENUFADE, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETSELECTIONFADE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_SELECTIONFADE, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETSELECTIONFADE(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_SELECTIONFADE, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETTOOLTIPANIMATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_TOOLTIPANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETTOOLTIPANIMATION(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_TOOLTIPANIMATION, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETTOOLTIPFADE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_TOOLTIPFADE, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETTOOLTIPFADE(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_TOOLTIPFADE, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETCURSORSHADOW(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_CURSORSHADOW, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETCURSORSHADOW(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    gspv.bMouseCursorShadow = PtrToUlong(pvParam) as _;
    SpiSetUserPref(UPM_CURSORSHADOW, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETUIEFFECTS(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_UIEFFECTS, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETUIEFFECTS(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_UIEFFECTS, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETMOUSESONAR(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bMouseSonar) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSESONAR(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bMouseSonar),
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSECLICKLOCK(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_CLICKLOCK, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSECLICKLOCK(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    gspv.bMouseClickLock = PtrToUlong(pvParam) as _;
    SpiSetUserPref(UPM_CLICKLOCK, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETMOUSEVANISH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bMouseVanish) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSEVANISH(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bMouseVanish),
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFLATMENU(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_FLATMENU, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETFLATMENU(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_FLATMENU, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETDROPSHADOW(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetUserPref(UPM_DROPSHADOW, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_SETDROPSHADOW(pvParam: *mut core::ffi::c_void, fl: FLONG) -> UINT_PTR {
    SpiSetUserPref(UPM_DROPSHADOW, pvParam, fl)
}

unsafe fn SpiGetSet_SPI_GETBLOCKSENDINPUTRESETS(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bBlockSendInputResets) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETBLOCKSENDINPUTRESETS(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetBool(
        core::ptr::addr_of_mut!(gspv.bBlockSendInputResets),
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        [0u16].as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFOREGROUNDLOCKTIMEOUT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwForegroundLockTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFOREGROUNDLOCKTIMEOUT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.dwForegroundLockTimeout) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETACTIVEWNDTRKTIMEOUT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwActiveTrackingTimeout) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETACTIVEWNDTRKTIMEOUT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.dwActiveTrackingTimeout) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFOREGROUNDFLASHCOUNT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwForegroundFlashCount) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFOREGROUNDFLASHCOUNT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.dwForegroundFlashCount) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        wide_str!(""),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETCARETWIDTH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwCaretWidth) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETCARETWIDTH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiSetDWord(
        core::ptr::addr_of_mut!(gspv.dwCaretWidth) as PVOID,
        PtrToUlong(pvParam) as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_CARETWIDTH.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETMOUSECLICKLOCKTIME(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.dwMouseClickLockTime) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETMOUSECLICKLOCKTIME(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetDWord(
        core::ptr::addr_of_mut!(gspv.dwMouseClickLockTime) as PVOID,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_CLICKLOCKTIME.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFONTSMOOTHINGTYPE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiFontSmoothingType) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFONTSMOOTHINGTYPE(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiSetDWord(
        core::ptr::addr_of_mut!(gspv.uiFontSmoothingType) as PVOID,
        PtrToUlong(pvParam) as INT,
        VAL_FONTSMOOTHINGTYPE.as_ptr(),
        VAL_FONTSMOOTHINGTYPE.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFONTSMOOTHINGCONTRAST(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiFontSmoothingContrast) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFONTSMOOTHINGCONTRAST(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiSetDWord(
        core::ptr::addr_of_mut!(gspv.uiFontSmoothingContrast) as PVOID,
        PtrToUlong(pvParam) as INT,
        VAL_FONTSMOOTHINGCONTRAST.as_ptr() as PCWSTR,
        KEY_DESKTOP.as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFOCUSBORDERWIDTH(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiFocusBorderWidth) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFOCUSBORDERWIDTH(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.uiFocusBorderWidth) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        w!("").as_ptr() as PCWSTR,
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFOCUSBORDERHEIGHT(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiFocusBorderHeight) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFOCUSBORDERHEIGHT(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.uiFocusBorderHeight) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_MOUSE.as_ptr(),
        wide_str!(""),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETFONTSMOOTHINGORIENTATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.uiFontSmoothingOrientation) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETFONTSMOOTHINGORIENTATION(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiSetDWord(
        core::ptr::addr_of_mut!(gspv.uiFontSmoothingOrientation) as PVOID,
        PtrToUlong(pvParam) as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_FONTSMOOTHINGORIENTATION.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETWINARRANGING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(g_bWindowSnapEnabled) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETWINARRANGING(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(g_bWindowSnapEnabled) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_SNAP_ENABLED.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet_SPI_GETDOCKMOVING(pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    SpiGetInt(pvParam, core::ptr::addr_of_mut!(gspv.bDockMoving) as PVOID, fl)
}

unsafe fn SpiGetSet_SPI_SETDOCKMOVING(uiParam: UINT, fl: FLONG) -> UINT_PTR {
    SpiSetInt(
        core::ptr::addr_of_mut!(gspv.bDockMoving) as *mut core::ffi::c_void,
        uiParam as INT,
        KEY_DESKTOP.as_ptr(),
        VAL_SNAP_DOCKMOVING.as_ptr(),
        fl,
    )
}

unsafe fn SpiGetSet(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fl: FLONG) -> UINT_PTR {
    match uiAction {
        SPI_GETBEEP => return SpiGetSet_SPI_GETBEEP(pvParam, fl),
        SPI_SETBEEP => return SpiGetSet_SPI_SETBEEP(uiParam, fl),
        SPI_GETMOUSE => return SpiGetSet_SPI_GETMOUSE(pvParam, fl),
        SPI_SETMOUSE => return SpiGetSet_SPI_SETMOUSE(pvParam, fl),
        SPI_GETBORDER => return SpiGetSet_SPI_GETBORDER(pvParam, fl),
        SPI_SETBORDER => return SpiGetSet_SPI_SETBORDER(uiParam, fl),
        SPI_GETKEYBOARDSPEED => return SpiGetSet_SPI_GETKEYBOARDSPEED(pvParam, fl),
        SPI_SETKEYBOARDSPEED => return SpiGetSet_SPI_SETKEYBOARDSPEED(uiParam, fl),
        SPI_LANGDRIVER => {}
        SPI_GETSCREENSAVETIMEOUT => return SpiGetSet_SPI_GETSCREENSAVETIMEOUT(pvParam, fl),
        SPI_SETSCREENSAVETIMEOUT => return SpiGetSet_SPI_SETSCREENSAVETIMEOUT(uiParam, fl),
        SPI_GETSCREENSAVEACTIVE => return SpiGetSet_SPI_GETSCREENSAVEACTIVE(pvParam, fl),
        SPI_SETSCREENSAVEACTIVE => return SpiGetSet_SPI_SETSCREENSAVEACTIVE(uiParam, fl),
        SPI_GETGRIDGRANULARITY => return SpiGetSet_SPI_GETGRIDGRANULARITY(pvParam, fl),
        SPI_SETGRIDGRANULARITY => return SpiGetSet_SPI_SETGRIDGRANULARITY(uiParam, fl),
        SPI_GETDESKWALLPAPER => return SpiGetSet_SPI_GETDESKWALLPAPER(uiParam, pvParam, fl),
        SPI_SETDESKWALLPAPER => return SpiGetSet_SPI_SETDESKWALLPAPER(pvParam, fl),
        SPI_SETDESKPATTERN => {}
        SPI_GETKEYBOARDDELAY => return SpiGetSet_SPI_GETKEYBOARDDELAY(pvParam, fl),
        SPI_SETKEYBOARDDELAY => return SpiGetSet_SPI_SETKEYBOARDDELAY(uiParam, fl),
        SPI_ICONHORIZONTALSPACING => return SpiGetSet_SPI_ICONHORIZONTALSPACING(uiParam, pvParam, fl),
        SPI_ICONVERTICALSPACING => return SpiGetSet_SPI_ICONVERTICALSPACING(uiParam, pvParam, fl),
        SPI_GETICONTITLEWRAP => return SpiGetSet_SPI_GETICONTITLEWRAP(pvParam, fl),
        SPI_SETICONTITLEWRAP => return SpiGetSet_SPI_SETICONTITLEWRAP(uiParam, fl),
        SPI_GETMENUDROPALIGNMENT => return SpiGetSet_SPI_GETMENUDROPALIGNMENT(pvParam, fl),
        SPI_SETMENUDROPALIGNMENT => return SpiGetSet_SPI_SETMENUDROPALIGNMENT(uiParam, fl),
        SPI_SETDOUBLECLKWIDTH => return SpiGetSet_SPI_SETDOUBLECLKWIDTH(uiParam, fl),
        SPI_SETDOUBLECLKHEIGHT => return SpiGetSet_SPI_SETDOUBLECLKHEIGHT(uiParam, fl),
        SPI_GETICONTITLELOGFONT => return SpiGetSet_SPI_GETICONTITLELOGFONT(pvParam, fl),
        SPI_SETICONTITLELOGFONT => return SpiGetSet_SPI_SETICONTITLELOGFONT(pvParam, fl),
        SPI_SETDOUBLECLICKTIME => return SpiGetSet_SPI_SETDOUBLECLICKTIME(uiParam, fl),
        SPI_SETMOUSEBUTTONSWAP => return SpiGetSet_SPI_SETMOUSEBUTTONSWAP(uiParam, fl),
        SPI_GETFASTTASKSWITCH => return SpiGetSet_SPI_GETFASTTASKSWITCH(pvParam, fl),
        SPI_SETFASTTASKSWITCH => return SpiGetSet_SPI_SETFASTTASKSWITCH(),
        SPI_GETDRAGFULLWINDOWS => return SpiGetSet_SPI_GETDRAGFULLWINDOWS(pvParam, fl),
        SPI_SETDRAGFULLWINDOWS => return SpiGetSet_SPI_SETDRAGFULLWINDOWS(uiParam, fl),
        SPI_GETNONCLIENTMETRICS => {
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.ncm) as PVOID, core::mem::size_of::<NONCLIENTMETRICSW>() as ULONG, fl);
        }
        SPI_SETNONCLIENTMETRICS => {
            let metrics = pvParam as LPNONCLIENTMETRICSW;
            (*metrics).cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
            if SpiSet(core::ptr::addr_of_mut!(gspv.ncm) as PVOID, metrics as PVOID, core::mem::size_of::<NONCLIENTMETRICSW>() as ULONG, fl) == 0 {
                return 0;
            }
            if (fl & SPIF_UPDATEINIFILE) != 0 {
                SpiStoreMetric(VAL_BORDER.as_ptr(), gspv.ncm.iBorderWidth);
                SpiStoreMetric(w!("ScrollWidth").as_ptr() as LPCWSTR, gspv.ncm.iScrollWidth);
                SpiStoreMetric(w!("ScrollHeight").as_ptr() as LPCWSTR, gspv.ncm.iScrollHeight);
                SpiStoreMetric(w!("CaptionWidth").as_ptr() as LPCWSTR, gspv.ncm.iCaptionWidth);
                SpiStoreMetric(w!("CaptionHeight").as_ptr() as LPCWSTR, gspv.ncm.iCaptionHeight);
                SpiStoreMetric(w!("SmCaptionWidth").as_ptr() as LPCWSTR, gspv.ncm.iSmCaptionWidth);
                SpiStoreMetric(w!("SmCaptionHeight").as_ptr() as LPCWSTR, gspv.ncm.iSmCaptionHeight);
                SpiStoreMetric(w!("MenuWidth").as_ptr() as LPCWSTR, gspv.ncm.iMenuWidth);
                SpiStoreMetric(w!("MenuHeight").as_ptr() as LPCWSTR, gspv.ncm.iMenuHeight);
                SpiStoreFont(w!("CaptionFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.ncm.lfCaptionFont));
                SpiStoreFont(w!("SmCaptionFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.ncm.lfSmCaptionFont));
                SpiStoreFont(w!("MenuFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.ncm.lfMenuFont));
                SpiStoreFont(w!("StatusFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.ncm.lfStatusFont));
                SpiStoreFont(w!("MessageFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.ncm.lfMessageFont));
            }
            if SpiNotifyNCMetricsChanged() == 0 {
                return 0;
            }
            return KEY_METRIC.as_ptr() as UINT_PTR;
        }
        SPI_GETMINIMIZEDMETRICS => {
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.mm) as PVOID, core::mem::size_of::<MINIMIZEDMETRICS>() as ULONG, fl);
        }
        SPI_SETMINIMIZEDMETRICS => {
            let metrics = pvParam as LPMINIMIZEDMETRICS;
            (*metrics).cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as UINT;
            if SpiSet(core::ptr::addr_of_mut!(gspv.mm) as PVOID, metrics as PVOID, core::mem::size_of::<MINIMIZEDMETRICS>() as ULONG, fl) == 0 {
                return 0;
            }
            gspv.mm.iWidth = core::cmp::max(0, gspv.mm.iWidth);
            gspv.mm.iHorzGap = core::cmp::max(0, gspv.mm.iHorzGap);
            gspv.mm.iVertGap = core::cmp::max(0, gspv.mm.iVertGap);
            gspv.mm.iArrange = gspv.mm.iArrange & 0xf;
            if (fl & SPIF_UPDATEINIFILE) != 0 {
                SpiStoreMetric(w!("MinWidth").as_ptr() as LPCWSTR, gspv.mm.iWidth);
                SpiStoreMetric(w!("MinHorzGap").as_ptr() as LPCWSTR, gspv.mm.iHorzGap);
                SpiStoreMetric(w!("MinVertGap").as_ptr() as LPCWSTR, gspv.mm.iVertGap);
                SpiStoreMetric(w!("MinArrange").as_ptr() as LPCWSTR, gspv.mm.iArrange);
            }
            return KEY_METRIC.as_ptr() as UINT_PTR;
        }
        SPI_GETICONMETRICS => {
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.im) as PVOID, core::mem::size_of::<ICONMETRICSW>() as ULONG, fl);
        }
        SPI_SETICONMETRICS => {
            let metrics = pvParam as LPICONMETRICSW;
            (*metrics).cbSize = core::mem::size_of::<ICONMETRICSW>() as UINT;
            if SpiSet(core::ptr::addr_of_mut!(gspv.im) as PVOID, metrics as PVOID, core::mem::size_of::<ICONMETRICSW>() as ULONG, fl) == 0 {
                return 0;
            }
            if (fl & SPIF_UPDATEINIFILE) != 0 {
                SpiStoreMetric(VAL_ICONSPC.as_ptr(), gspv.im.iHorzSpacing);
                SpiStoreMetric(VAL_ICONVSPC.as_ptr(), gspv.im.iVertSpacing);
                SpiStoreMetric(VAL_ITWRAP.as_ptr(), gspv.im.iTitleWrap);
                SpiStoreFont(w!("IconFont").as_ptr() as PCWSTR, core::ptr::addr_of_mut!(gspv.im.lfFont));
            }
            return KEY_METRIC.as_ptr() as UINT_PTR;
        }
        SPI_GETWORKAREA => {
            let pmonitor = UserGetPrimaryMonitor();
            if pmonitor.is_null() {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!((*pmonitor).rcWork) as PVOID, core::mem::size_of::<RECTL>() as ULONG, fl);
        }
        SPI_SETWORKAREA => {
            let mut rcWorkArea: RECTL = core::mem::zeroed();
            let mut rcIntersect: RECTL = core::mem::zeroed();
            if pvParam.is_null() {
                return 0;
            }
            core::ptr::copy_nonoverlapping(pvParam as *const u8, core::ptr::addr_of_mut!(rcWorkArea) as *mut u8, core::mem::size_of::<RECTL>());
            if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(rcWorkArea)) != 0 {
                return 0;
            }
            let pmonitor = UserMonitorFromRect(core::ptr::addr_of_mut!(rcWorkArea), MONITOR_DEFAULTTONEAREST);
            if pmonitor.is_null() {
                return 0;
            }
            if RECTL_bIntersectRect(core::ptr::addr_of_mut!(rcIntersect), core::ptr::addr_of_mut!((*pmonitor).rcMonitor), core::ptr::addr_of_mut!(rcWorkArea)) == 0
                || RtlEqualMemory(core::ptr::addr_of_mut!(rcIntersect) as PVOID, core::ptr::addr_of_mut!(rcWorkArea) as PVOID, core::mem::size_of::<RECTL>()) == 0
            {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!((*pmonitor).rcWork) as PVOID, pvParam, core::mem::size_of::<RECTL>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_SETPENWINDOWS => {}
        SPI_GETFILTERKEYS => {
            let FilterKeys = pvParam as LPFILTERKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<FILTERKEYS>() as UINT {
                return 0;
            }
            if FilterKeys.is_null() || (*FilterKeys).cbSize != core::mem::size_of::<FILTERKEYS>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.filterkeys) as PVOID, core::mem::size_of::<FILTERKEYS>() as ULONG, fl);
        }
        SPI_SETFILTERKEYS => {
            let FilterKeys = pvParam as LPFILTERKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<FILTERKEYS>() as UINT {
                return 0;
            }
            if FilterKeys.is_null() || (*FilterKeys).cbSize != core::mem::size_of::<FILTERKEYS>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.filterkeys) as PVOID, pvParam, core::mem::size_of::<FILTERKEYS>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETTOGGLEKEYS => {
            let ToggleKeys = pvParam as LPTOGGLEKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<TOGGLEKEYS>() as UINT {
                return 0;
            }
            if ToggleKeys.is_null() || (*ToggleKeys).cbSize != core::mem::size_of::<TOGGLEKEYS>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.togglekeys) as PVOID, core::mem::size_of::<TOGGLEKEYS>() as ULONG, fl);
        }
        SPI_SETTOGGLEKEYS => {
            let ToggleKeys = pvParam as LPTOGGLEKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<TOGGLEKEYS>() as UINT {
                return 0;
            }
            if ToggleKeys.is_null() || (*ToggleKeys).cbSize != core::mem::size_of::<TOGGLEKEYS>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.togglekeys) as PVOID, pvParam, core::mem::size_of::<TOGGLEKEYS>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETMOUSEKEYS => {
            let MouseKeys = pvParam as LPMOUSEKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<MOUSEKEYS>() as UINT {
                return 0;
            }
            if MouseKeys.is_null() || (*MouseKeys).cbSize != core::mem::size_of::<MOUSEKEYS>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.mousekeys) as PVOID, core::mem::size_of::<MOUSEKEYS>() as ULONG, fl);
        }
        SPI_SETMOUSEKEYS => {
            let MouseKeys = pvParam as LPMOUSEKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<MOUSEKEYS>() as UINT {
                return 0;
            }
            if MouseKeys.is_null() || (*MouseKeys).cbSize != core::mem::size_of::<MOUSEKEYS>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.mousekeys) as PVOID, pvParam, core::mem::size_of::<MOUSEKEYS>() as ULONG, fl) == 0 {
                return 0;
            }
            if (fl & SPIF_UPDATEINIFILE) != 0 {
                SpiStoreSzInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_FLAGS.as_ptr(), gspv.mousekeys.dwFlags as INT);
                SpiStoreSzInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_MAX.as_ptr(), gspv.mousekeys.iMaxSpeed as INT);
                SpiStoreSzInt(KEY_MOUSEKEYS.as_ptr(), VAL_MOUSEKEYS_TIMETOMAX.as_ptr(), gspv.mousekeys.iTimeToMaxSpeed as INT);
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETSHOWSOUNDS => return SpiGetSet_SPI_GETSHOWSOUNDS(pvParam, fl),
        SPI_SETSHOWSOUNDS => return SpiGetSet_SPI_SETSHOWSOUNDS(uiParam, fl),
        SPI_GETSTICKYKEYS => {
            let StickyKeys = pvParam as LPSTICKYKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<STICKYKEYS>() as UINT {
                return 0;
            }
            if StickyKeys.is_null() || (*StickyKeys).cbSize != core::mem::size_of::<STICKYKEYS>() as UINT {
                return 0;
            }
            return SpiGetEx(pvParam, core::ptr::addr_of_mut!(gspv.stickykeys) as PVOID, core::mem::size_of::<STICKYKEYS>() as ULONG, fl);
        }
        SPI_SETSTICKYKEYS => {
            let StickyKeys = pvParam as LPSTICKYKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<STICKYKEYS>() as UINT {
                return 0;
            }
            if StickyKeys.is_null() || (*StickyKeys).cbSize != core::mem::size_of::<STICKYKEYS>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.stickykeys) as PVOID, pvParam, core::mem::size_of::<STICKYKEYS>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETACCESSTIMEOUT => {
            let AccessTimeout = pvParam as LPACCESSTIMEOUT;
            if uiParam != 0 && uiParam != core::mem::size_of::<ACCESSTIMEOUT>() as UINT {
                return 0;
            }
            if AccessTimeout.is_null() || (*AccessTimeout).cbSize != core::mem::size_of::<ACCESSTIMEOUT>() as UINT {
                return 0;
            }
            return SpiGetEx(pvParam, core::ptr::addr_of_mut!(gspv.accesstimeout) as PVOID, core::mem::size_of::<ACCESSTIMEOUT>() as ULONG, fl);
        }
        SPI_SETACCESSTIMEOUT => {
            let AccessTimeout = pvParam as LPACCESSTIMEOUT;
            if uiParam != 0 && uiParam != core::mem::size_of::<ACCESSTIMEOUT>() as UINT {
                return 0;
            }
            if AccessTimeout.is_null() || (*AccessTimeout).cbSize != core::mem::size_of::<ACCESSTIMEOUT>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.accesstimeout) as PVOID, pvParam, core::mem::size_of::<ACCESSTIMEOUT>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETSERIALKEYS => {
            let SerialKeys = pvParam as LPSERIALKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<SERIALKEYS>() as UINT {
                return 0;
            }
            if SerialKeys.is_null() || (*SerialKeys).cbSize != core::mem::size_of::<SERIALKEYS>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.serialkeys) as PVOID, core::mem::size_of::<SERIALKEYS>() as ULONG, fl);
        }
        SPI_SETSERIALKEYS => {
            let SerialKeys = pvParam as LPSERIALKEYS;
            if uiParam != 0 && uiParam != core::mem::size_of::<SERIALKEYS>() as UINT {
                return 0;
            }
            if SerialKeys.is_null() || (*SerialKeys).cbSize != core::mem::size_of::<SERIALKEYS>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.serialkeys) as PVOID, pvParam, core::mem::size_of::<SERIALKEYS>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETSOUNDSENTRY => {
            let SoundsEntry = pvParam as LPSOUNDSENTRYW;
            if uiParam != 0 && uiParam != core::mem::size_of::<SOUNDSENTRYW>() as UINT {
                return 0;
            }
            if SoundsEntry.is_null() || (*SoundsEntry).cbSize != core::mem::size_of::<SOUNDSENTRYW>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.soundsentry) as PVOID, core::mem::size_of::<SOUNDSENTRYW>() as ULONG, fl);
        }
        SPI_SETSOUNDSENTRY => {
            let SoundsEntry = pvParam as LPSOUNDSENTRYW;
            if uiParam != 0 && uiParam != core::mem::size_of::<SOUNDSENTRYW>() as UINT {
                return 0;
            }
            if SoundsEntry.is_null() || (*SoundsEntry).cbSize != core::mem::size_of::<SOUNDSENTRYW>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.soundsentry) as PVOID, pvParam, core::mem::size_of::<SOUNDSENTRYW>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETHIGHCONTRAST => {
            let highcontrast = pvParam as LPHIGHCONTRASTW;
            if uiParam != 0 && uiParam != core::mem::size_of::<HIGHCONTRASTW>() as UINT {
                return 0;
            }
            if highcontrast.is_null() || (*highcontrast).cbSize != core::mem::size_of::<HIGHCONTRASTW>() as UINT {
                return 0;
            }
            return SpiGet(pvParam, core::ptr::addr_of_mut!(gspv.highcontrast) as PVOID, core::mem::size_of::<HIGHCONTRASTW>() as ULONG, fl);
        }
        SPI_SETHIGHCONTRAST => {
            let highcontrast = pvParam as LPHIGHCONTRASTW;
            if uiParam != 0 && uiParam != core::mem::size_of::<HIGHCONTRASTW>() as UINT {
                return 0;
            }
            if highcontrast.is_null() || (*highcontrast).cbSize != core::mem::size_of::<HIGHCONTRASTW>() as UINT {
                return 0;
            }
            if SpiSet(core::ptr::addr_of_mut!(gspv.highcontrast) as PVOID, pvParam, core::mem::size_of::<HIGHCONTRASTW>() as ULONG, fl) == 0 {
                return 0;
            }
            return KEY_DESKTOP.as_ptr() as UINT_PTR;
        }
        SPI_GETKEYBOARDPREF => return SpiGetSet_SPI_GETKEYBOARDPREF(pvParam, fl),
        SPI_SETKEYBOARDPREF => return SpiGetSet_SPI_SETKEYBOARDPREF(uiParam, fl),
        SPI_GETSCREENREADER => return SpiGetSet_SPI_GETSCREENREADER(pvParam, fl),
        SPI_SETSCREENREADER => return SpiGetSet_SPI_SETSCREENREADER(uiParam, fl),
        SPI_GETANIMATION => return SpiGetSet_SPI_GETANIMATION(pvParam, fl),
        SPI_SETANIMATION => return SpiGetSet_SPI_SETANIMATION(pvParam, fl),
        SPI_GETFONTSMOOTHING => return SpiGetSet_SPI_GETFONTSMOOTHING(pvParam, fl),
        SPI_SETFONTSMOOTHING => return SpiGetSet_SPI_SETFONTSMOOTHING(uiParam, fl),
        SPI_SETDRAGWIDTH => return SpiGetSet_SPI_SETDRAGWIDTH(uiParam, fl),
        SPI_SETDRAGHEIGHT => return SpiGetSet_SPI_SETDRAGHEIGHT(uiParam, fl),
        SPI_SETHANDHELD => return SpiGetSet_SPI_SETHANDHELD(uiParam, fl),
        SPI_GETLOWPOWERTIMEOUT => return SpiGetSet_SPI_GETLOWPOWERTIMEOUT(pvParam, fl),
        SPI_GETPOWEROFFTIMEOUT => return SpiGetSet_SPI_GETPOWEROFFTIMEOUT(pvParam, fl),
        SPI_SETLOWPOWERTIMEOUT => return SpiGetSet_SPI_SETLOWPOWERTIMEOUT(uiParam, fl),
        SPI_SETPOWEROFFTIMEOUT => return SpiGetSet_SPI_SETPOWEROFFTIMEOUT(uiParam, fl),
        SPI_GETLOWPOWERACTIVE => return SpiGetSet_SPI_GETLOWPOWERACTIVE(pvParam, fl),
        SPI_GETPOWEROFFACTIVE => return SpiGetSet_SPI_GETPOWEROFFACTIVE(pvParam, fl),
        SPI_SETLOWPOWERACTIVE => return SpiGetSet_SPI_SETLOWPOWERACTIVE(uiParam, fl),
        SPI_SETPOWEROFFACTIVE => return SpiGetSet_SPI_SETPOWEROFFACTIVE(uiParam, fl),
        SPI_SETCURSORS => {}
        SPI_SETICONS => {}
        SPI_GETDEFAULTINPUTLANG => return SpiGetSet_SPI_GETDEFAULTINPUTLANG(pvParam, fl),
        SPI_SETDEFAULTINPUTLANG => {
            let mut hkl: HKL = core::ptr::null_mut();
            if (fl & SPIF_UPDATEINIFILE) != 0 || SpiSet(core::ptr::addr_of_mut!(hkl) as PVOID, pvParam, core::mem::size_of::<HKL>() as ULONG, fl) == 0 {
                return 0;
            }
            return UserSetDefaultInputLang(hkl) as UINT_PTR;
        }
        SPI_SETLANGTOGGLE => return SpiGetSet_SPI_SETLANGTOGGLE(),
        SPI_GETWINDOWSEXTENSION => {}
        SPI_GETMOUSETRAILS => return SpiGetSet_SPI_GETMOUSETRAILS(pvParam, fl),
        SPI_SETMOUSETRAILS => return SpiGetSet_SPI_SETMOUSETRAILS(uiParam, fl),
        SPI_GETSNAPTODEFBUTTON => return SpiGetSet_SPI_GETSNAPTODEFBUTTON(pvParam, fl),
        SPI_SETSNAPTODEFBUTTON => return SpiGetSet_SPI_SETSNAPTODEFBUTTON(uiParam, fl),
        SPI_GETMOUSEHOVERWIDTH => return SpiGetSet_SPI_GETMOUSEHOVERWIDTH(pvParam, fl),
        SPI_SETMOUSEHOVERWIDTH => return SpiGetSet_SPI_SETMOUSEHOVERWIDTH(uiParam, fl),
        SPI_GETMOUSEHOVERHEIGHT => return SpiGetSet_SPI_GETMOUSEHOVERHEIGHT(pvParam, fl),
        SPI_SETMOUSEHOVERHEIGHT => return SpiGetSet_SPI_SETMOUSEHOVERHEIGHT(uiParam, fl),
        SPI_GETMOUSEHOVERTIME => return SpiGetSet_SPI_GETMOUSEHOVERTIME(pvParam, fl),
        SPI_SETMOUSEHOVERTIME => return SpiGetSet_SPI_SETMOUSEHOVERTIME(uiParam, fl),
        SPI_GETWHEELSCROLLLINES => return SpiGetSet_SPI_GETWHEELSCROLLLINES(pvParam, fl),
        SPI_SETWHEELSCROLLLINES => return SpiGetSet_SPI_SETWHEELSCROLLLINES(uiParam, fl),
        SPI_GETMENUSHOWDELAY => return SpiGetSet_SPI_GETMENUSHOWDELAY(pvParam, fl),
        SPI_SETMENUSHOWDELAY => return SpiGetSet_SPI_SETMENUSHOWDELAY(uiParam, fl),
        SPI_GETSHOWIMEUI => return SpiGetSet_SPI_GETSHOWIMEUI(pvParam, fl),
        SPI_SETSHOWIMEUI => return SpiGetSet_SPI_SETSHOWIMEUI(uiParam, fl),
        SPI_GETMOUSESPEED => return SpiGetSet_SPI_GETMOUSESPEED(pvParam, fl),
        SPI_SETMOUSESPEED => {
            if (pvParam as INT_PTR) < 1 || (pvParam as INT_PTR) > 20 {
                return 0;
            } else {
                return SpiSetInt(core::ptr::addr_of_mut!(gspv.iMouseSpeed) as *mut core::ffi::c_void, pvParam as INT_PTR as INT, KEY_MOUSE.as_ptr(), VAL_SENSITIVITY.as_ptr(), fl);
            }
        }
        SPI_GETSCREENSAVERRUNNING => return SpiGetSet_SPI_GETSCREENSAVERRUNNING(pvParam, fl),
        SPI_SETSCREENSAVERRUNNING => return SpiGetSet_SPI_SETSCREENSAVERRUNNING(uiParam, fl),
        SPI_GETACTIVEWINDOWTRACKING => return SpiGetSet_SPI_GETACTIVEWINDOWTRACKING(pvParam, fl),
        SPI_SETACTIVEWINDOWTRACKING => return SpiGetSet_SPI_SETACTIVEWINDOWTRACKING(pvParam, fl),
        SPI_GETMENUANIMATION => return SpiGetSet_SPI_GETMENUANIMATION(pvParam, fl),
        SPI_SETMENUANIMATION => return SpiGetSet_SPI_SETMENUANIMATION(pvParam, fl),
        SPI_GETCOMBOBOXANIMATION => return SpiGetSet_SPI_GETCOMBOBOXANIMATION(pvParam, fl),
        SPI_SETCOMBOBOXANIMATION => return SpiGetSet_SPI_SETCOMBOBOXANIMATION(pvParam, fl),
        SPI_GETLISTBOXSMOOTHSCROLLING => return SpiGetSet_SPI_GETLISTBOXSMOOTHSCROLLING(pvParam, fl),
        SPI_SETLISTBOXSMOOTHSCROLLING => return SpiGetSet_SPI_SETLISTBOXSMOOTHSCROLLING(pvParam, fl),
        SPI_GETGRADIENTCAPTIONS => {
            if NtGdiGetDeviceCaps(ScreenDeviceContext, BITSPIXEL as INT) <= 8 {
                let mut iValue: INT = 0;
                return SpiGetInt(pvParam, core::ptr::addr_of_mut!(iValue) as PVOID, fl);
            } else {
                return SpiGetUserPref(UPM_GRADIENTCAPTIONS, pvParam, fl);
            }
        }
        SPI_SETGRADIENTCAPTIONS => return SpiGetSet_SPI_SETGRADIENTCAPTIONS(pvParam, fl),
        SPI_GETKEYBOARDCUES => return SpiGetSet_SPI_GETKEYBOARDCUES(pvParam, fl),
        SPI_SETKEYBOARDCUES => return SpiGetSet_SPI_SETKEYBOARDCUES(pvParam, fl),
        SPI_GETACTIVEWNDTRKZORDER => return SpiGetSet_SPI_GETACTIVEWNDTRKZORDER(pvParam, fl),
        SPI_SETACTIVEWNDTRKZORDER => return SpiGetSet_SPI_SETACTIVEWNDTRKZORDER(pvParam, fl),
        SPI_GETHOTTRACKING => return SpiGetSet_SPI_GETHOTTRACKING(pvParam, fl),
        SPI_SETHOTTRACKING => return SpiGetSet_SPI_SETHOTTRACKING(pvParam, fl),
        SPI_GETMENUFADE => return SpiGetSet_SPI_GETMENUFADE(pvParam, fl),
        SPI_SETMENUFADE => return SpiGetSet_SPI_SETMENUFADE(pvParam, fl),
        SPI_GETSELECTIONFADE => return SpiGetSet_SPI_GETSELECTIONFADE(pvParam, fl),
        SPI_SETSELECTIONFADE => return SpiGetSet_SPI_SETSELECTIONFADE(pvParam, fl),
        SPI_GETTOOLTIPANIMATION => return SpiGetSet_SPI_GETTOOLTIPANIMATION(pvParam, fl),
        SPI_SETTOOLTIPANIMATION => return SpiGetSet_SPI_SETTOOLTIPANIMATION(pvParam, fl),
        SPI_GETTOOLTIPFADE => return SpiGetSet_SPI_GETTOOLTIPFADE(pvParam, fl),
        SPI_SETTOOLTIPFADE => return SpiGetSet_SPI_SETTOOLTIPFADE(pvParam, fl),
        SPI_GETCURSORSHADOW => return SpiGetSet_SPI_GETCURSORSHADOW(pvParam, fl),
        SPI_SETCURSORSHADOW => return SpiGetSet_SPI_SETCURSORSHADOW(pvParam, fl),
        SPI_GETUIEFFECTS => return SpiGetSet_SPI_GETUIEFFECTS(pvParam, fl),
        SPI_SETUIEFFECTS => return SpiGetSet_SPI_SETUIEFFECTS(pvParam, fl),
        SPI_GETMOUSESONAR => return SpiGetSet_SPI_GETMOUSESONAR(pvParam, fl),
        SPI_SETMOUSESONAR => return SpiGetSet_SPI_SETMOUSESONAR(uiParam, fl),
        SPI_GETMOUSECLICKLOCK => return SpiGetSet_SPI_GETMOUSECLICKLOCK(pvParam, fl),
        SPI_SETMOUSECLICKLOCK => return SpiGetSet_SPI_SETMOUSECLICKLOCK(pvParam, fl),
        SPI_GETMOUSEVANISH => return SpiGetSet_SPI_GETMOUSEVANISH(pvParam, fl),
        SPI_SETMOUSEVANISH => return SpiGetSet_SPI_SETMOUSEVANISH(uiParam, fl),
        SPI_GETFLATMENU => return SpiGetSet_SPI_GETFLATMENU(pvParam, fl),
        SPI_SETFLATMENU => return SpiGetSet_SPI_SETFLATMENU(pvParam, fl),
        SPI_GETDROPSHADOW => return SpiGetSet_SPI_GETDROPSHADOW(pvParam, fl),
        SPI_SETDROPSHADOW => return SpiGetSet_SPI_SETDROPSHADOW(pvParam, fl),
        SPI_GETBLOCKSENDINPUTRESETS => return SpiGetSet_SPI_GETBLOCKSENDINPUTRESETS(pvParam, fl),
        SPI_SETBLOCKSENDINPUTRESETS => return SpiGetSet_SPI_SETBLOCKSENDINPUTRESETS(uiParam, fl),
        SPI_GETFOREGROUNDLOCKTIMEOUT => return SpiGetSet_SPI_GETFOREGROUNDLOCKTIMEOUT(pvParam, fl),
        SPI_SETFOREGROUNDLOCKTIMEOUT => return SpiGetSet_SPI_SETFOREGROUNDLOCKTIMEOUT(uiParam, fl),
        SPI_GETACTIVEWNDTRKTIMEOUT => return SpiGetSet_SPI_GETACTIVEWNDTRKTIMEOUT(pvParam, fl),
        SPI_SETACTIVEWNDTRKTIMEOUT => return SpiGetSet_SPI_SETACTIVEWNDTRKTIMEOUT(uiParam, fl),
        SPI_GETFOREGROUNDFLASHCOUNT => return SpiGetSet_SPI_GETFOREGROUNDFLASHCOUNT(pvParam, fl),
        SPI_SETFOREGROUNDFLASHCOUNT => return SpiGetSet_SPI_SETFOREGROUNDFLASHCOUNT(uiParam, fl),
        SPI_GETCARETWIDTH => return SpiGetSet_SPI_GETCARETWIDTH(pvParam, fl),
        SPI_SETCARETWIDTH => return SpiGetSet_SPI_SETCARETWIDTH(pvParam, fl),
        SPI_GETMOUSECLICKLOCKTIME => return SpiGetSet_SPI_GETMOUSECLICKLOCKTIME(pvParam, fl),
        SPI_SETMOUSECLICKLOCKTIME => return SpiGetSet_SPI_SETMOUSECLICKLOCKTIME(uiParam, fl),
        SPI_GETFONTSMOOTHINGTYPE => return SpiGetSet_SPI_GETFONTSMOOTHINGTYPE(pvParam, fl),
        SPI_SETFONTSMOOTHINGTYPE => return SpiGetSet_SPI_SETFONTSMOOTHINGTYPE(pvParam, fl),
        SPI_GETFONTSMOOTHINGCONTRAST => return SpiGetSet_SPI_GETFONTSMOOTHINGCONTRAST(pvParam, fl),
        SPI_SETFONTSMOOTHINGCONTRAST => return SpiGetSet_SPI_SETFONTSMOOTHINGCONTRAST(pvParam, fl),
        SPI_GETFOCUSBORDERWIDTH => return SpiGetSet_SPI_GETFOCUSBORDERWIDTH(pvParam, fl),
        SPI_SETFOCUSBORDERWIDTH => return SpiGetSet_SPI_SETFOCUSBORDERWIDTH(uiParam, fl),
        SPI_GETFOCUSBORDERHEIGHT => return SpiGetSet_SPI_GETFOCUSBORDERHEIGHT(pvParam, fl),
        SPI_SETFOCUSBORDERHEIGHT => return SpiGetSet_SPI_SETFOCUSBORDERHEIGHT(uiParam, fl),
        SPI_GETFONTSMOOTHINGORIENTATION => return SpiGetSet_SPI_GETFONTSMOOTHINGORIENTATION(pvParam, fl),
        SPI_SETFONTSMOOTHINGORIENTATION => return SpiGetSet_SPI_SETFONTSMOOTHINGORIENTATION(pvParam, fl),
        SPI_GETWINARRANGING => return SpiGetSet_SPI_GETWINARRANGING(pvParam, fl),
        SPI_SETWINARRANGING => return SpiGetSet_SPI_SETWINARRANGING(uiParam, fl),
        SPI_GETDOCKMOVING => return SpiGetSet_SPI_GETDOCKMOVING(pvParam, fl),
        SPI_SETDOCKMOVING => return SpiGetSet_SPI_SETDOCKMOVING(uiParam, fl),
        0x1010 | 0x1011 | 0x1028 | 0x1029 | 0x102A | 0x102B | 0x102C | 0x102D | 0x102E | 0x102F
        | 0x1030 | 0x1031 | 0x1032 | 0x1033 | 0x1034 | 0x1035 | 0x1036 | 0x1037 | 0x1038 | 0x1039
        | 0x103A | 0x103B | 0x103C | 0x103D => {}
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    0
}

unsafe fn SpiGetSetProbeBuffer(uiAction: UINT, uiParam: UINT, pvParam: PVOID) -> BOOL {
    let mut bToUser: BOOL = 1;
    let mut cbSize: ULONG = 0;
    let Status: NTSTATUS;

    match uiAction {
        SPI_GETBEEP
        | SPI_GETBORDER
        | SPI_GETKEYBOARDSPEED
        | SPI_GETSCREENSAVETIMEOUT
        | SPI_GETSCREENSAVEACTIVE
        | SPI_GETGRIDGRANULARITY
        | SPI_GETKEYBOARDDELAY
        | SPI_GETICONTITLEWRAP
        | SPI_GETMENUDROPALIGNMENT
        | SPI_GETFASTTASKSWITCH
        | SPI_GETDRAGFULLWINDOWS
        | SPI_GETSHOWSOUNDS
        | SPI_GETKEYBOARDPREF
        | SPI_GETSCREENREADER
        | SPI_GETFONTSMOOTHING
        | SPI_GETLOWPOWERTIMEOUT
        | SPI_GETPOWEROFFTIMEOUT
        | SPI_GETLOWPOWERACTIVE
        | SPI_GETPOWEROFFACTIVE
        | SPI_GETMOUSETRAILS
        | SPI_GETSNAPTODEFBUTTON
        | SPI_GETMOUSEHOVERWIDTH
        | SPI_GETMOUSEHOVERHEIGHT
        | SPI_GETMOUSEHOVERTIME
        | SPI_GETWHEELSCROLLLINES
        | SPI_GETMENUSHOWDELAY
        | SPI_GETSHOWIMEUI
        | SPI_GETMOUSESPEED
        | SPI_GETSCREENSAVERRUNNING
        | SPI_GETACTIVEWINDOWTRACKING
        | SPI_GETMENUANIMATION
        | SPI_GETCOMBOBOXANIMATION
        | SPI_GETLISTBOXSMOOTHSCROLLING
        | SPI_GETGRADIENTCAPTIONS
        | SPI_GETKEYBOARDCUES
        | SPI_GETACTIVEWNDTRKZORDER
        | SPI_GETHOTTRACKING
        | SPI_GETMENUFADE
        | SPI_GETSELECTIONFADE
        | SPI_GETTOOLTIPANIMATION
        | SPI_GETTOOLTIPFADE
        | SPI_GETCURSORSHADOW
        | SPI_GETUIEFFECTS
        | SPI_GETMOUSESONAR
        | SPI_GETMOUSECLICKLOCK
        | SPI_GETMOUSEVANISH
        | SPI_GETFLATMENU
        | SPI_GETDROPSHADOW
        | SPI_GETBLOCKSENDINPUTRESETS
        | SPI_GETFOREGROUNDLOCKTIMEOUT
        | SPI_GETACTIVEWNDTRKTIMEOUT
        | SPI_GETFOREGROUNDFLASHCOUNT
        | SPI_GETCARETWIDTH
        | SPI_GETMOUSECLICKLOCKTIME
        | SPI_GETFONTSMOOTHINGTYPE
        | SPI_GETFONTSMOOTHINGCONTRAST
        | SPI_GETFOCUSBORDERWIDTH
        | SPI_GETFOCUSBORDERHEIGHT
        | SPI_GETFONTSMOOTHINGORIENTATION => {
            cbSize = core::mem::size_of::<INT>() as ULONG;
        }

        SPI_ICONHORIZONTALSPACING | SPI_ICONVERTICALSPACING => {
            if !pvParam.is_null() {
                cbSize = core::mem::size_of::<INT>() as ULONG;
            }
        }

        SPI_GETMOUSE => {
            cbSize = (3 * core::mem::size_of::<INT>()) as ULONG;
        }

        SPI_GETDESKWALLPAPER => {
            let a = uiParam as ULONG;
            let b = gspv.ustrWallpaper.Length as ULONG + 1u32;
            cbSize = if a < b { a } else { b };
        }

        SPI_GETICONTITLELOGFONT => {
            cbSize = core::mem::size_of::<LOGFONTW>() as ULONG;
        }

        SPI_GETNONCLIENTMETRICS => {
            cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as ULONG;
        }

        SPI_GETMINIMIZEDMETRICS => {
            cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as ULONG;
        }

        SPI_GETICONMETRICS => {
            cbSize = core::mem::size_of::<ICONMETRICSW>() as ULONG;
        }

        SPI_GETWORKAREA => {
            cbSize = core::mem::size_of::<RECTL>() as ULONG;
        }

        SPI_GETFILTERKEYS => {
            cbSize = core::mem::size_of::<FILTERKEYS>() as ULONG;
        }

        SPI_GETTOGGLEKEYS => {
            cbSize = core::mem::size_of::<TOGGLEKEYS>() as ULONG;
        }

        SPI_GETMOUSEKEYS => {
            cbSize = core::mem::size_of::<MOUSEKEYS>() as ULONG;
        }

        SPI_GETSTICKYKEYS => {
            cbSize = core::mem::size_of::<STICKYKEYS>() as ULONG;
        }

        SPI_GETACCESSTIMEOUT => {
            cbSize = core::mem::size_of::<ACCESSTIMEOUT>() as ULONG;
        }

        SPI_GETSERIALKEYS => {
            cbSize = core::mem::size_of::<SERIALKEYS>() as ULONG;
        }

        SPI_GETSOUNDSENTRY => {
            cbSize = core::mem::size_of::<SOUNDSENTRYW>() as ULONG;
        }

        SPI_GETHIGHCONTRAST => {
            cbSize = core::mem::size_of::<HIGHCONTRASTW>() as ULONG;
        }

        SPI_GETANIMATION => {
            cbSize = core::mem::size_of::<ANIMATIONINFO>() as ULONG;
        }

        SPI_GETDEFAULTINPUTLANG => {
            cbSize = core::mem::size_of::<HKL>() as ULONG;
        }

        SPI_SETMOUSE => {
            cbSize = (3 * core::mem::size_of::<INT>()) as ULONG;
            bToUser = 0;
        }

        SPI_SETICONTITLELOGFONT => {
            cbSize = core::mem::size_of::<LOGFONTW>() as ULONG;
            bToUser = 0;
        }

        SPI_SETNONCLIENTMETRICS => {
            cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as ULONG;
            bToUser = 0;
        }

        SPI_SETMINIMIZEDMETRICS => {
            cbSize = core::mem::size_of::<MINIMIZEDMETRICS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETICONMETRICS => {
            cbSize = core::mem::size_of::<ICONMETRICSW>() as ULONG;
            bToUser = 0;
        }

        SPI_SETWORKAREA => {
            cbSize = core::mem::size_of::<RECTL>() as ULONG;
            bToUser = 0;
        }

        SPI_SETFILTERKEYS => {
            cbSize = core::mem::size_of::<FILTERKEYS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETTOGGLEKEYS => {
            cbSize = core::mem::size_of::<TOGGLEKEYS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETMOUSEKEYS => {
            cbSize = core::mem::size_of::<MOUSEKEYS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETSTICKYKEYS => {
            cbSize = core::mem::size_of::<STICKYKEYS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETACCESSTIMEOUT => {
            cbSize = core::mem::size_of::<ACCESSTIMEOUT>() as ULONG;
            bToUser = 0;
        }

        SPI_SETSERIALKEYS => {
            cbSize = core::mem::size_of::<SERIALKEYS>() as ULONG;
            bToUser = 0;
        }

        SPI_SETSOUNDSENTRY => {
            cbSize = core::mem::size_of::<SOUNDSENTRYW>() as ULONG;
            bToUser = 0;
        }

        SPI_SETHIGHCONTRAST => {
            cbSize = core::mem::size_of::<HIGHCONTRASTW>() as ULONG;
            bToUser = 0;
        }

        SPI_SETANIMATION => {
            cbSize = core::mem::size_of::<ANIMATIONINFO>() as ULONG;
            bToUser = 0;
        }

        SPI_SETDEFAULTINPUTLANG => {
            cbSize = core::mem::size_of::<HKL>() as ULONG;
            bToUser = 0;
        }

        SPI_SETMOUSESPEED => {
            cbSize = core::mem::size_of::<INT>() as ULONG;
            bToUser = 0;
        }

        _ => {}
    }

    if cbSize != 0 {
        if bToUser != 0 {
            Status = SafeProbeForWrite(pvParam, cbSize as usize, core::mem::size_of::<UCHAR>() as u32);
        } else {
            Status = SafeProbeForRead(pvParam, cbSize as usize, core::mem::size_of::<UCHAR>() as u32);
        }
        if !NT_SUCCESS(Status) {
            return 0;
        }
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSystemParametersInfo(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    UserSystemParametersInfo_impl(uiAction, uiParam, pvParam, fWinIni)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSystemParametersInfo(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    UserSystemParametersInfo_impl(uiAction, uiParam, pvParam, fWinIni)
}

unsafe fn UserSystemParametersInfo_impl(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    fWinIni: UINT,
) -> BOOL {
    let mut ulResult: ULONG_PTR;
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    if gbSpiInitialized == 0 {
        KeRosDumpStackFrames(core::ptr::null_mut(), 20);
        return 0;
    }

    /* Get a pointer to the current Windowstation */
    if (*ppi).prpwinsta.is_null() {
        ERR!("UserSystemParametersInfo called without active window station.\n");
    }

    if (fWinIni & SPIF_PROTECT) != 0
        && SpiGetSetProbeBuffer(uiAction, uiParam, pvParam) == 0
    {
        EngSetLastError(ERROR_NOACCESS);
        return 0;
    }

    /* Do the actual operation */
    ulResult = SpiGetSet(uiAction, uiParam, pvParam, fWinIni);

    /* Did we change something? */
    if ulResult > 1 {
        SpiFixupValues();

        /* Update system metrics */
        InitMetrics();

        /* Send notification to toplevel windows, if requested */
        if (fWinIni & SPIF_SENDCHANGE) != 0 {
            co_IntSendMessageTimeout(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                uiAction as WPARAM,
                ulResult as LPARAM,
                SMTO_NORMAL,
                100,
                core::ptr::addr_of_mut!(ulResult),
            );
        }
        ulResult = 1;
    }

    ulResult as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSystemParametersInfo(
    uiAction: UINT,
    uiParam: UINT,
    pvParam: PVOID,
    mut fWinIni: UINT,
) -> BOOL {
    let bResult: BOOL;

    UserEnterExclusive();

    fWinIni |= SPIF_PROTECT;

    bResult = UserSystemParametersInfo(uiAction, uiParam, pvParam, fWinIni);

    UserLeave();

    bResult
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn RegWriteUserSetting(pwszKeyName: PCWSTR, pwszValueName: PCWSTR, ulType: u32, pvData: *const core::ffi::c_void, cbDataSize: u32,) -> i32;
    fn W32kDosPathNameToNtPathName(pwszPath: PCWSTR, pustrNt: *mut _UNICODE_STRING) -> BOOL;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
