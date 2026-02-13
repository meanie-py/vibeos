/*
 * PROJECT:         ReactOS Kernel
 * LICENSE:         GPL-2.0-or-later
 * FILE:            ntoskrnl/ke/arm64/stubs_asm.S
 * PURPOSE:         ARM64 Architecture-Specific Assembly Stubs
 */

.text

/* void _enable(void) - enable interrupts */
.globl _enable
_enable:
    msr daifclr, #0xF
    ret

/* void _disable(void) - disable interrupts */
.globl _disable
_disable:
    msr daifset, #0xF
    ret

/* VOID NTAPI KiSystemStartup(PLOADER_PARAMETER_BLOCK LoaderBlock) */
.globl KiSystemStartup
KiSystemStartup:
    /* Infinite loop for now */
    b KiSystemStartup
