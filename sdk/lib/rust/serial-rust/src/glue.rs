// glue.rs — driver-private macro/helper implementations and enum-variant aliases.
// The rustconv/c2rust output uses C enum member names directly (e.g. `WaitAny`,
// `Latched`) and C macro names (SER_LCR, TRACE_, etc.). This module provides
// all of them so callers can just `use crate::glue::*;`.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports)]

use ntkrnl_sys::base::{PUCHAR, UCHAR, ULONG, PVOID, NTSTATUS};

// Re-export types that the translated modules need but that aren't in base::*
// (they're in the bindgen ddk layer). Doing selective named imports here avoids
// the glob-on-glob ambiguity that occurs when a module already has base::*.
pub use ntkrnl_sys::ddk::{
    PCSTR, CCHAR,
    _UNICODE_STRING, IO_REMOVE_LOCK, _IO_REMOVE_LOCK_COMMON_BLOCK,
    KINTERRUPT_MODE, KPROCESSOR_MODE,
    CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR,
    FILE_POSITION_INFORMATION, FILE_STANDARD_INFORMATION,
    PFILE_POSITION_INFORMATION, PFILE_STANDARD_INFORMATION,
    ACCESS_MASK,
    // MM_PAGE_PRIORITY value used in devctrl.rs
    _MM_PAGE_PRIORITY_NormalPagePriority as NormalPagePriority,
    // Raw struct types used by name in pnp.rs / serial.rs / power.rs
    _KINTERRUPT, _IRP, _DEVICE_OBJECT, _DRIVER_OBJECT,
    // HANDLE / PUNICODE_STRING used in pnp.rs
    HANDLE, PUNICODE_STRING,
    // KEVENT used by IoReleaseRemoveLockEx inline
    KEVENT,
};

// ---------------------------------------------------------------------------
// IRP-stack inlines — FORCEINLINEs in wdm.h; the bindgen externs in ddk::
// compile but are NOT exported from ntoskrnl.lib on i386.  Use the Rust
// inline implementations from ntkrnl_sys::inline::tier1 instead.
// ---------------------------------------------------------------------------
pub use ntkrnl_sys::inline::tier1::{
    IoGetCurrentIrpStackLocation,
    IoSkipCurrentIrpStackLocation,
    IoMarkIrpPending,
};

// ---------------------------------------------------------------------------
// KeInitializeSpinLock — FORCEINLINE in wdm.h; same story on i386.
// ---------------------------------------------------------------------------
pub use ntkrnl_sys::inline::ke::KeInitializeSpinLock;

// ---------------------------------------------------------------------------
// UART register offset accessors — C macros SER_xxx(base) → base + n
// (serial.h §UART registers: RBR/THR/DLL=0, IER/DLM=1, IIR/FCR=2,
//  LCR=3, MCR=4, LSR=5, MSR=6, SCR=7)
// ---------------------------------------------------------------------------
#[inline(always)] pub unsafe fn SER_RBR(x: PUCHAR) -> PUCHAR { x.offset(0) }
#[inline(always)] pub unsafe fn SER_THR(x: PUCHAR) -> PUCHAR { x.offset(0) }
#[inline(always)] pub unsafe fn SER_DLL(x: PUCHAR) -> PUCHAR { x.offset(0) }
#[inline(always)] pub unsafe fn SER_IER(x: PUCHAR) -> PUCHAR { x.offset(1) }
#[inline(always)] pub unsafe fn SER_DLM(x: PUCHAR) -> PUCHAR { x.offset(1) }
#[inline(always)] pub unsafe fn SER_IIR(x: PUCHAR) -> PUCHAR { x.offset(2) }
#[inline(always)] pub unsafe fn SER_FCR(x: PUCHAR) -> PUCHAR { x.offset(2) }
#[inline(always)] pub unsafe fn SER_LCR(x: PUCHAR) -> PUCHAR { x.offset(3) }
#[inline(always)] pub unsafe fn SER_MCR(x: PUCHAR) -> PUCHAR { x.offset(4) }
#[inline(always)] pub unsafe fn SER_LSR(x: PUCHAR) -> PUCHAR { x.offset(5) }
#[inline(always)] pub unsafe fn SER_MSR(x: PUCHAR) -> PUCHAR { x.offset(6) }
#[inline(always)] pub unsafe fn SER_SCR(x: PUCHAR) -> PUCHAR { x.offset(7) }

// ---------------------------------------------------------------------------
// UART_TYPE enum variant aliases (types.rs emits UART_TYPE_Xxx constants)
// ---------------------------------------------------------------------------
pub use crate::types::UART_TYPE_UartUnknown as UartUnknown;
pub use crate::types::UART_TYPE_Uart8250    as Uart8250;
pub use crate::types::UART_TYPE_Uart16450   as Uart16450;
pub use crate::types::UART_TYPE_Uart16550   as Uart16550;
pub use crate::types::UART_TYPE_Uart16550A  as Uart16550A;

// ---------------------------------------------------------------------------
// SERIAL_DEVICE_STATE enum variant aliases
// ---------------------------------------------------------------------------
pub use crate::types::SERIAL_DEVICE_STATE_dsStopped  as dsStopped;
pub use crate::types::SERIAL_DEVICE_STATE_dsStarted  as dsStarted;

// ---------------------------------------------------------------------------
// Kernel enum variant aliases — bindgen emits TYPENAME_Variant but C code
// uses just the bare Variant name.
// ---------------------------------------------------------------------------
pub use ntkrnl_sys::ddk::_WAIT_TYPE_WaitAny                            as WaitAny;
pub use ntkrnl_sys::ddk::_KINTERRUPT_MODE_Latched                      as Latched;
pub use ntkrnl_sys::ddk::_KINTERRUPT_MODE_LevelSensitive               as LevelSensitive;
pub use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_DelayedWorkQueue             as DelayedWorkQueue;
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_BusRelations            as BusRelations;
pub use ntkrnl_sys::ddk::_DEVICE_RELATION_TYPE_RemovalRelations        as RemovalRelations;
pub use ntkrnl_sys::ddk::_CM_SHARE_DISPOSITION_CmResourceShareShared   as CmResourceShareShared;
pub use ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FileStandardInformation as FileStandardInformation;
pub use ntkrnl_sys::ddk::_FILE_INFORMATION_CLASS_FilePositionInformation as FilePositionInformation;
pub use ntkrnl_sys::ddk::_EVENT_TYPE_NotificationEvent                  as NotificationEvent;

// ---------------------------------------------------------------------------
// PIO_WORKITEM_ROUTINE type alias
// i386 bindgen uses PIO_WORKITEM_ROUTINE; amd64/arm64 use IO_WORKITEM_ROUTINE.
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86")]
pub use ntkrnl_sys::ddk::PIO_WORKITEM_ROUTINE as IO_WORKITEM_ROUTINE;
#[cfg(not(target_arch = "x86"))]
pub use ntkrnl_sys::ddk::IO_WORKITEM_ROUTINE;

// ---------------------------------------------------------------------------
// IsNEC_98 — HAL/ntoskrnl data export: 0 = standard PC, 1 = NEC PC-98.
// ---------------------------------------------------------------------------
// IsNEC_98 is a PC-98 detection macro (NOT a real export) — always 0 on PC.
pub const IsNEC_98: ULONG = 0;

// ---------------------------------------------------------------------------
// KdComPortInUse — HAL/ntoskrnl data export: PUCHAR of the debug COM port.
// ---------------------------------------------------------------------------
// KdComPortInUse is NOT an ntoskrnl export and the C serial driver does not reference it
// (translation artifact) — null stub so the dead check resolves to "KD port not in use".
pub static mut KdComPortInUse: PUCHAR = core::ptr::null_mut();

// ---------------------------------------------------------------------------
// SERIAL — DPFLTR component id for DbgPrintEx calls.
// ---------------------------------------------------------------------------
pub const SERIAL: ULONG = 53; // DPFLTR_SERIAL_ID

// ---------------------------------------------------------------------------
// DPFLTR constants used module-scope in cleanup.rs / misc.rs.
// ---------------------------------------------------------------------------
pub const DPFLTR_SERIAL_ID:    ULONG = 39;
pub const DPFLTR_TRACE_LEVEL:  ULONG = 2;

// ---------------------------------------------------------------------------
// TRACE_ / WARN_ / ERR_ — ReactOS debug.h macros that expand to DbgPrintEx in
// debug builds and __noop (no symbol) in retail builds.  The c2rust translation
// turned them into extern fn calls.  We declare them as variadic externs here;
// the matching #[no_mangle] cdecl definitions live in lib.rs so the rlib is
// self-contained.  cdecl caller-cleans-up means a 2-arg receiver is safe even
// when callers pass extra variadic args — the caller pops them from the stack.
// ---------------------------------------------------------------------------
extern "C" {
    pub fn WARN_(component: ULONG, fmt: *const core::ffi::c_char, ...);
    pub fn ERR_(component: ULONG, fmt: *const core::ffi::c_char, ...);
}

// ---------------------------------------------------------------------------
// wchar_t — C wide character type (= u16 on Windows)
// ---------------------------------------------------------------------------
pub type wchar_t = u16;

// ---------------------------------------------------------------------------
// READ_PORT_UCHAR / WRITE_PORT_UCHAR — x86 port I/O FORCEINLINEs in wdm.h.
// Not exported symbols.  arm64 stub: serial is x86-only hardware, never runs
// on arm64 but must compile + link.
// ---------------------------------------------------------------------------
#[inline(always)]
pub unsafe fn READ_PORT_UCHAR(port: PUCHAR) -> UCHAR {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let val: u8;
        core::arch::asm!(
            "in al, dx",
            in("dx") port as u16,
            out("al") val,
            options(nostack, preserves_flags),
        );
        val
    }
    #[cfg(target_arch = "aarch64")]
    {
        let _ = port;
        0u8
    }
}

#[inline(always)]
pub unsafe fn WRITE_PORT_UCHAR(port: PUCHAR, val: UCHAR) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16,
            in("al") val,
            options(nostack, preserves_flags),
        );
    }
    #[cfg(target_arch = "aarch64")]
    {
        let _ = (port, val);
    }
}

// ---------------------------------------------------------------------------
// ULongToPtr — intsafe.h macro: zero-extends ULONG to pointer width.
// ---------------------------------------------------------------------------
#[inline(always)]
pub fn ULongToPtr(x: ULONG) -> PVOID {
    x as usize as PVOID
}

// ---------------------------------------------------------------------------
// RtlAssert — in retail builds the ASSERT macro expands to nothing (no symbol).
// Provide a no-op inline so translated ASSERT call sites link in all configs.
// ---------------------------------------------------------------------------
#[inline(always)]
pub unsafe fn RtlAssert(
    _failed_assertion: *const core::ffi::c_void,
    _file_name: *const core::ffi::c_void,
    _line_number: ULONG,
    _message: *mut core::ffi::c_char,
) {}

// ---------------------------------------------------------------------------
// IO_REMOVE_LOCK FORCEINLINE helpers — wdm.h FORCEINLINEs, not exported symbols.
//
// Faithful to the non-DBG (retail) wdm.h path:
//   IoInitializeRemoveLockEx: zero Common block, set IoCount=1 (Removed=FALSE).
//   IoAcquireRemoveLockEx:    InterlockedIncrement(&IoCount);
//                             if Removed → undo + return STATUS_DELETE_PENDING.
//   IoReleaseRemoveLockEx:    InterlockedDecrement(&IoCount)==0 && Removed
//                             → KeSetEvent(&RemoveEvent).
//
// The non-Ex wrappers (IoAcquireRemoveLock / IoReleaseRemoveLock) call through
// with sizeof(IO_REMOVE_LOCK) as the size guard.
// ---------------------------------------------------------------------------
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};

// All four IO_REMOVE_LOCK helpers accept PVOID for the lock pointer so call
// sites that cast addr_of_mut!(RemoveLock) as PVOID compile without changes.
#[inline]
pub unsafe fn IoInitializeRemoveLockEx(
    lock: PVOID,
    _allocate_tag: ULONG,
    _max_locked_minutes: ULONG,
    _high_watermark: ULONG,
    _remlock_size: ULONG,
) {
    let lock = lock as *mut IO_REMOVE_LOCK;
    let common = core::ptr::addr_of_mut!((*lock).Common);
    core::ptr::write_bytes(
        common as *mut u8,
        0u8,
        core::mem::size_of::<_IO_REMOVE_LOCK_COMMON_BLOCK>(),
    );
    (*lock).Common.IoCount = 1;
}

#[inline]
pub unsafe fn IoAcquireRemoveLockEx(
    lock: PVOID,
    _tag: PVOID,
    _file: *const core::ffi::c_char,
    _line: ULONG,
    _remlock_size: ULONG,
) -> NTSTATUS {
    use ntkrnl_sys::constants::STATUS_DELETE_PENDING;
    use ntkrnl_sys::base::STATUS_SUCCESS;
    let lock = lock as *mut IO_REMOVE_LOCK;
    InterlockedIncrement(core::ptr::addr_of_mut!((*lock).Common.IoCount));
    if (*lock).Common.Removed != 0 {
        if InterlockedDecrement(core::ptr::addr_of_mut!((*lock).Common.IoCount)) == 0 {
            ntkrnl_sys::ddk::KeSetEvent(
                core::ptr::addr_of_mut!((*lock).Common.RemoveEvent) as *mut KEVENT,
                0,
                0,
            );
        }
        STATUS_DELETE_PENDING
    } else {
        STATUS_SUCCESS
    }
}

#[inline]
pub unsafe fn IoReleaseRemoveLockEx(
    lock: PVOID,
    _tag: PVOID,
    _remlock_size: ULONG,
) {
    let lock = lock as *mut IO_REMOVE_LOCK;
    if InterlockedDecrement(core::ptr::addr_of_mut!((*lock).Common.IoCount)) == 0
        && (*lock).Common.Removed != 0
    {
        ntkrnl_sys::ddk::KeSetEvent(
            core::ptr::addr_of_mut!((*lock).Common.RemoveEvent) as *mut KEVENT,
            0,
            0,
        );
    }
}

/// IoAcquireRemoveLock — non-Ex wrapper.
#[inline]
pub unsafe fn IoAcquireRemoveLock(lock: PVOID, tag: PVOID) -> NTSTATUS {
    IoAcquireRemoveLockEx(
        lock, tag, core::ptr::null(), 0,
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}

/// IoReleaseRemoveLock — non-Ex wrapper.
#[inline]
pub unsafe fn IoReleaseRemoveLock(lock: PVOID, tag: PVOID) {
    IoReleaseRemoveLockEx(
        lock, tag,
        core::mem::size_of::<IO_REMOVE_LOCK>() as ULONG,
    )
}

// SR_FCR_CLEAR_RCVR / SR_FCR_CLEAR_XMIT are defined in types_i386.rs (from bindgen)

// KeAcquireSpinLockRaiseToDpc — exported (stdcall) only on amd64/arm64; on i386 it's the
// inline KfAcquireSpinLock path → implement via the ke wrapper (acquire + return old IRQL).
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn KeAcquireSpinLockRaiseToDpc(lock: PVOID) -> ntkrnl_sys::base::KIRQL {
    let mut old: ntkrnl_sys::base::KIRQL = 0;
    ntkrnl_sys::ke::KeAcquireSpinLock(lock as *mut usize, &mut old);
    old
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    pub fn KeAcquireSpinLockRaiseToDpc(lock: PVOID) -> ntkrnl_sys::base::KIRQL;
}

// ---- IOCTL_SERIAL_* (ntddser.h) ----
// Generated by tools/gen_ioctl_consts.py from sdk/include/psdk/ntddser.h.
// bindgen cannot emit function-like macros, so these CTL_CODE() IOCTLs
// can never come from the bindgen island (field manual §9.112a). Values
// are COMPUTED from the header, never hand-typed — see §9.113.
pub const IOCTL_SERIAL_CLEAR_STATS: u32 = 0x001b0090; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 36, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_CLR_DTR: u32 = 0x001b0028; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 10, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_CLR_RTS: u32 = 0x001b0034; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 13, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_CONFIG_SIZE: u32 = 0x001b0080; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 32, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_BAUD_RATE: u32 = 0x001b0050; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 20, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_CHARS: u32 = 0x001b0058; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 22, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_COMMSTATUS: u32 = 0x001b006c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 27, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_DTRRTS: u32 = 0x001b0078; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 30, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_HANDFLOW: u32 = 0x001b0060; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 24, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_LINE_CONTROL: u32 = 0x001b0054; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 21, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_MODEMSTATUS: u32 = 0x001b0068; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 26, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_MODEM_CONTROL: u32 = 0x001b0094; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 37, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_PROPERTIES: u32 = 0x001b0074; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 29, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_STATS: u32 = 0x001b008c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 35, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_TIMEOUTS: u32 = 0x001b0020; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 8, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_GET_WAIT_MASK: u32 = 0x001b0040; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 16, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_IMMEDIATE_CHAR: u32 = 0x001b0018; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 6, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_INTERNAL_BASIC_SETTINGS: u32 = 0x001b000c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 3, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_INTERNAL_CANCEL_WAIT_WAKE: u32 = 0x001b0008; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 2, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_INTERNAL_DO_WAIT_WAKE: u32 = 0x001b0004; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 1, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_INTERNAL_RESTORE_SETTINGS: u32 = 0x001b0010; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 4, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_LSRMST_INSERT: u32 = 0x001b007c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 31, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_PURGE: u32 = 0x001b004c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 19, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_RESET_DEVICE: u32 = 0x001b002c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 11, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_BAUD_RATE: u32 = 0x001b0004; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 1, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_BREAK_OFF: u32 = 0x001b0014; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 5, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_BREAK_ON: u32 = 0x001b0010; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 4, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_CHARS: u32 = 0x001b005c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 23, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_DTR: u32 = 0x001b0024; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 9, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_FIFO_CONTROL: u32 = 0x001b009c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 39, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_HANDFLOW: u32 = 0x001b0064; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 25, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_LINE_CONTROL: u32 = 0x001b000c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 3, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_MODEM_CONTROL: u32 = 0x001b0098; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 38, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_QUEUE_SIZE: u32 = 0x001b0008; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 2, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_RTS: u32 = 0x001b0030; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 12, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_TIMEOUTS: u32 = 0x001b001c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 7, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_WAIT_MASK: u32 = 0x001b0044; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 17, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_XOFF: u32 = 0x001b0038; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 14, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_SET_XON: u32 = 0x001b003c; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 15, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_WAIT_ON_MASK: u32 = 0x001b0048; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 18, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_SERIAL_XOFF_COUNTER: u32 = 0x001b0070; // CTL_CODE(FILE_DEVICE_SERIAL_PORT, 28, METHOD_BUFFERED, FILE_ANY_ACCESS)
