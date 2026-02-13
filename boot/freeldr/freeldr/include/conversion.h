/*
 * PROJECT:         EFI Windows Loader
 * LICENSE:         GPL - See COPYING in the top level directory
 * FILE:            boot/freeldr/freeldr/windows/conversion.c
 * PURPOSE:         Physical <-> Virtual addressing mode conversions (arch-specific)
 * PROGRAMMERS:     Aleksey Bragin (aleksey@reactos.org)
 */

#pragma once

#if defined(_ZOOM2_) || defined(_M_ARM64)
/*
 * Identity mapping: VA == PA.
 * ARM64 runs with UEFI's identity-mapped page tables during boot,
 * so no PA<->VA conversion is needed.
 */
FORCEINLINE
PVOID
VaToPa(PVOID Va)
{
    return Va;
}

FORCEINLINE
PVOID
PaToVa(PVOID Pa)
{
    return Pa;
}
#else
/* Arch-specific addresses translation implementation */
FORCEINLINE
PVOID
VaToPa(PVOID Va)
{
    return (PVOID)((ULONG_PTR)Va & ~KSEG0_BASE);
}

FORCEINLINE
PVOID
PaToVa(PVOID Pa)
{
    return (PVOID)((ULONG_PTR)Pa | KSEG0_BASE);
}
#endif
