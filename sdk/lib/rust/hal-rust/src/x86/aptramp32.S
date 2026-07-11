/*
 * aptramp32.S — i386 AP real-mode startup trampoline (SMP bring-up, M2/i386).
 *
 * Assembled with SYSTEM GNU as/ld (NOT clang — .code16, see AGENTS.md):
 *   as --32 -o aptramp32.o aptramp32.S
 *   ld -m elf_i386 -Ttext=0 -e _start --oformat binary -o aptramp32.bin aptramp32.o
 * The flat binary is include_bytes!'d by trampoline.rs (TRAMPOLINE_I386).
 *
 * Runtime contract (must match trampoline.rs install()):
 *   - Blob copied to the head of a page-aligned physical page < 1MB
 *     ("low stub"), IDENTITY-MAPPED in the kernel CR3 (same design-(b)
 *     dependency as aptramp64.S — NOTES.md #12/#14: the boot CR3 does NOT
 *     identity-map low memory on this fork, so whoever installs the AP's
 *     kernel CR3 (from ProcessorState.SpecialRegisters.Cr3, i.e. the BSP's
 *     __readcr3() at KeStartAllProcessors time — see mproc.c line 116) must
 *     first punch a temporary VA==PA PTE for the stub page into that CR3's
 *     page tables, exactly as required for amd64. NOT solved here.).
 *   - SIPI vector = page_phys >> 12, so the AP enters in real mode at
 *     CS = vector<<8, IP = 0 (linear = page base). A20 already enabled.
 *   - Param block at page offset 0x800 (PARAM_BLOCK_OFFSET) — the SAME
 *     ApTrampolineParams struct shared with aptramp64.S/trampoline.rs.
 *     i386 reads only the LOW 32 BITS of every u64 field (kernel PA/VA
 *     space is 32-bit here); the struct keeps u64 width so one Rust type
 *     serves both arches per the mission brief.
 *       0x00  cr0        u64   kernel CR0 (PE|PG|..)              [low32 used]
 *       0x08  cr3        u64   kernel page-dir phys                [low32 used]
 *       0x10  cr4        u64   kernel CR4 (PAE iff PAE kernel)     [low32 used]
 *       0x18  gdtr       u16 limit + u64 base (packed contiguous)
 *             NOTE: i386 `lgdt`/`lidt` in 32-bit protected mode read exactly
 *             6 bytes (2-byte limit + 4-byte base) — bytes 0x18..0x1E are
 *             THE SAME BYTES amd64's 10-byte pseudo-descriptor starts with,
 *             so aptramp32.S can `lgdt 0x18(%esi)` directly against the
 *             shared struct with no repacking; bytes 0x1E-0x21 (the upper
 *             32 bits of the u64 base, always zero on i386) are simply not
 *             read by a 32-bit-mode lgdt.
 *       0x28  idtr       u16 limit + u64 base (same 6-bytes-consumed note)
 *       0x38  seg_cs     u16
 *       0x3A  seg_ds     u16
 *       0x3C  seg_es     u16
 *       0x3E  seg_ss     u16
 *       0x40  seg_fs     u16   (KGDT_R0_PCR on i386 — see mproc.c line 123)
 *       0x42  seg_gs     u16
 *       0x44  tr         u16
 *       0x46  (pad)      u16
 *       0x48  entry      u64   KiSystemStartup (ContextFrame.Eip)  [low32 used]
 *       0x50  stack      u64   AP kernel stack (ContextFrame.Esp)  [low32 used]
 *       0x58  eflags     u64                                       [low32 used]
 *       0x60  arg0       u64   LoaderBlock (documentation/parity only — see
 *                               "i386 KiSystemStartup arg contract" below;
 *                               NOT consumed by this trampoline)
 *
 * ---------------------------------------------------------------------
 * i386 KiSystemStartup arg contract (VERIFIED against the C tree):
 * ---------------------------------------------------------------------
 * ntoskrnl/ke/i386/kiinit.c:738 — `NTAPI KiSystemStartup(IN PLOADER_PARAMETER_BLOCK
 * LoaderBlock)`. NTAPI == __stdcall on i386, so the callee expects, AT ENTRY:
 *   [esp+0] = return address
 *   [esp+4] = LoaderBlock
 * ntoskrnl/ke/i386/mproc.c:28-32 defines exactly that shape:
 *   typedef struct _AP_SETUP_STACK { PVOID ReturnAddr; PVOID KxLoaderBlock; }
 * ...and mproc.c:136-139 PRE-BUILDS it on the AP's kernel stack BEFORE start:
 *   ProcessorState->ContextFrame.Esp -= sizeof(AP_SETUP_STACK);
 *   ApStack = (PAP_SETUP_STACK)ProcessorState->ContextFrame.Esp;
 *   ApStack->KxLoaderBlock = KeLoaderBlock;
 *   ApStack->ReturnAddr    = NULL;
 * So PARAM.stack (== ContextFrame.Esp) ALREADY points at a fully-formed
 * stdcall-entry stack image. The trampoline must NOT push LoaderBlock itself
 * (that would double it / misalign) — it only has to (1) load ESP = PARAM.stack
 * verbatim and (2) transfer control to KiSystemStartup via a MECHANISM THAT DOES
 * NOT PUSH ANYTHING ELSE ONTO THE STACK below what's already there, i.e. NOT a
 * `call`. `retf` (far return) is exactly right: we `push seg_cs; push entry;
 * retf` — retf POPS those two back off (net stack effect zero beyond the two
 * pushes it consumes), so once it lands in KiSystemStartup, ESP is back to
 * PARAM.stack unchanged — i.e. [esp]=ReturnAddr(NULL), [esp+4]=LoaderBlock,
 * precisely the AP_SETUP_STACK / stdcall-entry shape KiSystemStartup expects.
 * This mirrors aptramp64.S's `push seg_cs; push entry; lretq` structure exactly;
 * the only amd64-specific bit that does NOT carry over is passing arg0 in a
 * register (rcx) — i386 NTAPI passes args on the stack, and mproc.c already put
 * it there, so aptramp32.S has nothing analogous to amd64's `mov rcx, [arg0]`.
 * PARAM.arg0 is still populated by trampoline.rs (mission ask, "harmless") but
 * genuinely unused by this file — see fill_params_from_state's i386 arm.
 *
 * Position independence: identical scheme to aptramp64.S — assembled at
 * origin 0; the 16-bit prologue computes the page base (CS<<4) into EBX and
 * patches the one absolute far-pointer operand (the sole 16->32 transition;
 * i386 has no long-mode leg so there is only one far jump target, unlike
 * amd64's two) in place before using it.
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

    /* Patch the one absolute operand with the runtime base. */
    movl    %ebx, %eax
    addl    $tmp_gdt, %eax
    movl    %eax, tmp_gdtr+2        /* GDTR.base = P + tmp_gdt      */
    movl    %ebx, %eax
    addl    $pm32, %eax
    movl    %eax, farptr32          /* far ptr offset = P + pm32    */

    /* Protected mode with the flat temporary GDT. */
    lgdtl   tmp_gdtr
    movl    %cr0, %eax
    orl     $1, %eax                /* CR0.PE */
    movl    %eax, %cr0
    ljmpl   *farptr32               /* serialize; -> 0x08:pm32      */

/* ---------------------------------------------------------------- 32-bit */
    .code32
pm32:
    movw    $0x10, %ax              /* flat data (tmp GDT)          */
    movw    %ax, %ds
    movw    %ax, %es
    movw    %ax, %ss

    /* ESI = linear param block.  (EBX survived the mode switch.) */
    leal    PARAM(%ebx), %esi

    movl    0x10(%esi), %eax        /* kernel CR4                   */
    movl    %eax, %cr4
    movl    0x08(%esi), %eax        /* kernel page-dir (low 32)     */
    movl    %eax, %cr3

    movl    0x00(%esi), %eax        /* kernel CR0 (PE|PG) -> paging */
    movl    %eax, %cr0
    /* Identity-mapped stub page (design-(b) dependency, see header) keeps
     * us executing at the same linear address through this write — no far
     * jump needed here (unlike amd64's PE->long-mode transition): we're
     * already 32-bit protected mode before AND after enabling paging. */

    /* Install the real per-AP GDT/IDT. These are KERNEL VIRTUAL addresses
     * (APInfo->Gdt/Idt, pool-allocated — mproc.c:94-95); they are only
     * dereferenceable now that paging is on with the kernel CR3. Loading
     * the GDTR/IDTR *value* itself never dereferences memory, but the
     * segment/TR reloads below do, so paging must already be enabled —
     * which it now is. CS is still running on the OLD (tmp) cached
     * descriptor; that's fine, matches aptramp64.S's lm64 sequencing. */
    lgdt    0x18(%esi)
    lidt    0x28(%esi)

    /* LTR: mproc.c ALREADY clears the AP's own TSS busy bit in software
     * before this ever runs (mproc.c:102-103, `Type = I386_TSS`) — unlike
     * amd64, which snapshots a LIVE (already-busy) BSP GDT. Runtime-clear
     * here anyway, defensively, so this trampoline is correct even if a
     * future mproc.rs change stops pre-clearing it (belt-and-suspenders,
     * same shape as aptramp64.S; idempotent if already clear). */
    movzwl  0x44(%esi), %eax        /* tr selector                  */
    andl    $0xFFF8, %eax           /* descriptor index offset      */
    movl    0x1A(%esi), %edx        /* gdt base (low 32)             */
    addl    %eax, %edx
    andb    $0xFD, 5(%edx)          /* type.busy (bit 1 of byte 5)  */
    movw    0x44(%esi), %ax
    ltr     %ax

    /* Data segments. */
    movw    0x3A(%esi), %ax
    movw    %ax, %ds
    movw    0x3C(%esi), %ax
    movw    %ax, %es
    movw    0x3E(%esi), %ax
    movw    %ax, %ss
    movw    0x40(%esi), %ax         /* KGDT_R0_PCR                  */
    movw    %ax, %fs
    movw    0x42(%esi), %ax
    movw    %ax, %gs

    movl    0x50(%esi), %esp        /* AP kernel stack (AP_SETUP_STACK
                                        already built on it — see header) */
    pushl   0x58(%esi)
    popfl                           /* kernel EFLAGS (IF=0)         */

    /* Far-return into the kernel's code selector at KiSystemStartup.
     * NOTE: no push of LoaderBlock here — it's already baked into the
     * stack image at PARAM.stack by mproc.c; see the header note. */
    movzwl  0x38(%esi), %eax        /* seg_cs                       */
    pushl   %eax
    pushl   0x48(%esi)              /* entry (KiSystemStartup)      */
    lretl

/* ------------------------------------------------------------- data ---- */
    .balign 8
tmp_gdt:
    .quad   0                       /* null                         */
    .quad   0x00CF9A000000FFFF      /* 0x08: flat 32-bit code       */
    .quad   0x00CF92000000FFFF      /* 0x10: flat 32-bit data       */
tmp_gdt_end:

tmp_gdtr:
    .word   tmp_gdt_end - tmp_gdt - 1
    .long   0                       /* patched: P + tmp_gdt         */

farptr32:
    .long   0                       /* patched: P + pm32            */
    .word   0x08

/* The blob must stay clear of the param block at 0x800 — enforced by the
 * regen script (size check on the flat binary). */
