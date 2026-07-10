//! arm64-only ntdll surface for the WOW64/FEX host layer — Phase A of
//! `docs/engineering/wow64-plan.md`. These are the ntdll imports
//! `libwow64fex.dll` needs that the 0x502 baseline never had. Shim-quality on
//! purpose: correct for the single-process bring-up path; Phase C replaces the
//! interesting ones (a real `NtAllocateVirtualMemoryEx` syscall, cross-thread
//! `RtlWow64*` context access via `ThreadWow64Context`) with kernel-backed
//! versions. Exported via `-arch=arm64` lines in `dll/ntdll/def/ntdll.spec`
//! (§9.35: arch-gating leaves the x86 ordinal tables byte-identical).

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

/// `NtOpenKeyEx` — wrapper over `NtOpenKey`. The OpenOptions FEX passes don't
/// change behavior for its HKLM config probing; a plain open is compatible.
#[no_mangle]
pub unsafe extern "system" fn NtOpenKeyEx(
    KeyHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    _OpenOptions: ULONG,
) -> NTSTATUS {
    NtOpenKey(KeyHandle, DesiredAccess, ObjectAttributes)
}

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

/// Cross-thread WOW64 context access needs kernel support (the
/// `ThreadWow64Context` info class) — Phase C. Loud stubs until then; the
/// single-threaded bring-up path never hits them.
#[no_mangle]
pub unsafe extern "system" fn RtlWow64GetThreadContext(
    _ThreadHandle: HANDLE,
    _Context: PVOID,
) -> NTSTATUS {
    DbgPrint(b"RtlWow64GetThreadContext: not implemented (wow64-plan Phase C)\n\0".as_ptr() as *const i8);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64SetThreadContext(
    _ThreadHandle: HANDLE,
    _Context: PVOID,
) -> NTSTATUS {
    DbgPrint(b"RtlWow64SetThreadContext: not implemented (wow64-plan Phase C)\n\0".as_ptr() as *const i8);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RtlWow64SuspendThread(
    _ThreadHandle: HANDLE,
    _PreviousSuspendCount: *mut ULONG,
) -> NTSTATUS {
    DbgPrint(b"RtlWow64SuspendThread: not implemented (wow64-plan Phase C)\n\0".as_ptr() as *const i8);
    STATUS_NOT_IMPLEMENTED
}
