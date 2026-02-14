
/* INCLUDES ******************************************************************/

/* ARM64 stack probing functions for ReactOS CRT */

#include <kxarm64.h>

#define PAGE_SIZE 4096

/* CODE **********************************************************************/
    TEXTAREA

/*
 * __chkstk - Stack probe function for ARM64 Windows
 *
 * Called by the compiler when a function allocates more than one page (4096)
 * of stack space. Probes each page from the current SP downward so that
 * guard pages are triggered in order, enabling proper stack growth.
 *
 * On entry:  x15 = number of 16-byte units to allocate
 * On exit:   x15 is preserved (caller does: sub sp, sp, x15, lsl #4)
 * Clobbers:  x16, x17 (intra-procedure scratch registers)
 *
 * Reference: LLVM compiler-rt builtins/aarch64/chkstk.S
 */
    LEAF_ENTRY __chkstk
    lsl     x16, x15, #4           /* x16 = total bytes = x15 * 16 */
    mov     x17, sp                /* x17 = current stack pointer */
.Lprobe_loop:
    sub     x17, x17, #PAGE_SIZE   /* move down one page */
    subs    x16, x16, #PAGE_SIZE   /* subtract one page from remaining */
    b.lo    .Lprobe_done           /* if remaining < 0, done */
    ldr     xzr, [x17]            /* probe the page (load to zero register) */
    b       .Lprobe_loop
.Lprobe_done:
    ret
    LEAF_END __chkstk

/*
 * __alloca_probe - Alloca stack probe (same as __chkstk)
 */
    LEAF_ENTRY __alloca_probe
    lsl     x16, x15, #4
    mov     x17, sp
.Lalloca_loop:
    sub     x17, x17, #PAGE_SIZE
    subs    x16, x16, #PAGE_SIZE
    b.lo    .Lalloca_done
    ldr     xzr, [x17]
    b       .Lalloca_loop
.Lalloca_done:
    ret
    LEAF_END __alloca_probe

    END
/* EOF */
