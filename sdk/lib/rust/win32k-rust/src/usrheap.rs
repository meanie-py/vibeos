//! Rust translation of `win32ss/user/ntuser/usrheap.c` — the win32k user heap.
//! [ntuser wave 6: usrheap]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{MmCreateSection, MmMapViewInSessionSpace, MmMapViewOfSection,
    MmUnmapViewOfSection, PsGetCurrentProcessWin32Process, PsGetProcessWin32Process,
    ZwAllocateVirtualMemory, IoGetCurrentProcess, ObfDereferenceObject, RtlCreateHeap};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;

// SetLastNtError is FASTCALL.
#[cfg(target_arch = "x86")]
extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn SetLastNtError(Status: NTSTATUS); }

// NtCurrentProcess() == (HANDLE)-1.
#[inline(always)]
fn nt_current_process() -> HANDLE {
    (-1isize) as HANDLE
}

// SECTION_ALL_ACCESS (mmtypes.h) = STANDARD_RIGHTS_REQUIRED|SECTION_QUERY|MAP_*|EXTEND.
const SECTION_ALL_ACCESS: u32 = 0xF001F;

// The win32k user heap, referenced by main.c / ntstubs.c (Rust provides storage).
#[no_mangle]
pub static mut GlobalUserHeap: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut GlobalUserHeapSection: PVOID = core::ptr::null_mut();

// RTL_HEAP_COMMIT_ROUTINE is NTAPI: the bindgen fn-ptr type is `extern "stdcall"`
// on i386 and `extern "C"` elsewhere. The pointer's type must match exactly (an
// `extern "system" fn` is a *distinct type* even where the ABI coincides), so the
// exported callback is split per-arch over a shared impl.
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn IntUserHeapCommitRoutine(
    Base: PVOID, CommitAddress: *mut PVOID, CommitSize: PSIZE_T,
) -> NTSTATUS {
    commit_routine_impl(Base, CommitAddress, CommitSize)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntUserHeapCommitRoutine(
    Base: PVOID, CommitAddress: *mut PVOID, CommitSize: PSIZE_T,
) -> NTSTATUS {
    commit_routine_impl(Base, CommitAddress, CommitSize)
}

unsafe fn commit_routine_impl(
    Base: PVOID,
    CommitAddress: *mut PVOID,
    CommitSize: PSIZE_T,
) -> NTSTATUS {
    let W32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let mut UserBase: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if !W32Process.is_null() {
        // Search for the mapping.
        let mut Mapping: PW32HEAP_USER_MAPPING =
            core::ptr::addr_of_mut!((*W32Process).HeapMappings);
        while !Mapping.is_null() {
            if (*Mapping).KernelMapping == Base {
                UserBase = (*Mapping).UserMapping;
                break;
            }
            Mapping = (*Mapping).Next;
        }
    } else {
        // HACK (matches C): only during startup. Temporarily map into user space.
        let mut ViewSize: SIZE_T = 0;
        let mut Offset: LARGE_INTEGER = core::mem::zeroed();
        Offset.QuadPart = 0;
        Status = MmMapViewOfSection(
            GlobalUserHeapSection as _,
            IoGetCurrentProcess(),
            &mut UserBase as _,
            0,
            0,
            core::ptr::addr_of_mut!(Offset) as _,
            &mut ViewSize as _,
            ViewUnmap,
            SEC_NO_CHANGE as u32,
            PAGE_READONLY,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    // Apply the commit address offset to the user base address.
    let mut Delta: SIZE_T = ((*CommitAddress as usize) - (Base as usize)) as SIZE_T;
    let mut UserCommitAddress: PVOID = ((UserBase as usize) + (Delta as usize)) as PVOID;

    // Perform the actual commit.
    Status = ZwAllocateVirtualMemory(
        nt_current_process(),
        &mut UserCommitAddress as _,
        0,
        CommitSize as _,
        MEM_COMMIT as u32,
        PAGE_READONLY,
    );

    if NT_SUCCESS(Status) {
        Delta = ((UserCommitAddress as usize) - (UserBase as usize)) as SIZE_T;
        *CommitAddress = ((Base as usize) + (Delta as usize)) as PVOID;
    }

    if W32Process.is_null() {
        MmUnmapViewOfSection(IoGetCurrentProcess(), UserBase);
    }

    Status
}

// static IntUserHeapCreate — the RtlCreateHeap core (rustconv skipped it).
unsafe fn IntUserHeapCreate(
    SectionObject: PVOID,
    SystemMappedBase: *mut PVOID,
    HeapSize: ULONG,
) -> PWIN32HEAP {
    let mut MappedView: PVOID = core::ptr::null_mut();
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T = PAGE_SIZE as SIZE_T;
    let mut Parameters: RTL_HEAP_PARAMETERS = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Offset.QuadPart = 0;

    // Commit the first page before creating the heap.
    Status = MmMapViewOfSection(
        SectionObject as _,
        IoGetCurrentProcess(),
        &mut MappedView as _,
        0,
        0,
        core::ptr::addr_of_mut!(Offset) as _,
        &mut ViewSize as _,
        ViewUnmap,
        SEC_NO_CHANGE as u32,
        PAGE_READONLY,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    Status = ZwAllocateVirtualMemory(
        nt_current_process(),
        &mut MappedView as _,
        0,
        &mut ViewSize as _,
        MEM_COMMIT as u32,
        PAGE_READONLY,
    );

    MmUnmapViewOfSection(IoGetCurrentProcess(), MappedView);

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    // Create the heap; don't serialize in kmode (caller synchronizes).
    Parameters.Length = core::mem::size_of::<RTL_HEAP_PARAMETERS>() as ULONG;
    Parameters.InitialCommit = ViewSize;
    Parameters.InitialReserve = HeapSize as SIZE_T;
    Parameters.CommitRoutine = Some(IntUserHeapCommitRoutine);

    let pHeap: PVOID = RtlCreateHeap(
        HEAP_ZERO_MEMORY | HEAP_NO_SERIALIZE,
        *SystemMappedBase,
        HeapSize as SIZE_T,
        ViewSize,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Parameters) as _,
    );

    pHeap as PWIN32HEAP
}

// UserCreateHeap / Map / Unmap are cdecl (`_Name`, no @N) — extern "C".
#[no_mangle]
pub unsafe extern "C" fn UserCreateHeap(
    SectionObject: *mut PVOID,
    SystemBase: *mut PVOID,
    HeapSize: SIZE_T,
) -> PWIN32HEAP {
    let mut SizeHeap: LARGE_INTEGER = core::mem::zeroed();
    let mut HeapSize = HeapSize;

    SizeHeap.QuadPart = HeapSize as i64;

    // Create the section and map it into session space.
    let mut Status: NTSTATUS = MmCreateSection(
        SectionObject as _,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SizeHeap) as _,
        PAGE_READWRITE,
        SEC_RESERVE | 1,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    Status = MmMapViewInSessionSpace(*SectionObject, SystemBase, &mut HeapSize);
    if !NT_SUCCESS(Status) {
        ObfDereferenceObject(*SectionObject);
        *SectionObject = core::ptr::null_mut();
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    let pHeap: PWIN32HEAP = IntUserHeapCreate(*SectionObject, SystemBase, HeapSize as ULONG);

    if pHeap.is_null() {
        ObfDereferenceObject(*SectionObject);
        *SectionObject = core::ptr::null_mut();
        SetLastNtError(STATUS_UNSUCCESSFUL);
    }

    pHeap
}

#[no_mangle]
pub unsafe extern "C" fn UnmapGlobalUserHeap(Process: PEPROCESS) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let W32Process: PPROCESSINFO = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
    if W32Process.is_null() {
        // ASSERT(FALSE) — retail no-op.
    }

    // The first mapping entry must be the global user heap.
    let HeapMapping: PW32HEAP_USER_MAPPING = core::ptr::addr_of_mut!((*W32Process).HeapMappings);

    // Unmap if we're the last thread using the global user heap.
    (*HeapMapping).Count -= 1;
    if (*HeapMapping).Count == 0 {
        Status = MmUnmapViewOfSection(Process as _, (*HeapMapping).UserMapping);
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn MapGlobalUserHeap(
    Process: PEPROCESS,
    KernelMapping: *mut PVOID,
    UserMapping: *mut PVOID,
) -> NTSTATUS {
    let W32Process: PPROCESSINFO = PsGetProcessWin32Process(Process as _) as PPROCESSINFO;
    if W32Process.is_null() {
        // ASSERT(FALSE) — retail no-op.
    }

    // The first mapping entry must be the global user heap.
    let HeapMapping: PW32HEAP_USER_MAPPING = core::ptr::addr_of_mut!((*W32Process).HeapMappings);

    // Find out if another thread already mapped the global user heap.
    if (*HeapMapping).KernelMapping == GlobalUserHeap as PVOID {
        (*HeapMapping).Count += 1;
        *KernelMapping = (*HeapMapping).KernelMapping;
        *UserMapping = (*HeapMapping).UserMapping;
        return STATUS_SUCCESS;
    }

    // We're the first, map the global heap into the process.
    let mut UserBase: PVOID = core::ptr::null_mut();
    let mut ViewSize: SIZE_T = 0;
    let mut Offset: LARGE_INTEGER = core::mem::zeroed();
    Offset.QuadPart = 0;
    let Status: NTSTATUS = MmMapViewOfSection(
        GlobalUserHeapSection as _,
        Process as _,
        &mut UserBase as _,
        crate::init::W32Wow64ZeroBits(Process as _) as _,
        0,
        core::ptr::addr_of_mut!(Offset) as _,
        &mut ViewSize as _,
        ViewUnmap,
        SEC_NO_CHANGE as u32,
        PAGE_READONLY,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Add the mapping.
    (*HeapMapping).Next = core::ptr::null_mut();
    (*HeapMapping).KernelMapping = GlobalUserHeap as PVOID;
    (*HeapMapping).UserMapping = UserBase;
    (*HeapMapping).Limit = ViewSize;
    (*HeapMapping).Count = 1;

    *KernelMapping = (*HeapMapping).KernelMapping;
    *UserMapping = (*HeapMapping).UserMapping;

    STATUS_SUCCESS
}
