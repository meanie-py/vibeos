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
// `enum reg_name` in C is `unsigned`; the dispatch scrutinee is u32.
#[allow(non_camel_case_types)]
type reg_name = u32;
pub const RAX: reg_name = 0;
pub const RBX: reg_name = 1;
pub const RCX: reg_name = 2;
pub const RDX: reg_name = 3;
pub const RSI: reg_name = 4;
pub const RDI: reg_name = 5;
pub const RBP: reg_name = 6;
pub const RSP: reg_name = 7;
pub const R8: reg_name = 8;
pub const R9: reg_name = 9;
pub const R10: reg_name = 10;
pub const R11: reg_name = 11;
pub const R12: reg_name = 12;
pub const R13: reg_name = 13;
pub const R14: reg_name = 14;
pub const R15: reg_name = 15;
pub const RIP: reg_name = 16;
pub const EFLAGS: reg_name = 17;
pub const CS: reg_name = 18;
pub const SS: reg_name = 19;
pub const DS: reg_name = 20;
pub const ES: reg_name = 21;
pub const FS: reg_name = 22;
pub const GS: reg_name = 23;
pub const ST0: reg_name = 24;
pub const ST1: reg_name = 25;
pub const ST2: reg_name = 26;
pub const ST3: reg_name = 27;
pub const ST4: reg_name = 28;
pub const ST5: reg_name = 29;
pub const ST6: reg_name = 30;
pub const ST7: reg_name = 31;
pub const FCTRL: reg_name = 32;
pub const FSTAT: reg_name = 33;
pub const FTAG: reg_name = 34;
pub const FISEG: reg_name = 35;
pub const FIOFF: reg_name = 36;
pub const FOSEG: reg_name = 37;
pub const FOOFF: reg_name = 38;
pub const FOP: reg_name = 39;

pub static reg_size: [u8; 40] = [
    8, 8, 8, 8, 8, 8, 8, 8,
    8, 8, 8, 8, 8, 8, 8, 8,
    8,
    4,
    4, 4, 4, 4, 4, 4,
    10, 10, 10, 10, 10, 10, 10, 10,
    8, 8, 8, 8, 8, 8, 8, 8
];

unsafe fn ctx_to_reg(ctx: *mut CONTEXT, name: reg_name) -> *mut core::ffi::c_void {
    match name {
        RAX => core::ptr::addr_of_mut!((*ctx).Rax) as *mut core::ffi::c_void,
        RBX => core::ptr::addr_of_mut!((*ctx).Rbx) as *mut core::ffi::c_void,
        RCX => core::ptr::addr_of_mut!((*ctx).Rcx) as *mut core::ffi::c_void,
        RDX => core::ptr::addr_of_mut!((*ctx).Rdx) as *mut core::ffi::c_void,
        RSP => core::ptr::addr_of_mut!((*ctx).Rsp) as *mut core::ffi::c_void,
        RBP => core::ptr::addr_of_mut!((*ctx).Rbp) as *mut core::ffi::c_void,
        RSI => core::ptr::addr_of_mut!((*ctx).Rsi) as *mut core::ffi::c_void,
        RDI => core::ptr::addr_of_mut!((*ctx).Rdi) as *mut core::ffi::c_void,
        RIP => core::ptr::addr_of_mut!((*ctx).Rip) as *mut core::ffi::c_void,
        R8 => core::ptr::addr_of_mut!((*ctx).R8) as *mut core::ffi::c_void,
        R9 => core::ptr::addr_of_mut!((*ctx).R9) as *mut core::ffi::c_void,
        R10 => core::ptr::addr_of_mut!((*ctx).R10) as *mut core::ffi::c_void,
        R11 => core::ptr::addr_of_mut!((*ctx).R11) as *mut core::ffi::c_void,
        R12 => core::ptr::addr_of_mut!((*ctx).R12) as *mut core::ffi::c_void,
        R13 => core::ptr::addr_of_mut!((*ctx).R13) as *mut core::ffi::c_void,
        R14 => core::ptr::addr_of_mut!((*ctx).R14) as *mut core::ffi::c_void,
        R15 => core::ptr::addr_of_mut!((*ctx).R15) as *mut core::ffi::c_void,
        EFLAGS => core::ptr::addr_of_mut!((*ctx).EFlags) as *mut core::ffi::c_void,
        CS => core::ptr::addr_of_mut!((*ctx).SegCs) as *mut core::ffi::c_void,
        DS => core::ptr::addr_of_mut!((*ctx).SegSs) as *mut core::ffi::c_void,
        ES => core::ptr::addr_of_mut!((*ctx).SegEs) as *mut core::ffi::c_void,
        FS => core::ptr::addr_of_mut!((*ctx).SegFs) as *mut core::ffi::c_void,
        GS => core::ptr::addr_of_mut!((*ctx).SegGs) as *mut core::ffi::c_void,
        SS => core::ptr::addr_of_mut!((*ctx).SegSs) as *mut core::ffi::c_void,
        _ => core::ptr::null_mut(),
    }
}

unsafe fn thread_to_reg(Thread: PETHREAD, reg_name: reg_name) -> *mut core::ffi::c_void {
    static mut NullValue: *const core::ffi::c_void = core::ptr::null();

    if (*Thread).Tcb.InitialStack.is_null() {
        match reg_name {
            RSP | RBP | RIP => {
                KDDBGPRINT!(b"Returning NULL for register %d.\n\0".as_ptr() as PCHAR, reg_name);
                core::ptr::addr_of_mut!(NullValue) as *mut core::ffi::c_void
            }
            _ => core::ptr::null_mut(),
        }
    } else {
        match reg_name {
            RSP => core::ptr::addr_of_mut!((*Thread).Tcb.KernelStack) as *mut core::ffi::c_void,
            RIP => {
                let Rsp = (*Thread).Tcb.KernelStack as PULONG_PTR;
                core::ptr::addr_of_mut!((*Rsp.offset(3))) as *mut core::ffi::c_void
            }
            RBP => {
                let Rsp = (*Thread).Tcb.KernelStack as PULONG_PTR;
                core::ptr::addr_of_mut!((*Rsp.offset(4))) as *mut core::ffi::c_void
            }
            _ => core::ptr::null_mut(),
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_registers() -> KDSTATUS {
    let mut RegisterStr: [CHAR; 17] = [0; 17];
    let mut RegisterPtr: *mut u8;
    let mut i: u16;
    let mut size: u16;

    start_gdb_packet();

    KDDBGPRINT!(b"Sending registers of thread %lx.\n\0".as_ptr() as PCHAR, gdb_dbg_tid);
    KDDBGPRINT!(b"Current thread_id: %p.\n\0".as_ptr() as PCHAR, PsGetThreadId(CurrentStateChange.Thread as PETHREAD));
    
    if ((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
            gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId(CurrentStateChange.Thread as PETHREAD)
    {
        i = 0;
        while i < 24 {
            RegisterPtr = ctx_to_reg(core::ptr::addr_of_mut!(CurrentContext), i as u32) as *mut u8;
            size = reg_size[i as usize] as u16 * 2;
            RegisterStr[size as usize] = 0;
            while size > 0 {
                size -= 1;
                RegisterStr[size as usize] = hex_chars[(*RegisterPtr.offset((size / 2) as isize) & 0xF) as usize] as CHAR;
                size -= 1;
                RegisterStr[size as usize] = hex_chars[(*RegisterPtr.offset((size / 2) as isize) >> 4) as usize] as CHAR;
            }

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
            send_gdb_partial_packet(b"E03\0".as_ptr() as PCHAR);
            return finish_gdb_packet();
        }

        i = 0;
        while i < 24 {
            RegisterPtr = thread_to_reg(DbgThread, i as u32) as *mut u8;
            size = reg_size[i as usize] as u16 * 2;
            RegisterStr[size as usize] = 0;

            while size > 0 {
                if !RegisterPtr.is_null()
                {
                    size -= 1;
                    RegisterStr[size as usize] = hex_chars[(*RegisterPtr.offset((size / 2) as isize) & 0xF) as usize] as CHAR;
                    size -= 1;
                    RegisterStr[size as usize] = hex_chars[(*RegisterPtr.offset((size / 2) as isize) >> 4) as usize] as CHAR;
                }
                else
                {
                    size -= 1;
                    RegisterStr[size as usize] = b'x' as CHAR;
                    size -= 1;
                    RegisterStr[size as usize] = b'x' as CHAR;
                }
            }

            send_gdb_partial_packet(RegisterStr.as_ptr());
            i += 1;
        }
    }

    finish_gdb_packet()
}
#[no_mangle]
pub unsafe extern "C" fn gdb_send_register() -> KDSTATUS {
    let reg_name: reg_name;
    let ptr: *mut core::ffi::c_void;

    reg_name = ((hex_value(gdb_input[1]) << 4) | hex_value(gdb_input[2])) as reg_name;

    if ((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
            gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId(CurrentStateChange.Thread as PETHREAD)
    {
        ptr = ctx_to_reg(core::ptr::addr_of_mut!(CurrentContext), reg_name);
    }
    else
    {
        let DbgThread: PETHREAD;

        DbgThread = find_thread(gdb_dbg_pid, gdb_dbg_tid);

        if DbgThread.is_null()
        {
            return send_gdb_packet(b"E03\0".as_ptr() as *const i8);
        }

        ptr = thread_to_reg(DbgThread, reg_name);
    }

    if ptr.is_null()
    {
        let mut size = reg_size[reg_name as usize];
        start_gdb_packet();
        while size > 0 {
            send_gdb_partial_packet(b"xx\0".as_ptr() as *const i8);
            size -= 1;
        }
        return finish_gdb_packet();
    }
    else
    {
        KDDBGPRINT!(b"KDDBG : Sending registers as memory.\n\0".as_ptr() as *const i8);
        return send_gdb_memory(ptr, reg_size[reg_name as usize] as usize);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS KD dll - GDB stub
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Base functions for the kernel debugger
//  * COPYRIGHT:   Copyright 2021 Jérôme Gardou
//  */
//
// #include "kdgdb.h"
//
// enum reg_name
// {
//     RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP,
//     R8, R9, R10, R11, R12, R13, R14, R15,
//     RIP,
//     EFLAGS,
//     CS, SS, DS, ES, FS, GS,
//     ST0, ST1, ST2, ST3, ST4, ST5, ST6, ST7,
//     FCTRL, FSTAT, FTAG, FISEG, FIOFF, FOSEG, FOOFF, FOP
// };
//
// static const unsigned char reg_size[] =
// {
//     8, 8, 8, 8, 8, 8, 8, 8,
//     8, 8, 8, 8, 8, 8, 8, 8,
//     8,
//     4,
//     4, 4, 4, 4, 4, 4,
//     10, 10, 10, 10, 10, 10, 10, 10,
//     8, 8, 8, 8, 8, 8, 8, 8
// };
//
// static
// void*
// ctx_to_reg(CONTEXT* ctx, enum reg_name name)
// {
//     switch (name)
//     {
//         case RAX: return &ctx->Rax;
//         case RBX: return &ctx->Rbx;
//         case RCX: return &ctx->Rcx;
//         case RDX: return &ctx->Rdx;
//         case RSP: return &ctx->Rsp;
//         case RBP: return &ctx->Rbp;
//         case RSI: return &ctx->Rsi;
//         case RDI: return &ctx->Rdi;
//         case RIP: return &ctx->Rip;
//         case R8: return &ctx->R8;
//         case R9: return &ctx->R9;
//         case R10: return &ctx->R10;
//         case R11: return &ctx->R11;
//         case R12: return &ctx->R12;
//         case R13: return &ctx->R13;
//         case R14: return &ctx->R14;
//         case R15: return &ctx->R15;
//         case EFLAGS: return &ctx->EFlags;
//         case CS: return &ctx->SegCs;
//         case DS: return &ctx->SegSs;
//         case ES: return &ctx->SegEs;
//         case FS: return &ctx->SegFs;
//         case GS: return &ctx->SegGs;
//         case SS: return &ctx->SegSs;
//     }
// #undef return_reg
//     return 0;
// }
//
// static
// void*
// thread_to_reg(PETHREAD Thread, enum reg_name reg_name)
// {
//     static const void* NullValue = NULL;
//
//     if (!Thread->Tcb.InitialStack)
//     {
//         /* Terminated thread ? */
//         switch (reg_name)
//         {
//             case RSP:
//             case RBP:
//             case RIP:
//                 KDDBGPRINT!("Returning NULL for register %d.\n", reg_name);
//                 return &NullValue;
//             default:
//                 return NULL;
//         }
//     }
//     else
//     {
//         switch(reg_name)
//         {
//             case RSP: return &Thread->Tcb.KernelStack;
//             case RIP:
//             {
//                 PULONG_PTR Rsp = Thread->Tcb.KernelStack;
//                 return &Rsp[3];
//             }
//             case RBP:
//             {
//                 PULONG_PTR Rsp = Thread->Tcb.KernelStack;
//                 return &Rsp[4];
//             }
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
//     CHAR RegisterStr[17];
//     UCHAR* RegisterPtr;
//     unsigned short i;
//     unsigned short size;
//
//     start_gdb_packet();
//
//     KDDBGPRINT!("Sending registers of thread %" PRIxPTR ".\n", gdb_dbg_tid);
//     KDDBGPRINT!("Current thread_id: %p.\n", PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread));
//     if (((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
//             gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread))
//     {
//         for (i = 0; i < 24; i++)
//         {
//             RegisterPtr = ctx_to_reg(&CurrentContext, i);
//             size = reg_size[i] * 2;
//             RegisterStr[size] = 0;
//             while (size)
//             {
//                 size--;
//                 RegisterStr[size] = hex_chars[RegisterPtr[size/2] & 0xF];
//                 size--;
//                 RegisterStr[size] = hex_chars[RegisterPtr[size/2] >> 4];
//             }
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
//         for (i = 0; i < 24; i++)
//         {
//             RegisterPtr = thread_to_reg(DbgThread, i as u32);
//             size = reg_size[i] * 2;
//             RegisterStr[size] = 0;
//
//             while (size)
//             {
//                 if (RegisterPtr)
//                 {
//                     size--;
//                     RegisterStr[size] = hex_chars[RegisterPtr[size/2] & 0xF];
//                     size--;
//                     RegisterStr[size] = hex_chars[RegisterPtr[size/2] >> 4];
//                 }
//                 else
//                 {
//                     size--;
//                     RegisterStr[size] = 'x';
//                     size--;
//                     RegisterStr[size] = 'x';
//                 }
//             }
//
//             send_gdb_partial_packet(RegisterStr);
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
//
//     /* Get the GDB register name (gdb_input = "pXX") */
//     reg_name = (hex_value(gdb_input[1]) << 4) | hex_value(gdb_input[2]);
//
//     if (((gdb_dbg_pid == 0) && (gdb_dbg_tid == 0)) ||
//             gdb_tid_to_handle(gdb_dbg_tid) == PsGetThreadId((PETHREAD)(ULONG_PTR)CurrentStateChange.Thread))
//     {
//         /* We can get it from the context of the current exception */
//         ptr = ctx_to_reg(&CurrentContext, reg_name);
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
//         ptr = thread_to_reg(DbgThread, reg_name);
//     }
//
//     if (!ptr)
//     {
//         unsigned char size = reg_size[reg_name];
//         start_gdb_packet();
//         while (size--)
//             send_gdb_partial_packet("xx");
//         return finish_gdb_packet();
//     }
//     else
//     {
//         KDDBGPRINT!("KDDBG : Sending registers as memory.\n");
//         return send_gdb_memory(ptr, reg_size[reg_name]);
//     }
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
