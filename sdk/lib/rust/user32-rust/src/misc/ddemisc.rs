//! Translated from win32ss/user/user32/misc/ddemisc.c — DDEML / WDML core.
//!
//! Cohabitation module for the user32-rust staticlib. Defines the shared WDML_*
//! helpers + the WDML_szEventClass class-name global consumed by ddeclient.c /
//! ddeserver.c, plus the public Dde* / *DDElParam USER32 APIs.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// DestroyWindow is an ntwrapper inline (NtUser* multiplexer), not a link symbol.
use crate::ntwrapper::DestroyWindow;

// ================================================================
//  DDE / DDEML types (ddeml.h, dde.h) — not present in win32k_sys bindgen.
// ================================================================
pub type HCONVLIST = HANDLE;
pub type HCONV = HANDLE;
pub type HSZ = HANDLE;
pub type HDDEDATA = HANDLE;

// HDDEDATA (CALLBACK *PFNCALLBACK)(UINT,UINT,HCONV,HSZ,HSZ,HDDEDATA,ULONG_PTR,ULONG_PTR)
pub type PFNCALLBACK = Option<
    unsafe extern "system" fn(
        UINT,
        UINT,
        HCONV,
        HSZ,
        HSZ,
        HDDEDATA,
        ULONG_PTR,
        ULONG_PTR,
    ) -> HDDEDATA,
>;
pub type LPDWORD = *mut DWORD;

// DDEACK (dde.h): unsigned short bAppReturnCode:8, reserved:6, fBusy:1, fAck:1;
// Reproduced as a 16-bit bitfield-backed struct so `*(WORD*)&ddeAck` round-trips.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct DDEACK {
    pub bits: u16,
}
impl DDEACK {
    #[inline]
    fn set_bAppReturnCode(&mut self, v: u16) {
        self.bits = (self.bits & !0x00FF) | (v & 0x00FF);
    }
    #[inline]
    fn set_reserved(&mut self, v: u16) {
        self.bits = (self.bits & !0x3F00) | ((v & 0x3F) << 8);
    }
    #[inline]
    fn set_fBusy(&mut self, v: u16) {
        self.bits = (self.bits & !0x4000) | ((v & 1) << 14);
    }
    #[inline]
    fn set_fAck(&mut self, v: u16) {
        self.bits = (self.bits & !0x8000) | ((v & 1) << 15);
    }
    #[inline]
    fn fAck(&self) -> u16 {
        (self.bits >> 15) & 1
    }
}

// CONVCONTEXT (ddeml.h)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CONVCONTEXT {
    pub cb: UINT,
    pub wFlags: UINT,
    pub wCountryID: UINT,
    pub iCodePage: INT,
    pub dwLangID: DWORD,
    pub dwSecurity: DWORD,
}

// CONVINFO (ddeml.h)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CONVINFO {
    pub cb: DWORD,
    pub hUser: DWORD,
    pub hConvPartner: HCONV,
    pub hszSvcPartner: HSZ,
    pub hszServiceReq: HSZ,
    pub hszTopic: HSZ,
    pub hszItem: HSZ,
    pub wFmt: UINT,
    pub wType: UINT,
    pub wStatus: UINT,
    pub wConvst: UINT,
    pub wLastError: UINT,
    pub hConvList: HCONVLIST,
    pub ConvCtxt: CONVCONTEXT,
    pub hwnd: HWND,
    pub hwndPartner: HWND,
}
pub type PCONVINFO = *mut CONVINFO;

// DDEDATA (dde.h): unsigned short unused:12, fResponse:1, fRelease:1, reserved:1, fAckReq:1;
//                  short cfFormat; BYTE Value[1];
#[repr(C)]
pub struct DDEDATA {
    pub flags: u16,
    pub cfFormat: i16,
    pub Value: [BYTE; 1],
}

// WINE_DDEHEAD (dde_private.h): same first 4 bytes layout as DDE??? structures.
//   unsigned short unused:12, fResponse:1, fRelease:1, fDeferUpd:1, fAckReq:1;
//   short cfFormat;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct WINE_DDEHEAD {
    pub flags: u16,
    pub cfFormat: i16,
}
impl WINE_DDEHEAD {
    #[inline]
    fn set_unused(&mut self, v: u16) {
        self.flags = (self.flags & !0x0FFF) | (v & 0x0FFF);
    }
    #[inline]
    fn set_fResponse(&mut self, v: u16) {
        self.flags = (self.flags & !0x1000) | ((v & 1) << 12);
    }
    #[inline]
    fn set_fRelease(&mut self, v: u16) {
        self.flags = (self.flags & !0x2000) | ((v & 1) << 13);
    }
    #[inline]
    fn set_fDeferUpd(&mut self, v: u16) {
        self.flags = (self.flags & !0x4000) | ((v & 1) << 14);
    }
    #[inline]
    fn set_fAckReq(&mut self, v: u16) {
        self.flags = (self.flags & !0x8000) | ((v & 1) << 15);
    }
}

// ── DDE / DDEML constants (ddeml.h, dde.h) ──
const CP_WINANSI: INT = 1004;
const CP_WINUNICODE: INT = 1200;
const QID_SYNC: DWORD = 0xFFFFFFFF;

const XST_NULL: DWORD = 0;

const ST_CONNECTED: DWORD = 0x0001;
const ST_ADVISE: DWORD = 0x0002;
const ST_ISLOCAL: DWORD = 0x0004;
const ST_BLOCKED: DWORD = 0x0008;
const ST_CLIENT: DWORD = 0x0010;
const ST_BLOCKNEXT: DWORD = 0x0080;
const ST_ISSELF: DWORD = 0x0100;

const EC_ENABLEALL: UINT = 0;
const EC_ENABLEONE: UINT = ST_BLOCKNEXT as UINT;
const EC_DISABLE: UINT = ST_BLOCKED as UINT;
const EC_QUERYWAITING: UINT = 2;

const CBF_FAIL_ALLSVRXACTIONS: DWORD = 0x0003f000;
const CBF_SKIP_CONNECT_CONFIRMS: DWORD = 0x00040000;
const CBF_SKIP_REGISTRATIONS: DWORD = 0x00080000;
const CBF_SKIP_UNREGISTRATIONS: DWORD = 0x00100000;

const APPCMD_CLIENTONLY: DWORD = 0x00000010;
const APPCMD_FILTERINITS: DWORD = 0x00000020;
const APPCMD_MASK: DWORD = 0x00000FF0;
const APPCLASS_MONITOR: DWORD = 0x00000001;
const APPCLASS_MASK: DWORD = 0x0000000F;
const MF_MASK: DWORD = 0xFF000000;

const DNS_UNREGISTER: UINT = 0x0002;

// XTYP_* (composed from XCLASS_* | XTYPF_* in ddeml.h)
const XTYPF_NOBLOCK: UINT = 0x0002;
const XCLASS_NOTIFICATION: UINT = 0x8000;
const XTYP_CONNECT_CONFIRM: UINT = 0x0070 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;
const XTYP_REGISTER: UINT = 0x00A0 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;
const XTYP_UNREGISTER: UINT = 0x00D0 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;

const DMLERR_NO_ERROR: UINT = 0;
const DMLERR_DLL_USAGE: UINT = 0x4004;
const DMLERR_INVALIDPARAMETER: UINT = 0x4006;
const DMLERR_MEMORY_ERROR: UINT = 0x4008;
const DMLERR_NO_CONV_ESTABLISHED: UINT = 0x400a;
const DMLERR_POSTMSG_FAILED: UINT = 0x400c;
const DMLERR_SYS_ERROR: UINT = 0x400f;
const DMLERR_UNFOUND_QUEUE_ID: UINT = 0x4011;

const HDATA_APPOWNED: UINT = 0x0001;

// dde.h window messages
const WM_DDE_ACK: UINT = 0x3E4;
const WM_DDE_ADVISE: UINT = 0x3E2;
const WM_DDE_DATA: UINT = 0x3E5;
const WM_DDE_POKE: UINT = 0x3E7;
const WM_DDE_EXECUTE: UINT = 0x3E8;
const WM_DDE_TERMINATE: UINT = 0x3E1;

// winuser.h clipboard formats / window styles
const CF_TEXT: WORD = 1;
const CF_BITMAP: WORD = 2;
const WS_POPUP: DWORD = 0x80000000;

// kernel32 global memory flags
const GMEM_MOVEABLE: UINT = 0x0002;
const GMEM_DDESHARE: UINT = 0x2000;

// window long offsets (dde_private.h)
const GWL_WDML_INSTANCE: INT = 0;
const GWL_WDML_CONVERSATION: INT = core::mem::size_of::<ULONG_PTR>() as INT;

// WM_USER + offsets (dde_private.h)
const WM_USER: UINT = 0x0400;
const WM_WDML_REGISTER: UINT = WM_USER + 0x200;
const WM_WDML_UNREGISTER: UINT = WM_USER + 0x201;
const WM_WDML_CONNECT_CONFIRM: UINT = WM_USER + 0x202;

// dde_private.h
const MAX_ATOM_LEN: usize = 255;
const MAX_BUFFER_LEN: usize = MAX_ATOM_LEN + 1;
const WDML_CONV_MAGIC: UINT = 0xbabe1234;

const WDML_CLIENT_SIDE: usize = 0;
const WDML_SERVER_SIDE: usize = 1;

// WDML_QUEUE_STATE (dde_private.h) — used by externs declared below.
pub type WDML_QUEUE_STATE = INT;

// ================================================================
//  WDML internal structures (dde_private.h) — faithful #[repr(C)].
// ================================================================
#[repr(C)]
pub struct HSZNode {
    pub next: *mut HSZNode,
    pub hsz: HSZ,
    pub refCount: core::ffi::c_uint,
}

#[repr(C)]
pub struct WDML_SERVER {
    pub next: *mut WDML_SERVER,
    pub hszService: HSZ,
    pub hszServiceSpec: HSZ,
    pub atomService: ATOM,
    pub atomServiceSpec: ATOM,
    pub filterOn: BOOL,
    pub hwndServer: HWND,
}

#[repr(C)]
pub struct WDML_XACT {
    pub next: *mut WDML_XACT,
    pub xActID: DWORD,
    pub ddeMsg: UINT,
    pub hDdeData: HDDEDATA,
    pub dwTimeout: DWORD,
    pub hUser: DWORD,
    pub wType: UINT,
    pub wFmt: UINT,
    pub hszItem: HSZ,
    pub atom: ATOM,
    pub hMem: HGLOBAL,
    pub lParam: LPARAM,
}

#[repr(C)]
pub struct WDML_CONV {
    pub next: *mut WDML_CONV,
    pub instance: *mut WDML_INSTANCE,
    pub hszService: HSZ,
    pub hszTopic: HSZ,
    pub magic: UINT,
    pub afCmd: UINT,
    pub convContext: CONVCONTEXT,
    pub hwndClient: HWND,
    pub hwndServer: HWND,
    pub transactions: *mut WDML_XACT,
    pub hUser: DWORD,
    pub wStatus: DWORD,
    pub wConvst: DWORD,
}

#[repr(C)]
pub struct WDML_LINK {
    pub next: *mut WDML_LINK,
    pub hConv: HCONV,
    pub transactionType: UINT,
    pub hszItem: HSZ,
    pub uFmt: UINT,
}

#[repr(C)]
pub struct WDML_INSTANCE {
    pub next: *mut WDML_INSTANCE,
    pub instanceID: DWORD,
    pub threadID: DWORD,
    pub monitor: BOOL,
    pub clientOnly: BOOL,
    pub unicode: BOOL,
    pub nodeList: *mut HSZNode,
    pub callback: PFNCALLBACK,
    pub CBFflags: DWORD,
    pub monitorFlags: DWORD,
    pub lastError: DWORD,
    pub hwndEvent: HWND,
    pub wStatus: DWORD,
    pub servers: *mut WDML_SERVER,
    pub convs: [*mut WDML_CONV; 2],
    pub links: [*mut WDML_LINK; 2],
}

// header for the DDE Data objects (dde_private.h)
#[repr(C)]
pub struct DDE_DATAHANDLE_HEAD {
    pub cfFormat: WORD,
    pub bAppOwned: WORD,
}

// ================================================================
//  Externs — kernel32 GlobalAlloc/atoms, gdi32, and cross-file DDE symbols.
// ================================================================
extern "system" {
    // kernel32 global memory / atoms
    fn GlobalAlloc(uFlags: UINT, dwBytes: SIZE_T) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GlobalReAlloc(hMem: HGLOBAL, dwBytes: SIZE_T, uFlags: UINT) -> HGLOBAL;
    fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
    fn GlobalHandle(pMem: LPCVOID) -> HGLOBAL;
    fn GlobalAddAtomW(lpString: LPCWSTR) -> ATOM;
    fn GlobalGetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: INT) -> UINT;
    fn GlobalDeleteAtom(nAtom: ATOM) -> ATOM;
    fn AddAtomA(lpString: LPCSTR) -> ATOM;
    fn AddAtomW(lpString: LPCWSTR) -> ATOM;
    fn DeleteAtom(nAtom: ATOM) -> ATOM;
    fn GetAtomNameA(nAtom: ATOM, lpBuffer: LPSTR, nSize: INT) -> UINT;
    fn GetAtomNameW(nAtom: ATOM, lpBuffer: LPWSTR, nSize: INT) -> UINT;
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetCurrentThreadId() -> DWORD;
    fn GetCurrentProcessId() -> DWORD;
    fn EnterCriticalSection(lpCriticalSection: *mut c_void);
}

// InterlockedIncrement: kernel32 import on i386, compiler intrinsic on amd64/arm64
// (no link symbol) → implement via Rust atomics for a uniform cohab link.
#[inline]
unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}

extern "system" {
    fn LeaveCriticalSection(lpCriticalSection: *mut c_void);
    fn InitializeCriticalSection(lpCriticalSection: *mut c_void);
    fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT;

    // user32 window APIs (cross-DLL link symbols, stdcall on i386)
    fn RegisterClassExW(lpwcx: *const _WNDCLASSEXW) -> ATOM;
    fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        X: INT,
        Y: INT,
        nWidth: INT,
        nHeight: INT,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
    fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: LPDWORD) -> DWORD;

    // gdi32
    fn CreateBitmap(
        nWidth: INT,
        nHeight: INT,
        nPlanes: UINT,
        nBitCount: UINT,
        lpBits: *const c_void,
    ) -> HBITMAP;
    fn GetObjectW(h: HANDLE, c: INT, pv: LPVOID) -> INT;
    fn GetBitmapBits(hbit: HBITMAP, cb: LONG, lpvBits: LPVOID) -> LONG;

    // cross-file DDE entry points (defined in ddeclient.c / ddeserver.c)
    fn DdeDisconnect(hConv: HCONV) -> BOOL;
    fn DdeNameService(idInst: DWORD, hsz1: HSZ, hsz2: HSZ, afCmd: UINT) -> HDDEDATA;
}

// plain-C cross-file WDML_* helpers (defined in ddeclient.c / ddeserver.c).
extern "C" {
    fn WDML_ServerHandle(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE;
    fn WDML_ClientHandle(
        pConv: *mut WDML_CONV,
        pXAct: *mut WDML_XACT,
        dwTimeout: DWORD,
        pdwResult: LPDWORD,
    ) -> HDDEDATA;
    // user_x.h: WIN_GetFullHandle (defined extern "C" in controls/combo.rs)
    fn WIN_GetFullHandle(h: HWND) -> HWND;
}

pub type WNDENUMPROC =
    Option<unsafe extern "system" fn(arg1: HWND, arg2: LPARAM) -> BOOL>;

// ================================================================
//  File-scope statics.
// ================================================================
static mut WDML_InstanceList: *mut WDML_INSTANCE = core::ptr::null_mut();
static mut WDML_MaxInstanceID: LONG = 0;

// const WCHAR WDML_szEventClass[] = L"DDEMLEvent"; (non-static — exported)
#[no_mangle]
pub static WDML_szEventClass: [u16; 11] = [
    0x44, 0x44, 0x45, 0x4D, 0x4C, 0x45, 0x76, 0x65, 0x6E, 0x74, 0x00,
]; // "DDEMLEvent\0"

// CRITICAL_SECTION protecting the instance list. The C file uses a static
// initializer with a critsect_debug; we lazily InitializeCriticalSection it once.
#[repr(C)]
struct CRITICAL_SECTION_STORE {
    raw: [u8; 64],
}
static mut WDML_CritSect: CRITICAL_SECTION_STORE = CRITICAL_SECTION_STORE { raw: [0; 64] };
static mut WDML_CritSect_Init: BOOL = 0;

#[inline]
unsafe fn WDML_EnsureCritSect() {
    if WDML_CritSect_Init == 0 {
        InitializeCriticalSection(core::ptr::addr_of_mut!(WDML_CritSect) as *mut c_void);
        WDML_CritSect_Init = 1;
    }
}
#[inline]
unsafe fn WDML_Enter() {
    WDML_EnsureCritSect();
    EnterCriticalSection(core::ptr::addr_of_mut!(WDML_CritSect) as *mut c_void);
}
#[inline]
unsafe fn WDML_Leave() {
    LeaveCriticalSection(core::ptr::addr_of_mut!(WDML_CritSect) as *mut c_void);
}

// helpers mirroring the ATOM2HSZ / HSZ2ATOM macros
#[inline]
fn ATOM2HSZ(atom: ATOM) -> HSZ {
    atom as ULONG_PTR as HSZ
}
#[inline]
fn HSZ2ATOM(hsz: HSZ) -> ATOM {
    hsz as ULONG_PTR as ATOM
}

#[inline]
fn LOWORD(l: ULONG_PTR) -> UINT {
    (l & 0xffff) as UINT
}
#[inline]
fn HIWORD(l: ULONG_PTR) -> UINT {
    ((l >> 16) & 0xffff) as UINT
}
#[inline]
fn MAKELONG(lo: UINT_PTR, hi: UINT_PTR) -> LPARAM {
    (((lo as u32) & 0xffff) | (((hi as u32) & 0xffff) << 16)) as i32 as LPARAM
}

unsafe fn wcslen_local(s: *const u16) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

// ================================================================
//  Pure DDE (non DDEML) management
// ================================================================

/// PackDDElParam (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn PackDDElParam(msg: UINT, uiLo: UINT_PTR, uiHi: UINT_PTR) -> LPARAM {
    match msg {
        WM_DDE_ACK | WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
            let hMem = GlobalAlloc(GMEM_DDESHARE, (core::mem::size_of::<UINT_PTR>() * 2) as SIZE_T);
            if hMem.is_null() {
                return 0;
            }
            let params = GlobalLock(hMem) as *mut UINT_PTR;
            if params.is_null() {
                return 0;
            }
            *params.offset(0) = uiLo;
            *params.offset(1) = uiHi;
            GlobalUnlock(hMem);
            hMem as LPARAM
        }
        WM_DDE_EXECUTE => uiHi as LPARAM,
        _ => MAKELONG(uiLo, uiHi),
    }
}

/// UnpackDDElParam (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn UnpackDDElParam(
    msg: UINT,
    lParam: LPARAM,
    uiLo: *mut UINT_PTR,
    uiHi: *mut UINT_PTR,
) -> BOOL {
    match msg {
        WM_DDE_ACK | WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
            let params = if lParam == 0 {
                core::ptr::null_mut()
            } else {
                GlobalLock(lParam as HGLOBAL) as *mut UINT_PTR
            };
            if lParam == 0 || params.is_null() {
                if !uiLo.is_null() {
                    *uiLo = 0;
                }
                if !uiHi.is_null() {
                    *uiHi = 0;
                }
                return FALSE as BOOL;
            }
            if !uiLo.is_null() {
                *uiLo = *params.offset(0);
            }
            if !uiHi.is_null() {
                *uiHi = *params.offset(1);
            }
            GlobalUnlock(lParam as HGLOBAL);
            TRUE as BOOL
        }
        WM_DDE_EXECUTE => {
            if !uiLo.is_null() {
                *uiLo = 0;
            }
            if !uiHi.is_null() {
                *uiHi = lParam as UINT_PTR;
            }
            TRUE as BOOL
        }
        _ => {
            if !uiLo.is_null() {
                *uiLo = LOWORD(lParam as ULONG_PTR) as UINT_PTR;
            }
            if !uiHi.is_null() {
                *uiHi = HIWORD(lParam as ULONG_PTR) as UINT_PTR;
            }
            TRUE as BOOL
        }
    }
}

/// FreeDDElParam (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn FreeDDElParam(msg: UINT, lParam: LPARAM) -> BOOL {
    match msg {
        WM_DDE_ACK | WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
            if GlobalHandle(lParam as LPCVOID).is_null() {
                return TRUE as BOOL;
            }
            (GlobalFree(lParam as HGLOBAL).is_null()) as BOOL
        }
        _ => TRUE as BOOL,
    }
}

/// ReuseDDElParam (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ReuseDDElParam(
    lParam: LPARAM,
    msgIn: UINT,
    msgOut: UINT,
    uiLo: UINT_PTR,
    uiHi: UINT_PTR,
) -> LPARAM {
    match msgIn {
        WM_DDE_ACK | WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => match msgOut {
            WM_DDE_ACK | WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
                if lParam == 0 {
                    return 0;
                }
                let params = GlobalLock(lParam as HGLOBAL) as *mut UINT_PTR;
                if params.is_null() {
                    return 0;
                }
                *params.offset(0) = uiLo;
                *params.offset(1) = uiHi;
                GlobalUnlock(lParam as HGLOBAL);
                lParam
            }
            WM_DDE_EXECUTE => {
                FreeDDElParam(msgIn, lParam);
                uiHi as LPARAM
            }
            _ => {
                FreeDDElParam(msgIn, lParam);
                MAKELONG(uiLo, uiHi)
            }
        },
        _ => PackDDElParam(msgOut, uiLo, uiHi),
    }
}

/// ImpersonateDdeClientWindow (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn ImpersonateDdeClientWindow(
    _hWndClient: HWND,
    _hWndServer: HWND,
) -> BOOL {
    FALSE as BOOL
}

/// DdeSetQualityOfService (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeSetQualityOfService(
    _hwndClient: HWND,
    _pqosNew: *const c_void,
    _pqosPrev: *mut c_void,
) -> BOOL {
    TRUE as BOOL
}

// ================================================================
//  WDML Error management
// ================================================================

/// DdeGetLastError (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeGetLastError(idInst: DWORD) -> UINT {
    let error_code: DWORD;
    let pInstance = WDML_GetInstance(idInst);
    if pInstance.is_null() {
        error_code = DMLERR_INVALIDPARAMETER;
    } else {
        error_code = (*pInstance).lastError;
        (*pInstance).lastError = 0;
    }
    error_code
}

unsafe fn WDML_SetAllLastError(lastError: DWORD) {
    let threadID = GetCurrentThreadId();
    let mut pInstance = WDML_InstanceList;
    while !pInstance.is_null() {
        if (*pInstance).threadID == threadID {
            (*pInstance).lastError = lastError;
        }
        pInstance = (*pInstance).next;
    }
}

// ================================================================
//  String management
// ================================================================

unsafe fn WDML_FindNode(pInstance: *mut WDML_INSTANCE, hsz: HSZ) -> *mut HSZNode {
    if pInstance.is_null() {
        return core::ptr::null_mut();
    }
    let mut pNode = (*pInstance).nodeList;
    while !pNode.is_null() {
        if (*pNode).hsz == hsz {
            break;
        }
        pNode = (*pNode).next;
    }
    pNode
}

/// WDML_MakeAtomFromHsz
#[no_mangle]
pub unsafe extern "C" fn WDML_MakeAtomFromHsz(hsz: HSZ) -> ATOM {
    let mut nameBuffer: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    if GetAtomNameW(HSZ2ATOM(hsz), nameBuffer.as_mut_ptr(), MAX_BUFFER_LEN as INT) != 0 {
        return GlobalAddAtomW(nameBuffer.as_ptr());
    }
    0
}

/// WDML_MakeHszFromAtom
#[no_mangle]
pub unsafe extern "C" fn WDML_MakeHszFromAtom(pInstance: *const WDML_INSTANCE, atom: ATOM) -> HSZ {
    let mut nameBuffer: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    if atom == 0 {
        return core::ptr::null_mut();
    }
    if GlobalGetAtomNameW(atom, nameBuffer.as_mut_ptr(), MAX_BUFFER_LEN as INT) != 0 {
        return DdeCreateStringHandleW(
            (*pInstance).instanceID,
            nameBuffer.as_ptr(),
            CP_WINUNICODE,
        );
    }
    core::ptr::null_mut()
}

/// WDML_IncHSZ
#[no_mangle]
pub unsafe extern "C" fn WDML_IncHSZ(pInstance: *mut WDML_INSTANCE, hsz: HSZ) -> BOOL {
    let pNode = WDML_FindNode(pInstance, hsz);
    if pNode.is_null() {
        return FALSE as BOOL;
    }
    (*pNode).refCount += 1;
    TRUE as BOOL
}

/// WDML_DecHSZ
#[no_mangle]
pub unsafe extern "C" fn WDML_DecHSZ(pInstance: *mut WDML_INSTANCE, hsz: HSZ) -> BOOL {
    let mut pPrev: *mut HSZNode = core::ptr::null_mut();
    let mut pCurrent = (*pInstance).nodeList;
    while !pCurrent.is_null() {
        if (*pCurrent).hsz == hsz {
            (*pCurrent).refCount -= 1;
            if (*pCurrent).refCount == 0 {
                if pCurrent == (*pInstance).nodeList {
                    (*pInstance).nodeList = (*pCurrent).next;
                } else {
                    (*pPrev).next = (*pCurrent).next;
                }
                HeapFree(GetProcessHeap(), 0, pCurrent as LPVOID);
                DeleteAtom(HSZ2ATOM(hsz));
            }
            return TRUE as BOOL;
        }
        pPrev = pCurrent;
        pCurrent = (*pCurrent).next;
    }
    FALSE as BOOL
}

unsafe fn WDML_FreeAllHSZ(pInstance: *mut WDML_INSTANCE) {
    while !(*pInstance).nodeList.is_null() {
        DdeFreeStringHandle((*pInstance).instanceID, (*(*pInstance).nodeList).hsz);
    }
}

unsafe fn WDML_InsertHSZNode(pInstance: *mut WDML_INSTANCE, hsz: HSZ) {
    if !hsz.is_null() {
        let pNew = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<HSZNode>() as SIZE_T)
            as *mut HSZNode;
        if !pNew.is_null() {
            (*pNew).hsz = hsz;
            (*pNew).next = (*pInstance).nodeList;
            (*pNew).refCount = 1;
            (*pInstance).nodeList = pNew;
        }
    }
}

unsafe fn WDML_QueryString(
    _pInstance: *mut WDML_INSTANCE,
    hsz: HSZ,
    mut ptr: LPVOID,
    mut cchMax: DWORD,
    codepage: INT,
) -> INT {
    let mut pString: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    // If ptr is null, we have to return only the length of the string.
    if ptr.is_null() {
        ptr = pString.as_mut_ptr() as LPVOID;
        cchMax = MAX_BUFFER_LEN as DWORD;
    }

    // if there is no input windows returns a NULL string
    if hsz.is_null() {
        let t_ptr = ptr as *mut CHAR;
        *t_ptr = 0;
        return 1;
    }

    match codepage {
        CP_WINANSI => GetAtomNameA(HSZ2ATOM(hsz), ptr as LPSTR, cchMax as INT) as INT,
        CP_WINUNICODE => GetAtomNameW(HSZ2ATOM(hsz), ptr as LPWSTR, cchMax as INT) as INT,
        _ => 0,
    }
}

/// DdeQueryStringA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeQueryStringA(
    idInst: DWORD,
    hsz: HSZ,
    psz: LPSTR,
    cchMax: DWORD,
    mut iCodePage: INT,
) -> DWORD {
    let mut ret: DWORD = 0;
    let pInstance = WDML_GetInstance(idInst);
    if !pInstance.is_null() {
        if iCodePage == 0 {
            iCodePage = CP_WINANSI;
        }
        ret = WDML_QueryString(pInstance, hsz, psz as LPVOID, cchMax, iCodePage) as DWORD;
    }
    ret
}

/// DdeQueryStringW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeQueryStringW(
    idInst: DWORD,
    hsz: HSZ,
    psz: LPWSTR,
    cchMax: DWORD,
    mut iCodePage: INT,
) -> DWORD {
    let mut ret: DWORD = 0;
    let pInstance = WDML_GetInstance(idInst);
    if !pInstance.is_null() {
        if iCodePage == 0 {
            iCodePage = CP_WINUNICODE;
        }
        ret = WDML_QueryString(pInstance, hsz, psz as LPVOID, cchMax, iCodePage) as DWORD;
    }
    ret
}

unsafe fn WDML_CreateString(pInstance: *mut WDML_INSTANCE, ptr: LPCVOID, codepage: INT) -> HSZ {
    let hsz: HSZ = match codepage {
        CP_WINANSI => ATOM2HSZ(AddAtomA(ptr as LPCSTR)),
        CP_WINUNICODE => ATOM2HSZ(AddAtomW(ptr as LPCWSTR)),
        _ => return core::ptr::null_mut(),
    };
    WDML_InsertHSZNode(pInstance, hsz);
    hsz
}

/// DdeCreateStringHandleA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeCreateStringHandleA(
    idInst: DWORD,
    psz: LPCSTR,
    mut codepage: INT,
) -> HSZ {
    let mut hsz: HSZ = core::ptr::null_mut();
    let pInstance = WDML_GetInstance(idInst);
    if pInstance.is_null() {
        WDML_SetAllLastError(DMLERR_INVALIDPARAMETER);
    } else {
        if codepage == 0 {
            codepage = CP_WINANSI;
        }
        hsz = WDML_CreateString(pInstance, psz as LPCVOID, codepage);
    }
    hsz
}

/// DdeCreateStringHandleW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeCreateStringHandleW(
    idInst: DWORD,
    psz: LPCWSTR,
    mut codepage: INT,
) -> HSZ {
    let mut hsz: HSZ = core::ptr::null_mut();
    let pInstance = WDML_GetInstance(idInst);
    if pInstance.is_null() {
        WDML_SetAllLastError(DMLERR_INVALIDPARAMETER);
    } else {
        if codepage == 0 {
            codepage = CP_WINUNICODE;
        }
        hsz = WDML_CreateString(pInstance, psz as LPCVOID, codepage);
    }
    hsz
}

/// DdeFreeStringHandle (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeFreeStringHandle(idInst: DWORD, hsz: HSZ) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let pInstance = WDML_GetInstance(idInst);
    if !pInstance.is_null() {
        ret = WDML_DecHSZ(pInstance, hsz);
    }
    ret
}

/// DdeKeepStringHandle (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeKeepStringHandle(idInst: DWORD, hsz: HSZ) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let pInstance = WDML_GetInstance(idInst);
    if !pInstance.is_null() {
        ret = WDML_IncHSZ(pInstance, hsz);
    }
    ret
}

/// DdeCmpStringHandles (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeCmpStringHandles(hsz1: HSZ, hsz2: HSZ) -> INT {
    let mut psz1: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    let mut psz2: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    let mut ret: INT;

    let ret1 = GetAtomNameW(HSZ2ATOM(hsz1), psz1.as_mut_ptr(), MAX_BUFFER_LEN as INT);
    let ret2 = GetAtomNameW(HSZ2ATOM(hsz2), psz2.as_mut_ptr(), MAX_BUFFER_LEN as INT);

    if ret1 == 0 && ret2 == 0 {
        ret = 0;
    } else if ret1 == 0 {
        ret = -1;
    } else if ret2 == 0 {
        ret = 1;
    } else {
        ret = lstrcmpiW(psz1.as_ptr(), psz2.as_ptr());
        if ret < 0 {
            ret = -1;
        } else if ret > 0 {
            ret = 1;
        }
    }
    ret
}

// ================================================================
//  Instance management
// ================================================================

unsafe fn WDML_IncrementInstanceId(pInstance: *mut WDML_INSTANCE) {
    let id = InterlockedIncrement(core::ptr::addr_of_mut!(WDML_MaxInstanceID)) as DWORD;
    (*pInstance).instanceID = id;
}

unsafe extern "system" fn WDML_EventProc(
    hwndEvent: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pInstance: *mut WDML_INSTANCE;
    let hsz1: HSZ;
    let hsz2: HSZ;

    match uMsg {
        WM_WDML_REGISTER => {
            pInstance = WDML_GetInstanceFromWnd(hwndEvent);
            if !pInstance.is_null() && ((*pInstance).CBFflags & CBF_SKIP_REGISTRATIONS) == 0 {
                hsz1 = WDML_MakeHszFromAtom(pInstance, wParam as ATOM);
                hsz2 = WDML_MakeHszFromAtom(pInstance, lParam as ATOM);
                WDML_InvokeCallback(
                    pInstance,
                    XTYP_REGISTER,
                    0,
                    core::ptr::null_mut(),
                    hsz1,
                    hsz2,
                    core::ptr::null_mut(),
                    0,
                    0,
                );
                WDML_DecHSZ(pInstance, hsz1);
                WDML_DecHSZ(pInstance, hsz2);
            }
        }
        WM_WDML_UNREGISTER => {
            pInstance = WDML_GetInstanceFromWnd(hwndEvent);
            if !pInstance.is_null() && ((*pInstance).CBFflags & CBF_SKIP_UNREGISTRATIONS) == 0 {
                hsz1 = WDML_MakeHszFromAtom(pInstance, wParam as ATOM);
                hsz2 = WDML_MakeHszFromAtom(pInstance, lParam as ATOM);
                WDML_InvokeCallback(
                    pInstance,
                    XTYP_UNREGISTER,
                    0,
                    core::ptr::null_mut(),
                    hsz1,
                    hsz2,
                    core::ptr::null_mut(),
                    0,
                    0,
                );
                WDML_DecHSZ(pInstance, hsz1);
                WDML_DecHSZ(pInstance, hsz2);
            }
        }
        WM_WDML_CONNECT_CONFIRM => {
            pInstance = WDML_GetInstanceFromWnd(hwndEvent);
            if !pInstance.is_null() && ((*pInstance).CBFflags & CBF_SKIP_CONNECT_CONFIRMS) == 0 {
                let client = WIN_GetFullHandle(wParam as HWND);
                let server = WIN_GetFullHandle(lParam as HWND);
                let mut pConv = (*pInstance).convs[WDML_SERVER_SIDE];
                while !pConv.is_null() {
                    if (*pConv).hwndClient == client && (*pConv).hwndServer == server {
                        break;
                    }
                    pConv = (*pConv).next;
                }
                if !pConv.is_null() {
                    (*pConv).wStatus |= ST_ISLOCAL;
                    WDML_InvokeCallback(
                        pInstance,
                        XTYP_CONNECT_CONFIRM,
                        0,
                        pConv as HCONV,
                        (*pConv).hszTopic,
                        (*pConv).hszService,
                        core::ptr::null_mut(),
                        0,
                        if ((*pConv).wStatus & ST_ISSELF) != 0 { 1 } else { 0 },
                    );
                }
            }
        }
        _ => {
            return DefWindowProcW(hwndEvent, uMsg, wParam, lParam);
        }
    }
    0
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn WDML_Initialize(
    pidInst: LPDWORD,
    pfnCallback: PFNCALLBACK,
    afCmd: DWORD,
    ulRes: DWORD,
    bUnicode: BOOL,
) -> UINT {
    let ret: UINT;
    let mut reference_inst: *mut WDML_INSTANCE;

    if ulRes != 0 {
        // trap this and no more until we know more
        return DMLERR_NO_ERROR;
    }

    let pInstance = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<WDML_INSTANCE>() as SIZE_T)
        as *mut WDML_INSTANCE;
    if pInstance.is_null() {
        return DMLERR_SYS_ERROR;
    }

    (*pInstance).next = core::ptr::null_mut();
    (*pInstance).monitor = (afCmd | APPCLASS_MONITOR) as BOOL;

    (*pInstance).clientOnly = (afCmd & APPCMD_CLIENTONLY) as BOOL;
    (*pInstance).instanceID = *pidInst;
    (*pInstance).threadID = GetCurrentThreadId();
    (*pInstance).callback = pfnCallback;
    (*pInstance).unicode = bUnicode;
    (*pInstance).nodeList = core::ptr::null_mut();
    (*pInstance).monitorFlags = afCmd & MF_MASK;
    (*pInstance).wStatus = 0;
    (*pInstance).lastError = DMLERR_NO_ERROR;
    (*pInstance).servers = core::ptr::null_mut();
    (*pInstance).convs[0] = core::ptr::null_mut();
    (*pInstance).convs[1] = core::ptr::null_mut();
    (*pInstance).links[0] = core::ptr::null_mut();
    (*pInstance).links[1] = core::ptr::null_mut();

    (*pInstance).CBFflags =
        afCmd ^ ((afCmd & MF_MASK) | ((afCmd & APPCMD_MASK) | (afCmd & APPCLASS_MASK)));

    if (*pInstance).clientOnly == 0 {
        (*pInstance).clientOnly =
            (((*pInstance).CBFflags & CBF_FAIL_ALLSVRXACTIONS) == CBF_FAIL_ALLSVRXACTIONS) as BOOL;
    }

    if *pidInst == 0 {
        // Initialisation of new Instance Identifier
        WDML_Enter();

        if WDML_InstanceList.is_null() {
            WDML_InstanceList = pInstance;
            (*pInstance).CBFflags |= APPCMD_FILTERINITS;
            WDML_IncrementInstanceId(pInstance);
        } else {
            reference_inst = WDML_InstanceList;
            while !(*reference_inst).next.is_null() {
                if (*pInstance).instanceID == (*reference_inst).instanceID {
                    // Check 1 - must be same Client-only state
                    if (*pInstance).clientOnly != (*reference_inst).clientOnly {
                        ret = DMLERR_DLL_USAGE;
                        return WDML_Initialize_error(pInstance, ret);
                    }
                    // Check 2 - cannot use 'Monitor' with any non-monitor modes
                    if (*pInstance).monitor != (*reference_inst).monitor {
                        ret = DMLERR_INVALIDPARAMETER;
                        return WDML_Initialize_error(pInstance, ret);
                    }
                    // Check 3 - must supply different callback address
                    if (*pInstance).callback == (*reference_inst).callback {
                        ret = DMLERR_DLL_USAGE;
                        return WDML_Initialize_error(pInstance, ret);
                    }
                }
                reference_inst = (*reference_inst).next;
            }
            // All cleared, add to chain
            WDML_IncrementInstanceId(pInstance);
            (*reference_inst).next = pInstance;
        }
        WDML_Leave();

        *pidInst = (*pInstance).instanceID;

        // windows must always be created outside the critical section
        let mut wndclass: _WNDCLASSEXW = core::mem::zeroed();
        wndclass.cbSize = core::mem::size_of::<_WNDCLASSEXW>() as UINT;
        wndclass.style = 0;
        wndclass.lpfnWndProc = core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(WDML_EventProc);
        wndclass.cbClsExtra = 0;
        wndclass.cbWndExtra = core::mem::size_of::<ULONG_PTR>() as INT;
        wndclass.hInstance = core::ptr::null_mut();
        wndclass.hIcon = core::ptr::null_mut();
        wndclass.hCursor = core::ptr::null_mut();
        wndclass.hbrBackground = core::ptr::null_mut();
        wndclass.lpszMenuName = core::ptr::null_mut();
        wndclass.lpszClassName = WDML_szEventClass.as_ptr();
        wndclass.hIconSm = core::ptr::null_mut();

        RegisterClassExW(&wndclass);

        (*pInstance).hwndEvent = CreateWindowExW(
            0,
            WDML_szEventClass.as_ptr(),
            core::ptr::null_mut(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        SetWindowLongPtrW(
            (*pInstance).hwndEvent,
            GWL_WDML_INSTANCE,
            pInstance as ULONG_PTR as LONG_PTR,
        );
    } else {
        // Reinitialisation situation
        WDML_Enter();

        if WDML_InstanceList.is_null() {
            ret = DMLERR_INVALIDPARAMETER;
            return WDML_Initialize_error(pInstance, ret);
        }
        reference_inst = WDML_InstanceList;
        while !(*reference_inst).next.is_null() {
            if *pidInst == (*reference_inst).instanceID
                && pfnCallback == (*reference_inst).callback
            {
                // Check 1 - cannot change client-only mode if set via APPCMD_CLIENTONLY
                if (*reference_inst).clientOnly != 0
                    && ((*reference_inst).CBFflags & CBF_FAIL_ALLSVRXACTIONS)
                        != CBF_FAIL_ALLSVRXACTIONS
                    && (afCmd & APPCMD_CLIENTONLY) == 0
                {
                    ret = DMLERR_INVALIDPARAMETER;
                    return WDML_Initialize_error(pInstance, ret);
                }
                // Check 2 - cannot change monitor modes
                if (*pInstance).monitor != (*reference_inst).monitor {
                    ret = DMLERR_INVALIDPARAMETER;
                    return WDML_Initialize_error(pInstance, ret);
                }
                // Check 3 - trying to set Client-only via APPCMD when not set previously
                if (afCmd & APPCMD_CLIENTONLY) != 0 && (*reference_inst).clientOnly == 0 {
                    ret = DMLERR_INVALIDPARAMETER;
                    return WDML_Initialize_error(pInstance, ret);
                }
                break;
            }
            reference_inst = (*reference_inst).next;
        }
        if (*reference_inst).next.is_null() {
            ret = DMLERR_INVALIDPARAMETER;
            return WDML_Initialize_error(pInstance, ret);
        }
        // All checked - change relevant flags
        (*reference_inst).CBFflags = (*pInstance).CBFflags;
        (*reference_inst).clientOnly = (*pInstance).clientOnly;
        (*reference_inst).monitorFlags = (*pInstance).monitorFlags;

        HeapFree(GetProcessHeap(), 0, pInstance as LPVOID);

        WDML_Leave();
    }

    DMLERR_NO_ERROR
}

// theError: label from WDML_Initialize.
unsafe fn WDML_Initialize_error(pInstance: *mut WDML_INSTANCE, ret: UINT) -> UINT {
    HeapFree(GetProcessHeap(), 0, pInstance as LPVOID);
    WDML_Leave();
    ret
}

/// DdeInitializeA (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeInitializeA(
    pidInst: LPDWORD,
    pfnCallback: PFNCALLBACK,
    afCmd: DWORD,
    ulRes: DWORD,
) -> UINT {
    WDML_Initialize(pidInst, pfnCallback, afCmd, ulRes, FALSE as BOOL)
}

/// DdeInitializeW (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeInitializeW(
    pidInst: LPDWORD,
    pfnCallback: PFNCALLBACK,
    afCmd: DWORD,
    ulRes: DWORD,
) -> UINT {
    WDML_Initialize(pidInst, pfnCallback, afCmd, ulRes, TRUE as BOOL)
}

/// DdeUninitialize (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeUninitialize(idInst: DWORD) -> BOOL {
    let pInstance = WDML_GetInstance(idInst);
    if pInstance.is_null() {
        return FALSE as BOOL;
    }

    // first terminate all conversations client side
    let mut pConv = (*pInstance).convs[WDML_CLIENT_SIDE];
    while !pConv.is_null() {
        let pConvNext = (*pConv).next;
        DdeDisconnect(pConv as HCONV);
        pConv = pConvNext;
    }

    // then unregister all known service names
    DdeNameService(
        idInst,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        DNS_UNREGISTER,
    );

    // Free the nodes that were not freed by this instance
    WDML_FreeAllHSZ(pInstance);

    DestroyWindow((*pInstance).hwndEvent);

    // delete the instance handle itself
    if WDML_InstanceList == pInstance {
        WDML_InstanceList = (*pInstance).next;
    } else {
        let mut inst = WDML_InstanceList;
        while (*inst).next != pInstance {
            inst = (*inst).next;
        }
        (*inst).next = (*pInstance).next;
    }

    HeapFree(GetProcessHeap(), 0, pInstance as LPVOID);

    TRUE as BOOL
}

/// WDML_NotifyThreadDetach
#[no_mangle]
pub unsafe extern "C" fn WDML_NotifyThreadDetach() {
    let tid = GetCurrentThreadId();

    WDML_Enter();
    let mut pInstance = WDML_InstanceList;
    while !pInstance.is_null() {
        let next = (*pInstance).next;
        if (*pInstance).threadID == tid {
            WDML_Leave();
            DdeUninitialize((*pInstance).instanceID);
            WDML_Enter();
        }
        pInstance = next;
    }
    WDML_Leave();
}

/// WDML_InvokeCallback
#[no_mangle]
pub unsafe extern "C" fn WDML_InvokeCallback(
    pInstance: *mut WDML_INSTANCE,
    uType: UINT,
    uFmt: UINT,
    hConv: HCONV,
    hsz1: HSZ,
    hsz2: HSZ,
    hdata: HDDEDATA,
    dwData1: ULONG_PTR,
    dwData2: ULONG_PTR,
) -> HDDEDATA {
    if pInstance.is_null() {
        return core::ptr::null_mut();
    }
    match (*pInstance).callback {
        Some(cb) => cb(uType, uFmt, hConv, hsz1, hsz2, hdata, dwData1, dwData2),
        None => core::ptr::null_mut(),
    }
}

/// WDML_GetInstance
#[no_mangle]
pub unsafe extern "C" fn WDML_GetInstance(instId: DWORD) -> *mut WDML_INSTANCE {
    WDML_Enter();

    let mut pInstance = WDML_InstanceList;
    while !pInstance.is_null() {
        if (*pInstance).instanceID == instId {
            if GetCurrentThreadId() != (*pInstance).threadID {
                // Tried to get instance from wrong thread
                pInstance = (*pInstance).next;
                continue;
            }
            break;
        }
        pInstance = (*pInstance).next;
    }

    WDML_Leave();
    pInstance
}

/// WDML_GetInstanceFromWnd
#[no_mangle]
pub unsafe extern "C" fn WDML_GetInstanceFromWnd(hWnd: HWND) -> *mut WDML_INSTANCE {
    GetWindowLongPtrW(hWnd, GWL_WDML_INSTANCE) as *mut WDML_INSTANCE
}

// ================================================================
//  Data handle management
// ================================================================

/// DdeCreateDataHandle (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeCreateDataHandle(
    idInst: DWORD,
    pSrc: LPBYTE,
    cb: DWORD,
    cbOff: DWORD,
    hszItem: HSZ,
    wFmt: UINT,
    afCmd: UINT,
) -> HDDEDATA {
    let pInstance = WDML_GetInstance(idInst);
    if pInstance.is_null() {
        WDML_SetAllLastError(DMLERR_INVALIDPARAMETER);
        return core::ptr::null_mut();
    }

    let mut psz: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    if GetAtomNameW(HSZ2ATOM(hszItem), psz.as_mut_ptr(), MAX_BUFFER_LEN as INT) == 0 {
        psz[0] = HSZ2ATOM(hszItem);
        psz[1] = 0;
    }

    if afCmd != 0 && afCmd != HDATA_APPOWNED {
        return core::ptr::null_mut();
    }

    // we use the first 4 bytes to store the size
    let hMem = GlobalAlloc(
        GMEM_MOVEABLE | GMEM_DDESHARE,
        (cb + cbOff) as SIZE_T + core::mem::size_of::<DDE_DATAHANDLE_HEAD>() as SIZE_T,
    );
    if hMem.is_null() {
        return core::ptr::null_mut();
    }

    let pDdh = GlobalLock(hMem) as *mut DDE_DATAHANDLE_HEAD;
    if pDdh.is_null() {
        GlobalFree(hMem);
        return core::ptr::null_mut();
    }

    (*pDdh).cfFormat = wFmt as WORD;
    (*pDdh).bAppOwned = (afCmd == HDATA_APPOWNED) as WORD;

    let pByte = pDdh.add(1) as LPBYTE;
    if !pSrc.is_null() {
        core::ptr::copy_nonoverlapping(pSrc.offset(cbOff as isize), pByte, cb as usize);
    }
    GlobalUnlock(hMem);

    hMem
}

/// DdeAddData (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeAddData(
    mut hData: HDDEDATA,
    pSrc: LPBYTE,
    cb: DWORD,
    cbOff: DWORD,
) -> HDDEDATA {
    let mut old_sz: DWORD = 0;

    let mut pDst = DdeAccessData(hData, &mut old_sz);
    if pDst.is_null() {
        return core::ptr::null_mut();
    }

    let new_sz = cb + cbOff;
    if new_sz > old_sz {
        DdeUnaccessData(hData);
        hData = GlobalReAlloc(
            hData,
            new_sz as SIZE_T + core::mem::size_of::<DDE_DATAHANDLE_HEAD>() as SIZE_T,
            GMEM_MOVEABLE | GMEM_DDESHARE,
        );
        pDst = DdeAccessData(hData, &mut old_sz);
    }

    if pDst.is_null() {
        return core::ptr::null_mut();
    }

    core::ptr::copy_nonoverlapping(pSrc, pDst.offset(cbOff as isize), cb as usize);
    DdeUnaccessData(hData);
    hData
}

/// DdeGetData (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeGetData(
    hData: HDDEDATA,
    pDst: LPBYTE,
    cbMax: DWORD,
    cbOff: DWORD,
) -> DWORD {
    let mut dwSize: DWORD = 0;
    let dwRet: DWORD;

    let pByte = DdeAccessData(hData, &mut dwSize);

    if !pByte.is_null() {
        if pDst.is_null() {
            dwRet = dwSize;
        } else if cbOff + cbMax < dwSize {
            dwRet = cbMax;
        } else if cbOff < dwSize {
            dwRet = dwSize - cbOff;
        } else {
            dwRet = 0;
        }
        if !pDst.is_null() && dwRet != 0 {
            core::ptr::copy_nonoverlapping(pByte.offset(cbOff as isize), pDst, dwRet as usize);
        }
        DdeUnaccessData(hData);
    } else {
        dwRet = 0;
    }
    dwRet
}

/// DdeAccessData (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeAccessData(hData: HDDEDATA, pcbDataSize: LPDWORD) -> LPBYTE {
    let hMem: HGLOBAL = hData;

    let pDdh = GlobalLock(hMem) as *mut DDE_DATAHANDLE_HEAD;
    if pDdh.is_null() {
        return core::ptr::null_mut();
    }

    if !pcbDataSize.is_null() {
        *pcbDataSize =
            GlobalSize(hMem) as DWORD - core::mem::size_of::<DDE_DATAHANDLE_HEAD>() as DWORD;
    }
    pDdh.add(1) as LPBYTE
}

/// DdeUnaccessData (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeUnaccessData(hData: HDDEDATA) -> BOOL {
    let hMem: HGLOBAL = hData;
    GlobalUnlock(hMem);
    TRUE as BOOL
}

/// DdeFreeDataHandle (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeFreeDataHandle(hData: HDDEDATA) -> BOOL {
    // 1 is the handle value returned by an asynchronous operation.
    if hData as ULONG_PTR == 1 {
        return TRUE as BOOL;
    }
    (GlobalFree(hData).is_null()) as BOOL
}

/// WDML_IsAppOwned
#[no_mangle]
pub unsafe extern "C" fn WDML_IsAppOwned(hData: HDDEDATA) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let pDdh = GlobalLock(hData) as *mut DDE_DATAHANDLE_HEAD;
    if !pDdh.is_null() {
        ret = (*pDdh).bAppOwned as BOOL;
        GlobalUnlock(hData);
    }
    ret
}

// ================================================================
//  Global <=> Data handle management
// ================================================================

/// WDML_Global2DataHandle
#[no_mangle]
pub unsafe extern "C" fn WDML_Global2DataHandle(
    pConv: *mut WDML_CONV,
    hMem: HGLOBAL,
    p: *mut WINE_DDEHEAD,
) -> HDDEDATA {
    let mut ret: HDDEDATA = core::ptr::null_mut();

    if !hMem.is_null() {
        let pDd = GlobalLock(hMem) as *mut DDEDATA;
        let size = GlobalSize(hMem) as DWORD - core::mem::size_of::<WINE_DDEHEAD>() as DWORD;
        if !pDd.is_null() {
            if !p.is_null() {
                core::ptr::copy_nonoverlapping(
                    pDd as *const u8,
                    p as *mut u8,
                    core::mem::size_of::<WINE_DDEHEAD>(),
                );
            }
            match (*pDd).cfFormat as WORD {
                w if w != 0 && w != CF_TEXT && w != CF_BITMAP => {
                    // Unsupported format, passing raw information (fall through to raw)
                    ret = DdeCreateDataHandle(
                        (*(*pConv).instance).instanceID,
                        (*pDd).Value.as_mut_ptr(),
                        size,
                        0,
                        core::ptr::null_mut(),
                        (*pDd).cfFormat as UINT,
                        0,
                    );
                }
                CF_BITMAP => {
                    if size >= core::mem::size_of::<BITMAP>() as DWORD {
                        let bmp = (*pDd).Value.as_ptr() as *const BITMAP;
                        let count = (*bmp).bmWidthBytes
                            * (*bmp).bmHeight
                            * (*bmp).bmPlanes as LONG;
                        if size >= core::mem::size_of::<BITMAP>() as DWORD + count as DWORD {
                            let hbmp = CreateBitmap(
                                (*bmp).bmWidth,
                                (*bmp).bmHeight,
                                (*bmp).bmPlanes as UINT,
                                (*bmp).bmBitsPixel as UINT,
                                (*pDd).Value.as_ptr().add(core::mem::size_of::<BITMAP>())
                                    as *const c_void,
                            );
                            if !hbmp.is_null() {
                                ret = DdeCreateDataHandle(
                                    (*(*pConv).instance).instanceID,
                                    core::ptr::addr_of!(hbmp) as LPBYTE,
                                    core::mem::size_of_val(&hbmp) as DWORD,
                                    0,
                                    core::ptr::null_mut(),
                                    CF_BITMAP as UINT,
                                    0,
                                );
                            }
                        }
                    }
                }
                // 0 | CF_TEXT (and the raw fall-through default)
                _ => {
                    ret = DdeCreateDataHandle(
                        (*(*pConv).instance).instanceID,
                        (*pDd).Value.as_mut_ptr(),
                        size,
                        0,
                        core::ptr::null_mut(),
                        (*pDd).cfFormat as UINT,
                        0,
                    );
                }
            }
            GlobalUnlock(hMem);
        }
    }
    ret
}

/// WDML_DataHandle2Global
#[no_mangle]
pub unsafe extern "C" fn WDML_DataHandle2Global(
    hDdeData: HDDEDATA,
    fResponse: BOOL,
    fRelease: BOOL,
    fDeferUpd: BOOL,
    fAckReq: BOOL,
) -> HGLOBAL {
    let mut hMem: HGLOBAL = core::ptr::null_mut();

    let dwSize =
        GlobalSize(hDdeData) as DWORD - core::mem::size_of::<DDE_DATAHANDLE_HEAD>() as DWORD;
    let pDdh = GlobalLock(hDdeData) as *mut DDE_DATAHANDLE_HEAD;
    if dwSize != 0 && !pDdh.is_null() {
        let mut wdh: *mut WINE_DDEHEAD = core::ptr::null_mut();

        match (*pDdh).cfFormat {
            CF_BITMAP => {
                if dwSize >= core::mem::size_of::<HBITMAP>() as DWORD {
                    let mut bmp: BITMAP = core::mem::zeroed();
                    let hbmp = *(pDdh.add(1) as *const HBITMAP);

                    if GetObjectW(
                        hbmp as HANDLE,
                        core::mem::size_of::<BITMAP>() as INT,
                        core::ptr::addr_of_mut!(bmp) as LPVOID,
                    ) != 0
                    {
                        let count = (bmp.bmWidthBytes * bmp.bmHeight) as DWORD;
                        hMem = GlobalAlloc(
                            GMEM_MOVEABLE | GMEM_DDESHARE,
                            core::mem::size_of::<WINE_DDEHEAD>() as SIZE_T
                                + core::mem::size_of::<BITMAP>() as SIZE_T
                                + count as SIZE_T,
                        );
                        if !hMem.is_null() {
                            wdh = GlobalLock(hMem) as *mut WINE_DDEHEAD;
                            if !wdh.is_null() {
                                core::ptr::copy_nonoverlapping(
                                    core::ptr::addr_of!(bmp) as *const u8,
                                    wdh.add(1) as *mut u8,
                                    core::mem::size_of::<BITMAP>(),
                                );
                                GetBitmapBits(
                                    hbmp,
                                    count as LONG,
                                    (wdh.add(1) as *mut u8).add(core::mem::size_of::<BITMAP>())
                                        as LPVOID,
                                );
                            }
                        }
                    }
                }
            }
            // default / 0 / CF_TEXT — raw copy
            _ => {
                hMem = GlobalAlloc(
                    GMEM_MOVEABLE | GMEM_DDESHARE,
                    core::mem::size_of::<WINE_DDEHEAD>() as SIZE_T + dwSize as SIZE_T,
                );
                if !hMem.is_null() {
                    wdh = GlobalLock(hMem) as *mut WINE_DDEHEAD;
                    if !wdh.is_null() {
                        core::ptr::copy_nonoverlapping(
                            pDdh.add(1) as *const u8,
                            wdh.add(1) as *mut u8,
                            dwSize as usize,
                        );
                    }
                }
            }
        }

        if !wdh.is_null() {
            (*wdh).set_unused(0);
            (*wdh).set_fResponse(fResponse as u16);
            (*wdh).set_fRelease(fRelease as u16);
            (*wdh).set_fDeferUpd(fDeferUpd as u16);
            (*wdh).set_fAckReq(fAckReq as u16);
            (*wdh).cfFormat = (*pDdh).cfFormat as i16;
            GlobalUnlock(hMem);
        }
        GlobalUnlock(hDdeData);
    }

    hMem
}

// ================================================================
//  Server management
// ================================================================

/// WDML_AddServer
#[no_mangle]
pub unsafe extern "C" fn WDML_AddServer(
    pInstance: *mut WDML_INSTANCE,
    hszService: HSZ,
    _hszTopic: HSZ,
) -> *mut WDML_SERVER {
    let pServer = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<WDML_SERVER>() as SIZE_T)
        as *mut WDML_SERVER;
    if pServer.is_null() {
        return core::ptr::null_mut();
    }

    (*pServer).hszService = hszService;
    WDML_IncHSZ(pInstance, hszService);

    let mut buf1: [u16; 256] = [0; 256];
    let mut buf2: [u16; 256] = [0; 256];
    DdeQueryStringW(
        (*pInstance).instanceID,
        hszService,
        buf1.as_mut_ptr(),
        256,
        CP_WINUNICODE,
    );
    // snprintfW(buf2, 256, L"%s(0x%*x)", buf1, 2*sizeof(ULONG_PTR), GetCurrentProcessId())
    WDML_FormatServiceSpec(
        &mut buf2,
        buf1.as_ptr(),
        2 * core::mem::size_of::<ULONG_PTR>(),
        GetCurrentProcessId(),
    );
    (*pServer).hszServiceSpec =
        DdeCreateStringHandleW((*pInstance).instanceID, buf2.as_ptr(), CP_WINUNICODE);

    (*pServer).atomService = WDML_MakeAtomFromHsz((*pServer).hszService);
    (*pServer).atomServiceSpec = WDML_MakeAtomFromHsz((*pServer).hszServiceSpec);

    (*pServer).filterOn = TRUE as BOOL;

    (*pServer).next = (*pInstance).servers;
    (*pInstance).servers = pServer;
    pServer
}

// helper reproducing snprintfW(buf2, 256, L"%s(0x%*x)", buf1, width, pid):
// "<service>(0x<pid hex, zero-padded to width>)"
unsafe fn WDML_FormatServiceSpec(
    dst: &mut [u16; 256],
    svc: *const u16,
    width: usize,
    pid: DWORD,
) {
    let mut i = 0usize;
    // copy service name
    let svc_len = wcslen_local(svc);
    let mut k = 0usize;
    while k < svc_len && i < 254 {
        dst[i] = *svc.add(k);
        i += 1;
        k += 1;
    }
    // literal "(0x"
    for &c in &[b'(' as u16, b'0' as u16, b'x' as u16] {
        if i < 255 {
            dst[i] = c;
            i += 1;
        }
    }
    // hex digits right-justified to width — `%*x` pads with SPACES, not zeros
    let mut hexbuf: [u16; 16] = [0; 16];
    let mut hlen = 0usize;
    let mut v = pid;
    if v == 0 {
        hexbuf[hlen] = b'0' as u16;
        hlen += 1;
    }
    while v != 0 {
        let d = (v & 0xf) as u8;
        let ch = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
        hexbuf[hlen] = ch as u16;
        hlen += 1;
        v >>= 4;
    }
    while hlen < width && hlen < 16 {
        hexbuf[hlen] = b' ' as u16;
        hlen += 1;
    }
    // emit reversed
    let mut j = hlen;
    while j > 0 && i < 255 {
        j -= 1;
        dst[i] = hexbuf[j];
        i += 1;
    }
    // literal ")"
    if i < 255 {
        dst[i] = b')' as u16;
        i += 1;
    }
    dst[i] = 0;
}

/// WDML_RemoveServer
#[no_mangle]
pub unsafe extern "C" fn WDML_RemoveServer(
    pInstance: *mut WDML_INSTANCE,
    hszService: HSZ,
    _hszTopic: HSZ,
) {
    let mut pPrev: *mut WDML_SERVER = core::ptr::null_mut();
    let mut pServer = (*pInstance).servers;

    while !pServer.is_null() {
        if DdeCmpStringHandles((*pServer).hszService, hszService) == 0 {
            WDML_BroadcastDDEWindows(
                WDML_szEventClass.as_ptr(),
                WM_WDML_UNREGISTER,
                (*pServer).atomService as WPARAM,
                (*pServer).atomServiceSpec as LPARAM,
            );
            // terminate all conversations for given topic
            let mut pConv = (*pInstance).convs[WDML_SERVER_SIDE];
            while !pConv.is_null() {
                let pConvNext = (*pConv).next;
                if DdeCmpStringHandles((*pConv).hszService, hszService) == 0 {
                    let client = (*pConv).hwndClient;
                    let server = (*pConv).hwndServer;
                    WDML_RemoveConv(pConv, WDML_SERVER_SIDE as INT);
                    PostMessageW(client, WM_DDE_TERMINATE, server as WPARAM, 0);
                }
                pConv = pConvNext;
            }
            if pServer == (*pInstance).servers {
                (*pInstance).servers = (*pServer).next;
            } else {
                (*pPrev).next = (*pServer).next;
            }

            DestroyWindow((*pServer).hwndServer);
            WDML_DecHSZ(pInstance, (*pServer).hszServiceSpec);
            WDML_DecHSZ(pInstance, (*pServer).hszService);

            GlobalDeleteAtom((*pServer).atomService);
            GlobalDeleteAtom((*pServer).atomServiceSpec);

            HeapFree(GetProcessHeap(), 0, pServer as LPVOID);
            break;
        }

        pPrev = pServer;
        pServer = (*pServer).next;
    }
}

/// WDML_FindServer
#[no_mangle]
pub unsafe extern "C" fn WDML_FindServer(
    pInstance: *mut WDML_INSTANCE,
    hszService: HSZ,
    _hszTopic: HSZ,
) -> *mut WDML_SERVER {
    let mut pServer = (*pInstance).servers;
    while !pServer.is_null() {
        if hszService == (*pServer).hszService {
            return pServer;
        }
        pServer = (*pServer).next;
    }
    core::ptr::null_mut()
}

// ================================================================
//  Link (hot & warm) management
// ================================================================

/// WDML_AddLink
#[no_mangle]
pub unsafe extern "C" fn WDML_AddLink(
    pInstance: *mut WDML_INSTANCE,
    hConv: HCONV,
    side: INT,
    wType: UINT,
    hszItem: HSZ,
    wFmt: UINT,
) {
    let pLink = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<WDML_LINK>() as SIZE_T)
        as *mut WDML_LINK;
    if pLink.is_null() {
        return;
    }

    (*pLink).hConv = hConv;
    (*pLink).transactionType = wType;
    (*pLink).hszItem = hszItem;
    WDML_IncHSZ(pInstance, (*pLink).hszItem);
    (*pLink).uFmt = wFmt;
    (*pLink).next = (*pInstance).links[side as usize];
    (*pInstance).links[side as usize] = pLink;
}

/// WDML_RemoveLink
#[no_mangle]
pub unsafe extern "C" fn WDML_RemoveLink(
    pInstance: *mut WDML_INSTANCE,
    hConv: HCONV,
    side: INT,
    hszItem: HSZ,
    uFmt: UINT,
) {
    let mut pPrev: *mut WDML_LINK = core::ptr::null_mut();
    let mut pCurrent = (*pInstance).links[side as usize];

    while !pCurrent.is_null() {
        if (*pCurrent).hConv == hConv
            && DdeCmpStringHandles((*pCurrent).hszItem, hszItem) == 0
            && (*pCurrent).uFmt == uFmt
        {
            if pCurrent == (*pInstance).links[side as usize] {
                (*pInstance).links[side as usize] = (*pCurrent).next;
            } else {
                (*pPrev).next = (*pCurrent).next;
            }
            WDML_DecHSZ(pInstance, (*pCurrent).hszItem);
            HeapFree(GetProcessHeap(), 0, pCurrent as LPVOID);
            break;
        }

        pPrev = pCurrent;
        pCurrent = (*pCurrent).next;
    }
}

unsafe fn WDML_RemoveAllLinks(pInstance: *mut WDML_INSTANCE, pConv: *mut WDML_CONV, side: INT) {
    let mut pPrev: *mut WDML_LINK = core::ptr::null_mut();
    let mut pCurrent = (*pInstance).links[side as usize];
    let mut pNext: *mut WDML_LINK;

    while !pCurrent.is_null() {
        if (*pCurrent).hConv == pConv as HCONV {
            pNext = (*pCurrent).next;
            if pCurrent == (*pInstance).links[side as usize] {
                (*pInstance).links[side as usize] = (*pCurrent).next;
            } else {
                (*pPrev).next = (*pCurrent).next;
            }

            WDML_DecHSZ(pInstance, (*pCurrent).hszItem);

            HeapFree(GetProcessHeap(), 0, pCurrent as LPVOID);
            pCurrent = pNext;
        } else {
            pPrev = pCurrent;
            pCurrent = (*pCurrent).next;
        }
    }
}

/// WDML_FindLink
#[no_mangle]
pub unsafe extern "C" fn WDML_FindLink(
    pInstance: *mut WDML_INSTANCE,
    hConv: HCONV,
    side: INT,
    hszItem: HSZ,
    use_fmt: BOOL,
    uFmt: UINT,
) -> *mut WDML_LINK {
    let mut pCurrent = (*pInstance).links[side as usize];

    while !pCurrent.is_null() {
        if (*pCurrent).hConv == hConv
            && DdeCmpStringHandles((*pCurrent).hszItem, hszItem) == 0
            && (use_fmt == 0 || (*pCurrent).uFmt == uFmt)
        {
            break;
        }
        pCurrent = (*pCurrent).next;
    }

    pCurrent
}

// ================================================================
//  Transaction management
// ================================================================

static mut WDML_AllocTransaction_tid: WORD = 1;

/// WDML_AllocTransaction
#[no_mangle]
pub unsafe extern "C" fn WDML_AllocTransaction(
    pInstance: *mut WDML_INSTANCE,
    ddeMsg: UINT,
    wFmt: UINT,
    hszItem: HSZ,
) -> *mut WDML_XACT {
    let pXAct = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<WDML_XACT>() as SIZE_T)
        as *mut WDML_XACT;
    if pXAct.is_null() {
        (*pInstance).lastError = DMLERR_MEMORY_ERROR;
        return core::ptr::null_mut();
    }

    (*pXAct).xActID = WDML_AllocTransaction_tid as DWORD;
    WDML_AllocTransaction_tid = WDML_AllocTransaction_tid.wrapping_add(1);
    (*pXAct).ddeMsg = ddeMsg;
    (*pXAct).hDdeData = core::ptr::null_mut();
    (*pXAct).hUser = 0;
    (*pXAct).next = core::ptr::null_mut();
    (*pXAct).wType = 0;
    (*pXAct).wFmt = wFmt;
    (*pXAct).hszItem = hszItem;
    if !(*pXAct).hszItem.is_null() {
        WDML_IncHSZ(pInstance, (*pXAct).hszItem);
    }
    (*pXAct).atom = 0;
    (*pXAct).hMem = core::ptr::null_mut();
    (*pXAct).lParam = 0;

    pXAct
}

/// WDML_QueueTransaction
#[no_mangle]
pub unsafe extern "C" fn WDML_QueueTransaction(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) {
    // advance to last in queue
    let mut pt: *mut *mut WDML_XACT = core::ptr::addr_of_mut!((*pConv).transactions);
    while !(*pt).is_null() {
        pt = core::ptr::addr_of_mut!((**pt).next);
    }
    *pt = pXAct;
}

/// WDML_UnQueueTransaction
#[no_mangle]
pub unsafe extern "C" fn WDML_UnQueueTransaction(
    pConv: *mut WDML_CONV,
    pXAct: *mut WDML_XACT,
) -> BOOL {
    let mut pt: *mut *mut WDML_XACT = core::ptr::addr_of_mut!((*pConv).transactions);
    while !(*pt).is_null() {
        if *pt == pXAct {
            *pt = (*pXAct).next;
            return TRUE as BOOL;
        }
        pt = core::ptr::addr_of_mut!((**pt).next);
    }
    FALSE as BOOL
}

/// WDML_FreeTransaction
#[no_mangle]
pub unsafe extern "C" fn WDML_FreeTransaction(
    pInstance: *mut WDML_INSTANCE,
    pXAct: *mut WDML_XACT,
    doFreePmt: BOOL,
) {
    if doFreePmt != 0 && (*pXAct).hMem as ULONG_PTR > 1 {
        GlobalFree((*pXAct).hMem);
    }
    if !(*pXAct).hszItem.is_null() {
        WDML_DecHSZ(pInstance, (*pXAct).hszItem);
    }

    HeapFree(GetProcessHeap(), 0, pXAct as LPVOID);
}

unsafe fn WDML_FindTransaction(pConv: *mut WDML_CONV, tid: DWORD) -> *mut WDML_XACT {
    let tid = HIWORD(tid as ULONG_PTR);
    let mut pXAct = (*pConv).transactions;
    while !pXAct.is_null() {
        if (*pXAct).xActID == tid {
            break;
        }
        pXAct = (*pXAct).next;
    }
    pXAct
}

// ================================================================
//  Conversation management
// ================================================================

/// WDML_AddConv
#[no_mangle]
pub unsafe extern "C" fn WDML_AddConv(
    pInstance: *mut WDML_INSTANCE,
    side: INT,
    hszService: HSZ,
    hszTopic: HSZ,
    hwndClient: HWND,
    hwndServer: HWND,
) -> *mut WDML_CONV {
    let pConv = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<WDML_CONV>() as SIZE_T)
        as *mut WDML_CONV;
    if pConv.is_null() {
        return core::ptr::null_mut();
    }

    (*pConv).instance = pInstance;
    (*pConv).hszService = hszService;
    WDML_IncHSZ(pInstance, (*pConv).hszService);
    (*pConv).hszTopic = hszTopic;
    WDML_IncHSZ(pInstance, (*pConv).hszTopic);
    (*pConv).magic = WDML_CONV_MAGIC;
    (*pConv).hwndServer = hwndServer;
    (*pConv).hwndClient = hwndClient;
    (*pConv).transactions = core::ptr::null_mut();
    (*pConv).hUser = 0;
    (*pConv).wStatus = if side as usize == WDML_CLIENT_SIDE {
        ST_CLIENT
    } else {
        0
    };
    (*pConv).wStatus |= (*pInstance).wStatus;
    // check if both side of the conversation are of the same instance
    if GetWindowThreadProcessId(hwndClient, core::ptr::null_mut())
        == GetWindowThreadProcessId(hwndServer, core::ptr::null_mut())
        && WDML_GetInstanceFromWnd(hwndClient) == WDML_GetInstanceFromWnd(hwndServer)
    {
        (*pConv).wStatus |= ST_ISSELF;
    }
    (*pConv).wConvst = XST_NULL;

    (*pConv).next = (*pInstance).convs[side as usize];
    (*pInstance).convs[side as usize] = pConv;

    pConv
}

/// WDML_FindConv
#[no_mangle]
pub unsafe extern "C" fn WDML_FindConv(
    pInstance: *mut WDML_INSTANCE,
    side: INT,
    hszService: HSZ,
    hszTopic: HSZ,
) -> *mut WDML_CONV {
    let mut pCurrent = (*pInstance).convs[side as usize];
    while !pCurrent.is_null() {
        if DdeCmpStringHandles((*pCurrent).hszService, hszService) == 0
            && DdeCmpStringHandles((*pCurrent).hszTopic, hszTopic) == 0
        {
            return pCurrent;
        }
        pCurrent = (*pCurrent).next;
    }
    core::ptr::null_mut()
}

/// WDML_RemoveConv
#[no_mangle]
pub unsafe extern "C" fn WDML_RemoveConv(pRef: *mut WDML_CONV, side: INT) {
    let mut pPrev: *mut WDML_CONV = core::ptr::null_mut();
    let mut pCurrent: *mut WDML_CONV;
    let hWnd: HWND;

    if pRef.is_null() {
        return;
    }

    // remove any pending transaction
    let mut pXAct = (*pRef).transactions;
    while !pXAct.is_null() {
        let pXActNext = (*pXAct).next;
        WDML_FreeTransaction((*pRef).instance, pXAct, TRUE as BOOL);
        pXAct = pXActNext;
    }

    WDML_RemoveAllLinks((*pRef).instance, pRef, side);

    hWnd = if side as usize == WDML_CLIENT_SIDE {
        (*pRef).hwndClient
    } else {
        (*pRef).hwndServer
    };
    SetWindowLongPtrW(hWnd, GWL_WDML_CONVERSATION, 0);

    DestroyWindow(if side as usize == WDML_CLIENT_SIDE {
        (*pRef).hwndClient
    } else {
        (*pRef).hwndServer
    });

    WDML_DecHSZ((*pRef).instance, (*pRef).hszService);
    WDML_DecHSZ((*pRef).instance, (*pRef).hszTopic);

    pCurrent = (*(*pRef).instance).convs[side as usize];
    while !pCurrent.is_null() {
        if pCurrent == pRef {
            if pCurrent == (*(*pRef).instance).convs[side as usize] {
                (*(*pRef).instance).convs[side as usize] = (*pCurrent).next;
            } else {
                (*pPrev).next = (*pCurrent).next;
            }
            (*pCurrent).magic = 0;
            HeapFree(GetProcessHeap(), 0, pCurrent as LPVOID);
            break;
        }
        pPrev = pCurrent;
        pCurrent = (*pCurrent).next;
    }
}

unsafe fn WDML_EnableCallback(pConv: *mut WDML_CONV, wCmd: UINT) -> BOOL {
    if wCmd == EC_DISABLE {
        (*pConv).wStatus |= ST_BLOCKED;
        return TRUE as BOOL;
    }

    if wCmd == EC_QUERYWAITING {
        return (!(*pConv).transactions.is_null()) as BOOL;
    }

    if wCmd != EC_ENABLEALL && wCmd != EC_ENABLEONE {
        return FALSE as BOOL;
    }

    if wCmd == EC_ENABLEALL {
        (*pConv).wStatus &= !ST_BLOCKED;
    }

    while !(*pConv).transactions.is_null() {
        let pXAct = (*pConv).transactions;

        if ((*pConv).wStatus & ST_CLIENT) != 0 {
            // transaction should be in the queue until handled
            WDML_ClientHandle(pConv, pXAct, 0, core::ptr::null_mut());
            WDML_UnQueueTransaction(pConv, pXAct);
        } else {
            // transaction should be removed from the queue before handling
            WDML_UnQueueTransaction(pConv, pXAct);
            WDML_ServerHandle(pConv, pXAct);
        }

        WDML_FreeTransaction((*pConv).instance, pXAct, TRUE as BOOL);

        if wCmd == EC_ENABLEONE {
            break;
        }
    }
    TRUE as BOOL
}

/// DdeEnableCallback (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeEnableCallback(idInst: DWORD, hConv: HCONV, wCmd: UINT) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    if !hConv.is_null() {
        let pConv = WDML_GetConv(hConv, TRUE as BOOL);
        if !pConv.is_null() && (*(*pConv).instance).instanceID == idInst {
            ret = WDML_EnableCallback(pConv, wCmd);
        }
    } else {
        let pInstance = WDML_GetInstance(idInst);

        if pInstance.is_null() {
            return FALSE as BOOL;
        }

        (*pInstance).wStatus |= wCmd;

        if wCmd == EC_DISABLE {
            (*pInstance).wStatus |= ST_BLOCKED;
        } else if wCmd == EC_ENABLEALL {
            (*pInstance).wStatus &= !ST_BLOCKED;
        }

        ret = TRUE as BOOL;

        let mut pConv = (*pInstance).convs[WDML_CLIENT_SIDE];
        while !pConv.is_null() {
            ret = WDML_EnableCallback(pConv, wCmd);
            if ret != 0 && wCmd == EC_QUERYWAITING {
                break;
            }
            pConv = (*pConv).next;
        }
    }

    ret
}

/// WDML_GetConv
#[no_mangle]
pub unsafe extern "C" fn WDML_GetConv(hConv: HCONV, checkConnected: BOOL) -> *mut WDML_CONV {
    let pConv = hConv as *mut WDML_CONV;

    if pConv.is_null() || (*pConv).magic != WDML_CONV_MAGIC {
        return core::ptr::null_mut();
    }

    if (*pConv).instance.is_null() {
        return core::ptr::null_mut();
    }

    if (*(*pConv).instance).threadID != GetCurrentThreadId() {
        (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
        return core::ptr::null_mut();
    }

    if checkConnected != 0 && ((*pConv).wStatus & ST_CONNECTED) == 0 {
        (*(*pConv).instance).lastError = DMLERR_NO_CONV_ESTABLISHED;
        return core::ptr::null_mut();
    }

    pConv
}

/// WDML_GetConvFromWnd
#[no_mangle]
pub unsafe extern "C" fn WDML_GetConvFromWnd(hWnd: HWND) -> *mut WDML_CONV {
    GetWindowLongPtrW(hWnd, GWL_WDML_CONVERSATION) as *mut WDML_CONV
}

/// WDML_PostAck
#[no_mangle]
pub unsafe extern "C" fn WDML_PostAck(
    pConv: *mut WDML_CONV,
    side: INT,
    appRetCode: WORD,
    fBusy: BOOL,
    fAck: BOOL,
    pmt: UINT_PTR,
    mut lParam: LPARAM,
    oldMsg: UINT,
) -> BOOL {
    let from: HWND;
    let to: HWND;

    if side as usize == WDML_SERVER_SIDE {
        from = (*pConv).hwndServer;
        to = (*pConv).hwndClient;
    } else {
        to = (*pConv).hwndServer;
        from = (*pConv).hwndClient;
    }

    let mut ddeAck = DDEACK { bits: 0 };
    ddeAck.set_bAppReturnCode(appRetCode);
    ddeAck.set_reserved(0);
    ddeAck.set_fBusy(fBusy as u16);
    ddeAck.set_fAck(fAck as u16);

    let ackword: WORD = ddeAck.bits;

    lParam = if lParam != 0 {
        ReuseDDElParam(lParam, oldMsg, WM_DDE_ACK, ackword as UINT_PTR, pmt)
    } else {
        PackDDElParam(WM_DDE_ACK, ackword as UINT_PTR, pmt)
    };
    if PostMessageW(to, WM_DDE_ACK, from as WPARAM, lParam) == 0 {
        (*pConv).wStatus &= !ST_CONNECTED;
        (*(*pConv).instance).lastError = DMLERR_POSTMSG_FAILED;
        FreeDDElParam(WM_DDE_ACK, lParam);
        return FALSE as BOOL;
    }
    TRUE as BOOL
}

/// DdeSetUserHandle (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeSetUserHandle(hConv: HCONV, id: DWORD, hUser: DWORD) -> BOOL {
    let pConv = WDML_GetConv(hConv, FALSE as BOOL);
    if pConv.is_null() {
        return FALSE as BOOL;
    }

    if id == QID_SYNC {
        (*pConv).hUser = hUser;
    } else {
        let pXAct = WDML_FindTransaction(pConv, id);
        if !pXAct.is_null() {
            (*pXAct).hUser = hUser;
        } else {
            (*(*pConv).instance).lastError = DMLERR_UNFOUND_QUEUE_ID;
            return FALSE as BOOL;
        }
    }
    TRUE as BOOL
}

unsafe fn WDML_GetLocalConvInfo(pConv: *mut WDML_CONV, ci: *mut CONVINFO, id: DWORD) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;

    (*ci).hConvPartner = if ((*pConv).wStatus & ST_ISLOCAL) != 0 {
        (pConv as ULONG_PTR | 1) as HCONV
    } else {
        core::ptr::null_mut()
    };
    (*ci).hszSvcPartner = (*pConv).hszService;
    (*ci).hszServiceReq = (*pConv).hszService;
    (*ci).hszTopic = (*pConv).hszTopic;
    (*ci).wStatus = (*pConv).wStatus;

    let side: usize = if ((*pConv).wStatus & ST_CLIENT) != 0 {
        WDML_CLIENT_SIDE
    } else {
        WDML_SERVER_SIDE
    };

    let mut pLink = (*(*pConv).instance).links[side];
    while !pLink.is_null() {
        if (*pLink).hConv == pConv as HCONV {
            (*ci).wStatus |= ST_ADVISE;
            break;
        }
        pLink = (*pLink).next;
    }

    (*ci).wConvst = (*pConv).wConvst;

    (*ci).wLastError = 0;
    (*ci).hConvList = core::ptr::null_mut();
    (*ci).ConvCtxt = (*pConv).convContext;
    if ((*ci).wStatus & ST_CLIENT) != 0 {
        (*ci).hwnd = (*pConv).hwndClient;
        (*ci).hwndPartner = (*pConv).hwndServer;
    } else {
        (*ci).hwnd = (*pConv).hwndServer;
        (*ci).hwndPartner = (*pConv).hwndClient;
    }
    if id == QID_SYNC {
        (*ci).hUser = (*pConv).hUser;
        (*ci).hszItem = core::ptr::null_mut();
        (*ci).wFmt = 0;
        (*ci).wType = 0;
    } else {
        let pXAct = WDML_FindTransaction(pConv, id);
        if !pXAct.is_null() {
            (*ci).hUser = (*pXAct).hUser;
            (*ci).hszItem = (*pXAct).hszItem;
            (*ci).wFmt = (*pXAct).wFmt;
            (*ci).wType = (*pXAct).wType;
        } else {
            ret = 0;
            (*(*pConv).instance).lastError = DMLERR_UNFOUND_QUEUE_ID;
        }
    }
    ret
}

/// DdeQueryConvInfo (USER32.@)
#[no_mangle]
pub unsafe extern "system" fn DdeQueryConvInfo(
    hConv: HCONV,
    id: DWORD,
    lpConvInfo: PCONVINFO,
) -> UINT {
    let mut ret: UINT = (*lpConvInfo).cb;
    let mut ci: CONVINFO = core::mem::zeroed();

    if hConv.is_null() {
        return 0;
    }

    let pConv = WDML_GetConv(hConv, FALSE as BOOL);
    if !pConv.is_null() {
        if WDML_GetLocalConvInfo(pConv, &mut ci, id) == 0 {
            ret = 0;
        }
    } else {
        if (hConv as ULONG_PTR & 1) != 0 {
            let pConv2 = WDML_GetConv((hConv as ULONG_PTR & !1) as HCONV, FALSE as BOOL);
            let _ = pConv2; // FIXME: remote conversation info not implemented
        }
        ret = 0;
    }

    if ret != 0 {
        ci.cb = (*lpConvInfo).cb;
        let n = core::cmp::min(
            (*lpConvInfo).cb as usize,
            core::mem::size_of::<CONVINFO>(),
        );
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(ci) as *const u8,
            lpConvInfo as *mut u8,
            n,
        );
    }
    ret
}

// ================================================================
//  Information broadcast across DDEML implementations
// ================================================================

#[repr(C)]
struct tagWDML_BroadcastPmt {
    clsName: LPCWSTR,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
}

unsafe extern "system" fn WDML_BroadcastEnumProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let s = lParam as *mut tagWDML_BroadcastPmt;
    let mut buffer: [u16; 128] = [0; 128];

    if GetClassNameW(hWnd, buffer.as_mut_ptr(), 128) > 0
        && lstrcmpiW(buffer.as_ptr(), (*s).clsName) == 0
    {
        PostMessageW(hWnd, (*s).uMsg, (*s).wParam, (*s).lParam);
    }
    TRUE as BOOL
}

/// WDML_BroadcastDDEWindows
#[no_mangle]
pub unsafe extern "C" fn WDML_BroadcastDDEWindows(
    clsName: LPCWSTR,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut s = tagWDML_BroadcastPmt {
        clsName,
        uMsg,
        wParam,
        lParam,
    };
    EnumWindows(
        Some(WDML_BroadcastEnumProc),
        core::ptr::addr_of_mut!(s) as LPARAM,
    );
}
