/*
 * COPYRIGHT:       See COPYING in the top level directory
 * PROJECT:         ReactOS kernel
 * FILE:            ntoskrnl/ex/i386/fastinterlck_asm.S
 * PURPOSE:         FASTCALL Interlocked Functions
 * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
 */

/* INCLUDES ******************************************************************/

#include <asm.inc>
#include <ks386.inc>
#include <internal/i386/asmmacro.S>

/* FUNCTIONS ****************************************************************/

.code32

/*
 * NOTE: These functions must obey the following rules:
 *  - Acquire locks only on MP systems.
 *  - Be safe at HIGH_LEVEL (no paged access).
 *  - Preserve flags.
 *  - Disable interrupts.
 */

/*VOID
 *FASTCALL
 *ExInterlockedAddLargeStatistic(IN PLARGE_INTEGER Addend,
 *                               IN ULONG Increment)
 */
PUBLIC @ExInterlockedAddLargeStatistic@8
@ExInterlockedAddLargeStatistic@8:

#ifdef CONFIG_SMP
    /* Do the addition */
    lock add [ecx], edx

    /* Check for carry bit and return */
    jc .l1
    ret

.l1:
    /* Add carry */
    lock adc dword ptr [ecx+4], 0
#else
    /* Do the addition and add the carry */
    add dword ptr [ecx], edx
    adc dword ptr [ecx+4], 0
#endif
    /* Return */
    ret


/*
 * SList forwarder stubs — the actual implementations are in the Rust SList
 * crate (sdk/lib/rust/slist), linked via librtl. These stubs convert from
 * fastcall calling convention to stdcall (Rtl* functions).
 * The fault labels (ExpInterlockedPopEntrySList{Resume,Fault,End}) are
 * provided by the Rust SList crate.
 */

/*PSINGLE_LIST_ENTRY
 *FASTCALL
 *ExInterlockedPopEntrySList(IN PSINGLE_LIST_ENTRY ListHead,
 *                           IN PKSPIN_LOCK Lock)
 *
 * Forwards to RtlInterlockedPopEntrySList(ListHead) — stdcall, 1 arg.
 * Lock (edx) is ignored (same as original implementation).
 */
PUBLIC @ExInterlockedPopEntrySList@8
PUBLIC @InterlockedPopEntrySList@4
@ExInterlockedPopEntrySList@8:
@InterlockedPopEntrySList@4:
    push ecx
    call _RtlInterlockedPopEntrySList@4
    ret

/*PSINGLE_LIST_ENTRY
 *FASTCALL
 *ExInterlockedPushEntrySList(IN PSINGLE_LIST_ENTRY ListHead,
 *                            IN PSINGLE_LIST_ENTRY ListEntry,
 *                            IN PKSPIN_LOCK Lock)
 *
 * Forwards to RtlInterlockedPushEntrySList(ListHead, ListEntry, Lock) — stdcall, 3 args.
 * fastcall: ecx=ListHead, edx=ListEntry, [esp+4]=Lock
 */
PUBLIC @ExInterlockedPushEntrySList@12
@ExInterlockedPushEntrySList@12:
    push edx
    push ecx
    call _RtlInterlockedPushEntrySList@8
    ret 4

/*PSINGLE_LIST_ENTRY
 *FASTCALL
 *InterlockedPushEntrySList(IN PSINGLE_LIST_ENTRY ListHead,
 *                          IN PSINGLE_LIST_ENTRY ListEntry)
 *
 * Forwards to RtlInterlockedPushEntrySList(ListHead, ListEntry) — stdcall, 2 args.
 * fastcall: ecx=ListHead, edx=ListEntry
 */
PUBLIC @InterlockedPushEntrySList@8
@InterlockedPushEntrySList@8:
    push edx
    push ecx
    call _RtlInterlockedPushEntrySList@8
    ret

/*PSINGLE_LIST_ENTRY
 *FASTCALL
 *ExInterlockedFlushSList(IN PSINGLE_LIST_ENTRY ListHead)
 *
 * Forwards to RtlInterlockedFlushSList(ListHead) — stdcall, 1 arg.
 * fastcall: ecx=ListHead
 */
PUBLIC @ExInterlockedFlushSList@4
@ExInterlockedFlushSList@4:
    push ecx
    call _RtlInterlockedFlushSList@4
    ret

END
/* EOF */
