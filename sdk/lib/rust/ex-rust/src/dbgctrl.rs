use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::probe::*;

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _KAPC_STATE,
    PEPROCESS,
    KPROCESSOR_MODE,
    LUID,
    WORK_QUEUE_ITEM,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    PsGetNextProcess,
    PsTerminateProcess,
    SeSinglePrivilegeCheck,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    static SeDebugPrivilege: LUID;
    static mut KdDebuggerEnabled: BOOLEAN;
    static mut KdAutoEnableOnEvent: BOOLEAN;
    static mut KdPitchDebugger: BOOLEAN;
    static mut KdPrintBufferSize: ULONG;
    static mut KdIgnoreUmExceptions: BOOLEAN;
    static mut KdBlockEnable: BOOLEAN;
    static mut InitIsWinPEMode: BOOLEAN;
}

// These are NTAPI (stdcall on i386) — must use extern "system"
extern "system" {
    fn DbgBreakPointWithStatus(Status: ULONG);
    fn KdSystemDebugControl( Command: SYSDBG_COMMAND, InputBuffer: PVOID, InputBufferLength: ULONG, OutputBuffer: PVOID, OutputBufferLength: ULONG, ReturnLength: *mut ULONG, PreviousMode: KPROCESSOR_MODE, ) -> NTSTATUS;
    fn KdEnableDebugger() -> NTSTATUS;
    fn KdDisableDebugger() -> NTSTATUS;
    fn KdChangeOption( Option: ULONG, InBufferBytes: ULONG, InBuffer: PVOID, OutBufferBytes: ULONG, OutBuffer: PVOID, OutBufferNeeded: *mut ULONG, ) -> NTSTATUS;
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}

const DPFLTR_SYSTEM_ID: ULONG = 0;
const DPFLTR_ERROR_LEVEL: ULONG = 0x0;
const DBG_TERMINATE_PROCESS: NTSTATUS = 0x40010004u32 as NTSTATUS;
const DBG_STATUS_WORKER: ULONG = 0x07;
const DBG_STATUS_DEBUG_CONTROL: ULONG = 0x06;
const KD_OPTION_SET_BLOCK_ENABLE: ULONG = 0x00000000;

pub type SYSDBG_COMMAND = i32;
const SysDbgQueryModuleInformation: SYSDBG_COMMAND = 0;
const SysDbgQueryTraceInformation: SYSDBG_COMMAND = 1;
const SysDbgSetTracepoint: SYSDBG_COMMAND = 2;
const SysDbgSetSpecialCall: SYSDBG_COMMAND = 3;
const SysDbgClearSpecialCalls: SYSDBG_COMMAND = 4;
const SysDbgQuerySpecialCalls: SYSDBG_COMMAND = 5;
const SysDbgBreakPoint: SYSDBG_COMMAND = 6;
const SysDbgQueryVersion: SYSDBG_COMMAND = 7;
const SysDbgReadVirtual: SYSDBG_COMMAND = 8;
const SysDbgWriteVirtual: SYSDBG_COMMAND = 9;
const SysDbgReadPhysical: SYSDBG_COMMAND = 10;
const SysDbgWritePhysical: SYSDBG_COMMAND = 11;
const SysDbgReadControlSpace: SYSDBG_COMMAND = 12;
const SysDbgWriteControlSpace: SYSDBG_COMMAND = 13;
const SysDbgReadIoSpace: SYSDBG_COMMAND = 14;
const SysDbgWriteIoSpace: SYSDBG_COMMAND = 15;
const SysDbgReadMsr: SYSDBG_COMMAND = 16;
const SysDbgWriteMsr: SYSDBG_COMMAND = 17;
const SysDbgReadBusData: SYSDBG_COMMAND = 18;
const SysDbgWriteBusData: SYSDBG_COMMAND = 19;
const SysDbgCheckLowMemory: SYSDBG_COMMAND = 20;
const SysDbgEnableKernelDebugger: SYSDBG_COMMAND = 21;
const SysDbgDisableKernelDebugger: SYSDBG_COMMAND = 22;
const SysDbgGetAutoKdEnable: SYSDBG_COMMAND = 23;
const SysDbgSetAutoKdEnable: SYSDBG_COMMAND = 24;
const SysDbgGetPrintBufferSize: SYSDBG_COMMAND = 25;
const SysDbgSetPrintBufferSize: SYSDBG_COMMAND = 26;
const SysDbgGetKdUmExceptionEnable: SYSDBG_COMMAND = 27;
const SysDbgSetKdUmExceptionEnable: SYSDBG_COMMAND = 28;
const SysDbgGetTriageDump: SYSDBG_COMMAND = 29;
const SysDbgGetKdBlockEnable: SYSDBG_COMMAND = 30;
const SysDbgSetKdBlockEnable: SYSDBG_COMMAND = 31;

pub type WINKD_WORKER_STATE = i32;
const WinKdWorkerReady: WINKD_WORKER_STATE = 0;
const WinKdWorkerStart: WINKD_WORKER_STATE = 1;
const WinKdWorkerInitialized: WINKD_WORKER_STATE = 2;

#[no_mangle]
pub static mut ExpDebuggerWorkItem: WORK_QUEUE_ITEM = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut ExpDebuggerWork: WINKD_WORKER_STATE = WinKdWorkerReady;

#[no_mangle]
pub static mut ExpDebuggerProcessAttach: PEPROCESS = core::ptr::null_mut();

#[no_mangle]
pub static mut ExpDebuggerProcessKill: PEPROCESS = core::ptr::null_mut();

#[no_mangle]
pub static mut ExpDebuggerPageIn: ULONG_PTR = 0;

#[no_mangle]
pub unsafe extern "system" fn ExpDebuggerWorker(Context: PVOID) {
    let Context = opaque_ptr(Context);
    let ProcessToAttach: PEPROCESS;
    let ProcessToKill: PEPROCESS;
    let PageInAddress: ULONG_PTR;
    let mut Process: PEPROCESS;
    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

    if ExpDebuggerWork != WinKdWorkerInitialized {
        ExpDebuggerWork = WinKdWorkerReady;
        return;
    }

    ProcessToAttach = ExpDebuggerProcessAttach;
    ProcessToKill = ExpDebuggerProcessKill;
    PageInAddress = ExpDebuggerPageIn;

    ExpDebuggerProcessAttach = core::ptr::null_mut();
    ExpDebuggerProcessKill = core::ptr::null_mut();
    ExpDebuggerPageIn = 0;
    ExpDebuggerWork = WinKdWorkerReady;

    Process = core::ptr::null_mut();

    if !ProcessToAttach.is_null() || !ProcessToKill.is_null() {
        loop {
            Process = PsGetNextProcess(Process);
            if Process.is_null() {
                break;
            }

            if Process == ProcessToAttach {
                KeStackAttachProcess(
                    core::ptr::addr_of_mut!((*(Process as *mut _EPROCESS)).Pcb),
                    &mut ApcState as *mut _KAPC_STATE,
                );
                break;
            } else if Process == ProcessToKill {
                PsTerminateProcess(Process, DBG_TERMINATE_PROCESS);
                ObDereferenceObject(Process as PVOID);
                return;
            }
        }

        if Process.is_null() {
            crate::dbg_print_ex!(
                DPFLTR_SYSTEM_ID,
                DPFLTR_ERROR_LEVEL,
                b"EX debug work: Unable to find process %p\n",
                if !ProcessToAttach.is_null() { ProcessToAttach } else { ProcessToKill },
            );
        }
    }

    if PageInAddress != 0 {
        let result = probe_user_buffer(
            PageInAddress as *const core::ffi::c_void,
            1,
            1,
            0i8,
        );
        
        if let Err(status) = result {
            crate::dbg_print_ex!(
                DPFLTR_SYSTEM_ID,
                DPFLTR_ERROR_LEVEL,
                b"EX page in: Failed to page-in address 0x%p, Status 0x%08lx\n",
                PageInAddress,
                status,
            );
        }
    }

    DbgBreakPointWithStatus(DBG_STATUS_WORKER);

    if !Process.is_null() {
        KeUnstackDetachProcess(&mut ApcState as *mut _KAPC_STATE);
        ObDereferenceObject(Process as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtSystemDebugControl(
    Command: SYSDBG_COMMAND,
    InputBuffer: PVOID,
    InputBufferLength: ULONG,
    OutputBuffer: PVOID,
    OutputBufferLength: ULONG,
    ReturnLength: *mut ULONG,
) -> NTSTATUS {
    let InputBuffer = opaque_ptr(InputBuffer);
    let OutputBuffer = opaque_ptr(OutputBuffer);
    let ReturnLength = opaque_ptr(ReturnLength);
    let PreviousMode = KeGetPreviousMode();
    let mut Length: ULONG = 0;

    if SeSinglePrivilegeCheck(
        SeDebugPrivilege,
        PreviousMode,
    ) == 0u8 {
        return STATUS_ACCESS_DENIED;
    }

    let Status = 'seh: {
        if PreviousMode != 0i8 {
            if InputBufferLength != 0 {
                if let Err(e) = probe_user_buffer(
                    InputBuffer,
                    InputBufferLength as usize,
                    core::mem::align_of::<ULONG>(),
                    PreviousMode,
                ) {
                    break 'seh e;
                }
            }
            if OutputBufferLength != 0 {
                if let Err(e) = probe_user_buffer_write(
                    OutputBuffer,
                    OutputBufferLength as usize,
                    core::mem::align_of::<ULONG>(),
                    PreviousMode,
                ) {
                    break 'seh e;
                }
            }
            if !ReturnLength.is_null() {
                if let Err(e) = probe_user_buffer_write(
                    ReturnLength as *const core::ffi::c_void,
                    core::mem::size_of::<ULONG>(),
                    core::mem::align_of::<ULONG>(),
                    PreviousMode,
                ) {
                    break 'seh e;
                }
            }
        }

        let status = match Command {
            SysDbgQueryModuleInformation => {
                STATUS_INVALID_INFO_CLASS
            }

            #[cfg(target_arch = "x86")]
            SysDbgQueryTraceInformation
            | SysDbgSetTracepoint
            | SysDbgSetSpecialCall
            | SysDbgClearSpecialCalls
            | SysDbgQuerySpecialCalls => {
                STATUS_NOT_IMPLEMENTED
            }

            SysDbgQueryVersion
            | SysDbgReadVirtual
            | SysDbgWriteVirtual
            | SysDbgReadPhysical
            | SysDbgWritePhysical
            | SysDbgReadControlSpace
            | SysDbgWriteControlSpace
            | SysDbgReadIoSpace
            | SysDbgWriteIoSpace
            | SysDbgReadMsr
            | SysDbgWriteMsr
            | SysDbgReadBusData
            | SysDbgWriteBusData
            | SysDbgCheckLowMemory => {
                if InitIsWinPEMode != 0u8 {
                    KdSystemDebugControl(
                        Command,
                        InputBuffer,
                        InputBufferLength,
                        OutputBuffer,
                        OutputBufferLength,
                        &mut Length as *mut ULONG,
                        PreviousMode,
                    )
                } else {
                    STATUS_NOT_IMPLEMENTED
                }
            }

            SysDbgBreakPoint => {
                if KdDebuggerEnabled != 0u8 {
                    DbgBreakPointWithStatus(DBG_STATUS_DEBUG_CONTROL);
                    STATUS_SUCCESS
                } else {
                    STATUS_UNSUCCESSFUL
                }
            }

            SysDbgEnableKernelDebugger => {
                KdEnableDebugger()
            }

            SysDbgDisableKernelDebugger => {
                KdDisableDebugger()
            }

            SysDbgGetAutoKdEnable => {
                if OutputBufferLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else {
                    if let Err(e) = write_user(
                        OutputBuffer as *mut BOOLEAN,
                        KdAutoEnableOnEvent,
                        PreviousMode,
                    ) {
                        break 'seh e;
                    }
                    STATUS_SUCCESS
                }
            }

            SysDbgSetAutoKdEnable => {
                if InputBufferLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else if KdPitchDebugger != 0u8 {
                    STATUS_ACCESS_DENIED
                } else {
                    match ntkrnl_sys::probe::read_back(InputBuffer as *const BOOLEAN) {
                        Ok(v) => {
                            KdAutoEnableOnEvent = v;
                            STATUS_SUCCESS
                        }
                        Err(s) => s,
                    }
                }
            }

            SysDbgGetPrintBufferSize => {
                if OutputBufferLength != core::mem::size_of::<ULONG>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else {
                    let value = if KdPitchDebugger != 0u8 { 0 } else { KdPrintBufferSize };
                    if let Err(e) = write_user(
                        OutputBuffer as *mut ULONG,
                        value,
                        PreviousMode,
                    ) {
                        break 'seh e;
                    }
                    STATUS_SUCCESS
                }
            }

            SysDbgSetPrintBufferSize => {
                STATUS_NOT_IMPLEMENTED
            }

            SysDbgGetKdUmExceptionEnable => {
                if OutputBufferLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else {
                    let value = if KdIgnoreUmExceptions != 0u8 { 0u8 } else { 1u8 };
                    if let Err(e) = write_user(
                        OutputBuffer as *mut BOOLEAN,
                        value,
                        PreviousMode,
                    ) {
                        break 'seh e;
                    }
                    STATUS_SUCCESS
                }
            }

            SysDbgSetKdUmExceptionEnable => {
                if InputBufferLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else if KdPitchDebugger != 0u8 {
                    STATUS_ACCESS_DENIED
                } else {
                    match ntkrnl_sys::probe::read_back(InputBuffer as *const BOOLEAN) {
                        Ok(input_val) => {
                            KdIgnoreUmExceptions = if input_val != 0u8 { 0u8 } else { 1u8 };
                            STATUS_SUCCESS
                        }
                        Err(s) => s,
                    }
                }
            }

            SysDbgGetTriageDump => {
                STATUS_NOT_IMPLEMENTED
            }

            SysDbgGetKdBlockEnable => {
                if OutputBufferLength != core::mem::size_of::<BOOLEAN>() as ULONG {
                    STATUS_INFO_LENGTH_MISMATCH
                } else {
                    if let Err(e) = write_user(
                        OutputBuffer as *mut BOOLEAN,
                        KdBlockEnable,
                        PreviousMode,
                    ) {
                        break 'seh e;
                    }
                    STATUS_SUCCESS
                }
            }

            SysDbgSetKdBlockEnable => {
                KdChangeOption(
                    KD_OPTION_SET_BLOCK_ENABLE,
                    InputBufferLength,
                    InputBuffer,
                    OutputBufferLength,
                    OutputBuffer,
                    &mut Length as *mut ULONG,
                )
            }

            _ => {
                STATUS_INVALID_INFO_CLASS
            }
        };

        if !ReturnLength.is_null() {
            if let Err(e) = write_user(ReturnLength, Length, PreviousMode) {
                break 'seh e;
            }
        }

        status
    };

    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/dbgctrl.c
//  * PURPOSE:         System debug control
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* DATA **********************************************************************/
//
// /*
//  * WinDBG Debugger Worker State Machine data
//  */
// WORK_QUEUE_ITEM ExpDebuggerWorkItem;
// /*
//  * The following global variables must be visible through all the kernel
//  * because WinDBG explicitely search for them inside our symbols.
//  */
// WINKD_WORKER_STATE ExpDebuggerWork;
// PEPROCESS ExpDebuggerProcessAttach;
// PEPROCESS ExpDebuggerProcessKill;
// ULONG_PTR ExpDebuggerPageIn;
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * WinDBG Debugger Worker State Machine
//  *
//  * This functionality is used whenever WinDBG wants to attach or kill a user-mode
//  * process from within live kernel-mode session, and/or page-in an address region.
//  * It is implemented as a state machine: when it is in "Ready" state, WinDBG can
//  * initialize the data for the state machine, then switch its state to "Start".
//  * The worker thread balance manager detects this, switches the state to "Initialized"
//  * and queues a worker thread. As long as the state is not "Ready" again, WinDBG
//  * prevents from requeuing a new thread. When the thread is started, it captures
//  * all the data, then resets the machine state to "Ready", thus allowing WinDBG
//  * to requeue another worker thread.
//  *
//  * WinDBG commands:
//  *     .process /i <addr> (where <addr> is the address of the _EPROCESS block for this process)
//  *     .kill <addr>       (       "                "                "                "       )
//  *     .pagein <addr>     (where <addr> is the address to page in)
//  */
// VOID
// NTAPI
// ExpDebuggerWorker(
//     _In_ PVOID Context)
// {
//     PEPROCESS ProcessToAttach, ProcessToKill;
//     ULONG_PTR PageInAddress;
//     PEPROCESS Process;
//     _KAPC_STATE ApcState;
//
//     UNREFERENCED_PARAMETER(Context);
//
//     /* Be sure we were started in an initialized state */
//     ASSERTMSG("ExpDebuggerWorker being entered in non-initialized state!\n",
//               ExpDebuggerWork == WinKdWorkerInitialized);
//     if (ExpDebuggerWork != WinKdWorkerInitialized)
//     {
//         /* An error happened, so get a chance to restart proper */
//         ExpDebuggerWork = WinKdWorkerReady;
//         return;
//     }
//
//     /* Get the processes to be attached or killed, and the address to page in */
//     ProcessToAttach = ExpDebuggerProcessAttach;
//     ProcessToKill   = ExpDebuggerProcessKill;
//     PageInAddress   = ExpDebuggerPageIn;
//
//     /* Reset the state machine to its ready state */
//     ExpDebuggerProcessAttach = NULL;
//     ExpDebuggerProcessKill   = NULL;
//     ExpDebuggerPageIn = (ULONG_PTR)NULL;
//     ExpDebuggerWork = WinKdWorkerReady;
//
//     /* Default to the current process if we don't find the process to be attached or killed */
//     Process = NULL;
//
//     /* Check if we need to attach or kill some process */
//     if (ProcessToAttach || ProcessToKill)
//     {
//         /* Find the process in the list */
//         while ((Process = PsGetNextProcess(Process)))
//         {
//             /* Is this the process we want to attach to? */
//             if (Process == ProcessToAttach)
//             {
//                 /* Yes, attach ourselves to it */
//                 KeStackAttachProcess(&Process->Pcb, &ApcState);
//                 break;
//             }
//             /* Or is this the process we want to kill? */
//             else if (Process == ProcessToKill)
//             {
//                 /* Yes, kill and dereference it, then return */
//                 PsTerminateProcess(Process, DBG_TERMINATE_PROCESS);
//                 ObDereferenceObject(Process);
//                 return;
//             }
//         }
//
//         if (!Process)
//         {
//             DbgPrintEx(DPFLTR_SYSTEM_ID, DPFLTR_ERROR_LEVEL,
//                        "EX debug work: Unable to find process %p\n",
//                        ProcessToAttach ? ProcessToAttach : ProcessToKill);
//         }
//
//         /* We either have found a process, or we default to the current one */
//     }
//
//     /* If we have an address to page in... */
//     if (PageInAddress)
//     {
//         /* ... try to do it by attempting to read at this address */
//         _SEH2_TRY
//         {
//             ProbeForReadUchar(PageInAddress);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             DbgPrintEx(DPFLTR_SYSTEM_ID, DPFLTR_ERROR_LEVEL,
//                        "EX page in: Failed to page-in address 0x%p, Status 0x%08lx\n",
//                        PageInAddress, _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Break into the process (or the current one if Process == NULL) */
//     DbgBreakPointWithStatus(DBG_STATUS_WORKER);
//
//     /* If we are attached to a process, not the current one... */
//     if (Process)
//     {
//         /* ... we can detach from the process */
//         KeUnstackDetachProcess(&ApcState);
//         /* Dereference the process referenced by PsGetNextProcess() */
//         ObDereferenceObject(Process);
//     }
// }
//
// /**
//  * @brief
//  * Perform various queries to the kernel debugger.
//  *
//  * @param[in]   Command
//  * A SYSDBG_COMMAND value describing the kernel debugger command to perform.
//  *
//  * @param[in]   InputBuffer
//  * Pointer to a user-provided input command-specific buffer, whose length
//  * is given by InputBufferLength.
//  *
//  * @param[in]   InputBufferLength
//  * The size (in bytes) of the buffer pointed by InputBuffer.
//  *
//  * @param[out]  OutputBuffer
//  * Pointer to a user-provided command-specific output buffer, whose length
//  * is given by OutputBufferLength.
//  *
//  * @param[in]   OutputBufferLength
//  * The size (in bytes) of the buffer pointed by OutputBuffer.
//  *
//  * @param[out]  ReturnLength
//  * Optional pointer to a ULONG variable that receives the actual length of
//  * data written written in the output buffer. It is always zero, except for
//  * the live dump commands where an actual non-zero length is returned.
//  *
//  * @return
//  * STATUS_SUCCESS in case of success, or a proper error code otherwise.
//  *
//  * @remarks
//  *
//  * - The caller must have SeDebugPrivilege, otherwise the function fails
//  *   with STATUS_ACCESS_DENIED.
//  *
//  * - Only the live dump commands: SysDbgGetTriageDump, and SysDbgGetLiveKernelDump
//  *   (Win8.1+) are available even if the debugger is disabled or absent.
//  *
//  * - The following system-critical commands are not accessible anymore
//  *   for user-mode usage with this API on NT 5.2+ (Windows 2003 SP1 and later)
//  *   systems:
//  *
//  *   SysDbgQueryVersion,
//  *   SysDbgReadVirtual and SysDbgWriteVirtual,
//  *   SysDbgReadPhysical and SysDbgWritePhysical,
//  *   SysDbgReadControlSpace and SysDbgWriteControlSpace,
//  *   SysDbgReadIoSpace and SysDbgWriteIoSpace,
//  *   SysDbgReadMsr and SysDbgWriteMsr,
//  *   SysDbgReadBusData and SysDbgWriteBusData,
//  *   SysDbgCheckLowMemory.
//  *
//  *   For these, NtSystemDebugControl() will return STATUS_NOT_IMPLEMENTED.
//  *   They are now available from kernel-mode only with KdSystemDebugControl().
//  *
//  * @note
//  * See: https://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2004-2339
//  *
//  * @see KdSystemDebugControl()
//  **/
// NTSTATUS
// NTAPI
// NtSystemDebugControl(
//     _In_ SYSDBG_COMMAND Command,
//     _In_reads_bytes_(InputBufferLength) PVOID InputBuffer,
//     _In_ ULONG InputBufferLength,
//     _Out_writes_bytes_(OutputBufferLength) PVOID OutputBuffer,
//     _In_ ULONG OutputBufferLength,
//     _Out_opt_ PULONG ReturnLength)
// {
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//     ULONG Length = 0;
//     NTSTATUS Status;
//
//     /* Debugger controlling requires the debug privilege */
//     if (!SeSinglePrivilegeCheck(SeDebugPrivilege, PreviousMode))
//         return STATUS_ACCESS_DENIED;
//
//     _SEH2_TRY
//     {
//         if (PreviousMode != KernelMode)
//         {
//             if (InputBufferLength)
//                 ProbeForRead(InputBuffer, InputBufferLength, sizeof(ULONG));
//             if (OutputBufferLength)
//                 ProbeForWrite(OutputBuffer, OutputBufferLength, sizeof(ULONG));
//             if (ReturnLength)
//                 ProbeForWriteUlong(ReturnLength);
//         }
//
//         switch (Command)
//         {
//             case SysDbgQueryModuleInformation:
//                 /* Removed in WinNT4 */
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//
// #ifdef _M_IX86
//             case SysDbgQueryTraceInformation:
//             case SysDbgSetTracepoint:
//             case SysDbgSetSpecialCall:
//             case SysDbgClearSpecialCalls:
//             case SysDbgQuerySpecialCalls:
//                 UNIMPLEMENTED;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
// #endif
//
//             case SysDbgQueryVersion:
//             case SysDbgReadVirtual:
//             case SysDbgWriteVirtual:
//             case SysDbgReadPhysical:
//             case SysDbgWritePhysical:
//             case SysDbgReadControlSpace:
//             case SysDbgWriteControlSpace:
//             case SysDbgReadIoSpace:
//             case SysDbgWriteIoSpace:
//             case SysDbgReadMsr:
//             case SysDbgWriteMsr:
//             case SysDbgReadBusData:
//             case SysDbgWriteBusData:
//             case SysDbgCheckLowMemory:
//                 /* Those are implemented in KdSystemDebugControl */
//                 if (InitIsWinPEMode)
//                 {
//                     Status = KdSystemDebugControl(Command,
//                                                   InputBuffer, InputBufferLength,
//                                                   OutputBuffer, OutputBufferLength,
//                                                   &Length, PreviousMode);
//                 }
//                 else
//                 {
//                     Status = STATUS_NOT_IMPLEMENTED;
//                 }
//                 break;
//
//             case SysDbgBreakPoint:
//                 if (KdDebuggerEnabled)
//                 {
//                     DbgBreakPointWithStatus(DBG_STATUS_DEBUG_CONTROL);
//                     Status = STATUS_SUCCESS;
//                 }
//                 else
//                 {
//                     Status = STATUS_UNSUCCESSFUL;
//                 }
//                 break;
//
//             case SysDbgEnableKernelDebugger:
//                 Status = KdEnableDebugger();
//                 break;
//
//             case SysDbgDisableKernelDebugger:
//                 Status = KdDisableDebugger();
//                 break;
//
//             case SysDbgGetAutoKdEnable:
//                 if (OutputBufferLength != sizeof(BOOLEAN))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else
//                 {
//                     *(PBOOLEAN)OutputBuffer = KdAutoEnableOnEvent;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgSetAutoKdEnable:
//                 if (InputBufferLength != sizeof(BOOLEAN))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else if (KdPitchDebugger)
//                 {
//                     Status = STATUS_ACCESS_DENIED;
//                 }
//                 else
//                 {
//                     KdAutoEnableOnEvent = *(PBOOLEAN)InputBuffer;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgGetPrintBufferSize:
//                 if (OutputBufferLength != sizeof(ULONG))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else
//                 {
//                     /* Return buffer size only if KD is enabled */
//                     *(PULONG)OutputBuffer = KdPitchDebugger ? 0 : KdPrintBufferSize;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgSetPrintBufferSize:
//                 UNIMPLEMENTED;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//
//             case SysDbgGetKdUmExceptionEnable:
//                 if (OutputBufferLength != sizeof(BOOLEAN))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else
//                 {
//                     /* Unfortunately, the internal flag says if UM exceptions are disabled */
//                     *(PBOOLEAN)OutputBuffer = !KdIgnoreUmExceptions;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgSetKdUmExceptionEnable:
//                 if (InputBufferLength != sizeof(BOOLEAN))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else if (KdPitchDebugger)
//                 {
//                     Status = STATUS_ACCESS_DENIED;
//                 }
//                 else
//                 {
//                     /* Unfortunately, the internal flag says if UM exceptions are disabled */
//                     KdIgnoreUmExceptions = !*(PBOOLEAN)InputBuffer;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgGetTriageDump:
//                 UNIMPLEMENTED;
//                 Status = STATUS_NOT_IMPLEMENTED;
//                 break;
//
//             case SysDbgGetKdBlockEnable:
//                 if (OutputBufferLength != sizeof(BOOLEAN))
//                 {
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else
//                 {
//                     *(PBOOLEAN)OutputBuffer = KdBlockEnable;
//                     Status = STATUS_SUCCESS;
//                 }
//                 break;
//
//             case SysDbgSetKdBlockEnable:
//                 Status = KdChangeOption(KD_OPTION_SET_BLOCK_ENABLE,
//                                         InputBufferLength,
//                                         InputBuffer,
//                                         OutputBufferLength,
//                                         OutputBuffer,
//                                         &Length);
//                 break;
//
//             default:
//                 Status = STATUS_INVALID_INFO_CLASS;
//                 break;
//         }
//
//         if (ReturnLength)
//             *ReturnLength = Length;
//
//         _SEH2_YIELD(return Status);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         _SEH2_YIELD(return _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
