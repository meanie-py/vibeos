/*
 * PROJECT:     vibeos WoW64
 * LICENSE:     GPL-2.0-or-later
 * PURPOSE:     The CPU layer's memory-notification contract (BTCpuNotify*)
 * COPYRIGHT:   vibeos WoW64 lane
 *
 * The CPU layer keeps its own model of the guest's 32-bit address space, and on
 * a JIT host that model is authoritative: FEX's decoder REFUSES to translate any
 * guest address outside the executable set it has been told about, and it caches
 * translated blocks keyed on guest address. So every guest-visible change to the
 * address space has to be announced, or the emulator either runs stale code or
 * declines to run at all. `BTCpuNotifyMapViewOfSection` alone covers images; a
 * guest that generates code into VirtualAlloc'd memory — ATL/MFC thunks, packers,
 * any JIT — is invisible without the rest of this contract.
 *
 * PAIRING IS LOAD-BEARING. Every *Before below takes a lock inside the CPU layer
 * that the matching *After releases. Call them immediately either side of the
 * host syscall, with NO early return, goto, or longjmp in between: an unmatched
 * Before deadlocks the whole guest process, and an unmatched After releases a
 * mutex nobody holds. The `Wow64Notify*` wrappers tolerate a CPU layer that
 * exports none of this (amd64's native wow64cpu needs no invalidation at all),
 * so a NULL entry point is a silent no-op rather than a branch at every call.
 */

#ifndef _WOW64_BTCPU_H_
#define _WOW64_BTCPU_H_

/* TRUE when the handle names the calling process. The notifications describe
 * THIS process's guest address space; handing the CPU layer a range belonging to
 * some other process would invalidate — or reprotect — our own pages at those
 * addresses. */
BOOLEAN Wow64IsCurrentProcess(HANDLE Process);

/* NtAllocateVirtualMemory */
VOID Wow64NotifyAllocBefore(PVOID Base, SIZE_T Size, ULONG Type, ULONG Protect);
VOID Wow64NotifyAllocAfter(PVOID Base, SIZE_T Size, ULONG Type, ULONG Protect,
                           LONG Status);

/* NtProtectVirtualMemory */
VOID Wow64NotifyProtectBefore(PVOID Base, SIZE_T Size, ULONG NewProtect);
VOID Wow64NotifyProtectAfter(PVOID Base, SIZE_T Size, ULONG NewProtect,
                             LONG Status);

/* NtFreeVirtualMemory */
VOID Wow64NotifyFreeBefore(PVOID Base, SIZE_T Size, ULONG FreeType);
VOID Wow64NotifyFreeAfter(PVOID Base, SIZE_T Size, ULONG FreeType,
                          LONG Status);

/* NtUnmapViewOfSection */
VOID Wow64NotifyUnmapBefore(PVOID Base);
VOID Wow64NotifyUnmapAfter(PVOID Base, LONG Status);

/* NtReadFile — the kernel may write straight into a write-trapped RWX page, so
 * the CPU layer untraps it for the duration of the read. */
VOID Wow64NotifyReadFileBefore(HANDLE File, PVOID Buffer, SIZE_T Length);
VOID Wow64NotifyReadFileAfter(HANDLE File, PVOID Buffer, SIZE_T Length,
                              LONG Status);

/* Unpaired: a single point-in-time announcement. */
VOID Wow64NotifyMemoryDirty(PVOID Base, SIZE_T Size);   /* NtWriteVirtualMemory  */
VOID Wow64NotifyFlushInstructionCache(PVOID Base, SIZE_T Size); /* NtFlushInstructionCache */

/* ProcessExecuteFlags, once the kernel has accepted it. Records the value in
 * the WOW64INFO block the guest reads and tells the CPU layer, which is the
 * enforcer for a JIT host (the kernel's NX bits describe HOST execution). */
VOID Wow64SetExecuteFlags(ULONG Flags);

/* Publish the notification entry points. Called once, AFTER BTCpuProcessInit:
 * the CPU layer builds its address-space model there, so a notification that
 * arrives earlier has nothing to land in. */
VOID Wow64pResolveNotifyContract(PVOID CpuDll);

#endif /* _WOW64_BTCPU_H_ */
