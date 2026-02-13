/*
 * kxarm64.h - ARM64 assembly macros for ReactOS
 *
 * This file provides MSVC-style assembly macros for use with GNU assembler (gas)
 * and Clang integrated assembler.
 */

#ifndef _KXARM64_H_
#define _KXARM64_H_

#ifdef _MSC_VER
/* MSVC / ARMASM64 native macros would go here */
#error "MSVC ARM64 assembler not yet supported"
#else

/* Clang integrated assembler macros for Windows PE/COFF targets */

.macro TEXTAREA
    .text
    .p2align 2
.endm

.macro DATAAREA
    .data
.endm

.macro RODATAAREA
    .section .rdata,"dr"
.endm

/*
 * LEAF_ENTRY - Start a leaf function (no stack frame, no calls to other functions)
 */
.macro LEAF_ENTRY Name
    .globl \Name
    .p2align 2
\Name:
.endm

/*
 * LEAF_END - End a leaf function
 */
.macro LEAF_END Name
    /* No .size directive for COFF */
.endm

/*
 * NESTED_ENTRY - Start a nested function (has stack frame, calls other functions)
 */
.macro NESTED_ENTRY Name
    .globl \Name
    .p2align 2
\Name:
.endm

/*
 * NESTED_END - End a nested function
 */
.macro NESTED_END Name
    /* No .size directive for COFF */
.endm

/*
 * PROLOG_PUSH - Push registers in prolog
 */
.macro PROLOG_PUSH Regs
    stp \Regs
.endm

/*
 * EPILOG_POP - Pop registers in epilog
 */
.macro EPILOG_POP Regs
    ldp \Regs
.endm

/*
 * ALTERNATE_ENTRY - Define an alternate entry point to a function
 */
.macro ALTERNATE_ENTRY Name
    .globl \Name
\Name:
.endm

/*
 * END - End of assembly file (no-op for gas)
 */
.macro END
    /* No-op */
.endm

#endif /* _MSC_VER */

#endif /* _KXARM64_H_ */
