use core::ptr::addr_of;
// ke globals across the crate boundary as link symbols (#[no_mangle] in
// ntoskrnl-rust ke/*). crate-split Phase 1b.
extern "C" {
    static mut KeServiceDescriptorTable: ntkrnl_sys::ddk::_KSERVICE_TABLE_DESCRIPTOR;
    static mut KeServiceDescriptorTableShadow: ntkrnl_sys::ddk::_KSERVICE_TABLE_DESCRIPTOR;
}
use core::ptr::addr_of_mut;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NO_MEMORY, STATUS_ALREADY_COMMITTED};
use ntkrnl_sys::constants::*;

extern "system" {
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}
extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}


const ERROR_NOT_ENOUGH_MEMORY: ULONG = 8;
// Matches the xdk KERNEL_STACK_SIZE macro per arch: amd64/arm64 0xC000 (48KB),
// i386 0x3000 (12KB). The old not(x86_64)=0x6000 was wrong for BOTH i386 (double
// the 0x3000 header value → GUI-thread KeSwitchKernelStack computed StackLimit
// 0x3000 too low) and arm64 (should be 0xC000; dead there — KeSwitchKernelStack
// unimplemented on arm64 — but still wrong). See per-arch-cfg-divergence-audit.md.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
const KERNEL_STACK_SIZE: ULONG_PTR = 0xC000;
#[cfg(target_arch = "x86")]
const KERNEL_STACK_SIZE: ULONG_PTR = 0x3000;
const STATUS_ALREADY_WIN32: NTSTATUS = 0x4000001B;

const PsW32ThreadCalloutInitialize: ULONG = 0;

#[no_mangle]
static mut PspW32ProcessCallout: PKWIN32_PROCESS_CALLOUT = None;
#[no_mangle]
static mut PspW32ThreadCallout: PKWIN32_THREAD_CALLOUT = None;
/// win32k's job callout (UI restrictions), set by PsEstablishWin32Callouts (upstream b272427baa3).
#[no_mangle]
pub static mut PspW32JobCallout: PKWIN32_JOB_CALLOUT = None;
#[no_mangle]
pub static mut KeGdiFlushUserBatch: PGDI_BATCHFLUSH_ROUTINE = None;

extern "C" {
    static mut ExpWindowStationObjectParse: PKWIN32_SESSION_CALLOUT;
    static mut ExpWindowStationObjectDelete: PKWIN32_SESSION_CALLOUT;
    static mut ExpWindowStationObjectOkToClose: PKWIN32_SESSION_CALLOUT;
    static mut ExpDesktopObjectOkToClose: PKWIN32_SESSION_CALLOUT;
    static mut ExpDesktopObjectDelete: PKWIN32_SESSION_CALLOUT;
    static mut ExpDesktopObjectOpen: PKWIN32_SESSION_CALLOUT;
    static mut ExpDesktopObjectClose: PKWIN32_SESSION_CALLOUT;
    static mut PopEventCallout: PKWIN32_POWEREVENT_CALLOUT;
}

#[no_mangle]
pub unsafe extern "system" fn PsConvertToGuiThread() -> NTSTATUS {
    #[cfg(not(target_arch = "aarch64"))]
    let mut new_stack: ULONG_PTR;
    #[cfg(not(target_arch = "aarch64"))]
    let old_stack: PVOID;

    let thread = PsGetCurrentThread();
    let process = ntkrnl_sys::ps::PsGetCurrentProcess();
    let mut status: NTSTATUS;

    if KeGetPreviousMode() == 0i8 {
        return STATUS_INVALID_PARAMETER;
    }

    if (*(&raw const PspW32ProcessCallout)).is_none() {
        return STATUS_ACCESS_DENIED;
    }

    if (*(thread as *mut _ETHREAD)).Tcb.ServiceTable != addr_of!(KeServiceDescriptorTable) as *mut _ {
        return STATUS_ALREADY_WIN32;
    }

    #[cfg(target_arch = "x86")]
    let large_stack = (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_7.__bindgen_anon_4.LargeStack;
    #[cfg(not(target_arch = "x86"))]
    let large_stack = (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_5.__bindgen_anon_4.LargeStack;

    if large_stack == 0 {
        #[cfg(target_arch = "aarch64")]
        {
            (*(thread as *mut _ETHREAD)).Tcb.__bindgen_anon_5.__bindgen_anon_4.LargeStack = 1;
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            new_stack = MmCreateKernelStack(1, 0) as ULONG_PTR;
            if new_stack == 0 {
                (*NtCurrentTeb()).LastErrorValue = ERROR_NOT_ENOUGH_MEMORY;
                return STATUS_NO_MEMORY;
            }

            KeEnterGuardedRegion();

            old_stack = KeSwitchKernelStack(
                new_stack as PVOID,
                (new_stack - KERNEL_STACK_SIZE) as PVOID,
            );

            KeLeaveGuardedRegion();

            MmDeleteKernelStack(old_stack, 0);
        }
    }

    status = (PspW32ProcessCallout.unwrap())(process as *mut _EPROCESS, 1);
    if status < 0 {
        return status;
    }

    (*(thread as *mut _ETHREAD)).Tcb.ServiceTable = addr_of_mut!(KeServiceDescriptorTableShadow) as *mut _;

    status = (PspW32ThreadCallout.unwrap())(thread, PsW32ThreadCalloutInitialize);
    if status < 0 {
        (*(thread as *mut _ETHREAD)).Tcb.ServiceTable = addr_of_mut!(KeServiceDescriptorTable) as *mut _;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsEstablishWin32Callouts(callout_data: PWIN32_CALLOUTS_FPNS) {
    PspW32ProcessCallout = (*callout_data).ProcessCallout;
    PspW32ThreadCallout = (*callout_data).ThreadCallout;
    PspW32JobCallout = (*callout_data).JobCallout;
    ExpWindowStationObjectParse = (*callout_data).WindowStationParseProcedure;
    ExpWindowStationObjectDelete = (*callout_data).WindowStationDeleteProcedure;
    ExpWindowStationObjectOkToClose = (*callout_data).WindowStationOkToCloseProcedure;
    ExpDesktopObjectOkToClose = (*callout_data).DesktopOkToCloseProcedure;
    ExpDesktopObjectDelete = (*callout_data).DesktopDeleteProcedure;
    ExpDesktopObjectOpen = (*callout_data).DesktopOpenProcedure;
    ExpDesktopObjectClose = (*callout_data).DesktopCloseProcedure;
    PopEventCallout = (*callout_data).PowerEventCallout;
    KeGdiFlushUserBatch = (*callout_data).BatchFlushRoutine;
}
