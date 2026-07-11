# Execution-context ownership — the CPU as one owned resource

**Status:** design v2 (2026-07-06). The static counterpart to the dynamic
`cswitchdep-design.md`. Generalizes the in-flight IRQL RAII spike
(`ntkrnl-sys/src/guards.rs`, `rust-kernel-patterns.md`) and the planned
shadow-IRQL check (`sync-instrumentation-design.md` §6) into a single ownership
model that makes context-switch-safety a compile-time property.
v2 incorporates an adversarial review: the `-> RunToken<'new>` signature was
**unsound as written** (caller could unify the brands), the by-value `wait_for`
outlawed the kernel's own legal wait protocols (WaitNext/signal-and-wait,
APC-level waits), and Layer 2 had no capture set to bound without CPS. All three
are repaired by one structural change — **`wait_for` is continuation-passing** —
plus a token typestate. Details below, marked v2.

## The one idea

**A kernel context switch is an `.await` point.** At `.await`, an async task
yields, something else runs on the thread, and when it resumes the world has
moved — so Rust forbids holding a `!Send` value across it. A kernel context
switch is the same event: the CPU stops running this thread, runs another, and
resumes you later. Everything that is only valid "while this frame is
executing" — a spinlock guard, a raised-IRQL token, a per-CPU (`KPCR`/`KPRCB`)
pointer, a `KeGetCurrentThread()` reference — is the kernel's `!Send`: it must
**not** survive a *voluntary* switch.

So we model **the right to run on this processor as a single owned linear
resource.** Raising IRQL, disabling preemption, and acquiring a spinlock all
*borrow* it. A voluntary context switch is the one operation that *gives it
back*. The borrow checker then makes the classic kernel bugs uncompilable:

| Async Rust | Kernel | Enforced by |
|---|---|---|
| `.await` suspension | voluntary wait (`KeWait*`, `KeDelay*`) | switch point consumes the run token (CPS) |
| `Send` (safe to move across threads) | **`SwitchSafe`** (safe to hold across a switch) | auto-trait leakage on the **continuation's captures** |
| `!Send` `MutexGuard` can't cross `.await` | spinlock guard / per-CPU ref can't cross a wait | `!SwitchSafe` marker + bound on the continuation |
| `Future` fields = what's live across await | continuation captures = what survives the wait | the CPS closure *is* the reified capture set |
| holding a lock across `.await` → `!Send` future → `spawn` rejects | sleeping with a spinlock held → won't compile | the same auto-trait obligation |

Two scoping notes (v2):
- **Voluntary only.** An *involuntary* preemption or a trap cannot "consume" a
  type — the preempted frame's token stays frozen with it, which is sound
  because the token is `!Send` and unusable while descheduled. Preemption-window
  invariants belong to Layer 3 and cswitchdep, not the token.
- **The analogy's load-bearing part is the capture set.** `.await` works because
  the coroutine transform reifies live locals into a type the auto trait leaks
  through. Kernel code has no coroutine transform, so a *plain function call*
  `wait_for(tok, obj)` has no capture set to inspect — a caller's local guard
  live across the call appears in no type any bound can see. **CPS is not
  sugar; it is the only thing that gives Layer 2 semantics** (§Layer 2).

This is the same static machinery Rust ships for async, repurposed. Prior art
converges on it from three directions (Theseus's `!Send` `PreemptionGuard`,
RTIC's compile-time priority ceilings, Rust-for-Linux's klint preempt-count
analyzer), and the design below is their synthesis mapped onto ntoskrnl.

## Toolchain reality (v3 — 2026-07-07: STABLE-EVERYTHING is the direction)

**Decision (user, 2026-07-07): perma-unstable features are not a viable
foundation; the goal is stable-for-everything.** That kills Layer 2 as
designed (see its section) and reframes this section as the *inventory of
remaining nightly dependencies*, each with its exit path:

1. ~~`link_llvm_intrinsics` (lockdep/cswitchdep `llvm.returnaddress` culprit
   capture — the arm64 E0554 wall)~~ — **KILLED 2026-07-07**: replaced by
   the stable `code_addr!()` inline-asm macro (`ke/lockdep.rs`,
   crate-exported — `lea [rip]` / `call;pop` / `adr .` per arch), with `pc`
   threaded through the tap signatures. (An intermediate naked-fn
   frame-record shim reading `[rbp+8]` was tried and is DEAD: Windows-x64
   clang builds no classic frame chain even with `"frame-pointer":"always"`
   — measured selftest pc=0. Details in cswitchdep-design.md.) Side effect:
   **arm64 arms lockdep/cswitchdep on its native stable channel** — the
   build-cli fail-fast is lifted.
2. **`-Z build-std` + custom JSON targets (x86 kernels)** — the structural
   gap: i386/amd64 build `i686-kernel.json`/`x86_64-kernel.json` on nightly;
   arm64 already builds the builtin `aarch64-pc-windows-gnullvm` on stable.
   Exit path (unproven, needs a spike): builtin `i686/x86_64-pc-windows-gnu`
   + per-crate RUSTFLAGS for the kernel constraints (soft-float/no-SSE,
   kernel code model, no-red-zone is already the Windows x64 ABI) — but
   mixing `-C target-feature` with a precompiled std is unsound for float
   ABI, so it likely still wants build-std → the honest blocker is build-std
   stabilization or a vendored core. Track upstream; spike when idle.
3. **`c_variadic`** (~12 x86-only driver crates: scsiport, uniata, usbport,
   storahci, …) — still unstable on 1.92 (checked). Coupled to gap 2: those
   crates only build into the x86 kernels, which are nightly anyway. Becomes
   actionable the day gap 2 closes; the printf-shape entry points could be
   rewritten to fixed-arity thunks then.
4. **`linkage`** (`hal-cargo/src/main.rs:12`) — weak-symbol cohab. Needs its
   own small design (lld `/alternatename` or explicit strong stubs) — owed.

Historical (v2) analysis of the pin options, kept for the record: an
unpinned nightly means perma-unstable churn breaks every lane at once with no
code change; a pin (rust-toolchain.toml) trades that for a bump ritual (rerun
scan_ud2 / captures audit / sacred-primitives per bump) and drags arm64 off
stable; `RUSTC_BOOTSTRAP=1` on pinned stable is the smaller pin but
explicitly unsupported. All moot for Layer 2 now — rejected — and the
remaining nightly surface (gap 2) carries no perma-unstable *language*
features, so an unpinned nightly for x86 kernels stays tolerable until the
spike lands.

3. (v2 note, still true) **There is no stable Layer 2 auto-trait.** Evaluated
   and rejected: `Send`-as-proxy —
   the translated kernel is wall-to-wall raw pointers (`PKTHREAD`, `PKMUTANT`
   locals in nearly every function), all `!Send`, so a `Send` bound
   false-rejects essentially every real wait site and breeds a `SendPtr`
   wrapper epidemic that makes the check vacuous. Sealed-trait-with-blanket-impl
   — stable has no negative reasoning; nothing can be excluded. The honest
   stable fallback is **Layers 1 + 3 + the wait-site capture lint** (below).

## Three layers

Layers 1–2 are library code, no compiler fork (Layer 2 needs the nightly
features above); layer 3 is an external analyzer for the one property types
genuinely cannot express.

```
Layer 1  RunToken   — the linear "right-to-run" resource: a !Send, branded,
                      TYPESTATED RAII token. IRQL-raise / spinlock /
                      preempt-disable borrow it; a voluntary wait consumes it
                      (CPS); per-CPU accessors demand it.
Layer 2  SwitchSafe — an auto-trait mirroring Send-across-await, bounded on the
                      WAIT CONTINUATION's captures. Nightly-only (see above).
Layer 3  irqlck     — call-graph IRQL adjust/expect checking: first as a
                      cross-function extension of the existing tree-sitter scan,
                      klint-style MIR only if that proves insufficient.
```

They compose with what already exists: the `guards.rs` RAII guards
(`IrqlGuard`, `SpinLockGuard`, `SpinLockAtDpcGuard`) are layer-1 tokens
*without the brand, the typestate, or the `!Send` marker yet*; adding those is
most of layer 1.

## Layer 1 — `RunToken`, the linear right-to-run

### The resource

Today `guards.rs` has saved-value RAII guards: `IrqlGuard{previous: KIRQL}` raises
on `raise()` and lowers on `Drop` (`guards.rs:103-139`), `SpinLockGuard` likewise
(`:152-182`), `SpinLockAtDpcGuard` (`:184-208`). They preserve the *ordering* of
transitions against LLVM (the module doc: guards "encode kernel invariants … as
visible values that LLVM's optimizer must preserve"), but they carry **no
compile-time facts** — no "you are at DISPATCH," no "you hold the CPU," and they
are freely `Send`. Layer 1 adds exactly those.

```rust
/// The right to run on the current processor while this frame is executing.
/// Linear within a Rust region (see Minting): you get one, you can borrow it,
/// and a voluntary wait consumes it. `!Send`/`!Sync`. Branded with an invariant
/// lifetime so per-CPU references minted from it cannot outlive it. TYPESTATED
/// by level so "wait from the wrong context" is unrepresentable.
pub struct RunToken<'brand, L: Level = Passive> {
    _brand: InvariantLifetime<'brand>,   // PhantomData<fn(&'brand ()) -> &'brand ()>
    _not_send: PhantomData<*const ()>,   // !Send + !Sync
    _level: PhantomData<L>,
}

/// Typestates. v1 carried a runtime `entry_irql: KIRQL` field — dead weight
/// inside a compile-time capability. v2 moves the level into the type, which is
/// what lets the wait API accept exactly the legal states (below).
pub trait Level {}
pub struct Passive;   impl Level for Passive {}
pub struct Apc;       impl Level for Apc {}       // waits still legal (WaitIrql)
pub struct Dispatch;  impl Level for Dispatch {}  // zero-timeout polls only
pub struct Interrupt; impl Level for Interrupt {} // ISR/DPC-minted: NO wait API
```

`InvariantLifetime` is the GhostCell brand trick verbatim: a lifetime used in
both argument and return position of a `PhantomData<fn(...)->...>` is invariant,
so `RunToken<'a>` never coerces to `RunToken<'b>` (not even `'static`) — a brand
is a genuine capability, zero-sized, no code.

### Raising IRQL / holding a lock = borrowing the token

```rust
impl<'b, L: Level> RunToken<'b, L> {
    /// Raise IRQL. Returns a guard that borrows the token for its lifetime.
    pub fn raise(&mut self, new: KIRQL) -> IrqlGuard<'_, 'b> { … }

    /// Acquire a spinlock; nests IRQL to the lock's level (RTIC "ceiling").
    pub fn acquire<'g>(&'g mut self, lock: PKSPIN_LOCK) -> SpinLockGuard<'g, 'b> { … }
}
```

Because `raise`/`acquire` take `&mut self`, **the guard exclusively borrows the
`RunToken` for its whole scope.** Anything else that needs the token — most
importantly a voluntary wait — cannot be called while a guard is alive. "No
sleeping with a spinlock held" becomes a borrow-checker error, not a lockdep
runtime assert.

**Implementation trap (found by compile-fail probe, 2026-07-07, P2):** a
token-borrowing guard whose only tie to `'g` is `PhantomData<&'g mut ()>` does
**not** enforce this — dropck only requires a value's lifetime params to be
live at its drop when the type itself `impl Drop`, so NLL ends the token loan
immediately and `wait_for(tok, …)` with a live guard *compiles*. The wrapper
guards therefore carry a load-bearing **empty `Drop` impl**
(`ntkrnl-sys/src/token.rs`, `TokenIrqlGuard`/`TokenSpinLockGuard`), which
extends the loan to the scope end and restores the E0505. Any future
token-borrowing guard must do the same — and re-verify with the probe (hold a
guard across `wait_for`, expect E0505).

### A wait consumes the token — **continuation-passing** (v2)

v1's `fn wait_for<'old>(tok: RunToken<'old>) -> (NTSTATUS, RunToken<'new>)` is
**not expressible soundly**: a function cannot conjure a fresh invariant
lifetime as a return value — write it as a generic `<'old, 'new>` and *the
caller chooses `'new`*, instantiating `'new = 'old`, and every stale
`Pcpu<'old>` typechecks after the wait. Generativity requires callee-scoped
binding. The correct shape is CPS (or the `generativity` crate's macro, which
is the same thing in macro clothing):

```rust
/// A voluntary wait. Consumes the token; the continuation receives a FRESHLY
/// branded one (higher-ranked 'new — the callee binds it, the caller cannot
/// unify it with 'old). Everything live across the wait is a capture of `k`,
/// which is where the SwitchSafe bound (Layer 2) attaches.
pub fn wait_for<L: Waitable, R>(
    tok: RunToken<'_, L>,
    obj: &Dispatcher,
    mode: WaitMode,
    k: impl for<'new> FnOnce(RunToken<'new, L>, NTSTATUS) -> R + SwitchSafe,
) -> R { … }
```

Three things fall out:
1. You cannot call `wait_for` while a guard borrows the token → **no wait with
   a spinlock/at raised-by-guard IRQL** (borrow conflict).
2. The continuation's token has a callee-bound `'new` brand → every per-CPU
   reference minted before the wait is a lifetime error after → **no stale
   per-CPU pointer across a switch.**
3. The continuation's **captures are a real type** → Layer 2's bound has
   something to inspect — the exact analogue of `spawn`'s `Send` bound.

**Cost, acknowledged:** every wait site becomes a nested closure (no coroutine
transform to flatten it); loops-around-waits get awkward (a closure can't
`continue` the outer loop — restructure as `loop { match wait_for(…, |t,s| …) }`
returning a verdict). The migration surface is real: ~109 `KeWaitForSingleObject`
references across ~29 ntoskrnl-rust files + ~128 `KeDelayExecutionThread` sites.
Budget it per-subsystem like the `offset_of!` campaign; unmigrated sites keep
the raw FFI calls and Layer-3/cswitchdep coverage.

### The kernel's own wait protocols must stay expressible (v2)

The v1 rule "wait_for cannot be called at raised IRQL" **outlaws legal NT**:

- **Waits at APC_LEVEL are legal and used.** `WaitIrql` exists for this
  (`ke/wait.rs:606` saves it; `:614` gates APC delivery on `< APC_LEVEL` — the
  code *expects* APC_LEVEL callers). `ARM3/pagfault.rs:1230` waits holding the
  working-set guarded region — the MiWaitForInPageComplete shape. Hence the
  `Waitable` bound: `Passive` and `Apc` tokens can wait; `Dispatch` only with a
  zero timeout (a distinct `poll` API); `Interrupt` has no wait API at all.
- **WaitNext / signal-and-wait.** `ex/resource.rs:522` raises to SYNCH, sets
  `Thread->WaitNext = 1`, then calls `KeWaitForSingleObject`, which sees
  `WaitNext` and *skips its own raise* (`ke/wait.rs:603`); same for
  `NtSignalAndWaitForSingleObject` (`ob/obwait.rs:418,428`) where
  `KeSetEvent(obj, inc, Wait=TRUE)` returns *at raised IRQL with the dispatcher
  context armed* and the following wait consumes it. In token terms the signal
  primitive returns a **`WaitArmed<'b>`** typestate — at raised IRQL, carrying
  a live obligation — that `wait_for` alone can consume:

```rust
/// Returned by signal-with-Wait=TRUE primitives. Not Waitable-generic:
/// the ONLY consumer is wait_for; dropping it is a compile hole plugged by
/// #[must_use] + a Drop that feeds cswitchdep in debug builds.
pub struct WaitArmed<'b> { /* brand, !Send, raised level */ }
pub fn set_event_and_arm<'b>(tok: RunToken<'b, Passive>, ev: &Event, inc: KPRIORITY)
    -> WaitArmed<'b> { … }
pub fn wait_for_armed<R>(armed: WaitArmed<'_>, obj: &Dispatcher,
    k: impl for<'new> FnOnce(RunToken<'new, Passive>, NTSTATUS) -> R + SwitchSafe) -> R { … }
```

Without the typestate split, the model is over-strong — it forbids the kernel's
own dispatcher.

### Minting — the FFI boundary, and what "linear" honestly means (v2)

Nearly every migrated function is `#[no_mangle] extern "system"` called *from C*
(`KeWaitForSingleObject` itself is: `ke/wait.rs:580`). C callers cannot pass a
`RunToken`, so exported functions must re-mint one at entry:

```rust
/// SAFETY: caller must be genuine thread context (not ISR unless minting
/// RunToken<Interrupt>), at the level L claims. The sanctioned FFI escape
/// hatch; every use is greppable and Layer 3 verifies mint sites are entry
/// points only.
pub unsafe fn remint_at_ffi_entry<'b, L: Level>() -> RunToken<'b, L> { … }
// idiom: with_token(|tok| { ... }) — scoped, keeps mint sites uniform.
```

Consequence, stated plainly: while the C remnant exists, **linearity is
per-region, not global** — N live tokens per thread (one per Rust region on the
call stack) is the steady state. What is actually enforced is "within a
contiguous Rust call subtree, borrows and brands are checked." That is still
the win (each migrated subtree gets full checking), and it converges to global
linearity as translation proceeds — but v1's implication of one-token-per-thread
was theater, and cswitchdep is the runtime witness across the seams. Internal
(non-exported) functions take `&mut RunToken` — mechanical but large churn;
phase it per-subsystem.

### Per-CPU state is minted from the token and dies with it

The hazard is concrete: `KeGetCurrentThread` is a raw read of `gs:[0x188]` /
`fs:[0x124]` (`ntkrnl-sys/src/mm.rs:2166-2193`) — a cached pointer a switch
invalidates. Gate it behind the token, **split by affinity** (v2 — the two have
different validity conditions):

```rust
impl<'b, L: Level> RunToken<'b, L> {
    /// THREAD-affine: valid for the brand (until the next wait). Survives
    /// preemption — the value follows the thread.
    pub fn current_thread(&self) -> Pcpu<'b, KTHREAD> { … }
}
impl<'b> RunToken<'b, Dispatch> {
    /// CPU-affine: only meaningful while preemption is off, so minting demands
    /// a Dispatch-level token (on SMP, a migrated thread's stale PRCB pointer
    /// is the classic bug; at DISPATCH you cannot migrate). Theseus's
    /// PreemptionGuard-held rule, statically.
    pub fn prcb(&self) -> Pcpu<'b, KPRCB> { … }
}
/// Valid only for the token's brand; !Send. ACCESS API IS RAW TYPED FIELDS
/// ONLY (offset_of!-backed load/store, volatile where required) — NO
/// Deref/DerefMut, ever: C code aliases all of this state, and a &/&mut into
/// it is instant UB. (v2: v1 left the access API unspecified.)
pub struct Pcpu<'b, T> { ptr: *mut T, _brand: InvariantLifetime<'b>, _ns: PhantomData<*const ()> }
```

All field access routes through `offset_of!`/typed bindgen fields, so the
existing `offset_check_*.rs` asserts and `scan_magic_offsets.py` lint cover it
automatically — the branded wrapper and the offset-drift defense reinforce each
other.

### Interrupt context (v2 — the story v1 didn't have)

An ISR/DPC preempts a thread whose token is frozen in its frame, and needs
per-CPU access of its own. It mints `RunToken<'_, Interrupt>` at its entry
(RTIC's answer: each priority level is its own context; Theseus: a guard per
nesting level). `Interrupt` implements no `Waitable`, so **"wait from ISR/DPC"
is unrepresentable**, not merely dynamically checked. The frozen thread-level
token is unusable meanwhile (`!Send`, and the frame isn't executing) — sound by
construction. What Layers 1–2 do *not* claim: anything about the preemption
window itself (what the ISR may touch of the interrupted frame's state) — that
is Layer 3 / cswitchdep territory.

### Composing with what exists, and the escape hatch

- `IrqlGuard`/`SpinLockGuard` gain a `'brand` parameter and the `!Send` marker;
  their `Drop` bodies (lower IRQL / release lock) are unchanged. Existing
  call-sites (`ex/timer.rs:101`, `po/events.rs:131`) migrate mechanically.
- The **linear-consume escape hatch already exists**: `probe.rs`'s
  `disarm(self)` via `core::mem::forget` (`:390`, `:424`) is exactly how you hand
  a token's obligation across an abstraction boundary without firing its `Drop`.

### The hard constraint: split acquire/release sites

The earlier mass-RAII migration was **reverted** (`bb5e8ce6014`) because ~24 KE
sites acquire a lock in one function and release it in another, with the IRQL
parked in a global — a single lexical guard can't span them, and forcing it
destabilized CGU partitioning into ud2 cascades (`rust-kernel-patterns.md:261-284`).
Layer 1 must not repeat that mistake. Two accommodations:
- **Token-in-a-field.** Store the `disarm`ed guard (or its `previous_irql`) in
  the owning object, typed, and reconstitute at release. Be honest about what
  this is: structurally it is the same parked-IRQL shape that motivated the
  model — the obligation is *carried*, not *checked*. **These ~24 sites get
  zero Layer-1/2 protection**; they are Layer 3 + cswitchdep's job. The
  reverted-migration class is not re-covered by types in this design either —
  the types cover everything *around* it and the analyzer covers *it*.
- **Leave them to layer 3** explicitly (the "types for the lifetime half,
  analyzer for the flow half" split Rust-for-Linux landed on).

### Uniprocessor caveat (today) — why the brand still earns its keep

`KeGetCurrentProcessorNumber()` ≡ 0 (`ntkrnl-sys/src/inline/ke.rs:157`): the
kernel is uniprocessor, so there is no CPU *migration* to guard against yet.
The brand's value today is the **"current thread changed across the wait"**
invariant — a `Pcpu<KTHREAD>` minted before a wait pointing at the wrong (old)
thread after it is the live hazard, and the CPS re-brand catches it. On SMP
(v2 correction — v1 overclaimed): the brand alone does **not** become a CPU pin,
because *involuntary* migration never re-brands. The CPU-pinning guarantee
comes from the affinity split above — CPU-affine `Pcpu`s are only mintable from
a `Dispatch`-level token (preemption off ⇒ no migration), with Theseus-style
runtime `cpu_id()` drop-checks as the backstop to add then. Thread-affine
`Pcpu`s (`current_thread()`) are correct under migration with no change.

## Layer 2 — `SwitchSafe`, an auto-trait for "safe across a switch"

> **REJECTED AS DESIGNED (2026-07-07).** `auto_traits`/`negative_impls` are
> *permanently* unstable — no stabilization path — so this layer would be a
> forever toolchain tax. The marginal value didn't justify it: Layer 1's
> `&mut`-borrow already forbids every *borrow* across a wait (guards borrow
> the token `wait_for` consumes — proven by compile-fail probe), leaving only
> *owned* context-bound captures, whose realistic inventory is per-CPU
> handles — a hazard that cannot manifest on today's UP kernel. Replaced by a
> **stable syntactic lint**: `scan_irql_balance.py` `CTX_CAPTURE_ACROSS_WAIT`
> flags a `wait_for`/`wait_for_armed` continuation capturing a local whose
> `let` initializer reads per-CPU state (`KeGetPcr`/`KeGetCurrentPrcb`/
> `Pcpu<`), with the usual 0-NEW baseline gate. What the lint concedes vs the
> auto-trait: deep-struct smuggling and non-local flows — the same class 4a
> already concedes to a future MIR pass; the translated code's wait sites are
> flat C-shaped locals, so the concession is small. Revisit only alongside MP
> work, and only if the features' status changes. The section below is kept
> as the design record.

Rust decides a value is held across `.await` by computing the coroutine's saved
locals, and the generated `Future` is `Send` iff every saved local is `Send` —
auto-trait leakage, inferred, never written. We mimic it — **on the CPS
continuation's captures**, which is the only place a capture set exists (see
"The one idea"):

```rust
#![cfg_attr(feature = "ctx-ownership", feature(auto_traits, negative_impls))] // nightly; pin first

/// Auto-implemented for a type iff all its fields are SwitchSafe — i.e. it is
/// safe to hold across a kernel context switch.
pub unsafe auto trait SwitchSafe {}

impl<'b, L> !SwitchSafe for RunToken<'b, L> {}
impl<'g,'b> !SwitchSafe for IrqlGuard<'g,'b> {}
impl<'g,'b> !SwitchSafe for SpinLockGuard<'g,'b> {}
impl<'b>    !SwitchSafe for WaitArmed<'b> {}
impl<'b,T>  !SwitchSafe for Pcpu<'b,T> {}

// The bound lives on wait_for's continuation (see Layer 1):
//   k: impl for<'new> FnOnce(RunToken<'new, L>, NTSTATUS) -> R + SwitchSafe
```

A guard or `Pcpu` captured by the continuation makes the closure type
`!SwitchSafe` and the bound fails — the exact parity of the
`MutexGuard`-across-`.await` E0277, pointing at the capturing line, and it sees
through struct fields *of captured values* the same way. What Layer 2 does
**not** see: values in the caller's frame that are *not* captured — but those
don't survive the wait either (the continuation is all that runs after), so the
model is exactly aligned with what's live. Layer 1's `&mut`-borrow already
forbids the direct-borrow case; Layer 2 catches ownership smuggling into the
continuation.

(v1 carried a "use block scopes, not `drop()`" caveat imported from the async
world. It does not apply here: that was a coroutine-*layout* artifact, largely
fixed in modern rustc, and Layer 2 has no coroutine — what matters is only
whether the value is *captured by `k`*. For Layer 1's `&mut` borrows, NLL ends
the borrow at `drop(guard)` just as well as a block. Deleted.)

## Layer 3 — `irqlck`, call-graph IRQL checking (v2: phased, klint demoted)

Types encode "the guard exists for this lifetime and is `!Send`." They **cannot**
encode "function X may sleep" across an arbitrary call graph — and the
split-site/~24 and FFI-minted regions are explicitly delegated here. The
contract model is klint's, ported from preempt_count to IRQL:

```rust
#[irqlck(adjust = +1, expect = 0..DISPATCH, unchecked)]   // KeRaiseIrql to DISPATCH
#[irqlck(adjust = -1, expect = 1..)]                       // KeLowerIrql (guard Drop)
#[irqlck(expect = ..=APC)]                                 // KeWaitForSingleObject (WaitIrql!)
```

- **`adjust`** = net IRQL change a function effects; **`expect`** = the IRQL
  range legal at its call sites; propagate through the call graph to a fixpoint;
  a call whose tracked level ∉ callee's `expect` is the violation. Note the wait
  contract is `expect = ..=APC`, **not** `= PASSIVE` — waits at APC_LEVEL are
  legal (see Layer 1), and encoding NT's real rule is the point.
- **Implementation, honestly costed (v2).** klint is a research-prototype
  `rustc_driver` pinned to one specific nightly using unstable compiler
  internals; this repo's nightly is unpinned, and a MIR tool costs a re-port on
  compiler-internal churn per release, forever. Meanwhile the translated code
  is C-shaped — direct calls, no closures/dyn/generics at wait boundaries — so
  a **syntactic call-graph fixpoint gets most of the value at zero toolchain
  coupling**. Hence:
  - **Phase 4a — extend `tools/scan_irql_balance.py`** (tree-sitter; today an
    intraprocedural held-set walk with an honest linear-approximation caveat;
    it already caught the usbport `TransferSpinLock` `goto Exit` leak, and is
    on master + wired into `vibeos-cli check` since 2026-07-06) with
    per-function adjust/expect summaries and a cross-function fixpoint, keeping
    the existing 0-NEW baseline gate.
    **BUILT (2026-07-07), with two lessons the naive form missed:**
    (a) **an uncapped may-sleep fixpoint is useless** — everything touching
    the dispatcher (`KeSetEvent` → `KiExitDispatcher` → the swap) becomes
    "may-sleep" even though it never parks the *caller*; measured 13.6K
    may-sleep functions / 762 findings at fixpoint vs a reviewable 9 at the
    shipped depth cap of 2 (direct wait calls + one Rust wrapper hop,
    `SLEEP_DEPTH`). (b) **wait legality is per-level** — `KeRaiseIrql(
    APC_LEVEL, ..)` holds are leak-checked but NOT sleep-checked (NT's rule is
    waits ≤ APC_LEVEL; the first run flagged `PspUserThreadStartup`'s legal
    APC-level termination path). Shipped summaries: RAISE-WRAPPER (returns the
    OldIrql — e.g. `MiAcquirePfnLock`), LOWER-WRAPPER(arg-i) (38 found, e.g.
    `MiReleasePfnLock`, the USBPORT family), MAY-SLEEP (depth-capped).
    New finding kind `IRQL_SLEEP_WHILE_HELD` = the static mirror of
    cswitchdep's `SLEEP_WITH_SPINLOCK`. The 9 baselined findings are all the
    Mm PFN-lock **callee-releases contract** (`MiDispatchFault` hands
    `LockIrql` to resolvers that drop the lock; the post-call
    `ASSERT(OldIrql == KeGetCurrentIrql())` proves the C contract) — exactly
    the cross-call residue this syntactic phase cannot see through; a 4b MIR
    pass or `#[irqlck]` annotations would clear them. `--summaries` dumps the
    call-graph facts for inspection.
  - **Phase 4b — the klint-style MIR port only if 4a's misses prove material**
    (what MIR sees that syntax doesn't: function pointers — DPC/APC routines,
    IRP completion —, const-prop'd branches, drop-elaboration order). Requires
    the pinned nightly from the toolchain section.
- **Limits & backstop.** Data-dependent acquisition (`KeTryToAcquireSpinLock` —
  adjust "0 or 1") is klint's known gap; those sites are `unchecked`, and
  **cswitchdep is their runtime backstop** (Theseus pairs the same static
  `!Send` with a runtime `cpu_id()` drop-check for exactly this reason).

## How the three layers and cswitchdep compose

```
                compile time                    │        run time
  ┌─────────────────────────────────────────────┼──────────────────────────┐
  │ Layer 1  RunToken borrows + typestate →      │                          │
  │          no sleep-with-spinlock, no stale    │                          │
  │          per-CPU ref, no wait-from-ISR       │                          │
  │ Layer 2  SwitchSafe on the continuation →    │                          │
  │          guard smuggled into the wait's      │                          │
  │          capture set (nightly-only)          │                          │
  │ Layer 3  irqlck call-graph pass → block-at-  │  cswitchdep (dynamic):   │
  │          bad-IRQL across the graph; owns the │  validates the C/asm     │
  │          split-site ~24 + FFI-minted regions │  remnant + `unchecked`   │
  │                                              │  sites + region seams;   │
  │                                              │  farm-soakable           │
  └─────────────────────────────────────────────┴──────────────────────────┘
```

- The IRQL RAII spike in flight *is* the beginning of layer 1 — this doc says
  add the brand + `!Send` + the typestate + the CPS wait API, and reuse
  `disarm`/`forget` for the split sites.
- cswitchdep (`cswitchdep-design.md`) is the runtime backstop and the checker
  for everything the static layers can't see (asm switch, still-C drivers,
  `unchecked` sites, region seams between reminted tokens). Each layer's job
  shrinks as Rust coverage grows: invariants migrate from cswitchdep asserts →
  `irqlck` errors → borrow-check errors, the same
  runtime-landmine-to-toolchain-guarantee trajectory as the captures(none)
  class.

## Verification reuse

Every per-CPU/`KTHREAD` field the branded wrappers touch is declared with
`offset_of!` (the `debugblock.rs:110-148` inventory) and asserted in
`offset_check_*.rs`/`size_check_*.rs` (generated by `verify_offsets.py` /
`verify_sizes.py`), with `scan_magic_offsets.py` forbidding raw numeric offsets.
Routing `Pcpu<T>` access through typed fields means the ownership model inherits
the entire offset-drift defense for free — the two safety systems reinforce
rather than duplicate.

## Phasing

1. **Brand + `!Send` on the existing `IrqlGuard`/`SpinLockGuard`** — smallest
   step; migrate the clean lexical sites (`ex/timer.rs`, `po/events.rs`). Proves
   the token model on real code without touching the split sites. Stable-safe.
2. **`RunToken` typestates + the CPS `wait_for`/`wait_for_armed` API +
   `Pcpu<T>` (affinity-split, raw-field-access-only) + `remint_at_ffi_entry`**
   — turns "no sleep with spinlock," "no stale per-CPU ref," and "no wait from
   ISR" into compile errors on the migrated paths. Stable-safe. Budget the
   closure-migration churn per-subsystem (~109 + ~128 wait sites total).
3. ~~**`SwitchSafe` auto-trait + negative impls + the continuation bound**~~ —
   **REJECTED 2026-07-07** (perma-unstable foundation; see the Layer 2
   banner). Replaced by the stable `CTX_CAPTURE_ACROSS_WAIT` lint in
   `scan_irql_balance.py` (landed same day). Revisit only alongside MP.
4. **`irqlck`:** (a) cross-function `scan_irql_balance.py` (already on master,
   gated in `vibeos-cli check`) with adjust/expect summaries, keeping the 0-NEW
   gate; (b) the MIR port only if (a)'s misses prove material.

## Sources

Prior art the design is drawn from (external): Theseus OS `PreemptionGuard`
(`!Send` + `cpu_id()` drop-check) — OSDI '20; RTIC compile-time priority ceilings
(`lock()` raises to a computed ceiling) — rtic.rs; GhostCell branded
invariant-lifetime aliasing + the `generativity` crate — GhostCell (RustBelt)
paper; async `Send`-across-`.await` via coroutine saved-locals + auto-trait
leakage — rustc `rustc_mir_transform::coroutine`; Rust-for-Linux `klint`
preempt-count MIR analyzer (`adjust`/`expect`/`drop_preempt_count`) and
`Lock`/`Guard`/`ScopeGuard` — Kangrejos '23 / LWN 951550; `critical-section`'s
explicit reasoning about whether its proof token may cross a boundary; Tock
unforgeable zero-sized capability tokens.
