use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::ObReferenceObjectByHandle;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_INFO_LENGTH_MISMATCH, STATUS_INVALID_INFO_CLASS};
use core::ffi::c_void;

// SafeProbeForRead/Write: NTAPI (extern "system" = stdcall i386 / cdecl else),
// Length = SIZE_T (canonical signature — see icm.rs).
extern "system" {
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}

const INVALID_HANDLE_VALUE: isize = -1;     // (HANDLE)-1
const UserMode: i8 = 1;                       // KPROCESSOR_MODE

// ExDesktopObjectType is a DATA export from ntoskrnl (ntoskrnl.spec `@ extern`).
// Per-arch cross-DLL import (same shape as ExEventObjectType / csr.rs's
// LpcPortObjectType — [[feedback_exeventobjecttype_amd64_ptr_to_ptr]]):
//  - i386/arm64: raw-dylib +verbatim makes the __imp_ thunk; symbol IS POBJECT_TYPE.
//  - amd64: bind to __imp_ explicitly (ptr-to-ptr) + deref. A plain extern here
//    resolves to garbage → STATUS_OBJECT_TYPE_MISMATCH at runtime (latent: only
//    NtUserConsoleControl hits it, off the critical boot path).
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut ExDesktopObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_ExDesktopObjectType"]
    static mut ExDesktopObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> PVOID {
    ExDesktopObjectType as PVOID
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ex_desktop_object_type() -> PVOID {
    *ExDesktopObjectType_imp as PVOID
}

// ---- file-scope preamble (translated) ----
// DBG_DEFAULT_CHANNEL(UserMisc) - debug channel macro, no declarations emitted

// NtUserConsoleControl is APIENTRY (stdcall on i386, cdecl elsewhere). The body
// is ABI-independent, so it lives in one _impl and the two #[no_mangle] wrappers
// just forward. NOTE: the C used a `switch` on the CONSOLECONTROL enum; the bindgen
// values are plain consts, so a Rust `match` arm `ConsoleCtrlX => ...` would be a
// catch-all BINDING (always taking the first arm) — we use an `==` if/else chain.
unsafe fn NtUserConsoleControl_impl(
    ConsoleCtrl: CONSOLECONTROL,
    ConsoleCtrlInfo: PVOID,
    ConsoleCtrlInfoLength: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    /* Allow only the Console Server to perform this operation (via CSRSS) */
    if PsGetCurrentProcess() as *mut c_void != gpepCSRSS as *mut c_void {
        return STATUS_ACCESS_DENIED;
    }

    UserEnterExclusive();

    if ConsoleCtrl == ConsoleCtrlDesktopConsoleThread {
        'block: {
            let mut DesktopConsoleThreadInfo: DESKTOP_CONSOLE_THREAD = core::mem::zeroed();
            let mut Desktop: PDESKTOP = core::ptr::null_mut();
            let OldThreadId: ULONG_PTR;

            if ConsoleCtrlInfoLength as usize != core::mem::size_of::<DESKTOP_CONSOLE_THREAD>() {
                Status = STATUS_INFO_LENGTH_MISMATCH;
                break 'block;
            }

            Status = SafeProbeForWrite(
                ConsoleCtrlInfo as LPVOID,
                ConsoleCtrlInfoLength as SIZE_T,
                core::mem::size_of::<USHORT>() as u32,
            );
            if !NT_SUCCESS(Status) {
                break 'block;
            } else {
                DesktopConsoleThreadInfo = *(ConsoleCtrlInfo as PDESKTOP_CONSOLE_THREAD);
            }

            /* Reference the desktop */
            // Cross-crate bridges (ntkrnl_sys ObReferenceObjectByHandle vs win32k_sys
            // HDESK/_OBJECT_TYPE). ExDesktopObjectType is a plain POBJECT_TYPE on all
            // 3 arches (no amd64 ptr-to-ptr, unlike ExEventObjectType).
            Status = ObReferenceObjectByHandle(
                DesktopConsoleThreadInfo.DesktopHandle as _,
                0,
                ex_desktop_object_type() as _,
                UserMode as i8,
                core::ptr::addr_of_mut!(Desktop) as *mut _,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                break 'block;
            }

            /* Save the old thread ID, it is always returned to the caller */
            OldThreadId = (*Desktop).dwConsoleThreadId as ULONG_PTR;  // DWORD→ULONG_PTR (widen on 64-bit)

            /* Set the new console input thread ID for this desktop if required */
            if DesktopConsoleThreadInfo.ThreadId != INVALID_HANDLE_VALUE as ULONG_PTR {
                (*Desktop).dwConsoleThreadId = DesktopConsoleThreadInfo.ThreadId as DWORD;  // ULONG_PTR→DWORD (narrow, as in C)
            }

            /* Always return the old thread ID */
            DesktopConsoleThreadInfo.ThreadId = OldThreadId;

            /* Dereference the desktop */
            ObDereferenceObject(Desktop as PVOID);

            /* Return the information back to the caller (user-mode output pointer —
             * write_volatile so LLVM can't treat the buffer as captures(none)). */
            core::ptr::write_volatile(
                ConsoleCtrlInfo as PDESKTOP_CONSOLE_THREAD,
                DesktopConsoleThreadInfo,
            );
        }
    } else if ConsoleCtrl == GuiConsoleWndClassAtom {
        'block: {
            if ConsoleCtrlInfoLength as usize != core::mem::size_of::<ATOM>() {
                Status = STATUS_INFO_LENGTH_MISMATCH;
                break 'block;
            }

            Status = SafeProbeForRead(
                ConsoleCtrlInfo as LPVOID,
                ConsoleCtrlInfoLength as SIZE_T,
                core::mem::size_of::<USHORT>() as u32,
            );
            if NT_SUCCESS(Status) {
                gaGuiConsoleWndClass = *(ConsoleCtrlInfo as *mut ATOM);
            }
        }
    } else if ConsoleCtrl == ConsoleMakePalettePublic {
        'block: {
            let hPalette: HPALETTE;

            if ConsoleCtrlInfoLength as usize != core::mem::size_of::<HPALETTE>() {
                Status = STATUS_INFO_LENGTH_MISMATCH;
                break 'block;
            }

            Status = SafeProbeForRead(
                ConsoleCtrlInfo as LPVOID,
                ConsoleCtrlInfoLength as SIZE_T,
                core::mem::size_of::<USHORT>() as u32,
            );
            if !NT_SUCCESS(Status) {
                break 'block;
            } else {
                hPalette = *(ConsoleCtrlInfo as *mut HPALETTE);
            }

            /* Make the palette handle public */
            GreSetObjectOwnerEx(hPalette as _, GDI_OBJ_HMGR_PUBLIC, GDIOBJFLAG_IGNOREPID);
        }
    } else if ConsoleCtrl == ConsoleAcquireDisplayOwnership {
        Status = STATUS_NOT_IMPLEMENTED;
    } else {
        Status = STATUS_INVALID_INFO_CLASS;
    }

    UserLeave();

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtUserConsoleControl(
    ConsoleCtrl: CONSOLECONTROL,
    ConsoleCtrlInfo: PVOID,
    ConsoleCtrlInfoLength: ULONG,
) -> NTSTATUS {
    NtUserConsoleControl_impl(ConsoleCtrl, ConsoleCtrlInfo, ConsoleCtrlInfoLength)
}
