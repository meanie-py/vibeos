/*
 * Provide __bss_start__ and __bss_end__ symbols for lld (PE-COFF).
 * GNU ld gets these from the linker script (freeldr_gcc.lds), but lld
 * in MinGW PE mode doesn't support -T linker scripts fully.
 * PE-COFF merges sections with the same prefix; $ suffix controls ordering.
 */

#include <asm.inc>

/* .bss$a sorts first within the merged .bss section */
.section .bss$a, "bw"
PUBLIC __bss_start__
__bss_start__:

/* .bss$z sorts last within the merged .bss section */
.section .bss$z, "bw"
PUBLIC __bss_end__
__bss_end__:

END
