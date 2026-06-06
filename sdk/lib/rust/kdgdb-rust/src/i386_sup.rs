use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CONTEXT, KDSTATUS, PETHREAD, PsGetThreadId};
use core::ffi::c_void;

use crate::KDDBGPRINT;
use crate::kdgdb_h::gdb_tid_to_handle;
use crate::gdb_receive::{hex_value, gdb_input};
use crate::gdb_send::{hex_chars, start_gdb_packet, send_gdb_partial_packet, finish_gdb_packet, send_gdb_packet, send_gdb_memory};
use crate::kdpacket::{CurrentContext, CurrentStateChange};
use crate::gdb_input::{gdb_dbg_pid, gdb_dbg_tid};
use crate::utils::find_thread;

// ---- file-scope preamble (translated) ----
// enum reg_name in C is `unsigned`; the dispatch scrutinee `name` is u32.
pub const EAX: u32 = 0;
pub const ECX: u32 = 1;
pub const EDX: u32 = 2;
pub const EBX: u32 = 3;
pub const ESP: u32 = 4;
pub const EBP: u32 = 5;
pub const ESI: u32 = 6;
pub const EDI: u32 = 7;
pub const EIP: u32 = 8;
pub const EFLAGS: u32 = 9;
pub const CS: u32 = 10;
pub const SS: u32 = 11;
pub const DS: u32 = 12;
pub const ES: u32 = 13;
pub const FS: u32 = 14;
pub const GS: u32 = 15;
pub const ST0: u32 = 16;
pub const ST1: u32 = 17;
pub const ST2: u32 = 18;
pub const ST3: u32 = 19;
pub const ST4: u32 = 20;
pub const ST5: u32 = 21;
pub const ST6: u32 = 22;
pub const ST7: u32 = 23;
pub const FCTRL: u32 = 24;
pub const FSTAT: u32 = 25;
pub const FTAG: u32 = 26;
pub const FISEG: u32 = 27;
pub const FIOFF: u32 = 28;
pub const FOSEG: u32 = 29;
pub const FOOFF: u32 = 30;
pub const FOP: u32 = 31;
pub const XMM0: u32 = 32;
pub const XMM1: u32 = 33;
pub const XMM2: u32 = 34;
pub const XMM3: u32 = 35;
pub const XMM4: u32 = 36;
pub const XMM5: u32 = 37;
pub const XMM6: u32 = 38;
pub const XMM7: u32 = 39;
pub const MXCSR: u32 = 40;

unsafe fn ctx_to_reg(ctx: *mut CONTEXT, name: u32, size: *mut u16) -> *mut core::ffi::c_void {
    *size = 4;
    match name {
        EAX => core::ptr::addr_of_mut!((*ctx).Eax) as *mut core::ffi::c_void,
        EBX => core::ptr::addr_of_mut!((*ctx).Ebx) as *mut core::ffi::c_void,
        ECX => core::ptr::addr_of_mut!((*ctx).Ecx) as *mut core::ffi::c_void,
        EDX => core::ptr::addr_of_mut!((*ctx).Edx) as *mut core::ffi::c_void,
        ESP => core::ptr::addr_of_mut!((*ctx).Esp) as *mut core::ffi::c_void,
        EBP => core::ptr::addr_of_mut!((*ctx).Ebp) as *mut core::ffi::c_void,
        ESI => core::ptr::addr_of_mut!((*ctx).Esi) as *mut core::ffi::c_void,
        EDI => core::ptr::addr_of_mut!((*ctx).Edi) as *mut core::ffi::c_void,
        EIP => core::ptr::addr_of_mut!((*ctx).Eip) as *mut core::ffi::c_void,
        EFLAGS => core::ptr::addr_of_mut!((*ctx).EFlags) as *mut core::ffi::c_void,
        CS => core::ptr::addr_of_mut!((*ctx).SegCs) as *mut core::ffi::c_void,
        DS => core::ptr::addr_of_mut!((*ctx).SegDs) as *mut core::ffi::c_void,
        ES => core::ptr::addr_of_mut!((*ctx).SegEs) as *mut core::ffi::c_void,
        FS => core::ptr::addr_of_mut!((*ctx).SegFs) as *mut core::ffi::c_void,
        GS => core::ptr::addr_of_mut!((*ctx).SegGs) as *mut core::ffi::c_void,
        SS => core::ptr::addr_of_mut!((*ctx).SegSs) as *mut core::ffi::c_void,
        ST0 | ST1 | ST2 | ST3 | ST4 | ST5 | ST6 | ST7 => {
            *size = 10;
            core::ptr::addr_of_mut!((*ctx).FloatSave.RegisterArea[(10 * (name - ST0)) as usize]) as *mut core::ffi::c_void
        }
        FCTRL => core::ptr::addr_of_mut!((*ctx).FloatSave.ControlWord) as *mut core::ffi::c_void,
        FSTAT => core::ptr::addr_of_mut!((*ctx).FloatSave.StatusWord) as *mut core::ffi::c_void,
        FTAG => core::ptr::addr_of_mut!((*ctx).FloatSave.TagWord) as *mut core::ffi::c_void,
        FISEG => core::ptr::addr_of_mut!((*ctx).FloatSave.DataSelector) as *mut core::ffi::c_void,
        FIOFF => core::ptr::addr_of_mut!((*ctx).FloatSave.DataOffset) as *mut core::ffi::c_void,
        FOSEG => core::ptr::addr_of_mut!((*ctx).FloatSave.ErrorSelector) as *mut core::ffi::c_void,
        FOOFF => core::ptr::addr_of_mut!((*ctx).FloatSave.ErrorOffset) as *mut core::ffi::c_void,
        FOP => core::ptr::addr_of_mut!((*ctx).FloatSave.Cr0NpxState) as *mut core::ffi::c_void,
        XMM0 | XMM1 | XMM2 | XMM3 | XMM4 | XMM5 | XMM6 | XMM7 => {
            *size = 16;
            core::ptr::addr_of_mut!((*ctx).ExtendedRegisters[(160 + (name - XMM0) * 16) as usize]) as *mut core::ffi::c_void
        }
        MXCSR => core::ptr::addr_of_mut!((*ctx).ExtendedRegisters[24]) as *mut core::ffi::c_void,
        _ => core::ptr::null_mut(),
    }
}

unsafe fn thread_to_reg(Thread: PETHREAD, reg_name: u32, size: *mut u16) -> *mut core::ffi::c_void {
    static mut NullValue: *const core::ffi::c_void = core::ptr::null();
    static mut Esp: *mut u32 = core::ptr::null_mut();

    if (*Thread).Tcb.InitialStack.is_null() {
        match reg_name {
            4 | 5 | 8 => {
                KDDBGPRINT!(b"Returning NULL for register %d.\n\0".as_ptr() as *const i8, reg_name);
                *size = 4;
                core::ptr::addr_of!(NullValue) as *mut core::ffi::c_void
            }
            _ => core::ptr::null_mut()
        }
    } else {
        Esp = (*Thread).Tcb.KernelStack as *mut u32;
        *size = 4;
        match reg_name {
            5 => core::ptr::addr_of_mut!(*Esp.offset(3)) as *mut core::ffi::c_void,
            4 => core::ptr::addr_of_mut!(Esp) as *mut core::ffi::c_void,
            8 => core::ptr::addr_of!(NullValue) as *mut core::ffi::c_void,
            _ => core::ptr::null_mut()
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_registers() -> KDSTATUS {
    let mut RegisterStr: [i8; 9] = [0; 9];
    let mut RegisterPtr: *mut u8;
    let mut i: u32;
    let mut size: u16 = 0;

    RegisterStr[8] = b'\0' as i8;

    start_gdb_packet();

    KDDBGPRINT!(b"Sending registers of thread %lx.\n\0".as_ptr() as *const i8, gdb_dbg_tid);
    KDDBGPRINT!(b"Current thread_id: %p.\n\0".as_ptr() as *const i8, PsGetThreadId(CurrentStateChange.Thread as PETHREAD));
    if ((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
            gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId(CurrentStateChange.Thread as PETHREAD)
    {
        i = 0;
        while i < 16 {
            RegisterPtr = ctx_to_reg(core::ptr::addr_of_mut!(CurrentContext), i, core::ptr::addr_of_mut!(size)) as *mut u8;
            RegisterStr[0] = hex_chars[(*RegisterPtr.offset(0) >> 4) as usize] as i8;
            RegisterStr[1] = hex_chars[(*RegisterPtr.offset(0) & 0xF) as usize] as i8;
            RegisterStr[2] = hex_chars[(*RegisterPtr.offset(1) >> 4) as usize] as i8;
            RegisterStr[3] = hex_chars[(*RegisterPtr.offset(1) & 0xF) as usize] as i8;
            RegisterStr[4] = hex_chars[(*RegisterPtr.offset(2) >> 4) as usize] as i8;
            RegisterStr[5] = hex_chars[(*RegisterPtr.offset(2) & 0xF) as usize] as i8;
            RegisterStr[6] = hex_chars[(*RegisterPtr.offset(3) >> 4) as usize] as i8;
            RegisterStr[7] = hex_chars[(*RegisterPtr.offset(3) & 0xF) as usize] as i8;

            send_gdb_partial_packet(RegisterStr.as_ptr());
            i += 1;
        }
    }
    else
    {
        let DbgThread: PETHREAD;

        DbgThread = find_thread(gdb_dbg_pid, gdb_dbg_tid);

        if DbgThread.is_null()
        {
            send_gdb_partial_packet(b"E03\0".as_ptr() as *const i8);
            return finish_gdb_packet();
        }

        i = 0;
        while i < 16 {
            RegisterPtr = thread_to_reg(DbgThread, i, core::ptr::addr_of_mut!(size)) as *mut u8;
            if !RegisterPtr.is_null()
            {
                RegisterStr[0] = hex_chars[(*RegisterPtr.offset(0) >> 4) as usize] as i8;
                RegisterStr[1] = hex_chars[(*RegisterPtr.offset(0) & 0xF) as usize] as i8;
                RegisterStr[2] = hex_chars[(*RegisterPtr.offset(1) >> 4) as usize] as i8;
                RegisterStr[3] = hex_chars[(*RegisterPtr.offset(1) & 0xF) as usize] as i8;
                RegisterStr[4] = hex_chars[(*RegisterPtr.offset(2) >> 4) as usize] as i8;
                RegisterStr[5] = hex_chars[(*RegisterPtr.offset(2) & 0xF) as usize] as i8;
                RegisterStr[6] = hex_chars[(*RegisterPtr.offset(3) >> 4) as usize] as i8;
                RegisterStr[7] = hex_chars[(*RegisterPtr.offset(3) & 0xF) as usize] as i8;

                send_gdb_partial_packet(RegisterStr.as_ptr());
            }
            else
            {
                send_gdb_partial_packet(b"xxxxxxxx\0".as_ptr() as *const i8);
            }
            i += 1;
        }
    }

    finish_gdb_packet()
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_register() -> KDSTATUS {
    let mut reg_name: u32;
    let mut ptr: *mut core::ffi::c_void;
    let mut size: u16 = 0;

    reg_name = ((hex_value(gdb_input[1]) << 4) | hex_value(gdb_input[2])) as u32;

    if ((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
            gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId(CurrentStateChange.Thread as PETHREAD) {
        ptr = ctx_to_reg(core::ptr::addr_of_mut!(CurrentContext), reg_name, core::ptr::addr_of_mut!(size));
    } else {
        let DbgThread: PETHREAD;

        DbgThread = find_thread(gdb_dbg_pid, gdb_dbg_tid);

        if DbgThread.is_null() {
            return send_gdb_packet(b"E03\0".as_ptr() as *const i8);
        }

        ptr = thread_to_reg(DbgThread, reg_name, core::ptr::addr_of_mut!(size));
    }

    if ptr.is_null() {
        return send_gdb_packet(b"xxxxxxxx\0".as_ptr() as *const i8);
    } else {
        KDDBGPRINT!(b"KDDBG : Sending registers as memory.\n\0".as_ptr() as *const i8);
        return send_gdb_memory(ptr, size as usize);
    }
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
// enum reg_name
// {
//     EAX, ECX, EDX, EBX, ESP, EBP, ESI, EDI,
//     EIP,
//     EFLAGS,
//     CS, SS, DS, ES, FS, GS,
//     ST0, ST1, ST2, ST3, ST4, ST5, ST6, ST7,
//     FCTRL, FSTAT, FTAG, FISEG, FIOFF, FOSEG, FOOFF, FOP,
//     XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6, XMM7,
//     MXCSR
// };
//
// static
// void*
// ctx_to_reg(CONTEXT* ctx, enum reg_name name, unsigned short* size)
// {
//     /* For general registers: 32bits */
//     *size = 4;
//     switch (name)
//     {
//     case EAX: return &ctx->Eax;
//     case EBX: return &ctx->Ebx;
//     case ECX: return &ctx->Ecx;
//     case EDX: return &ctx->Edx;
//     case ESP: return &ctx->Esp;
//     case EBP: return &ctx->Ebp;
//     case ESI: return &ctx->Esi;
//     case EDI: return &ctx->Edi;
//     case EIP: return &ctx->Eip;
//     case EFLAGS: return &ctx->EFlags;
//     case CS: return &ctx->SegCs;
//     case DS: return &ctx->SegDs;
//     case ES: return &ctx->SegEs;
//     case FS: return &ctx->SegFs;
//     case GS: return &ctx->SegGs;
//     case SS: return &ctx->SegSs;
//     /* 80 bits */
//     case ST0:
//     case ST1:
//     case ST2:
//     case ST3:
//     case ST4:
//     case ST5:
//     case ST6:
//     case ST7:
//         *size = 10;
//         return &ctx->FloatSave.RegisterArea[10 * (name - ST0)];
//     /* X87 registers */
//     case FCTRL: return &ctx->FloatSave.ControlWord;
//     case FSTAT: return &ctx->FloatSave.StatusWord;
//     case FTAG: return &ctx->FloatSave.TagWord;
//     case FISEG: return &ctx->FloatSave.DataSelector;
//     case FIOFF: return &ctx->FloatSave.DataOffset;
//     case FOSEG: return &ctx->FloatSave.ErrorSelector;
//     case FOOFF: return &ctx->FloatSave.ErrorOffset;
//     case FOP: return &ctx->FloatSave.Cr0NpxState;
//     /* SSE */
//     case XMM0:
//     case XMM1:
//     case XMM2:
//     case XMM3:
//     case XMM4:
//     case XMM5:
//     case XMM6:
//     case XMM7:
//         *size = 16;
//         return &ctx->ExtendedRegisters[160 + (name - XMM0)*16];
//     case MXCSR: return &ctx->ExtendedRegisters[24];
//     }
//     return 0;
// }
//
// static
// void*
// thread_to_reg(PETHREAD Thread, enum reg_name reg_name, unsigned short* size)
// {
//     static const void* NullValue = NULL;
//
//     if (!Thread->Tcb.InitialStack)
//     {
//         /* Terminated thread ? */
//         switch (reg_name)
//         {
//             case ESP:
//             case EBP:
//             case EIP:
//                 KDDBGPRINT!("Returning NULL for register %d.\n", reg_name);
//                 *size = 4;
//                 return &NullValue;
//             default:
//                 return NULL;
//         }
//     }
//     else
//     {
//         static PULONG Esp;
//         Esp = Thread->Tcb.KernelStack;
//         *size = 4;
//         switch(reg_name)
//         {
//             case EBP: return &Esp[3];
//             case ESP: return &Esp;
//             case EIP: return &NullValue;
//             default:
//                 return NULL;
//         }
//     }
//
//     return NULL;
// }
//
// KDSTATUS
// gdb_send_registers(void)
// {
//     CHAR RegisterStr[9];
//     UCHAR* RegisterPtr;
//     unsigned i;
//     unsigned short size;
//
//     RegisterStr[8] = '\0';
//
//     start_gdb_packet();
//
//     KDDBGPRINT!("Sending registers of thread %" PRIxPTR ".\n", gdb_dbg_tid);
//     KDDBGPRINT!("Current thread_id: %p.\n", PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread));
//     if (((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
//             gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread))
//     {
//         for(i=0; i < 16; i++)
//         {
//             RegisterPtr = ctx_to_reg(&CurrentContext, i, &size);
//             RegisterStr[0] = hex_chars[RegisterPtr[0] >> 4];
//             RegisterStr[1] = hex_chars[RegisterPtr[0] & 0xF];
//             RegisterStr[2] = hex_chars[RegisterPtr[1] >> 4];
//             RegisterStr[3] = hex_chars[RegisterPtr[1] & 0xF];
//             RegisterStr[4] = hex_chars[RegisterPtr[2] >> 4];
//             RegisterStr[5] = hex_chars[RegisterPtr[2] & 0xF];
//             RegisterStr[6] = hex_chars[RegisterPtr[3] >> 4];
//             RegisterStr[7] = hex_chars[RegisterPtr[3] & 0xF];
//
//             send_gdb_partial_packet(RegisterStr);
//         }
//     }
//     else
//     {
//         PETHREAD DbgThread;
//
//         DbgThread = find_thread(gdb_dbg_pid, gdb_dbg_tid);
//
//         if (DbgThread == NULL)
//         {
//             /* Thread is dead */
//             send_gdb_partial_packet("E03");
//             return finish_gdb_packet();
//         }
//
//         for(i=0; i < 16; i++)
//         {
//             RegisterPtr = thread_to_reg(DbgThread, i, &size);
//             if (RegisterPtr)
//             {
//                 RegisterStr[0] = hex_chars[RegisterPtr[0] >> 4];
//                 RegisterStr[1] = hex_chars[RegisterPtr[0] & 0xF];
//                 RegisterStr[2] = hex_chars[RegisterPtr[1] >> 4];
//                 RegisterStr[3] = hex_chars[RegisterPtr[1] & 0xF];
//                 RegisterStr[4] = hex_chars[RegisterPtr[2] >> 4];
//                 RegisterStr[5] = hex_chars[RegisterPtr[2] & 0xF];
//                 RegisterStr[6] = hex_chars[RegisterPtr[3] >> 4];
//                 RegisterStr[7] = hex_chars[RegisterPtr[3] & 0xF];
//
//                 send_gdb_partial_packet(RegisterStr);
//             }
//             else
//             {
//                 send_gdb_partial_packet("xxxxxxxx");
//             }
//         }
//     }
//
//     return finish_gdb_packet();
// }
//
// KDSTATUS
// gdb_send_register(void)
// {
//     enum reg_name reg_name;
//     void *ptr;
//     unsigned short size;
//
//     /* Get the GDB register name (gdb_input = "pXX") */
//     reg_name = (hex_value(gdb_input[1]) << 4) | hex_value(gdb_input[2]);
//
//     if (((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
//             gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread))
//     {
//         /* We can get it from the context of the current exception */
//         ptr = ctx_to_reg(&CurrentContext, reg_name, &size);
//     }
//     else
//     {
//         PETHREAD DbgThread;
//
//         DbgThread = find_thread(gdb_dbg_pid, gdb_dbg_tid);
//
//         if (DbgThread == NULL)
//         {
//             /* Thread is dead */
//             return send_gdb_packet("E03");
//         }
//
//         ptr = thread_to_reg(DbgThread, reg_name, &size);
//     }
//
//     if (!ptr)
//     {
//         /* Undefined. Let's assume 32 bit register */
//         return send_gdb_packet("xxxxxxxx");
//     }
//     else
//     {
//         KDDBGPRINT!("KDDBG : Sending registers as memory.\n");
//         return send_gdb_memory(ptr, size as usize);
//     }
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
