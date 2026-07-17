# Inline-asm safety — regimes, a constraint linter, and how to test kernel-mode asm

**Status:** DESIGN / research note (2026-07-16). Captures a design conversation, not yet a
lane. Companion to [idiomatic-rust-idiomization-design.md](idiomatic-rust-idiomization-design.md)
— the "shrink the asm to nothing testable-adjacent" move below *is* the trampoline/`_rs` lever
from that note, applied to asm. Roadmap neighbors: §4 (Testing Infra), §5a (Runtime Invariant
Checking).

## What the asm is (~850 `asm!`/`global_asm!` blocks)

Concentrated (survey: ntoskrnl-rust 210, hal-rust 147, ntkrnl-sys 85, freeldr 77). By class:

| Class | ~count | For |
|---|---|---|
| CR read/write (`mov crN`) | 406 | paging (CR3), IRQL-via-CR8, CR0/CR4 mode bits |
| cli/sti/hlt/invlpg/swapgs/iret | 177 | interrupt masking, TLB, entry/exit |
| port I/O (`in`/`out`) | 118 | legacy device access |
| barriers (mfence/dmb/dsb/isb) | 109 | ordering |
| PCR / thread-reg (`gs:`/`fs:`/`tpidr`) | 103 | per-CPU / per-thread base pointer |
| MSR (rd/wrmsr) | 84 | APIC base, EFER |
| atomics (cmpxchg16b/ldxp) | 16 | only the exotic ones — rest use `core::sync::atomic` |

## The two regimes (the whole safety story)

- **Pure / value-effectful** — barriers, atomics, port-I/O logic, byteswap/bitscan. Compute a
  value or do a well-defined memory op. **Testable.**
- **Privileged / machine-effectful** — CR/MSR writes, cli/sti/invlpg/swapgs/iret, context
  switch, PCR access. They `#GP` in ring 3, and the "output" is *global CPU state* (TLB
  flushed, address space switched, interrupts masked), not a return value. **Not classically
  testable.**

## Making it safer — the real bug surface is constraint discipline

The danger is not the scary instructions; it's a **wrong clobber/option**, because the
compiler then reorders or reuses a register around asm that actually stomped it. 674/850 blocks
carry `options(...)`. The canonical scar (AGENTS.md): the CMPXCHG16B `rbx` bug — LLVM reserves
`rbx`, the asm clobbered it via the xchg workaround, a 32-bit write silently zero-extended and
destroyed the upper half of a caller's pointer → multi-hour BUGCHECK 0x50. One under-declared
clobber.

### Highest-value new tool: an asm-constraint linter (the `verify_ffi` of asm)

Parse every `asm!` block, cross-check instructions against declared options/clobbers. Rules:
- `mov crN` / `invlpg` / `wrmsr` (change memory-visible state) must **not** be `nomem`.
- `cli` / `sti` / `popf` (touch flags) must **not** be `preserves_flags`.
- anything using the stack must **not** be `nostack`.
- anything clobbering LLVM-reserved `rbx`/`rbp`/`ebx` must go through the xchg workaround.

Every rule is mechanically checkable and catches exactly the class that has cost hours.

### Two supporting moves

- **One wrapper per instruction.** 406 CR references should bottom out in a handful of
  `__write_cr3()` / `__read_cr8()` wrappers — collapse the audit surface from ~850 sites to
  ~40 primitives. (Already concentrated in ntoskrnl/hal, so this is dedup, not a rewrite.)
- **Prefer `core::arch` intrinsics** where they exist (`__cpuid`, fences, atomic intrinsics) —
  then the *compiler* owns the constraints. The 16 hand atomics are the genuinely-no-intrinsic
  ones (CMPXCHG16B/LDXP); the rest should be `core::sync::atomic`.

## Testing — is this a unit-test target?

**Regime 1 (pure): yes, and it already is.** `slist` runs `cargo test` + **Kani** (exhaustive
model check) + **loom** (all thread interleavings) against its LDXP/CMPXCHG16B. Barriers,
byteswap, CRC, bitmap: differential-test the asm against a pure-Rust reference on random inputs.

**Regime 2 (privileged): you can't run `mov cr3` in `cargo test`** — and even if you could,
there's no return value to assert; the effect is on the machine. Four techniques, by leverage:

1. **Shrink the asm to the irreducible instruction; test the logic around it.** Almost every
   privileged "asm function" is `compute a value (pure) → commit it with one instruction
   (untestable)`. `KeSwapIrql` = an IRQL→TPR table lookup (pure, Kani-able) + a 1-instruction
   `mov cr8` tail. Push the logic into a pure `_rs`, unit-test *that*, and the untestable
   surface collapses to a byte or two you review by eye. **This is the trampoline/`_rs` lever
   from the idiomization note** — the biggest win.
2. **QEMU ring-0 harness — already exists.** The 28 `test_kernel.py` tests boot a real kernel
   and dispatch over serial. *That* is where the actual `cli`/`cr3`/`swapgs` run in ring 0 and
   get asserted. The answer to "how do you unit-test a privileged instruction" is: you don't —
   you have a booted ring-0 test kernel, and this project already built one. Underused for the
   privileged asm specifically.
3. **"Instruction survived optimization" gate — already exists.** `verify_sacred_primitives.sh`
   checks that `KfRaiseIrql`/`KeAcquireSpinLock`'s real instructions are present in the compiled
   binary (LLVM didn't elide/miscompile them); `scan_ud2.py` catches proved-UB. Extend the
   sacred list to the CR3 write, the barriers, swapgs → a regression gate that the instruction
   is still there and intact.
4. **Golden-encoding tests** for cases where exact *bytes* matter — SList Pop's `0x8B 0x18`
   that `KiCheckForSListFault` pattern-matches (AGENTS.md). Assemble and assert on the encoding
   at build time.

## The program, in one line

The asm is safer than it looks because it's small and concentrated; the real risk is
constraint/clobber correctness (**linter-catchable**); the pure half is already Kani/loom
tested; and the privileged half's answer is **shrink it to nothing testable-adjacent + run the
residue in the QEMU ring-0 harness + gate that it survived the optimizer.**
