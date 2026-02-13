/*
 * PROJECT:         ReactOS Run-Time Library
 * LICENSE:         GPL-2.0-or-later
 * FILE:            lib/rtl/arm64/slist.S
 * PURPOSE:         ARM64 Interlocked SList Functions
 */

.text

/*
 * PSLIST_ENTRY RtlInterlockedPopEntrySList(PSLIST_HEADER ListHead)
 *
 * x0 = pointer to SLIST_HEADER (16 bytes, 16-byte aligned)
 * Returns: pointer to popped entry, or NULL if empty
 */
.globl RtlInterlockedPopEntrySList
RtlInterlockedPopEntrySList:
    mov x4, x0              /* Save ListHead pointer */
1:
    ldaxp x0, x1, [x4]      /* Load header atomically: x0=Alignment, x1=Region */

    /* Extract NextEntry pointer from header */
    /* In 64-bit SLIST_HEADER, the next pointer is in the Region field shifted */
    cbz x1, 3f              /* If Region is 0, list is empty */

    /* x1 contains the next entry pointer (with some flags in low bits) */
    and x2, x1, #~0xF       /* Mask off low 4 bits to get clean pointer */
    cbz x2, 3f              /* If pointer is NULL, list is empty */

    /* Read new next pointer from the entry we're about to pop */
    ldr x3, [x2]            /* x3 = entry->Next */

    /* Prepare new header: decrement depth, set new next */
    sub x5, x0, #1          /* Decrement depth (low 16 bits of Alignment) */
    and x5, x5, #0xFFFF     /* Mask depth */
    bic x6, x0, #0xFFFF     /* Keep upper bits (sequence etc) */
    add x7, x0, #0x10000    /* Increment sequence */
    and x7, x7, #~0xFFFF    /* Keep only sequence bits */
    orr x5, x5, x7          /* Combine depth + new sequence */

    /* Store new next pointer in Region */
    mov x6, x3

    /* Try to atomically update */
    stlxp w8, x5, x6, [x4]
    cbnz w8, 1b             /* Retry if exclusive store failed */

    mov x0, x2              /* Return the popped entry */
    ret

3:  /* List is empty */
    clrex
    mov x0, #0
    ret

/*
 * PSLIST_ENTRY RtlInterlockedPushEntrySList(PSLIST_HEADER ListHead, PSLIST_ENTRY ListEntry)
 *
 * x0 = ListHead, x1 = ListEntry
 * Returns: previous first entry
 */
.globl RtlInterlockedPushEntrySList
RtlInterlockedPushEntrySList:
    mov x4, x0              /* Save ListHead */
    mov x5, x1              /* Save ListEntry */
1:
    ldaxp x0, x1, [x4]      /* Load current header */

    /* Extract current next pointer from Region */
    and x2, x1, #~0xF

    /* Store current head as entry->Next */
    str x2, [x5]

    /* Prepare new header: increment depth, set new next to ListEntry */
    add x6, x0, #1          /* Increment depth */
    and x6, x6, #0xFFFF     /* Mask depth */
    add x7, x0, #0x10000    /* Increment sequence */
    and x7, x7, #~0xFFFF
    orr x6, x6, x7

    /* New Region = new entry pointer */
    mov x7, x5

    stlxp w8, x6, x7, [x4]
    cbnz w8, 1b

    mov x0, x2              /* Return old first entry */
    ret

/*
 * PSLIST_ENTRY RtlInterlockedFlushSList(PSLIST_HEADER ListHead)
 *
 * x0 = ListHead
 * Returns: pointer to old first entry, or NULL if empty
 */
.globl RtlInterlockedFlushSList
RtlInterlockedFlushSList:
    mov x4, x0
1:
    ldaxp x0, x1, [x4]

    /* Extract current next pointer */
    and x2, x1, #~0xF
    cbz x2, 3f              /* Already empty */

    /* Set header to empty: depth=0, next=NULL */
    mov x5, #0
    mov x6, #0
    stlxp w8, x5, x6, [x4]
    cbnz w8, 1b

    mov x0, x2              /* Return old head */
    ret

3:  /* Already empty */
    clrex
    mov x0, #0
    ret

/* EOF */
