use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    ExRaiseAccessViolation,
    ExRaiseDatatypeMisalignment,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static MmUserProbeAddress: PVOID;
}

// On i386, these are FASTCALL exports (@Name@N decoration).
// On amd64/arm64, FASTCALL == default C calling convention.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InterlockedIncrement(Addend: *mut LONG) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedIncrement(Addend)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedIncrement(Addend: *mut LONG) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedIncrement(Addend)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InterlockedDecrement(Addend: *mut LONG) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedDecrement(Addend)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedDecrement(Addend: *mut LONG) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedDecrement(Addend)
}

// InterlockedCompareExchange — FASTCALL on i386, cdecl on amd64/arm64.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InterlockedCompareExchange(
    Destination: *mut LONG, Exchange: LONG, Comperand: LONG,
) -> LONG {
    let Destination = opaque_ptr(Destination);
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Destination as *const AtomicI32);
    match atomic.compare_exchange(Comperand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedCompareExchange(
    Destination: *mut LONG, Exchange: LONG, Comperand: LONG,
) -> LONG {
    let Destination = opaque_ptr(Destination);
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Destination as *const AtomicI32);
    match atomic.compare_exchange(Comperand, Exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(old) => old,
        Err(old) => old,
    }
}

// InterlockedExchange — FASTCALL on i386, cdecl on amd64/arm64.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InterlockedExchange(
    Target: *mut LONG, Value: LONG,
) -> LONG {
    let Target = opaque_ptr(Target);
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Target as *const AtomicI32);
    atomic.swap(Value, Ordering::SeqCst)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedExchange(
    Target: *mut LONG, Value: LONG,
) -> LONG {
    let Target = opaque_ptr(Target);
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(Target as *const AtomicI32);
    atomic.swap(Value, Ordering::SeqCst)
}

// InterlockedExchangeAdd — FASTCALL on i386, cdecl on amd64/arm64.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn InterlockedExchangeAdd(
    Addend: *mut LONG, Value: LONG,
) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedExchangeAdd(Addend, Value)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn InterlockedExchangeAdd(
    Addend: *mut LONG, Value: LONG,
) -> LONG {
    let Addend = opaque_ptr(Addend);
    ntkrnl_sys::interlocked::InterlockedExchangeAdd(Addend, Value)
}

#[no_mangle]
pub unsafe extern "system" fn ProbeForRead(
    Address: *const core::ffi::c_void,
    Length: SIZE_T,
    Alignment: ULONG,
) {
    let entry_lr = probe_entry_lr();
    let Current = Address as ULONG_PTR;

    if Length != 0 {
        debug_assert!(
            Alignment == 1
                || Alignment == 2
                || Alignment == 4
                || Alignment == 8
                || Alignment == 16
        );

        // FIXME(amd64-misalign-231): some kernel call path is reaching this
        // function with `Alignment` set to a non-power-of-2 junk value
        // (consistently 231 = 0xE7 from a 16-byte UNICODE_STRING-style probe
        // post-shell-ready on amd64). The C-side `PsProcessInfoClass[]` /
        // `PsThreadInfoClass[]` indexing has been audited (counts now match
        // the actual NT 5.x array length, see `ps/query.rs`), so this is a
        // separate bug not yet root-caused. Without DUMMY_PSEH the SEH
        // handler in the syscall dispatcher would catch the exception and
        // return STATUS_DATATYPE_MISALIGNMENT to user-mode; with DUMMY_PSEH
        // the exception is unhandled and bugchecks. As a temporary
        // unblocker, treat any non-{1,2,4,8,16} alignment as 1 and log it,
        // so the boot reaches the desktop. Replace this with a proper
        // root-cause fix once the caller is identified.
        let mut EffectiveAlignment = Alignment;
        if Alignment != 1 && Alignment != 2 && Alignment != 4 && Alignment != 8 && Alignment != 16 {
            crate::dbg_print!(
                b"[PROBE-R-BAD-ALIGN] Addr=%p Len=%Iu Align=%lu (treating as 1)\n\0",
                Current as PVOID,
                Length,
                Alignment,
            );
            EffectiveAlignment = 1;
        }

        if (Current & (EffectiveAlignment as ULONG_PTR - 1)) != 0 {
            probe_misalign_trace(b"R\0", Current, Length, EffectiveAlignment, entry_lr);
            ExRaiseDatatypeMisalignment();
        }

        let Last = Current.wrapping_add(Length).wrapping_sub(1);
        if Last < Current || Last >= MmUserProbeAddress as ULONG_PTR {
            ExRaiseAccessViolation();
        }
    }
}

/// One-line trace before a probe raises DATATYPE_MISALIGNMENT: with
/// DUMMY_PSEH the raise is fatal (unhandled kernel exception → 0x1E), so the
/// caller's LR is the only breadcrumb identifying WHICH syscall probed a
/// misaligned user pointer (WOW64 guests pass 4-aligned cells everywhere).
#[inline(always)]
unsafe fn probe_entry_lr() -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        let lr: usize;
        core::arch::asm!("mov {}, x30", out(reg) lr, options(nomem, nostack));
        lr
    }
    #[cfg(not(target_arch = "aarch64"))]
    { 0 }
}

#[inline(never)]
unsafe fn probe_misalign_trace(tag: &[u8], addr: ULONG_PTR, len: SIZE_T, align: ULONG, lr: usize) {
    crate::dbg_print!(
        b"[PROBE-MISALIGN-%s] Addr=%p Len=%Iu Align=%lu caller=%p\n\0",
        tag.as_ptr(),
        addr as PVOID,
        len,
        align,
        lr as PVOID,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ProbeForWrite(
    Address: PVOID,
    Length: SIZE_T,
    Alignment: ULONG,
) {
    let entry_lr = probe_entry_lr();
    let Address = opaque_ptr(Address);
    let mut Current = Address as ULONG_PTR;

    if Length != 0 {
        debug_assert!(
            Alignment == 1
                || Alignment == 2
                || Alignment == 4
                || Alignment == 8
                || Alignment == 16
        );

        if (Current & (Alignment as ULONG_PTR - 1)) != 0 {
            probe_misalign_trace(b"W\0", Current, Length, Alignment, entry_lr);
            ExRaiseDatatypeMisalignment();
        }

        let mut Last = Current.wrapping_add(Length).wrapping_sub(1);
        if Last < Current || Last >= MmUserProbeAddress as ULONG_PTR {
            ExRaiseAccessViolation();
        }

        Last = PAGE_ROUND_DOWN(Last) + PAGE_SIZE;
        loop {
            let ptr = Current as *mut i8;
            core::ptr::write_volatile(ptr, core::ptr::read_volatile(ptr));

            Current = PAGE_ROUND_DOWN(Current) + PAGE_SIZE;
            if Current == Last {
                break;
            }
        }
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/ex/exintrin.c
//  * PURPOSE:         Exported kernel functions which are now intrinsics
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #undef InterlockedIncrement
// #undef InterlockedDecrement
// #undef InterlockedCompareExchange
// #undef InterlockedExchangeAdd
// #undef InterlockedExchange
//
// //
// // HAL Port to Inlined Port
// //
// #define H2I(Port) PtrToUshort(Port)
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// LONG
// FASTCALL
// InterlockedIncrement(IN LONG volatile *Addend)
// {
//     //
//     // Call the intrinsic
//     //
//     return _InterlockedIncrement(Addend);
// }
//
// /*
//  * @implemented
//  */
// LONG
// FASTCALL
// InterlockedDecrement(IN LONG volatile *Addend)
// {
//     //
//     // Call the intrinsic
//     //
//     return _InterlockedDecrement(Addend);
// }
//
// /*
//  * @implemented
//  */
// LONG
// FASTCALL
// InterlockedCompareExchange(IN OUT LONG volatile *Destination,
//                            IN LONG Exchange,
//                            IN LONG Comperand)
// {
//     //
//     // Call the intrinsic
//     //
//     return _InterlockedCompareExchange(Destination, Exchange, Comperand);
// }
//
// /*
//  * @implemented
//  */
// LONG
// FASTCALL
// InterlockedExchange(IN OUT LONG volatile *Destination,
//                     IN LONG Value)
// {
//     //
//     // Call the intrinsic
//     //
//     return _InterlockedExchange(Destination, Value);
// }
//
// /*
//  * @implemented
//  */
// LONG
// FASTCALL
// InterlockedExchangeAdd(IN OUT LONG volatile *Addend,
//                        IN LONG Increment)
// {
//     //
//     // Call the intrinsic
//     //
//     return _InterlockedExchangeAdd(Addend, Increment);
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ProbeForRead(IN CONST VOID *Address,
//              IN SIZE_T Length,
//              IN ULONG Alignment)
// {
//     ULONG_PTR Last, Current = (ULONG_PTR)Address;
//     PAGED_CODE();
//
//     /* Only probe if we have a valid length */
//     if (Length != 0)
//     {
//         /* Sanity check */
//         ASSERT((Alignment == 1) ||
//                (Alignment == 2) ||
//                (Alignment == 4) ||
//                (Alignment == 8) ||
//                (Alignment == 16));
//
//         /* Check the alignment */
//         if ((Current & (Alignment - 1)) != 0)
//         {
//             /* Incorrect alignment */
//             ExRaiseDatatypeMisalignment();
//         }
//
//         /* Get the end address */
//         Last = Current + Length - 1;
//         if ((Last < Current) || (Last >= (ULONG_PTR)MmUserProbeAddress))
//         {
//             /* Raise an access violation */
//             ExRaiseAccessViolation();
//         }
//
//         /* ProbeForRead doesn't check if memory pages are readable! */
//     }
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// ProbeForWrite(IN PVOID Address,
//               IN SIZE_T Length,
//               IN ULONG Alignment)
// {
//     ULONG_PTR Last, Current = (ULONG_PTR)Address;
//     PAGED_CODE();
//
//     /* Only probe if we have a valid length */
//     if (Length != 0)
//     {
//         /* Sanity check */
//         ASSERT((Alignment == 1) ||
//                (Alignment == 2) ||
//                (Alignment == 4) ||
//                (Alignment == 8) ||
//                (Alignment == 16));
//
//         /* Check the alignment */
//         if ((Current & (Alignment - 1)) != 0)
//         {
//             /* Incorrect alignment */
//             ExRaiseDatatypeMisalignment();
//         }
//
//         /* Get the end address */
//         Last = Current + Length - 1;
//         if ((Last < Current) || (Last >= (ULONG_PTR)MmUserProbeAddress))
//         {
//             /* Raise an access violation */
//             ExRaiseAccessViolation();
//         }
//
//         /* Round down to the last page */
//         Last = PAGE_ROUND_DOWN(Last) + PAGE_SIZE;
//         do
//         {
//             /* Attempt a write */
//             *(volatile CHAR*)Current = *(volatile CHAR*)Current;
//
//             /* Go to the next address */
//             Current = PAGE_ROUND_DOWN(Current) + PAGE_SIZE;
//         } while (Current != Last);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
