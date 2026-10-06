use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;

// ---------------------------------------------------------------------------
// Real exported kernel symbols, re-declared against win32k-sys nominal types.
// Same pattern as mdevobj.rs / engevent.rs / gdiobj.rs — avoids the cross-crate
// `POOL_TYPE` collision between `ntkrnl_sys::ddk::_POOL_TYPE` and
// `win32k_sys::win32k::_POOL_TYPE`. The symbol/ABI is unchanged; only Rust
// nominal types differ vs ntkrnl-sys's own bindgen.
//
// ExAllocatePoolWithTag / ExFreePoolWithTag are NTAPI exports (ntoskrnl.spec
// `@ stdcall`). NTSYSAPI/NTKERNELAPI NTAPI = stdcall on i386, C on amd64/arm64:
// `extern "system"` resolves correctly per arch.
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

// SafeProbeFor{Read,Write} — pre-translation shims declared in
// `win32ss/include/safe_user_io.h`. NTAPI (stdcall on i386 / cdecl elsewhere).
// Length is SIZE_T per the C header, NOT ULONG. (PVOID/SIZE_T = win32k-sys
// nominal types via the blanket `use` above; no cross-crate collision since
// the probe shim only touches SIZE_T/PVOID, not _POOL_TYPE / _MDL.)
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const TAG_ALLOCUSERMEM: u32 = 0x6d55676e;

/// EngAllocMem — GDI driver pool allocator.
/// `ExAllocatePoolWithTag((fl & FL_NONPAGED_MEMORY) ? NonPagedPool : PagedPool, ...);`
/// optionally zeroed when `FL_ZERO_MEMORY`.
#[no_mangle]
pub unsafe extern "system" fn EngAllocMem(
    fl: ULONG,
    cjMemSize: ULONG,
    ulTag: ULONG,
) -> PVOID {
    let pool_type: POOL_TYPE = if (fl & FL_NONPAGED_MEMORY) != 0 {
        NonPagedPool
    } else {
        PagedPool
    };

    let pvBaseAddress = ExAllocatePoolWithTag(pool_type, cjMemSize as SIZE_T, ulTag);

    if pvBaseAddress.is_null() {
        return core::ptr::null_mut();
    }

    if (fl & FL_ZERO_MEMORY) != 0 {
        core::ptr::write_bytes(pvBaseAddress as *mut u8, 0, cjMemSize as usize);
    }

    pvBaseAddress
}

#[no_mangle]
pub unsafe extern "system" fn EngFreeMem(pvBaseAddress: PVOID) {
    if !pvBaseAddress.is_null() {
        ExFreePoolWithTag(pvBaseAddress, 0);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EngAllocUserMem(
    cjMemSize: SIZE_T,
    ulTag: ULONG,
) -> PVOID {
    let mut pvBaseAddress: PVOID = core::ptr::null_mut();
    let mut cjMemSize = cjMemSize;

    let Status: NTSTATUS = ntkrnl_sys::ddk::ZwAllocateVirtualMemory(
        ntkrnl_sys::macros::NtCurrentProcess(),
        &mut pvBaseAddress as *mut PVOID as *mut ntkrnl_sys::ddk::PVOID,
        0,
        &mut cjMemSize as *mut SIZE_T as *mut ntkrnl_sys::ddk::SIZE_T,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_READWRITE,
    );

    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    pvBaseAddress
}

#[no_mangle]
pub unsafe extern "system" fn EngFreeUserMem(pvBaseAddress: PVOID) {
    let mut pvBaseAddress = pvBaseAddress;
    let mut cjSize: SIZE_T = 0;

    ntkrnl_sys::ddk::ZwFreeVirtualMemory(
        ntkrnl_sys::macros::NtCurrentProcess(),
        &mut pvBaseAddress as *mut PVOID as *mut ntkrnl_sys::ddk::PVOID,
        &mut cjSize as *mut SIZE_T as *mut ntkrnl_sys::ddk::SIZE_T,
        MEM_RELEASE,
    );
}

// HackSecureVirtualMemory / HackUnsecureVirtualMemory: kernel-side MDL plumbing.
// All MDL APIs cross the win32k↔ntoskrnl boundary, so we use the
// `ntkrnl_sys::ddk` PMDL type directly (NOT the win32k-sys re-bindgen) to
// match the function signatures. The C source wraps MmProbeAndLockPages in
// SEH (the only genuine FAULT_RECOVER probe in this file), whose handler takes
// the status and frees the MDL; the status-returning probe gives that same path.
#[no_mangle]
pub unsafe extern "C" fn HackSecureVirtualMemory(
    Address: PVOID,
    Size: SIZE_T,
    ProbeMode: ULONG,
    SafeAddress: *mut PVOID,
) -> PVOID {
    let Status: NTSTATUS;
    let pmdl: ntkrnl_sys::ddk::PMDL;
    let Operation: ntkrnl_sys::ddk::LOCK_OPERATION;

    if ProbeMode == PAGE_READONLY {
        Operation = ntkrnl_sys::ddk::_LOCK_OPERATION_IoReadAccess;
    } else if ProbeMode == PAGE_READWRITE {
        Operation = ntkrnl_sys::ddk::_LOCK_OPERATION_IoModifyAccess;
    } else {
        return core::ptr::null_mut();
    }

    pmdl = ntkrnl_sys::ddk::IoAllocateMdl(
        Address as ntkrnl_sys::ddk::PVOID,
        Size as ULONG,
        0u8,                                  // FALSE: SecondaryBuffer
        1u8,                                  // TRUE: ChargeQuota
        core::ptr::null_mut(),                // Irp
    );
    if pmdl.is_null() {
        return core::ptr::null_mut();
    }

    // UserMode = 1 (KPROCESSOR_MODE = i8).
    Status = ntkrnl_sys::probe::MmTryProbeAndLockPages(pmdl, 1i8, Operation);

    if !NT_SUCCESS(Status) {
        ntkrnl_sys::ddk::IoFreeMdl(pmdl);
        return core::ptr::null_mut();
    }

    *SafeAddress = ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe(
        pmdl,
        ntkrnl_sys::ddk::_MM_PAGE_PRIORITY_NormalPagePriority as ULONG,
    ) as PVOID;
    if (*SafeAddress).is_null() {
        ntkrnl_sys::ddk::MmUnlockPages(pmdl);
        ntkrnl_sys::ddk::IoFreeMdl(pmdl);
        return core::ptr::null_mut();
    }
    pmdl as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn HackUnsecureVirtualMemory(SecureHandle: PVOID) {
    let pmdl: ntkrnl_sys::ddk::PMDL = SecureHandle as ntkrnl_sys::ddk::PMDL;
    ntkrnl_sys::ddk::MmUnlockPages(pmdl);
    ntkrnl_sys::ddk::IoFreeMdl(pmdl);
}

// EngSecureMem / EngSecureMemForRead — the C source intentionally returns
// (HANDLE)-1 BEFORE calling MmSecureVirtualMemory (the "// HACK!!!" comment),
// so the Mm* call is dead code. Mirror that exactly: faithful translation of
// the normalized C wins over cleaning up the HACK.
#[no_mangle]
pub unsafe extern "system" fn EngSecureMem(Address: PVOID, Length: ULONG) -> HANDLE {
    let __SehStatus = SafeProbeForWrite(Address, Length as SIZE_T, 1);
    if !NT_SUCCESS(__SehStatus) {
        return core::ptr::null_mut();
    }
    -1isize as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn EngSecureMemForRead(Address: PVOID, Length: ULONG) -> HANDLE {
    let __SehStatus = SafeProbeForRead(Address, Length as SIZE_T, 1);
    if !NT_SUCCESS(__SehStatus) {
        return core::ptr::null_mut();
    }
    // ADDRESS_AND_SIZE_TO_SPAN_PAGES(Va, Size) — DDK macro:
    //   (ULONG)((((SIZE_T)(Va) & (PAGE_SIZE - 1)) + (Size) + (PAGE_SIZE - 1)) >> PAGE_SHIFT)
    let mut cPages: ULONG = ((((Address as usize) & (PAGE_SIZE as usize - 1))
        + Length as usize
        + (PAGE_SIZE as usize - 1)) / PAGE_SIZE as usize) as ULONG;
    // ALIGN_DOWN_POINTER_BY(addr, align) — `addr & ~(align - 1)` (align must be power of 2).
    let mut pjProbe: *mut u8 =
        ((Address as usize) & !(PAGE_SIZE as usize - 1)) as *mut u8;
    while cPages > 0 {
        cPages -= 1;
        // (void)*pjProbe; — read probe.
        core::ptr::read_volatile(pjProbe);
        pjProbe = pjProbe.add(PAGE_SIZE as usize);
    }
    -1isize as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn EngUnsecureMem(Mem: HANDLE) {
    if Mem == (-1isize) as HANDLE {
        return;
    }
    ntkrnl_sys::ddk::MmUnsecureVirtualMemory(Mem as ntkrnl_sys::ddk::HANDLE);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Memory Management Functions
//  * FILE:              win32ss/gdi/eng/mem.c
//  * PROGRAMER:         Jason Filby
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// _Must_inspect_result_
// _When_(fl & FL_ZERO_MEMORY, _Ret_opt_bytecount_(cjMemSize))
// _When_(!(fl & FL_ZERO_MEMORY), _Ret_opt_bytecap_(cjMemSize))
// __drv_allocatesMem(Mem)
// ENGAPI
// PVOID
// APIENTRY
// EngAllocMem(
//     _In_ ULONG fl,
//     _In_ ULONG cjMemSize,
//     _In_ ULONG ulTag)
// {
//     PVOID pvBaseAddress;
//
//     pvBaseAddress = ExAllocatePoolWithTag((fl & FL_NONPAGED_MEMORY) ?
//                                                     NonPagedPool : PagedPool,
//                                           cjMemSize,
//                                           ulTag);
//
//     if (pvBaseAddress == NULL)
//         return NULL;
//
//     if (fl & FL_ZERO_MEMORY)
//         RtlZeroMemory(pvBaseAddress, cjMemSize);
//
//     return pvBaseAddress;
// }
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngFreeMem(PVOID pvBaseAddress)
// {
//     /* Windows allows to pass NULL */
//     if (pvBaseAddress)
//     {
//         /* Use 0 as tag, which equals a call to ExFreePool */
//         ExFreePoolWithTag(pvBaseAddress, 0);
//     }
// }
//
// /*
//  * @implemented
//  */
// _Must_inspect_result_
// _Ret_opt_bytecount_(cjMemSize)
// __drv_allocatesMem(UserMem)
// ENGAPI
// PVOID
// APIENTRY
// EngAllocUserMem(
//     _In_ SIZE_T cjMemSize,
//     _In_ ULONG ulTag)
// {
//     PVOID pvBaseAddress = NULL;
//     NTSTATUS Status;
//
//     Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                      &pvBaseAddress,
//                                      0,
//                                      &cjMemSize,
//                                      MEM_COMMIT | MEM_RESERVE,
//                                      PAGE_READWRITE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return NULL;
//     }
//
//     /* TODO: Add allocation info to AVL tree (stored inside W32PROCESS structure) */
//     //hSecure = EngSecureMem(pvBaseAddress, cj);
//
//     return pvBaseAddress;
// }
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngFreeUserMem(PVOID pvBaseAddress)
// {
//     SIZE_T cjSize = 0;
//
//     ZwFreeVirtualMemory(NtCurrentProcess(),
//                         &pvBaseAddress,
//                         &cjSize,
//                         MEM_RELEASE);
//
//   /* TODO: Remove allocation info from AVL tree */
// }
//
// PVOID
// APIENTRY
// HackSecureVirtualMemory(
//     IN PVOID Address,
//     IN SIZE_T Size,
//     IN ULONG ProbeMode,
//     OUT PVOID *SafeAddress)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PMDL pmdl;
//     LOCK_OPERATION Operation;
//
//     if (ProbeMode == PAGE_READONLY) Operation = IoReadAccess;
//     else if (ProbeMode == PAGE_READWRITE) Operation = IoModifyAccess;
//     else return NULL;
//
//     pmdl = IoAllocateMdl(Address, (ULONG)Size, FALSE, TRUE, NULL);
//     if (pmdl == NULL)
//     {
//         return NULL;
//     }
//
//     _SEH2_TRY
//     {
//         MmProbeAndLockPages(pmdl, UserMode, Operation);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END
//
//     if (!NT_SUCCESS(Status))
//     {
//         IoFreeMdl(pmdl);
//         return NULL;
//     }
//
//     *SafeAddress = MmGetSystemAddressForMdlSafe(pmdl, NormalPagePriority);
//
//     if(!*SafeAddress)
//     {
//         MmUnlockPages(pmdl);
//         IoFreeMdl(pmdl);
//         return NULL;
//     }
//
//     return pmdl;
// }
//
// VOID
// APIENTRY
// HackUnsecureVirtualMemory(
//     IN PVOID  SecureHandle)
// {
//     PMDL pmdl = (PMDL)SecureHandle;
//
//     MmUnlockPages(pmdl);
//     IoFreeMdl(pmdl);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// APIENTRY
// EngSecureMem(PVOID Address, ULONG Length)
// {
//     {// HACK!!!
//         {
//             NTSTATUS __SehStatus = SafeProbeForWrite(Address, Length, 1);
//             if (!NT_SUCCESS(__SehStatus))
//             {
//                 return NULL;
//             }
//         }
//         return (HANDLE)-1;
//     }
//     return MmSecureVirtualMemory(Address, Length, PAGE_READWRITE);
// }
//
// HANDLE
// APIENTRY
// EngSecureMemForRead(PVOID Address, ULONG Length)
// {
//     {// HACK!!!
//         ULONG cPages;
//         volatile BYTE *pjProbe;
//
//         {
//             NTSTATUS __SehStatus = SafeProbeForRead(Address, Length, 1);
//             if (!NT_SUCCESS(__SehStatus))
//             {
//                 return NULL;
//             }
//         }
//         cPages = ADDRESS_AND_SIZE_TO_SPAN_PAGES(Address, Length);
//         pjProbe = ALIGN_DOWN_POINTER_BY(Address, PAGE_SIZE);
//         while(cPages--)
//         {
//             /* Do a read probe */
//             (void)*pjProbe;
//             pjProbe += PAGE_SIZE;
//         }
//         return (HANDLE)-1;
//     }
//     return MmSecureVirtualMemory(Address, Length, PAGE_READONLY);
// }
//
// /*
//  * @implemented
//  */
// VOID APIENTRY
// EngUnsecureMem(HANDLE Mem)
// {
//     if (Mem == (HANDLE)-1) return;  // HACK!!!
//     MmUnsecureVirtualMemory((PVOID) Mem);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
