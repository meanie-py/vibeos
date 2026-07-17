# De-transliteration to idiomatic Rust — the trampoline / idiomization program

**Status:** DESIGN / research note (2026-07-16). Captures a design conversation, not yet a
lane. This is the concrete plan behind roadmap **§2c (C-Shaped Rust → Idiomatic Rust)** and
**§3 (Pipeline Stage 3)** — it makes those aspirational entries executable. Companion note:
[asm-safety-design.md](asm-safety-design.md) (the "shrink the asm to nothing testable-adjacent"
move here *is* the asm-safety lever there).

## The problem, quantified

The tree is ~3.9M LOC of hand-written translated Rust (+1.5M generated `-sys`). It's a
faithful C transliteration, so `unsafe` is the *medium*, not the exception:
- ~206,800 `*mut`/`*const` raw pointers (hand-written), ~147K `unsafe {}` blocks.
- The c2rust idiom is **per-statement** `unsafe {}` wrapping: e.g. `cdrom-rust` has 346
  translated fns but 16,202 inner `unsafe {}` blocks — ~47 per function.
- Safety discipline is ~absent: **22** `// SAFETY:` comments across 5.4M LOC, 0
  `#![forbid(unsafe)]`.

"Reduce unsafe" is not a meaningful goal in a kernel — the ceiling is ~0% safe in the FFI and
driver layers. The tractable goals are **contain** it (the way the algorithm crates already
do) and **document** the invariants where soundness actually rides.

## Phase structure — two phases, decoupled (this is the key refinement)

rustconv's Stage 3 was framed as *one* per-function LLM pass (`_rs` with Result AND idiomatic
params together). Splitting it is strictly better:

### Phase 1 — the trampoline breakout (mechanical, free, behavior-preserving)

An **AST transform**, not an LLM task. Every export splits in two:

```rust
// BEFORE
#[no_mangle] pub unsafe extern "system" fn Foo(a: PIRP, b: PDEVICE_OBJECT) -> NTSTATUS { <body> }

// AFTER
#[no_mangle] pub unsafe extern "system" fn Foo(a: PIRP, b: PDEVICE_OBJECT) -> NTSTATUS {
    Foo_rs(a, b)                    // thin trampoline: keeps the ABI, owns the unsafe boundary
}
fn Foo_rs(a: PIRP, b: PDEVICE_OBJECT) -> NTSTATUS { <body> }   // impl, same sig, pass-through
```

Properties that make it landable in bulk:
- **Deterministic** (syn/tree-sitter), so it's a `rustconv breakout <crate>` subcommand — no
  API spend, unlike an LLM pass.
- **Behavior-preserving** — Phase 1 changes nothing at runtime, so it lands across all
  ~20–24K exports (23,743 `#[no_mangle]`) as one big **gate-once** operation.
- Establishes the seam *everywhere* so Phase 2 is incremental and optional per function.

Even for functions that never idiomize, Phase 1 still pays: it **concentrates the unsafe** at
the trampoline and gives every function **one canonical home for a `// SAFETY:` contract** —
which closes the discipline gap *before* any Result conversion. That is the sleeper value.

### Phase 2 — idiomization of `_rs` (incremental, optional, per function)

Once the seam exists, idiomize `_rs` where it pays and leave the rest as pass-through forever.
Target shape (already pinned in `rustconv-pipeline.md` §Stage 3):
`fn Foo_rs(...) -> Result<(T, NTSTATUS), NTSTATUS>` — **preserve the exact NTSTATUS**; carry
informational-success codes (`STATUS_PENDING`, `STATUS_MORE_ENTRIES` are not errors), never
collapse to `Result<T, ()>`. Raw ptr → `&T`/`&mut T`/`Option<&T>`/`NonNull<T>` where the
object model permits (see the ceiling below).

## The reachability ceiling — which `_rs` can actually idiomize

~99% of the raw pointers are the **NT object graph**, not hardware (MMIO/ports/physaddr are
~0.9% — ~1,850 sites, confined to HAL + device drivers). The object graph splits by one test:
**does a compiled driver binary read this field at a fixed byte offset?**

| | Examples (site counts) | Idiomatizable? |
|---|---|---|
| **Public DDK ABI** — driver-facing, fixed layout | `DeviceExtension` 15,584; `IO_STACK_LOCATION` fields 4,105; `IoStatus` 2,142; `IoGetCurrentIrpStackLocation` 1,728; MDL macros, `FsContext` | **No** — binary contract with code we don't recompile; raw stays |
| **Private plumbing** — ntoskrnl-only | `HANDLE_TABLE` 1,672; `MMVAD`/VadRoot 197; `MMPFN` 71; `KWAIT_BLOCK` 62; `OBJECT_HEADER` 44 | **Yes** — no external ABI; current raw treatment is translation inertia |

The layout is fixed *in practice* for three compounding reasons even though we recompile our
own drivers: (a) binary Windows-driver compat is the project's reason to exist; (b) C
consumers still exist in-tree and pin the layout to the DDK headers; (c) the `-sys` bindgen
`size_of` assertions nail it to those headers.

**So Phase 2's reachable core is bigger than "leaf functions":** it's every function operating
on a *purely-internal* object (Mm/Ob/Cc/Ke private state) — those `_rs` can take `&mut Vad`,
`&PfnEntry`, safe handle-table refs *now*, with no membrane. The raw residue concentrates in
the **driver-facing I/O path** (IRP/DEVICE_OBJECT/PnP), where the ABI + genuine aliasing both
bite and `_rs` stays raw.

## The façade / DTO experiment — for the driver-facing objects (the harder half)

You do **not** have to store the driver-facing object in our internal representation. A
**handle/DTO façade**: keep an idiomatic internal object, hand the driver a `#[repr(C)]` ABI
**mailbox** with the DDK layout, and reconcile at the API seams we own. The block's *address*
can key our internal object (back-pointer in a private field, O(1)).

**Soundness test:** the façade is sound iff **the kernel never has to *observe* a driver's
field-poke without an intervening API call we own.** By that test:
- **IRP passes** — it's a message. Every consequential transition routes through an owned API:
  `IoCallDriver` (730) down, `IoCompleteRequest` (937) up, `KeInsertDeviceQueue`/`IoStartPacket`
  (65) for queueing, and even async cancel is gated by `IoAcquireCancelSpinLock` (297). Pure
  field-pokes (`IoMarkIrpPending` — a macro, 281; filling `IoStatus`) are local scribbles
  observed at the next owned call. IRP is the sweet spot.
- **DEVICE_OBJECT is weaker** — the driver reads `DeviceExtension`/`Flags` continuously with no
  transaction boundary to hook reconciliation to. Long-lived node, not a message.
- **KTIMER/KDPC/KSPIN_LOCK fail** — read by our interrupt/DPC dispatch *asynchronously*, no API
  call in between. The block *is* the coordination variable; can't be a pure DTO.

**Cost, honestly:** a façade institutionalizes two representations of one object that must stay
bug-for-bug consistent — the "two views drifted" class (§9.13 PFN union, §9.41 TEB offset, the
MMPFN shift). At this project's throughput (~4M LOC / 6 mo / ~$950 total) that drift is cheap
to recover from — a few agent-hours, not a gate on the design — **but** if built, the
reconciliation layer must ship with a debug-mode invariant-checker asserting DTO↔internal
agreement at every seam. Recommendation: **prototype on the IRP alone, behind a feature flag,
invariant-checker first**, measure the hot-path cost before it's policy. Not a big-bang.

## Why this collapses the hot paths at compile time (the payoff)

Idiomizing IO/IRQL/context-switch with traits + RAII/typestate guards lets the compiler
collapse indirection that's currently runtime — *on our side of the ABI*. It's already
underway:
- IRQL primitives are already inline, not opaque exports (**42 `#[inline]` vs 10
  `#[no_mangle]`** for `Kf(Raise|Lower)Irql`).
- RAII guards (`IrqlGuard`/`SpinLockGuard`/`PfnLockGuard`) are deployed kernel-wide (dozens of
  sites across ke, mm/ARM3, iomgr, ex, ps, po, netstack, kd64; canonical in
  `ntkrnl-sys/src/guards.rs`).

**The design rule that makes it work** (and dodges the firewall): the workspace runs
`lto = false` + `codegen-units = 256` (the captures(none)/ABI-mismatch firewall), which blocks
cross-crate inlining of `#[no_mangle]` functions — **but generics and `#[inline]` still
monomorphize/inline across CGU boundaries at the use site.** So:

> **Hot-path primitives are generic/`#[inline]` so they cross the CGU firewall and collapse;
> `#[no_mangle]` is reserved for the ABI wall.**

That single rule *is* the trampoline shape: the `#[no_mangle]` trampoline is the wall (the 504
`MajorFunction[]` indirect calls into separately-compiled drivers — irreducible), and the
generic/inline `_rs` interior collapses. The delta to the *full* collapse is small: today's
guards are RAII (Drop-based release, already inlined); the compile-time *elision* (dropping
redundant raise/lower, making IRQL discipline a compile error) needs them to carry the level
in the *type* — `IrqlGuard<DISPATCH>` typestate with a const level. That's an upgrade to a
deployed pattern, not a rewrite.

**The floor** — two things never collapse: (a) the asm *mechanism* (the context-switch
register/stack swap and the CR8/MSR IRQL write are the same instructions whatever the types —
traits collapse the *policy* around them, not the mechanism; see the asm-safety note), and
(b) the 504 driver indirect calls.

**Second-order bonus:** idiomatic *owned* interior objects are `noalias` to LLVM; the current
shared-pointer graph is not. So the façade's internal representation isn't only a safety win —
it's what lets the optimizer stop assuming everything aliases and optimize the interior harder.
Compile-collapse and façade are the same lever from two angles.

## The arm64 / x86 asymmetry (the strongest argument for the internal/external split)

The driver fleet is **arch-shared source** (0 `cfg(target_arch)` splits — same files on all
three arches). So arm64 isn't free today; it's **shackled to x86's WDM/IRP model because it
runs the same source.** Representation == ABI means arm64 carries x86's driver-compat baggage
for a driver ecosystem that, on arm64, *does not exist.*

The internal/external split is the only thing that lets arm64 diverge while sharing a codebase:
x86 keeps the membrane (the binary ecosystem is the point), arm64 runs a thin-or-absent
membrane over the *same* idiomatic core. Clear-eyed caveat: arm64 having no legacy drivers cuts
both ways — freeing it makes *our own* arm64 drivers nicer (maintainability/safety), it does
not unlock "a world of arm64 Windows drivers" (there isn't one). Worth wanting, smaller than
the x86 binary-compat constraint on the other side.

## Sequencing (keep these three phases distinct)

1. **(a) Finish translating what exists** — the through-line (roadmap Tier 3).
2. **(b) Idiomize to how we want it — ABI-preserving.** Trampoline breakout, guard typestate,
   trait-collapse, and the optional IRP façade all live here. Still XP's WDM at the boundary,
   idiomatic behind it. **This doc.**
3. **(c) Driver-model XP → Win10 (WDF etc.)** — a *separate later project* and the only one
   that actually moves the ABI. Do NOT conflate it with (b); the façade needs none of it.

## Economics footnote (why the risk framing is "cheap to recover," not "gate")

~4M LOC translated in <6 months on a ~$200/mo Claude subscription + ~$750 API (sonnet +
deepseek). At that throughput the drift/tarpit bugs are rounding error — the aggressive
translate-then-fix loop dominates careful-design-upfront. Build the invariant-checker as cheap
insurance, but don't let drift risk gate the architecture.
