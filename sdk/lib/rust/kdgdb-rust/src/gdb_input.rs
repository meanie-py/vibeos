use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
// Use bindgen process/thread types (full layout), NOT the opaque ps:: stubs.
use ntkrnl_sys::ddk::{CONTEXT, DBGKD_MANIPULATE_STATE64, DBGKM_EXCEPTION64, EPROCESS, HANDLE, _EPROCESS, _ETHREAD, KDSTATUS, LDR_DATA_TABLE_ENTRY, LIST_ENTRY, PEPROCESS, PETHREAD, PKD_CONTEXT, PKPROCESS, PLDR_DATA_TABLE_ENTRY, PSTRING, PsGetCurrentProcessId, PsGetCurrentThreadId, PsGetThreadId, PsGetThreadProcessId, STRING, UINT_PTR};
use core::ffi::c_void;

// PsGetCurrentProcess — declared returning the full bindgen PEPROCESS (the
// ps:: stub is opaque and lacks .Pcb). Resolves to ntoskrnl at final link.
extern "system" {
    fn PsGetCurrentProcess() -> PEPROCESS;
}

type PCSTR = *const core::ffi::c_char;

// kdgdb.h arch macros (_M_IX86 / _M_AMD64 are identical here).
// EFLAGS single-step bit (ndk i386 ke.h: EFLAGS_TF == 0x100).
const EFLAGS_TF: ULONG = 0x100;
// KD_BREAKPOINT_VALUE: int3 (0xCC). KD_BREAKPOINT_SIZE: sizeof(UCHAR).
const KD_BREAKPOINT_VALUE: u8 = 0xCC;
const KD_BREAKPOINT_SIZE: usize = 1;

#[inline]
unsafe fn KdpGetContextPc(Context: *const CONTEXT) -> usize {
    #[cfg(target_arch = "x86")]
    { (*Context).Eip as usize }
    #[cfg(target_arch = "x86_64")]
    { (*Context).Rip as usize }
}

#[inline]
unsafe fn KdpSetContextPc(Context: *mut CONTEXT, ProgramCounter: usize) {
    #[cfg(target_arch = "x86")]
    { (*Context).Eip = ProgramCounter as _; }
    #[cfg(target_arch = "x86_64")]
    { (*Context).Rip = ProgramCounter as _; }
}

// cr3 write — switch the active page directory (process address space).
#[inline]
unsafe fn __writecr3(value: usize) {
    core::arch::asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags));
}

// kdgdb's own globals / functions live in sibling modules.
use crate::KDDBGPRINT;
use crate::kdgdb_h::{gdb_pid_to_handle, gdb_tid_to_handle, handle_to_gdb_pid, handle_to_gdb_tid};
use crate::gdb_receive::{gdb_input, gdb_receive_packet, hex_value};
use crate::gdb_send::{
    finish_gdb_packet, gdb_send_debug_io, gdb_send_exception, send_gdb_memory,
    send_gdb_ntstatus, send_gdb_packet, send_gdb_partial_binary, send_gdb_partial_packet,
    start_gdb_packet,
};
use crate::kdpacket::{
    ContinueManipulateStateHandler, CurrentContext, CurrentStateChange,
    KdpManipulateStateHandler, KdpSendPacketHandler, ModuleListHead, ProcessListHead,
    SetContextManipulateHandler,
};
use crate::utils::{find_process, find_thread};

// Disambiguate consts that live in both globals + constants.
use ntkrnl_sys::constants::{PACKET_TYPE_KD_STATE_MANIPULATE, DbgKdExceptionStateChange};

// arch_sup.c exports (i386_sup.rs / amd64_sup.rs).
#[cfg(target_arch = "x86")]
use crate::i386_sup::{gdb_send_register, gdb_send_registers};
#[cfg(target_arch = "x86_64")]
use crate::amd64_sup::{gdb_send_register, gdb_send_registers};

// CRT (link from libcntpr at final link).
extern "C" {
    fn _snprintf(buffer: *mut i8, count: usize, format: *const i8, ...) -> i32;
    fn sprintf(buffer: *mut i8, format: *const i8, ...) -> i32;
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
    fn strncmp(s1: *const i8, s2: *const i8, n: usize) -> i32;
    fn strstr(haystack: *const i8, needle: *const i8) -> *mut i8;
}

// KdpSetSingleStep is the kdgdb.h macro `(Context)->EFlags |= EFLAGS_TF`.
#[inline]
unsafe fn KdpSetSingleStep(Context: *mut CONTEXT) {
    (*Context).EFlags |= EFLAGS_TF;
}

// ---- file-scope preamble (translated) ----
static mut gdb_run_tid: ULONG_PTR = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AnonymousStruct {
    pub Address: ULONG_PTR,
    pub Handle: ULONG,
}

static mut BreakPointHandles: [AnonymousStruct; 32] = [AnonymousStruct { Address: 0, Handle: 0 }; 32];

pub static mut gdb_dbg_pid: ULONG_PTR = 0;
pub static mut gdb_dbg_tid: ULONG_PTR = 0;
unsafe fn LOOP_IF_SUCCESS(x: KDSTATUS) -> KDSTATUS {
    if x == KdPacketReceived {
        (-1i32) as KDSTATUS
    } else {
        x
    }
}

unsafe fn hex_to_tid(buffer: *mut i8) -> ULONG_PTR {
    let mut ret: ULONG_PTR = 0;
    let mut hex: i8;
    let mut buf_ptr = buffer;
    while *buf_ptr != 0 {
        hex = hex_value(*buf_ptr);
        buf_ptr = buf_ptr.offset(1);
        if hex < 0 {
            return ret;
        }
        ret <<= 4;
        ret += hex as ULONG_PTR;
    }
    return ret;
}

/* #define hex_to_pid hex_to_tid */
#[inline]
unsafe fn hex_to_pid(buffer: *mut i8) -> ULONG_PTR {
    hex_to_tid(buffer)
}

unsafe fn hex_to_address(mut buffer: *mut i8) -> u64 {
    let mut ret: u64 = 0;
    let mut hex: i8;
    while *buffer != 0 {
        hex = hex_value(*buffer);
        buffer = buffer.offset(1);
        if hex < 0 {
            return ret;
        }
        ret <<= 4;
        ret += hex as u64;
    }
    return ret;
}

unsafe fn handle_gdb_set_thread() -> KDSTATUS {
    let Status: KDSTATUS;

    match (*gdb_input.as_mut_ptr().offset(1)) as u8 {
        b'c' => {
            if strcmp(gdb_input.as_mut_ptr().offset(2), b"-1\0".as_ptr() as *const i8) == 0 {
                gdb_run_tid = (-1i32) as ULONG_PTR;
            } else {
                gdb_run_tid = hex_to_tid(gdb_input.as_mut_ptr().offset(2));
            }
            Status = send_gdb_packet(b"OK\0".as_ptr() as *const i8);
        }
        b'g' => {
            KDDBGPRINT!(b"Setting debug thread: %s.\n\0".as_ptr() as *const i8, gdb_input);
            #[cfg(feature = "MONOPROCESS")]
            {
                gdb_dbg_pid = 0;
                if strncmp(gdb_input.as_mut_ptr().offset(2), b"-1\0".as_ptr() as *const i8, 2) == 0 {
                    gdb_dbg_tid = (-1i32) as ULONG_PTR;
                } else {
                    gdb_dbg_tid = hex_to_tid(gdb_input.as_mut_ptr().offset(2));
                }
            }
            #[cfg(not(feature = "MONOPROCESS"))]
            {
                if strncmp(gdb_input.as_mut_ptr().offset(2), b"p-1\0".as_ptr() as *const i8, 3) == 0 {
                    gdb_dbg_pid = (-1i32) as ULONG_PTR;
                    gdb_dbg_tid = (-1i32) as ULONG_PTR;
                } else {
                    let ptr = strstr(gdb_input.as_mut_ptr(), b".\0".as_ptr() as *const i8).offset(1);
                    gdb_dbg_pid = hex_to_pid(gdb_input.as_mut_ptr().offset(3));
                    if strncmp(ptr, b"-1\0".as_ptr() as *const i8, 2) == 0 {
                        gdb_dbg_tid = (-1i32) as ULONG_PTR;
                    } else {
                        gdb_dbg_tid = hex_to_tid(ptr);
                    }
                }
            }
            Status = send_gdb_packet(b"OK\0".as_ptr() as *const i8);
        }
        _ => {
            KDDBGPRINT!(b"KDGBD: Unknown 'H' command: %s\n\0".as_ptr() as *const i8, gdb_input);
            Status = send_gdb_packet(b"\0".as_ptr() as *const i8);
        }
    }

    return Status;
}

unsafe fn handle_gdb_thread_alive() -> KDSTATUS {
    let Pid: ULONG_PTR;
    let Tid: ULONG_PTR;
    let Thread: PETHREAD;
    let Status: KDSTATUS;

    #[cfg(feature = "MONOPROCESS")]
    {
        Pid = 0;
        Tid = hex_to_tid(gdb_input.as_mut_ptr().offset(1));

        KDDBGPRINT!(b"Checking if %p is alive.\n\0".as_ptr() as *const i8, Tid);
    }

    #[cfg(not(feature = "MONOPROCESS"))]
    {
        Pid = hex_to_pid(gdb_input.as_mut_ptr().offset(2));
        Tid = hex_to_tid(strstr(gdb_input.as_mut_ptr(), b".\0".as_ptr() as *const i8).offset(1));

        KDDBGPRINT!(b"Checking if p%p.%p is alive.\n\0".as_ptr() as *const i8, Pid, Tid);
    }

    Thread = find_thread(Pid, Tid);

    if Thread != core::ptr::null_mut() {
        Status = send_gdb_packet(b"OK\0".as_ptr() as *const i8);
    } else {
        Status = send_gdb_packet(b"E03\0".as_ptr() as *const i8);
    }

    return Status;
}

unsafe fn handle_gdb_query() -> KDSTATUS {

    if strncmp(gdb_input.as_mut_ptr(), b"qSupported:\0".as_ptr() as *const i8, 11) == 0 {
        return send_gdb_packet(b"PacketSize=1000;multiprocess+;qXfer:libraries:read+;\0".as_ptr() as *const i8);
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qAttached\0".as_ptr() as *const i8, 9) == 0 {
        let queried_pid = hex_to_pid(gdb_input.as_mut_ptr().offset(10));
        if gdb_pid_to_handle(queried_pid) == core::ptr::null_mut() {
            return send_gdb_packet(b"0\0".as_ptr() as *const i8);
        } else {
            return send_gdb_packet(b"1\0".as_ptr() as *const i8);
        }
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qRcmd,\0".as_ptr() as *const i8, 6) == 0 {
        return send_gdb_packet(b"OK\0".as_ptr() as *const i8);
    }

    if strcmp(gdb_input.as_mut_ptr(), b"qC\0".as_ptr() as *const i8) == 0 {
        let mut gdb_out: [i8; 64] = [0; 64];
        sprintf(
            gdb_out.as_mut_ptr(),
            b"QC:p%lx.%lx;\0".as_ptr() as *const i8,
            handle_to_gdb_pid(PsGetThreadProcessId((&raw const CurrentStateChange as *const _ as ULONG_PTR) as PETHREAD)),
            handle_to_gdb_tid(PsGetThreadId((&raw const CurrentStateChange as *const _ as ULONG_PTR) as PETHREAD))
        );
        return send_gdb_packet(gdb_out.as_ptr());
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qfThreadInfo\0".as_ptr() as *const i8, 12) == 0 {
        let mut gdb_out: [i8; 40] = [0; 40];
        let CurrentProcessEntry = (*ProcessListHead).Flink;
        if CurrentProcessEntry == core::ptr::null_mut() {
            return send_gdb_packet(b"mp1.1\0".as_ptr() as *const i8);
        }

        start_gdb_packet();
        send_gdb_partial_packet(b"mp1.1\0".as_ptr() as *const i8);

        let mut CurrentEntry = CurrentProcessEntry;
        while CurrentEntry != ProcessListHead {
            let Process = (CurrentEntry as *mut u8).offset(-(core::mem::offset_of!(_EPROCESS, ActiveProcessLinks) as isize)) as PEPROCESS;
            let mut CurrentThreadEntry = (*Process).ThreadListHead.Flink;

            while CurrentThreadEntry != core::ptr::addr_of_mut!((*Process).ThreadListHead) {
                let Thread = (CurrentThreadEntry as *mut u8).offset(-(core::mem::offset_of!(_ETHREAD, ThreadListEntry) as isize)) as PETHREAD;
                _snprintf(
                    gdb_out.as_mut_ptr(),
                    40,
                    b",p%p.%p\0".as_ptr() as *const i8,
                    handle_to_gdb_pid((*Process).UniqueProcessId),
                    handle_to_gdb_tid((*Thread).Cid.UniqueThread)
                );
                send_gdb_partial_packet(gdb_out.as_ptr());
                CurrentThreadEntry = (*CurrentThreadEntry).Flink;
            }
            CurrentEntry = (*CurrentEntry).Flink;
        }

        return finish_gdb_packet();
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qsThreadInfo\0".as_ptr() as *const i8, 12) == 0 {
        return send_gdb_packet(b"l\0".as_ptr() as *const i8);
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qGetTIBAddr:\0".as_ptr() as *const i8, 12) == 0 {
        let Pid = hex_to_pid(gdb_input.as_mut_ptr().offset(13));
        let Tid = hex_to_tid(strstr(gdb_input.as_mut_ptr().offset(13), b".\0".as_ptr() as *const i8).offset(1));
        KDDBGPRINT!(b" p%p.%p.\n\0".as_ptr() as *const i8, Pid, Tid);
        let Thread = find_thread(Pid, Tid);
        return send_gdb_memory(core::ptr::addr_of!((*Thread).Tcb.Teb) as *const core::ffi::c_void, core::mem::size_of_val(&(*Thread).Tcb.Teb));
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qThreadExtraInfo,\0".as_ptr() as *const i8, 17) == 0 {
        let mut out_string: [i8; 64] = [0; 64];
        let mut String = STRING {
            Length: 0,
            MaximumLength: 64,
            Buffer: out_string.as_mut_ptr(),
        };

        KDDBGPRINT!(b"Giving extra info for\0".as_ptr() as *const i8);

        let Pid = hex_to_pid(gdb_input.as_mut_ptr().offset(18));
        let Tid = hex_to_tid(strstr(gdb_input.as_mut_ptr().offset(18), b".\0".as_ptr() as *const i8).offset(1));
        KDDBGPRINT!(b" p%p.%p.\n\0".as_ptr() as *const i8, Pid, Tid);

        let Process = find_process(Pid);
        let Thread = find_thread(Pid, Tid);

        if PsGetThreadProcessId(Thread) as ULONG_PTR == 0 {
            String.Length = sprintf(out_string.as_mut_ptr(), b"SYSTEM\0".as_ptr() as *const i8) as u16;
        } else {
            String.Length = sprintf(out_string.as_mut_ptr(), b"%.*s\0".as_ptr() as *const i8, 16, (*Process).ImageFileName.as_ptr()) as u16;
        }

        return gdb_send_debug_io(core::ptr::addr_of_mut!(String), 0u8);
    }

    if strcmp(gdb_input.as_mut_ptr(), b"qTStatus\0".as_ptr() as *const i8) == 0 {
        return send_gdb_packet(b"T0\0".as_ptr() as *const i8);
    }

    if strcmp(gdb_input.as_mut_ptr(), b"qSymbol::\0".as_ptr() as *const i8) == 0 {
        return send_gdb_packet(b"OK\0".as_ptr() as *const i8);
    }

    if strncmp(gdb_input.as_mut_ptr(), b"qXfer:libraries:read::\0".as_ptr() as *const i8, 22) == 0 {
        static mut CurrentEntry: *mut LIST_ENTRY = core::ptr::null_mut();
        static mut allDone: u8 = 0u8;
        let mut str_helper: [i8; 256] = [0; 256];
        let mut name_helper: [i8; 64] = [0; 64];
        let Offset = hex_to_address(gdb_input.as_mut_ptr().offset(22));
        let ToSend = hex_to_address(strstr(gdb_input.as_mut_ptr().offset(22), b",\0".as_ptr() as *const i8).offset(1));
        let mut Sent: u32 = 0;

        KDDBGPRINT!(b"KDGDB: qXfer:libraries:read !\n\0".as_ptr() as *const i8);

        start_gdb_packet();

        if allDone != 0 {
            send_gdb_partial_packet(b"l\0".as_ptr() as *const i8);
            allDone = 0u8;
            return finish_gdb_packet();
        }

        send_gdb_partial_packet(b"m\0".as_ptr() as *const i8);
        Sent += 1;

        if Offset == 0 {
            Sent += send_gdb_partial_binary(b"<?xml version=\"1.0\"?>\0".as_ptr() as *const core::ffi::c_void, 21);
            Sent += send_gdb_partial_binary(b"<library-list>\0".as_ptr() as *const core::ffi::c_void, 14);

            CurrentEntry = (*ModuleListHead).Flink;

            if CurrentEntry == core::ptr::null_mut() {
                Sent += send_gdb_partial_binary(b"</library-list>\0".as_ptr() as *const core::ffi::c_void, 15);
                allDone = 1u8;
                return finish_gdb_packet();
            }
        }

        while CurrentEntry != ModuleListHead {
            let TableEntry = (CurrentEntry as *mut u8).offset(-(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks) as isize)) as PLDR_DATA_TABLE_ENTRY;
            let DllBase = ((*TableEntry).DllBase as ULONG_PTR + 0x1000) as *mut core::ffi::c_void;
            let mem_length: i32;

            for i in 0..((*TableEntry).BaseDllName.Length as usize / core::mem::size_of::<u16>()) {
                name_helper[i] = *(*TableEntry).BaseDllName.Buffer.offset(i as isize) as i8;
                if name_helper[i] >= b'A' as i8 && name_helper[i] <= b'Z' as i8 {
                    name_helper[i] += (b'a' - b'A') as i8;
                }
            }
            name_helper[(*TableEntry).BaseDllName.Length as usize / core::mem::size_of::<u16>()] = 0;

            mem_length = _snprintf(
                str_helper.as_mut_ptr(),
                256,
                b"<library name=\"C:\\%s\"><segment address=\"0x%p\"/></library>\0".as_ptr() as *const i8,
                name_helper.as_ptr(),
                DllBase
            );

            if mem_length < 0 {
                KDDBGPRINT!(b"Failed to report %wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of!((*TableEntry).BaseDllName));
                CurrentEntry = (*CurrentEntry).Flink;
                continue;
            }

            if (Sent + mem_length as u32) > ToSend as u32 {
                return finish_gdb_packet();
            }

            Sent += send_gdb_partial_binary(str_helper.as_ptr() as *const core::ffi::c_void, mem_length as usize);
            CurrentEntry = (*CurrentEntry).Flink;
        }

        if (ToSend as u32 - Sent) > 15 {
            Sent += send_gdb_partial_binary(b"</library-list>\0".as_ptr() as *const core::ffi::c_void, 15);
            allDone = 1u8;
        }

        return finish_gdb_packet();
    }

    KDDBGPRINT!(b"KDGDB: Unknown query: %s\n\0".as_ptr() as *const i8, gdb_input);
    return send_gdb_packet(b"\0".as_ptr() as *const i8);
}

unsafe fn ReadMemorySendHandler(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    MessageData: PSTRING,
) -> BOOLEAN {
    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

    if PacketType != PACKET_TYPE_KD_STATE_MANIPULATE {
        KDDBGPRINT!(
            b"Wrong packet type (%lu) received after DbgKdReadVirtualMemoryApi request.\n\0".as_ptr() as PCHAR,
            PacketType,
        );
        return 0u8;
    }

    if (*State).ApiNumber != DbgKdReadVirtualMemoryApi {
        KDDBGPRINT!(
            b"Wrong API number (%lu) after DbgKdReadVirtualMemoryApi request.\n\0".as_ptr() as PCHAR,
            (*State).ApiNumber,
        );
        return 0u8;
    }

    if (*MessageData).Length == 0 && !NT_SUCCESS((*State).ReturnStatus) {
        send_gdb_ntstatus((*State).ReturnStatus);
    } else {
        send_gdb_memory((*MessageData).Buffer as *const core::ffi::c_void, (*MessageData).Length as usize);
    }
    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = None;

    #[cfg(feature = "MONOPROCESS")]
    {
        if gdb_dbg_tid != 0 {
            if !(*ProcessListHead).Flink.is_null() {
                __writecr3(((*PsGetCurrentProcess()).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }
    #[cfg(not(feature = "MONOPROCESS"))]
    {
        if (gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId() {
            if !(*ProcessListHead).Flink.is_null() {
                __writecr3(((*PsGetCurrentProcess()).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }

    1u8
}

unsafe fn handle_gdb_read_mem(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    _KdContext: PKD_CONTEXT,
) -> KDSTATUS {

    (*State).ApiNumber = DbgKdReadVirtualMemoryApi;
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    if !MessageData.is_null() {
        (*MessageData).Length = 0;
    }
    *MessageLength = 0;

    #[cfg(feature = "MONOPROCESS")]
    {
        if (gdb_dbg_tid != 0) && gdb_tid_to_handle(gdb_dbg_tid) != PsGetCurrentThreadId() {
            let AttachedThread = find_thread(0, gdb_dbg_tid);
            if AttachedThread.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }

            let AttachedProcess = (*AttachedThread).Tcb.Process;
            if AttachedProcess.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }
            __writecr3(((*AttachedProcess).DirectoryTableBase[0]) as usize);
        }
    }

    #[cfg(not(feature = "MONOPROCESS"))]
    {
        if (gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId() {
            let AttachedProcess = find_process(gdb_dbg_pid);
            if AttachedProcess.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }
            if !(*ProcessListHead).Flink.is_null() {
                __writecr3(((*AttachedProcess).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }

    (*State).u.ReadMemory.TargetBaseAddress = hex_to_address(gdb_input.as_mut_ptr().offset(1));
    (*State).u.ReadMemory.TransferCount = hex_to_address(strstr(gdb_input.as_ptr().offset(1), b",\0".as_ptr() as *const i8).offset(1)) as ULONG;

    KdpSendPacketHandler = Some(ReadMemorySendHandler);
    return KdPacketReceived;
}
unsafe fn WriteMemorySendHandler(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    _MessageData: PSTRING) -> BOOLEAN {
    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

    if PacketType != PACKET_TYPE_KD_STATE_MANIPULATE
    {
        KDDBGPRINT!(b"Wrong packet type (%lu) received after DbgKdWriteVirtualMemoryApi request.\n\0".as_ptr() as PCHAR, PacketType);
        return 0u8;
    }

    if (*State).ApiNumber != DbgKdWriteVirtualMemoryApi
    {
        KDDBGPRINT!(b"Wrong API number (%lu) after DbgKdWriteVirtualMemoryApi request.\n\0".as_ptr() as PCHAR, (*State).ApiNumber);
        return 0u8;
    }

    if !NT_SUCCESS((*State).ReturnStatus)
    {
        send_gdb_ntstatus((*State).ReturnStatus);
    }
    else
    {
        send_gdb_packet(b"OK\0".as_ptr() as PCHAR);
    }
    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = None;

    #[cfg(feature = "MONOPROCESS")]
    {
        if gdb_dbg_tid != 0
        {
            if !(*ProcessListHead).Flink.is_null()
            {
                __writecr3(((*PsGetCurrentProcess()).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }
    #[cfg(not(feature = "MONOPROCESS"))]
    {
        if (gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId()
        {
            if !(*ProcessListHead).Flink.is_null()
            {
                __writecr3(((*PsGetCurrentProcess()).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }
    1u8
}

unsafe fn handle_gdb_write_mem(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    _MessageLength: *mut u32,
    _KdContext: PKD_CONTEXT,
) -> KDSTATUS {
    static mut OutBuffer: [u8; 0x800] = [0; 0x800];
    let mut BufferLength: u32;
    let mut blob_ptr: *mut i8;
    let mut OutPtr: *mut u8;

    (*State).ApiNumber = DbgKdWriteVirtualMemoryApi;
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;

    #[cfg(feature = "MONOPROCESS")]
    {
        if (gdb_dbg_tid != 0) && gdb_tid_to_handle(gdb_dbg_tid) != PsGetCurrentThreadId() {
            let AttachedThread: PETHREAD = find_thread(0, gdb_dbg_tid);
            let AttachedProcess: PKPROCESS;
            if AttachedThread.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }

            AttachedProcess = (*AttachedThread).Tcb.Process;
            if AttachedProcess.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }
            __writecr3(((*AttachedProcess).DirectoryTableBase[0]) as usize);
        }
    }

    #[cfg(not(feature = "MONOPROCESS"))]
    {
        if (gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId() {
            let AttachedProcess: PEPROCESS = find_process(gdb_dbg_pid);
            if AttachedProcess.is_null() {
                KDDBGPRINT!(b"The current GDB debug thread is invalid!\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E03\0".as_ptr() as *const i8));
            }
            if !(*ProcessListHead).Flink.is_null() {
                __writecr3(((*AttachedProcess).Pcb.DirectoryTableBase[0]) as usize);
            }
        }
    }

    (*State).u.WriteMemory.TargetBaseAddress = hex_to_address(gdb_input.as_mut_ptr().offset(1));
    BufferLength = hex_to_address(strstr(gdb_input.as_mut_ptr().offset(1), b",\0".as_ptr() as *const i8).offset(1)) as u32;
    if BufferLength == 0 {
        return LOOP_IF_SUCCESS(send_gdb_packet(b"OK\0".as_ptr() as *const i8));
    }

    (*State).u.WriteMemory.TransferCount = BufferLength;
    (*MessageData).Length = BufferLength as u16;
    (*MessageData).Buffer = OutBuffer.as_mut_ptr() as *mut i8;

    OutPtr = OutBuffer.as_mut_ptr();
    blob_ptr = strstr(strstr(gdb_input.as_mut_ptr().offset(1), b",\0".as_ptr() as *const i8).offset(1), b":\0".as_ptr() as *const i8).offset(1);
    while BufferLength > 0 {
        if BufferLength >= 4 {
            *(OutPtr as *mut u32) = *(blob_ptr as *mut u32);
            OutPtr = OutPtr.offset(4);
            blob_ptr = blob_ptr.offset(4);
            BufferLength -= 4;
        } else if BufferLength >= 2 {
            *(OutPtr as *mut u16) = *(blob_ptr as *mut u16);
            OutPtr = OutPtr.offset(2);
            blob_ptr = blob_ptr.offset(2);
            BufferLength -= 2;
        } else {
            *OutPtr = *blob_ptr as u8;
            OutPtr = OutPtr.offset(1);
            blob_ptr = blob_ptr.offset(1);
            BufferLength -= 1;
        }
    }

    KdpSendPacketHandler = Some(WriteMemorySendHandler);
    return KdPacketReceived;
}

unsafe fn WriteBreakPointSendHandler(
    PacketType: u32,
    MessageHeader: PSTRING,
    _MessageData: PSTRING
) -> u8 {
    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

    if PacketType != PACKET_TYPE_KD_STATE_MANIPULATE {
        KDDBGPRINT!(b"Wrong packet type (%lu) received after DbgKdWriteBreakPointApi request.\n\0".as_ptr() as *const i8, PacketType);
        return 0u8;
    }

    if (*State).ApiNumber != DbgKdWriteBreakPointApi {
        KDDBGPRINT!(b"Wrong API number (%lu) after DbgKdWriteBreakPointApi request.\n\0".as_ptr() as *const i8, (*State).ApiNumber);
        return 0u8;
    }

    if !NT_SUCCESS((*State).ReturnStatus) {
        KDDBGPRINT!(b"Inserting breakpoint failed!\n\0".as_ptr() as *const i8);
        send_gdb_ntstatus((*State).ReturnStatus);
    } else {
        let mut i: u32 = 0;
        while i < 32 {
            if BreakPointHandles[i as usize].Address == 0 {
                BreakPointHandles[i as usize].Address = (*State).u.WriteBreakPoint.BreakPointAddress as usize;
                BreakPointHandles[i as usize].Handle = (*State).u.WriteBreakPoint.BreakPointHandle;
                break;
            }
            i += 1;
        }
        send_gdb_packet(b"OK\0".as_ptr() as *const i8);
    }
    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = None;
    1u8
}

unsafe fn handle_gdb_insert_breakpoint(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    _KdContext: PKD_CONTEXT,
) -> KDSTATUS {
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    if !MessageData.is_null() {
        (*MessageData).Length = 0;
    }
    *MessageLength = 0;

    match (*gdb_input.as_mut_ptr().offset(1)) as u8 {
        b'0' => {
            let Address = hex_to_address(gdb_input.as_mut_ptr().offset(3));
            let mut i: u32;
            let mut HasFreeSlot: u8 = 0u8;

            KDDBGPRINT!(
                b"Inserting breakpoint at %p.\n\0".as_ptr() as *const i8,
                Address as *mut core::ffi::c_void,
            );

            i = 0;
            while i < 32 {
                if BreakPointHandles[i as usize].Address == 0 {
                    HasFreeSlot = 1u8;
                }
                i += 1;
            }

            if HasFreeSlot == 0 {
                KDDBGPRINT!(b"No breakpoint slot available!\n\0".as_ptr() as *const i8);
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E01\0".as_ptr() as *const i8));
            }

            (*State).ApiNumber = DbgKdWriteBreakPointApi;
            (*State).u.WriteBreakPoint.BreakPointAddress = Address;

            KdpSendPacketHandler = Some(WriteBreakPointSendHandler);
            return KdPacketReceived;
        }
        _ => {}
    }

    KDDBGPRINT!(
        b"Unhandled 'Z' packet: %s\n\0".as_ptr() as *const i8,
        gdb_input,
    );
    return LOOP_IF_SUCCESS(send_gdb_packet(b"E01\0".as_ptr() as *const i8));
}

unsafe fn RestoreBreakPointSendHandler(
    PacketType: ULONG,
    MessageHeader: PSTRING,
    _MessageData: PSTRING,
) -> BOOLEAN {

    let State = (*MessageHeader).Buffer as *mut DBGKD_MANIPULATE_STATE64;

    if PacketType != PACKET_TYPE_KD_STATE_MANIPULATE {
        KDDBGPRINT!(
            b"Wrong packet type (%lu) received after DbgKdRestoreBreakPointApi request.\n\0".as_ptr() as PCSTR,
            PacketType,
        );
        return 0u8;
    }

    if (*State).ApiNumber != DbgKdRestoreBreakPointApi {
        KDDBGPRINT!(
            b"Wrong API number (%lu) after DbgKdRestoreBreakPointApi request.\n\0".as_ptr() as PCSTR,
            (*State).ApiNumber,
        );
        return 0u8;
    }

    for i in 0..(core::mem::size_of_val(&BreakPointHandles) / core::mem::size_of_val(&BreakPointHandles[0])) {
        if BreakPointHandles[i].Handle == (*State).u.RestoreBreakPoint.BreakPointHandle {
            BreakPointHandles[i].Address = 0;
            BreakPointHandles[i].Handle = 0;
            break;
        }
    }

    send_gdb_packet(b"OK\0".as_ptr() as PCSTR);

    KdpSendPacketHandler = None;
    KdpManipulateStateHandler = None;
    1u8
}

unsafe fn handle_gdb_remove_breakpoint(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    _KdContext: PKD_CONTEXT,
) -> KDSTATUS {
    (*State).ReturnStatus = STATUS_SUCCESS;
    (*State).Processor = CurrentStateChange.Processor;
    (*State).ProcessorLevel = CurrentStateChange.ProcessorLevel;
    if !MessageData.is_null() {
        (*MessageData).Length = 0;
    }
    *MessageLength = 0;

    match (*gdb_input.as_mut_ptr().offset(1)) as u8 {
        b'0' => {
            let Address = hex_to_address(gdb_input.as_mut_ptr().offset(3));
            let mut Handle: u32 = 0;

            KDDBGPRINT!(
                b"Removing breakpoint on %p.\n\0".as_ptr() as *const i8,
                Address as *mut core::ffi::c_void,
            );

            for i in 0..(core::mem::size_of_val(&BreakPointHandles) / core::mem::size_of::<(usize, u32)>()) {
                if BreakPointHandles[i].Address == Address as usize {
                    Handle = BreakPointHandles[i].Handle;
                    break;
                }
            }

            if Handle == 0 {
                KDDBGPRINT!(
                    b"Received %s, but breakpoint was never inserted ?!\n\0".as_ptr() as *const i8,
                    gdb_input as *const i8,
                );
                return LOOP_IF_SUCCESS(send_gdb_packet(b"E01\0".as_ptr() as *mut i8));
            }

            (*State).ApiNumber = DbgKdRestoreBreakPointApi;
            (*State).u.RestoreBreakPoint.BreakPointHandle = Handle;

            KdpSendPacketHandler = Some(RestoreBreakPointSendHandler);
            return KdPacketReceived;
        }
        _ => {}
    }

    KDDBGPRINT!(
        b"Unhandled 'Z' packet: %s\n\0".as_ptr() as *const i8,
        gdb_input as *const i8,
    );
    return LOOP_IF_SUCCESS(send_gdb_packet(b"E01\0".as_ptr() as *mut i8));
}

unsafe fn handle_gdb_c(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {
    let Status: KDSTATUS;

    Status = send_gdb_packet(b"OK\0".as_ptr() as *const i8);
    if Status != KdPacketReceived {
        return Status;
    }

    if CurrentStateChange.NewState == DbgKdExceptionStateChange {
        let Exception: *mut DBGKM_EXCEPTION64 = core::ptr::addr_of_mut!(CurrentStateChange.u.Exception);
        let ProgramCounter: usize = KdpGetContextPc(core::ptr::addr_of_mut!(CurrentContext));

        if ((*Exception).ExceptionRecord.ExceptionCode == STATUS_BREAKPOINT)
            && ((*(ProgramCounter as *const u8)) == KD_BREAKPOINT_VALUE)
        {
            KdpSetContextPc(core::ptr::addr_of_mut!(CurrentContext), ProgramCounter + KD_BREAKPOINT_SIZE);

            SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
            KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
            return KdPacketReceived;
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if ((*Exception).ExceptionRecord.ExceptionCode == STATUS_ASSERTION_FAILURE)
                && ((*(ProgramCounter as *const u8)) == 0xCD)
                && (*((ProgramCounter as *const u8).offset(1)) == 0x2C)
            {
                KdpSetContextPc(core::ptr::addr_of_mut!(CurrentContext), ProgramCounter + 2);

                SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
                KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
                return KdPacketReceived;
            }
        }
    }

    return ContinueManipulateStateHandler(State, MessageData, MessageLength, KdContext);
}

unsafe fn handle_gdb_s(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut ULONG,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {

    KDDBGPRINT!(b"Single stepping.\n\0".as_ptr() as *const i8);
    KdpSetSingleStep(core::ptr::addr_of_mut!(CurrentContext));
    SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
    KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
    KdPacketReceived
}

unsafe fn handle_gdb_v(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {

    if strncmp(gdb_input.as_mut_ptr(), b"vCont\0".as_ptr() as *const i8, 5) == 0 {
        if (*gdb_input.as_mut_ptr().offset(5)) as u8 == b'?' {
            return LOOP_IF_SUCCESS(send_gdb_packet(b"vCont;c;s\0".as_ptr() as *const i8));
        }

        if strncmp(gdb_input.as_mut_ptr(), b"vCont;c\0".as_ptr() as *const i8, 7) == 0 {
            return handle_gdb_c(State, MessageData, MessageLength, KdContext);
        }

        if strncmp(gdb_input.as_mut_ptr(), b"vCont;s\0".as_ptr() as *const i8, 7) == 0 {
            return handle_gdb_s(State, MessageData, MessageLength, KdContext);
        }
    }

    KDDBGPRINT!(b"Unhandled 'v' packet: %s\n\0".as_ptr() as *const i8, gdb_input);
    return LOOP_IF_SUCCESS(send_gdb_packet(b"\0".as_ptr() as *const i8));
}
#[no_mangle]
pub unsafe extern "C" fn gdb_receive_and_interpret_packet(
    State: *mut DBGKD_MANIPULATE_STATE64,
    MessageData: PSTRING,
    MessageLength: *mut u32,
    KdContext: PKD_CONTEXT,
) -> KDSTATUS {

    let mut Status: KDSTATUS;

    loop {
        KDDBGPRINT!(b"KDGBD: Receiving packet.\n\0".as_ptr() as *const i8);
        Status = gdb_receive_packet(KdContext);
        KDDBGPRINT!(
            b"KDGBD: Packet \"%s\" received with status %u\n\0".as_ptr() as *const i8,
            gdb_input.as_ptr(),
            Status as u32,
        );

        if Status != KdPacketReceived {
            return Status;
        }

        Status = (-1i32) as KDSTATUS;

        match (*gdb_input.as_ptr()) as u8 {
            b'?' => {
                Status = LOOP_IF_SUCCESS(gdb_send_exception());
            }
            b'!' => {
                Status = LOOP_IF_SUCCESS(send_gdb_packet(b"OK\0".as_ptr() as *const i8));
            }
            b'c' | b'C' => {
                Status = handle_gdb_c(State, MessageData, MessageLength, KdContext);
            }
            b'g' => {
                Status = LOOP_IF_SUCCESS(gdb_send_registers());
            }
            b'H' => {
                Status = LOOP_IF_SUCCESS(handle_gdb_set_thread());
            }
            b'm' => {
                Status = handle_gdb_read_mem(State, MessageData, MessageLength, KdContext);
            }
            b'p' => {
                Status = LOOP_IF_SUCCESS(gdb_send_register());
            }
            b'q' => {
                Status = LOOP_IF_SUCCESS(handle_gdb_query());
            }
            b's' => {
                Status = handle_gdb_s(State, MessageData, MessageLength, KdContext);
            }
            b'T' => {
                Status = LOOP_IF_SUCCESS(handle_gdb_thread_alive());
            }
            b'v' => {
                Status = handle_gdb_v(State, MessageData, MessageLength, KdContext);
            }
            b'X' => {
                Status = handle_gdb_write_mem(State, MessageData, MessageLength, KdContext);
            }
            b'z' => {
                Status = handle_gdb_remove_breakpoint(State, MessageData, MessageLength, KdContext);
            }
            b'Z' => {
                Status = handle_gdb_insert_breakpoint(State, MessageData, MessageLength, KdContext);
            }
            _ => {
                KDDBGPRINT!(
                    b"Unsupported GDB command: %s.\n\0".as_ptr() as *const i8,
                    gdb_input.as_ptr(),
                );
                Status = LOOP_IF_SUCCESS(send_gdb_packet(b"\0".as_ptr() as *const i8));
            }
        }

        if Status != (-1i32) as KDSTATUS {
            break;
        }
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       GPL, see COPYING in the top level directory
//  * PROJECT:         ReactOS kernel
//  * FILE:            drivers/base/kddll/gdb_input.c
//  * PURPOSE:         Base functions for the kernel debugger.
//  */
//
// #include "kdgdb.h"
//
// /* LOCALS *********************************************************************/
// static ULONG_PTR gdb_run_tid;
// static struct
// {
//     ULONG_PTR Address;
//     ULONG Handle;
// } BreakPointHandles[32];
//
//
// /* GLOBALS ********************************************************************/
// UINT_PTR gdb_dbg_pid;
// UINT_PTR gdb_dbg_tid;
//
// static inline
// KDSTATUS
// LOOP_IF_SUCCESS(int x)
// {
//     return (x == KdPacketReceived) ? (KDSTATUS)-1 : x;
// }
//
// /* PRIVATE FUNCTIONS **********************************************************/
// static
// UINT_PTR
// hex_to_tid(char* buffer)
// {
//     ULONG_PTR ret = 0;
//     char hex;
//     while (*buffer)
//     {
//         hex = hex_value(*buffer++);
//         if (hex < 0)
//             return ret;
//         ret <<= 4;
//         ret += hex;
//     }
//     return ret;
// }
// #define hex_to_pid hex_to_tid
//
// static
// ULONG64
// hex_to_address(char* buffer)
// {
//     ULONG64 ret = 0;
//     char hex;
//     while (*buffer)
//     {
//         hex = hex_value(*buffer++);
//         if (hex < 0)
//             return ret;
//         ret <<= 4;
//         ret += hex;
//     }
//     return ret;
// }
//
// /* H* packets */
// static
// KDSTATUS
// handle_gdb_set_thread(void)
// {
//     KDSTATUS Status;
//
//     switch (gdb_input[1])
//     {
//     case 'c':
//         if (strcmp(&gdb_input[2], "-1") == 0)
//             gdb_run_tid = (ULONG_PTR)-1;
//         else
//             gdb_run_tid = hex_to_tid(&gdb_input[2]);
//         Status = send_gdb_packet("OK");
//         break;
//     case 'g':
//         KDDBGPRINT!("Setting debug thread: %s.\n", gdb_input);
// #if MONOPROCESS
//         gdb_dbg_pid = 0;
//         if (strncmp(&gdb_input[2], "-1", 2) == 0)
//         {
//             gdb_dbg_tid = (UINT_PTR)-1;
//         }
//         else
//         {
//             gdb_dbg_tid = hex_to_tid(&gdb_input[2]);
//         }
// #else
//         if (strncmp(&gdb_input[2], "p-1", 3) == 0)
//         {
//             gdb_dbg_pid = (UINT_PTR)-1;
//             gdb_dbg_tid = (UINT_PTR)-1;
//         }
//         else
//         {
//             char* ptr = strstr(gdb_input.as_mut_ptr(), ".") + 1;
//             gdb_dbg_pid = hex_to_pid(&gdb_input[3]);
//             if (strncmp(ptr, "-1", 2) == 0)
//                 gdb_dbg_tid = (UINT_PTR)-1;
//             else
//                 gdb_dbg_tid = hex_to_tid(ptr);
//         }
// #endif
//         Status = send_gdb_packet("OK");
//         break;
//     default:
//         KDDBGPRINT!("KDGBD: Unknown 'H' command: %s\n", gdb_input);
//         Status = send_gdb_packet("");
//     }
//
//     return Status;
// }
//
// static
// KDSTATUS
// handle_gdb_thread_alive(void)
// {
//     ULONG_PTR Pid, Tid;
//     PETHREAD Thread;
//     KDSTATUS Status;
//
// #if MONOPROCESS
//     Pid = 0;
//     Tid = hex_to_tid(&gdb_input[1]);
//
//     KDDBGPRINT!("Checking if %p is alive.\n", Tid);
//
// #else
//     Pid = hex_to_pid(&gdb_input[2]);
//     Tid = hex_to_tid(strstr(gdb_input.as_mut_ptr(), ".") + 1);
//
//     /* We cannot use PsLookupProcessThreadByCid as we could be running at any IRQL.
//      * So loop. */
//     KDDBGPRINT!("Checking if p%p.%p is alive.\n", Pid, Tid);
// #endif
//
//     Thread = find_thread(Pid, Tid);
//
//     if (Thread != NULL)
//         Status = send_gdb_packet("OK");
//     else
//         Status = send_gdb_packet("E03");
//
//     return Status;
// }
//
// /* q* packets */
// static
// KDSTATUS
// handle_gdb_query(void)
// {
//     if (strncmp(gdb_input.as_mut_ptr(), "qSupported:", 11) == 0)
//     {
// #if MONOPROCESS
//         return send_gdb_packet("PacketSize=1000;qXfer:libraries:read+;");
// #else
//         return send_gdb_packet("PacketSize=1000;multiprocess+;qXfer:libraries:read+;");
// #endif
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qAttached", 9) == 0)
//     {
// #if MONOPROCESS
//         return send_gdb_packet("1");
// #else
//         UINT_PTR queried_pid = hex_to_pid(&gdb_input[10]);
//         /* Let's say we created system process */
//         if (gdb_pid_to_handle(queried_pid) == NULL)
//             return send_gdb_packet("0");
//         else
//             return send_gdb_packet("1");
// #endif
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qRcmd,", 6) == 0)
//     {
//         return send_gdb_packet("OK");
//     }
//
//     if (strcmp(gdb_input.as_mut_ptr(), "qC") == 0)
//     {
//         char gdb_out[64];
// #if MONOPROCESS
//         sprintf(gdb_out, "QC:%"PRIxPTR";",
//             handle_to_gdb_tid(PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread)));
// #else
//         sprintf(gdb_out, "QC:p%"PRIxPTR".%"PRIxPTR";",
//             handle_to_gdb_pid(PsGetThreadProcessId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread)),
//             handle_to_gdb_tid(PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread)));
// #endif
//         return send_gdb_packet(gdb_out);
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qfThreadInfo", 12) == 0)
//     {
//         PEPROCESS Process;
//         char gdb_out[40];
//         LIST_ENTRY* CurrentProcessEntry;
//
//         CurrentProcessEntry = ProcessListHead->Flink;
//         if (CurrentProcessEntry == NULL) /* Ps is not initialized */
//         {
// #if MONOPROCESS
//             return send_gdb_packet("m1");
// #else
//             return send_gdb_packet("mp1.1");
// #endif
//         }
//
//         /* We will push threads as we find them */
//         start_gdb_packet();
//
//         /* Start with the system thread */
// #if MONOPROCESS
//         send_gdb_partial_packet("m1");
// #else
//         send_gdb_partial_packet("mp1.1");
// #endif
//
//         /* List all the processes */
//         for ( ;
//             CurrentProcessEntry != ProcessListHead;
//             CurrentProcessEntry = CurrentProcessEntry->Flink)
//         {
//             LIST_ENTRY* CurrentThreadEntry;
//
//             Process = CONTAINING_RECORD(CurrentProcessEntry, EPROCESS, ActiveProcessLinks);
//
//             /* List threads from this process */
//             for ( CurrentThreadEntry = Process->ThreadListHead.Flink;
//                  CurrentThreadEntry != &Process->ThreadListHead;
//                  CurrentThreadEntry = CurrentThreadEntry->Flink)
//             {
//                 PETHREAD Thread = CONTAINING_RECORD(CurrentThreadEntry, ETHREAD, ThreadListEntry);
//
// #if MONOPROCESS
//                 _snprintf(gdb_out, 40, ",%p", handle_to_gdb_tid(Thread->Cid.UniqueThread));
// #else
//                 _snprintf(gdb_out, 40, ",p%p.%p",
//                     handle_to_gdb_pid(Process->UniqueProcessId),
//                     handle_to_gdb_tid(Thread->Cid.UniqueThread));
// #endif
//                 send_gdb_partial_packet(gdb_out);
//             }
//         }
//
//         return finish_gdb_packet();
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qsThreadInfo", 12) == 0)
//     {
//         /* We sent the whole thread list on first qfThreadInfo call */
//         return send_gdb_packet("l");
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qGetTIBAddr:", 12) == 0)
//     {
//         ULONG_PTR Pid, Tid;
//         PETHREAD Thread;
//
// #if MONOPROCESS
//         Pid = 0;
//         Tid = hex_to_tid(&gdb_input[12]);
//
//         KDDBGPRINT!(" %p.\n", Tid);
//
//         Thread = find_thread(Pid, Tid);
// #else
//         Pid = hex_to_pid(&gdb_input[13]);
//         Tid = hex_to_tid(strstr(&gdb_input[13], ".") + 1);
//
//         /* We cannot use PsLookupProcessThreadByCid as we could be running at any IRQL.
//          * So loop. */
//         KDDBGPRINT!(" p%p.%p.\n", Pid, Tid);
//         Thread = find_thread(Pid, Tid);
// #endif
//         return send_gdb_memory(&Thread->Tcb.Teb, sizeof(Thread->Tcb.Teb));
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qThreadExtraInfo,", 17) == 0)
//     {
//         ULONG_PTR Pid, Tid;
//         PETHREAD Thread;
//         PEPROCESS Process;
//         char out_string[64];
//         STRING String = {0, 64, out_string};
//
//         KDDBGPRINT!("Giving extra info for");
//
// #if MONOPROCESS
//         Pid = 0;
//         Tid = hex_to_tid(&gdb_input[17]);
//
//         KDDBGPRINT!(" %p.\n", Tid);
//
//         Thread = find_thread(Pid, Tid);
//         Process = CONTAINING_RECORD(Thread->Tcb.Process, EPROCESS, Pcb);
// #else
//         Pid = hex_to_pid(&gdb_input[18]);
//         Tid = hex_to_tid(strstr(&gdb_input[18], ".") + 1);
//
//         /* We cannot use PsLookupProcessThreadByCid as we could be running at any IRQL.
//          * So loop. */
//         KDDBGPRINT!(" p%p.%p.\n", Pid, Tid);
//
//         Process = find_process(Pid);
//         Thread = find_thread(Pid, Tid);
// #endif
//
//         if (PsGetThreadProcessId(Thread) == 0)
//         {
//             String.Length = sprintf(out_string, "SYSTEM");
//         }
//         else
//         {
//             String.Length = sprintf(out_string, "%.*s", 16, Process->ImageFileName);
//         }
//
//         return gdb_send_debug_io(&String, FALSE);
//     }
//
//     if (strcmp(gdb_input.as_mut_ptr(), "qTStatus") == 0)
//     {
//         /* No tracepoint support */
//         return send_gdb_packet("T0");
//     }
//
//     if (strcmp(gdb_input.as_mut_ptr(), "qSymbol::") == 0)
//     {
//         /* No need */
//         return send_gdb_packet("OK");
//     }
//
//     if (strncmp(gdb_input.as_mut_ptr(), "qXfer:libraries:read::", 22) == 0)
//     {
//         static LIST_ENTRY* CurrentEntry = NULL;
//         char str_helper[256];
//         char name_helper[64];
//         ULONG_PTR Offset = hex_to_address(&gdb_input[22]);
//         ULONG_PTR ToSend = hex_to_address(strstr(&gdb_input[22], ",") + 1);
//         ULONG Sent = 0;
//         static BOOLEAN allDone = FALSE;
//
//         KDDBGPRINT!("KDGDB: qXfer:libraries:read !\n");
//
//         /* Start the packet */
//         start_gdb_packet();
//
//         if (allDone)
//         {
//             send_gdb_partial_packet("l");
//             allDone = FALSE;
//             return finish_gdb_packet();
//         }
//
//         send_gdb_partial_packet("m");
//         Sent++;
//
//         /* Are we starting ? */
//         if (Offset == 0)
//         {
//             Sent += send_gdb_partial_binary("<?xml version=\"1.0\"?>", 21);
//             Sent += send_gdb_partial_binary("<library-list>", 14);
//
//             CurrentEntry = ModuleListHead->Flink;
//
//             if (!CurrentEntry)
//             {
//                 /* Ps is not initialized. Send end of XML data or mark that we are finished. */
//                 Sent += send_gdb_partial_binary("</library-list>", 15);
//                 allDone = TRUE;
//                 return finish_gdb_packet();
//             }
//         }
//
//         for ( ;
//             CurrentEntry != ModuleListHead;
//             CurrentEntry = CurrentEntry->Flink)
//         {
//             PLDR_DATA_TABLE_ENTRY TableEntry = CONTAINING_RECORD(CurrentEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//             PVOID DllBase = (PVOID)((ULONG_PTR)TableEntry->DllBase + 0x1000);
//             LONG mem_length;
//             USHORT i;
//
//             /* Convert names to lower case. Yes this _is_ ugly */
//             for (i = 0; i < (TableEntry->BaseDllName.Length / sizeof(WCHAR)); i++)
//             {
//                 name_helper[i] = (char)TableEntry->BaseDllName.Buffer[i];
//                 if (name_helper[i] >= 'A' && name_helper[i] <= 'Z')
//                     name_helper[i] += 'a' - 'A';
//             }
//             name_helper[i] = 0;
//
//             /* GDB doesn't load the file if you don't prefix it with a drive letter... */
//             mem_length = _snprintf(str_helper, 256, "<library name=\"C:\\%s\"><segment address=\"0x%p\"/></library>", &name_helper, DllBase);
//
//             /* DLL name must be too long. */
//             if (mem_length < 0)
//             {
//                 KDDBGPRINT!("Failed to report %wZ\n", &TableEntry->BaseDllName);
//                 continue;
//             }
//
//             if ((Sent + mem_length) > ToSend)
//             {
//                 /* We're done for this pass */
//                 return finish_gdb_packet();
//             }
//
//             Sent += send_gdb_partial_binary(str_helper, mem_length);
//         }
//
//         if ((ToSend - Sent) > 15)
//         {
//             Sent += send_gdb_partial_binary("</library-list>", 15);
//             allDone = TRUE;
//         }
//
//         return finish_gdb_packet();
//     }
//
//     KDDBGPRINT!("KDGDB: Unknown query: %s\n", gdb_input);
//     return send_gdb_packet("");
// }
//
//
// static
// BOOLEAN
// ReadMemorySendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//
//     if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         // KdAssert
//         KDDBGPRINT!("Wrong packet type (%lu) received after DbgKdReadVirtualMemoryApi request.\n", PacketType);
//         return FALSE;
//     }
//
//     if (State->ApiNumber != DbgKdReadVirtualMemoryApi)
//     {
//         KDDBGPRINT!("Wrong API number (%lu) after DbgKdReadVirtualMemoryApi request.\n", State->ApiNumber);
//         return FALSE;
//     }
//
//     /* Check status. Allow to send partial data. */
//     if (!MessageData->Length && !NT_SUCCESS(State->ReturnStatus))
//         send_gdb_ntstatus(State->ReturnStatus);
//     else
//         send_gdb_memory(MessageData->Buffer, MessageData->Length);
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = NULL;
//
// #if MONOPROCESS
//     if (gdb_dbg_tid != 0)
//     /* Reset the TLB */
// #else
//     if ((gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId())
// #endif
//     {
//         /* Only do this if Ps is initialized */
//         if (ProcessListHead->Flink)
//             __writecr3((PsGetCurrentProcess()->Pcb.DirectoryTableBase[0]) as usize);
//     }
//
//     return TRUE;
// }
//
// static
// KDSTATUS
// handle_gdb_read_mem(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     State->ApiNumber = DbgKdReadVirtualMemoryApi;
//     State->ReturnStatus = STATUS_SUCCESS; /* ? */
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     if (MessageData)
//         MessageData->Length = 0;
//     *MessageLength = 0;
//
//     /* Set the TLB according to the process being read. Pid 0 means any process. */
// #if MONOPROCESS
//     if ((gdb_dbg_tid != 0) && gdb_tid_to_handle(gdb_dbg_tid) != PsGetCurrentThreadId())
//     {
//         PETHREAD AttachedThread = find_thread(0, gdb_dbg_tid);
//         PKPROCESS AttachedProcess;
//         if (AttachedThread == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//
//         AttachedProcess = AttachedThread->Tcb.Process;
//         if (AttachedProcess == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//         __writecr3((AttachedProcess->DirectoryTableBase[0]) as usize);
//     }
// #else
//     if ((gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId())
//     {
//         PEPROCESS AttachedProcess = find_process(gdb_dbg_pid);
//         if (AttachedProcess == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//         /* Only do this if Ps is initialized */
//         if (ProcessListHead->Flink)
//             __writecr3((AttachedProcess->Pcb.DirectoryTableBase[0]) as usize);
//     }
// #endif
//
//     State->u.ReadMemory.TargetBaseAddress = hex_to_address(&gdb_input[1]);
//     State->u.ReadMemory.TransferCount = hex_to_address(strstr(&gdb_input[1], ",") + 1);
//
//     /* KD will reply with KdSendPacket. Catch it */
//     KdpSendPacketHandler = Some(ReadMemorySendHandler);
//     return KdPacketReceived;
// }
//
// static
// BOOLEAN
// WriteMemorySendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//
//     if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         // KdAssert
//         KDDBGPRINT!("Wrong packet type (%lu) received after DbgKdWriteVirtualMemoryApi request.\n", PacketType);
//         return FALSE;
//     }
//
//     if (State->ApiNumber != DbgKdWriteVirtualMemoryApi)
//     {
//         KDDBGPRINT!("Wrong API number (%lu) after DbgKdWriteVirtualMemoryApi request.\n", State->ApiNumber);
//         return FALSE;
//     }
//
//     /* Check status */
//     if (!NT_SUCCESS(State->ReturnStatus))
//         send_gdb_ntstatus(State->ReturnStatus);
//     else
//         send_gdb_packet("OK");
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = NULL;
//
// #if MONOPROCESS
//     if (gdb_dbg_tid != 0)
//     /* Reset the TLB */
// #else
//     if ((gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId())
// #endif
//     {
//         /* Only do this if Ps is initialized */
//         if (ProcessListHead->Flink)
//             __writecr3((PsGetCurrentProcess()->Pcb.DirectoryTableBase[0]) as usize);
//     }
//     return TRUE;
// }
//
// static
// KDSTATUS
// handle_gdb_write_mem(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     /* Maximal input buffer is 0x1000. Each byte is encoded on two bytes by GDB */
//     static UCHAR OutBuffer[0x800];
//     ULONG BufferLength;
//     char* blob_ptr;
//     UCHAR* OutPtr;
//
//     State->ApiNumber = DbgKdWriteVirtualMemoryApi;
//     State->ReturnStatus = STATUS_SUCCESS; /* ? */
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//
//     /* Set the TLB according to the process being read. Pid 0 means any process. */
// #if MONOPROCESS
//     if ((gdb_dbg_tid != 0) && gdb_tid_to_handle(gdb_dbg_tid) != PsGetCurrentThreadId())
//     {
//         PETHREAD AttachedThread = find_thread(0, gdb_dbg_tid);
//         PKPROCESS AttachedProcess;
//         if (AttachedThread == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//
//         AttachedProcess = AttachedThread->Tcb.Process;
//         if (AttachedProcess == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//         __writecr3((AttachedProcess->DirectoryTableBase[0]) as usize);
//     }
// #else
//     if ((gdb_dbg_pid != 0) && gdb_pid_to_handle(gdb_dbg_pid) != PsGetCurrentProcessId())
//     {
//         PEPROCESS AttachedProcess = find_process(gdb_dbg_pid);
//         if (AttachedProcess == NULL)
//         {
//             KDDBGPRINT!("The current GDB debug thread is invalid!");
//             return LOOP_IF_SUCCESS(send_gdb_packet("E03"));
//         }
//         /* Only do this if Ps is initialized */
//         if (ProcessListHead->Flink)
//             __writecr3((AttachedProcess->Pcb.DirectoryTableBase[0]) as usize);
//     }
// #endif
//
//     State->u.WriteMemory.TargetBaseAddress = hex_to_address(&gdb_input[1]);
//     BufferLength = hex_to_address(strstr(&gdb_input[1], ",") + 1);
//     if (BufferLength == 0)
//     {
//         /* Nothing to do */
//         return LOOP_IF_SUCCESS(send_gdb_packet("OK"));
//     }
//
//     State->u.WriteMemory.TransferCount = BufferLength;
//     MessageData->Length = BufferLength;
//     MessageData->Buffer = (CHAR*)OutBuffer;
//
//     OutPtr = OutBuffer;
//     blob_ptr = strstr(strstr(&gdb_input[1], ",") + 1, ":") + 1;
//     while (BufferLength)
//     {
//         if (BufferLength >= 4)
//         {
//             *((ULONG*)OutPtr) = *((ULONG*)blob_ptr);
//             OutPtr += 4;
//             blob_ptr += 4;
//             BufferLength -= 4;
//         }
//         else if (BufferLength >= 2)
//         {
//             *((USHORT*)OutPtr) = *((USHORT*)blob_ptr);
//             OutPtr += 2;
//             blob_ptr += 2;
//             BufferLength -= 2;
//         }
//         else
//         {
//             *OutPtr++ = *blob_ptr++;
//             BufferLength--;
//         }
//     }
//
//     /* KD will reply with KdSendPacket. Catch it */
//     KdpSendPacketHandler = WriteMemorySendHandler;
//     return KdPacketReceived;
// }
//
// static
// BOOLEAN
// WriteBreakPointSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//
//     if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         // KdAssert
//         KDDBGPRINT!("Wrong packet type (%lu) received after DbgKdWriteBreakPointApi request.\n", PacketType);
//         return FALSE;
//     }
//
//     if (State->ApiNumber != DbgKdWriteBreakPointApi)
//     {
//         KDDBGPRINT!("Wrong API number (%lu) after DbgKdWriteBreakPointApi request.\n", State->ApiNumber);
//         return FALSE;
//     }
//
//     /* Check status */
//     if (!NT_SUCCESS(State->ReturnStatus))
//     {
//         KDDBGPRINT!("Inserting breakpoint failed!\n");
//         send_gdb_ntstatus(State->ReturnStatus);
//     }
//     else
//     {
//         /* Keep track of the address+handle couple */
//         ULONG i;
//         for (i = 0; i < (sizeof(BreakPointHandles) / sizeof(BreakPointHandles[0])); i++)
//         {
//             if (BreakPointHandles[i].Address == 0)
//             {
//                 BreakPointHandles[i].Address = (ULONG_PTR)State->u.WriteBreakPoint.BreakPointAddress;
//                 BreakPointHandles[i].Handle = State->u.WriteBreakPoint.BreakPointHandle;
//                 break;
//             }
//         }
//         send_gdb_packet("OK");
//     }
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = NULL;
//     return TRUE;
// }
//
// static
// KDSTATUS
// handle_gdb_insert_breakpoint(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     State->ReturnStatus = STATUS_SUCCESS; /* ? */
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     if (MessageData)
//         MessageData->Length = 0;
//     *MessageLength = 0;
//
//     switch (gdb_input[1])
//     {
//         case '0':
//         {
//             ULONG_PTR Address = hex_to_address(&gdb_input[3]);
//             ULONG i;
//             BOOLEAN HasFreeSlot = FALSE;
//
//             KDDBGPRINT!("Inserting breakpoint at %p.\n", (void*)Address);
//
//             for (i = 0; i < (sizeof(BreakPointHandles) / sizeof(BreakPointHandles[0])); i++)
//             {
//                 if (BreakPointHandles[i].Address == 0)
//                     HasFreeSlot = TRUE;
//             }
//
//             if (!HasFreeSlot)
//             {
//                 /* We don't have a way to keep track of this break point. Fail. */
//                 KDDBGPRINT!("No breakpoint slot available!\n");
//                 return LOOP_IF_SUCCESS(send_gdb_packet("E01"));
//             }
//
//             State->ApiNumber = DbgKdWriteBreakPointApi;
//             State->u.WriteBreakPoint.BreakPointAddress = Address;
//             /* FIXME : ignoring all other Z0 arguments */
//
//             /* KD will reply with KdSendPacket. Catch it */
//             KdpSendPacketHandler = WriteBreakPointSendHandler;
//             return KdPacketReceived;
//         }
//     }
//
//     KDDBGPRINT!("Unhandled 'Z' packet: %s\n", gdb_input);
//     return LOOP_IF_SUCCESS(send_gdb_packet("E01"));
// }
//
// static
// BOOLEAN
// RestoreBreakPointSendHandler(
//      ULONG PacketType,
//      PSTRING MessageHeader,
//      PSTRING MessageData)
// {
//     DBGKD_MANIPULATE_STATE64* State = (DBGKD_MANIPULATE_STATE64*)MessageHeader->Buffer;
//     ULONG i;
//
//     if (PacketType != PACKET_TYPE_KD_STATE_MANIPULATE)
//     {
//         // KdAssert
//         KDDBGPRINT!("Wrong packet type (%lu) received after DbgKdRestoreBreakPointApi request.\n", PacketType);
//         return FALSE;
//     }
//
//     if (State->ApiNumber != DbgKdRestoreBreakPointApi)
//     {
//         KDDBGPRINT!("Wrong API number (%lu) after DbgKdRestoreBreakPointApi request.\n", State->ApiNumber);
//         return FALSE;
//     }
//
//     /* We ignore failure here. If DbgKdRestoreBreakPointApi fails,
//      * this means that the breakpoint was already invalid for KD. So clean it up on our side. */
//     for (i = 0; i < (sizeof(BreakPointHandles) / sizeof(BreakPointHandles[0])); i++)
//     {
//         if (BreakPointHandles[i].Handle == State->u.RestoreBreakPoint.BreakPointHandle)
//         {
//             BreakPointHandles[i].Address = 0;
//             BreakPointHandles[i].Handle = 0;
//             break;
//         }
//     }
//
//     send_gdb_packet("OK");
//
//     KdpSendPacketHandler = NULL;
//     KdpManipulateStateHandler = NULL;
//     return TRUE;
// }
//
// static
// KDSTATUS
// handle_gdb_remove_breakpoint(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     State->ReturnStatus = STATUS_SUCCESS; /* ? */
//     State->Processor = CurrentStateChange.Processor;
//     State->ProcessorLevel = CurrentStateChange.ProcessorLevel;
//     if (MessageData)
//         MessageData->Length = 0;
//     *MessageLength = 0;
//
//     switch (gdb_input[1])
//     {
//         case '0':
//         {
//             ULONG_PTR Address = hex_to_address(&gdb_input[3]);
//             ULONG i, Handle = 0;
//
//             KDDBGPRINT!("Removing breakpoint on %p.\n", (void*)Address);
//
//             for (i = 0; i < (sizeof(BreakPointHandles) / sizeof(BreakPointHandles[0])); i++)
//             {
//                 if (BreakPointHandles[i].Address == Address)
//                 {
//                     Handle = BreakPointHandles[i].Handle;
//                     break;
//                 }
//             }
//
//             if (Handle == 0)
//             {
//                 KDDBGPRINT!("Received %s, but breakpoint was never inserted ?!\n", gdb_input);
//                 return LOOP_IF_SUCCESS(send_gdb_packet("E01"));
//             }
//
//             State->ApiNumber = DbgKdRestoreBreakPointApi;
//             State->u.RestoreBreakPoint.BreakPointHandle = Handle;
//             /* FIXME : ignoring all other z0 arguments */
//
//             /* KD will reply with KdSendPacket. Catch it */
//             KdpSendPacketHandler = RestoreBreakPointSendHandler;
//             return KdPacketReceived;
//         }
//     }
//
//     KDDBGPRINT!("Unhandled 'Z' packet: %s\n", gdb_input);
//     return LOOP_IF_SUCCESS(send_gdb_packet("E01"));
// }
//
// static
// KDSTATUS
// handle_gdb_c(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     KDSTATUS Status;
//
//     /* Tell GDB everything is fine, we will handle it */
//     Status = send_gdb_packet("OK");
//     if (Status != KdPacketReceived)
//         return Status;
//
//
//     if (CurrentStateChange.NewState == DbgKdExceptionStateChange)
//     {
//         DBGKM_EXCEPTION64* Exception = &CurrentStateChange.u.Exception;
//         ULONG_PTR ProgramCounter = KdpGetContextPc(&CurrentContext);
//
//         /* See if we should update the program counter */
//         if ((Exception->ExceptionRecord.ExceptionCode == STATUS_BREAKPOINT)
//             && ((*(KD_BREAKPOINT_TYPE*)ProgramCounter) == KD_BREAKPOINT_VALUE))
//         {
//             /* We must get past the breakpoint instruction */
//             KdpSetContextPc(&CurrentContext, ProgramCounter + KD_BREAKPOINT_SIZE);
//
//             SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
//             KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
//             return KdPacketReceived;
//         }
// #if defined(_M_IX86) || defined(_M_AMD64)
//         if ((Exception->ExceptionRecord.ExceptionCode == STATUS_ASSERTION_FAILURE)
//             && ((*(KD_BREAKPOINT_TYPE*)ProgramCounter) == 0xCD)
//             && (*((KD_BREAKPOINT_TYPE*)ProgramCounter + 1) == 0x2C))
//         {
//             /* INT 2C (a.k.a. runtime check failure) */
//             KdpSetContextPc(&CurrentContext, ProgramCounter + 2);
//
//             SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
//             KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
//             return KdPacketReceived;
//         }
// #endif
//     }
//
//     return ContinueManipulateStateHandler(State, MessageData, MessageLength, KdContext);
// }
//
// static
// KDSTATUS
// handle_gdb_s(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     KDDBGPRINT!("Single stepping.\n");
//     /* Set CPU single step mode and continue */
//     KdpSetSingleStep(&CurrentContext);
//     SetContextManipulateHandler(State, MessageData, MessageLength, KdContext);
//     KdpManipulateStateHandler = Some(ContinueManipulateStateHandler);
//     return KdPacketReceived;
// }
//
// static
// KDSTATUS
// handle_gdb_v(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     if (strncmp(gdb_input.as_mut_ptr(), "vCont", 5) == 0)
//     {
//         if (gdb_input[5] == '?')
//         {
//             /* Report what we support */
//             return LOOP_IF_SUCCESS(send_gdb_packet("vCont;c;s"));
//         }
//
//         if (strncmp(gdb_input.as_mut_ptr(), "vCont;c", 7) == 0)
//         {
//             return handle_gdb_c(State, MessageData, MessageLength, KdContext);
//         }
//
//         if (strncmp(gdb_input.as_mut_ptr(), "vCont;s", 7) == 0)
//         {
//
//             return handle_gdb_s(State, MessageData, MessageLength, KdContext);
//         }
//     }
//
//     KDDBGPRINT!("Unhandled 'v' packet: %s\n", gdb_input);
//     return LOOP_IF_SUCCESS(send_gdb_packet(""));
// }
//
// KDSTATUS
// gdb_receive_and_interpret_packet(
//      DBGKD_MANIPULATE_STATE64* State,
//      PSTRING MessageData,
//      PULONG MessageLength,
//      PKD_CONTEXT KdContext)
// {
//     KDSTATUS Status;
//
//     do
//     {
//         KDDBGPRINT!("KDGBD: Receiving packet.\n");
//         Status = gdb_receive_packet(KdContext);
//         KDDBGPRINT!("KDGBD: Packet \"%s\" received with status %u\n", gdb_input, Status);
//
//         if (Status != KdPacketReceived)
//             return Status;
//
//         Status = (KDSTATUS)-1;
//
//         switch (gdb_input[0])
//         {
//         case '?':
//             /* Send the Status */
//             Status = LOOP_IF_SUCCESS(gdb_send_exception());
//             break;
//         case '!':
//             Status = LOOP_IF_SUCCESS(send_gdb_packet("OK"));
//             break;
//         case 'c':
//         case 'C':
//             Status = handle_gdb_c(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'g':
//             Status = LOOP_IF_SUCCESS(gdb_send_registers());
//             break;
//         case 'H':
//             Status = LOOP_IF_SUCCESS(handle_gdb_set_thread());
//             break;
//         case 'm':
//             Status = handle_gdb_read_mem(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'p':
//             Status = LOOP_IF_SUCCESS(gdb_send_register());
//             break;
//         case 'q':
//             Status = LOOP_IF_SUCCESS(handle_gdb_query());
//             break;
//         case 's':
//             Status = handle_gdb_s(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'T':
//             Status = LOOP_IF_SUCCESS(handle_gdb_thread_alive());
//             break;
//         case 'v':
//             Status = handle_gdb_v(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'X':
//             Status = handle_gdb_write_mem(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'z':
//             Status = handle_gdb_remove_breakpoint(State, MessageData, MessageLength, KdContext);
//             break;
//         case 'Z':
//             Status = handle_gdb_insert_breakpoint(State, MessageData, MessageLength, KdContext);
//             break;
//         default:
//             /* We don't know how to handle this request. */
//             KDDBGPRINT!("Unsupported GDB command: %s.\n", gdb_input);
//             Status = LOOP_IF_SUCCESS(send_gdb_packet(""));
//         }
//     } while (Status == (KDSTATUS)-1);
//
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
