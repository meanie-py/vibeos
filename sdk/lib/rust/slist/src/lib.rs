//! Cross-platform Interlocked SList Functions — Rust implementation.
//!
//! Replaces architecture-specific assembly:
//! - `arm64/slist.S` (132 lines, LDXP/STXP)
//! - `amd64/slist.S` (450 lines, CMPXCHG16B, Header8+Header16)
//! - `i386/interlck.S` (154 lines, CMPXCHG8B)
//!
//! Architecture:
//! - `header.rs`: Pure bit-field pack/unpack (cfg per pointer width)
//! - `algorithm.rs`: Pure state transitions (Kani-verifiable, no unsafe)
//! - `atomics.rs`: Unified load/try_store/cancel (cfg per target_arch)
//! - `lib.rs`: FFI entry points (cfg per arch for calling convention + fault labels)

#![cfg_attr(not(any(test, kani)), no_std)]
#![allow(unexpected_cfgs)]
#![allow(non_snake_case)]

pub mod algorithm;
pub mod header;

// At opt-level >= 1, the compiler emits .eh_frame data that references this symbol.
// Since we use panic=abort, unwinding never occurs, but we still need the symbol.
#[cfg(not(any(test, kani)))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// Inline asm + FFI entry points — excluded from test/kani builds which use std
#[cfg(not(any(test, kani)))]
mod atomics;

// `RtlpUse16ByteSLists` (64-bit only): the kernel (ke/amd64/krnlinit) and ntdll
// (ldr/ldrinit) WRITE this global to record CPU CMPXCHG16B support. slist.c used
// to define it (`BOOLEAN RtlpUse16ByteSLists = -1;`); now that the C file is
// replaced, this crate owns the definition so those external writers still link.
// The crate itself always uses Header16 / 16-byte CAS (see the AMD64 note below),
// matching the existing always-16-byte choice for Push/Pop/Flush — so the value is
// informational for our implementation. Each module linking librtl gets its own
// copy, exactly as the C definition did (per-consumer static-lib semantics).
#[cfg(all(not(any(test, kani)), target_pointer_width = "64"))]
#[no_mangle]
pub static mut RtlpUse16ByteSLists: u8 = 0xFF;

// ============================================================================
// ARM64 FFI
// ============================================================================

#[cfg(all(not(any(test, kani)), target_arch = "aarch64"))]
mod ffi {
    use super::{algorithm, atomics, header};
    use core::ptr;

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPopEntrySList(list_head: *mut u8) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };

            let head_ptr = header::region_to_ptr(region);
            if head_ptr == 0 {
                unsafe { atomics::cancel() };
                return ptr::null_mut();
            }

            let next_of_head = unsafe { ptr::read(head_ptr as *const u64) };

            let result = match algorithm::pop_transition(alignment, region, next_of_head) {
                Some(r) => r,
                None => {
                    unsafe { atomics::cancel() };
                    return ptr::null_mut();
                }
            };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.popped as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPushEntrySList(
        list_head: *mut u8,
        list_entry: *mut u8,
    ) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };
            let result = algorithm::push_transition(alignment, region, list_entry as u64);

            // Write old head pointer into entry->Next
            unsafe { ptr::write(list_entry as *mut u64, result.old_head) };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedFlushSList(list_head: *mut u8) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };

            let result = match algorithm::flush_transition(alignment, region) {
                Some(r) => r,
                None => {
                    unsafe { atomics::cancel() };
                    return ptr::null_mut();
                }
            };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPushListSList(
        list_head: *mut u8,
        list: *mut u8,
        list_end: *mut u8,
        count: u32,
    ) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };
            let result =
                algorithm::push_list_transition(alignment, region, list as u64, count as u16);

            // Splice the chain tail onto the previous head.
            unsafe { ptr::write(list_end as *mut u64, result.old_head) };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInitializeSListHead(list_head: *mut u8) {
        // Zero both 64-bit halves (Alignment @0, Region @8). The C version also
        // raises STATUS_DATATYPE_MISALIGNMENT on a non-16-byte-aligned header;
        // that is a debug guard the kernel never trips (headers are always
        // 16-byte aligned), so we omit the RtlRaiseStatus dependency.
        unsafe {
            ptr::write_volatile(list_head as *mut u64, 0);
            ptr::write_volatile((list_head as *mut u64).add(1), 0);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFirstEntrySList(list_head: *const u8) -> *mut u8 {
        // Query (inherently racy, like the C `return SListHead->Region & ~0xF`).
        // We always use Header16, so the first entry is the masked region pointer.
        let (_alignment, region) = unsafe { atomics::load(list_head) };
        header::region_to_ptr(region) as *mut u8
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlQueryDepthSList(list_head: *mut u8) -> u16 {
        let (alignment, region) = unsafe { atomics::load(list_head) };
        header::unpack(alignment, region).depth
    }

    // Exp* aliases: zero-overhead branch trampolines for ntoskrnl internal use.
    core::arch::global_asm!(
        ".globl ExpInterlockedPopEntrySList",
        "ExpInterlockedPopEntrySList:",
        "  b RtlInterlockedPopEntrySList",
        "",
        ".globl ExpInterlockedPushEntrySList",
        "ExpInterlockedPushEntrySList:",
        "  b RtlInterlockedPushEntrySList",
        "",
        ".globl ExpInterlockedFlushSList",
        "ExpInterlockedFlushSList:",
        "  b RtlInterlockedFlushSList",
    );
}

// ============================================================================
// AMD64 FFI
// ============================================================================

#[cfg(all(not(any(test, kani)), target_arch = "x86_64"))]
mod ffi {
    use super::{algorithm, atomics, header};
    use core::ptr;

    // Always use Header16 — RtlpUse16ByteSLists defaults to TRUE on all modern CPUs.

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPopEntrySList(list_head: *mut u8) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };

            let head_ptr = header::region_to_ptr(region);
            if head_ptr == 0 {
                return ptr::null_mut();
            }

            // Read next pointer — this can page fault if another thread freed the entry.
            // The kernel's KiCheckForSListFault handles this on amd64 via the fault labels.
            let next_of_head = unsafe { ptr::read(head_ptr as *const u64) };

            let result = match algorithm::pop_transition(alignment, region, next_of_head) {
                Some(r) => r,
                None => return ptr::null_mut(),
            };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.popped as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPushEntrySList(
        list_head: *mut u8,
        list_entry: *mut u8,
    ) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };
            let result = algorithm::push_transition(alignment, region, list_entry as u64);

            // Write old head pointer into entry->Next
            unsafe { ptr::write(list_entry as *mut u64, result.old_head) };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedFlushSList(list_head: *mut u8) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };

            let result = match algorithm::flush_transition(alignment, region) {
                Some(r) => r,
                None => return ptr::null_mut(),
            };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInterlockedPushListSList(
        list_head: *mut u8,
        list: *mut u8,
        list_end: *mut u8,
        count: u32,
    ) -> *mut u8 {
        loop {
            let (alignment, region) = unsafe { atomics::load(list_head) };
            let result =
                algorithm::push_list_transition(alignment, region, list as u64, count as u16);

            // Splice the chain tail onto the previous head (ListEnd->Next = FirstEntry).
            unsafe { ptr::write(list_end as *mut u64, result.old_head) };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, region),
                    (result.new_alignment, result.new_region),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlInitializeSListHead(list_head: *mut u8) {
        // Zero both 64-bit halves (Alignment @0, Region @8). The C version also
        // raises STATUS_DATATYPE_MISALIGNMENT on a misaligned header — a debug
        // guard the kernel never trips, so we omit the RtlRaiseStatus dependency.
        unsafe {
            ptr::write_volatile(list_head as *mut u64, 0);
            ptr::write_volatile((list_head as *mut u64).add(1), 0);
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlFirstEntrySList(list_head: *const u8) -> *mut u8 {
        // Always Header16: first entry is the masked region pointer (NULL if empty).
        let (_alignment, region) = unsafe { atomics::load(list_head) };
        header::region_to_ptr(region) as *mut u8
    }

    #[no_mangle]
    pub unsafe extern "C" fn RtlQueryDepthSList(list_head: *mut u8) -> u16 {
        let (alignment, region) = unsafe { atomics::load(list_head) };
        header::unpack(alignment, region).depth
    }

    // Exp* aliases + fault label symbols for amd64.
    // ntdll.spec exports ExpInterlockedPopEntrySList{Resume,Fault,End} and 16-byte variants.
    // ntoskrnl.spec forwards Exp* to Rtl* via the spec file, not via asm aliases.
    // The fault labels are exported from ntdll for compatibility but no kernel trap
    // handler references them on amd64 (KiCheckForSListFault is i386-only).
    core::arch::global_asm!(
        // Exp* trampolines
        ".globl ExpInterlockedPopEntrySList",
        "ExpInterlockedPopEntrySList:",
        "  jmp RtlInterlockedPopEntrySList",
        "",
        ".globl ExpInterlockedPushEntrySList",
        "ExpInterlockedPushEntrySList:",
        "  jmp RtlInterlockedPushEntrySList",
        "",
        ".globl ExpInterlockedFlushSList",
        "ExpInterlockedFlushSList:",
        "  jmp RtlInterlockedFlushSList",
        "",
        // Fault labels — empty symbols for ntdll.spec exports.
        // Header8 variants (not used, we always use Header16)
        ".globl ExpInterlockedPopEntrySListResume",
        "ExpInterlockedPopEntrySListResume:",
        ".globl ExpInterlockedPopEntrySListFault",
        "ExpInterlockedPopEntrySListFault:",
        ".globl ExpInterlockedPopEntrySListEnd",
        "ExpInterlockedPopEntrySListEnd:",
        // Header16 variants
        ".globl ExpInterlockedPopEntrySListResume16",
        "ExpInterlockedPopEntrySListResume16:",
        ".globl ExpInterlockedPopEntrySListFault16",
        "ExpInterlockedPopEntrySListFault16:",
        ".globl ExpInterlockedPopEntrySListEnd16",
        "ExpInterlockedPopEntrySListEnd16:",
    );
}

// ============================================================================
// i386 FFI
// ============================================================================

#[cfg(all(not(any(test, kani)), target_arch = "x86"))]
mod ffi {
    use super::{algorithm, atomics, header};
    use core::ptr;

    // Pop MUST be in global_asm! because KiCheckForSListFault (traphdlr.c:1221)
    // checks if the page fault EIP matches ExpInterlockedPopEntrySListFault and
    // validates the exact instruction bytes: 0x8B 0x18 = `mov ebx, [eax]`.
    //
    // Layout matches sdk/lib/rtl/i386/interlck.S exactly:
    //   push ebx; push ebp; mov ebp,[esp+12]
    //   Resume: mov eax,[ebp]; mov edx,[ebp+4]; or eax,eax; jz empty
    //   lea ecx,[edx-1]
    //   Fault: mov ebx,[eax]          <-- 0x8B 0x18
    //   End:   lock cmpxchg8b [ebp]; jnz Resume
    //   empty: pop ebp; pop ebx; ret 4
    core::arch::global_asm!(
        // RTL Pop (stdcall, 1 arg)
        ".globl _ExpInterlockedPopEntrySListResume@0",
        ".globl _ExpInterlockedPopEntrySListFault@0",
        ".globl _ExpInterlockedPopEntrySListEnd@0",
        ".globl _RtlInterlockedPopEntrySList@4",
        "_RtlInterlockedPopEntrySList@4:",
        "  push ebx",
        "  push ebp",
        "  mov ebp, [esp + 12]",
        "_ExpInterlockedPopEntrySListResume@0:",
        "  mov eax, [ebp]",
        "  mov edx, [ebp + 4]",
        "  or eax, eax",
        "  jz 2f",
        "  lea ecx, [edx - 1]",
        "_ExpInterlockedPopEntrySListFault@0:",
        "  mov ebx, [eax]",
        "_ExpInterlockedPopEntrySListEnd@0:",
        "  lock cmpxchg8b qword ptr [ebp]",
        "  jnz _ExpInterlockedPopEntrySListResume@0",
        "2:",
        "  pop ebp",
        "  pop ebx",
        "  ret 4",
    );

    // Push — Rust implementation with stdcall calling convention.
    // i386 NTAPI = __stdcall. The compiler decorates as _Name@N on i686-pc-windows-gnu.
    #[no_mangle]
    pub unsafe extern "stdcall" fn RtlInterlockedPushEntrySList(
        list_head: *mut u8,
        list_entry: *mut u8,
    ) -> *mut u8 {
        loop {
            let (alignment, _region) = unsafe { atomics::load(list_head) };
            let result = algorithm::push_transition(alignment, 0, list_entry as u64);

            // Write old head pointer into entry->Next (32-bit pointer)
            unsafe { ptr::write(list_entry as *mut u32, result.old_head as u32) };

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, 0),
                    (result.new_alignment, 0),
                )
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    // Flush — Rust implementation with stdcall calling convention.
    // On i386, flush preserves the sequence counter (only zeros Next + Depth).
    // This matches the original asm: `mov ecx, edx; xor cx, cx` (keeps Sequence in high 16 bits).
    #[no_mangle]
    pub unsafe extern "stdcall" fn RtlInterlockedFlushSList(list_head: *mut u8) -> *mut u8 {
        loop {
            let (alignment, _region) = unsafe { atomics::load(list_head) };

            let old = header::unpack(alignment, 0);
            let old_head = header::region_to_ptr(old.next_entry);

            if old_head == 0 {
                return ptr::null_mut();
            }

            // Preserve sequence, zero depth and next
            let new_alignment = alignment & 0xFFFF_0000_0000_0000;

            if unsafe {
                atomics::try_store(
                    list_head,
                    (alignment, 0),
                    (new_alignment, 0),
                )
            } {
                return old_head as *mut u8;
            }
        }
    }

    // PushList — FASTCALL on i386 (slist.c declares it `FASTCALL`, not `NTAPI`);
    // the decoration is `@RtlInterlockedPushListSList@16`. Matches the i386 `#else`
    // branch of slist.c (InterlockedCompareExchange64 on the 8-byte header).
    #[no_mangle]
    pub unsafe extern "fastcall" fn RtlInterlockedPushListSList(
        list_head: *mut u8,
        list: *mut u8,
        list_end: *mut u8,
        count: u32,
    ) -> *mut u8 {
        loop {
            let (alignment, _region) = unsafe { atomics::load(list_head) };
            let result = algorithm::push_list_transition(alignment, 0, list as u64, count as u16);

            // Splice the chain tail onto the previous head (32-bit pointer).
            unsafe { ptr::write(list_end as *mut u32, result.old_head as u32) };

            if unsafe {
                atomics::try_store(list_head, (alignment, 0), (result.new_alignment, 0))
            } {
                return result.old_head as *mut u8;
            }
        }
    }

    // Init — stdcall. On i386 the header is 8 bytes; just zero it.
    #[no_mangle]
    pub unsafe extern "stdcall" fn RtlInitializeSListHead(list_head: *mut u8) {
        unsafe { ptr::write_volatile(list_head as *mut u64, 0) };
    }

    // FirstEntry — stdcall. C returns `SListHead->Next.Next` (low 32 bits).
    #[no_mangle]
    pub unsafe extern "stdcall" fn RtlFirstEntrySList(list_head: *const u8) -> *mut u8 {
        let (alignment, _region) = unsafe { atomics::load(list_head) };
        header::region_to_ptr(header::unpack(alignment, 0).next_entry) as *mut u8
    }

    // QueryDepth — stdcall. C returns `SListHead->Depth` (bits 47:32).
    #[no_mangle]
    pub unsafe extern "stdcall" fn RtlQueryDepthSList(list_head: *mut u8) -> u16 {
        let (alignment, _region) = unsafe { atomics::load(list_head) };
        header::unpack(alignment, 0).depth
    }

    // Exp* aliases for i386: Pop aliases are defined inline in the global_asm above
    // (the Resume/Fault/End labels). For Push and Flush, the kernel uses
    // fastinterlck_asm.S (separate @Ex* symbols), so no Exp aliases needed in RTL.
}

// ============================================================================
// Panic handler (no_std)
// ============================================================================
