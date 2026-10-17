//! SHAppBarMessage — translation of dll/win32/shell32/appbar.c.
//!
//! Single WINAPI export `SHAppBarMessage` (i386 → `_SHAppBarMessage@8`), plus two
//! private static helpers (AppBar_CopyIn / AppBar_CopyOut) kept as non-exported unsafe
//! fns. Builds the private APPBAR_COMMAND interop blob and ships it to Shell_TrayWnd via
//! WM_COPYDATA, using shlwapi shared-memory (SHAllocShared/SHLockShared/...) for the
//! out-of-band output buffer. TRACE/WARN/ERR are dropped (no observable behavior).

use crate::types::*;
// One declaration per external symbol: island.rs.
use crate::island::FindWindowW;
use crate::island::SendMessageW;
use core::mem::size_of;
use core::ptr;

// ── message-loop / WM types not in types.rs ────────────────────────────────────────────
type WPARAM = UINT_PTR; // winuser.h
type LPARAM = LONG_PTR; // winuser.h
type LRESULT = LONG_PTR; // winuser.h
type LONGLONG = i64; // winnt.h

// ── constants pinned from SDK headers ──────────────────────────────────────────────────
const ABM_QUERYPOS: DWORD = 2; // shellapi.h
const ABM_SETPOS: DWORD = 3; // shellapi.h
const ABM_GETTASKBARPOS: DWORD = 5; // shellapi.h
const WM_COPYDATA: UINT = 74; // winuser.h (0x4A)
const TABDMC_APPBAR: ULONG_PTR = 0; // reactos/undocshell.h

// ── struct types defined file-locally (faithful #[repr(C)]) ────────────────────────────

/// RECT — windef.h.
#[repr(C)]
#[derive(Copy, Clone)]
struct RECT {
    left: LONG,
    top: LONG,
    right: LONG,
    bottom: LONG,
}

/// APPBARDATA (_AppBarData) — shellapi.h. The public, caller-facing struct.
#[repr(C)]
#[derive(Copy, Clone)]
struct APPBARDATA {
    cbSize: DWORD,
    hWnd: HWND,
    uCallbackMessage: UINT,
    uEdge: UINT,
    rc: RECT,
    lParam: LPARAM,
}
type PAPPBARDATA = *mut APPBARDATA;

/// APPBARDATAINTEROP (tagAPPBARDATAINTEROP) — reactos/undocshell.h, under <pshpack8.h>.
/// Width-stable 32→64 interop image: hWnd is a truncated UINT32, lParam a LONGLONG.
/// All field offsets are identical whether i64 aligns to 4 or 8 (lParam64 lands at 32,
/// already 8-aligned), so #[repr(C)] reproduces the C layout (sizeof == 40).
#[repr(C)]
#[derive(Copy, Clone)]
struct APPBARDATAINTEROP {
    cbSize: DWORD,
    hWnd32: u32,
    uCallbackMessage: UINT,
    uEdge: UINT,
    rc: RECT,
    lParam64: LONGLONG,
}

/// APPBAR_OUTPUT — reactos/undocshell.h: UINT64 on _WIN64, HANDLE otherwise (the handle
/// returned by shlwapi!SHAllocShared, width-stable across the 32↔64 boundary).
#[cfg(target_pointer_width = "64")]
type APPBAR_OUTPUT = u64;
#[cfg(target_pointer_width = "32")]
type APPBAR_OUTPUT = HANDLE;

/// APPBAR_COMMAND (tagAPPBAR_COMMAND) — reactos/undocshell.h, under <pshpack8.h>.
/// C_ASSERT: sizeof == 0x38 (32-bit) / 0x40 (64-bit). The trailing i64 in `abd` gives the
/// struct 8-byte alignment on the Windows ABI, padding it to those exact sizes.
#[repr(C)]
#[derive(Copy, Clone)]
struct APPBAR_COMMAND {
    abd: APPBARDATAINTEROP,
    dwMessage: DWORD,
    hOutput: APPBAR_OUTPUT,
    dwProcessId: DWORD,
}

/// COPYDATASTRUCT (tagCOPYDATASTRUCT) — winuser.h.
#[repr(C)]
#[derive(Copy, Clone)]
struct COPYDATASTRUCT {
    dwData: ULONG_PTR,
    cbData: DWORD,
    lpData: PVOID,
}

// ── imports (resolved from shell32's import libs at link) ───────────────────────────────
extern "system" {
    // shlwapi
    fn SHAllocShared(pvData: LPCVOID, dwSize: DWORD, dwProcessId: DWORD) -> HANDLE;
    fn SHLockShared(hData: HANDLE, dwProcessId: DWORD) -> PVOID;
    fn SHUnlockShared(pvData: PVOID) -> BOOL;
    fn SHFreeShared(hData: HANDLE, dwProcessId: DWORD) -> BOOL;
    // user32
    // kernel32
    fn GetCurrentProcessId() -> DWORD;
}

// ── inline cast helpers (HandleToUlong / UlongToHandle — winnt.h) ───────────────────────

/// HandleToUlong: truncate a HANDLE to ULONG. "Truncated on x64, as on Windows!"
#[inline]
fn handle_to_ulong(h: HANDLE) -> u32 {
    (h as usize) as u32
}

/// UlongToHandle: zero-extend a ULONG to a HANDLE.
#[inline]
fn ulong_to_handle(u: u32) -> HANDLE {
    (u as usize) as HANDLE
}

/// (APPBAR_OUTPUT)hMem — store a HANDLE into the width-stable output slot.
#[cfg(target_pointer_width = "64")]
#[inline]
fn handle_to_output(h: HANDLE) -> APPBAR_OUTPUT {
    (h as usize) as u64
}
#[cfg(target_pointer_width = "32")]
#[inline]
fn handle_to_output(h: HANDLE) -> APPBAR_OUTPUT {
    h
}

/// (HANDLE)cmd.hOutput — recover the HANDLE from the output slot.
#[cfg(target_pointer_width = "64")]
#[inline]
fn output_to_handle(o: APPBAR_OUTPUT) -> HANDLE {
    (o as usize) as HANDLE
}
#[cfg(target_pointer_width = "32")]
#[inline]
fn output_to_handle(o: APPBAR_OUTPUT) -> HANDLE {
    o
}

/// `if (cmd.hOutput)` — non-NULL / non-zero test on the output slot.
#[cfg(target_pointer_width = "64")]
#[inline]
fn output_is_set(o: APPBAR_OUTPUT) -> bool {
    o != 0
}
#[cfg(target_pointer_width = "32")]
#[inline]
fn output_is_set(o: APPBAR_OUTPUT) -> bool {
    !o.is_null()
}

/// CopyMemory (RtlCopyMemory / memmove). Buffers here are always distinct, but use a
/// move-safe copy to match the C semantics exactly.
#[inline]
unsafe fn copy_memory(dst: PVOID, src: LPCVOID, len: usize) {
    ptr::copy(src as *const u8, dst as *mut u8, len);
}

// ── private static helpers (C: static, cdecl — kept non-exported) ───────────────────────

unsafe fn appbar_copyin(pv_src: LPCVOID, dw_size: usize, dw_process_id: DWORD) -> HANDLE {
    let h_mem = SHAllocShared(ptr::null(), dw_size as DWORD, dw_process_id);
    if h_mem.is_null() {
        return ptr::null_mut();
    }

    let pv_dest = SHLockShared(h_mem, dw_process_id);
    if pv_dest.is_null() {
        SHFreeShared(h_mem, dw_process_id);
        return ptr::null_mut();
    }

    copy_memory(pv_dest, pv_src, dw_size);
    SHUnlockShared(pv_dest);
    h_mem
}

unsafe fn appbar_copyout(
    h_output: HANDLE,
    pv_dest: PVOID,
    cb_dest: usize,
    dw_process_id: DWORD,
) -> BOOL {
    let pv_src = SHLockShared(h_output, dw_process_id);
    if !pv_src.is_null() {
        copy_memory(pv_dest, pv_src, cb_dest);
        SHUnlockShared(pv_src);
    }

    SHFreeShared(h_output, dw_process_id);
    if pv_src.is_null() {
        FALSE
    } else {
        TRUE
    }
}

/// *************************************************************************
/// SHAppBarMessage            [SHELL32.@]
#[no_mangle]
pub unsafe extern "system" fn SHAppBarMessage(dwMessage: DWORD, pData: PAPPBARDATA) -> UINT_PTR {
    // L"Shell_TrayWnd"
    const SHELL_TRAYWND: [WCHAR; 14] = [
        b'S' as WCHAR,
        b'h' as WCHAR,
        b'e' as WCHAR,
        b'l' as WCHAR,
        b'l' as WCHAR,
        b'_' as WCHAR,
        b'T' as WCHAR,
        b'r' as WCHAR,
        b'a' as WCHAR,
        b'y' as WCHAR,
        b'W' as WCHAR,
        b'n' as WCHAR,
        b'd' as WCHAR,
        0,
    ];

    let h_tray_wnd = FindWindowW(SHELL_TRAYWND.as_ptr(), ptr::null());
    let cb_size = ptr::read(ptr::addr_of!((*pData).cbSize));
    if h_tray_wnd.is_null() || cb_size as usize > size_of::<APPBARDATA>() {
        return FALSE as UINT_PTR;
    }

    let mut cmd: APPBAR_COMMAND = core::mem::zeroed();
    cmd.abd.cbSize = size_of::<APPBARDATAINTEROP>() as DWORD;
    cmd.abd.hWnd32 = handle_to_ulong(ptr::read((ptr::addr_of!((*pData).hWnd)).cast())); // Truncated on x64, as on Windows!
    cmd.abd.uCallbackMessage = ptr::read(ptr::addr_of!((*pData).uCallbackMessage));
    cmd.abd.uEdge = ptr::read(ptr::addr_of!((*pData).uEdge));
    cmd.abd.rc = ptr::read(ptr::addr_of!((*pData).rc));
    cmd.abd.lParam64 = ptr::read(ptr::addr_of!((*pData).lParam)) as LONGLONG;
    cmd.dwMessage = dwMessage;
    cmd.hOutput = handle_to_output(ptr::null_mut());
    cmd.dwProcessId = GetCurrentProcessId();

    // Make output data if necessary
    match dwMessage {
        ABM_QUERYPOS | ABM_SETPOS | ABM_GETTASKBARPOS => {
            let h_mem = appbar_copyin(
                ptr::addr_of!(cmd.abd) as LPCVOID,
                size_of::<APPBARDATAINTEROP>(),
                cmd.dwProcessId,
            );
            cmd.hOutput = handle_to_output(h_mem);
            if !output_is_set(cmd.hOutput) {
                return FALSE as UINT_PTR;
            }
        }
        _ => {}
    }

    // Send WM_COPYDATA message
    let copy_data = COPYDATASTRUCT {
        dwData: TABDMC_APPBAR,
        cbData: size_of::<APPBAR_COMMAND>() as DWORD,
        lpData: ptr::addr_of!(cmd) as PVOID,
    };
    let wparam = ptr::read(ptr::addr_of!((*pData).hWnd)) as WPARAM;
    let ret = SendMessageW(
        h_tray_wnd,
        WM_COPYDATA,
        wparam,
        ptr::addr_of!(copy_data) as LPARAM,
    ) as UINT_PTR;

    // Copy back output data
    if output_is_set(cmd.hOutput) {
        if appbar_copyout(
            output_to_handle(cmd.hOutput),
            ptr::addr_of_mut!(cmd.abd) as PVOID,
            size_of::<APPBARDATAINTEROP>(),
            cmd.dwProcessId,
        ) == FALSE
        {
            return FALSE as UINT_PTR;
        }
        ptr::write_volatile(ptr::addr_of_mut!((*pData).hWnd), (ulong_to_handle(cmd.abd.hWnd32)).cast());
        ptr::write_volatile(
            ptr::addr_of_mut!((*pData).uCallbackMessage),
            cmd.abd.uCallbackMessage,
        );
        ptr::write_volatile(ptr::addr_of_mut!((*pData).uEdge), cmd.abd.uEdge);
        ptr::write_volatile(ptr::addr_of_mut!((*pData).rc), cmd.abd.rc);
        ptr::write_volatile(ptr::addr_of_mut!((*pData).lParam), cmd.abd.lParam64 as LPARAM);
    }

    ret
}

// ADDED-TO-TYPES?: RECT, COPYDATASTRUCT, WPARAM/LPARAM/LRESULT (winuser.h common types);
// APPBARDATA/PAPPBARDATA (shellapi.h); APPBARDATAINTEROP/APPBAR_COMMAND/APPBAR_OUTPUT
// (undocshell.h, appbar-specific — probably keep file-local).
