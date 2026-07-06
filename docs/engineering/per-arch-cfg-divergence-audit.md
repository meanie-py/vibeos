# Per-arch `#[cfg]` divergence audit (2026-07-05)

A workspace-wide audit of `#[cfg(target_arch/target_os)]` sites, hunting cfg that
gates **actual logic divergence** (which hides bugs) vs legitimate arch differences
(ABI, asm, layout). Motivated by the notepad/taskbar arch-gating smell and the two
`0x4E` cfg-drift crashes already in the field manual (§9.15/§9.16).

**Headline: the §9.15 "4-level-paging block gated to one 4-level arch" drift is
systemic** — the same latent-bug pattern recurs in ~8 more Mm functions, in both
directions. That is the audit's main payoff and Tier 1 below.

## Method (reproducible)
`tools/scan_arch_cfg.py` classifies every cfg site by the item it attaches to and
keeps only the **logic-gating candidates**, stripping legit ABI/asm/layout/module
noise: **8,449 raw cfg sites → 2,453 candidates** (win32k alone: 4,070 → 194). A
6-way subagent fleet then reviewed all 2,453 against this rubric, cross-checking C
headers/upstream:
- **DRIFT-BUG** — cfg includes/excludes arches inconsistently (a fix/block/constant
  on one arch its sibling also needs). The §9.15 pattern.
- **BRINGUP-SHORTCUT** — a feature/behavior gated to one arch as dev scaffolding.
- **UNNECESSARY-DIVERGENCE** — same algorithm forked per arch with no HW/ABI reason.
- **ARCH-STUB** — one arch has a no-op/TODO where others have a real impl.
- **LEGIT (not reported)** — calling-convention, size/offset/layout, arch asm/registers,
  genuinely-different hardware, correct paging-level (i386 2-level really lacks PXE/PPE).

Re-run: `python3 tools/scan_arch_cfg.py -o /tmp/cand.txt`.

---

## TIER 1 — latent bugs (fix)

### The PXE 4th-level drift cluster (Mm/ARM3)
amd64 and arm64 are **both** `_MI_PAGING_LEVELS == 4`, so any top-level PXE
walk/validate/fault-in must be `#[cfg(any(target_arch="x86_64", target_arch="aarch64"))]`
— never gated to one of them. These siblings of the §9.15 fix (`procsup.rs:701` already
has the correct rule) were never brought into line. Each only survives because the
offending top-level entry is almost always resident for the VAs the path touches.

| Site | Function | Gap |
|---|---|---|
| `ARM3/virtual.rs:609` | `MiDeleteVirtualAddresses` | PXE x86_64-only → **arm64 teardown** derefs PPE without validating PXE (§9.16 family) |
| `ARM3/virtual.rs:3369` | `MiLockVirtualMemory` | PXE **aarch64-only → amd64** skips PXE validation |
| `ARM3/virtual.rs:3618` | `MiUnlockVirtualMemory` | PXE aarch64-only → amd64 skips PXE validation |
| `ARM3/mdlsup.rs:693` | `MmProbeAndLockPages` | PXE x86_64-only → arm64 never faults PXE |
| `ARM3/virtual.rs:239` | `MiCalculatePageCommitment` | PXE x86_64-only → arm64 loses top-level skip/make-valid |
| `ARM3/virtual.rs:993` | `MiQueryAddressState` | same |
| `ARM3/virtual.rs:1305` | `MiIsEntireRangeCommitted` | same |
| `ARM3/virtual.rs:1400` | `MiMakePdeExistAndMakeValid` | arm64 arm checks only PPE+PDE; also a tautological `all(aarch64, not(x86_64))` cfg smell |

**Fix:** gate each PXE block `any(x86_64, aarch64)`. Core Mm across two arches → **gate ×3**.

### Other Tier-1
- **`ps/win32.rs:20` — wrong constant.** i386 `KERNEL_STACK_SIZE = 0x6000`; the header
  (`xdk/x86/ke.h:218`, `ndk/i386/asm.h:665`) says **`0x3000`**. Used in the GUI-thread
  `KeSwitchKernelStack`, so the switched `StackLimit` is computed 0x3000 too low on i386.
  Fix: `0x6000` → `0x3000`.
- **`ke/wait.rs:411` — `KeDelayExecutionThread` is a no-op stub on arm64** (just
  `NtYieldExecution()` + `STATUS_SUCCESS`). A real-interval sleep returns immediately →
  busy-spin / broken timing on arm64. Fix: share the `not(aarch64)` timer-block body (or
  mark as a tracked TODO if timers genuinely aren't ready).

## TIER 2 — divergence-from-C, verify + unify
- **✅ DONE `ex/init.rs` — amd64 alone skipped the SMSS wait + `SESSION5_INITIALIZATION_FAILED`
  (0x6A) bugcheck** (Rust-only carve-out `#[cfg(not(x86_64))]`; the C does it unconditionally,
  `ntoskrnl/ex/init.c:2350`). Verified: a March bring-up shortcut (5802b15) masking an
  early-smss-death rooted in the pre-fix Mm/pushlock wedges (§9.13-9.17). Restored the guard
  on all arches to match C (`8738b221955`) — gate ×3 green, amd64 smss confirmed healthy
  (STATUS_TIMEOUT, no bugcheck), so no latent death was hiding behind the shortcut.
- **✅ DONE `mm/mmfault.rs` — WS-lock recursion guard was arm64-only**, with a *false*
  "amd64/i386 KVM schedules around it" comment. Verified now-redundant: it band-aided the
  §9.13-class fault-inside-teardown self-deadlock, and amd64 (which never had the guard) has
  booted green ×3 since §9.13's PFN-union root fix. Removed it — all arches now take the WS
  lock unconditionally, matching C + the amd64/i386 path (`4be55078de3`); also deletes a
  per-arch `__bindgen_anon_9` layout-drift hazard. Gate ×3 green (arm64 under test, no
  self-deadlock).
- **➡ ROUTED `win32k/main.rs:673` — `co_IntClientThreadSetup()` call gated i386-only** →
  skipped on amd64+arm64 (fully implemented on all arches; only the call site is gated).
  Verified = deliberate bring-up shortcut skipping the per-GUI-thread CLIENTTHREADSTARTUP
  user-callback on **exactly the two render-gap arches**. Re-enabling is a real behavioral
  change (extra kernel→user callback per GUI thread) *and* a plausible cause of the
  non-client-paint gap → routed to the **screen-painting fable** as a diagnostic lead
  (`ui-render-completeness-investigation.md`) rather than unified blind.

## TIER 3 — arm64 bring-up shortcuts (retire now that arm64 boots to desktop)
A coherent ARM64-native-boot cluster in win32k/userland. Some Rust-introduced, some
faithfully mirror the C's `#ifdef _M_ARM64`; all retire-able:
- `winlogon-rust/winlogon.rs`: spawns notepad/cmd (835-918); **skips RPC/services/lsass/
  notifications/GINA** (585-664); hand-creates `\Windows\WindowStations` (553-572).
- `winlogon-rust/wlx.rs:568` — skips all security-descriptor creation + screen-saver desktop.
- `kernel32-rust/dllmain.rs:199` + `proc_create.rs:2212` — tolerate missing csrss; skip the
  CSRSS `BasepCreateProcess` notification.
- `smss-csrss-rust/csrsrv/{init.rs:1115, api.rs:185}` — forked SM connect / skipped
  `CsrConnectToUser` retry.
- `win32k-rust/desktop.rs:3657/3734` — `IntHideDesktop` gated `not(aarch64)` (arm64 leaves
  old desktop shown on switch); `init.rs:114` — dxg init failure non-fatal on arm64.
- `ntoskrnl-rust/ex/init.rs:{1325,1347,1359,1372}` — Phase-1 init-failure bugchecks skipped
  on arm64 (faithful to C `#ifdef _M_ARM64`).

## TIER 4 — low / diagnostic / cosmetic
- `fastfat-rust/strucsup.rs:19` — arm64 `RtlFillMemoryUlong` drops its fill pattern
  (zero-fills); latent (poison-fill caller compiled out today) but would blind UAF
  detection on arm64 if enabled.
- `fastfat-rust/fatinit.rs:74` — `__int2c` → `unreachable_unchecked()` (UB) on non-x86.
- `freeldr-rust/winldr.rs:1313` — `WinLdrpDumpArcDisks` excluded on amd64 only (TRACE-only).
- `rtl-diag/exception.rs:132` — no arm64 register dump (diagnostic-only).
- `ntoskrnl-rust/ke/ipi.rs:129` — amd64 IPI_SYNCH_REQUEST handler is a `DbgBreakPoint` stub.
- `ntkrnl-sys/mm.rs:344` — identical-body cfg + misleading comment (value correct; bit 8 all arches).
- `ntoskrnl-rust/ke/{wait.rs:351, thrdschd.rs:474/826}` — arm64-only ReadySummary preemption
  recheck the x86 arches lack (drift-watch — confirm vs stock ReactOS).

---

## Status / work queue
- **Tier 1** — ✅ PXE cluster + KERNEL_STACK_SIZE landed (`475c10677e9`), gate ×3 green.
  Remaining: `KeDelayExecutionThread` arm64 stub (ke/wait.rs:411) — **routed to the
  arm64-interrupts fable** (it depends on the timer/interrupt machinery that lane is
  actively rebuilding; flipping arm64 onto the real timer-block wait solo would build on
  not-yet-ready arm64 timer-wake and collide with their changes).
- **Tier 2** — ✅ both actionable items landed (ex/init SMSS guard `8738b221955`, mmfault
  guard removal `4be55078de3`); win32k `co_IntClientThreadSetup` routed to the
  screen-painting fable. Tier 2 complete.
- **Tier 3** — track under "retire arm64 native-boot scaffolding"; some blocked on the
  arm64-interrupts + service-subsystem bring-up.
- **Tier 4** — opportunistic.

Everything the fleet cleared as LEGIT (register/CPUID/paging-level/ABI/layout, ~2.4K−~40
sites) is *correct* per-arch divergence — this audit is not an argument against `#[cfg]`,
only against cfg that gates logic which shouldn't diverge.
