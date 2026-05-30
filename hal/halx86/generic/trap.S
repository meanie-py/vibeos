/*
 * FILE:            hal/halx86/generic/trap.S
 * COPYRIGHT:       See COPYING in the top level directory
 * PURPOSE:         System Traps, Entrypoints and Exitpoints
 * PROGRAMMER:      Timo Kreuzer (timo.kreuzer@reactos.org)
 * NOTE:            See asmmacro.S for the shared entry/exit code.
 */

/* INCLUDES ******************************************************************/

#include <asm.inc>

#include <ks386.inc>
#include <internal/i386/asmmacro.S>

.code

/* HalpTrap0D (V86-mode #GP dispatch) removed: its handler HalpTrap0DHandler and
 * sole installer HalpBiosCall lived in generic/bios.c, retired in the pre-Vista
 * prune (V86 BIOS emulation is dead under UEFI). Nothing installs IDT[0x0D]. */
TRAP_ENTRY HalpApcInterrupt, KI_SOFTWARE_TRAP
TRAP_ENTRY HalpClockInterrupt, KI_PUSH_FAKE_ERROR_CODE
TRAP_ENTRY HalpProfileInterrupt, KI_PUSH_FAKE_ERROR_CODE

END
