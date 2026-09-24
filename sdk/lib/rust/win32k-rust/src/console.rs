use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::ObReferenceObjectByHandle;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NOT_IMPLEMENTED};
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_INFO_LENGTH_MISMATCH, STATUS_INVALID_INFO_CLASS};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

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
            } else if !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(DesktopConsoleThreadInfo) as *mut core::ffi::c_void,
                ConsoleCtrlInfo as *const core::ffi::c_void,
                core::mem::size_of::<DESKTOP_CONSOLE_THREAD>(),
            ) {
                // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
                Status = STATUS_ACCESS_VIOLATION;
                break 'block;
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

            // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
            // C (win32ss/user/ntuser/console.c, NtUserConsoleControl,
            // ConsoleCtrlDesktopConsoleThread case):
            //   _SEH2_TRY {
            //       *(PDESKTOP_CONSOLE_THREAD)ConsoleCtrlInfo = DesktopConsoleThreadInfo;
            //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
            //       Status = _SEH2_GetExceptionCode();
            //   } _SEH2_END;
            //   break;
            //
            // Shape: the handler only ASSIGNS Status and falls through to the
            // case's `break` — there is no separate cleanup, so setting `Status`
            // here and letting this 'block end naturally (falling through to
            // `UserLeave(); return Status;` below) reproduces it exactly.
            //
            // Note this C _SEH2_TRY wraps a BARE store with no ProbeFor* call at
            // all (unlike the SafeProbeForWrite used earlier in this same case,
            // for the initial capture) — so per rule 1 there is no literal
            // alignment to carry forward, and per safe_copy_raw's own doc this is
            // exactly the "bare _SEH2_TRY { store } _SEH2_EXCEPT" shape it exists
            // for: a fault-tolerant copy with NO probing, catching only the
            // hardware fault the raw store could take.
            if let Err(st) = ntkrnl_sys::probe::safe_copy_raw(
                ConsoleCtrlInfo as *mut c_void,
                core::ptr::addr_of!(DesktopConsoleThreadInfo) as *const c_void,
                core::mem::size_of::<DESKTOP_CONSOLE_THREAD>(),
            ) {
                Status = st;
            }
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
            if NT_SUCCESS(Status)
                // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
                && !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(gaGuiConsoleWndClass) as *mut core::ffi::c_void,
                    ConsoleCtrlInfo as *const core::ffi::c_void,
                    core::mem::size_of::<ATOM>(),
                )
            {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }
    } else if ConsoleCtrl == ConsoleMakePalettePublic {
        'block: {
            let mut hPalette: HPALETTE = core::ptr::null_mut();

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
            } else if !crate::copy::capture_from_user(
                core::ptr::addr_of_mut!(hPalette) as *mut core::ffi::c_void,
                ConsoleCtrlInfo as *const core::ffi::c_void,
                core::mem::size_of::<HPALETTE>(),
            ) {
                // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
                Status = STATUS_ACCESS_VIOLATION;
                break 'block;
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
