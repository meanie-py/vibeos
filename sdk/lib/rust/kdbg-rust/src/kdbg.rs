use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    CONTEXT, EXCEPTION_RECORD64, KDSTATUS, PDBGKD_ANY_WAIT_STATE_CHANGE,
    PDBGKD_MANIPULATE_STATE64, PKD_CONTEXT, PLDR_DATA_TABLE_ENTRY, PSTRING,
};

// External declarations from other modules.
//
// KdD0Transition / KdSendPacket / KdReceivePacket / etc. are NTAPI
// (stdcall) per kddll.h. KdbpSymFindModule, KdbSymProcessSymbols,
// KdbEnterDebuggerException, KeGet/SetContextPc, KdbPrintf are cdecl
// (no NTAPI in their declarations).
extern "system" {
    fn KdD0Transition() -> NTSTATUS;
    fn KdD3Transition() -> NTSTATUS;
    fn KdSave(SleepTransition: BOOLEAN) -> NTSTATUS;
    fn KdRestore(SleepTransition: BOOLEAN) -> NTSTATUS;
    fn KdSendPacket( PacketType: ULONG, MessageHeader: PSTRING, MessageData: PSTRING, Context: PKD_CONTEXT, );
    fn KdReceivePacket( PacketType: ULONG, MessageHeader: PSTRING, MessageData: PSTRING, DataLength: *mut ULONG, Context: PKD_CONTEXT, ) -> KDSTATUS;
}
extern "C" {
    fn KdbpSymFindModule( Address: PVOID, Index: LONG, LdrEntry: *mut PLDR_DATA_TABLE_ENTRY, ) -> BOOLEAN;
    fn KdbSymProcessSymbols(LdrEntry: PLDR_DATA_TABLE_ENTRY, Load: BOOLEAN);
    fn KdbEnterDebuggerException( ExceptionRecord: *mut EXCEPTION_RECORD64, PreviousMode: BOOLEAN, Context: *mut CONTEXT, FirstChance: BOOLEAN, ) -> KD_CONTINUE_TYPE;
    fn KeSetContextPc(Context: *mut CONTEXT, Pc: ULONG_PTR);
    fn KeGetContextPc(Context: *const CONTEXT) -> ULONG_PTR;
}

// Constants
const PACKET_TYPE_KD_DEBUG_IO: ULONG = 3;
const PACKET_TYPE_KD_STATE_CHANGE64: ULONG = 7;
const PACKET_TYPE_KD_STATE_MANIPULATE: ULONG = 2;
const PACKET_TYPE_KD_POLL_BREAKIN: ULONG = 0x8;

// windbgkd.h. The port's 0x3130.. and the state changes 3/4 below never matched
// what kd64 sends, so KDBG never saw an exception or a symbol load.
const DbgKdContinueApi: ULONG = 0x3136;
const DbgKdGetContextApi: ULONG = 0x3132;
const DbgKdSetContextApi: ULONG = 0x3133;

const DbgKdLoadSymbolsStateChange: ULONG = 0x00003031;
const DbgKdExceptionStateChange: ULONG = 0x00003030;

const STATUS_ASSERTION_FAILURE: NTSTATUS = 0xC0000420u32 as NTSTATUS;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;

const KdPacketReceived: KDSTATUS = 0;
const KdPacketTimedOut: KDSTATUS = 1;

type KD_CONTINUE_TYPE = u32;
const kdHandleException: KD_CONTINUE_TYPE = 0;

static mut KdbgNextApiNumber: ULONG = DbgKdContinueApi;
static mut KdbgContext: CONTEXT = unsafe { core::mem::zeroed() };
static mut KdbgExceptionRecord: EXCEPTION_RECORD64 = unsafe { core::mem::zeroed() };
static mut KdbgFirstChanceException: BOOLEAN = 0u8;
static mut KdbgContinueStatus: NTSTATUS = STATUS_SUCCESS;

#[no_mangle]
pub unsafe extern "system" fn KdbgD0Transition() -> NTSTATUS {
    KdD0Transition()
}

#[no_mangle]
pub unsafe extern "system" fn KdbgD3Transition() -> NTSTATUS {
    KdD3Transition()
}

#[no_mangle]
pub unsafe extern "system" fn KdbgSave(SleepTransition: BOOLEAN) -> NTSTATUS {
    KdSave(SleepTransition)
}

#[no_mangle]
pub unsafe extern "system" fn KdbgRestore(SleepTransition: BOOLEAN) -> NTSTATUS {
    KdRestore(SleepTransition)
}

#[no_mangle]
pub unsafe extern "system" fn KdbgSendPacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    Context: PKD_CONTEXT,
) {
    if PacketType == PACKET_TYPE_KD_DEBUG_IO {
        KdSendPacket(PacketType, MessageHeader, MessageData, Context);
        return;
    }

    if PacketType == PACKET_TYPE_KD_STATE_CHANGE64 {
        let WaitStateChange = (*MessageHeader).Buffer as PDBGKD_ANY_WAIT_STATE_CHANGE;
        if (*WaitStateChange).NewState == DbgKdLoadSymbolsStateChange {
            let mut LdrEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
            if KdbpSymFindModule(
                (*WaitStateChange).u.LoadSymbols.BaseOfDll as PVOID,
                -1,
                &mut LdrEntry,
            ) != 0
            {
                KdbSymProcessSymbols(
                    LdrEntry,
                    if (*WaitStateChange).u.LoadSymbols.UnloadSymbols != 0 {
                        0u8
                    } else {
                        1u8
                    },
                );
            }
            return;
        } else if (*WaitStateChange).NewState == DbgKdExceptionStateChange {
            KdbgNextApiNumber = DbgKdGetContextApi;
            KdbgExceptionRecord = (*WaitStateChange).u.Exception.ExceptionRecord;
            KdbgFirstChanceException = (*WaitStateChange).u.Exception.FirstChance as BOOLEAN;
            return;
        }
    } else if PacketType == PACKET_TYPE_KD_STATE_MANIPULATE {
        let ManipulateState = (*MessageHeader).Buffer as PDBGKD_MANIPULATE_STATE64;
        if (*ManipulateState).ApiNumber == DbgKdGetContextApi {
            if KdbgExceptionRecord.ExceptionCode == STATUS_ASSERTION_FAILURE {
                KeSetContextPc(&raw mut KdbgContext, KeGetContextPc(&raw const KdbgContext) + 2);
            }

            let Result = KdbEnterDebuggerException(
                &raw mut KdbgExceptionRecord,
                (KdbgContext.SegCs & 1) as BOOLEAN,
                &raw mut KdbgContext,
                KdbgFirstChanceException,
            );

            if Result != kdHandleException {
                KdbgContinueStatus = STATUS_SUCCESS;
            } else {
                KdbgContinueStatus = STATUS_UNSUCCESSFUL;
            }
            KdbgNextApiNumber = DbgKdSetContextApi;
            return;
        } else if (*ManipulateState).ApiNumber == DbgKdSetContextApi {
            KdbgNextApiNumber = DbgKdContinueApi;
            return;
        }
    }

    crate::kdb_printf!(b"%s: PacketType %d is UNIMPLEMENTED\n\0",
        b"KdSendPacket\0".as_ptr() as PCHAR,
        PacketType,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KdbgReceivePacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    DataLength: *mut ULONG,
    Context: PKD_CONTEXT,
) -> KDSTATUS {
    if PacketType == PACKET_TYPE_KD_POLL_BREAKIN {
        return KdPacketTimedOut;
    }

    if PacketType == PACKET_TYPE_KD_DEBUG_IO {
        return KdReceivePacket(PacketType, MessageHeader, MessageData, DataLength, Context);
    }

    if PacketType == PACKET_TYPE_KD_STATE_MANIPULATE {
        let ManipulateState = (*MessageHeader).Buffer as PDBGKD_MANIPULATE_STATE64;
        core::ptr::write_bytes(
            (*MessageHeader).Buffer as *mut u8,
            0,
            (*MessageHeader).MaximumLength as usize,
        );
        if KdbgNextApiNumber == DbgKdGetContextApi {
            (*ManipulateState).ApiNumber = DbgKdGetContextApi;
            (*MessageData).Length = 0;
            (*MessageData).Buffer = &raw mut KdbgContext as *mut CONTEXT as PCHAR;
            return KdPacketReceived;
        } else if KdbgNextApiNumber == DbgKdSetContextApi {
            (*ManipulateState).ApiNumber = DbgKdSetContextApi;
            (*MessageData).Length = core::mem::size_of::<CONTEXT>() as USHORT;
            (*MessageData).Buffer = &raw mut KdbgContext as *mut CONTEXT as PCHAR;
            return KdPacketReceived;
        } else if KdbgNextApiNumber != DbgKdContinueApi {
            crate::kdb_printf!(b"%s:%d is UNIMPLEMENTED\n\0",
                b"KdReceivePacket\0".as_ptr() as PCHAR,
                447,
            );
        }
        (*ManipulateState).ApiNumber = DbgKdContinueApi;
        (*ManipulateState).u.Continue.ContinueStatus = KdbgContinueStatus;

        KdbgNextApiNumber = DbgKdContinueApi;
        KdbgContinueStatus = STATUS_SUCCESS;

        return KdPacketReceived;
    }

    crate::kdb_printf!(b"%s: PacketType %d is UNIMPLEMENTED\n\0",
        b"KdReceivePacket\0".as_ptr() as PCHAR,
        PacketType,
    );
    KdPacketTimedOut
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KDBG Kernel Debugger
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Kernel Debugger Initialization
//  * COPYRIGHT:   Copyright 2020-2021 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2021 Jérôme Gardou <jerome.gardou@reactos.org>
//  *              Copyright 2023 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #include "kdb.h"
//
// /* GLOBALS *******************************************************************/
//
// static ULONG KdbgNextApiNumber = DbgKdContinueApi;
// static CONTEXT KdbgContext;
// static EXCEPTION_RECORD64 KdbgExceptionRecord;
// static BOOLEAN KdbgFirstChanceException;
// static NTSTATUS KdbgContinueStatus = STATUS_SUCCESS;
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// KdD0Transition(VOID)
// #undef KdD0Transition
// #define pKdD0Transition KdD0Transition
// {
//     /* Call KdTerm */
//     return pKdD0Transition();
// }
//
// NTSTATUS
// NTAPI
// KdD3Transition(VOID)
// #undef KdD3Transition
// #define pKdD3Transition KdD3Transition
// {
//     /* Call KdTerm */
//     return pKdD3Transition();
// }
//
// NTSTATUS
// NTAPI
// KdSave(
//     _In_ BOOLEAN SleepTransition)
// #undef KdSave
// #define pKdSave KdSave
// {
//     /* Call KdTerm */
//     return pKdSave(SleepTransition);
// }
//
// NTSTATUS
// NTAPI
// KdRestore(
//     _In_ BOOLEAN SleepTransition)
// #undef KdRestore
// #define pKdRestore KdRestore
// {
//     /* Call KdTerm */
//     return pKdRestore(SleepTransition);
// }
//
// VOID
// NTAPI
// KdSendPacket(
//     _In_ ULONG PacketType,
//     _In_ PSTRING MessageHeader,
//     _In_opt_ PSTRING MessageData,
//     _Inout_ PKD_CONTEXT Context)
// #undef KdSendPacket
// #define pKdSendPacket KdSendPacket
// {
//     if (PacketType == PACKET_TYPE_KD_DEBUG_IO)
//     {
//         /* Call KdTerm */
//         pKdSendPacket(PacketType, MessageHeader, MessageData, Context);
//         return;
//     }
//
//     /* Debugger-only packets */
//     if (PacketType == PACKET_TYPE_KD_STATE_CHANGE64)
//     {
//         PDBGKD_ANY_WAIT_STATE_CHANGE WaitStateChange = (PDBGKD_ANY_WAIT_STATE_CHANGE)MessageHeader->Buffer;
//         if (WaitStateChange->NewState == DbgKdLoadSymbolsStateChange)
//         {
//             /* Load or unload symbols */
//             PLDR_DATA_TABLE_ENTRY LdrEntry;
//             if (KdbpSymFindModule((PVOID)(ULONG_PTR)WaitStateChange->u.LoadSymbols.BaseOfDll, -1, &LdrEntry))
//             {
//                 KdbSymProcessSymbols(LdrEntry, !WaitStateChange->u.LoadSymbols.UnloadSymbols);
//             }
//             return;
//         }
//         else if (WaitStateChange->NewState == DbgKdExceptionStateChange)
//         {
//             KdbgNextApiNumber = DbgKdGetContextApi;
//             KdbgExceptionRecord = WaitStateChange->u.Exception.ExceptionRecord;
//             KdbgFirstChanceException = WaitStateChange->u.Exception.FirstChance;
//             return;
//         }
//     }
//     else if (PacketType == PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         PDBGKD_MANIPULATE_STATE64 ManipulateState = (PDBGKD_MANIPULATE_STATE64)MessageHeader->Buffer;
//         if (ManipulateState->ApiNumber == DbgKdGetContextApi)
//         {
//             KD_CONTINUE_TYPE Result;
//
//             /* Check if this is an assertion failure */
//             if (KdbgExceptionRecord.ExceptionCode == STATUS_ASSERTION_FAILURE)
//             {
//                 /* Bump EIP to the instruction following the int 2C */
//                 KeSetContextPc(&KdbgContext, KeGetContextPc(&KdbgContext) + 2);
//             }
//
//             Result = KdbEnterDebuggerException(&KdbgExceptionRecord,
//                                                KdbgContext.SegCs & 1,
//                                                &KdbgContext,
//                                                KdbgFirstChanceException);
// #if 0
//             /* Manually dump the stack for the user */
//             KeRosDumpStackFrames(NULL, 0);
//             Result = kdHandleException;
// #endif
//             if (Result != kdHandleException)
//                 KdbgContinueStatus = STATUS_SUCCESS;
//             else
//                 KdbgContinueStatus = STATUS_UNSUCCESSFUL;
//             KdbgNextApiNumber = DbgKdSetContextApi;
//             return;
//         }
//         else if (ManipulateState->ApiNumber == DbgKdSetContextApi)
//         {
//             KdbgNextApiNumber = DbgKdContinueApi;
//             return;
//         }
//     }
//
//     KdbPrintf("%s: PacketType %d is UNIMPLEMENTED\n", __FUNCTION__, PacketType);
//     return;
// }
//
// KDSTATUS
// NTAPI
// KdReceivePacket(
//     _In_ ULONG PacketType,
//     _Out_ PSTRING MessageHeader,
//     _Out_ PSTRING MessageData,
//     _Out_ PULONG DataLength,
//     _Inout_ PKD_CONTEXT Context)
// #undef KdReceivePacket
// #define pKdReceivePacket KdReceivePacket
// {
//     if (PacketType == PACKET_TYPE_KD_POLL_BREAKIN)
//     {
//         // FIXME TODO: Implement break-in for the debugger
//         // and return KdPacketReceived when handled properly.
//         return KdPacketTimedOut;
//     }
//
//     if (PacketType == PACKET_TYPE_KD_DEBUG_IO)
//     {
//         /* Call KdTerm */
//         return pKdReceivePacket(PacketType,
//                                 MessageHeader,
//                                 MessageData,
//                                 DataLength,
//                                 Context);
//     }
//
//     /* Debugger-only packets */
//     if (PacketType == PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         PDBGKD_MANIPULATE_STATE64 ManipulateState = (PDBGKD_MANIPULATE_STATE64)MessageHeader->Buffer;
//         RtlZeroMemory(MessageHeader->Buffer, MessageHeader->MaximumLength);
//         if (KdbgNextApiNumber == DbgKdGetContextApi)
//         {
//             ManipulateState->ApiNumber = DbgKdGetContextApi;
//             MessageData->Length = 0;
//             MessageData->Buffer = (PCHAR)&KdbgContext;
//             return KdPacketReceived;
//         }
//         else if (KdbgNextApiNumber == DbgKdSetContextApi)
//         {
//             ManipulateState->ApiNumber = DbgKdSetContextApi;
//             MessageData->Length = sizeof(KdbgContext);
//             MessageData->Buffer = (PCHAR)&KdbgContext;
//             return KdPacketReceived;
//         }
//         else if (KdbgNextApiNumber != DbgKdContinueApi)
//         {
//             KdbPrintf("%s:%d is UNIMPLEMENTED\n", __FUNCTION__, __LINE__);
//         }
//         ManipulateState->ApiNumber = DbgKdContinueApi;
//         ManipulateState->u.Continue.ContinueStatus = KdbgContinueStatus;
//
//         /* Prepare for next time */
//         KdbgNextApiNumber = DbgKdContinueApi;
//         KdbgContinueStatus = STATUS_SUCCESS;
//
//         return KdPacketReceived;
//     }
//
//     KdbPrintf("%s: PacketType %d is UNIMPLEMENTED\n", __FUNCTION__, PacketType);
//     return KdPacketTimedOut;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
