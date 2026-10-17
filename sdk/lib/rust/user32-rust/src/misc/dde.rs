//! Translated from win32ss/user/user32/misc/dde.c — Dynamic Data Exchange.
//!
//! Cohabitation Rust replacement for the user32 dde.c translation unit.

use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::message::GlobalAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;

// LOWORD/HIWORD/MAKELONG live in crate::internal; ZwCallbackReturn too.
use crate::internal::{HIWORD, MAKELONG, ZwCallbackReturn};
// memcpy (msvcrt) lives in crate::winbase.
use crate::winbase::memcpy;

// ── kernel32 DDElParam packing API (WINAPI exports, defined in ddemisc.c). On i386
//   these are stdcall (`_Name@N`) — declare extern "system".
// ── kernel32 Global* heap API (WINAPI). GlobalAlloc/Lock/Size/Free/Unlock.
extern "system" {
    fn UnpackDDElParam(msg: UINT, lParam: LPARAM, uiLo: *mut UINT_PTR, uiHi: *mut UINT_PTR) -> BOOL;
    fn PackDDElParam(msg: UINT, uiLo: UINT_PTR, uiHi: UINT_PTR) -> LPARAM;
    fn FreeDDElParam(msg: UINT, lParam: LPARAM) -> BOOL;

    fn GlobalSize(hMem: HGLOBAL) -> SIZE_T;
    fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
}

// ── DdeAddPair / DdeGetPair are FASTCALL (defined in windows/message.c).
//   FASTCALL → extern "fastcall" on i386, extern "C" elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DdeAddPair(ClientMem: HGLOBAL, ServerMem: HGLOBAL) -> BOOL;
    fn DdeGetPair(ServerMem: HGLOBAL) -> HGLOBAL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DdeAddPair(ClientMem: HGLOBAL, ServerMem: HGLOBAL) -> BOOL;
    fn DdeGetPair(ServerMem: HGLOBAL) -> HGLOBAL;
}

// ── Constants not present in the win32k_sys glob (psdk/dde.h, winbase.h, ntstatus.h).
const WM_DDE_ACK: UINT = 0x3E4;
const WM_DDE_DATA: UINT = 0x3E5;
const WM_DDE_POKE: UINT = 0x3E7;
const WM_DDE_ADVISE: UINT = 0x3E2;
const WM_DDE_EXECUTE: UINT = 0x3E8;

const GMEM_MOVEABLE: UINT = 0x0002;
const GMEM_DDESHARE: UINT = 0x2000;

const STATUS_SUCCESS: NTSTATUS = 0x0000_0000;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;

// ── DDE data-object layouts (psdk/dde.h). Reproduced #[repr(C)] so the FIELD_OFFSET /
//   sizeof size-guards match the C ABI exactly.
//   DDEDATA: { WORD bitfields; short cfFormat; BYTE Value[1]; } → offsetof(Value) == 4.
#[repr(C)]
struct DDEDATA {
    _flags: u16,
    cfFormat: i16,
    Value: [u8; 1],
}
//   DDEADVISE: { WORD bitfields; short cfFormat; } → sizeof == 4.
#[repr(C)]
struct DDEADVISE {
    _flags: u16,
    cfFormat: i16,
}
//   DDEPOKE: { WORD bitfields; short cfFormat; BYTE Value[1]; } → offsetof(Value) == 4.
#[repr(C)]
struct DDEPOKE {
    _flags: u16,
    cfFormat: i16,
    Value: [u8; 1],
}

// ── The kernel DDE post/get callback argument block (win32ss/include/callback.h).
//   The C struct embeds an anonymous `MSG;` member; bindgen flattened it to padding,
//   so reproduce the faithful #[repr(C)] layout here (INT Type; MSG; int size; PVOID
//   pvData; BYTE buffer[1];). MSG = { HWND hwnd; UINT message; WPARAM wParam;
//   LPARAM lParam; DWORD time; POINTL pt; }.
#[repr(C)]
struct DDEPOSTGET_CALLBACK_ARGUMENTS {
    Type: INT,
    // ----- inlined anonymous MSG -----
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINTL,
    // ----- end MSG -----
    size: ::core::ffi::c_int,
    pvData: PVOID,
    buffer: [BYTE; 1],
}
type PDDEPOSTGET_CALLBACK_ARGUMENTS = *mut DDEPOSTGET_CALLBACK_ARGUMENTS;

/* description of the data fields that need to be packed along with a sent message */
#[repr(C)]
struct packed_message {
    count: i32,
    data: *const c_void,
    size: i32,
}

/* add a data field to a packed message */
#[inline]
unsafe fn push_data(data: *mut packed_message, ptr: *const c_void, size: i32) {
    (*data).data = ptr;
    (*data).size = size;
    (*data).count += 1;
}

/* pack a pointer into a 32/64 portable format */
#[inline]
unsafe fn pack_ptr(ptr: *const c_void) -> ULONGLONG {
    (ptr as ULONG_PTR) as ULONGLONG
}

/* unpack a potentially 64-bit pointer, returning 0 when truncated */
#[inline]
unsafe fn unpack_ptr(ptr64: ULONGLONG) -> *mut c_void {
    if (ptr64 as ULONG_PTR) as ULONGLONG != ptr64 {
        return core::ptr::null_mut();
    }
    (ptr64 as ULONG_PTR) as *mut c_void
}

/***********************************************************************
 *		post_dde_message
 *
 * Post a DDE message
 */
#[no_mangle]
pub unsafe extern "C" fn post_dde_message(
    data: *mut packed_message,
    message: UINT,
    lParam: LPARAM,
    lp: *mut LPARAM,
) -> BOOL {
    let mut ptr: *mut c_void = core::ptr::null_mut();
    let mut size: i32 = 0;
    let mut uiLo: UINT_PTR = 0;
    let mut uiHi: UINT_PTR = 0;
    let mut hunlock: HGLOBAL = core::ptr::null_mut();
    let hpack: ULONGLONG;

    if UnpackDDElParam(message, lParam, &mut uiLo, &mut uiHi) == 0 {
        // ERR("Unpack failed %x\n",message);
        return FALSE as BOOL;
    }

    *lp = lParam;
    match message {
        /* DDE messages which don't require packing are:
         * WM_DDE_INITIATE
         * WM_DDE_TERMINATE
         * WM_DDE_REQUEST
         * WM_DDE_UNADVISE
         */
        WM_DDE_ACK => {
            if HIWORD(uiHi as DWORD) != 0 {
                /* uiHi should contain a hMem from WM_DDE_EXECUTE */
                let h: HGLOBAL = DdeGetPair(uiHi as HANDLE);
                if !h.is_null() {
                    hpack = pack_ptr(h as *const c_void);
                    /* send back the value of h on the other side */
                    push_data(
                        data,
                        core::ptr::addr_of!(hpack) as *const c_void,
                        core::mem::size_of::<ULONGLONG>() as i32,
                    );
                    *lp = uiLo as LPARAM;
                }
            } else {
                /* uiHi should contain either an atom or 0 */
                *lp = MAKELONG(uiLo as WORD, uiHi as WORD) as LPARAM;
            }
        }
        WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
            size = 0;
            if uiLo != 0 {
                size = GlobalSize(uiLo as HGLOBAL) as i32;
                if (message == WM_DDE_ADVISE && (size as usize) < core::mem::size_of::<DDEADVISE>())
                    || (message == WM_DDE_DATA
                        && (size as usize) < field_offset_ddedata_value())
                    || (message == WM_DDE_POKE
                        && (size as usize) < field_offset_ddepoke_value())
                {
                    return FALSE as BOOL;
                }
            } else if message != WM_DDE_DATA {
                return FALSE as BOOL;
            }

            *lp = uiHi as LPARAM;
            if uiLo != 0 {
                ptr = GlobalLock(uiLo as HGLOBAL);
                if !ptr.is_null() {
                    // DDEDATA *dde_data = ptr; (used only by the TRACE below, a no-op here)
                    let _dde_data = ptr as *mut DDEDATA;
                    push_data(data, ptr as *const c_void, size);
                    hunlock = uiLo as HGLOBAL;
                }
            }
        }
        WM_DDE_EXECUTE => {
            if lParam != 0 {
                ptr = GlobalLock(lParam as HGLOBAL);
                if !ptr.is_null() {
                    size = GlobalSize(lParam as HGLOBAL) as i32;
                    push_data(data, ptr as *const c_void, size);
                    /* so that the other side can send it back on ACK */
                    *lp = lParam;
                    hunlock = lParam as HGLOBAL;
                }
            }
        }
        _ => {}
    }

    FreeDDElParam(message, lParam);

    if !hunlock.is_null() {
        GlobalUnlock(hunlock);
    }

    let _ = size;
    let _ = ptr;
    TRUE as BOOL
}

// FIELD_OFFSET(DDEDATA, Value) / FIELD_OFFSET(DDEPOKE, Value) — both == 4 with the
// faithful #[repr(C)] layout above. Computed rather than literal so it tracks the type.
#[inline]
fn field_offset_ddedata_value() -> usize {
    core::mem::offset_of!(DDEDATA, Value)
}
#[inline]
fn field_offset_ddepoke_value() -> usize {
    core::mem::offset_of!(DDEPOKE, Value)
}

/***********************************************************************
 *		unpack_dde_message
 *
 * Unpack a posted DDE message received from another process.
 */
#[no_mangle]
pub unsafe extern "C" fn unpack_dde_message(
    _hwnd: HWND,
    message: UINT,
    lparam: *mut LPARAM,
    buffer: PVOID,
    size: i32,
) -> BOOL {
    let mut uiLo: UINT_PTR;
    let uiHi: UINT_PTR;
    let mut hMem: HGLOBAL = core::ptr::null_mut();
    let ptr: *mut c_void;

    match message {
        WM_DDE_ACK => {
            if size != 0 {
                let mut hpack: ULONGLONG = 0;
                /* hMem is being passed */
                if size as usize != core::mem::size_of::<ULONGLONG>() {
                    return FALSE as BOOL;
                }
                if buffer.is_null() {
                    return FALSE as BOOL;
                }
                uiLo = *lparam as UINT_PTR;
                memcpy(
                    core::ptr::addr_of_mut!(hpack) as *mut c_void,
                    buffer as *const c_void,
                    size as usize,
                );
                hMem = unpack_ptr(hpack);
                uiHi = hMem as UINT_PTR;
            } else {
                uiLo = LOWORD_lparam(*lparam) as UINT_PTR;
                uiHi = HIWORD(*lparam as DWORD) as UINT_PTR;
            }
            *lparam = PackDDElParam(WM_DDE_ACK, uiLo, uiHi);
        }
        WM_DDE_ADVISE | WM_DDE_DATA | WM_DDE_POKE => {
            if buffer.is_null() && message != WM_DDE_DATA {
                return FALSE as BOOL;
            }
            uiHi = *lparam as UINT_PTR;
            if size != 0 {
                hMem = GlobalAlloc(GMEM_MOVEABLE | GMEM_DDESHARE, size as SIZE_T);
                if hMem.is_null() {
                    return FALSE as BOOL;
                }
                ptr = GlobalLock(hMem);
                if !ptr.is_null() {
                    memcpy(ptr, buffer as *const c_void, size as usize);
                    GlobalUnlock(hMem);
                } else {
                    GlobalFree(hMem);
                    return FALSE as BOOL;
                }
            }
            uiLo = hMem as UINT_PTR;

            *lparam = PackDDElParam(message, uiLo, uiHi);
        }
        WM_DDE_EXECUTE => {
            if size != 0 {
                if buffer.is_null() {
                    return FALSE as BOOL;
                }
                hMem = GlobalAlloc(GMEM_MOVEABLE | GMEM_DDESHARE, size as SIZE_T);
                if hMem.is_null() {
                    return FALSE as BOOL;
                }
                ptr = GlobalLock(hMem);
                if !ptr.is_null() {
                    memcpy(ptr, buffer as *const c_void, size as usize);
                    GlobalUnlock(hMem);
                    if DdeAddPair(*lparam as HGLOBAL, hMem) == 0 {
                        GlobalFree(hMem);
                        return FALSE as BOOL;
                    }
                } else {
                    GlobalFree(hMem);
                    return FALSE as BOOL;
                }
            } else {
                return FALSE as BOOL;
            }

            *lparam = hMem as LPARAM;
        }
        _ => {}
    }
    TRUE as BOOL
}

// LOWORD over an LPARAM (DWORD-truncating, as in the C macro on *lparam).
#[inline]
fn LOWORD_lparam(l: LPARAM) -> WORD {
    (l as DWORD & 0xffff) as WORD
}

//
//  DDE Post kernel callback.
//
#[no_mangle]
pub unsafe extern "system" fn User32CallDDEPostFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let mut data: packed_message = core::mem::zeroed();
    let Ret: BOOL;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Common: PDDEPOSTGET_CALLBACK_ARGUMENTS = Arguments as PDDEPOSTGET_CALLBACK_ARGUMENTS;

    data.data = core::ptr::null();
    data.size = 0;
    Ret = post_dde_message(
        &mut data,
        (*Common).message,
        (*Common).lParam,
        core::ptr::addr_of_mut!((*Common).lParam),
    );

    if Ret != 0 {
        (*Common).pvData = data.data as PVOID;
        (*Common).size = data.size;
    } else {
        // ERR("DDE Post CB Return bad msg ...")
        Status = STATUS_UNSUCCESSFUL;
    }

    ZwCallbackReturn(Arguments, ArgumentLength, Status)
}

//
//  DDE Get/Peek kernel callback.
//
#[no_mangle]
pub unsafe extern "system" fn User32CallDDEGetFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let Ret: BOOL;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Common: PDDEPOSTGET_CALLBACK_ARGUMENTS = Arguments as PDDEPOSTGET_CALLBACK_ARGUMENTS;

    Ret = unpack_dde_message(
        (*Common).hwnd,
        (*Common).message,
        core::ptr::addr_of_mut!((*Common).lParam),
        core::ptr::addr_of_mut!((*Common).buffer) as PVOID,
        (*Common).size,
    );

    if Ret == 0 {
        // ERR("DDE Get CB Return bad msg ...")
        Status = STATUS_UNSUCCESSFUL;
    }
    ZwCallbackReturn(Arguments, ArgumentLength, Status)
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn DdeGetQualityOfService(
    _hWnd: HWND,
    _Reserved: DWORD,
    _pqosPrev: PSECURITY_QUALITY_OF_SERVICE,
) -> BOOL {
    // UNIMPLEMENTED;
    FALSE as BOOL
}
