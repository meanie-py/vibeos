use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
// Use bindgen process/thread types (full layout), NOT the opaque ps:: stubs.
use ntkrnl_sys::ddk::{CONTEXT, DBGKD_ANY_WAIT_STATE_CHANGE, DBGKD_DEBUG_IO, DBGKD_GET_VERSION64, DBGKD_MANIPULATE_STATE64, EPROCESS, HANDLE, KDDEBUGGER_DATA64, KDSTATUS, LIST_ENTRY, PEPROCESS, PETHREAD, PKD_CONTEXT, PLIST_ENTRY, PSTRING, UINT_PTR, _DBGKD_DEBUG_DATA_HEADER64, PsGetThreadId, PsGetThreadProcessId};
use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kdgdb_h::{handle_to_gdb_pid, handle_to_gdb_tid};
use crate::gdb_send::{gdb_send_exception, gdb_send_debug_io};
use crate::kdcom::KdpPollBreakIn;
use crate::gdb_input::gdb_receive_and_interpret_packet;
// Disambiguate: these consts live in both globals and constants (both
// glob-imported). Pin them to `constants` so the glob ambiguity resolves.
use ntkrnl_sys::constants::{
    PACKET_TYPE_KD_STATE_MANIPULATE, PACKET_TYPE_KD_STATE_CHANGE64,
    DbgKdExceptionStateChange,
};

// ---- file-scope preamble (translated) ----
pub static mut InException: BOOLEAN = 0;
pub static mut KdVersion: DBGKD_GET_VERSION64 = DBGKD_GET_VERSION64 {
    MajorVersion: 0,
    MinorVersion: 0,
    ProtocolVersion: 0,
    KdSecondaryVersion: 0,
    Flags: 0,
    MachineType: 0,
    MaxPacketType: 0,
    MaxStateChange: 0,
    MaxManipulate: 0,
    Simulation: 0,
    Unused: [0; 1],
    KernBase: 0,
    PsLoadedModuleList: 0,
    DebuggerDataList: 0,
};
pub static mut KdDebuggerDataBlock: *mut KDDEBUGGER_DATA64 = core::ptr::null_mut();
pub static mut ProcessListHead: *mut LIST_ENTRY = core::ptr::null_mut();
pub static mut ModuleListHead: *mut LIST_ENTRY = core::ptr::null_mut();
// C: function pointers that may be NULL → model as `Option<fn>`.
pub type KDP_SEND_HANDLER =
    Option<unsafe fn(ULONG, PSTRING, PSTRING) -> BOOLEAN>;
pub type KDP_MANIPULATESTATE_HANDLER =
    Option<unsafe fn(*mut DBGKD_MANIPULATE_STATE64, PSTRING, *mut ULONG, PKD_CONTEXT) -> KDSTATUS>;
pub static mut KdpSendPacketHandler: KDP_SEND_HANDLER = Some(FirstSendHandler);
pub static mut KdpManipulateStateHandler: KDP_MANIPULATESTATE_HANDLER = None;
pub static mut CurrentStateChange: DBGKD_ANY_WAIT_STATE_CHANGE = DBGKD_ANY_WAIT_STATE_CHANGE {
    NewState: 0,
    ProcessorLevel: 0,
    Processor: 0,
    NumberProcessors: 0,
    Thread: 0,
    ProgramCounter: 0,
    u: unsafe { core::mem::zeroed() },
    __bindgen_anon_1: unsafe { core::mem::zeroed() },
};
pub static mut CurrentContext: CONTEXT = unsafe { core::mem::zeroed() };
pub static mut TheIdleProcess: PEPROCESS = core::ptr::null_mut();
pub static mut TheIdleThread: PETHREAD = core::ptr::null_mut();

// gdb_dbg_tid / gdb_dbg_pid are defined in gdb_input.c (gdb_input.rs).
use crate::gdb_input::{gdb_dbg_tid, gdb_dbg_pid};

unsafe fn GetContextSendHandler(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING
) -> BOOLEAN {

    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;
    let Context = (*MessageData).Buffer as *const CONTEXT;

    if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
        || ((*State).ApiNumber != DbgKdGetContextApi)
        || ((*MessageData).Length < core::mem::size_of::<CONTEXT>() as u16)
    {
        KDDBGPRINT!(b"ERROR: Received wrong packet from KD.\n\0".as_ptr() as *const i8);
        return 0u8;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!(CurrentContext) as *mut core::ffi::c_void,
        Context as *const core::ffi::c_void,
        core::mem::size_of::<CONTEXT>()
    );
    KdpSendPacketHandler = None;
    1u8
}

unsafe fn GetContextManipulateHandler(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT
) -> KDSTATUS {
    (*State).ApiNumber = DbgKdGetContextApi;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    (*MessageData).Length = 0;

    KdpSendPacketHandler = Some(GetContextSendHandler);
    KdpManipulateStateHandler = None;

    return KdPacketReceived;
}
unsafe fn SetContextSendHandler(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING
) -> BOOLEAN {
    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

    if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
        || ((*State).ApiNumber != DbgKdSetContextApi)
        || ((*State).ReturnStatus != STATUS_SUCCESS)
    {
        KDDBGPRINT!(b"BAD BAD BAD not manipulating state for sending context.\n\0".as_ptr() as PCHAR);
        return 0u8;
    }

    KdpSendPacketHandler = None;
    return 1u8;
}
// Stored in KdpManipulateStateHandler (an `Option<unsafe fn ...>`), so it must
// be a plain `unsafe fn` (Rust ABI), not `extern "C"`.
pub unsafe fn SetContextManipulateHandler(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT
) -> KDSTATUS {
    (*State).ApiNumber = DbgKdSetContextApi;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    (*MessageData).Length = core::mem::size_of::<CONTEXT>() as u16;

    if ((*MessageData).MaximumLength as usize) < core::mem::size_of::<CONTEXT>() {
        KDDBGPRINT!(b"Wrong message length %u.\n\0".as_ptr() as *const i8, (*MessageData).MaximumLength);
        loop {}
    }

    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!(CurrentContext) as *const u8,
        (*MessageData).Buffer as *mut u8,
        core::mem::size_of::<CONTEXT>()
    );

    KdpSendPacketHandler = Some(SetContextSendHandler);
    KdpManipulateStateHandler = None;

    KdPacketReceived
}

unsafe fn send_kd_state_change(StateChange: *mut DBGKD_ANY_WAIT_STATE_CHANGE) {
    InException = 1u8;

    match (*StateChange).NewState {
        DbgKdLoadSymbolsStateChange | DbgKdExceptionStateChange => {
            let Thread = (*StateChange).Thread as PETHREAD;
            CurrentStateChange = *StateChange;
            KDDBGPRINT!(
                b"Exception 0x%08x in thread p%p.%p.\n\0".as_ptr() as PCHAR,
                (*StateChange).__bindgen_anon_1.Exception.ExceptionRecord.ExceptionCode,
                PsGetThreadProcessId(Thread),
                PsGetThreadId(Thread)
            );
            gdb_dbg_tid = handle_to_gdb_tid(PsGetThreadId(Thread));
            #[cfg(not(feature = "MONOPROCESS"))]
            {
                gdb_dbg_pid = handle_to_gdb_pid(PsGetThreadProcessId(Thread));
            }
            #[cfg(feature = "MONOPROCESS")]
            {
                gdb_dbg_pid = 0;
            }
            gdb_send_exception();
            KdpManipulateStateHandler = Some(GetContextManipulateHandler);
        }
        _ => {
            KDDBGPRINT!(
                b"Unknown StateChange %u.\n\0".as_ptr() as PCHAR,
                (*StateChange).NewState
            );
            loop {}
        }
    }
}
unsafe fn send_kd_debug_io(
    DebugIO: *mut DBGKD_DEBUG_IO,
    String: PSTRING) {

    if InException != 0 {
        return;
    }

    match (*DebugIO).ApiNumber {
        DbgKdPrintStringApi | DbgKdGetStringApi => {
            gdb_send_debug_io(String, 1u8);
        }
        _ => {
            KDDBGPRINT!(b"Unknown ApiNumber %u.\n\0".as_ptr(), (*DebugIO).ApiNumber);
            loop {}
        }
    }
}
unsafe fn send_kd_state_manipulate(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING) {
    match (*State).ApiNumber {
        _ => {
            KDDBGPRINT!(b"Unknown ApiNumber %u.\n\0".as_ptr() as *const i8, (*State).ApiNumber);
            loop {}
        }
    }
}
// Stored in KdpManipulateStateHandler (an `Option<unsafe fn ...>`), so it must
// be a plain `unsafe fn` (Rust ABI), not `extern "C"`.
pub unsafe fn ContinueManipulateStateHandler(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT
) -> KDSTATUS {
    (*State).ApiNumber = DbgKdContinueApi;
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    if !MessageData.is_null() {
        (*MessageData).Length = 0;
    }
    *MessageLength = 0;
    (*State).u.Continue.ContinueStatus = STATUS_SUCCESS;

    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = None;
    InException = 0u8;

    KdPacketReceived
}

unsafe fn GetVersionSendHandler(
    PacketType: u32,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
) -> u8 {
    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;
    let DebuggerDataList: PLIST_ENTRY;

    if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
        || ((*State).ApiNumber != DbgKdGetVersionApi)
        || !NT_SUCCESS((*State).ReturnStatus)
    {
        KDDBGPRINT!(b"Wrong packet received after asking for data.\n\0".as_ptr());
        return 0u8;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!(KdVersion) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*State).u.GetVersion64) as *const core::ffi::c_void,
        core::mem::size_of_val(&KdVersion),
    );
    DebuggerDataList = *(core::ptr::addr_of!(KdVersion.DebuggerDataList) as *const PLIST_ENTRY);
    KdDebuggerDataBlock = ((*DebuggerDataList).Flink as usize
        - core::mem::offset_of!(KDDEBUGGER_DATA64, Header)
        - core::mem::offset_of!(_DBGKD_DEBUG_DATA_HEADER64, List)) as *mut KDDEBUGGER_DATA64;
    ProcessListHead = *(core::ptr::addr_of!((*KdDebuggerDataBlock).PsActiveProcessHead) as *const PLIST_ENTRY);
    ModuleListHead = *(core::ptr::addr_of!((*KdDebuggerDataBlock).PsLoadedModuleList) as *const PLIST_ENTRY);

    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = Some(GetContextManipulateHandler);
    return 1u8;
}

unsafe fn GetVersionManipulateStateHandler(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {
    (*State).ApiNumber = DbgKdGetVersionApi;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;

    KdpSendPacketHandler = Some(GetVersionSendHandler);
    KdpManipulateStateHandler = None;

    KdPacketReceived
}

unsafe fn FirstSendHandler(
    PacketType: u32,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
) -> u8 {

    let StateChange = (*MessageHeader).Buffer as *mut DBGKD_ANY_WAIT_STATE_CHANGE;
    let Thread: PETHREAD;

    if PacketType != PACKET_TYPE_KD_STATE_CHANGE64 {
        KDDBGPRINT!(b"First KD packet is not a state change!\n\0".as_ptr() as *const i8);
        return 0u8;
    }

    KDDBGPRINT!(b"KDGDB: START!\n\0".as_ptr() as *const i8);

    Thread = (*StateChange).Thread as usize as PETHREAD;

    CurrentStateChange = *StateChange;
    gdb_dbg_tid = handle_to_gdb_tid(PsGetThreadId(Thread));
    #[cfg(feature = "MONOPROCESS")]
    {
        gdb_dbg_pid = 0;
    }
    #[cfg(not(feature = "MONOPROCESS"))]
    {
        gdb_dbg_pid = handle_to_gdb_pid(PsGetThreadProcessId(Thread));
    }

    TheIdleThread = Thread;
    TheIdleProcess = (*Thread).Tcb.__bindgen_anon_1.ApcState.Process as PEPROCESS;

    KDDBGPRINT!(
        b"Pid Tid of the first message: %lx, %lx.\n\0".as_ptr() as *const i8,
        gdb_dbg_pid,
        gdb_dbg_tid,
    );

    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = Some(GetVersionManipulateStateHandler);
    return 1u8;
}

#[no_mangle]
pub unsafe extern "system" fn KdReceivePacket(
    PacketType: u32,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    DataLength: *mut u32,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {

    KDDBGPRINT!(b"KdReceivePacket --> \0".as_ptr() as *const i8);

    if PacketType == PACKET_TYPE_KD_POLL_BREAKIN {
        static mut firstTime: u8 = 1u8;
        KDDBGPRINT!(b"Polling break in.\n\0".as_ptr() as *const i8);
        if firstTime != 0 {
            firstTime = 0u8;
            return 0;
        }

        return KdpPollBreakIn();
    }

    if PacketType == PACKET_TYPE_KD_DEBUG_IO {
        static mut ignore: u8 = 0u8;
        KDDBGPRINT!(b"Debug prompt.\n\0".as_ptr() as *const i8);
        (*MessageData).Length = 1;
        *(*MessageData).Buffer.offset(0) = if ignore != 0 { b'i' as i8 } else { b'b' as i8 };
        ignore = if ignore != 0 { 0u8 } else { 1u8 };
        return 0;
    }

    if PacketType == PACKET_TYPE_KD_STATE_MANIPULATE {
        let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

        KDDBGPRINT!(b"State manipulation: \0".as_ptr() as *const i8);

        if let Some(handler) = KdpManipulateStateHandler {
            KDDBGPRINT!(b"We have a manipulate state handler.\n\0".as_ptr() as *const i8);
            return handler(State, MessageData, DataLength, KdContext);
        }

        KDDBGPRINT!(b"Receiving data from GDB.\n\0".as_ptr() as *const i8);
        return gdb_receive_and_interpret_packet(State, MessageData, DataLength, KdContext);
    }

    loop {}
}
#[no_mangle]
pub unsafe extern "system" fn KdSendPacket(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
    KdContext: PKD_CONTEXT) {
    if PacketType == PACKET_TYPE_KD_DEBUG_IO {
        send_kd_debug_io((*MessageHeader).Buffer as *mut DBGKD_DEBUG_IO, MessageData);
        return;
    }

    if KdpSendPacketHandler.is_some()
        && (KdpSendPacketHandler.unwrap())(PacketType, MessageHeader, MessageData) != 0
    {
        return;
    }

    match PacketType {
        PACKET_TYPE_KD_STATE_CHANGE64 => {
            send_kd_state_change((*MessageHeader).Buffer as *mut DBGKD_ANY_WAIT_STATE_CHANGE);
            return;
        }
        PACKET_TYPE_KD_DEBUG_IO => {
            send_kd_debug_io((*MessageHeader).Buffer as *mut DBGKD_DEBUG_IO, MessageData);
        }
        PACKET_TYPE_KD_STATE_MANIPULATE => {
            send_kd_state_manipulate((*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64, MessageData);
        }
        _ => {
            KDDBGPRINT!(b"Unknown packet type %u.\n\0".as_ptr(), PacketType);
            loop {}
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/kdpacket.c
//  * PURPOSE:         Base functions for the kernel debugger.
//  */
//
// #include "kdgdb.h"
//
// /* LOCALS *********************************************************************/
// static
// BOOLEAN
// FirstSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData);
// static BOOLEAN InException = FALSE;
//
// /* GLOBALS ********************************************************************/
// DBGKD_GET_VERSION64 KdVersion;
// KDDEBUGGER_DATA64* KdDebuggerDataBlock;
// LIST_ENTRY* ProcessListHead;
// LIST_ENTRY* ModuleListHead;
// /* Callbacks used to communicate with KD aside from GDB */
// KDP_SEND_HANDLER KdpSendPacketHandler = FirstSendHandler;
// KDP_MANIPULATESTATE_HANDLER KdpManipulateStateHandler = NULL;
// /* Data describing the current exception */
// DBGKD_ANY_WAIT_STATE_CHANGE CurrentStateChange;
// CONTEXT CurrentContext;
// PEPROCESS TheIdleProcess;
// PETHREAD TheIdleThread;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// BOOLEAN
// GetContextSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData
// )
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//     const CONTEXT* Context = (const CONTEXT*)MessageData->Buffer;
//
//     if ((PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//             || (State->ApiNumber != DbgKdGetContextApi)
//             || (MessageData->Length < sizeof(*Context)))
//     {
//         KDDBGPRINT!("ERROR: Received wrong packet from KD.\n");
//         return FALSE;
//     }
//
//     /* Just copy it */
//     RtlCopyMemory(&CurrentContext, Context, sizeof(*Context));
//     KdpSendPacketHandler = NULL;
//     return TRUE;
// }
//
// static
// KDSTATUS
// GetContextManipulateHandler(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext
// )
// {
//     State->ApiNumber = DbgKdGetContextApi;
//     State->Processor = CurrentStateChange.Processor;
//     State->ReturnStatus = STATUS_SUCCESS;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     MessageData->Length = 0;
//
//     /* Update the send <-> receive loop handler */
//     KdpSendPacketHandler = GetContextSendHandler;
//     KdpManipulateStateHandler = NULL;
//
//     return KdPacketReceived;
// }
//
// static
// BOOLEAN
// SetContextSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData
// )
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//
//     /* We just confirm that all went well */
//     if ((PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//             || (State->ApiNumber != DbgKdSetContextApi)
//             || (State->ReturnStatus != STATUS_SUCCESS))
//     {
//         /* Should we bugcheck ? */
//         KDDBGPRINT!("BAD BAD BAD not manipulating state for sending context.\n");
//         return FALSE;
//     }
//
//     KdpSendPacketHandler = NULL;
//     return TRUE;
// }
//
// KDSTATUS
// SetContextManipulateHandler(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext
// )
// {
//     State->ApiNumber = DbgKdSetContextApi;
//     State->Processor = CurrentStateChange.Processor;
//     State->ReturnStatus = STATUS_SUCCESS;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     MessageData->Length = sizeof(CurrentContext);
//
//     if (MessageData->MaximumLength < sizeof(CurrentContext))
//     {
//         KDDBGPRINT!("Wrong message length %u.\n", MessageData->MaximumLength);
//         while (1);
//     }
//
//     RtlCopyMemory(MessageData->Buffer, &CurrentContext, sizeof(CurrentContext));
//
//     /* Update the send <-> receive loop handlers */
//     KdpSendPacketHandler = Some(SetContextSendHandler);
//     KdpManipulateStateHandler = NULL;
//
//     return KdPacketReceived;
// }
//
// static
// void
// send_kd_state_change(DBGKD_ANY_WAIT_STATE_CHANGE* StateChange)
// {
//     InException = TRUE;
//
//     switch (StateChange->NewState)
//     {
//     case DbgKdLoadSymbolsStateChange:
//     case DbgKdExceptionStateChange:
//     {
//         PETHREAD Thread = (PETHREAD)(ULONG_PTR)StateChange->Thread;
//         /* Save current state for later GDB queries */
//         CurrentStateChange = *StateChange;
//         KDDBGPRINT!("Exception 0x%08x in thread p%p.%p.\n",
//             StateChange->u.Exception.ExceptionRecord.ExceptionCode,
//             PsGetThreadProcessId(Thread),
//             PsGetThreadId(Thread));
//         /* Set the current debugged process/thread accordingly */
//         gdb_dbg_tid = handle_to_gdb_tid(PsGetThreadId(Thread));
// #if MONOPROCESS
//         gdb_dbg_pid = 0;
// #else
//         gdb_dbg_pid = handle_to_gdb_pid(PsGetThreadProcessId(Thread));
// #endif
//         gdb_send_exception();
//         /* Next receive call will ask for the context */
//         KdpManipulateStateHandler = GetContextManipulateHandler;
//         break;
//     }
//     default:
//         KDDBGPRINT!("Unknown StateChange %u.\n", StateChange->NewState);
//         while (1);
//     }
// }
//
// static
// void
// send_kd_debug_io(
//      DBGKD_DEBUG_IO* DebugIO,
//      PSTRING String)
// {
//     if (InException)
//         return;
//
//     switch (DebugIO->ApiNumber)
//     {
//     case DbgKdPrintStringApi:
//     case DbgKdGetStringApi:
//         gdb_send_debug_io(String, TRUE);
//         break;
//     default:
//         KDDBGPRINT!("Unknown ApiNumber %u.\n", DebugIO->ApiNumber);
//         while (1);
//     }
// }
//
// static
// void
// send_kd_state_manipulate(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData)
// {
//     switch (State->ApiNumber)
//     {
//     default:
//         KDDBGPRINT!("Unknown ApiNumber %u.\n", State->ApiNumber);
//         while (1);
//     }
// }
//
// KDSTATUS
// ContinueManipulateStateHandler(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext
// )
// {
//     /* Let's go on */
//     State->ApiNumber = DbgKdContinueApi;
//     State->ReturnStatus = STATUS_SUCCESS; /* ? */
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     if (MessageData)
//         MessageData->Length = 0;
//     *MessageLength = 0;
//     State->u.Continue.ContinueStatus = STATUS_SUCCESS;
//
//     /* We definitely are at the end of the send <-> receive loop, if any */
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = NULL;
//     /* We're not handling an exception anymore */
//     InException = FALSE;
//
//     return KdPacketReceived;
// }
//
// static
// BOOLEAN
// GetVersionSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//     PLIST_ENTRY DebuggerDataList;
//
//     /* Confirm that all went well */
//     if ((PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//             || (State->ApiNumber != DbgKdGetVersionApi)
//             || !NT_SUCCESS(State->ReturnStatus))
//     {
//         KDDBGPRINT!("Wrong packet received after asking for data.\n");
//         return FALSE;
//     }
//
//     /* Copy the relevant data */
//     RtlCopyMemory(&KdVersion, &State->u.GetVersion64, sizeof(KdVersion));
//     DebuggerDataList = *(PLIST_ENTRY*)&KdVersion.DebuggerDataList;
//     KdDebuggerDataBlock = CONTAINING_RECORD(DebuggerDataList->Flink, KDDEBUGGER_DATA64, Header.List);
//     ProcessListHead = *(PLIST_ENTRY*)&KdDebuggerDataBlock->PsActiveProcessHead;
//     ModuleListHead = *(PLIST_ENTRY*)&KdDebuggerDataBlock->PsLoadedModuleList;
//
//     /* Now we can get the context for the current state */
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = GetContextManipulateHandler;
//     return TRUE;
// }
//
// static
// KDSTATUS
// GetVersionManipulateStateHandler(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     /* Ask for the version data */
//     State->ApiNumber = DbgKdGetVersionApi;
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//
//     /* The next send call will serve this query */
//     KdpSendPacketHandler = Some(GetVersionSendHandler);
//     KdpManipulateStateHandler = NULL;
//
//     return KdPacketReceived;
// }
//
// static
// BOOLEAN
// FirstSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_ANY_WAIT_STATE_CHANGE* StateChange = (DBGKD_ANY_WAIT_STATE_CHANGE*)MessageHeader->Buffer;
//     PETHREAD Thread;
//
//     if (PacketType != PACKET_TYPE_KD_STATE_CHANGE64)
//     {
//         KDDBGPRINT!("First KD packet is not a state change!\n");
//         return FALSE;
//     }
//
//     KDDBGPRINT!("KDGDB: START!\n");
//
//     Thread = (PETHREAD)(ULONG_PTR)StateChange->Thread;
//
//     /* Set up the current state */
//     CurrentStateChange = *StateChange;
//     gdb_dbg_tid = handle_to_gdb_tid(PsGetThreadId(Thread));
// #if MONOPROCESS
//     gdb_dbg_pid = 0;
// #else
//     gdb_dbg_pid = handle_to_gdb_pid(PsGetThreadProcessId(Thread));
// #endif
//     /* This is the idle process. Save it! */
//     TheIdleThread = Thread;
//     TheIdleProcess = (PEPROCESS)Thread->Tcb.ApcState.Process;
//
//     KDDBGPRINT!("Pid Tid of the first message: %" PRIxPTR", %" PRIxPTR ".\n", gdb_dbg_pid, gdb_dbg_tid);
//
//     /* The next receive call will be asking for the version data */
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = Some(GetVersionManipulateStateHandler);
//     return TRUE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /******************************************************************************
//  * \name KdReceivePacket
//  * \brief Receive a packet from the KD port.
//  * \param [in] PacketType Describes the type of the packet to receive.
//  *        This can be one of the PACKET_TYPE_ constants.
//  * \param [out] MessageHeader Pointer to a STRING structure for the header.
//  * \param [out] MessageData Pointer to a STRING structure for the data.
//  * \return KdPacketReceived if successful, KdPacketTimedOut if the receive
//  *         timed out, KdPacketNeedsResend to signal that the last packet needs
//  *         to be sent again.
//  * \note If PacketType is PACKET_TYPE_KD_POLL_BREAKIN, the function doesn't
//  *       wait for any data, but returns KdPacketTimedOut instantly if no breakin
//  *       packet byte is received.
//  * \sa http://www.nynaeve.net/?p=169
//  */
// KDSTATUS
// NTAPI
// KdReceivePacket(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData,
//      PULONG DataLength,
//      PKD_CONTEXT KdContext)
// {
//     KDDBGPRINT!("KdReceivePacket --> ");
//
//     if (PacketType == PACKET_TYPE_KD_POLL_BREAKIN)
//     {
//         static BOOLEAN firstTime = TRUE;
//         KDDBGPRINT!("Polling break in.\n");
//         if (firstTime)
//         {
//             /* Force debug break on init */
//             firstTime = FALSE;
//             return KdPacketReceived;
//         }
//
//         return KdpPollBreakIn();
//     }
//
//     if (PacketType == PACKET_TYPE_KD_DEBUG_IO)
//     {
//         static BOOLEAN ignore = 0;
//         KDDBGPRINT!("Debug prompt.\n");
//         /* HACK ! Debug prompt asks for break or ignore. First break, then ignore. */
//         MessageData->Length = 1;
//         MessageData->Buffer[0] = ignore ? 'i' : 'b';
//         ignore = !ignore;
//         return KdPacketReceived;
//     }
//
//     if (PacketType == PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//
//         KDDBGPRINT!("State manipulation: ");
//
//         /* Maybe we are in a send<->receive loop that GDB doesn't need to know about */
//         if (KdpManipulateStateHandler != NULL)
//         {
//             KDDBGPRINT!("We have a manipulate state handler.\n");
//             return KdpManipulateStateHandler(State, MessageData, DataLength, KdContext);
//         }
//
//         /* Receive data from GDB  and interpret it */
//         KDDBGPRINT!("Receiving data from GDB.\n");
//         return gdb_receive_and_interpret_packet(State, MessageData, DataLength, KdContext);
//     }
//
//     /* What should we do ? */
//     while (1);
//     return KdPacketNeedsResend;
// }
//
// VOID
// NTAPI
// KdSendPacket(
//     IN ULONG PacketType,
//     IN PSTRING MessageHeader,
//     IN PSTRING MessageData,
//     IN OUT PKD_CONTEXT KdContext)
// {
//     /* Override if we have some debug print from KD. */
//     if (PacketType == PACKET_TYPE_KD_DEBUG_IO)
//     {
//         send_kd_debug_io((DBGKD_DEBUG_IO*)MessageHeader->Buffer, MessageData);
//         return;
//     }
//
//     /* Maybe we are in a send <-> receive loop that GDB doesn't need to know about */
//     if (KdpSendPacketHandler
//         && KdpSendPacketHandler(PacketType, MessageHeader, MessageData))
//     {
//         return;
//     }
//
//     switch (PacketType)
//     {
//     case PACKET_TYPE_KD_STATE_CHANGE64:
//         send_kd_state_change((DBGKD_ANY_WAIT_STATE_CHANGE*)MessageHeader->Buffer);
//         return;
//     case PACKET_TYPE_KD_DEBUG_IO:
//         send_kd_debug_io((DBGKD_DEBUG_IO*)MessageHeader->Buffer, MessageData);
//         break;
//     case PACKET_TYPE_KD_STATE_MANIPULATE:
//         send_kd_state_manipulate((DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer, MessageData);
//         break;
//     default:
//         KDDBGPRINT!("Unknown packet type %u.\n", PacketType);
//         while (1);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
