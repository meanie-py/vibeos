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
//!   * `NtCallbackReturn`: the end of a win32k callback run in the guest
//!     (`callback.rs`), which must restore the guest state the callback
//!     interrupted before the kernel unwinds the callout.
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

/// Generated from sysfuncs.lst / w32ksvc64.h (build.rs). Matched on by PATH
/// (`svc::SVC_...`) below, never by bare name: an undefined bare constant in a
/// match pattern is a catch-all binding, and a path cannot be one.
pub mod svc {
    include!(concat!(env!("OUT_DIR"), "/services.rs"));
}
use svc::{SERVICE_ARGC_NT, SERVICE_ARGC_W32K};

const STATUS_OBJECT_NAME_NOT_FOUND: u64 = 0xC000_0034;
const STATUS_NOT_IMPLEMENTED: u64 = 0xC000_0002;
const MEM_IMAGE: u32 = 0x0100_0000;

/// Guest syscalls traced to serial before going quiet. Bounded so a real
/// guest (thousands of syscalls) cannot flood the UART — whose cost on real
/// hardware is ~90ms/KB inside the trap path — yet the first ones, which are
/// where a bring-up breaks, are always visible.
const TRACED_SYSCALLS: u32 = 16;
static SyscallCount: AtomicU32 = AtomicU32::new(0);

/// Most arguments any service takes: `NtAccessCheckByTypeResultListAndAudit
/// AlarmByHandle` has 17. `NativeSyscall` passes all of them (8 registers + 9
/// stack words).
pub const WOWX64_SERVICE_ARGS: usize = 17;

/// How many arguments guest service `Service` takes, or None for a number in
/// neither table (forwarded with its register arguments only; the kernel
/// rejects it).
fn ArgCount(Service: u32) -> Option<usize> {
    let Index = (Service & 0xFFF) as usize;
    match Service >> 12 {
        0 => SERVICE_ARGC_NT.get(Index).map(|&n| n as usize),
        1 => SERVICE_ARGC_W32K.get(Index).map(|&n| n as usize),
        _ => None,
    }
}

/// `WOWX64_HOST_SERVICES::SystemService`: one guest `syscall`. The register
/// arguments come from the CPU layer; the stack ones are read here, exactly as
/// many as the service takes, from the guest stack at rsp+0x28 (the stub's
/// return address at [rsp], then the caller's 32-byte home area).
pub unsafe extern "C" fn WowX64SystemService(Service: u32, RegisterArgs: *const u64, GuestRsp: u64) -> u64 {
    let mut Args = [0u64; WOWX64_SERVICE_ARGS];
    core::ptr::copy_nonoverlapping(RegisterArgs, Args.as_mut_ptr(), WOWX64_REGISTER_ARGS);
    let Count = ArgCount(Service).unwrap_or(0).min(WOWX64_SERVICE_ARGS);
    for i in WOWX64_REGISTER_ARGS..Count {
        Args[i] = *((GuestRsp + 0x28 + 8 * (i - WOWX64_REGISTER_ARGS) as u64) as *const u64);
    }

    // Published for RemoteGuestContext, and restored rather than cleared: a
    // win32k callback runs guest code, and its syscalls, inside this one.
    let Rec = crate::thread::Record();
    let Saved = ((*Rec).SyscallRsp, (*Rec).SyscallService);
    (*Rec).SyscallService = Service as u64;
    core::ptr::write_volatile(&mut (*Rec).SyscallRsp, GuestRsp);
    let Ret = Dispatch(Service, &mut Args);
    core::ptr::write_volatile(&mut (*Rec).SyscallRsp, Saved.0);
    (*Rec).SyscallService = Saved.1;

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
        svc::SVC_NtContinue => {
            return crate::apc::Continue(Args);
        }
        svc::SVC_NtRaiseException => {
            return crate::except::RaiseException(
                Args[0] as *const crate::except::GuestExceptionRecord,
                Args[1] as *const WowX64Context,
                Args[2] as u8 != 0,
            );
        }
        svc::SVC_NtTerminateProcess if Args[0] as HANDLE == CURRENT_PROCESS || Args[0] == 0 => {
            // The guest's own verdict, said once from our side too, with the
            // guest rip it came from: a guest that ends itself on an error path
            // is otherwise indistinguishable from one that crashed.
            let mut Ctx: WowX64Context = core::mem::zeroed();
            (Cpu.GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), &mut Ctx);
            crate::trace!(
                "guest NtTerminateProcess(%08lx) from rip %p",
                Args[1] as u32,
                Ctx.Rip as *const c_void,
            );
        }
        svc::SVC_NtCallbackReturn => {
            return crate::callback::CallbackReturn(Args);
        }
        svc::SVC_NtCreateThread => {
            return crate::thread::CreateThread(Args);
        }
        svc::SVC_NtTerminateThread => {
            return crate::thread::TerminateThread(Args);
        }
        svc::SVC_NtGetContextThread | svc::SVC_NtSetContextThread => {
            if !IsCurrentThread(Args[0] as HANDLE) {
                // Forwarding would have the kernel read or write an ARM64
                // CONTEXT through the guest's x86-64 one. Another thread's
                // guest state lives in that thread's CPU layer; a GET is
                // answered for a thread parked in a syscall (thread.rs).
                if Service == svc::SVC_NtGetContextThread {
                    return crate::thread::RemoteGuestContext(Args[0] as HANDLE, Args[1] as *mut WowX64Context);
                }
                crate::trace!("NtSetContextThread on another thread: not supported yet");
                return STATUS_NOT_IMPLEMENTED;
            }
            let Ctx = Args[1] as *mut WowX64Context;
            let Status = if Service == svc::SVC_NtGetContextThread {
                (Cpu.GetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Ctx)
            } else {
                (Cpu.SetContext)(CURRENT_THREAD, CURRENT_PROCESS, null_mut(), Ctx)
            };
            return Status as u32 as u64;
        }
        svc::SVC_NtOpenFile | svc::SVC_NtCreateFile | svc::SVC_NtOpenDirectoryObject => {
            let Oa = Args[2] as *const OBJECT_ATTRIBUTES;
            if Service == svc::SVC_NtOpenDirectoryObject {
                if IsKnownDlls(Oa) {
                    return STATUS_OBJECT_NAME_NOT_FOUND;
                }
            } else if Redirect(Oa, &mut Shadow) {
                Args[2] = &Shadow.Oa as *const _ as u64;
            }
        }
        svc::SVC_NtQueryAttributesFile | svc::SVC_NtQueryFullAttributesFile => {
            if Redirect(Args[0] as *const OBJECT_ATTRIBUTES, &mut Shadow) {
                Args[0] = &Shadow.Oa as *const _ as u64;
            }
        }
        svc::SVC_NtAllocateVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[3] as *mut SIZE_T;
            let (Type, Prot) = (Args[4] as ULONG, Args[5] as ULONG);
            (Cpu.NotifyMemoryAlloc)(*Base, *Size, Type, Prot, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryAlloc)(*Base, *Size, Type, Prot, 1, Status as u32);
            return Status;
        }
        svc::SVC_NtProtectVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[2] as *mut SIZE_T;
            let Prot = Args[3] as ULONG;
            (Cpu.NotifyMemoryProtect)(*Base, *Size, Prot, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryProtect)(*Base, *Size, Prot, 1, Status as u32);
            return Status;
        }
        svc::SVC_NtFreeVirtualMemory if IsSelf(Args[0]) => {
            let Base = Args[1] as *mut PVOID;
            let Size = Args[2] as *mut SIZE_T;
            let Type = Args[3] as ULONG;
            (Cpu.NotifyMemoryFree)(*Base, *Size, Type, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryFree)(*Base, *Size, Type, 1, Status as u32);
            return Status;
        }
        svc::SVC_NtMapViewOfSection if IsSelf(Args[1]) => {
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
        svc::SVC_NtUnmapViewOfSection if IsSelf(Args[0]) => {
            let Base = Args[1] as PVOID;
            (Cpu.NotifyUnmapViewOfSection)(Base, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyUnmapViewOfSection)(Base, 1, Status as u32);
            return Status;
        }
        // Kernel-side writes into guest memory: FEX's SMC tracking sees only
        // the guest's own stores, so code translated from a buffer the KERNEL
        // fills must be dropped (selfmod's ReadFile-into-code rung).
        svc::SVC_NtReadFile => {
            let (File, Buffer, Length) = (Args[0] as HANDLE, Args[5] as PVOID, Args[6] as SIZE_T);
            (Cpu.NotifyReadFile)(File, Buffer, Length, 0, 0);
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyReadFile)(File, Buffer, Length, 1, Status as NTSTATUS);
            return Status;
        }
        svc::SVC_NtWriteVirtualMemory if IsSelf(Args[0]) => {
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryDirty)(Args[1] as PVOID, Args[3] as SIZE_T);
            return Status;
        }
        svc::SVC_NtReadVirtualMemory => {
            // The destination is always in THIS process, whichever is read.
            let Status = NativeSyscall(Service, Args.as_ptr());
            (Cpu.NotifyMemoryDirty)(Args[2] as PVOID, Args[3] as SIZE_T);
            return Status;
        }
        svc::SVC_NtFlushInstructionCache if IsSelf(Args[0]) => {
            (Cpu.FlushInstructionCache2)(Args[1] as *const c_void, Args[2] as SIZE_T);
        }
        _ => {}
    }
    NativeSyscall(Service, Args.as_ptr())
}

static FallbackCount: AtomicU32 = AtomicU32::new(0);
/// Is the file `Oa` names a DLL for a machine other than x86-64? Read from its
/// headers; anything unreadable or not a PE is not. An EXE is not: an x86-64
/// process on Windows on ARM sees system32 unredirected and runs its arm64
/// programs as native children (thread.rs `CreateNativeChildThread`); what it
/// must never get is a foreign module where it expects its own.
unsafe fn ForeignDll(Oa: *const OBJECT_ATTRIBUTES) -> bool {
    const IMAGE_FILE_DLL: u16 = 0x2000;
    const GENERIC_READ: ULONG = 0x8000_0000;
    const SYNCHRONIZE: ULONG = 0x0010_0000;
    const FILE_SHARE_READ_WRITE: ULONG = 3;
    const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x20;
    const FILE_NON_DIRECTORY_FILE: ULONG = 0x40;
    let mut Open = *Oa;
    let mut File: HANDLE = null_mut();
    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    if NtOpenFile(
        &mut File,
        GENERIC_READ | SYNCHRONIZE,
        &mut Open,
        &mut Iosb,
        FILE_SHARE_READ_WRITE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    ) < 0
    {
        return false;
    }
    let mut Head = [0u8; 1024];
    let Read = NtReadFile(
        File,
        null_mut(),
        None,
        null_mut(),
        &mut Iosb,
        Head.as_mut_ptr() as PVOID,
        Head.len() as ULONG,
        null_mut(),
        null_mut(),
    );
    NtClose(File);
    if Read < 0 || Head[0] != b'M' || Head[1] != b'Z' {
        return false;
    }
    let Pe = u32::from_le_bytes([Head[0x3C], Head[0x3D], Head[0x3E], Head[0x3F]]) as usize;
    // IMAGE_FILE_HEADER: Machine at +4, Characteristics at +22.
    if Pe + 24 > Head.len() || &Head[Pe..Pe + 4] != b"PE\0\0" {
        return false;
    }
    let Machine = u16::from_le_bytes([Head[Pe + 4], Head[Pe + 5]]);
    let Characteristics = u16::from_le_bytes([Head[Pe + 22], Head[Pe + 23]]);
    Machine != 0x8664 && Characteristics & IMAGE_FILE_DLL != 0
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
/// passes to the kernel instead -- when the name EXISTS in sysamd64. Guest
/// memory is never written.
///
/// Conditional on purpose, where WoW64 redirects system32 wholesale: an x86-64
/// process is not a WoW64 process (Windows on ARM does not redirect it at
/// all), and system32 holds things that are the guest's as much as anyone's --
/// its own exe (spawn64 relaunching itself found nothing), configuration,
/// drivers\etc. What must NOT resolve to system32 is an arm64 module where the
/// guest expects its own, and every module the guest can load is in sysamd64
/// (mkramdisk stages the whole closure), so existence there is the test.
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

    // The bare directory stays system32: it is where the guest's own exe lives
    // and what its current directory names.
    if Name.len() == Hit + 10 {
        return false;
    }
    let mut Info: FILE_BASIC_INFORMATION = core::mem::zeroed();
    if NtQueryAttributesFile(&mut Shadow.Oa, &mut Info) >= 0 {
        return true;
    }
    // Not in sysamd64. Leave it in system32 -- unless what system32 holds
    // under that name is a DLL for ANOTHER machine, which the guest must never
    // be handed: a module loaded at run time (gdi32's LoadLibrary of lpk.dll)
    // is outside the staged closure, and falling back to the native arm64 copy
    // had the guest loader map it and call arm64 code as x86-64. Redirecting
    // to the absent sysamd64 copy gives the guest "not found", which is what
    // WoW64 gives a 32-bit process in the same spot.
    // A refusal is always logged: it is a module the guest wanted and the
    // staged closure lacks, and capping it hid every one after the eighth
    // benign fallback (manifest probes, .Local checks) of a busy boot.
    let Foreign = ForeignDll(Oa);
    if Foreign || FallbackCount.fetch_add(1, Ordering::Relaxed) < 8 {
        crate::trace!(
            "not in sysamd64: %wZ -> %s",
            (*Oa).ObjectName,
            if Foreign { b"a foreign-machine DLL in system32: NOT FOUND\0".as_ptr() } else { b"left in system32\0".as_ptr() },
        );
    }
    Foreign
}
