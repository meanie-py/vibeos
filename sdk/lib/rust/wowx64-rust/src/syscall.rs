//! Guest system calls. Almost all are forwarded by register shuffle alone —
//! amd64 and arm64 share one service table and are both LLP64, so the
//! guest's number and arguments ARE the native ones (see the crate docs). The
//! exceptions are the services whose meaning depends on which side of the CPU
//! layer the caller is on, and each is here for a reason it states:
//!
//!   * CONTEXT in or out (`NtContinue`, `Nt{Get,Set}ContextThread`,
//!     `NtRaiseException`): the guest's CONTEXT is the x86-64 one and the
//!     kernel's is arm64. These go to the CPU layer (and `except.rs`), never
//!     the kernel.
//!   * paths (`NtOpenFile`, `NtCreateFile`, `NtQuery*AttributesFile`): the
//!     guest's `system32` is `sysamd64`, the WoW64 filesystem redirector one
//!     width up. `NtOpenDirectoryObject(\KnownDlls)`: those sections are the
//!     native DLLs, so the guest loader must not find them.
//!   * threads (`NtCreateThread`): its CONTEXT is x86-64 too, and the new
//!     thread must start in wowx64.dll, not at a guest address (`thread.rs`).
//!   * address-space changes (`Nt{Allocate,Protect,Free}VirtualMemory`,
//!     `Nt{Map,Unmap}ViewOfSection`, `NtFlushInstructionCache`): reported to
//!     the CPU layer so its code cache and image tracking stay true.

use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicU32, Ordering};

use ntdll_sys::nt::*;

use crate::contract::*;
use crate::host::{Cpu, NativeSyscall, CURRENT_PROCESS, CURRENT_THREAD};
use crate::image::GUEST_SYSDIR;

include!(concat!(env!("OUT_DIR"), "/services.rs"));

const STATUS_OBJECT_NAME_NOT_FOUND: u64 = 0xC000_0034;
const STATUS_NOT_IMPLEMENTED: u64 = 0xC000_0002;
const MEM_IMAGE: u32 = 0x0100_0000;

/// Guest syscalls traced to serial before going quiet. Bounded so a real
/// guest (thousands of syscalls) cannot flood the UART — whose cost on real
/// hardware is ~90ms/KB inside the trap path — yet the first ones, which are
/// where a bring-up breaks, are always visible.
const TRACED_SYSCALLS: u32 = 16;
static SyscallCount: AtomicU32 = AtomicU32::new(0);

/// `WOWX64_HOST_SERVICES::SystemService`: one guest `syscall`. The CPU layer
/// has gathered the arguments in guest order (r10, rdx, r8, r9, then
/// [rsp+0x28...]).
pub unsafe extern "C" fn WowX64SystemService(Service: u32, GuestArgs: *const u64) -> u64 {
    let mut Args = [0u64; WOWX64_SERVICE_ARGS];
    core::ptr::copy_nonoverlapping(GuestArgs, Args.as_mut_ptr(), WOWX64_SERVICE_ARGS);

    let Ret = Dispatch(Service, &mut Args);

    let N = SyscallCount.fetch_add(1, Ordering::Relaxed);
    if N < TRACED_SYSCALLS {
        crate::trace!(
            "guest syscall #%u: service %u (%p, %p, %p, %p) -> %p",
            N + 1,
            Service,
            Args[0] as *const c_void,
            Args[1] as *const c_void,
            Args[2] as *const c_void,
            Args[3] as *const c_void,
            Ret as *const c_void,
        );
        if N + 1 == TRACED_SYSCALLS {
            crate::trace!("(syscall trace capped at %u)", TRACED_SYSCALLS);
        }
    }
    Ret
}

/// Storage for a redirected path, alive across the native call that reads it.
struct Shadow {
    Oa: OBJECT_ATTRIBUTES,
    Name: UNICODE_STRING,
    Buffer: [u16; 512],
}

unsafe fn Dispatch(Service: u32, Args: &mut [u64; WOWX64_SERVICE_ARGS]) -> u64 {
    let Cpu = Cpu();
    let mut Shadow: Shadow = core::mem::zeroed();

    match Service {
        SVC_NtContinue => {
            // The guest's KiUserApcDispatcher / exception dispatcher resuming a
            // saved x86-64 context. The CPU layer installs it; the handler sees
            // rip changed and does not overwrite it with this call's return.
            let Status = (Cpu.SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Args[0] as *const WowX64Context);
            return Status as u32 as u64;
        }
        SVC_NtRaiseException => {
            return crate::except::RaiseException(
                Args[0] as *const crate::except::GuestExceptionRecord,
                Args[1] as *const WowX64Context,
                Args[2] as u8 != 0,
            );
        }
        SVC_NtCreateThread if IsSelf(Args[3]) => {
            return crate::thread::CreateThread(Args);
        }
        SVC_NtGetContextThread | SVC_NtSetContextThread => {
            if !IsCurrentThread(Args[0] as HANDLE) {
                // Forwarding would have the kernel read or write an ARM64
                // CONTEXT through the guest's x86-64 one. Another thread's
                // guest state lives in that thread's CPU layer (owed: threads).
                crate::trace!("Nt%sContextThread on another thread: not supported yet",
                    if Service == SVC_NtGetContextThread { b"Get\0".as_ptr() } else { b"Set\0".as_ptr() });
                return STATUS_NOT_IMPLEMENTED;
            }
            let Ctx = Args[1] as *mut WowX64Context;
            let Status = if Service == SVC_NtGetContextThread {
                (Cpu.GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Ctx)
            } else {
                (Cpu.SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Ctx)
            };
            return Status as u32 as u64;
        }
        SVC_NtOpenFile | SVC_NtCreateFile | SVC_NtOpenDirectoryObject => {
            let Oa = Args[2] as *const OBJECT_ATTRIBUTES;
            if Service == SVC_NtOpenDirectoryObject {
                if IsKnownDlls(Oa) {
                    return STATUS_OBJECT_NAME_NOT_FOUND;
                }
            } else if Redirect(Oa, &mut Shadow) {
                Args[2] = &Shadow.Oa as *const _ as u64;
            }
        }
        SVC_NtQueryAttributesFile | SVC_NtQueryFullAttributesFile => {
            if Redirect(Args[0] as *const OBJECT_ATTRIBUTES, &mut Shadow) {
                Args[0] = &Shadow.Oa as *const _ as u64;
            }
        }
        SVC_NtAllocateVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[3] as *mut SIZE_T;
            let (Type, Prot) = (Args[4] as ULONG, Args[5] as ULONG);
            (Cpu.NotifyMemoryAlloc)(*Base, *Size, Type, Prot, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryAlloc)(*Base, *Size, Type, Prot, 1, Status as u32);
            return Status;
        }
        SVC_NtProtectVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[2] as *mut SIZE_T;
            let Prot = Args[3] as ULONG;
            (Cpu.NotifyMemoryProtect)(*Base, *Size, Prot, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryProtect)(*Base, *Size, Prot, 1, Status as u32);
            return Status;
        }
        SVC_NtFreeVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[2] as *mut SIZE_T;
            let Type = Args[3] as ULONG;
            (Cpu.NotifyMemoryFree)(*Base, *Size, Type, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryFree)(*Base, *Size, Type, 1, Status as u32);
            return Status;
        }
        SVC_NtMapViewOfSection if IsSelf(Args[1]) => {
            let Status = NativeSyscall(Service, Args.as_ptr());
            if (Status as NTSTATUS) >= 0 {
                let Base = *(Args[2] as *const PVOID);
                let Size = *(Args[6] as *const SIZE_T);
                // Only an IMAGE view is a module to track; a data view of a PE
                // file has its sections at file offsets and must not be parsed.
                if IsImageView(Base) {
                    (Cpu.NotifyMapViewOfSection)(null_mut(), Base, null_mut(), Size, Args[8] as ULONG, Args[9] as ULONG);
                }
            }
            return Status;
        }
        SVC_NtUnmapViewOfSection if IsSelf(Args[0]) => {
            let Base = Args[1] as PVOID;
            (Cpu.NotifyUnmapViewOfSection)(Base, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyUnmapViewOfSection)(Base, 1, Status as u32);
            return Status;
        }
        SVC_NtFlushInstructionCache if IsSelf(Args[0]) => {
            (Cpu.FlushInstructionCache2)(Args[1] as *const c_void, Args[2] as SIZE_T);
        }
        _ => {}
    }
    NativeSyscall(Service, Args.as_ptr())
}

fn IsSelf(Process: u64) -> bool {
    Process as HANDLE == CURRENT_PROCESS
}

unsafe fn IsCurrentThread(Thread: HANDLE) -> bool {
    if Thread == CURRENT_THREAD {
        return true;
    }
    let mut Info: THREAD_BASIC_INFORMATION = core::mem::zeroed();
    if NtQueryInformationThread(
        Thread,
        ThreadBasicInformation,
        &mut Info as *mut _ as PVOID,
        core::mem::size_of::<THREAD_BASIC_INFORMATION>() as ULONG,
        null_mut(),
    ) < 0
    {
        return false;
    }
    Info.ClientId.UniqueThread == (*NtCurrentTeb()).ClientId.UniqueThread
}

unsafe fn IsImageView(Base: PVOID) -> bool {
    let mut Mbi: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    NtQueryVirtualMemory(
        CURRENT_PROCESS,
        Base,
        MemoryBasicInformation,
        &mut Mbi as *mut _ as PVOID,
        core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as SIZE_T,
        null_mut(),
    ) >= 0
        && Mbi.Type == MEM_IMAGE
}

fn Lower(c: u16) -> u16 {
    if (b'A' as u16..=b'Z' as u16).contains(&c) { c + 32 } else { c }
}

/// The object name as a slice, or None for a relative open or no name.
unsafe fn ObjectName<'a>(Oa: *const OBJECT_ATTRIBUTES) -> Option<&'a [u16]> {
    if Oa.is_null() || !(*Oa).RootDirectory.is_null() || (*Oa).ObjectName.is_null() {
        return None;
    }
    let Name = (*Oa).ObjectName;
    if (*Name).Buffer.is_null() {
        return None;
    }
    Some(core::slice::from_raw_parts((*Name).Buffer, (*Name).Length as usize / 2))
}

unsafe fn IsKnownDlls(Oa: *const OBJECT_ATTRIBUTES) -> bool {
    const KNOWN: &[u8] = b"\\knowndlls";
    match ObjectName(Oa) {
        Some(N) => N.len() == KNOWN.len() && N.iter().zip(KNOWN).all(|(&a, &b)| Lower(a) == b as u16),
        None => false,
    }
}

/// `\system32\` -> `\sysamd64\` (same length) into `Shadow`, which the caller
/// passes to the kernel instead. Guest memory is never written.
unsafe fn Redirect(Oa: *const OBJECT_ATTRIBUTES, Shadow: &mut Shadow) -> bool {
    const SYS32: &[u8; 10] = b"\\system32\\";
    let Name = match ObjectName(Oa) {
        Some(N) => N,
        None => return false,
    };
    let Hit = match Name.windows(10).position(|w| w.iter().zip(SYS32).all(|(&a, &b)| Lower(a) == b as u16)) {
        Some(i) => i,
        None => return false,
    };
    if Name.len() > Shadow.Buffer.len() {
        // Loud, not silent: an unredirected system32 open hands an x86-64 guest
        // the NATIVE file and reports success.
        crate::trace!("path redirect skipped: %u chars > %u", Name.len() as u32, Shadow.Buffer.len() as u32);
        return false;
    }
    Shadow.Buffer[..Name.len()].copy_from_slice(Name);
    for (i, &c) in GUEST_SYSDIR.iter().enumerate() {
        Shadow.Buffer[Hit + i] = c as u16;
    }
    Shadow.Oa = *Oa;
    Shadow.Name = *(*Oa).ObjectName;
    Shadow.Name.Buffer = Shadow.Buffer.as_mut_ptr();
    Shadow.Oa.ObjectName = &mut Shadow.Name;
    true
}
