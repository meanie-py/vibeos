//! ddeclient.rs — Rust port of win32ss/user/user32/misc/ddeclient.c (DDEML client side)
//!
//! Faithful cohab translation: the public Dde* APIs (WINAPI → extern "system"),
//! WDML_ClientHandle (plain cross-file → extern "C"), the static WDML_Client* /
//! WDML_Handle* helpers (private), and the WDML_ClientProc CALLBACK window proc.
//!
//! Shared WDML_* structs come from win32ss/user/user32/include/dde_private.h and are
//! reproduced #[repr(C)] locally (DDEML/dde.h types are NOT in the win32k bindgen).
//! Cross-file WDML_* helpers DEFINED in ddemisc.c/ddeserver.c are declared extern,
//! and the DDEML/user32 public APIs we call are declared extern "system".

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::message::GlobalAlloc;
// The crate's own functions: the definition is the declaration.
use crate::misc::ddemisc::WDML_AddConv;
use crate::misc::ddemisc::WDML_AddLink;
use crate::misc::ddemisc::WDML_AllocTransaction;
use crate::misc::ddemisc::WDML_DecHSZ;
use crate::misc::ddemisc::WDML_FindLink;
use crate::misc::ddemisc::WDML_FreeTransaction;
use crate::misc::ddemisc::WDML_GetConv;
use crate::misc::ddemisc::WDML_GetConvFromWnd;
use crate::misc::ddemisc::WDML_GetInstance;
use crate::misc::ddemisc::WDML_GetInstanceFromWnd;
use crate::misc::ddemisc::WDML_InvokeCallback;
use crate::misc::ddemisc::WDML_MakeHszFromAtom;
use crate::misc::ddemisc::WDML_PostAck;
use crate::misc::ddemisc::WDML_QueueTransaction;
use crate::misc::ddemisc::WDML_RemoveConv;
use crate::misc::ddemisc::WDML_RemoveLink;
#[allow(unused_imports)]
use core::ffi::c_void;

// LOWORD/HIWORD live in crate::internal (mirror misc/misc.rs).
use crate::internal::{HIWORD, LOWORD};

// ────────────────────────────────────────────────────────────────────────────
// DDEML / DDE constants — these come from <ddeml.h> / <dde.h>, neither of which
// is pulled into the win32k bindgen, so reproduce them locally with their real
// header values.
// ────────────────────────────────────────────────────────────────────────────

// DDE window messages (dde.h)
const WM_DDE_INITIATE: UINT = 0x3E0;
const WM_DDE_TERMINATE: UINT = 0x3E1;
const WM_DDE_ADVISE: UINT = 0x3E2;
const WM_DDE_UNADVISE: UINT = 0x3E3;
const WM_DDE_ACK: UINT = 0x3E4;
const WM_DDE_DATA: UINT = 0x3E5;
const WM_DDE_REQUEST: UINT = 0x3E6;
const WM_DDE_POKE: UINT = 0x3E7;
const WM_DDE_EXECUTE: UINT = 0x3E8;
const WM_DDE_FIRST: UINT = WM_DDE_INITIATE;
const WM_DDE_LAST: UINT = WM_DDE_EXECUTE;

// winuser.h
const HWND_BROADCAST: HWND = 0xffff as HWND;
const WS_POPUP: DWORD = 0x80000000;
const SMTO_ABORTIFHUNG: UINT = 0x0002;
const PM_REMOVE: UINT = 0x0001;
const QS_POSTMESSAGE: DWORD = 0x0008;
const WAIT_OBJECT_0: DWORD = 0;

// GWL offsets for the WDML conv window (dde_private.h):
//   GWL_WDML_INSTANCE = 0, GWL_WDML_CONVERSATION = sizeof(ULONG_PTR)
const GWL_WDML_INSTANCE: INT = 0;
const GWL_WDML_CONVERSATION: INT = core::mem::size_of::<ULONG_PTR>() as INT;

// winbase.h global memory flags
const GMEM_MOVEABLE: UINT = 0x0002;
const GHND: UINT = 0x0042;
const GMEM_DDESHARE: UINT = 0x2000;

// ddeml.h code pages
const CP_ACP: UINT = 0;
const CP_WINUNICODE: INT = 1200;
const CP_WINANSI: INT = 1004;

// ddeml.h transaction types / flags
const XCLASS_BOOL: UINT = 0x1000;
const XCLASS_DATA: UINT = 0x2000;
const XCLASS_FLAGS: UINT = 0x4000;
const XCLASS_NOTIFICATION: UINT = 0x8000;
const XTYPF_NOBLOCK: UINT = 0x0002;
const XTYPF_NODATA: UINT = 0x0004;
const XTYPF_ACKREQ: UINT = 0x0008;
const XTYP_ADVDATA: UINT = 0x0010 | XCLASS_FLAGS;
const XTYP_ADVSTART: UINT = 0x0030 | XCLASS_BOOL;
const XTYP_ADVSTOP: UINT = 0x0040 | XCLASS_NOTIFICATION;
const XTYP_EXECUTE: UINT = 0x0050 | XCLASS_FLAGS;
const XTYP_XACT_COMPLETE: UINT = 0x0080 | XCLASS_NOTIFICATION;
const XTYP_POKE: UINT = 0x0090 | XCLASS_FLAGS;
const XTYP_REQUEST: UINT = 0x00B0 | XCLASS_DATA;
const XTYP_DISCONNECT: UINT = 0x00C0 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;

const TIMEOUT_ASYNC: DWORD = 0xFFFFFFFF;

// ddeml.h ack / status flags
const DDE_FACK: WORD = 0x8000;
const DDE_FNOTPROCESSED: WORD = 0x0000;

// conversation status bits (ddeml.h)
const XST_CONNECTED: DWORD = 2;
const XST_INIT1: DWORD = 3;
const ST_CONNECTED: DWORD = 0x0001;
const ST_ISLOCAL: DWORD = 0x0004;
const ST_BLOCKED: DWORD = 0x0008;
const ST_CLIENT: DWORD = 0x0010;
const ST_TERMINATED: DWORD = 0x0020;
const ST_ISSELF: DWORD = 0x0100;

const CBF_SKIP_DISCONNECTS: DWORD = 0x0020_0000;

// DMLERR_* (ddeml.h)
const DMLERR_NO_ERROR: DWORD = 0;
const DMLERR_ADVACKTIMEOUT: DWORD = 0x4000;
const DMLERR_DATAACKTIMEOUT: DWORD = 0x4002;
const DMLERR_EXECACKTIMEOUT: DWORD = 0x4005;
const DMLERR_INVALIDPARAMETER: DWORD = 0x4006;
const DMLERR_MEMORY_ERROR: DWORD = 0x4008;
const DMLERR_NOTPROCESSED: DWORD = 0x4009;
const DMLERR_NO_CONV_ESTABLISHED: DWORD = 0x400a;
const DMLERR_POKEACKTIMEOUT: DWORD = 0x400b;
const DMLERR_POSTMSG_FAILED: DWORD = 0x400c;
const DMLERR_UNADVACKTIMEOUT: DWORD = 0x4010;

// DDEML handle types (ddeml.h) — opaque pointer-width handles.
type HSZ = HANDLE;
type HCONV = HANDLE;
type HCONVLIST = HANDLE;
type HDDEDATA = HANDLE;
type PCONVCONTEXT = *mut CONVCONTEXT;

type PFNCALLBACK = HDDEDATA; // not used in this TU; placeholder for symmetry

// ────────────────────────────────────────────────────────────────────────────
// DDEML structs (ddeml.h / dde.h) — reproduced #[repr(C)] to faithful layout.
// ────────────────────────────────────────────────────────────────────────────

// CONVCONTEXT (ddeml.h)

// WINE_DDEHEAD (dde_private.h) — bitfield pack of a DDE??? header word.
//   unsigned short unused:12, fResponse:1, fRelease:1, fDeferUpd:1, fAckReq:1;
//   short cfFormat;
impl WINE_DDEHEAD {
    #[inline]
    fn fResponse(&self) -> bool {
        (self.flags & 0x1000) != 0
    }
    #[inline]
    fn fRelease(&self) -> bool {
        (self.flags & 0x2000) != 0
    }
    #[inline]
    fn fDeferUpd(&self) -> bool {
        (self.flags & 0x4000) != 0
    }
    #[inline]
    fn fAckReq(&self) -> bool {
        (self.flags & 0x8000) != 0
    }
}

// DDEACK (dde.h): unsigned short bAppReturnCode:8, reserved:6, fBusy:1, fAck:1;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DDEACK {
    pub raw: u16,
}
impl DDEACK {
    #[inline]
    fn fAck(&self) -> bool {
        (self.raw & 0x8000) != 0
    }
}

// DDEADVISE (dde.h): unsigned short reserved:14, fDeferUpd:1, fAckReq:1; short cfFormat;
#[repr(C)]
struct DDEADVISE {
    flags: u16,
    cfFormat: i16,
}

// DDEPOKE (dde.h): unsigned short unused:13, fRelease:1, fReserved:2; short cfFormat; BYTE Value[1];
#[repr(C)]
struct DDEPOKE {
    flags: u16,
    cfFormat: i16,
    Value: [u8; 1],
}

// DDE_DATAHANDLE_HEAD (dde_private.h)

// ────────────────────────────────────────────────────────────────────────────
// Shared WDML_* internal structs (dde_private.h) — #[repr(C)], faithful layout.
// These are shared with ddemisc.c / ddeserver.c.
// ────────────────────────────────────────────────────────────────────────────

const WDML_CLIENT_SIDE: usize = 0;
const WDML_SERVER_SIDE: usize = 1;

// WDML_QUEUE_STATE enum (dde_private.h)
type WDML_QUEUE_STATE = INT;
const WDML_QS_ERROR: WDML_QUEUE_STATE = 0;
const WDML_QS_HANDLED: WDML_QUEUE_STATE = 1;
const WDML_QS_PASS: WDML_QUEUE_STATE = 2;
const WDML_QS_SWALLOWED: WDML_QUEUE_STATE = 3;
const WDML_QS_BLOCK: WDML_QUEUE_STATE = 4;

// The dde_private.h structs are ddemisc's: one definition, so the ddemisc functions
// this module calls take the same types (a byte-identical copy here was a distinct type).
pub use crate::misc::ddemisc::{CONVCONTEXT, DDE_DATAHANDLE_HEAD, HSZNode, WDML_CONV, WDML_INSTANCE, WDML_LINK, WDML_SERVER, WDML_XACT, WINE_DDEHEAD};

// ────────────────────────────────────────────────────────────────────────────
// Class-name globals.
//   WDML_szClientConvClassA[] = "DDEMLAnsiClient"
//   WDML_szClientConvClassW[] = L"DDEMLUnicodeClient"
// Non-static → exported. Cross-file server class names live in ddeserver.c.
// ────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub static WDML_szClientConvClassA: [u8; 16] =
    *b"DDEMLAnsiClient\0";

#[no_mangle]
pub static WDML_szClientConvClassW: [u16; 19] = [
    b'D' as u16, b'D' as u16, b'E' as u16, b'M' as u16, b'L' as u16,
    b'U' as u16, b'n' as u16, b'i' as u16, b'c' as u16, b'o' as u16,
    b'd' as u16, b'e' as u16, b'C' as u16, b'l' as u16, b'i' as u16,
    b'e' as u16, b'n' as u16, b't' as u16, 0,
];

// ────────────────────────────────────────────────────────────────────────────
// Cross-file WDML_* helpers (defined in ddemisc.c / ddeserver.c) — plain cdecl.
// ────────────────────────────────────────────────────────────────────────────
extern "C" {
    fn WDML_FindConv(
        pInstance: *mut WDML_INSTANCE,
        side: INT,
        hszService: HSZ,
        hszTopic: HSZ,
    ) -> *mut WDML_CONV;
    fn WDML_MakeAtomFromHsz(hsz: HSZ) -> ATOM;
    fn WDML_UnQueueTransaction(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> BOOL;
    fn WDML_Global2DataHandle(
        pConv: *mut WDML_CONV,
        hMem: HGLOBAL,
        da: *mut WINE_DDEHEAD,
    ) -> HDDEDATA;
}

// Cross-file server-side conversation class-name globals (defined in ddeserver.c).
extern "C" {
    static WDML_szServerConvClassA: [u8; 0];
    static WDML_szServerConvClassW: [u16; 0];
}

// ────────────────────────────────────────────────────────────────────────────
// DDEML / user32 / kernel32 public APIs we call (WINAPI → extern "system").
// PackDDElParam/UnpackDDElParam/FreeDDElParam/Dde* are defined in ddemisc.c.
// ────────────────────────────────────────────────────────────────────────────
extern "system" {
    fn PackDDElParam(msg: UINT, uiLo: UINT_PTR, uiHi: UINT_PTR) -> LPARAM;
    fn UnpackDDElParam(msg: UINT, lParam: LPARAM, uiLo: *mut UINT_PTR, uiHi: *mut UINT_PTR)
        -> BOOL;
    fn FreeDDElParam(msg: UINT, lParam: LPARAM) -> BOOL;
    fn DdeCmpStringHandles(hsz1: HSZ, hsz2: HSZ) -> INT;
    fn DdeAccessData(hData: HDDEDATA, pcbDataSize: *mut DWORD) -> *mut u8;
    fn DdeUnaccessData(hData: HDDEDATA) -> BOOL;
    fn DdeFreeDataHandle(hData: HDDEDATA) -> BOOL;
    fn ImpersonateDdeClientWindow(hWndClient: HWND, hWndServer: HWND) -> BOOL;

    // user32 public window/message exports
    fn RegisterClassExW(lpwcx: *const _WNDCLASSEXW) -> ATOM;
    fn RegisterClassExA(lpwcx: *const _WNDCLASSEXA) -> ATOM;
    fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: LPCWSTR,
        lpWindowName: LPCWSTR,
        dwStyle: DWORD,
        x: INT,
        y: INT,
        nWidth: INT,
        nHeight: INT,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn CreateWindowExA(
        dwExStyle: DWORD,
        lpClassName: LPCSTR,
        lpWindowName: LPCSTR,
        dwStyle: DWORD,
        x: INT,
        y: INT,
        nWidth: INT,
        nHeight: INT,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: LPVOID,
    ) -> HWND;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendMessageTimeoutW(
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        fuFlags: UINT,
        uTimeout: UINT,
        lpdwResult: *mut DWORD_PTR,
    ) -> LRESULT;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn PeekMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn IsWindowUnicode(hWnd: HWND) -> BOOL;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetClassNameA(hWnd: HWND, lpClassName: LPSTR, nMaxCount: INT) -> INT;
    fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    fn lstrcmpiA(lpString1: LPCSTR, lpString2: LPCSTR) -> INT;
    fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT;

    // kernel32 exports
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
    fn GlobalDeleteAtom(nAtom: ATOM) -> ATOM;
    fn GetLastError() -> DWORD;
    fn GetTickCount() -> DWORD;
    fn MsgWaitForMultipleObjects(
        nCount: DWORD,
        pHandles: *const HANDLE,
        fWaitAll: BOOL,
        dwMilliseconds: DWORD,
        dwWakeMask: DWORD,
    ) -> DWORD;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: *mut INT,
    ) -> INT;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
}

// ────────────────────────────────────────────────────────────────────────────
// Helper macros (winuser.h / windef.h) reproduced as inline fns.
//   WIN_GetFullHandle(h) == (HWND)(h)  — identity cast (user_x.h).
//   MAKELPARAM(l,h)  MAKELONG(l,h)
// ────────────────────────────────────────────────────────────────────────────
#[inline]
unsafe fn WIN_GetFullHandle(h: HWND) -> HWND {
    h
}
#[inline]
fn MAKELONG(lo: UINT, hi: UINT) -> LONG {
    (((lo & 0xffff) | ((hi & 0xffff) << 16)) as u32) as LONG
}
#[inline]
fn MAKELPARAM(lo: UINT, hi: UINT) -> LPARAM {
    MAKELONG(lo, hi) as LPARAM
}

// dde_private.h: static __inline void WDML_ExtractAck(WORD status, DDEACK* da)
//   *da = *((DDEACK*)&status);  — reinterpret the WORD as DDEACK.
#[inline]
unsafe fn WDML_ExtractAck(status: WORD, da: *mut DDEACK) {
    (*da).raw = status;
}

// ────────────────────────────────────────────────────────────────────────────
// static LRESULT CALLBACK WDML_ClientProc — forward declaration only via Rust fn.
// ────────────────────────────────────────────────────────────────────────────

// ============================================================================
// DdeConnectList [USER32.@]  (stub in ReactOS)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeConnectList(
    _idInst: DWORD,
    _hszService: HSZ,
    _hszTopic: HSZ,
    _hConvList: HCONVLIST,
    _pCC: PCONVCONTEXT,
) -> HCONVLIST {
    1 as HCONVLIST
}

// ============================================================================
// DdeQueryNextServer [USER32.@]  (stub in ReactOS)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeQueryNextServer(
    _hConvList: HCONVLIST,
    _hConvPrev: HCONV,
) -> HCONV {
    core::ptr::null_mut()
}

// ============================================================================
// DdeDisconnectList [USER32.@]  (stub in ReactOS)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeDisconnectList(_hConvList: HCONVLIST) -> BOOL {
    1
}

// ============================================================================
// DdeConnect  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeConnect(
    idInst: DWORD,
    hszService: HSZ,
    hszTopic: HSZ,
    pCC: PCONVCONTEXT,
) -> HCONV {
    let hwndClient: HWND;
    let mut pInstance: *mut WDML_INSTANCE;
    let mut pConv: *mut WDML_CONV;
    let mut aSrv: ATOM = 0;
    let mut aTpc: ATOM = 0;

    pInstance = (WDML_GetInstance(idInst)).cast();
    if pInstance.is_null() {
        return core::ptr::null_mut();
    }

    /* make sure this conv is never created */
    pConv = WDML_FindConv(pInstance, WDML_CLIENT_SIDE as INT, hszService, hszTopic);
    if !pConv.is_null() {
        return core::ptr::null_mut();
    }

    /* we need to establish a conversation with server, so create a window for it */
    if (*pInstance).unicode != 0 {
        let mut wndclass: _WNDCLASSEXW = core::mem::zeroed();

        wndclass.cbSize = core::mem::size_of::<_WNDCLASSEXW>() as UINT;
        wndclass.style = 0;
        wndclass.lpfnWndProc = core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(WDML_ClientProc);
        wndclass.cbClsExtra = 0;
        wndclass.cbWndExtra = (2 * core::mem::size_of::<ULONG_PTR>()) as INT;
        wndclass.hInstance = core::ptr::null_mut();
        wndclass.hIcon = core::ptr::null_mut();
        wndclass.hCursor = core::ptr::null_mut();
        wndclass.hbrBackground = core::ptr::null_mut();
        wndclass.lpszMenuName = core::ptr::null_mut();
        wndclass.lpszClassName = WDML_szClientConvClassW.as_ptr();
        wndclass.hIconSm = core::ptr::null_mut();

        RegisterClassExW(&wndclass);

        hwndClient = CreateWindowExW(
            0,
            WDML_szClientConvClassW.as_ptr(),
            core::ptr::null(),
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
    } else {
        let mut wndclass: _WNDCLASSEXA = core::mem::zeroed();

        wndclass.cbSize = core::mem::size_of::<_WNDCLASSEXA>() as UINT;
        wndclass.style = 0;
        wndclass.lpfnWndProc = core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(WDML_ClientProc);
        wndclass.cbClsExtra = 0;
        wndclass.cbWndExtra = (2 * core::mem::size_of::<ULONG_PTR>()) as INT;
        wndclass.hInstance = core::ptr::null_mut();
        wndclass.hIcon = core::ptr::null_mut();
        wndclass.hCursor = core::ptr::null_mut();
        wndclass.hbrBackground = core::ptr::null_mut();
        wndclass.lpszMenuName = core::ptr::null_mut();
        wndclass.lpszClassName = WDML_szClientConvClassA.as_ptr() as LPCSTR;
        wndclass.hIconSm = core::ptr::null_mut();

        RegisterClassExA(&wndclass);

        hwndClient = CreateWindowExA(
            0,
            WDML_szClientConvClassA.as_ptr() as LPCSTR,
            core::ptr::null(),
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
    }

    SetWindowLongPtrW(hwndClient, GWL_WDML_INSTANCE, pInstance as LONG_PTR);

    if !hszService.is_null() {
        aSrv = WDML_MakeAtomFromHsz(hszService);
        if aSrv == 0 {
            return theEnd(pConv, aSrv, aTpc);
        }
    }
    if !hszTopic.is_null() {
        aTpc = WDML_MakeAtomFromHsz(hszTopic);
        if aTpc == 0 {
            return theEnd(pConv, aSrv, aTpc);
        }
    }

    /* note: sent messages shall not use packing */
    SendMessageTimeoutW(
        HWND_BROADCAST,
        WM_DDE_INITIATE,
        hwndClient as WPARAM,
        MAKELPARAM(aSrv as UINT, aTpc as UINT),
        SMTO_ABORTIFHUNG,
        0,
        core::ptr::null_mut(),
    );

    pInstance = (WDML_GetInstance(idInst)).cast();
    if pInstance.is_null() {
        return theEnd(pConv, aSrv, aTpc);
    }

    /* At this point, Client WM_DDE_ACK should have saved hwndServer ... */
    pConv = (WDML_GetConvFromWnd(hwndClient)).cast();
    if pConv.is_null() || (*pConv).hwndServer.is_null() {
        pConv = core::ptr::null_mut();
        (*pInstance).lastError = DMLERR_NO_CONV_ESTABLISHED;
        return theEnd(pConv, aSrv, aTpc);
    }
    (*pConv).wConvst = XST_CONNECTED;

    /* finish init of pConv */
    if !pCC.is_null() {
        (*pConv).convContext = *pCC;
    } else {
        (*pConv).convContext = core::mem::zeroed();
        (*pConv).convContext.cb = core::mem::size_of::<CONVCONTEXT>() as UINT;
        (*pConv).convContext.iCodePage = if (*pInstance).unicode != 0 {
            CP_WINUNICODE
        } else {
            CP_WINANSI
        };
    }

    theEnd(pConv, aSrv, aTpc)
}

#[inline]
unsafe fn theEnd(pConv: *mut WDML_CONV, aSrv: ATOM, aTpc: ATOM) -> HCONV {
    if aSrv != 0 {
        GlobalDeleteAtom(aSrv);
    }
    if aTpc != 0 {
        GlobalDeleteAtom(aTpc);
    }
    pConv as HCONV
}

// ============================================================================
// DdeReconnect  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeReconnect(hConv: HCONV) -> HCONV {
    let mut pConv: *mut WDML_CONV;
    let mut pNewConv: *mut WDML_CONV = core::ptr::null_mut();
    let mut aSrv: ATOM = 0;
    let mut aTpc: ATOM = 0;

    pConv = (WDML_GetConv(hConv, 0)).cast();
    if !pConv.is_null() && ((*pConv).wStatus & ST_CLIENT) != 0 {
        let ret: BOOL;

        /* to reestablish a connection ... */
        if pConv == (WDML_GetConvFromWnd((*pConv).hwndClient)).cast()
            && ((*pConv).wStatus & ST_TERMINATED) != 0
            && ((*pConv).wStatus & ST_CONNECTED) == 0
        {
            let hwndClient: HWND = (*pConv).hwndClient;
            let hwndServer: HWND = (*pConv).hwndServer;

            SetWindowLongPtrW((*pConv).hwndClient, GWL_WDML_CONVERSATION, 0);

            aSrv = WDML_MakeAtomFromHsz((*pConv).hszService);
            aTpc = WDML_MakeAtomFromHsz((*pConv).hszTopic);
            if aSrv == 0 || aTpc == 0 {
                return reconnectEnd(pNewConv, aSrv, aTpc);
            }

            /* note: sent messages shall not use packing */
            ret = SendMessageW(
                hwndServer,
                WM_DDE_INITIATE,
                hwndClient as WPARAM,
                MAKELPARAM(aSrv as UINT, aTpc as UINT),
            ) as BOOL;

            pConv = (WDML_GetConv(hConv, 0)).cast();
            if pConv.is_null() {
                return reconnectEnd(pNewConv, aSrv, aTpc);
            }

            pNewConv = (WDML_GetConvFromWnd((*pConv).hwndClient)).cast();
            if ret != 0 && !pNewConv.is_null() {
                /* re-establish all links... */
                let mut pLink: *mut WDML_LINK;

                pLink = (*(*pConv).instance).links[WDML_CLIENT_SIDE];
                while !pLink.is_null() {
                    if (*pLink).hConv == hConv {
                        /* try to reestablish the links... */
                        DdeClientTransaction(
                            core::ptr::null_mut(),
                            0,
                            pNewConv as HCONV,
                            (*pLink).hszItem,
                            (*pLink).uFmt,
                            (*pLink).transactionType,
                            1000,
                            core::ptr::null_mut(),
                        );
                    }
                    pLink = (*pLink).next;
                }
            } else {
                /* reset the conversation as it was */
                SetWindowLongPtrW(
                    (*pConv).hwndClient,
                    GWL_WDML_CONVERSATION,
                    pConv as LONG_PTR,
                );
            }
        }
    }

    reconnectEnd(pNewConv, aSrv, aTpc)
}

#[inline]
unsafe fn reconnectEnd(pNewConv: *mut WDML_CONV, aSrv: ATOM, aTpc: ATOM) -> HCONV {
    if aSrv != 0 {
        GlobalDeleteAtom(aSrv);
    }
    if aTpc != 0 {
        GlobalDeleteAtom(aTpc);
    }
    pNewConv as HCONV
}

// ============================================================================
// WDML_ClientQueueAdvise  (static)
// ============================================================================
unsafe fn WDML_ClientQueueAdvise(
    pConv: *mut WDML_CONV,
    wType: UINT,
    wFmt: UINT,
    hszItem: HSZ,
) -> *mut WDML_XACT {
    let pDdeAdvise: *mut DDEADVISE;
    let pXAct: *mut WDML_XACT;
    let atom: ATOM;

    atom = WDML_MakeAtomFromHsz(hszItem);
    if atom == 0 {
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_ADVISE, wFmt, hszItem)).cast();
    if pXAct.is_null() {
        GlobalDeleteAtom(atom);
        return core::ptr::null_mut();
    }

    (*pXAct).wType = wType & !0x0F;
    (*pXAct).hMem = GlobalAlloc(GHND | GMEM_DDESHARE, core::mem::size_of::<DDEADVISE>() as SIZE_T);
    /* FIXME: hMem is unfreed for now... should be deleted in server */

    /* pack DdeAdvise */
    pDdeAdvise = GlobalLock((*pXAct).hMem) as *mut DDEADVISE;
    // DDEADVISE bitfields: reserved:14, fDeferUpd:1, fAckReq:1
    //   fDeferUpd -> bit 14 (0x4000), fAckReq -> bit 15 (0x8000)
    let mut flags: u16 = 0;
    if (wType & XTYPF_ACKREQ) != 0 {
        flags |= 0x8000;
    }
    if (wType & XTYPF_NODATA) != 0 {
        flags |= 0x4000;
    }
    (*pDdeAdvise).flags = flags;
    (*pDdeAdvise).cfFormat = wFmt as i16;
    GlobalUnlock((*pXAct).hMem);

    (*pXAct).lParam = PackDDElParam(WM_DDE_ADVISE, (*pXAct).hMem as UINT_PTR, atom as UINT_PTR);

    pXAct
}

// ============================================================================
// WDML_HandleAdviseReply  (static)
// ============================================================================
unsafe fn WDML_HandleAdviseReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    pXAct: *mut WDML_XACT,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let mut ddeAck: DDEACK = core::mem::zeroed();
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let hsz: HSZ;

    if (*msg).message != WM_DDE_ACK
        || WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer
    {
        return WDML_QS_PASS;
    }

    UnpackDDElParam(WM_DDE_ACK, (*msg).lParam, &mut uiLo, &mut uiHi);
    hsz = WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM);

    if DdeCmpStringHandles(hsz, (*pXAct).hszItem) != 0 {
        return WDML_QS_PASS;
    }

    GlobalDeleteAtom(uiHi as ATOM);
    FreeDDElParam(WM_DDE_ACK, (*msg).lParam);

    if !ack.is_null() {
        *ack = uiLo as DWORD;
    }
    WDML_ExtractAck(uiLo as WORD, &mut ddeAck);

    if ddeAck.fAck() {
        let pLink: *mut WDML_LINK;

        /* billx: first to see if the link is already created. */
        pLink = (WDML_FindLink(
            ((*pConv).instance).cast(),
            pConv as HCONV,
            WDML_CLIENT_SIDE as INT,
            (*pXAct).hszItem,
            1,
            (*pXAct).wFmt,
        )).cast();
        if !pLink.is_null() {
            (*pLink).transactionType = (*pXAct).wType;
        } else {
            WDML_AddLink(
                ((*pConv).instance).cast(),
                pConv as HCONV,
                WDML_CLIENT_SIDE as INT,
                (*pXAct).wType,
                (*pXAct).hszItem,
                (*pXAct).wFmt,
            );
        }
        (*pXAct).hDdeData = 1 as HDDEDATA;
    } else {
        GlobalFree((*pXAct).hMem);
        (*pXAct).hDdeData = core::ptr::null_mut();
    }

    WDML_QS_HANDLED
}

// ============================================================================
// WDML_ClientQueueUnadvise  (static)
// ============================================================================
unsafe fn WDML_ClientQueueUnadvise(
    pConv: *mut WDML_CONV,
    wFmt: UINT,
    hszItem: HSZ,
) -> *mut WDML_XACT {
    let pXAct: *mut WDML_XACT;
    let atom: ATOM;

    atom = WDML_MakeAtomFromHsz(hszItem);
    if atom == 0 {
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_UNADVISE, wFmt, hszItem)).cast();
    if pXAct.is_null() {
        GlobalDeleteAtom(atom);
        return core::ptr::null_mut();
    }

    (*pXAct).lParam = PackDDElParam(WM_DDE_UNADVISE, wFmt as UINT_PTR, atom as UINT_PTR);
    pXAct
}

// ============================================================================
// WDML_HandleUnadviseReply  (static)
// ============================================================================
unsafe fn WDML_HandleUnadviseReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    pXAct: *mut WDML_XACT,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let mut ddeAck: DDEACK = core::mem::zeroed();
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let hsz: HSZ;

    if (*msg).message != WM_DDE_ACK
        || WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer
    {
        return WDML_QS_PASS;
    }

    UnpackDDElParam(WM_DDE_ACK, (*msg).lParam, &mut uiLo, &mut uiHi);
    hsz = WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM);

    if DdeCmpStringHandles(hsz, (*pXAct).hszItem) != 0 {
        return WDML_QS_PASS;
    }

    FreeDDElParam(WM_DDE_ACK, (*msg).lParam);
    GlobalDeleteAtom(uiHi as ATOM);

    if !ack.is_null() {
        *ack = uiLo as DWORD;
    }
    WDML_ExtractAck(uiLo as WORD, &mut ddeAck);

    if !ddeAck.fAck() {
        (*pXAct).hDdeData = core::ptr::null_mut();
    } else {
        /* billx: remove the link */
        WDML_RemoveLink(
            ((*pConv).instance).cast(),
            pConv as HCONV,
            WDML_CLIENT_SIDE as INT,
            (*pXAct).hszItem,
            (*pXAct).wFmt,
        );
        (*pXAct).hDdeData = 1 as HDDEDATA;
    }
    WDML_QS_HANDLED
}

// ============================================================================
// WDML_ClientQueueRequest  (static)
// ============================================================================
unsafe fn WDML_ClientQueueRequest(
    pConv: *mut WDML_CONV,
    wFmt: UINT,
    hszItem: HSZ,
) -> *mut WDML_XACT {
    let pXAct: *mut WDML_XACT;
    let atom: ATOM;

    atom = WDML_MakeAtomFromHsz(hszItem);
    if atom == 0 {
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_REQUEST, wFmt, hszItem)).cast();
    if pXAct.is_null() {
        GlobalDeleteAtom(atom);
        return core::ptr::null_mut();
    }

    (*pXAct).lParam = PackDDElParam(WM_DDE_REQUEST, wFmt as UINT_PTR, atom as UINT_PTR);

    pXAct
}

// ============================================================================
// WDML_HandleRequestReply  (static)
// ============================================================================
unsafe fn WDML_HandleRequestReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    pXAct: *mut WDML_XACT,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let mut ddeAck: DDEACK = core::mem::zeroed();
    let mut wdh: WINE_DDEHEAD = core::mem::zeroed();
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let hsz: HSZ;

    if WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer {
        return WDML_QS_PASS;
    }

    match (*msg).message {
        WM_DDE_ACK => {
            UnpackDDElParam(WM_DDE_ACK, (*msg).lParam, &mut uiLo, &mut uiHi);
            FreeDDElParam(WM_DDE_ACK, (*msg).lParam);
            GlobalDeleteAtom(uiHi as ATOM);
            if !ack.is_null() {
                *ack = uiLo as DWORD;
            }
            WDML_ExtractAck(uiLo as WORD, &mut ddeAck);
            (*pXAct).hDdeData = core::ptr::null_mut();
            /* ddeAck.fAck positive answer warning omitted (TRACE/ERR only) */
            let _ = ddeAck.fAck();
        }

        WM_DDE_DATA => {
            UnpackDDElParam(WM_DDE_DATA, (*msg).lParam, &mut uiLo, &mut uiHi);

            hsz = WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM);

            if DdeCmpStringHandles(hsz, (*pXAct).hszItem) != 0 {
                return WDML_QS_PASS;
            }

            (*pXAct).hDdeData = WDML_Global2DataHandle(pConv, uiLo as HGLOBAL, &mut wdh);
            if wdh.fRelease() {
                GlobalFree(uiLo as HGLOBAL);
            }
            if wdh.fAckReq() {
                (*(*pConv).instance).lastError = DMLERR_MEMORY_ERROR;
            } else {
                GlobalDeleteAtom(uiHi as ATOM);
                FreeDDElParam(WM_DDE_ACK, (*msg).lParam);
            }
        }

        _ => {
            FreeDDElParam((*msg).message, (*msg).lParam);
            return WDML_QS_PASS;
        }
    }

    WDML_QS_HANDLED
}

// ============================================================================
// WDML_BuildExecuteCommand  (static)
// ============================================================================
unsafe fn WDML_BuildExecuteCommand(
    pConv: *mut WDML_CONV,
    pData: LPCVOID,
    cbData: DWORD,
) -> HGLOBAL {
    let hMem: HGLOBAL;
    let clientUnicode: BOOL;
    let serverUnicode: BOOL;
    let memSize: DWORD;

    clientUnicode = (*(*pConv).instance).unicode;
    /* FIXME: how exactly Windows determines what to use for the server side? */
    serverUnicode = (IsWindowUnicode((*pConv).hwndServer) != 0
        && IsWindowUnicode((*pConv).hwndClient) != 0) as BOOL;

    if clientUnicode == serverUnicode {
        memSize = cbData;
    } else if clientUnicode != 0 {
        memSize = WideCharToMultiByte(
            CP_ACP,
            0,
            pData as LPCWSTR,
            (cbData / core::mem::size_of::<WCHAR>() as DWORD) as INT,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as DWORD;
    } else {
        memSize = (MultiByteToWideChar(
            CP_ACP,
            0,
            pData as LPCSTR,
            cbData as INT,
            core::ptr::null_mut(),
            0,
        ) as DWORD)
            * core::mem::size_of::<WCHAR>() as DWORD;
    }

    hMem = GlobalAlloc(GMEM_MOVEABLE | GMEM_DDESHARE, memSize as SIZE_T);

    if !hMem.is_null() {
        let pDst: LPSTR;

        pDst = GlobalLock(hMem) as LPSTR;
        if !pDst.is_null() {
            if clientUnicode == serverUnicode {
                core::ptr::copy_nonoverlapping(
                    pData as *const u8,
                    pDst as *mut u8,
                    cbData as usize,
                );
            } else if clientUnicode != 0 {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    pData as LPCWSTR,
                    (cbData / core::mem::size_of::<WCHAR>() as DWORD) as INT,
                    pDst,
                    memSize as INT,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            } else {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    pData as LPCSTR,
                    cbData as INT,
                    pDst as LPWSTR,
                    (memSize / core::mem::size_of::<WCHAR>() as DWORD) as INT,
                );
            }

            GlobalUnlock(hMem);
            return hMem;
        } else {
            GlobalFree(hMem);
            return core::ptr::null_mut();
        }
    }
    hMem
}

// ============================================================================
// WDML_ClientQueueExecute  (static)
// ============================================================================
unsafe fn WDML_ClientQueueExecute(
    pConv: *mut WDML_CONV,
    mut pData: LPVOID,
    mut cbData: DWORD,
) -> *mut WDML_XACT {
    let pXAct: *mut WDML_XACT;

    if pData.is_null() {
        if cbData == (-1i32) as DWORD {
            (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
        } else {
            (*(*pConv).instance).lastError = DMLERR_MEMORY_ERROR;
        }
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_EXECUTE, 0, core::ptr::null_mut())).cast();
    if pXAct.is_null() {
        return core::ptr::null_mut();
    }

    if cbData == (-1i32) as DWORD {
        let hDdeData: HDDEDATA = pData as HDDEDATA;

        pData = DdeAccessData(hDdeData, &mut cbData) as LPVOID;
        if !pData.is_null() {
            (*pXAct).hMem = WDML_BuildExecuteCommand(pConv, pData as LPCVOID, cbData);
            DdeUnaccessData(hDdeData);
        }
    } else {
        (*pXAct).hMem = WDML_BuildExecuteCommand(pConv, pData as LPCVOID, cbData);
    }

    (*pXAct).lParam = (*pXAct).hMem as LPARAM;

    pXAct
}

// ============================================================================
// WDML_HandleExecuteReply  (static)
// ============================================================================
unsafe fn WDML_HandleExecuteReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    pXAct: *mut WDML_XACT,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let mut ddeAck: DDEACK = core::mem::zeroed();
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;

    if (*msg).message != WM_DDE_ACK
        || WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer
    {
        return WDML_QS_PASS;
    }

    UnpackDDElParam(WM_DDE_ACK, (*msg).lParam, &mut uiLo, &mut uiHi);
    FreeDDElParam(WM_DDE_ACK, (*msg).lParam);

    if (uiHi as HANDLE) != (*pXAct).hMem {
        return WDML_QS_PASS;
    }

    if !ack.is_null() {
        *ack = uiLo as DWORD;
    }
    WDML_ExtractAck(uiLo as WORD, &mut ddeAck);
    (*pXAct).hDdeData = (ddeAck.fAck() as UINT_PTR) as HDDEDATA;

    (*(*pConv).instance).lastError = if !(*pXAct).hDdeData.is_null() {
        DMLERR_NO_ERROR
    } else {
        DMLERR_NOTPROCESSED
    };

    WDML_QS_HANDLED
}

// ============================================================================
// WDML_ClientQueuePoke  (static)
// ============================================================================
unsafe fn WDML_ClientQueuePoke(
    pConv: *mut WDML_CONV,
    mut pData: LPVOID,
    mut cbData: DWORD,
    wFmt: UINT,
    hszItem: HSZ,
) -> *mut WDML_XACT {
    let dh: *mut DDE_DATAHANDLE_HEAD;
    let pXAct: *mut WDML_XACT;
    let ddePoke: *mut DDEPOKE;
    let hglobal: HGLOBAL;
    let atom: ATOM;

    atom = WDML_MakeAtomFromHsz(hszItem);
    if atom == 0 {
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_POKE, wFmt, hszItem)).cast();
    if pXAct.is_null() {
        GlobalDeleteAtom(atom);
        return core::ptr::null_mut();
    }

    if cbData == (-1i32) as DWORD {
        hglobal = pData as HGLOBAL;
        dh = GlobalLock(hglobal) as *mut DDE_DATAHANDLE_HEAD;
        cbData =
            (GlobalSize(hglobal) - core::mem::size_of::<DDE_DATAHANDLE_HEAD>() as SIZE_T) as DWORD;
        pData = dh.add(1) as LPVOID;
        GlobalUnlock(hglobal);
    }

    // FIELD_OFFSET(DDEPOKE, Value[cbData]) = offsetof(DDEPOKE, Value) + cbData
    let value_off = core::mem::offset_of!(DDEPOKE, Value);
    (*pXAct).hMem = GlobalAlloc(GHND | GMEM_DDESHARE, (value_off + cbData as usize) as SIZE_T);
    ddePoke = GlobalLock((*pXAct).hMem) as *mut DDEPOKE;
    if ddePoke.is_null() {
        (*(*pConv).instance).lastError = DMLERR_MEMORY_ERROR;
        return core::ptr::null_mut();
    }

    // DDEPOKE bitfields: unused:13, fRelease:1, fReserved:2
    //   fRelease -> bit 13 (0x2000). C sets unused=0, fRelease=TRUE.
    (*ddePoke).flags = 0x2000;
    (*ddePoke).cfFormat = wFmt as i16;
    core::ptr::copy_nonoverlapping(
        pData as *const u8,
        (*ddePoke).Value.as_mut_ptr(),
        cbData as usize,
    );
    GlobalUnlock((*pXAct).hMem);

    (*pXAct).lParam = PackDDElParam(WM_DDE_POKE, (*pXAct).hMem as UINT_PTR, atom as UINT_PTR);

    pXAct
}

// ============================================================================
// WDML_HandlePokeReply  (static)
// ============================================================================
unsafe fn WDML_HandlePokeReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    pXAct: *mut WDML_XACT,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let hsz: HSZ;

    if (*msg).message != WM_DDE_ACK
        && WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer
    {
        return WDML_QS_PASS;
    }

    UnpackDDElParam(WM_DDE_ACK, (*msg).lParam, &mut uiLo, &mut uiHi);
    hsz = WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM);
    if DdeCmpStringHandles(hsz, (*pXAct).hszItem) != 0 {
        return WDML_QS_PASS;
    }
    FreeDDElParam(WM_DDE_ACK, (*msg).lParam);
    GlobalDeleteAtom(uiHi as ATOM);

    if !ack.is_null() {
        *ack = uiLo as DWORD;
    }
    (*pXAct).hMem = GlobalFree((*pXAct).hMem);

    (*pXAct).hDdeData = 1 as HDDEDATA; // (HDDEDATA)TRUE
    // C returns TRUE here (== WDML_QS_HANDLED == 1)
    WDML_QS_HANDLED
}

// ============================================================================
// WDML_ClientQueueTerminate  (static)
// ============================================================================
unsafe fn WDML_ClientQueueTerminate(pConv: *mut WDML_CONV) -> *mut WDML_XACT {
    let pXAct: *mut WDML_XACT;

    pXAct =
        (WDML_AllocTransaction(((*pConv).instance).cast(), WM_DDE_TERMINATE, 0, core::ptr::null_mut())).cast();
    if pXAct.is_null() {
        return core::ptr::null_mut();
    }

    (*pXAct).lParam = 0;
    (*pConv).wStatus &= !ST_CONNECTED;

    pXAct
}

// ============================================================================
// WDML_HandleTerminateReply  (static)
// ============================================================================
unsafe fn WDML_HandleTerminateReply(pConv: *mut WDML_CONV, msg: *mut MSG) -> WDML_QUEUE_STATE {
    if (*msg).message != WM_DDE_TERMINATE {
        /* FIXME: should delete data passed here */
        return WDML_QS_SWALLOWED;
    }

    if WIN_GetFullHandle((*msg).wParam as HWND) != (*pConv).hwndServer {
        return WDML_QS_PASS;
    }
    if ((*(*pConv).instance).CBFflags & CBF_SKIP_DISCONNECTS) == 0 {
        WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_DISCONNECT,
            0,
            pConv as HCONV,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            if ((*pConv).wStatus & ST_ISSELF) != 0 { 1 } else { 0 },
        );
    }
    WDML_RemoveConv((pConv).cast(), WDML_CLIENT_SIDE as INT);
    WDML_QS_HANDLED
}

// ============================================================================
// WDML_HandleIncomingData  (static)
// ============================================================================
unsafe fn WDML_HandleIncomingData(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    _hdd: *mut HDDEDATA,
) -> WDML_QUEUE_STATE {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let hDdeDataIn: HDDEDATA;
    let hDdeDataOut: HDDEDATA;
    let pLink: *mut WDML_LINK;
    let mut wdh: WINE_DDEHEAD = core::mem::zeroed();
    let hsz: HSZ;

    /* wParam -- sending window handle; lParam -- hDdeData & item HSZ */
    UnpackDDElParam(WM_DDE_DATA, (*msg).lParam, &mut uiLo, &mut uiHi);
    hsz = WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM);

    hDdeDataIn = WDML_Global2DataHandle(pConv, uiLo as HGLOBAL, &mut wdh);

    /* billx: for hot link, data should be passed to its callback with XTYP_ADVDATA */
    pLink = (WDML_FindLink(
        ((*pConv).instance).cast(),
        pConv as HCONV,
        WDML_CLIENT_SIDE as INT,
        hsz,
        (uiLo != 0) as BOOL,
        wdh.cfFormat as UINT,
    )).cast();
    if pLink.is_null() {
        WDML_DecHSZ(((*pConv).instance).cast(), hsz);
        DdeFreeDataHandle(hDdeDataIn);
        return WDML_QS_PASS;
    }

    if !hDdeDataIn.is_null() && wdh.fAckReq() {
        WDML_PostAck(
            (pConv).cast(),
            WDML_CLIENT_SIDE as INT,
            0,
            0,
            1,
            uiHi,
            (*msg).lParam,
            WM_DDE_DATA,
        );
        if (*msg).lParam != 0 {
            (*msg).lParam = 0;
        }
    } else {
        GlobalDeleteAtom(uiHi as ATOM);
    }

    hDdeDataOut = WDML_InvokeCallback(
        ((*pConv).instance).cast(),
        XTYP_ADVDATA,
        (*pLink).uFmt,
        (*pLink).hConv,
        (*pConv).hszTopic,
        (*pLink).hszItem,
        hDdeDataIn,
        0,
        0,
    );

    if hDdeDataOut != (DDE_FACK as UINT_PTR) as HDDEDATA || wdh.fRelease() {
        if uiLo != 0 {
            GlobalFree(uiLo as HANDLE);
        }
    }

    DdeFreeDataHandle(hDdeDataIn);

    WDML_DecHSZ(((*pConv).instance).cast(), hsz);
    if (*msg).lParam != 0 {
        FreeDDElParam(WM_DDE_DATA, (*msg).lParam);
    }

    WDML_QS_HANDLED
}

// ============================================================================
// WDML_HandleIncomingTerminate  (static)
// ============================================================================
unsafe fn WDML_HandleIncomingTerminate(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    _hdd: *mut HDDEDATA,
) -> WDML_QUEUE_STATE {
    if (*pConv).hwndServer != WIN_GetFullHandle((*msg).wParam as HWND) {
        return WDML_QS_PASS;
    }

    (*pConv).wStatus |= ST_TERMINATED;
    if ((*pConv).wStatus & ST_CONNECTED) != 0 {
        /* don't care about result code (if server exists or not) */
        PostMessageW(
            (*pConv).hwndServer,
            WM_DDE_TERMINATE,
            (*pConv).hwndClient as WPARAM,
            0,
        );
        (*pConv).wStatus &= !ST_CONNECTED;
    }
    /* have to keep connection around to allow reconnection */
    WDML_QS_HANDLED
}

// ============================================================================
// WDML_HandleReply  (static)
// ============================================================================
unsafe fn WDML_HandleReply(
    pConv: *mut WDML_CONV,
    msg: *mut MSG,
    hdd: *mut HDDEDATA,
    ack: *mut DWORD,
) -> WDML_QUEUE_STATE {
    let pXAct: *mut WDML_XACT = (*pConv).transactions;
    let mut qs: WDML_QUEUE_STATE;

    if !(*pConv).transactions.is_null() {
        if !ack.is_null() {
            *ack = DDE_FNOTPROCESSED as DWORD;
        }
        /* first check message against a pending transaction, if any */
        qs = match (*pXAct).ddeMsg {
            WM_DDE_ADVISE => WDML_HandleAdviseReply(pConv, msg, pXAct, ack),
            WM_DDE_UNADVISE => WDML_HandleUnadviseReply(pConv, msg, pXAct, ack),
            WM_DDE_EXECUTE => WDML_HandleExecuteReply(pConv, msg, pXAct, ack),
            WM_DDE_REQUEST => WDML_HandleRequestReply(pConv, msg, pXAct, ack),
            WM_DDE_POKE => WDML_HandlePokeReply(pConv, msg, pXAct, ack),
            WM_DDE_TERMINATE => WDML_HandleTerminateReply(pConv, msg),
            _ => WDML_QS_ERROR,
        };
    } else {
        qs = WDML_QS_PASS;
    }

    /* now check the results */
    match qs {
        WDML_QS_ERROR | WDML_QS_SWALLOWED => {
            *hdd = core::ptr::null_mut();
        }
        WDML_QS_HANDLED => {
            /* ok, we have resolved a pending transaction; notify cb if asynchronous */
            if (*pXAct).dwTimeout == TIMEOUT_ASYNC && (*pXAct).ddeMsg != WM_DDE_TERMINATE {
                WDML_InvokeCallback(
                    ((*pConv).instance).cast(),
                    XTYP_XACT_COMPLETE,
                    (*pXAct).wFmt,
                    pConv as HCONV,
                    (*pConv).hszTopic,
                    (*pXAct).hszItem,
                    (*pXAct).hDdeData,
                    MAKELONG(0, (*pXAct).xActID) as ULONG_PTR,
                    0, /* FIXME */
                );
                qs = WDML_QS_PASS;
            } else {
                *hdd = (*pXAct).hDdeData;
            }
        }
        WDML_QS_PASS => {
            /* no pending transaction found, try a warm/hot link or a termination request */
            match (*msg).message {
                WM_DDE_DATA => {
                    qs = WDML_HandleIncomingData(pConv, msg, hdd);
                }
                WM_DDE_TERMINATE => {
                    qs = WDML_HandleIncomingTerminate(pConv, msg, hdd);
                }
                WM_DDE_ACK => {
                    /* end of DdeClientTransaction XTYP_EXECUTE */
                    *hdd = 1 as HDDEDATA; // (HDDEDATA)TRUE
                    if !ack.is_null() {
                        *ack = DDE_FACK as DWORD;
                    }
                }
                _ => {}
            }
        }
        WDML_QS_BLOCK => {
            /* FIXME: shouldn't be used on client side */
        }
        _ => {}
    }

    qs
}

// ============================================================================
// WDML_SyncWaitTransactionReply  (static)
// ============================================================================
unsafe fn WDML_SyncWaitTransactionReply(
    hConv: HCONV,
    dwTimeout: DWORD,
    pXAct: *const WDML_XACT,
    ack: *mut DWORD,
) -> HDDEDATA {
    let start: DWORD;
    let mut elapsed: DWORD;
    let err: DWORD;
    let mut pConv: *mut WDML_CONV;

    start = GetTickCount();
    loop {
        elapsed = GetTickCount().wrapping_sub(start);
        if elapsed >= dwTimeout {
            break;
        }

        /* we cannot be in the crit sect all the time ... */
        if MsgWaitForMultipleObjects(
            0,
            core::ptr::null(),
            0,
            dwTimeout - elapsed,
            QS_POSTMESSAGE,
        ) == WAIT_OBJECT_0
        {
            let mut msg: MSG = core::mem::zeroed();

            while PeekMessageW(
                &mut msg,
                core::ptr::null_mut(),
                WM_DDE_FIRST,
                WM_DDE_LAST,
                PM_REMOVE,
            ) != 0
            {
                let mut hdd: HDDEDATA = core::ptr::null_mut();

                pConv = (WDML_GetConv(hConv, 0)).cast();
                if pConv.is_null() {
                    /* conversation no longer available... return failure */
                    return core::ptr::null_mut();
                }
                if msg.hwnd == (*pConv).hwndClient {
                    /* check that either pXAct has been processed or no more xActions are pending */
                    let mut ret: BOOL = ((*pConv).transactions as *const WDML_XACT == pXAct) as BOOL;
                    if WDML_HandleReply(pConv, &mut msg, &mut hdd, ack) == WDML_QS_HANDLED {
                        ret = 1;
                    } else {
                        ret = ((*pConv).transactions.is_null() || ret != 0) as BOOL;
                    }

                    if ret != 0 {
                        (*(*pConv).instance).lastError = if !hdd.is_null() {
                            DMLERR_NO_ERROR
                        } else {
                            DMLERR_NOTPROCESSED
                        };
                        return hdd;
                    }
                } else {
                    DispatchMessageW(&msg);
                }
            }
        }
    }

    pConv = (WDML_GetConv(hConv, 0)).cast();
    if !pConv.is_null() && !(*pConv).transactions.is_null() {
        err = match (*(*pConv).transactions).ddeMsg {
            WM_DDE_ADVISE => DMLERR_ADVACKTIMEOUT,
            WM_DDE_REQUEST => DMLERR_DATAACKTIMEOUT,
            WM_DDE_EXECUTE => DMLERR_EXECACKTIMEOUT,
            WM_DDE_POKE => DMLERR_POKEACKTIMEOUT,
            WM_DDE_UNADVISE => DMLERR_UNADVACKTIMEOUT,
            _ => DMLERR_INVALIDPARAMETER,
        };

        (*(*pConv).instance).lastError = err;
    }

    core::ptr::null_mut()
}

// ============================================================================
// WDML_ClientHandle  — cross-file (plain, declared in dde_private.h) → extern "C"
// ============================================================================
#[no_mangle]
pub unsafe extern "C" fn WDML_ClientHandle(
    pConv: *mut WDML_CONV,
    pXAct: *mut WDML_XACT,
    dwTimeout: DWORD,
    pdwResult: *mut DWORD,
) -> HDDEDATA {
    let hDdeData: HDDEDATA;

    if PostMessageW(
        (*pConv).hwndServer,
        (*pXAct).ddeMsg,
        (*pConv).hwndClient as WPARAM,
        (*pXAct).lParam,
    ) == 0
    {
        let _ = GetLastError();
        (*pConv).wStatus &= !ST_CONNECTED;
        (*(*pConv).instance).lastError = DMLERR_POSTMSG_FAILED;
        return core::ptr::null_mut();
    }
    (*pXAct).dwTimeout = dwTimeout;
    /* FIXME: should set the app bits on *pdwResult */

    if dwTimeout == TIMEOUT_ASYNC {
        if !pdwResult.is_null() {
            *pdwResult = MAKELONG(0, (*pXAct).xActID) as DWORD;
        }

        hDdeData = 1 as HDDEDATA;
    } else {
        hDdeData = WDML_SyncWaitTransactionReply(pConv as HCONV, dwTimeout, pXAct, pdwResult);
    }

    hDdeData
}

// ============================================================================
// DdeClientTransaction  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeClientTransaction(
    pData: *mut u8,
    cbData: DWORD,
    hConv: HCONV,
    hszItem: HSZ,
    wFmt: UINT,
    wType: UINT,
    dwTimeout: DWORD,
    pdwResult: *mut DWORD,
) -> HDDEDATA {
    let pConv: *mut WDML_CONV;
    let pXAct: *mut WDML_XACT;
    let hDdeData: HDDEDATA;

    if hConv.is_null() {
        return core::ptr::null_mut();
    }

    pConv = (WDML_GetConv(hConv, 1)).cast();
    if pConv.is_null() {
        /* cannot set error... cannot get back to DDE instance */
        return core::ptr::null_mut();
    }

    // XTYP_ADVSTART variants used as match arms:
    const ADVSTART_NODATA: UINT = XTYP_ADVSTART | XTYPF_NODATA;
    const ADVSTART_NODATA_ACKREQ: UINT = XTYP_ADVSTART | XTYPF_NODATA | XTYPF_ACKREQ;
    const ADVSTART_ACKREQ: UINT = XTYP_ADVSTART | XTYPF_ACKREQ;

    match wType {
        XTYP_EXECUTE => {
            /* Windows simply ignores hszItem and wFmt in this case */
            pXAct = WDML_ClientQueueExecute(pConv, pData as LPVOID, cbData);
            if pXAct.is_null() {
                return core::ptr::null_mut();
            }
        }
        XTYP_POKE => {
            if hszItem.is_null() {
                (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
                return core::ptr::null_mut();
            }
            pXAct = WDML_ClientQueuePoke(pConv, pData as LPVOID, cbData, wFmt, hszItem);
        }
        ADVSTART_NODATA | ADVSTART_NODATA_ACKREQ | XTYP_ADVSTART | ADVSTART_ACKREQ => {
            if !pData.is_null() {
                (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
                return core::ptr::null_mut();
            }
            pXAct = WDML_ClientQueueAdvise(pConv, wType, wFmt, hszItem);
        }
        XTYP_ADVSTOP => {
            if !pData.is_null() {
                (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
                return core::ptr::null_mut();
            }
            pXAct = WDML_ClientQueueUnadvise(pConv, wFmt, hszItem);
        }
        XTYP_REQUEST => {
            if !pData.is_null() || hszItem.is_null() {
                (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
                return core::ptr::null_mut();
            }
            pXAct = WDML_ClientQueueRequest(pConv, wFmt, hszItem);
        }
        _ => {
            /* unknown transaction type */
            (*(*pConv).instance).lastError = DMLERR_INVALIDPARAMETER;
            return core::ptr::null_mut();
        }
    }

    if pXAct.is_null() {
        (*(*pConv).instance).lastError = DMLERR_MEMORY_ERROR;
        return core::ptr::null_mut();
    }

    WDML_QueueTransaction((pConv).cast(), (pXAct).cast());

    if ((*pConv).wStatus & ST_BLOCKED) != 0 {
        return 1 as HDDEDATA;
    }

    hDdeData = WDML_ClientHandle(pConv, pXAct, dwTimeout, pdwResult);
    if dwTimeout != TIMEOUT_ASYNC {
        WDML_UnQueueTransaction(pConv, pXAct);
        WDML_FreeTransaction(((*pConv).instance).cast(), (pXAct).cast(), 1);
    }

    hDdeData
}

// ============================================================================
// DdeAbandonTransaction  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeAbandonTransaction(
    idInst: DWORD,
    hConv: HCONV,
    idTransaction: DWORD,
) -> BOOL {
    let pInstance: *mut WDML_INSTANCE = (WDML_GetInstance(idInst)).cast();
    let mut pConv: *mut WDML_CONV;
    let mut pXAct: *mut WDML_XACT;

    if !pInstance.is_null() {
        if !hConv.is_null() {
            pConv = (WDML_GetConv(hConv, 1)).cast();
            if !pConv.is_null() && (*pConv).instance == pInstance {
                pXAct = (*pConv).transactions;
                while !pXAct.is_null() {
                    let nextXAct: *mut WDML_XACT = (*pXAct).next;

                    if (*pXAct).dwTimeout == TIMEOUT_ASYNC
                        && (idTransaction == 0 || (*pXAct).xActID == idTransaction)
                    {
                        WDML_UnQueueTransaction(pConv, pXAct);
                        WDML_FreeTransaction((pInstance).cast(), (pXAct).cast(), 1);
                    }
                    pXAct = nextXAct;
                }
            }
        } else {
            pConv = (*pInstance).convs[WDML_CLIENT_SIDE];
            while !pConv.is_null() {
                if ((*pConv).wStatus & ST_CONNECTED) == 0 {
                    pConv = (*pConv).next;
                    continue;
                }
                pXAct = (*pConv).transactions;
                while !pXAct.is_null() {
                    let nextXAct: *mut WDML_XACT = (*pXAct).next;

                    if (*pXAct).dwTimeout == TIMEOUT_ASYNC {
                        WDML_UnQueueTransaction(pConv, pXAct);
                        WDML_FreeTransaction((pInstance).cast(), (pXAct).cast(), 1);
                    }
                    pXAct = nextXAct;
                }
                pConv = (*pConv).next;
            }
        }
    }

    1
}

// ============================================================================
// WDML_ClientProc  — static CALLBACK window proc (private).
// CALLBACK = stdcall; keep private (static) → no #[no_mangle], extern "system".
// ============================================================================
unsafe extern "system" fn WDML_ClientProc(
    hwnd: HWND,
    iMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let uiLo: UINT;
    let uiHi: UINT;
    let mut pConv: *mut WDML_CONV = core::ptr::null_mut();
    let hszSrv: HSZ;
    let hszTpc: HSZ;

    if iMsg == WM_DDE_ACK && {
        pConv = (WDML_GetConvFromWnd(hwnd)).cast();
        pConv.is_null() || (*pConv).wStatus == XST_INIT1
    } {
        /* In response to WM_DDE_INITIATE, save server window  */
        let mut buf: [u8; 256] = [0; 256];
        let pInstance: *mut WDML_INSTANCE;

        /* note: sent messages do not need packing */
        uiLo = LOWORD(lParam as DWORD) as UINT;
        uiHi = HIWORD(lParam as DWORD) as UINT;

        /* FIXME: convlist should be handled here */
        if !pConv.is_null() {
            /* we already have started the conv with a server, drop other replies */
            GlobalDeleteAtom(uiLo as ATOM);
            GlobalDeleteAtom(uiHi as ATOM);
            PostMessageW(wParam as HWND, WM_DDE_TERMINATE, hwnd as WPARAM, 0);
            return 0;
        }

        pInstance = (WDML_GetInstanceFromWnd(hwnd)).cast();

        hszSrv = WDML_MakeHszFromAtom(pInstance, uiLo as ATOM);
        hszTpc = WDML_MakeHszFromAtom(pInstance, uiHi as ATOM);

        pConv = (WDML_AddConv(
            (pInstance).cast(),
            WDML_CLIENT_SIDE as INT,
            hszSrv,
            hszTpc,
            hwnd,
            wParam as HWND,
        )).cast();

        SetWindowLongPtrW(hwnd, GWL_WDML_CONVERSATION, pConv as LONG_PTR);
        (*pConv).wStatus |= ST_CONNECTED;
        (*pConv).wConvst = XST_INIT1;

        /* check if server is handled by DDEML */
        if (GetClassNameA(
            wParam as HWND,
            buf.as_mut_ptr() as LPSTR,
            core::mem::size_of_val(&buf) as INT,
        ) != 0
            && lstrcmpiA(
                buf.as_ptr() as LPCSTR,
                WDML_szServerConvClassA.as_ptr() as LPCSTR,
            ) == 0)
            || (GetClassNameW(
                wParam as HWND,
                buf.as_mut_ptr() as LPWSTR,
                (core::mem::size_of_val(&buf) / core::mem::size_of::<WCHAR>()) as INT,
            ) != 0
                && lstrcmpiW(
                    buf.as_ptr() as LPCWSTR,
                    WDML_szServerConvClassW.as_ptr(),
                ) == 0)
        {
            (*pConv).wStatus |= ST_ISLOCAL;
        }

        GlobalDeleteAtom(uiLo as ATOM);
        GlobalDeleteAtom(uiHi as ATOM);

        /* accept conversation */
        return 1;
    }

    if iMsg >= WM_DDE_FIRST && iMsg <= WM_DDE_LAST {
        pConv = (WDML_GetConvFromWnd(hwnd)).cast();

        if !pConv.is_null() {
            let mut msg: MSG = core::mem::zeroed();
            let mut hdd: HDDEDATA = core::mem::zeroed();

            msg.hwnd = hwnd;
            msg.message = iMsg;
            msg.wParam = wParam;
            msg.lParam = lParam;

            WDML_HandleReply(pConv, &mut msg, &mut hdd, core::ptr::null_mut());
        }

        return 0;
    }

    if IsWindowUnicode(hwnd) != 0 {
        DefWindowProcW(hwnd, iMsg, wParam, lParam)
    } else {
        DefWindowProcA(hwnd, iMsg, wParam, lParam)
    }
}

// ============================================================================
// DdeDisconnect  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeDisconnect(hConv: HCONV) -> BOOL {
    let pConv: *mut WDML_CONV;
    let pXAct: *mut WDML_XACT;
    let mut ret: BOOL = 0;

    if hConv.is_null() {
        return 0;
    }

    pConv = (WDML_GetConv(hConv, 1)).cast();
    if !pConv.is_null() && ((*pConv).wStatus & ST_CLIENT) != 0 {
        /* FIXME: should abandon all pending transactions */
        pXAct = WDML_ClientQueueTerminate(pConv);
        if !pXAct.is_null() {
            if PostMessageW(
                (*pConv).hwndServer,
                (*pXAct).ddeMsg,
                (*pConv).hwndClient as WPARAM,
                (*pXAct).lParam,
            ) != 0
            {
                WDML_SyncWaitTransactionReply(hConv, 10000, pXAct, core::ptr::null_mut());
                ret = 1;
            } else {
                (*(*pConv).instance).lastError = DMLERR_POSTMSG_FAILED;
            }

            WDML_FreeTransaction(((*pConv).instance).cast(), (pXAct).cast(), 1);
            /* still have to destroy data associated with conversation */
            WDML_RemoveConv((pConv).cast(), WDML_CLIENT_SIDE as INT);
        }
        /* else: FIXME — not implemented yet for a server side conversation */
    }

    ret
}

// ============================================================================
// DdeImpersonateClient  (USER32.@)
// ============================================================================
#[no_mangle]
pub unsafe extern "system" fn DdeImpersonateClient(hConv: HCONV) -> BOOL {
    let pConv: *mut WDML_CONV;
    let mut ret: BOOL = 0;

    pConv = (WDML_GetConv(hConv, 1)).cast();
    if !pConv.is_null() {
        ret = ImpersonateDdeClientWindow((*pConv).hwndClient, (*pConv).hwndServer);
    }
    ret
}
