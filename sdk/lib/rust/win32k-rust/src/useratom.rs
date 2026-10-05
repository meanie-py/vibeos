//! Rust translation of `win32ss/user/ntuser/useratom.c` — USER atom helpers.
//! [ntuser wave 1: opening the USER subsystem]

use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ddk::{RtlAddAtomToAtomTable, RtlQueryAtomInAtomTable, RtlPinAtomInAtomTable};
use crate::session::gAtomTable;

extern "system" {
    fn PsGetCurrentThreadWin32Thread() -> PTHREADINFO;
}
// SetLastNtError is FASTCALL on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

#[inline]
unsafe fn int_add_atom(AtomName: LPWSTR) -> RTL_ATOM {
    let mut Atom: RTL_ATOM = 0;

    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    if (*pti).rpdesk.is_null() {
        // C passes the (still STATUS_SUCCESS) Status here verbatim.
        SetLastNtError(0);
        return 0;
    }

    let Status = RtlAddAtomToAtomTable(gAtomTable, AtomName, &mut Atom);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    Atom
}

#[inline]
unsafe fn int_get_atom_name(nAtom: RTL_ATOM, lpBuffer: LPWSTR, cjBufSize: ULONG) -> ULONG {
    let mut Size: ULONG = cjBufSize;

    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    if (*pti).rpdesk.is_null() {
        SetLastNtError(0);
        return 0;
    }

    let Status = RtlQueryAtomInAtomTable(
        gAtomTable,
        nAtom,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpBuffer,
        &mut Size,
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    Size
}

#[inline]
unsafe fn int_add_global_atom(lpBuffer: LPWSTR, PinAtom: BOOL) -> RTL_ATOM {
    let mut Atom: RTL_ATOM = 0;

    let Status = RtlAddAtomToAtomTable(gAtomTable, lpBuffer, &mut Atom);
    if !NT_SUCCESS(Status) {
        // ERR("Error init Global Atom.\n");
        return 0;
    }

    if Atom != 0 && PinAtom != 0 {
        RtlPinAtomInAtomTable(gAtomTable, Atom);
    }
    Atom
}

// IntAddAtom / IntGetAtomName / IntAddGlobalAtom are FASTCALL (cross-file:
// class.c / cursoricon.c call them) — dual-cfg trampolines.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntAddAtom(AtomName: LPWSTR) -> RTL_ATOM {
    int_add_atom(AtomName)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntAddAtom(AtomName: LPWSTR) -> RTL_ATOM {
    int_add_atom(AtomName)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetAtomName(nAtom: RTL_ATOM, lpBuffer: LPWSTR, cjBufSize: ULONG) -> ULONG {
    int_get_atom_name(nAtom, lpBuffer, cjBufSize)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetAtomName(nAtom: RTL_ATOM, lpBuffer: LPWSTR, cjBufSize: ULONG) -> ULONG {
    int_get_atom_name(nAtom, lpBuffer, cjBufSize)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntAddGlobalAtom(lpBuffer: LPWSTR, PinAtom: BOOL) -> RTL_ATOM {
    int_add_global_atom(lpBuffer, PinAtom)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntAddGlobalAtom(lpBuffer: LPWSTR, PinAtom: BOOL) -> RTL_ATOM {
    int_add_global_atom(lpBuffer, PinAtom)
}

/// Returns the name of an atom. APIENTRY (`_NtUserGetAtomName@8`, stdcall i386).
#[no_mangle]
pub unsafe extern "system" fn NtUserGetAtomName(atom: ATOM, pustrName: PUNICODE_STRING) -> ULONG {
    let mut awcBuffer: [WCHAR; 256] = core::mem::zeroed();

    /* Retrieve the atom name into a local buffer (max length is 255 chars) */
    let mut cjLength: ULONG = int_get_atom_name(
        atom as RTL_ATOM,
        awcBuffer.as_mut_ptr(),
        core::mem::size_of_val(&awcBuffer) as ULONG,
    );
    if cjLength != 0 {
        // C: _SEH2_TRY { ProbeForRead(pustrName, ..); ProbeForWrite(Buffer, MaximumLength, 1);
        //                copy + NUL-terminate }
        //    _SEH2_EXCEPT { SetLastNtError(code); cjLength = 0; }
        // The header is captured once (the C re-reads it from user memory).
        let r: Result<ULONG, NTSTATUS> = (|| {
            let ustr: UNICODE_STRING = crate::safe_user_io::capture_user(pustrName as *const UNICODE_STRING)?;
            crate::safe_user_io::probe_for_write(
                ustr.Buffer as *mut core::ffi::c_void,
                ustr.MaximumLength as usize,
                1,
            )?;

            // sizeof(UNICODE_NULL) == 2 (a WCHAR).
            if ustr.MaximumLength as ULONG >= 2 {
                /* Limit the length to the buffer size */
                let len = core::cmp::min(ustr.MaximumLength as ULONG - 2, cjLength);

                /* Copy the string (len is a BYTE count) and NUL-terminate */
                ntkrnl_sys::probe::safe_copy_raw(
                    ustr.Buffer as *mut core::ffi::c_void,
                    awcBuffer.as_ptr() as *const core::ffi::c_void,
                    len as usize,
                )?;
                let nul: WCHAR = 0;
                ntkrnl_sys::probe::safe_copy_raw(
                    ustr.Buffer.add(len as usize / core::mem::size_of::<WCHAR>()) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(nul) as *const core::ffi::c_void,
                    core::mem::size_of::<WCHAR>(),
                )?;
                Ok(len)
            } else {
                Ok(0)
            }
        })();
        cjLength = match r {
            Ok(len) => len,
            Err(st) => {
                /* On exception, set last error and fail */
                SetLastNtError(st);
                0
            }
        };
    }

    /* Return the length in characters */
    cjLength / core::mem::size_of::<WCHAR>() as ULONG
}
