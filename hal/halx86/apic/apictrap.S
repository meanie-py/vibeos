/*
 * FILE:            hal/halx86/apic/apictrap.S
 * COPYRIGHT:       See COPYING in the top level directory
 * PURPOSE:         System Traps, Entrypoints and Exitpoints
 * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
 * NOTE:            See asmmacro.S for the shared entry/exit code.
 */

/* INCLUDES ******************************************************************/

#include <asm.inc>

#ifdef _M_AMD64
#include <ksamd64.inc>
#include <trapamd64.inc>
.code

TRAP_ENTRY HalpClockInterrupt, TF_SEND_EOI
TRAP_ENTRY HalpClockIpi, TF_SEND_EOI
TRAP_ENTRY HalpProfileInterrupt, TF_SEND_EOI

PUBLIC ApicSpuriousService
ApicSpuriousService:
    iretq

#else
#include <ks386.inc>
#include <internal/i386/asmmacro.S>
.code

TRAP_ENTRY HalpClockInterrupt, KI_PUSH_FAKE_ERROR_CODE
TRAP_ENTRY HalpClockIpi, KI_PUSH_FAKE_ERROR_CODE
TRAP_ENTRY HalpProfileInterrupt, KI_PUSH_FAKE_ERROR_CODE
/* Kernel IPI (vector 0xE1) receive stub — i386 only: amd64 receives 0xE1 in
   the kernel (KiIpiInterrupt).  Handler: rtctimer.rs HalpIpiInterruptHandler. */
TRAP_ENTRY HalpIpiInterrupt, KI_PUSH_FAKE_ERROR_CODE
/* HalpTrap0D (V86-mode #GP dispatch) removed with generic/bios.c — see trap.S. */
TRAP_ENTRY HalpApcInterrupt, KI_PUSH_FAKE_ERROR_CODE
TRAP_ENTRY HalpDispatchInterrupt, KI_PUSH_FAKE_ERROR_CODE

PUBLIC _ApicSpuriousService
_ApicSpuriousService:
    iret

#endif





END
