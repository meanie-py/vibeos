/*
 * aptramp64.S — amd64 AP real-mode startup trampoline (SMP bring-up, M2).
 *
 * Assembled with SYSTEM GNU as/ld (NOT clang — .code16, see AGENTS.md):
 *   as --64 -o aptramp64.o aptramp64.S
 *   ld -Ttext=0 -e _start --oformat binary -o aptramp64.bin aptramp64.o
 * The flat binary is include_bytes!'d by trampoline.rs (TRAMPOLINE_AMD64).
 *
 * Runtime contract (must match trampoline.rs install()):
 *   - Blob copied to the head of a page-aligned physical page < 1MB
 *     ("low stub"), IDENTITY-MAPPED in the kernel CR3.
 *   - SIPI vector = page_phys >> 12, so the AP enters in real mode at
 *     CS = vector<<8, IP = 0 (linear = page base).  A20 already enabled
 *     (long past firmware boot; QEMU has it on).
 *   - Param block at page offset 0x800 (PARAM_BLOCK_OFFSET), layout below
 *     (REVISED vs the first staging draft: GDTR/IDTR limit+base are packed
 *     CONTIGUOUS so lgdt/lidt can consume them in place — the draft had 6
 *     pad bytes between limit and base, which would have loaded a garbage
 *     descriptor-table base; and arg0 was added for the KiSystemStartup
 *     rcx contract):
 *       0x00  cr0        u64   kernel CR0 (PE|PG|..)
 *       0x08  cr3        u64   kernel PML4 phys (< 4GB assumed; loaded as u32)
 *       0x10  cr4        u64   kernel CR4 (PAE=1)
 *       0x18  gdtr       u16 limit + u64 base   (10 bytes, packed)
 *       0x28  idtr       u16 limit + u64 base   (10 bytes, packed)
 *       0x38  seg_cs     u16
 *       0x3A  seg_ds     u16
 *       0x3C  seg_es     u16
 *       0x3E  seg_ss     u16
 *       0x40  seg_fs     u16
 *       0x42  seg_gs     u16
 *       0x44  tr         u16
 *       0x46  (pad)      u16
 *       0x48  entry      u64   KiSystemStartup (ContextFrame.Rip)
 *       0x50  stack      u64   AP kernel stack  (ContextFrame.Rsp)
 *       0x58  eflags     u64
 *       0x60  arg0       u64   LoaderBlock — KiSystemStartup's rcx
 *
 * Position independence: assembled at origin 0; the 16-bit prologue computes
 * the page base (CS<<4) into EBX and patches the three absolute operands
 * (tmp GDTR base, the two far-jump pointers) in place before using them.
 */

    .section .text
    .globl _start

    .equ PARAM, 0x800

/* ---------------------------------------------------------------- 16-bit */
    .code16
_start:
    cli
    wbinvd

    /* DS := CS so plain offsets address this page in real mode. */
    movw    %cs, %ax
    movw    %ax, %ds

    /* EBX = linear page base = CS << 4. */
    movzwl  %ax, %ebx
    shll    $4, %ebx

    /* Patch the absolute operands with the runtime base. */
    movl    %ebx, %eax
    addl    $tmp_gdt, %eax
    movl    %eax, tmp_gdtr+2        /* GDTR.base = P + tmp_gdt      */
    movl    %ebx, %eax
    addl    $pm32, %eax
    movl    %eax, farptr32          /* far ptr offset = P + pm32    */
    movl    %ebx, %eax
    addl    $lm64, %eax
    movl    %eax, farptr64          /* far ptr offset = P + lm64    */

    /* Protected mode with the flat temporary GDT. */
    lgdtl   tmp_gdtr
    movl    %cr0, %eax
    orl     $1, %eax                /* CR0.PE */
    movl    %eax, %cr0
    ljmpl   *farptr32               /* serialize; -> 0x08:pm32      */

/* ---------------------------------------------------------------- 32-bit */
    .code32
pm32:
    movw    $0x10, %ax              /* flat data */
    movw    %ax, %ds
    movw    %ax, %es
    movw    %ax, %ss

    /* ESI = linear param block.  (EBX survived the mode switch.) */
    leal    PARAM(%ebx), %esi

    movl    0x10(%esi), %eax        /* kernel CR4 (PAE)             */
    movl    %eax, %cr4
    movl    0x08(%esi), %eax        /* kernel PML4 (low 32 bits)    */
    movl    %eax, %cr3

    movl    $0xC0000080, %ecx       /* IA32_EFER                    */
    rdmsr
    orl     $0x100, %eax            /* EFER.LME                     */
    wrmsr

    movl    0x00(%esi), %eax        /* kernel CR0 (PG|PE) -> LMA    */
    movl    %eax, %cr0
    ljmpl   *farptr64(%ebx)         /* -> 0x18:lm64 (64-bit CS)     */

/* ---------------------------------------------------------------- 64-bit */
    .code64
lm64:
    /* RSI = param block (ESI zero-extends). Install the real per-AP
     * descriptor tables — limit+base are packed contiguous at 0x18/0x28. */
    lgdt    0x18(%rsi)
    lidt    0x28(%rsi)

    /* LTR: the GDT snapshot came from a LIVE processor state, so the TSS
     * descriptor carries the BUSY type — clear it or ltr #GPs. */
    movzwq  0x44(%rsi), %rax        /* tr selector                  */
    andl    $0xFFF8, %eax           /* descriptor index offset      */
    movq    0x1A(%rsi), %rdx        /* gdt base (unaligned load ok) */
    addq    %rax, %rdx
    andb    $0xFD, 5(%rdx)          /* type.busy (bit 1 of byte 5)  */
    movw    0x44(%rsi), %ax
    ltr     %ax

    /* Data segments.  Writing GS zeroes GSBASE — fine: KiSystemStartup
     * establishes the PCR GS base via MSR before any gs: access. */
    movw    0x3A(%rsi), %ax
    movw    %ax, %ds
    movw    0x3C(%rsi), %ax
    movw    %ax, %es
    movw    0x3E(%rsi), %ax
    movw    %ax, %ss
    movw    0x40(%rsi), %ax
    movw    %ax, %fs
    movw    0x42(%rsi), %ax
    movw    %ax, %gs

    movq    0x50(%rsi), %rsp        /* AP kernel stack              */
    /* Fake NULL return address: KiSystemStartup is compiled for the
     * standard ABI entry state rsp == 8 (mod 16) (caller's CALL pushed a
     * return address). Entering with a bare 16-aligned stack top shifts
     * every aligned spill by 8 and its movaps xmm save #GPs (first AP
     * bring-up bug, 2026-07-10). It never returns, so 0 is fine — and it
     * terminates backtraces cleanly. */
    pushq   $0
    pushq   0x58(%rsi)
    popfq                           /* kernel EFLAGS (IF=0)         */

    movq    0x60(%rsi), %rcx        /* arg0: LoaderBlock            */

    /* Far-return into the kernel's code selector at KiSystemStartup. */
    movzwq  0x38(%rsi), %rax        /* seg_cs                       */
    pushq   %rax
    pushq   0x48(%rsi)              /* entry                        */
    lretq

/* ------------------------------------------------------------- data ---- */
    .balign 8
tmp_gdt:
    .quad   0                       /* null                         */
    .quad   0x00CF9A000000FFFF      /* 0x08: flat 32-bit code       */
    .quad   0x00CF92000000FFFF      /* 0x10: flat 32-bit data       */
    .quad   0x00209A0000000000      /* 0x18: 64-bit code (L=1)      */
tmp_gdt_end:

tmp_gdtr:
    .word   tmp_gdt_end - tmp_gdt - 1
    .long   0                       /* patched: P + tmp_gdt         */

farptr32:
    .long   0                       /* patched: P + pm32            */
    .word   0x08
farptr64:
    .long   0                       /* patched: P + lm64            */
    .word   0x18

/* The blob must stay clear of the param block at 0x800 — enforced by the
 * regen script (size check on the flat binary) since GAS can't .if on a
 * not-yet-absolute dot expression here. */
