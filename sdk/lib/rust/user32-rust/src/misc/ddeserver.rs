//! Translated from win32ss/user/user32/misc/ddeserver.c — DDEML server side.
//!
//! Cohabitation module for the `user32-rust` staticlib. The `#[no_mangle]` symbols
//! here REPLACE the C ddeserver.c TU at link time. Shares the WDML_* structures and
//! DDE constants of dde_private.h / ddeml.h / dde.h with ddemisc.c + ddeclient.c.
//!
//! ABI rules (see CONTRACT.md):
//!   * `WINAPI` public APIs (DdePostAdvise, DdeNameService) → extern "system"
//!   * non-WINAPI cross-file WDML_* (only WDML_ServerHandle here) → extern "C"
//!   * static helpers stay private Rust fns
//!   * the two CALLBACK window procs (WDML_ServerNameProc / WDML_ServerConvProc) are
//!     static in C — kept private; their address is handed to RegisterClassEx*.

#![allow(non_snake_case, non_upper_case_globals, dead_code, non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use win32k_sys::win32k::*;
// The crate's own functions: the definition is the declaration.
use crate::windows::class::GetWindowLongPtrW;
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
use core::ffi::c_void;

use crate::internal::MAKELPARAM;
// DestroyWindow is an ntwrapper INLINE (NtUser* multiplexer) — no link symbol;
// import it rather than declaring it extern (per the cohab contract).
use crate::ntwrapper::DestroyWindow;

// ───────────────────────────────────────────────────────────────────────────
//  DDE types not present in the win32k_sys bindgen surface (DECLARE_HANDLE in
//  ddeml.h → opaque pointer; structs from ddeml.h / dde.h). Reproduce faithfully.
// ───────────────────────────────────────────────────────────────────────────
type HSZ = *mut c_void;
type HDDEDATA = *mut c_void;
type HCONV = *mut c_void;

// tagHSZPAIR (ddeml.h)
// The dde_private.h structs are ddemisc's: one definition, one type.
pub use crate::misc::ddemisc::{CONVCONTEXT, HSZNode, WDML_CONV, WDML_INSTANCE, WDML_LINK, WDML_SERVER, WDML_XACT};

#[repr(C)]
#[derive(Copy, Clone)]
struct HSZPAIR {
    hszSvc: HSZ,
    hszTopic: HSZ,
}

// tagCONVCONTEXT (ddeml.h)

// DDEADVISE (dde.h): unsigned short reserved:14, fDeferUpd:1, fAckReq:1; short cfFormat;
#[repr(C)]
#[derive(Copy, Clone)]
struct DDEADVISE {
    flags: u16,
    cfFormat: i16,
}
impl DDEADVISE {
    #[inline]
    fn fDeferUpd(&self) -> u16 {
        (self.flags >> 14) & 0x1
    }
    #[inline]
    fn fAckReq(&self) -> u16 {
        (self.flags >> 15) & 0x1
    }
}

// DDEPOKE (dde.h): unsigned short unused:13, fRelease:1, fReserved:2; short cfFormat; BYTE Value[1];
#[repr(C)]
struct DDEPOKE {
    flags: u16,
    cfFormat: i16,
    Value: [u8; 1],
}

// PFNCALLBACK (ddeml.h)
type PFNCALLBACK = Option<
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

// ───────────────────────────────────────────────────────────────────────────
//  WDML internal structures (dde_private.h). Reproduce layout faithfully.
// ───────────────────────────────────────────────────────────────────────────






// WDML_QUEUE_STATE enum (dde_private.h)
type WDML_QUEUE_STATE = INT;
const WDML_QS_ERROR: WDML_QUEUE_STATE = 0;
const WDML_QS_HANDLED: WDML_QUEUE_STATE = 1;
#[allow(dead_code)]
const WDML_QS_PASS: WDML_QUEUE_STATE = 2;
#[allow(dead_code)]
const WDML_QS_SWALLOWED: WDML_QUEUE_STATE = 3;
const WDML_QS_BLOCK: WDML_QUEUE_STATE = 4;

// WDML_SIDE enum
const WDML_CLIENT_SIDE: INT = 0;
const WDML_SERVER_SIDE: INT = 1;

// ───────────────────────────────────────────────────────────────────────────
//  DDE constants (ddeml.h / dde.h)
// ───────────────────────────────────────────────────────────────────────────
const CADV_LATEACK: UINT = 0xFFFF;

const XTYPF_NOBLOCK: UINT = 0x0002;
const XTYPF_NODATA: UINT = 0x0004;
const XTYPF_ACKREQ: UINT = 0x0008;

const XCLASS_BOOL: UINT = 0x1000;
const XCLASS_DATA: UINT = 0x2000;
const XCLASS_FLAGS: UINT = 0x4000;
const XCLASS_NOTIFICATION: UINT = 0x8000;

const XTYP_ADVREQ: UINT = 0x0020 | XCLASS_DATA | XTYPF_NOBLOCK;
const XTYP_ADVSTART: UINT = 0x0030 | XCLASS_BOOL;
const XTYP_ADVSTOP: UINT = 0x0040 | XCLASS_NOTIFICATION;
const XTYP_EXECUTE: UINT = 0x0050 | XCLASS_FLAGS;
const XTYP_CONNECT: UINT = 0x0060 | XCLASS_BOOL | XTYPF_NOBLOCK;
const XTYP_CONNECT_CONFIRM: UINT = 0x0070 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;
const XTYP_POKE: UINT = 0x0090 | XCLASS_FLAGS;
const XTYP_REQUEST: UINT = 0x00B0 | XCLASS_DATA;
const XTYP_DISCONNECT: UINT = 0x00C0 | XCLASS_NOTIFICATION | XTYPF_NOBLOCK;
const XTYP_WILDCONNECT: UINT = 0x00E0 | XCLASS_DATA | XTYPF_NOBLOCK;

const CBF_FAIL_SELFCONNECTIONS: DWORD = 0x00001000;
const CBF_FAIL_CONNECTIONS: DWORD = 0x00002000;
const CBF_FAIL_ADVISES: DWORD = 0x00004000;
const CBF_FAIL_EXECUTES: DWORD = 0x00008000;
const CBF_FAIL_POKES: DWORD = 0x00010000;
const CBF_FAIL_REQUESTS: DWORD = 0x00020000;
const CBF_SKIP_DISCONNECTS: DWORD = 0x00200000;

// CBR_BLOCK = (HDDEDATA)(LONG_PTR)-1
const CBR_BLOCK: usize = usize::MAX;

const DNS_REGISTER: UINT = 0x0001;
const DNS_UNREGISTER: UINT = 0x0002;
const DNS_FILTERON: UINT = 0x0004;
const DNS_FILTEROFF: UINT = 0x0008;

const ST_CONNECTED: DWORD = 0x0001;
const ST_ISLOCAL: DWORD = 0x0004;
const ST_BLOCKED: DWORD = 0x0008;
const ST_TERMINATED: DWORD = 0x0020;
const ST_ISSELF: DWORD = 0x0100;

const DMLERR_DLL_USAGE: DWORD = 0x4004;
const DMLERR_INVALIDPARAMETER: DWORD = 0x4006;
const DMLERR_MEMORY_ERROR: DWORD = 0x4008;
const DMLERR_POSTMSG_FAILED: DWORD = 0x400c;

const DDE_FACK: usize = 0x8000;
const DDE_FBUSY: usize = 0x4000;
const DDE_FNOTPROCESSED: usize = 0x0000;

const CP_WINUNICODE: INT = 1200;
const CP_WINANSI: INT = 1004;
const CP_ACP: UINT = 0;
const CF_TEXT: UINT = 1;

// dde.h WM_DDE_* messages
const WM_DDE_INITIATE: UINT = 0x3E0;
const WM_DDE_TERMINATE: UINT = 0x3E1;
const WM_DDE_ADVISE: UINT = 0x3E2;
const WM_DDE_UNADVISE: UINT = 0x3E3;
const WM_DDE_ACK: UINT = 0x3E4;
const WM_DDE_DATA: UINT = 0x3E5;
const WM_DDE_REQUEST: UINT = 0x3E6;
const WM_DDE_POKE: UINT = 0x3E7;
const WM_DDE_EXECUTE: UINT = 0x3E8;
const WM_DDE_LAST: UINT = WM_DDE_EXECUTE;
const WM_DDE_FIRST: UINT = WM_DDE_INITIATE;

// dde_private.h WM_WDML_* (WM_USER + 0x200 ...), WM_USER = 1024 (winuser.h)
const WM_USER: UINT = 1024;
const WM_WDML_REGISTER: UINT = WM_USER + 0x200;

// dde_private.h GWL_WDML_* window-long offsets
const GWL_WDML_INSTANCE: INT = 0;
const GWL_WDML_CONVERSATION: INT = core::mem::size_of::<ULONG_PTR>() as INT;
const GWL_WDML_SERVER: INT = core::mem::size_of::<ULONG_PTR>() as INT;

// other window-message constants used below
const WM_DESTROY: UINT = 0x0002;
const WS_POPUP: DWORD = 0x80000000;
const WS_CHILD: DWORD = 0x40000000;

// ───────────────────────────────────────────────────────────────────────────
//  Class-name globals.
//    - szServerNameClass: static in C → private.
//    - WDML_szServerConvClassA / W: non-static in C → exported (#[no_mangle]).
// ───────────────────────────────────────────────────────────────────────────
// L"DDEMLMom"
static szServerNameClass: [WCHAR; 9] = [
    b'D' as u16, b'D' as u16, b'E' as u16, b'M' as u16, b'L' as u16, b'M' as u16,
    b'o' as u16, b'm' as u16, 0,
];

// const char WDML_szServerConvClassA[] = "DDEMLAnsiServer";
#[no_mangle]
pub static WDML_szServerConvClassA: [u8; 16] =
    *b"DDEMLAnsiServer\0";

// const WCHAR WDML_szServerConvClassW[] = L"DDEMLUnicodeServer";
#[no_mangle]
pub static WDML_szServerConvClassW: [WCHAR; 19] = [
    b'D' as u16, b'D' as u16, b'E' as u16, b'M' as u16, b'L' as u16, b'U' as u16,
    b'n' as u16, b'i' as u16, b'c' as u16, b'o' as u16, b'd' as u16, b'e' as u16,
    b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'e' as u16, b'r' as u16, 0,
];

// ───────────────────────────────────────────────────────────────────────────
//  Cross-file WDML_* helpers DEFINED elsewhere (ddemisc.c) → plain cdecl → "C".
//  Class-name globals defined elsewhere (ddemisc.c / ddeclient.c).
// ───────────────────────────────────────────────────────────────────────────
extern "C" {
    fn WDML_MakeAtomFromHsz(hsz: HSZ) -> ATOM;
    fn WDML_DataHandle2Global(
        hDdeData: HDDEDATA,
        fResponse: BOOL,
        fRelease: BOOL,
        fDeferUpd: BOOL,
        dAckReq: BOOL,
    ) -> HGLOBAL;
    fn WDML_AddServer(
        pInstance: *mut WDML_INSTANCE,
        hszService: HSZ,
        hszTopic: HSZ,
    ) -> *mut WDML_SERVER;
    fn WDML_RemoveServer(pInstance: *mut WDML_INSTANCE, hszService: HSZ, hszTopic: HSZ);
    fn WDML_FindServer(
        pInstance: *mut WDML_INSTANCE,
        hszService: HSZ,
        hszTopic: HSZ,
    ) -> *mut WDML_SERVER;
    fn WDML_IncHSZ(pInstance: *mut WDML_INSTANCE, hsz: HSZ) -> BOOL;
    fn WDML_IsAppOwned(hDdeData: HDDEDATA) -> BOOL;
    fn WDML_BroadcastDDEWindows(clsName: LPCWSTR, uMsg: UINT, wParam: WPARAM, lParam: LPARAM);

    // L"DDEMLEvent" — defined (non-static) in ddemisc.c.
    static WDML_szEventClass: [WCHAR; 0];
    // ANSI/Unicode client-conv class names — defined (non-static) in ddemisc.c / ddeclient.c.
    static WDML_szClientConvClassA: [u8; 0];
    static WDML_szClientConvClassW: [WCHAR; 0];

    // WIN_GetFullHandle — cdecl helper (defined in controls/combo.rs / win C side).
    fn WIN_GetFullHandle(h: HWND) -> HWND;

    // CRT (cdecl)
    fn memset(s: *mut c_void, c: INT, n: usize) -> *mut c_void;
    fn memchr(s: *const c_void, c: INT, n: usize) -> *mut c_void;
}

// ───────────────────────────────────────────────────────────────────────────
//  Cross-DLL Win32 client APIs (WINAPI / NTAPI → stdcall on i386) → "system".
// ───────────────────────────────────────────────────────────────────────────
extern "system" {
    // user32 (peers / forwarders)
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn DefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    fn RegisterClassExA(lpwcx: *const WNDCLASSEXA) -> ATOM;
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
    fn IsWindowUnicode(hWnd: HWND) -> BOOL;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: LPDWORD) -> DWORD;
    fn GetClassNameA(hWnd: HWND, lpClassName: LPSTR, nMaxCount: INT) -> INT;
    fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    fn lstrcmpiA(lpString1: LPCSTR, lpString2: LPCSTR) -> INT;
    fn lstrcmpiW(lpString1: LPCWSTR, lpString2: LPCWSTR) -> INT;

    // kernel32
    fn GetCurrentThreadId() -> DWORD;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
    fn GlobalDeleteAtom(nAtom: ATOM) -> ATOM;
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: INT,
        lpWideCharStr: LPWSTR,
        cchWideChar: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: INT,
        lpMultiByteStr: LPSTR,
        cbMultiByte: INT,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: LPBOOL,
    ) -> INT;

    // user32 DDE peer APIs (PackDDElParam etc. + Dde* — defined in ddemisc.c)
    fn PackDDElParam(msg: UINT, uiLo: UINT_PTR, uiHi: UINT_PTR) -> LPARAM;
    fn UnpackDDElParam(msg: UINT, lParam: LPARAM, uiLo: *mut UINT_PTR, uiHi: *mut UINT_PTR) -> BOOL;
    fn ReuseDDElParam(
        lParam: LPARAM,
        msgIn: UINT,
        msgOut: UINT,
        uiLo: UINT_PTR,
        uiHi: UINT_PTR,
    ) -> LPARAM;
    fn FreeDDElParam(msg: UINT, lParam: LPARAM) -> BOOL;
    fn DdeCmpStringHandles(hsz1: HSZ, hsz2: HSZ) -> INT;
    fn DdeFreeDataHandle(hData: HDDEDATA) -> BOOL;
    fn DdeAccessData(hData: HDDEDATA, pcbDataSize: LPDWORD) -> LPBYTE;
    fn DdeUnaccessData(hData: HDDEDATA) -> BOOL;
    fn DdeCreateDataHandle(
        idInst: DWORD,
        pSrc: LPBYTE,
        cb: DWORD,
        cbOff: DWORD,
        hszItem: HSZ,
        wFmt: UINT,
        afCmd: UINT,
    ) -> HDDEDATA;
}

// CreateWindowW/A wrappers (windows.h #define → *Ex with dwExStyle = 0).
#[inline]
unsafe fn CreateWindowW(
    lpClassName: LPCWSTR,
    lpWindowName: LPCWSTR,
    dwStyle: DWORD,
    x: INT,
    y: INT,
    w: INT,
    h: INT,
    parent: HWND,
    menu: HMENU,
    inst: HINSTANCE,
    param: LPVOID,
) -> HWND {
    CreateWindowExW(0, lpClassName, lpWindowName, dwStyle, x, y, w, h, parent, menu, inst, param)
}
#[inline]
unsafe fn CreateWindowA(
    lpClassName: LPCSTR,
    lpWindowName: LPCSTR,
    dwStyle: DWORD,
    x: INT,
    y: INT,
    w: INT,
    h: INT,
    parent: HWND,
    menu: HMENU,
    inst: HINSTANCE,
    param: LPVOID,
) -> HWND {
    CreateWindowExA(0, lpClassName, lpWindowName, dwStyle, x, y, w, h, parent, menu, inst, param)
}

// memchrW (wine/unicode.h) — search a WCHAR buffer of `n` elements for `ch`.
#[inline]
unsafe fn memchrW(ptr: *const WCHAR, ch: WCHAR, n: usize) -> *const WCHAR {
    let mut p = ptr;
    let mut left = n;
    while left != 0 {
        if *p == ch {
            return p;
        }
        p = p.add(1);
        left -= 1;
    }
    core::ptr::null()
}

// ═══════════════════════════════════════════════════════════════════════════
//  DdePostAdvise [USER32.@]
// ═══════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn DdePostAdvise(idInst: DWORD, hszTopic: HSZ, hszItem: HSZ) -> BOOL {
    let pInstance: *mut WDML_INSTANCE;
    let mut pLink: *mut WDML_LINK;
    let mut hDdeData: HDDEDATA;
    let mut hItemData: HGLOBAL;
    let mut pConv: *mut WDML_CONV;
    let atom: ATOM;
    let mut count: UINT;

    pInstance = (WDML_GetInstance(idInst)).cast();

    if pInstance.is_null() {
        return FALSE as BOOL;
    }

    atom = WDML_MakeAtomFromHsz(hszItem);
    if atom == 0 {
        return FALSE as BOOL;
    }

    // first compute the number of links which will trigger a message
    count = 0;
    pLink = (*pInstance).links[WDML_SERVER_SIDE as usize];
    while !pLink.is_null() {
        if DdeCmpStringHandles(hszItem, (*pLink).hszItem) == 0 {
            count += 1;
        }
        pLink = (*pLink).next;
    }
    if count >= CADV_LATEACK {
        // FIXME("too high value for count\n");
        count &= 0xFFFF;
    }

    pLink = (*pInstance).links[WDML_SERVER_SIDE as usize];
    while !pLink.is_null() {
        if DdeCmpStringHandles(hszItem, (*pLink).hszItem) == 0 {
            count = count.wrapping_sub(1);
            hDdeData = WDML_InvokeCallback(
                (pInstance).cast(),
                XTYP_ADVREQ,
                (*pLink).uFmt,
                (*pLink).hConv,
                hszTopic,
                hszItem,
                core::ptr::null_mut(),
                count as ULONG_PTR,
                0,
            );

            if hDdeData as usize == CBR_BLOCK {
                // MS doc is not consistent here
                // FIXME("CBR_BLOCK returned for ADVREQ\n");
                pLink = (*pLink).next;
                continue;
            }
            if !hDdeData.is_null() {
                if (*pLink).transactionType & XTYPF_NODATA != 0 {
                    hItemData = core::ptr::null_mut();
                } else {
                    hItemData = WDML_DataHandle2Global(
                        hDdeData,
                        FALSE as BOOL,
                        FALSE as BOOL,
                        FALSE as BOOL,
                        FALSE as BOOL,
                    );
                }

                pConv = (WDML_GetConv((*pLink).hConv, TRUE as BOOL)).cast();

                if pConv.is_null() {
                    if WDML_IsAppOwned(hDdeData) == 0 {
                        DdeFreeDataHandle(hDdeData);
                    }
                    // goto theError
                    GlobalDeleteAtom(atom);
                    return FALSE as BOOL;
                }

                if PostMessageW(
                    (*pConv).hwndClient,
                    WM_DDE_DATA,
                    (*pConv).hwndServer as WPARAM,
                    PackDDElParam(WM_DDE_DATA, hItemData as UINT_PTR, atom as UINT_PTR),
                ) == 0
                {
                    // ERR("post message failed\n");
                    (*pConv).wStatus &= !ST_CONNECTED;
                    (*(*pConv).instance).lastError = DMLERR_POSTMSG_FAILED;
                    if WDML_IsAppOwned(hDdeData) == 0 {
                        DdeFreeDataHandle(hDdeData);
                    }
                    GlobalFree(hItemData);
                    // goto theError
                    GlobalDeleteAtom(atom);
                    return FALSE as BOOL;
                }
                if WDML_IsAppOwned(hDdeData) == 0 {
                    DdeFreeDataHandle(hDdeData);
                }
            }
        }
        pLink = (*pLink).next;
    }
    TRUE as BOOL
}

// ═══════════════════════════════════════════════════════════════════════════
//  DdeNameService [USER32.@]
// ═══════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "system" fn DdeNameService(
    idInst: DWORD,
    hsz1: HSZ,
    hsz2: HSZ,
    afCmd: UINT,
) -> HDDEDATA {
    let mut pServer: *mut WDML_SERVER;
    let pInstance: *mut WDML_INSTANCE;
    let hwndServer: HWND;
    let mut wndclass: WNDCLASSEXW = core::mem::zeroed();

    // First check instance
    pInstance = (WDML_GetInstance(idInst)).cast();
    if pInstance.is_null() {
        // Nothing has been initialised - exit now!
        return core::ptr::null_mut();
    }

    if !hsz2.is_null() {
        // Illegal, reserved parameter
        (*pInstance).lastError = DMLERR_INVALIDPARAMETER;
        return core::ptr::null_mut();
    }
    if hsz1.is_null() && (afCmd & DNS_UNREGISTER) == 0 {
        (*pInstance).lastError = DMLERR_INVALIDPARAMETER;
        return core::ptr::null_mut();
    }

    match afCmd & (DNS_REGISTER | DNS_UNREGISTER) {
        DNS_REGISTER => {
            pServer = WDML_FindServer(pInstance, hsz1, core::ptr::null_mut());
            if !pServer.is_null() {
                // ERR("Trying to register already registered service!\n");
                (*pInstance).lastError = DMLERR_DLL_USAGE;
                return core::ptr::null_mut();
            }

            WDML_IncHSZ(pInstance, hsz1);

            pServer = WDML_AddServer(pInstance, hsz1, core::ptr::null_mut());

            WDML_BroadcastDDEWindows(
                WDML_szEventClass.as_ptr(),
                WM_WDML_REGISTER,
                (*pServer).atomService as WPARAM,
                (*pServer).atomServiceSpec as LPARAM,
            );

            wndclass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
            wndclass.style = 0;
            wndclass.lpfnWndProc = core::mem::transmute::<
                unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
                WNDPROC,
            >(WDML_ServerNameProc);
            wndclass.cbClsExtra = 0;
            wndclass.cbWndExtra = (2 * core::mem::size_of::<ULONG_PTR>()) as INT;
            wndclass.hInstance = core::ptr::null_mut();
            wndclass.hIcon = core::ptr::null_mut();
            wndclass.hCursor = core::ptr::null_mut();
            wndclass.hbrBackground = core::ptr::null_mut();
            wndclass.lpszMenuName = core::ptr::null();
            wndclass.lpszClassName = szServerNameClass.as_ptr();
            wndclass.hIconSm = core::ptr::null_mut();

            RegisterClassExW(&wndclass);

            hwndServer = CreateWindowW(
                szServerNameClass.as_ptr(),
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

            SetWindowLongPtrW(hwndServer, GWL_WDML_INSTANCE, pInstance as LONG_PTR);
            SetWindowLongPtrW(hwndServer, GWL_WDML_SERVER, pServer as LONG_PTR);

            (*pServer).hwndServer = hwndServer;
        }

        DNS_UNREGISTER => {
            if hsz1.is_null() {
                // General unregister situation: terminate all server side convs
                while !(*pInstance).servers.is_null() {
                    WDML_RemoveServer(
                        pInstance,
                        (*(*pInstance).servers).hszService,
                        core::ptr::null_mut(),
                    );
                }
                (*pInstance).servers = core::ptr::null_mut();
            } else {
                WDML_RemoveServer(pInstance, hsz1, core::ptr::null_mut());
            }
        }

        _ => {}
    }

    if afCmd & (DNS_FILTERON | DNS_FILTEROFF) != 0 {
        // Set filter flags to hold notifications of connection
        pServer = WDML_FindServer(pInstance, hsz1, core::ptr::null_mut());
        if pServer.is_null() {
            // trying to filter where no service names
            (*pInstance).lastError = DMLERR_DLL_USAGE;
            return core::ptr::null_mut();
        } else {
            (*pServer).filterOn = ((afCmd & DNS_FILTERON) != 0) as BOOL;
        }
    }
    // (HDDEDATA)TRUE
    TRUE as usize as HDDEDATA
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_CreateServerConv  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_CreateServerConv(
    pInstance: *mut WDML_INSTANCE,
    hwndClient: HWND,
    hwndServerName: HWND,
    hszApp: HSZ,
    hszTopic: HSZ,
) -> *mut WDML_CONV {
    let hwndServerConv: HWND;
    let pConv: *mut WDML_CONV;

    if (*pInstance).unicode != 0 {
        let mut wndclass: WNDCLASSEXW = core::mem::zeroed();

        wndclass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
        wndclass.style = 0;
        wndclass.lpfnWndProc = core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(WDML_ServerConvProc);
        wndclass.cbClsExtra = 0;
        wndclass.cbWndExtra = (2 * core::mem::size_of::<ULONG_PTR>()) as INT;
        wndclass.hInstance = core::ptr::null_mut();
        wndclass.hIcon = core::ptr::null_mut();
        wndclass.hCursor = core::ptr::null_mut();
        wndclass.hbrBackground = core::ptr::null_mut();
        wndclass.lpszMenuName = core::ptr::null();
        wndclass.lpszClassName = WDML_szServerConvClassW.as_ptr();
        wndclass.hIconSm = core::ptr::null_mut();

        RegisterClassExW(&wndclass);

        hwndServerConv = CreateWindowW(
            WDML_szServerConvClassW.as_ptr(),
            core::ptr::null(),
            WS_CHILD,
            0,
            0,
            0,
            0,
            hwndServerName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    } else {
        let mut wndclass: WNDCLASSEXA = core::mem::zeroed();

        wndclass.cbSize = core::mem::size_of::<WNDCLASSEXA>() as UINT;
        wndclass.style = 0;
        wndclass.lpfnWndProc = core::mem::transmute::<
            unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
            WNDPROC,
        >(WDML_ServerConvProc);
        wndclass.cbClsExtra = 0;
        wndclass.cbWndExtra = (2 * core::mem::size_of::<ULONG_PTR>()) as INT;
        wndclass.hInstance = core::ptr::null_mut();
        wndclass.hIcon = core::ptr::null_mut();
        wndclass.hCursor = core::ptr::null_mut();
        wndclass.hbrBackground = core::ptr::null_mut();
        wndclass.lpszMenuName = core::ptr::null();
        wndclass.lpszClassName = WDML_szServerConvClassA.as_ptr() as LPCSTR;
        wndclass.hIconSm = core::ptr::null_mut();

        RegisterClassExA(&wndclass);

        hwndServerConv = CreateWindowA(
            WDML_szServerConvClassA.as_ptr() as LPCSTR,
            core::ptr::null(),
            WS_CHILD,
            0,
            0,
            0,
            0,
            hwndServerName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    pConv = (WDML_AddConv(
        (pInstance).cast(),
        WDML_SERVER_SIDE,
        hszApp,
        hszTopic,
        hwndClient,
        hwndServerConv,
    )).cast();
    if !pConv.is_null() {
        SetWindowLongPtrW(hwndServerConv, GWL_WDML_INSTANCE, pInstance as LONG_PTR);
        SetWindowLongPtrW(hwndServerConv, GWL_WDML_CONVERSATION, pConv as LONG_PTR);

        // this should be the only place using SendMessage for WM_DDE_ACK
        // note: sent messages shall not use packing
        SendMessageW(
            hwndClient,
            WM_DDE_ACK,
            hwndServerConv as WPARAM,
            MAKELPARAM(
                WDML_MakeAtomFromHsz(hszApp),
                WDML_MakeAtomFromHsz(hszTopic),
            ),
        );
        // we assume we're connected since we've sent an answer...
        (*pConv).wStatus |= ST_CONNECTED;
    } else {
        DestroyWindow(hwndServerConv);
    }
    pConv
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerNameProc  (static CALLBACK) — kept private; addr → RegisterClassEx.
// ═══════════════════════════════════════════════════════════════════════════
unsafe extern "system" fn WDML_ServerNameProc(
    hwndServer: HWND,
    iMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let hwndClient: HWND;
    let hszApp: HSZ;
    let hszTop: HSZ;
    let mut hDdeData: HDDEDATA;
    let pInstance: *mut WDML_INSTANCE;
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;

    match iMsg {
        WM_DDE_INITIATE => {
            // wParam         -- sending window handle
            // LOWORD(lParam) -- application atom
            // HIWORD(lParam) -- topic atom
            hwndClient = wParam as HWND;

            pInstance = (WDML_GetInstanceFromWnd(hwndServer)).cast();
            if pInstance.is_null() {
                return 0;
            }

            // don't free DDEParams, since this is a broadcast
            UnpackDDElParam(WM_DDE_INITIATE, lParam, &mut uiLo, &mut uiHi);

            hszApp = WDML_MakeHszFromAtom(pInstance, uiLo as ATOM);
            hszTop = WDML_MakeHszFromAtom(pInstance, uiHi as ATOM);

            if (*pInstance).CBFflags & CBF_FAIL_CONNECTIONS == 0 {
                let mut self_: BOOL = FALSE as BOOL;
                let mut cc: CONVCONTEXT = core::mem::zeroed();
                let mut pcc: *mut CONVCONTEXT = core::ptr::null_mut();
                let mut buf: [u8; 256] = [0; 256];

                if GetWindowThreadProcessId(hwndClient, core::ptr::null_mut())
                    == GetWindowThreadProcessId(hwndServer, core::ptr::null_mut())
                    && WDML_GetInstanceFromWnd(hwndClient) == WDML_GetInstanceFromWnd(hwndServer)
                {
                    self_ = TRUE as BOOL;
                }
                // FIXME: build a default context if remote is handled under DDEML
                if (GetClassNameA(hwndClient, buf.as_mut_ptr() as LPSTR, buf.len() as INT) != 0
                    && lstrcmpiA(
                        buf.as_ptr() as LPCSTR,
                        WDML_szClientConvClassA.as_ptr() as LPCSTR,
                    ) == 0)
                    || (GetClassNameW(
                        hwndClient,
                        buf.as_mut_ptr() as LPWSTR,
                        (buf.len() / core::mem::size_of::<WCHAR>()) as INT,
                    ) != 0
                        && lstrcmpiW(
                            buf.as_ptr() as LPCWSTR,
                            WDML_szClientConvClassW.as_ptr(),
                        ) == 0)
                {
                    pcc = &mut cc;
                    memset(pcc as *mut c_void, 0, core::mem::size_of::<CONVCONTEXT>());
                    (*pcc).cb = core::mem::size_of::<CONVCONTEXT>() as UINT;
                    (*pcc).iCodePage = if IsWindowUnicode(hwndClient) != 0 {
                        CP_WINUNICODE
                    } else {
                        CP_WINANSI
                    };
                }
                if (*pInstance).CBFflags & CBF_FAIL_SELFCONNECTIONS != 0 && self_ != 0 {
                    // Don't do self connection as requested
                } else if !hszApp.is_null() && !hszTop.is_null() {
                    let pServer: *mut WDML_SERVER =
                        GetWindowLongPtrW(hwndServer, GWL_WDML_SERVER) as *mut WDML_SERVER;

                    // check filters for name service
                    if (*pServer).filterOn == 0
                        || DdeCmpStringHandles((*pServer).hszService, hszApp) == 0
                    {
                        // pass on to the callback
                        hDdeData = WDML_InvokeCallback(
                            (pInstance).cast(),
                            XTYP_CONNECT,
                            0,
                            core::ptr::null_mut(),
                            hszTop,
                            hszApp,
                            core::ptr::null_mut(),
                            pcc as ULONG_PTR,
                            self_ as ULONG_PTR,
                        );
                        if hDdeData as usize != 0 {
                            let pConv2 = WDML_CreateServerConv(
                                pInstance, hwndClient, hwndServer, hszApp, hszTop,
                            );
                            if !pConv2.is_null() {
                                if !pcc.is_null() {
                                    (*pConv2).wStatus |= ST_ISLOCAL;
                                }
                                WDML_InvokeCallback(
                                    (pInstance).cast(),
                                    XTYP_CONNECT_CONFIRM,
                                    0,
                                    pConv2 as HCONV,
                                    hszTop,
                                    hszApp,
                                    core::ptr::null_mut(),
                                    pcc as ULONG_PTR,
                                    self_ as ULONG_PTR,
                                );
                            }
                        }
                    }
                } else if !(*pInstance).servers.is_null() {
                    // pass on to the callback
                    hDdeData = WDML_InvokeCallback(
                        (pInstance).cast(),
                        XTYP_WILDCONNECT,
                        0,
                        core::ptr::null_mut(),
                        hszTop,
                        hszApp,
                        core::ptr::null_mut(),
                        pcc as ULONG_PTR,
                        self_ as ULONG_PTR,
                    );

                    if hDdeData as usize == CBR_BLOCK {
                        // MS doc is not consistent here
                        // FIXME("CBR_BLOCK returned for WILDCONNECT\n");
                    } else if hDdeData as usize != 0 {
                        let hszp: *mut HSZPAIR =
                            DdeAccessData(hDdeData, core::ptr::null_mut()) as *mut HSZPAIR;
                        if !hszp.is_null() {
                            let mut i: isize = 0;
                            while !(*hszp.offset(i)).hszSvc.is_null()
                                && !(*hszp.offset(i)).hszTopic.is_null()
                            {
                                let pConv3 = WDML_CreateServerConv(
                                    pInstance,
                                    hwndClient,
                                    hwndServer,
                                    (*hszp.offset(i)).hszSvc,
                                    (*hszp.offset(i)).hszTopic,
                                );
                                if !pConv3.is_null() {
                                    if !pcc.is_null() {
                                        (*pConv3).wStatus |= ST_ISLOCAL;
                                    }
                                    WDML_InvokeCallback(
                                        (pInstance).cast(),
                                        XTYP_CONNECT_CONFIRM,
                                        0,
                                        pConv3 as HCONV,
                                        (*hszp.offset(i)).hszTopic,
                                        (*hszp.offset(i)).hszSvc,
                                        core::ptr::null_mut(),
                                        pcc as ULONG_PTR,
                                        self_ as ULONG_PTR,
                                    );
                                }
                                i += 1;
                            }
                            DdeUnaccessData(hDdeData);
                        }
                        if WDML_IsAppOwned(hDdeData) == 0 {
                            DdeFreeDataHandle(hDdeData);
                        }
                    }
                }
            }

            0
        }

        WM_DDE_REQUEST => 0,
        WM_DDE_ADVISE => 0,
        WM_DDE_UNADVISE => 0,
        WM_DDE_EXECUTE => 0,
        WM_DDE_POKE => 0,
        WM_DDE_TERMINATE => 0,
        _ => DefWindowProcW(hwndServer, iMsg, wParam, lParam),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueueRequest  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueueRequest(pConv: *mut WDML_CONV, lParam: LPARAM) -> *mut WDML_XACT {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let pXAct: *mut WDML_XACT;

    UnpackDDElParam(WM_DDE_REQUEST, lParam, &mut uiLo, &mut uiHi);

    pXAct = (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_REQUEST,
        uiLo as UINT,
        WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM),
    )).cast();
    if !pXAct.is_null() {
        (*pXAct).atom = uiHi as ATOM;
    }
    pXAct
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandleRequest  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandleRequest(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let mut hDdeData: HDDEDATA = core::ptr::null_mut();
    let mut fAck: BOOL = TRUE as BOOL;

    if (*(*pConv).instance).CBFflags & CBF_FAIL_REQUESTS == 0 {
        hDdeData = WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_REQUEST,
            (*pXAct).wFmt,
            pConv as HCONV,
            (*pConv).hszTopic,
            (*pXAct).hszItem,
            core::ptr::null_mut(),
            0,
            0,
        );
    }

    match hDdeData as usize {
        0 => {
            // No data returned from the Callback
            fAck = FALSE as BOOL;
        }
        CBR_BLOCK => {
            return WDML_QS_BLOCK;
        }
        _ => {
            let hMem: HGLOBAL = WDML_DataHandle2Global(
                hDdeData,
                TRUE as BOOL,
                FALSE as BOOL,
                FALSE as BOOL,
                FALSE as BOOL,
            );
            if PostMessageW(
                (*pConv).hwndClient,
                WM_DDE_DATA,
                (*pConv).hwndServer as WPARAM,
                ReuseDDElParam(
                    (*pXAct).lParam,
                    WM_DDE_REQUEST,
                    WM_DDE_DATA,
                    hMem as UINT_PTR,
                    (*pXAct).atom as UINT_PTR,
                ),
            ) == 0
            {
                (*(*pConv).instance).lastError = DMLERR_POSTMSG_FAILED;
                DdeFreeDataHandle(hDdeData);
                GlobalFree(hMem);
                fAck = FALSE as BOOL;
            }
        }
    }

    WDML_PostAck(
        (pConv).cast(),
        WDML_SERVER_SIDE,
        0,
        FALSE as BOOL,
        fAck,
        (*pXAct).atom as UINT_PTR,
        (*pXAct).lParam,
        WM_DDE_REQUEST,
    );

    WDML_DecHSZ(((*pConv).instance).cast(), (*pXAct).hszItem);

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueueAdvise  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueueAdvise(pConv: *mut WDML_CONV, lParam: LPARAM) -> *mut WDML_XACT {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let pXAct: *mut WDML_XACT;

    // XTYP_ADVSTART transaction
    if UnpackDDElParam(WM_DDE_ADVISE, lParam, &mut uiLo, &mut uiHi) == 0 {
        return core::ptr::null_mut();
    }

    pXAct = (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_ADVISE,
        0,
        WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM),
    )).cast();
    if !pXAct.is_null() {
        (*pXAct).hMem = uiLo as HGLOBAL;
        (*pXAct).atom = uiHi as ATOM;
    }
    pXAct
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandleAdvise  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandleAdvise(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let uType: UINT;
    let pLink: *mut WDML_LINK;
    let pDdeAdvise: *mut DDEADVISE;
    let mut hDdeData: HDDEDATA = core::ptr::null_mut();
    let mut fAck: BOOL = TRUE as BOOL;

    pDdeAdvise = GlobalLock((*pXAct).hMem) as *mut DDEADVISE;
    uType = XTYP_ADVSTART
        | (if (*pDdeAdvise).fDeferUpd() != 0 { XTYPF_NODATA } else { 0 })
        | (if (*pDdeAdvise).fAckReq() != 0 { XTYPF_ACKREQ } else { 0 });

    if (*(*pConv).instance).CBFflags & CBF_FAIL_ADVISES == 0 {
        hDdeData = WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_ADVSTART,
            (*pDdeAdvise).cfFormat as UINT,
            pConv as HCONV,
            (*pConv).hszTopic,
            (*pXAct).hszItem,
            core::ptr::null_mut(),
            0,
            0,
        );
    }

    match hDdeData as usize {
        0 => {
            // No data returned from the Callback
            fAck = FALSE as BOOL;
        }
        CBR_BLOCK => {
            return WDML_QS_BLOCK;
        }
        _ => {
            // billx: first to see if the link is already created.
            pLink = (WDML_FindLink(
                ((*pConv).instance).cast(),
                pConv as HCONV,
                WDML_SERVER_SIDE,
                (*pXAct).hszItem,
                TRUE as BOOL,
                (*pDdeAdvise).cfFormat as UINT,
            )).cast();

            if !pLink.is_null() {
                (*pLink).transactionType = uType;
            } else {
                WDML_AddLink(
                    ((*pConv).instance).cast(),
                    pConv as HCONV,
                    WDML_SERVER_SIDE,
                    uType,
                    (*pXAct).hszItem,
                    (*pDdeAdvise).cfFormat as UINT,
                );
            }
        }
    }

    GlobalUnlock((*pXAct).hMem);
    if fAck != 0 {
        GlobalFree((*pXAct).hMem);
    }
    (*pXAct).hMem = core::ptr::null_mut();

    WDML_PostAck(
        (pConv).cast(),
        WDML_SERVER_SIDE,
        0,
        FALSE as BOOL,
        fAck,
        (*pXAct).atom as UINT_PTR,
        (*pXAct).lParam,
        WM_DDE_ADVISE,
    );

    WDML_DecHSZ(((*pConv).instance).cast(), (*pXAct).hszItem);

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueueUnadvise  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueueUnadvise(pConv: *mut WDML_CONV, lParam: LPARAM) -> *mut WDML_XACT {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let pXAct: *mut WDML_XACT;

    UnpackDDElParam(WM_DDE_UNADVISE, lParam, &mut uiLo, &mut uiHi);

    pXAct = (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_UNADVISE,
        uiLo as UINT,
        WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM),
    )).cast();
    if !pXAct.is_null() {
        (*pXAct).atom = uiHi as ATOM;
    }
    pXAct
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandleUnadvise  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandleUnadvise(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let pLink: *mut WDML_LINK;

    if (*pXAct).hszItem.is_null() || (*pXAct).wFmt == 0 {
        // ERR("Unsupported yet options (null item or clipboard format)\n");
        return WDML_QS_ERROR;
    }

    pLink = (WDML_FindLink(
        ((*pConv).instance).cast(),
        pConv as HCONV,
        WDML_SERVER_SIDE,
        (*pXAct).hszItem,
        TRUE as BOOL,
        (*pXAct).wFmt,
    )).cast();
    if pLink.is_null() {
        // ERR("Couldn't find link, dropping request\n");
        FreeDDElParam(WM_DDE_UNADVISE, (*pXAct).lParam);
        return WDML_QS_ERROR;
    }

    if (*(*pConv).instance).CBFflags & CBF_FAIL_ADVISES == 0 {
        WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_ADVSTOP,
            (*pXAct).wFmt,
            pConv as HCONV,
            (*pConv).hszTopic,
            (*pXAct).hszItem,
            core::ptr::null_mut(),
            0,
            0,
        );
    }

    WDML_RemoveLink(
        ((*pConv).instance).cast(),
        pConv as HCONV,
        WDML_SERVER_SIDE,
        (*pXAct).hszItem,
        (*pXAct).wFmt,
    );

    // send back ack
    WDML_PostAck(
        (pConv).cast(),
        WDML_SERVER_SIDE,
        0,
        FALSE as BOOL,
        TRUE as BOOL,
        (*pXAct).atom as UINT_PTR,
        (*pXAct).lParam,
        WM_DDE_UNADVISE,
    );

    WDML_DecHSZ(((*pConv).instance).cast(), (*pXAct).hszItem);

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueueExecute  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueueExecute(pConv: *mut WDML_CONV, lParam: LPARAM) -> *mut WDML_XACT {
    let pXAct: *mut WDML_XACT;

    pXAct = (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_EXECUTE,
        0,
        core::ptr::null_mut(),
    )).cast();
    if !pXAct.is_null() {
        (*pXAct).hMem = lParam as HGLOBAL;
    }
    pXAct
}

// data_looks_unicode (static)
unsafe fn data_looks_unicode(data: *const WCHAR, size: DWORD) -> BOOL {
    if size % core::mem::size_of::<WCHAR>() as DWORD != 0 {
        return FALSE as BOOL;
    }
    let n = size / core::mem::size_of::<WCHAR>() as DWORD;
    let mut i: DWORD = 0;
    while i < n {
        if *data.add(i as usize) > 255 {
            return FALSE as BOOL;
        }
        i += 1;
    }
    TRUE as BOOL
}

// convert data to Unicode, unless it looks like it's already Unicode (static)
unsafe fn map_A_to_W(instance: DWORD, ptr: *mut c_void, mut size: DWORD) -> HDDEDATA {
    let ret: HDDEDATA;
    let len: DWORD;

    if data_looks_unicode(ptr as *const WCHAR, size) == 0 {
        let end = memchr(ptr, 0, size as usize);
        if !end.is_null() {
            size = (end as usize + 1 - ptr as usize) as DWORD;
        }
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            ptr as LPCSTR,
            size as INT,
            core::ptr::null_mut(),
            0,
        ) as DWORD;
        ret = DdeCreateDataHandle(
            instance,
            core::ptr::null_mut(),
            len * core::mem::size_of::<WCHAR>() as DWORD,
            0,
            core::ptr::null_mut(),
            CF_TEXT,
            0,
        );
        MultiByteToWideChar(
            CP_ACP,
            0,
            ptr as LPCSTR,
            size as INT,
            DdeAccessData(ret, core::ptr::null_mut()) as LPWSTR,
            len as INT,
        );
    } else {
        ret = DdeCreateDataHandle(
            instance,
            ptr as LPBYTE,
            size,
            0,
            core::ptr::null_mut(),
            CF_TEXT,
            0,
        );
    }

    ret
}

// convert data to ASCII, unless it looks like it's not in Unicode format (static)
unsafe fn map_W_to_A(instance: DWORD, ptr: *mut c_void, mut size: DWORD) -> HDDEDATA {
    let ret: HDDEDATA;
    let len: DWORD;

    if data_looks_unicode(ptr as *const WCHAR, size) != 0 {
        size /= core::mem::size_of::<WCHAR>() as DWORD;
        let end = memchrW(ptr as *const WCHAR, 0, size as usize);
        if !end.is_null() {
            size = (end as usize + 1 - ptr as usize) as DWORD / core::mem::size_of::<WCHAR>() as DWORD;
        }
        len = WideCharToMultiByte(
            CP_ACP,
            0,
            ptr as LPCWSTR,
            size as INT,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
            core::ptr::null_mut(),
        ) as DWORD;
        ret = DdeCreateDataHandle(
            instance,
            core::ptr::null_mut(),
            len,
            0,
            core::ptr::null_mut(),
            CF_TEXT,
            0,
        );
        WideCharToMultiByte(
            CP_ACP,
            0,
            ptr as LPCWSTR,
            size as INT,
            DdeAccessData(ret, core::ptr::null_mut()) as LPSTR,
            len as INT,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    } else {
        ret = DdeCreateDataHandle(
            instance,
            ptr as LPBYTE,
            size,
            0,
            core::ptr::null_mut(),
            CF_TEXT,
            0,
        );
    }

    ret
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandleExecute  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandleExecute(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let mut hDdeData: HDDEDATA = DDE_FNOTPROCESSED as HDDEDATA;
    let mut fAck: BOOL = FALSE as BOOL;
    let mut fBusy: BOOL = FALSE as BOOL;

    if (*(*pConv).instance).CBFflags & CBF_FAIL_EXECUTES == 0 {
        let ptr: LPVOID = GlobalLock((*pXAct).hMem);
        let size: DWORD = GlobalSize((*pXAct).hMem) as DWORD;

        if !ptr.is_null() {
            if (*(*pConv).instance).unicode != 0 {
                // Unicode server, try to map A->W
                hDdeData = map_A_to_W((*(*pConv).instance).instanceID, ptr, size);
            } else if IsWindowUnicode((*pConv).hwndClient) == 0 {
                // ASCII server and client, try to map W->A
                hDdeData = map_W_to_A((*(*pConv).instance).instanceID, ptr, size);
            } else {
                hDdeData = DdeCreateDataHandle(
                    (*(*pConv).instance).instanceID,
                    ptr as LPBYTE,
                    size,
                    0,
                    core::ptr::null_mut(),
                    CF_TEXT,
                    0,
                );
            }
            GlobalUnlock((*pXAct).hMem);
        }
        hDdeData = WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_EXECUTE,
            0,
            pConv as HCONV,
            (*pConv).hszTopic,
            core::ptr::null_mut(),
            hDdeData,
            0,
            0,
        );
    }

    match hDdeData as usize {
        CBR_BLOCK => {
            return WDML_QS_BLOCK;
        }
        DDE_FACK => {
            fAck = TRUE as BOOL;
        }
        DDE_FBUSY => {
            fBusy = TRUE as BOOL;
        }
        DDE_FNOTPROCESSED => {
            // break
        }
        _ => {
            // FIXME("Unsupported returned value %p\n", hDdeData);
            // fall through (no extra action)
        }
    }
    WDML_PostAck(
        (pConv).cast(),
        WDML_SERVER_SIDE,
        0,
        fBusy,
        fAck,
        (*pXAct).hMem as UINT_PTR,
        0,
        0,
    );

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueuePoke  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueuePoke(pConv: *mut WDML_CONV, lParam: LPARAM) -> *mut WDML_XACT {
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let pXAct: *mut WDML_XACT;

    UnpackDDElParam(WM_DDE_POKE, lParam, &mut uiLo, &mut uiHi);

    pXAct = (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_POKE,
        0,
        WDML_MakeHszFromAtom((*pConv).instance, uiHi as ATOM),
    )).cast();
    if !pXAct.is_null() {
        (*pXAct).atom = uiHi as ATOM;
        (*pXAct).hMem = uiLo as HGLOBAL;
    }
    pXAct
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandlePoke  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandlePoke(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let pDdePoke: *mut DDEPOKE;
    let hDdeData: HDDEDATA;
    let mut fBusy: BOOL = FALSE as BOOL;
    let mut fAck: BOOL = FALSE as BOOL;

    pDdePoke = GlobalLock((*pXAct).hMem) as *mut DDEPOKE;
    if pDdePoke.is_null() {
        return WDML_QS_ERROR;
    }

    if (*(*pConv).instance).CBFflags & CBF_FAIL_POKES == 0 {
        // FIELD_OFFSET(DDEPOKE, Value)
        let value_off = core::mem::offset_of!(DDEPOKE, Value) as DWORD;
        hDdeData = DdeCreateDataHandle(
            (*(*pConv).instance).instanceID,
            (*pDdePoke).Value.as_mut_ptr(),
            GlobalSize((*pXAct).hMem) as DWORD - value_off,
            0,
            core::ptr::null_mut(),
            (*pDdePoke).cfFormat as UINT,
            0,
        );
        if !hDdeData.is_null() {
            let hDdeDataOut: HDDEDATA = WDML_InvokeCallback(
                ((*pConv).instance).cast(),
                XTYP_POKE,
                (*pDdePoke).cfFormat as UINT,
                pConv as HCONV,
                (*pConv).hszTopic,
                (*pXAct).hszItem,
                hDdeData,
                0,
                0,
            );
            match hDdeDataOut as usize {
                DDE_FACK => {
                    fAck = TRUE as BOOL;
                }
                DDE_FBUSY => {
                    fBusy = TRUE as BOOL;
                }
                DDE_FNOTPROCESSED => {
                    // break
                }
                _ => {
                    // FIXME("Unsupported returned value %p\n", hDdeDataOut);
                    // fall through
                }
            }
            DdeFreeDataHandle(hDdeData);
        }
    }
    GlobalUnlock((*pXAct).hMem);

    if fAck == 0 {
        GlobalFree((*pXAct).hMem);
    }
    WDML_PostAck(
        (pConv).cast(),
        WDML_SERVER_SIDE,
        0,
        fBusy,
        fAck,
        (*pXAct).atom as UINT_PTR,
        (*pXAct).lParam,
        WM_DDE_POKE,
    );

    WDML_DecHSZ(((*pConv).instance).cast(), (*pXAct).hszItem);

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerQueueTerminate  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerQueueTerminate(pConv: *mut WDML_CONV, _lParam: LPARAM) -> *mut WDML_XACT {
    (WDML_AllocTransaction(
        ((*pConv).instance).cast(),
        WM_DDE_TERMINATE,
        0,
        core::ptr::null_mut(),
    )).cast()
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandleTerminate  (static)
// ═══════════════════════════════════════════════════════════════════════════
unsafe fn WDML_ServerHandleTerminate(pConv: *mut WDML_CONV, _pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    // billx: two things to remove: the conv, and associated links.
    if (*(*pConv).instance).CBFflags & CBF_SKIP_DISCONNECTS == 0 {
        WDML_InvokeCallback(
            ((*pConv).instance).cast(),
            XTYP_DISCONNECT,
            0,
            pConv as HCONV,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            if (*pConv).wStatus & ST_ISSELF != 0 { 1 } else { 0 },
        );
    }
    PostMessageW(
        (*pConv).hwndClient,
        WM_DDE_TERMINATE,
        (*pConv).hwndServer as WPARAM,
        0,
    );
    WDML_RemoveConv((pConv).cast(), WDML_SERVER_SIDE);

    WDML_QS_HANDLED
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerHandle  (non-static → exported, cdecl "C")
// ═══════════════════════════════════════════════════════════════════════════
#[no_mangle]
pub unsafe extern "C" fn WDML_ServerHandle(pConv: *mut WDML_CONV, pXAct: *mut WDML_XACT) -> WDML_QUEUE_STATE {
    let mut qs: WDML_QUEUE_STATE = WDML_QS_ERROR;

    match (*pXAct).ddeMsg {
        WM_DDE_INITIATE => {
            // FIXME("WM_DDE_INITIATE shouldn't be there!\n");
        }
        WM_DDE_REQUEST => {
            qs = WDML_ServerHandleRequest(pConv, pXAct);
        }
        WM_DDE_ADVISE => {
            qs = WDML_ServerHandleAdvise(pConv, pXAct);
        }
        WM_DDE_UNADVISE => {
            qs = WDML_ServerHandleUnadvise(pConv, pXAct);
        }
        WM_DDE_EXECUTE => {
            qs = WDML_ServerHandleExecute(pConv, pXAct);
        }
        WM_DDE_POKE => {
            qs = WDML_ServerHandlePoke(pConv, pXAct);
        }
        WM_DDE_TERMINATE => {
            qs = WDML_ServerHandleTerminate(pConv, pXAct);
        }
        WM_DDE_ACK => {
            // WARN("Shouldn't receive a ACK message. Ignoring it\n");
        }
        _ => {
            // FIXME("Unsupported message %d\n", pXAct->ddeMsg);
        }
    }
    qs
}

// ═══════════════════════════════════════════════════════════════════════════
//  WDML_ServerConvProc  (static CALLBACK) — kept private; addr → RegisterClassEx.
// ═══════════════════════════════════════════════════════════════════════════
unsafe extern "system" fn WDML_ServerConvProc(
    hwndServer: HWND,
    iMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pInstance: *mut WDML_INSTANCE;
    let mut pConv: *mut WDML_CONV;
    let mut pXAct: *mut WDML_XACT = core::ptr::null_mut();

    if iMsg == WM_DESTROY {
        pConv = (WDML_GetConvFromWnd(hwndServer)).cast();
        if !pConv.is_null() && (*pConv).wStatus & ST_TERMINATED == 0 {
            WDML_ServerHandleTerminate(pConv, core::ptr::null_mut());
        }
    }
    if iMsg < WM_DDE_FIRST || iMsg > WM_DDE_LAST {
        return if IsWindowUnicode(hwndServer) != 0 {
            DefWindowProcW(hwndServer, iMsg, wParam, lParam)
        } else {
            DefWindowProcA(hwndServer, iMsg, wParam, lParam)
        };
    }

    pInstance = (WDML_GetInstanceFromWnd(hwndServer)).cast();
    pConv = (WDML_GetConvFromWnd(hwndServer)).cast();

    if pConv.is_null() {
        // ERR("Got a message on a not known conversation, dropping request\n");
        return 0;
    }
    if (*pConv).hwndClient != WIN_GetFullHandle(wParam as HWND) || (*pConv).hwndServer != hwndServer {
        // ERR("mismatch between C/S windows and conversation\n");
        return 0;
    }
    if (*pConv).instance != pInstance || (*pConv).instance.is_null() {
        // ERR("mismatch in instances\n");
        return 0;
    }

    match iMsg {
        WM_DDE_INITIATE => {
            // FIXME("WM_DDE_INITIATE message received!\n");
        }
        WM_DDE_REQUEST => {
            pXAct = WDML_ServerQueueRequest(pConv, lParam);
        }
        WM_DDE_ADVISE => {
            pXAct = WDML_ServerQueueAdvise(pConv, lParam);
        }
        WM_DDE_UNADVISE => {
            pXAct = WDML_ServerQueueUnadvise(pConv, lParam);
        }
        WM_DDE_EXECUTE => {
            pXAct = WDML_ServerQueueExecute(pConv, lParam);
        }
        WM_DDE_POKE => {
            pXAct = WDML_ServerQueuePoke(pConv, lParam);
        }
        WM_DDE_TERMINATE => {
            pXAct = WDML_ServerQueueTerminate(pConv, lParam);
        }
        WM_DDE_ACK => {
            // WARN("Shouldn't receive a ACK message. Ignoring it\n");
        }
        _ => {
            // FIXME("Unsupported message %x\n", iMsg);
        }
    }

    if !pXAct.is_null() {
        (*pXAct).lParam = lParam;

        if (*pConv).wStatus & ST_BLOCKED != 0
            || WDML_ServerHandle(pConv, pXAct) == WDML_QS_BLOCK
        {
            WDML_QueueTransaction((pConv).cast(), (pXAct).cast());
        } else {
            WDML_FreeTransaction((pInstance).cast(), (pXAct).cast(), TRUE as BOOL);
        }
    } else {
        (*(*pConv).instance).lastError = DMLERR_MEMORY_ERROR;
    }

    0
}
