//! arm64-only ntdll surface for the WOW64/FEX host layer — Phase A of
//! `docs/engineering/wow64-plan.md`. These are the ntdll imports
//! `libwow64fex.dll` needs that the 0x502 baseline never had. Shim-quality on
//! purpose: correct for the single-process bring-up path; Phase C replaces the
//! interesting ones (a real `NtAllocateVirtualMemoryEx` syscall, cross-thread
//! `RtlWow64*` context access via `ThreadWow64Context`) with kernel-backed
//! versions. Exported via `-arch=arm64` lines in `dll/ntdll/def/ntdll.spec`
//! (§9.43: arch-gating leaves the x86 ordinal tables byte-identical).

#![allow(non_snake_case)]

use core::ptr::{null_mut, write_volatile};
use ntdll_sys::nt::*;

/// Wine-compatible CPU-area slot: `TEB->TlsSlots[WOW64_TLS_CPURESERVED]`
/// points at a `WOW64_CPURESERVED` header followed (16-aligned) by the guest
/// machine's context. wow64.dll (ours, Phase B) installs it per thread;
/// wow64fex consumes it through `RtlWow64GetCurrentCpuArea`.
const WOW64_TLS_CPURESERVED: usize = 1;

#[repr(C)]
struct WOW64_CPURESERVED {
    Flags: u16,
    Machine: u16,
    // guest context follows, 16-byte aligned
}

/// `MEM_EXTENDED_PARAMETER` (Win10) — just the shape we need to parse.
#[repr(C)]
struct MEM_EXTENDED_PARAMETER_SHIM {
    TypeAndReserved: u64, // Type : 8, Reserved : 56
    PointerOrValue: u64,
}

const MEM_EXTENDED_PARAMETER_ADDRESS_REQUIREMENTS: u64 = 1;
const MEM_TOP_DOWN: ULONG = 0x100000;
const STATUS_DLL_NOT_FOUND: NTSTATUS = 0xC0000135_u32 as NTSTATUS;
const STATUS_PARTIAL_COPY: NTSTATUS = 0x8000000D_u32 as NTSTATUS;

#[inline(always)]
fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

/// `NtAllocateVirtualMemoryEx` — user-mode wrapper (no 0x502 syscall exists).
/// wow64fex only passes `MemExtendedParameterAddressRequirements` with a
/// LowestStartingAddress of 4GB, to keep host allocations out of the guest's
/// 32-bit space. On our 48-bit arm64 user VA, `MEM_TOP_DOWN` satisfies that
/// (allocations land just below the VA ceiling), so honor address requirements
/// that way and reject parameter types we cannot honor.
#[no_mangle]
pub unsafe extern "system" fn NtAllocateVirtualMemoryEx(
    ProcessHandle: HANDLE,
    BaseAddress: *mut PVOID,
    RegionSize: PSIZE_T,
    mut AllocationType: ULONG,
    PageProtection: ULONG,
    ExtendedParameters: *mut MEM_EXTENDED_PARAMETER_SHIM,
    ParameterCount: ULONG,
) -> NTSTATUS {
    for i in 0..ParameterCount as usize {
        let ty = (*ExtendedParameters.add(i)).TypeAndReserved & 0xFF;
        if ty == MEM_EXTENDED_PARAMETER_ADDRESS_REQUIREMENTS {
            // Honored approximately: keep the allocation high.
            AllocationType |= MEM_TOP_DOWN;
        } else {
            DbgPrint(
                b"NtAllocateVirtualMemoryEx: unsupported extended parameter type %u\n\0".as_ptr()
                    as *const i8,
                ty as u32,
            );
            return STATUS_INVALID_PARAMETER;
        }
    }

    NtAllocateVirtualMemory(
        ProcessHandle,
        BaseAddress,
        0,
        RegionSize,
        AllocationType,
        PageProtection,
    )
}

// NtOpenKeyEx moved to rtl/openkeyex.rs: it is a Win7 export every arch needs.

/// `LdrGetDllFullName` — copy the module's `FullDllName` (NULL = main image)
/// into the caller's buffer. Win10/Wine semantics: always NUL-terminated,
/// `STATUS_BUFFER_TOO_SMALL` on truncation.
#[no_mangle]
pub unsafe extern "system" fn LdrGetDllFullName(
    DllHandle: PVOID,
    FullDllName: PUNICODE_STRING,
) -> NTSTATUS {
    let Peb = NtCurrentPeb();
    let Target = if DllHandle.is_null() {
        (*Peb).ImageBaseAddress
    } else {
        DllHandle
    };
    let mut Status = STATUS_DLL_NOT_FOUND;

    RtlEnterCriticalSection((*Peb).LoaderLock as *mut _);

    let Head = &mut (*(*Peb).Ldr).InLoadOrderModuleList as *mut LIST_ENTRY;
    let mut Entry = (*Head).Flink;
    while Entry != Head {
        let LdrEntry = Entry as *mut _LDR_DATA_TABLE_ENTRY;
        if (*LdrEntry).DllBase == Target {
            let Name = &(*LdrEntry).FullDllName;
            let mut Copy = Name.Length;

            if Copy as usize + 2 > (*FullDllName).MaximumLength as usize {
                Copy = (*FullDllName).MaximumLength.saturating_sub(2);
                Status = STATUS_BUFFER_TOO_SMALL;
            } else {
                Status = STATUS_SUCCESS;
            }

            core::ptr::copy_nonoverlapping(
                Name.Buffer as *const u8,
                (*FullDllName).Buffer as *mut u8,
                Copy as usize,
            );
            *(*FullDllName).Buffer.add(Copy as usize / 2) = 0;
            write_volatile(&mut (*FullDllName).Length, Copy);
            break;
        }
        Entry = (*Entry).Flink;
    }

    RtlLeaveCriticalSection((*Peb).LoaderLock as *mut _);
    Status
}

/// `RtlQueryPerformanceCounter`/`Frequency` — BOOLEAN wrappers over the
/// syscall (real ntdll fast-paths via KUSER_SHARED_DATA; the syscall is
/// correct, just slower).
#[no_mangle]
pub unsafe extern "system" fn RtlQueryPerformanceCounter(
    PerformanceCounter: PLARGE_INTEGER,
) -> BOOLEAN {
    let mut Frequency: LARGE_INTEGER = core::mem::zeroed();
    nt_success(NtQueryPerformanceCounter(PerformanceCounter, &mut Frequency)) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryPerformanceFrequency(Frequency: PLARGE_INTEGER) -> BOOLEAN {
    let mut Counter: LARGE_INTEGER = core::mem::zeroed();
    nt_success(NtQueryPerformanceCounter(&mut Counter, Frequency)) as BOOLEAN
}

/// `RtlWow64GetCurrentCpuArea` — Wine-compatible: return the guest machine
/// type and a pointer to the guest context stashed in the current thread's
/// CPU-reserved TLS slot.
#[no_mangle]
pub unsafe extern "system" fn RtlWow64GetCurrentCpuArea(
    Machine: *mut u16,
    Context: *mut PVOID,
    ContextEx: *mut PVOID,
) -> NTSTATUS {
    let Cpu = (*NtCurrentTeb()).TlsSlots[WOW64_TLS_CPURESERVED] as *mut WOW64_CPURESERVED;
    if Cpu.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if !Machine.is_null() {
        write_volatile(Machine, (*Cpu).Machine);
    }
    if !Context.is_null() {
        // Guest context follows the header, 16-byte aligned.
        let Ctx = (Cpu.add(1) as usize + 15) & !15usize;
        write_volatile(Context, Ctx as PVOID);
    }
    if !ContextEx.is_null() {
        write_volatile(ContextEx, null_mut());
    }
    STATUS_SUCCESS
}

/// `LdrSystemDllInitBlock` — the Win10 loader's system-DLL info block, a DATA
/// export wow64fex reads `ntdll_handle` (offset 72: the 32-bit ntdll base)
/// from during `BTCpuProcessInit`. Layout per FEX's winternl.h: ULONG version
/// + ULONG[3] + 9×ULONG64 (96 bytes). Zero until the WoW64 process bootstrap
/// (wow64probe in Phase B, the real 64-bit loader in Phase C) populates it.
#[repr(C, align(8))]
pub struct SYSTEM_DLL_INIT_BLOCK {
    pub version: u32,
    pub unknown1: [u32; 3],
    pub unknown2: u64,
    pub pLdrInitializeThunk: u64,
    pub pKiUserExceptionDispatcher: u64,
    pub pKiUserApcDispatcher: u64,
    pub pKiUserCallbackDispatcher: u64,
    pub pRtlUserThreadStart: u64,
    pub pRtlpQueryProcessDebugInformationRemote: u64,
    pub ntdll_handle: u64,
    pub pLdrSystemDllInitBlock: u64,
    pub pRtlpFreezeTimeBias: u64,
}

#[no_mangle]
pub static mut LdrSystemDllInitBlock: SYSTEM_DLL_INIT_BLOCK = SYSTEM_DLL_INIT_BLOCK {
    version: 0,
    unknown1: [0; 3],
    unknown2: 0,
    pLdrInitializeThunk: 0,
    pKiUserExceptionDispatcher: 0,
    pKiUserApcDispatcher: 0,
    pKiUserCallbackDispatcher: 0,
    pRtlUserThreadStart: 0,
    pRtlpQueryProcessDebugInformationRemote: 0,
    ntdll_handle: 0,
    pLdrSystemDllInitBlock: 0,
    pRtlpFreezeTimeBias: 0,
};

#[repr(C)]
struct WOW64_THREAD_BASIC_INFORMATION {
    exit_status: NTSTATUS,
    teb_base_address: PVOID,
    client_id: CLIENT_ID,
    affinity_mask: usize,
    priority: i32,
    base_priority: i32,
}

extern "system" {
    fn NtQueryInformationThread(
        ThreadHandle: HANDLE,
        ThreadInformationClass: ULONG,
        ThreadInformation: PVOID,
        ThreadInformationLength: ULONG,
        ReturnLength: *mut ULONG,
    ) -> NTSTATUS;
}

/// Another thread's guest context, read the way a debugger reads it: its TEB's
/// CPU-reserved TLS slot, then the 16-aligned context after the `WOW64_CPURESERVED`
/// header, both from the thread's own process. That area is current whenever the guest
/// thread is stopped at a syscall or exception boundary (a suspended thread parked in a
/// win32k call, which is what a sampler wants). Writes stay current-thread only.
/// Diverges from Windows on access: the handle also needs THREAD_QUERY_(LIMITED_)INFORMATION
/// (for the TEB address), where Windows reads the context in the kernel
/// (`ThreadWow64Context`) and needs only THREAD_GET_CONTEXT. A GET_CONTEXT-only handle
/// gets STATUS_ACCESS_DENIED, never another thread's context.
unsafe fn wow64_remote_context(Thread: HANDLE, Out: *mut WOW64_CONTEXT) -> NTSTATUS {
    const THREAD_BASIC_INFORMATION_CLASS: ULONG = 0;
    const PROCESS_VM_READ_: ACCESS_MASK = 0x0010;
    let mut Info: WOW64_THREAD_BASIC_INFORMATION = core::mem::zeroed();
    let Status = NtQueryInformationThread(
        Thread,
        THREAD_BASIC_INFORMATION_CLASS,
        &mut Info as *mut _ as PVOID,
        core::mem::size_of::<WOW64_THREAD_BASIC_INFORMATION>() as ULONG,
        null_mut(),
    );
    if !nt_success(Status) {
        return Status;
    }
    if Info.teb_base_address.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let mut Process: HANDLE = null_mut();
    let mut Attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    Attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    let mut Client = Info.client_id;
    Client.UniqueThread = null_mut();
    let Status = NtOpenProcess(&mut Process, PROCESS_VM_READ_, &mut Attributes, &mut Client);
    if !nt_success(Status) {
        return Status;
    }
    let read = |at: usize, to: PVOID, len: usize| -> NTSTATUS {
        let mut Done: SIZE_T = 0;
        let s = NtReadVirtualMemory(Process, at as PVOID, to, len as SIZE_T, &mut Done);
        if nt_success(s) && Done as usize != len {
            STATUS_PARTIAL_COPY
        } else {
            s
        }
    };
    let Slot = Info.teb_base_address as usize
        + core::mem::offset_of!(TEB, TlsSlots)
        + WOW64_TLS_CPURESERVED * core::mem::size_of::<PVOID>();
    let mut Cpu: usize = 0;
    let mut Header = WOW64_CPURESERVED { Flags: 0, Machine: 0 };
    let mut Status = read(Slot, &mut Cpu as *mut usize as PVOID, core::mem::size_of::<usize>());
    if nt_success(Status) && Cpu == 0 {
        Status = STATUS_INVALID_PARAMETER; // not a WOW64 thread, or not started yet
    }
    if nt_success(Status) {
        Status = read(Cpu, &mut Header as *mut _ as PVOID, core::mem::size_of::<WOW64_CPURESERVED>());
    }
    if nt_success(Status) && Header.Machine != 0x014c {
        Status = STATUS_INVALID_PARAMETER; // IMAGE_FILE_MACHINE_I386 is the only guest with a WOW64_CONTEXT
    }
    if nt_success(Status) {
        let Ctx = (Cpu + core::mem::size_of::<WOW64_CPURESERVED>() + 15) & !15usize;
        Status = read(Ctx, Out as PVOID, core::mem::size_of::<WOW64_CONTEXT>());
    }
    NtClose(Process);
    Status
}

/// The current thread (including a duplicated current-thread handle) reads its own CPU
/// area directly; any other thread is read from its process (`wow64_remote_context`).
#[no_mangle]
pub unsafe extern "system" fn RtlWow64GetThreadContext(
    ThreadHandle: HANDLE,
    Context: PVOID,
) -> NTSTATUS {
    if Context.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let current = ThreadHandle == NtCurrentThread() || {
        let mut Info: WOW64_THREAD_BASIC_INFORMATION = core::mem::zeroed();
        nt_success(NtQueryInformationThread(
            ThreadHandle,
            0,
            &mut Info as *mut _ as PVOID,
            core::mem::size_of::<WOW64_THREAD_BASIC_INFORMATION>() as ULONG,
            null_mut(),
        )) && Info.client_id.UniqueThread == (*NtCurrentTeb()).ClientId.UniqueThread
    };
    if !current {
        return wow64_remote_context(ThreadHandle, Context as *mut WOW64_CONTEXT);
    }
    let mut CpuContext: PVOID = null_mut();
    let Status = RtlWow64GetCurrentCpuArea(null_mut(), &mut CpuContext, null_mut());
    if !nt_success(Status) || CpuContext.is_null() {
        return Status;
    }
    core::ptr::copy(
        CpuContext as *const WOW64_CONTEXT,
        Context as *mut WOW64_CONTEXT,
        1,
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64SetThreadContext(
    _ThreadHandle: HANDLE,
    Context: PVOID,
) -> NTSTATUS {
    if Context.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let mut CpuContext: PVOID = null_mut();
    let Status = RtlWow64GetCurrentCpuArea(null_mut(), &mut CpuContext, null_mut());
    if !nt_success(Status) || CpuContext.is_null() {
        return Status;
    }
    core::ptr::copy(
        Context as *const WOW64_CONTEXT,
        CpuContext as *mut WOW64_CONTEXT,
        1,
    );
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64SuspendThread(
    _ThreadHandle: HANDLE,
    _PreviousSuspendCount: *mut ULONG,
) -> NTSTATUS {
    DbgPrint(b"RtlWow64SuspendThread: not implemented (wow64-plan Phase C)\n\0".as_ptr() as *const i8);
    STATUS_NOT_IMPLEMENTED
}
