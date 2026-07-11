# Kernel-Mode Exception Handling — Trap-to-Panic Bridge Plan

> **ARCHIVED 2026-07-10** — deferred design (2026-05-09, "NO work started"); the trap-to-panic bridge stays deferred and is summarized as Tier-2 in the ruling `dummy-pseh-verdict.md`. Kept for the detailed bridge design. Outstanding items (if any) live in `docs/product/vibeos-roadmap.md`.


Last updated 2026-05-09. **Status: deferred, design specified.** No
work has started. This audit captures the design and execution plan
for hooking the kernel trap chain into Rust's panic/unwind semantics
across i386, amd64, and arm64 — integrating CPU-exception recovery
into ordinary Rust `Result`/`Drop` flow.

Companion to:
- [pseh-elimination.md](archive/pseh-elimination.md) — the validate-first
  strategy this project would *eventually* extend
- [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md) —
  validates that win32k can ship without this project landing first
- [archive/win32k-translation-audit.md](archive/win32k-translation-audit.md) — same
- `qemu/~temp-chat.md` — full pre-consolidation chat with code shapes
- `qemu/~temp-chat-consolidated.md` §8 — the consolidated 1-week plan

This doc is for the project itself. It defines what `try_user_access`,
the per-thread probe-region stack, the trap-frame rewriter, and the
catch-unwind plumbing actually look like across our three
architectures — and why we're not building it yet.

## TL;DR

- **Today: `panic = "abort"` everywhere.** Confirmed in
  `sdk/lib/rust/Cargo.toml` (both dev and release profiles).
  `panic_handler = loop { ... }`. No unwinder. No `catch_unwind`.
  Every kernel-mode page fault on a user pointer that wasn't
  pre-validated → bugcheck.
- **Today's only RIP-rewrite recovery** is `KiCheckForSListFault`
  (`sdk/lib/rust/ntoskrnl-rust/src/ke/i386/traphdlr.rs:703`) — a
  hard-coded RIP comparison for the SList interlocked-pop race. Not
  a generic mechanism.
- **The plan: integrate kernel-mode unwinding + a trap-to-panic
  bridge** so a CPU exception (page fault, etc.) inside a registered
  probe region becomes a Rust panic, runs `Drop`s, lands at a
  `catch_unwind` boundary at the syscall entry, and returns
  `Err(NTSTATUS)` to userland. **One-week project** if Rust-only,
  unit-test-driven, with static-analysis tooling. Multi-month
  otherwise — the calendar cost is in cross-arch porting and edge-case
  testing.
- **Five testable pieces, each independently shippable as a Rust
  crate that runs unit tests on the host:**
  1. Probe-region registration as a pure data structure (Day 1)
  2. Per-arch trap-frame rewriter as a pure function (Day 2)
  3. `unwinding` crate integration + `catch_unwind` proof-of-life (Day 3)
  4. Trap-to-panic bridge (`__probe_recovery_trampoline` + `panic_any`) (Day 4)
  5. Public `try_user_access<R>(kind, f) -> Result<R, NTSTATUS>` API (Day 5)
- **Day 6: kernel integration** (per-arch IDT page-fault handler patches).
  **Day 7: Python static-inspection tooling** (`.probe_records` section
  reader, `#[unsafe_reason]` audit, panic-graph check).
- **Why deferred:** validate-first probes are sufficient for win32k.
  Designing the unwinder *after* win32k translation exposes which
  probe patterns are actually common — measured reality vs assumptions.
  Per `pseh-elimination.md`, the project's current safety story is
  "validate first, never recover." This work doesn't break that
  story; it adds a second tier of safety (`recoverable kernel-mode
  fault`) on top.
- **What this is *not*:** user-mode exception delivery. That's a
  separate problem (DUMMY_PSEH in user-mode ntdll → user `__try`
  is dead code), with its own toolchain fix path. The kernel
  already delivers exceptions to user mode correctly; what's
  broken is the user-side dispatcher chain. **Decoupled.**
- **Critical insight from the consolidated audit:** in a *pure*
  Rust kernel this would be transformative (driver isolation
  without process boundaries, hot reload, composable error handling
  via `?` everywhere). In our hybrid C/Rust ReactOS-shaped kernel
  the C boundary is hostile to unwinding; we get a fraction of the
  upside. **Worthwhile, just less transformative than it would be
  on a clean-slate design.** Documented for context, not as a
  reason to skip.

## Current state of the trap chain

Per the audit summarized in `qemu/~temp-chat-consolidated.md` §4 and
verified against current code:

### What exists

| Thing | Location | Notes |
|---|---|---|
| Page-fault entry per arch | `ke/{i386,amd64,arm64}/{traphdlr,traphandler,trap}.rs` | All three arches handle `MmAccessFault` for resolvable faults; bugcheck otherwise |
| `KiCheckForSListFault` | `ke/i386/traphdlr.rs:703`, sister files for amd64/arm64 | The *only* RIP-rewrite recovery in the entire trap chain. Hard-coded EIP comparison against `KeUserPopEntrySListFault` and `ExpInterlockedPopEntrySListFault` |
| `MmAccessFault` resolution | `mm/mmfault.rs` (Rust); falls back to bugcheck | Demand paging, COW, transition PTE — all resolved here. Genuine page faults on valid-looking user pointers either get resolved or bugcheck |
| `KiDispatchException` | `ke/except.rs` | Already Rust. Walks user-side handlers when `PreviousMode == UserMode`. **Works** for delivering to user mode. Falls through to `KeBugCheckEx(KMODE_EXCEPTION_NOT_HANDLED)` for kernel-mode |
| Validate-first probe API | `ntkrnl-sys/src/probe.rs` | `probe_read_user`, `probe_write_user`, `probe_user_buffer`, etc. Returns `Result<_, NTSTATUS>`. Bounds + alignment + `read_volatile` upfront |
| RAII guards | `ntkrnl-sys/src/guards.rs` | `IrqlGuard`, `SpinLockGuard`, `SpinLockAtDpcGuard`. Plus `PfnLockGuard`, `ObjRefGuard`, `PoolAllocGuard` in subsystem files |

### What does NOT exist

| Thing | Notes |
|---|---|
| `panic = "unwind"` anywhere | Workspace-wide `panic = "abort"` in `sdk/lib/rust/Cargo.toml` (both profiles) |
| `unwinding` crate or libpanic_unwind | Not vendored, not a dependency |
| `catch_unwind` | Not callable from kernel-mode `no_std` build |
| Probe-region registration mechanism | None — `probe.rs` validates upfront, doesn't register a fault-recovery handler |
| Per-thread record stack analogous to `EXCEPTION_REGISTRATION_RECORD` | None |
| Generic trap-frame rewriter | None — `KiCheckForSListFault` is the lone hard-coded case |
| `__probe_recovery_trampoline` | Doesn't exist |
| `try_user_access<R>` API | Doesn't exist |

### Why "validate-first" works without this

The `probe_*` family in `probe.rs` does:

1. Bounds check (within user address space, no overflow)
2. Alignment check
3. `read_volatile` / `write_volatile` actual access

Step 3 *can* still fault — and when it does today, the trap handler
falls through to bugcheck. But practically, "valid-looking" user
pointers that fault here are rare:

- Demand paging / COW / transition PTEs are caught earlier by
  `MmAccessFault` (step 3 doesn't reach the bugcheck path).
- Race conditions where user-mode unmaps memory between probe and
  use — possible but requires timing pressure.
- Genuinely malformed pointers that pass bounds+alignment — also rare.

So the "kernel that bugchecks on bad user pointers" property is real
but doesn't fire often. The `try_user_access` project closes that
gap.

## What we're building

Public API (the user-facing surface that win32k and other subsystems
will call):

```rust
pub fn try_user_access<R>(
    kind: ProbeKind,
    f: impl FnOnce() -> R,
) -> Result<R, NTSTATUS>;

pub enum ProbeKind {
    Read,
    Write,
    CaptureRead,    // probe + copy in
    CaptureWrite,   // probe + copy out
    Execute,
}

// Existing probe.rs functions get an unwinding-aware variant:
pub unsafe fn try_probe_read_user<T: Copy>(ptr: *const T) -> Result<T, NTSTATUS>;
// etc.
```

Semantics:
- Push a `ProbeRecord` onto the per-thread probe stack.
- Run `f()`.
- If `f` runs to completion, pop record, return `Ok`.
- If `f` causes a CPU exception inside a kernel-mode access to a
  user address, the trap handler rewrites the trap frame to redirect
  to `__probe_recovery_trampoline` with `STATUS_ACCESS_VIOLATION` in
  the first-arg register.
- The trampoline calls `panic_any(ProbeFault { status })`.
- The unwinder runs frame-by-frame, executing `Drop` impls in reverse
  acquisition order (lock guards, refcount guards, etc. — including
  `IrqlGuard` which lowers IRQL on the way out).
- `catch_unwind` inside `try_user_access` catches the panic, downcasts
  to `ProbeFault`, returns `Err(STATUS_ACCESS_VIOLATION)`.
- The `?` operator at the call site propagates the error normally.

## The 7-day execution plan

### Day 1 — Probe-region registration as a pure data structure

New crate: `sdk/lib/rust/probe-region/`. No kernel deps.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ProbeRecord {
    pub magic: u32,                    // = PROBE_RECORD_MAGIC, easy to find in binary
    pub version: u32,                  // bump when struct changes
    pub recovery_pc: usize,
    pub recovery_sp: usize,
    pub source_file: &'static str,     // captured by macro
    pub source_line: u32,
    pub probe_kind: ProbeKind,
    pub function_name: &'static str,   // for static inspection
    // guarded_range: Range<usize> — DEFERRED, see day 5 design note
}

pub trait ProbeStack {
    fn push(&mut self, record: ProbeRecord);
    fn pop(&mut self);
    fn topmost(&self) -> Option<&ProbeRecord>;
}

// Test impl (host-only): Vec-backed.
// Production impl: KPCR-rooted intrusive list (Day 6).
```

Unit tests: nested registration, push/pop ordering, topmost-on-empty,
topmost-on-stack-of-N, find-by-RIP-range (kept for future though
unused by current rule). **~200 LOC inc. tests.**

The `register_probe!` macro captures `file!()`, `line!()`,
`probe_kind`, `function_name` at the call site. The
`#[link_section = ".probe_records"]` attribute is what the Day 7
Python tool walks.

### Day 2 — Per-arch trap-frame rewriter as a pure function

New crate: `sdk/lib/rust/probe-trap-rewrite/`. Depends on
`ntkrnl-sys` for `KTRAP_FRAME` / `KEXCEPTION_FRAME` definitions.

```rust
pub fn rewrite_for_recovery(
    frame: &mut KTRAP_FRAME,
    record: &ProbeRecord,
    status: NTSTATUS,
) {
    #[cfg(target_arch = "x86_64")]
    {
        frame.Rip = record.recovery_pc as u64;
        frame.Rsp = record.recovery_sp as u64;
        // System V / Microsoft x64 calling conv: first arg in RCX/RDI
        // KTRAP_FRAME ABI on Windows: RCX is first arg
        frame.Rcx = status as u64;
    }
    #[cfg(target_arch = "x86")]
    {
        frame.Eip = record.recovery_pc as u32;
        frame.HardwareEsp = record.recovery_sp as u32;
        // i386 fastcall: first arg in ECX
        frame.Ecx = status as u32;
    }
    #[cfg(target_arch = "aarch64")]
    {
        frame.Pc = record.recovery_pc as u64;
        frame.Sp = record.recovery_sp as u64;
        // AAPCS64: first arg in X0
        frame.X0 = status as u64;
    }
}
```

**Critical:** the calling convention for `__probe_recovery_trampoline`
must match what `rewrite_for_recovery` sets up. We use the
platform-default first-arg register so the trampoline is regular Rust
with no inline asm.

Unit tests: construct a fake `KTRAP_FRAME` per arch, call
`rewrite_for_recovery`, assert the mutated frame has the expected
field values. Run on the host. Also runs under `cargo miri` for free.

**Compile-time `KTRAP_FRAME` layout assertions:**

```rust
#[cfg(target_arch = "x86_64")]
const _: () = assert!(core::mem::offset_of!(KTRAP_FRAME, Rip) == 0xF8);
const _: () = assert!(core::mem::offset_of!(KTRAP_FRAME, Rsp) == 0x110);
// ... etc
```

These exist as a cross-language invariant: if bindgen ever produces
a different layout than what the asm trampolines (`KiTrap0E*`,
`KiPageFault`, etc.) expect, build fails. Python static inspection
(Day 7) extends this with per-binary verification.

**~150 LOC across 3 arches inc. tests.**

### Day 3 — `unwinding` crate integration + `catch_unwind` proof-of-life

This is the riskiest day for cost overrun. Tasks:

1. Vendor or git-submodule the [`unwinding`](https://github.com/nbdd0121/unwinding)
   crate into `sdk/lib/3rdparty/unwinding/` (alongside FreeType etc.).
2. Get it building `no_std` for `i686-pc-windows-gnu`,
   `x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`. Verify
   `.eh_frame` is emitted and preserved by lld.
3. Add `panic = "unwind"` to a *new* test crate (not the workspace
   root yet — opt-in per crate so we don't disturb the kernel build).
4. Wire `rust_eh_personality` and `catch_unwind` into the test crate's
   no_std environment.
5. Build a kernel test driver (loaded into ntoskrnl via the existing
   `tests/qemu/test_kernel.py` harness) that deliberately panics
   inside `catch_unwind` and asserts Drops ran in reverse order.

```rust
// Test driver:
extern "C" fn DriverEntry(...) -> NTSTATUS {
    let log = TestLogBuffer::new();
    let result = unwinding::panic::catch_unwind(AssertUnwindSafe(|| {
        let _outer = TestDropGuard::new(&log, "outer");
        let _inner = TestDropGuard::new(&log, "inner");
        panic!("test panic from kernel");
    }));
    match result {
        Err(_) => assert_eq!(log.entries(), &["drop:inner", "drop:outer"]),
        Ok(_) => panic!("catch_unwind returned Ok??"),
    }
    STATUS_SUCCESS
}
```

**The thing to discover:** does `unwinding` work out-of-the-box on
llvm-mingw PE/COFF for our three arches? Specifically:

- **i386 PE/COFF DWARF:** llvm-mingw emits `.eh_frame`; `unwinding`
  expects it. Should work.
- **amd64 PE/COFF DWARF:** same. Should work.
- **arm64 PE/COFF SEH:** llvm-mingw on arm64-PE emits `.pdata`/`.xdata`
  (SEH unwind), not DWARF. `unwinding` may not speak this. **Risk
  surface.** If `unwinding` can't, we either:
  - (A) Translate SEH unwind data to DWARF at link time (toolchain
    work, ~3-5 days)
  - (B) Write a minimal SEH unwind interpreter in `unwinding` fork
    (~3-5 days, ~500-1000 LOC)
  - (C) Defer arm64 entirely to phase 2; ship i386+amd64 first.

**Plan a full day for Day 3.** If the day blows past, the **fallback
is the panic=abort variant** (Day 4 alternative below) — preserves
the safety property without the unwinder.

### Day 4 — Trap-to-panic bridge

Goal: connect the trap-handler RIP rewrite (Day 2) to the unwinder
(Day 3) via a recovery trampoline.

```rust
#[no_mangle]
pub extern "C" fn __probe_recovery_trampoline(status: NTSTATUS) -> ! {
    panic_any(ProbeFault { status });
}

#[derive(Debug)]
pub struct ProbeFault {
    pub status: NTSTATUS,
}
```

The trampoline is a normal function. Day 2's rewriter sets up the
calling convention (status in RCX/ECX/X0) so the trampoline picks it
up as a regular argument. **No inline asm.** This is the
non-obvious win of Day 2's per-arch register-by-ABI choice.

**End-to-end test without taking a real fault:**

```rust
#[test]
fn synthetic_probe_fault_caught() {
    let result = catch_unwind(|| {
        __probe_recovery_trampoline(STATUS_ACCESS_VIOLATION);
    });
    let payload = result.unwrap_err();
    let fault = payload.downcast::<ProbeFault>().unwrap();
    assert_eq!(fault.status, STATUS_ACCESS_VIOLATION);
}
```

Validates the entire trampoline → panic → unwind → catch path
without the trap handler being involved.

**Day-4 fallback if Day 3 didn't deliver `unwinding`:** keep
`panic = "abort"`, replace `panic_any` with a custom panic_handler
that does longjmp-style direct recovery to the `try_user_access`
caller. No Drop on panic path, but `try_user_access` still returns
`Err`. ~50 LOC `setjmp.S` / `longjmp.S` per arch. Drop-on-panic is
nice-to-have, not load-bearing for safety.

### Day 5 — Public `try_user_access` API

```rust
pub fn try_user_access<R>(
    kind: ProbeKind,
    f: impl FnOnce() -> R,
) -> Result<R, NTSTATUS> {
    let record = ProbeRecord {
        magic: PROBE_RECORD_MAGIC,
        version: 1,
        recovery_pc: __probe_recovery_trampoline as usize,
        recovery_sp: current_sp(),
        source_file: file!(),
        source_line: line!(),
        probe_kind: kind,
        function_name: caller_function_name(),
    };
    push_probe_record(&record);
    let result = catch_unwind(AssertUnwindSafe(f));
    pop_probe_record();
    match result {
        Ok(value) => Ok(value),
        Err(payload) => match payload.downcast_ref::<ProbeFault>() {
            Some(fault) => Err(fault.status),
            None => resume_unwind(payload),  // not our panic — re-raise
        }
    }
}
```

**Key design decision: skip RIP-range matching, use topmost-record
rule.** The trap handler walks the per-thread probe stack; the
**topmost record** is the recovery target for any kernel-mode fault
on a user address. This matches NT's `EXCEPTION_REGISTRATION_RECORD`
chain semantics (handlers are scoped by stack position, not by RIP
range). Simpler. Faster. Document the contract: **don't call
non-probing code from inside a `try_user_access` closure unless that
code is safe to be aborted at any RIP.**

The `guarded_range` field stays in the struct (reserved for future)
but is unset/unused under this rule.

Tests on the host:
- `try_user_access` with closure that returns Ok → returns Ok
- `try_user_access` with closure that calls `__probe_recovery_trampoline`
  directly → returns Err(STATUS_ACCESS_VIOLATION)
- `try_user_access` with closure that panics with non-`ProbeFault`
  payload → `resume_unwind` propagates (not silently caught)
- Nested `try_user_access` calls — outer catches outer panics,
  inner catches inner panics

### Day 6 — Kernel integration + per-arch trap handler patch

Now the day where Rust touches the actual kernel page-fault entry.
For each arch, modify the `KiTrap0E` / `KiPageFault` / `KiPageFaultException`
handler in `ke/{i386,amd64,arm64}/{traphdlr,traphandler,trap}.rs` to:

```rust
unsafe fn handle_kernel_user_address_fault(frame: &mut KTRAP_FRAME, addr: PVOID) -> bool {
    if PreviousMode != UserMode &&
       (addr as usize) < MmHighestUserAddress as usize {
        if let Some(record) = current_thread_probe_stack().topmost() {
            rewrite_for_recovery(frame, record, STATUS_ACCESS_VIOLATION);
            return true;  // IRETQ to recovery trampoline
        }
    }
    false  // existing bugcheck path
}
```

Insertion point in the existing handlers (per arch):
- **i386** (`ke/i386/traphdlr.rs:KiTrap0EHandler`): after
  `KiCheckForSListFault` returns false, before the `MmAccessFault`
  call. Or *after* `MmAccessFault` returns failure but before
  bugcheck.
- **amd64** (`ke/amd64/traphandler.rs`): equivalent location after
  `MmAccessFault` failure.
- **arm64** (`ke/arm64/trap.rs`): after the existing
  `MmAccessFault` block — currently bugchecks on failure;
  we slot recovery in between.

**Recommendation: post-`MmAccessFault`-failure** insertion point.
That way demand paging / COW / transition PTEs are still resolved
normally; only genuinely-failing accesses go through the recovery
path. Matches NT's classic chain semantics.

Test by writing a kernel test (loaded via `tests/qemu/test_kernel.py`):

```rust
#[test_kernel] fn test_try_user_access_recovers_from_unmapped() {
    // 0x12345678 is unmapped in user space
    let result = try_user_access(ProbeKind::Read, || unsafe {
        *(0x12345678 as *const u32)
    });
    assert_eq!(result, Err(STATUS_ACCESS_VIOLATION));
    // Kernel still alive — we got here.
}
```

Run across all 3 arches. **Pass = success.**

### Day 7 — Static inspection tooling + cross-language invariants

Python tool: `tools/probe_records_inspect.py`. Reads the kernel binary,
walks `.probe_records` section, validates:

| Check | What |
|---|---|
| **Magic + version** | Every record starts with `PROBE_RECORD_MAGIC`; struct version matches Rust definition |
| **Recovery PC** | Resolves to `__probe_recovery_trampoline` exactly. Catches anyone trying to do clever custom recovery (which would bypass `catch_unwind`). |
| **Source file allowlist** | Every record's `source_file` is in `(probe-region/, probe-trap-rewrite/, probe.rs, ntkrnl-sys/, allowlisted callers)` |
| **Function-name marker** | Every function containing a probe record has `#[probe_function]` (no-op attribute marker via a side `.probe_functions` section) |
| **Cross-ref** | Every static `.probe_records` entry corresponds to a `try_user_access` call site at the same `(file, line)` |

Plus broader annotations the audit recommends:

```rust
#[unsafe_reason("dereferences raw user pointer; caller validated alignment")]
unsafe fn read_user_word(ptr: *const u32) -> u32 { ... }

#[may_panic("calls unwrap on Option in cold path")]
fn rare_path() -> Foo { ... }

#[no_panic]
fn leaf() -> Foo { ... }  // Python verifies no transitive may_panic call
```

These are no-op compile-time attributes that emit marker symbols.
Python tool consumes them. **CI build gate**: any unattributed
`unsafe fn` in the probe machinery, any `#[no_panic]` transitively
calling `#[may_panic]`, fails the build.

**Integration test suite:** kernel test driver exercises every
`ProbeKind` × valid/invalid pointer combination, asserts
`try_user_access` returns the expected `Result`. Run nightly under
all 3 arches.

**~200 LOC Python**, single file in `tools/`.

## Out of scope for week 1

| Item | Why deferred |
|---|---|
| arm64 SEH unwind data fights | Possible Day 3 risk. If `unwinding` doesn't speak SEH, ship i386+amd64 first; arm64 follows in week 2 with whatever toolchain workaround is needed. |
| C-side unwind support | Pure-Rust callers/callees only. C code must not fault inside its calls; if it does, bugcheck stays. Most kernel C is gone (~99.7% Rust); the remaining shims don't fault. |
| `.eh_frame` registration for dynamically loaded drivers | Static kernel image only for week 1. Drivers come later; needs `__register_frame`-equivalent at driver load. |
| Performance optimization | Probe push/pop is ~20 instructions; the unwinder is the slow path that only runs on actual faults. Don't optimize until measured. |
| User-mode exception delivery | Different problem; see [pseh-elimination.md](archive/pseh-elimination.md) and the consolidated audit §5/§9. The kernel already delivers; the user-side dispatcher is DUMMY_PSEH'd. **Toolchain fix, not this project.** |

## Risks and fallbacks

### Risk 1: `unwinding` crate doesn't work on llvm-mingw PE/COFF

Most likely on arm64 (SEH vs DWARF). Mitigation:
- Day 0 (today, before committing the schedule): read `unwinding`
  source briefly to see how it discovers `.eh_frame`. If it expects
  ELF section discovery via `dl_iterate_phdr`-equivalents, plan to
  write a kernel-side registration shim that feeds it `.eh_frame`
  bounds.
- If Day 3 takes >1 day: switch to **Day-4 fallback** (panic=abort
  + custom longjmp recovery). Loses Drop-on-panic but preserves
  safety property and `try_user_access` semantics.
- If arm64 is irreducibly stuck: ship i386 + amd64 in week 1,
  arm64 in week 2 or as a separate effort.

### Risk 2: Drop-during-unwind requires panic-during-unwind handling

Rust treats panic-during-unwind as abort. If a Drop impl can panic,
we lose the Drop chain. **Audit guard types' Drop impls:** none
should be fallible.

- `IrqlGuard::Drop` → `KeLowerIrql` — infallible (no fault path)
- `SpinLockGuard::Drop` → `KeReleaseSpinLock` — infallible
- `PfnLockGuard::Drop` → PFN database lock release — infallible
- `ObjRefGuard::Drop` → `ObDereferenceObject` (ref count decrement) —
  should be infallible; verify
- `PoolAllocGuard::Drop` → `ExFreePool` — infallible

**Audit task: 1 hour, before Day 1.** Add `#[no_panic]` annotation
to each Drop impl; CI gate prevents regression.

### Risk 3: Kernel-binary size growth from `.eh_frame`

Unwind tables typically add 5–15% to binary size. Acceptable but
worth measuring. **Day 7 task:** report binary size before/after
across 3 arches; add to the smoke-test report.

### Risk 4: Trap handler's `topmost-record` rule too permissive

The contract says "don't call non-probing code from inside the
closure unless safe to be aborted at any RIP." Hard to enforce
statically without RIP-range matching. **Mitigation:** Day 7 Python
tool adds a check that `try_user_access` closure bodies only call
functions in an allowlist of "abort-safe" operations (essentially,
functions with no Drop side effects beyond their own guards).
**Best-effort static check; doesn't catch runtime composition.**

## Sequencing — when does this work get done?

The pre-consolidated audit and the `pseh-elimination.md` strategy
make a clear ordering case:

1. **Now → 4–6 months:** validate-first probes for win32k.
   Translation phase. Doesn't need this project.
2. **Parallel track (separate person/calendar):** user-mode
   exception delivery fix (DUMMY_PSEH in user-mode ntdll).
   Independent project.
3. **After win32k stable:** this project. Designed against measured
   reality — the patterns that win32k actually exercises will inform
   which `ProbeKind`s matter, what the typical nesting depth looks
   like, where the hot paths are.

**Critical reframe (per consolidated audit §9):** "exceptions back
into user-mode threads have to work" is mostly a *user-mode*
unwinder problem, not a kernel one. The kernel just delivers the
`EXCEPTION_RECORD`+`CONTEXT`; that path mostly already exists.
**Decoupled.**

If the win32k translation surfaces a structurally-fault-dependent
pattern that validate-first can't handle (rare; the
`MmMapLockedPagesSpecifyCache` audit in
[win32k-pretranslation-audit.md](win32k-pretranslation-audit.md)
expects 0–3 sites), this project gets pulled forward. Otherwise,
defer.

## Why this matters more in a pure Rust kernel

Documented for context (the user asked the question explicitly).

In our hybrid C/Rust ReactOS kernel, the C boundary is hostile to
unwinding: C frames may or may not have unwind info, may have called
into Rust which called back into C, etc. The pragmatic move was
"don't unwind, validate first, bugcheck on fault" — and it's the
right call given heterogeneity.

In a pure Rust kernel:
- Every function has unwind tables (or `#[no_unwind]`).
- Every Drop is a Rust Drop.
- Every panic propagates through code the compiler understands.
- Cost-benefit shifts hard toward enabling the unwinder.

What pure-Rust + working unwinder unlocks:
- **Driver isolation without process boundaries.** A driver panics,
  `catch_unwind` at the driver entry point catches it, the driver is
  marked dead, the kernel keeps running. Theseus does this.
- **Composable error handling via `?` everywhere**, without
  validate-first contortion. Hardware register access on a hot-unplugged
  device: `try_mmio_read(...)?`. Disk read on a failed disk: same shape.
- **Genuine recovery from "shouldn't happen" bugs.** `.unwrap()`
  panics become recoverable at the syscall boundary.
- **RAII-shaped IRQL/lock discipline that's actually reliable.**
  `IrqlGuard::Drop` runs during unwinding; the type system's promise
  becomes a runtime guarantee, not "if you don't panic, this works."
- **Driver hot-reload.**

ReactOS-Rust gets a fraction of these because we're pinned to NT
data structures and conventions, but the floor is still
"kernel-mode user-pointer faults are recoverable instead of
fatal" — already a meaningful safety upgrade.

## Definition of done

- `try_user_access<R>(kind, f) -> Result<R, NTSTATUS>` is callable
  from any kernel-mode Rust crate.
- All three arches handle `try_user_access` with an unmapped user
  address by returning `Err(STATUS_ACCESS_VIOLATION)` and **not
  bugchecking**.
- All Drop impls run during unwinding (verified by integration test
  exercising a probe fault inside a `IrqlGuard` scope).
- `tools/probe_records_inspect.py` validates the probe-record set
  in CI; build fails on violations.
- `unwinding` crate vendored, builds for all 3 arches.
- Existing `probe_*` API in `ntkrnl-sys/src/probe.rs` gains
  `try_*` variants that use `try_user_access` internally.
- All existing smoke tests still pass.
- New kernel test suite (`tests/qemu/test_probe.py` or similar)
  exercises every `ProbeKind` × valid/invalid pointer combo.

## Decision tree for "do we pull this forward?"

1. **Did win32k translation surface a structurally-fault-dependent
   pattern that can't be validated?** If yes, pull forward to
   unblock. If no, continue.
2. **Did a security audit/CVE reveal that "malicious user pointer
   bugchecks the system" is being exploited?** If yes, pull
   forward. If no, continue.
3. **Is there a clean-slate Rust kernel project starting that
   could share infrastructure?** If yes, build this jointly. If
   no, continue.
4. **Otherwise:** schedule for after win32k translation is stable,
   ~6+ months out, and let it inform driver-isolation work that
   comes after.

## Open questions

1. **`unwinding` crate vs vendored fork?** Upstream is active but
   small. **Recommendation: git submodule from upstream**, with a
   pinned commit; we can fork if needed.
2. **`panic = "unwind"` workspace-wide or per-crate?** Per-crate is
   safer (rollout incremental). **Recommendation: per-crate, opt-in
   via a `kernel_panic_unwind` workspace feature flag.**
3. **`probe-region` crate location.** `sdk/lib/rust/probe-region/`
   is consistent with the workspace shape. Could fold into
   `ntkrnl-sys/src/probe.rs` if simpler. **Recommendation: standalone
   crate** — different consumer set (probe-region used by many,
   probe.rs used only by Nt* syscall implementations).
4. **Static-analysis tool language.** Python is the project default.
   **Recommendation: Python**, in `tools/`, sibling to existing
   `verify_abi.py`/`verify_constants.py`/`scan_ud2.py`.
5. **Fallback path commitment.** Day-4-fallback (panic=abort + custom
   longjmp) is the safety-property-only escape hatch. **Recommendation:
   build it deliberately as a feature flag `kernel_unwind_fallback`,
   not as plan-B-when-things-break.** Even with `unwinding` working,
   some configurations may want abort semantics for smaller binary
   size or determinism guarantees.
6. **Per-thread probe stack location.** KPCR vs `ETHREAD->KernelStack`
   chain (i386-Windows-style) vs `KTHREAD` field. **Recommendation:
   KPCR-rooted intrusive list**, matches stack-discipline assumptions
   and composes with nesting. Day 6 task to wire correctly.
7. **`ProbeKind::Execute`.** Do we actually need this? Kernel-mode
   code rarely executes user-mode code directly (outside of the
   `KeUserModeCallback` path which is its own thing). **Recommendation:
   omit from Day 1; add later if a use case emerges.**

## What this audit doesn't cover

- **The actual implementation.** This is a project plan, not a spec.
  Code shapes are illustrative.
- **User-mode exception delivery.** Separate project; see
  [pseh-elimination.md](archive/pseh-elimination.md).
- **Driver hot-reload / driver isolation.** Future work this project
  enables; not in scope here.
- **C-side unwinding.** Out of scope; we have ~99.7% Rust and the
  remaining C is shims that don't fault.
- **Performance.** Measure when the implementation exists; don't
  optimize speculatively.
