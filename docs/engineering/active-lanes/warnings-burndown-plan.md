# Rust warnings burndown (lane: `warnings-burndown`)

**Status:** OPEN (opened 2026-07-16). Worktree `vibeos-warnings-burndown`, branch
`warnings-burndown`. Follow-on from the closed `rust-warnings` lane (which fixed the 3
dangling-pointer UAFs + landed `scan_dangling_transmute`); this lane burns down the *rest*.

**Deliverable:** the Rust workspace compiles with a dramatically smaller warning surface,
the genuine-signal warnings are audited and fixed, and — critically — the fixed classes
**stay** fixed (no blanket re-suppression). Incremental, per-crate, each cluster gated ×3
and landed on its own so conflicts stay localized and every step is revertible.

## The census finding that reframes everything (2026-07-16)

**201 of 219 workspace crates (92%) carry a crate-root `#![allow(...)]`** — most of them
blanket `dead_code`/`non_snake_case`/`unused` suppressions parked at `lib.rs` top. So:

- The "~9,555 warnings" figure from the rust-warnings survey was measured **with those
  blanket allows in place** — it is an *undercount*. The real surface is hidden.
- A burndown that only fixes what currently *leaks past* the allows is cosmetic. The
  durable version **peels the blanket allow off a crate**, fixes what that exposes, and
  lands the crate clean **without** re-adding the blanket — so future warnings can't hide.
- These allows exist because the c2rust/LLM translation output is noisy by construction
  (non-snake-case FFI names, dead translated-but-unused helpers). Much of it is legitimately
  suppressible — but **at the item/module level with a reason**, not a crate-root sledgehammer
  that also hides real bugs (an unused `Result`, an unreachable arm, a shadowed write).

## Three tiers — sort every warning into one

### Tier 1 — NOISE (mechanical, subagent-fannable, the bulk)
Auto-fixable style: `unused_imports`, `unused_mut`, `unused_variables` (→ `_`-prefix),
redundant clones, `unused_parens`, needless `return`. **`cargo fix` handles these.**
- This is the **subagent work** the user green-lit ("sonnet or even haiku to burn down
  simple things"). Fan-out model per AGENTS.md: one crate per agent, agent runs `cargo fix`
  + hand-cleans the residual in its crate ONLY, writes the patched `.rs` to
  `~/vibeos-rust-staging/warnings-<crate>/`, does **not** edit shared files or build. **You**
  integrate + build + gate ×3 centrally, per cluster.
- Keep legitimate suppressions, but **narrow them**: crate-root `#![allow(non_snake_case)]`
  on an FFI crate is fine to keep (the whole crate is FFI); crate-root `#![allow(dead_code)]`
  is NOT — dead code should be deleted or `#[allow(dead_code)]`'d per-item with a reason.

### Tier 2 — SIGNAL (manual audit — NOT subagent work)
Warnings that can indicate real bugs. Do these yourself, carefully:
- **`clashing_extern_declarations`** (~60 in the last census) — the same extern symbol
  declared with two different signatures across crates. This is an ABI-mismatch class; it's
  what `verify_ffi.py` also hunts. Each one is a potential silent-miscall on amd64/arm64
  (links fine, wrong ABI) that only i386 would catch. Reconcile to one declaration.
- **`unpredictable_function_pointer_comparisons`** — `ob-rust/src/obname.rs`,
  `ob-rust/src/obsecure.rs`, `ps-rust/src/psnotify.rs`. Comparing fn pointers is
  optimization-fragile (LLVM may merge/split identical bodies). Verify each compares what the
  C intended; replace with a stable discriminant if the identity matters.

### Tier 3 — TOOLCHAIN TIMEBOMB (do FIRST — it's the only time-sensitive one)
**`deref_nullptr` / FIELD_OFFSET-via-null: 55 sites and GROWING** (~32 at the last census;
ole32/oleaut32/msvcrt landing added ~23). Shape: `addr_of!((*null::<T>()).field) as usize` —
the C `FIELD_OFFSET(T,f)` idiom translated literally. **On pinned rustc 1.92 it's a warning;
on a newer toolchain it's a HARD ERROR.** The stable-toolchain lane just landed; the next
rustc bump breaks the build on all 55.
- **Fix = `core::mem::offset_of!(T, field)`** — this is `scan_magic_offsets.py`'s domain
  (the offset-drift lint). Coordinate with it: the fix is exactly what that gate wants.
- Do this tier first and land it — it's the one with an external deadline (the toolchain pin).

## ⚠️ Coordination fence — do NOT churn crates active lanes are editing

A workspace-wide `cargo fix` touches ~200 crates. Landing that under an active lane's feet
creates brutal merge conflicts. **Before touching any crate, check `git worktree list` +
recent commits.** Known-HOT crates as of 2026-07-16 — **skip these until their lane lands**:

| Active lane | Hot crates — off-limits |
|---|---|
| `shell32-com` | `shell32-rust`, `tools/rustconv/**` |
| `smp-bringup` | `ntoskrnl-rust` (ke/mm), `ntkrnlmp-cargo`, `hal-rust`, `ke-*` |
| `msvcrt-homebrew` | `msvcrt-*`, `cnt-rust`, `libm`-side crates |
| `research` | `ole32-rust`, `oleaut32-rust` |

**Start with cold crates** — the driver fleet, the smaller algorithm crates, the settled
user-DLLs. There are ~200; there is plenty to do without touching a hot one. The timebomb
(Tier 3) is the exception where you may need to touch a hot crate's file — if so, hand that
lane a tested one-line `offset_of!` diff rather than landing it yourself.

## Ordering

1. **Census first** — real per-crate warning counts on the current tree (the 9,555 is stale;
   run `cargo build` capturing warnings, or per-crate `cargo check`, across all 3 arches).
   Record the baseline so progress is measurable. Sort into the three tiers.
2. **Tier 3 (timebomb)** — the 55 `deref_nullptr` → `offset_of!`. External deadline. Land it.
3. **Tier 2 (signal)** — clashing externs + fn-ptr comparisons. Manual. Land per-cluster.
4. **Tier 1 (noise)** — subagent fan-out over COLD crates, cluster-integrated + gated by you.
5. As hot lanes land, fold their crates into the Tier-1 sweep.

## Progress log

- **2026-07-16 — Tier-3 (timebombs) LANDED** (`5125e242d40` on master): deref_nullptr
  128 sites → `offset_of!`; unsupported_calling_conventions 164 sites (fastcall/stdcall,
  per-arch ABI preserved via a `fastcall_fn!` dual-emit macro); bindgen
  unnecessary_transmutes 345 sites → casts, **fixed at the generator** (`fix_transmutes`
  in all 5 postprocess_bindgen.py + standalone for usb/acpica) so regen stays clean.
  All three classes now **0 on i386/amd64/arm64**. Gate ×3 green.
- **2026-07-16 — Tier-2b (fn-ptr comparisons) + Tier-1 (cargo-fix noise sweep) batched.**
  T2b: 65 sites / 34 fns annotated `#[allow(unpredictable_function_pointer_comparisons)]`
  with a greppable `FNPTR-CMP(audited)` tag (all verified intentional NT callback/sentinel
  identity compares mirroring the C). T1: central `cargo fix` over ~80 cold crates, 3
  targets (~5k noise warnings). Also fixed a real swallowed-error bug found by triage
  (lsasrv `LsapLogonUser`, see below). Landed `eda1e0e9bc8` after a full gate-build
  recovery (see the "gate-build-is-the-oracle" lane rule for the cfg-blind botch recipe).
- **2026-07-17 — Tier-1 noise sweep batches 2-4 (the un-swept cold crates).** Same
  `cargo fix` + central-integration process, now with the full trap-handling recipe
  (3-arch **clean** build as oracle, committed_drift ×3, per-arch baseline reconcile).
  - **Batch 2** (`79256ccfb10`): win32k-rust 73 files (644 warnings) + samsrv + rtl-*.
    Over-removal reverted: rtl-diag/exception.rs (arm64 `ZwContinue`).
  - **Batch 3** (`87e41e59a4e`): fastfat/cmlib/afd + drivers, 71 files (~500). Two
    over-removals: cmlib/hiveinit.rs (i386/arm64 hive-recovery types) + smss-csrss/csrss.rs
    (`let mut Status` — a WARM-build-masked i386-cfg-mut regression the gate's clean build
    caught; hence the clean-build rule).
  - **Batch 4** (`b8185deb011`): winsrv-rust console, 30 files (~440). The deferred
    cfg-fragile crate — swept cleanly by running `cargo fix` on the **i386 target** (keeps
    the `winsrv_sys::nt::*` glob i386 needs for `SetWindowLongPtrW`); 0 over-removals.
  Real bug also fixed in batch 1: disk-rust 6 wrong STATUS_* constants (verify_constants).
- **2026-07-17 — `static_mut_refs` mechanical batch (the first "riskier" Tier-1.5 class).**
  ~272 `&STATIC`→`&raw const STATIC` / `&mut STATIC`→`&raw mut STATIC` conversions across
  56 files / 21 cold crates (samsrv, ks, win32k, smss-csrss, fastfat, classpnp, usbaudio,
  lsasrv, acpi, mountmgr, rtl-threadpool, uniata, …). **`cargo fix` cannot do this** (the
  suggestion is `MaybeIncorrect`), so the transform is driven by rustc's own byte-span
  JSON suggestions (`smr_apply_json.py`): capture `--message-format=json` from a
  `--keep-going` workspace build on all 3 targets, dedup convertible (pure `&raw` prefix)
  edits by byte-span, apply once per file descending. **The build is a total oracle here:**
  a `&raw` site genuinely *used as a reference* (`for x in &STATIC`, `&STATIC` passed to a
  `&T` param) fails to compile — never silently miscompiles. 22 such ref-needed sites
  (kdgdb, ks/pin, lsasrv/privileges iterators, services/database, win32k/keyboard+input+
  msgqueue, winsrv/terminal) were caught by the post-apply build and **reverted to
  reference form** (they stay as warnings — the honest state; converting them needs real
  refactor). All 3 arches build clean (0 real breakage). No generated files, no line-shift
  → **no baseline reconciliation needed** (every edit is a same-line in-place rewrite).
  Fenced (untouched): ntoskrnl-rust 53, oleaut32-rust 19, hal-rust, etc.
- **2026-07-17 — leftover-noise re-sweep ABANDONED (negative result, worth recording).**
  A `--keep-going` amd64 workspace census (past the freeldr/netstack plain-build bail that
  stops `cargo fix --workspace`) showed ~265 standard-fixable warnings still present in
  editable crates. Ran `cargo fix` per-crate over 33 of them → it yielded only 9 files and,
  critically, **re-introduced the exact cfg-blind over-removals already fixed in batches 1/3/4**
  (cmlib/hiveinit `#[cfg(not(bldr))]` imports, crt-startup/excpt x86_64 re-export, smss-csrss
  `let mut Status`, winsrv `use winsrv_sys::nt::*` glob). Stashed + dropped. **Lesson: the
  cargo-fix noise sweep IS complete.** What survives is the cfg-fragile tail that cargo fix
  *cannot* fix correctly (that's *why* it survived) — do not re-run `cargo fix` hoping for more;
  it will only re-botch the known-fragile files. The true remaining noise (unused_unsafe 219,
  unused_labels 156 cfg-gated, unused_macros 71 = intentional debug shims) needs per-site hand
  work or `#[allow]`, not autofix.
- **2026-07-17 — clashing_extern ARGCOUNT-6 investigated.** 5 unique syms: DrawStateW +
  FsRtlNotifyFullChangeDirectory are classifier false-positives (nested-callback commas
  miscounted; actually same arg-count, `Option<fn>` vs `*mut c_void` = ABI-identical); TracePrint
  is variadic-vs-fixed (benign on Windows x64 ABI); the two genuine ones — cdrom
  `DeviceErrorHandlerForMmc` (0 vs 4) + `DevicePowerSettingCallback` (3 vs 4) — are the
  same-crate-redecl pattern (fixable by `use`, address-of only so no live miscall). This is what
  surfaced the 520-symbol fix vector documented below. No live bug found; all latent.

## Remaining work (2026-07-17, prioritized) — the plain-buildable cold-crate noise is DONE

The `cargo fix`-reachable Tier-1 noise is largely burned down. What's left:
- **`static_mut_refs` — cold-crate `&raw` batch DONE (2026-07-17, ~272 sites, see log).**
  Residual owed: **(a)** the 22 ref-needed sites reverted to reference form — these need a
  real fix, and the cleanest is often `static mut`→`static` where the datum is provably
  read-only (e.g. lsasrv `WellKnownPrivileges`/`WellKnownRights` are `static const` tables in
  the C source, only ever indexed/iterated) — but that cascades into `unused_unsafe` cleanup,
  so it's a separate semantic pass, not mechanical. **(b)** fenced hot-lane sites (ntoskrnl
  53, oleaut32 19, hal) — do when those lanes are quiet. **(c)** freeldr/netstack sites (don't
  plain-build). The transform driver is reusable: `smr_apply_json.py` + `smr_revert_broken.py`.
- **`clashing_extern_declarations` (1457) — TRIAGED 2026-07-17 (classifier: `clash_classify.py`).**
  Parses the two `expected/found` fn-signature notes and buckets each clash by ABI impact:
  - **CONV `C`↔`system` — 277 sites / 207 syms — the priority, i386-REAL.** On i386
    `"system"`=stdcall (callee cleans stack), `"C"`=cdecl (caller cleans) → calling one
    through the other's decl is a stack-cleanup mismatch = corruption. On amd64/arm64 the two
    conventions are identical, so it's *invisible there* and the ×3 gate stays green — a
    latent i386-only footgun on less-exercised paths. Fix = make each local re-decl's `extern`
    string match the true export (NTAPI → `"system"`; genuine cdecl helpers → `"C"`); build
    oracle catches wrong guesses.
  - **ARGCOUNT differs — 6 sites — the highest-value needles.** A different *arg count* is
    almost always a real bug (a caller passing the wrong number of args) or a macro-shim
    artifact. Investigate these 6 first — cheap, potentially real.
  - scalar size/class 295 (BOOLEAN `u8` vs BOOL `i32` etc. — mostly benign return-register
    width), ret void↔value 181 (benign, void side ignores), ABI-identical 681 (pure
    pointer-name/const-mut noise). These ~1157 are reconcile-for-tidiness, not bug-hunts.
  Distribution: fastfat 390, win32k 185, cdrom 135, cdfs 118, user32 73, classpnp 66, disk 46.
  - **THE big safe fix vector — same-crate-redecl: 520 of 828 clashing syms (2026-07-17).**
    The dominant shape is NOT cross-crate: it's a driver crate **forward-declaring, in its own
    `extern "C" {}`/`extern "system" {}` block, a function that the SAME crate defines** as
    `pub (unsafe) extern fn`. The translator emitted a cross-module forward-decl with a
    *different* (often wrong) signature — e.g. cdrom `init.rs:79` `fn DeviceErrorHandlerForMmc();`
    (0-arg) vs the real 4-arg `pub unsafe extern "C" fn DeviceErrorHandlerForMmc(..)` in
    `sense.rs:253`; the forward-decl is used only for `SYM as *const c_void` (address-of, so no
    live miscall today — but a latent footgun if ever *called* through the wrong decl).
    Per-crate hit rate: cdrom 83/85, cdfs 63/65, fastfat 62/142, win32k 36/51, usbhub 25/28,
    user32 26/44, mountmgr 15/17, … (full table via the census one-liner in the progress note).
    **Fix = delete the clashing `fn SYM(..);` line from the extern block, add
    `use crate::<module>::SYM;`** (module resolved by grepping the crate for `pub .. fn SYM(`).
    **POC RESULT 2026-07-17: NOT mechanically fixable — the loose forward-decl sigs are
    load-bearing.** Ran the full transform on cdrom (83 decls → `use crate::<mod>::SYM`, clean
    tool run, `clashing_extern` dropped 83→0). Build ×3 = **80 × E0308 mismatched-types.** The
    forward-decls are NOT used only for address-of as first assumed — they're used for **calls**,
    and the call sites were written to the forward-decl's *deliberately-loose* signature
    (`*mut c_void`, generic WDF types) while the real fn takes specific struct pointers. The clash
    is ABI-compatible (pointer=pointer, so runtime-correct *today*), but re-pointing the name at
    the real typed fn makes every call site fail Rust type-checking. So the "fix" is not
    remove-decl-+-`use`; it's **per-call-site** (cast each arg to the real type, or loosen the
    real fn — both real work). Reverted (stashed+dropped). **Lesson: clashing_extern is a genuine
    per-symbol AND per-call-site audit; there is no mechanical decl-replacement sweep.** The tool
    (`clash_fix_crate.py`) + this result are the starting point for a *supervised* audit that also
    fixes call sites. Do the true-signal buckets (CONV i386, the handful of real ARGCOUNT) by hand
    first; the ABI-benign majority is low-value churn that needs call-site work per symbol. Task #4.
  - The rest (CONV, ARGCOUNT, scalar/void/noise buckets) as above. NOT a safe unsupervised bulk
    edit — a per-symbol audit (verify_ffi's domain). Do the same-crate-redecl POC + 6 ARGCOUNT
    + a CONV pass next. Task #4.
- **`unused_assignments` (~1436)** — triaged in batch-1 as verified-safe noise; `cargo fix`
  has no autofix for it. Low priority (deferred). Task #8 done as triage; suppression owed.
- **Non-plain-buildable crate noise** (freeldr/netstack/cdfs/config/kdbg) — these need the
  reactos-link/cmake build env, so plain `cargo fix` can't reach them; a cmake-env fix path
  or manual sweep is owed.
- **`-sys` crate bindgen noise** — mostly under crate-root `#![allow]` already (low-value).
- **Blanket-`#![allow]` peel** (task #9) — the durability endgame; still owed.

## Owed follow-ups

- **FNPTR-CMP re-architecture (user-requested).** The 34 `#[allow(unpredictable_function_pointer_comparisons)]`
  suppressions are the interim treatment. The idiomatic endpoint is
  `core::ptr::fn_addr_eq` (stable 1.85), which removes the lint *without* suppression.
  It doesn't fit the Option-wrapped / sentinel-value (`SIG_IGN`=`1 as fn`) / transmute
  forms ergonomically today. A future pass: `grep -rn 'FNPTR-CMP'` finds all 34 sites;
  convert the clean `fn == fn` ones to `fn_addr_eq`, keep scoped allows only for the
  sentinel/transmute oddballs.
- **Upstream ReactOS-C bugs faithfully preserved by translation** (found during
  unused_assignments triage, NOT translation-introduced — tracked here, not fixed in
  this lane): `pcix-rust/enumr.rs:1575`, `pcix-rust/init.rs:367`, `pcix-rust/utils.rs:714`
  (unchecked status / computed-but-unused / hardcoded wrong return status — each matches
  the C original, incl. a commented-out `//if (!NT_SUCCESS(Status)) break;` in init.c:752).

## Lane rules

- **Batch big, gate once** (2026-07-16 user directive, overrides the old per-cluster rule):
  roll mechanical work up into big commits and run ONE `vibeos-cli gate` at the end — at
  thousands of commits/month nobody reads commit-level history, and per-cluster gating
  serializes the fleet on the host CPU-lease pool. Validate cheaply as you go
  (`cargo check` per crate × 3 targets). Auto-push on ×3-green still applies.
- **cargo-fix `--broken-code` hazard (central-integration duty):** a workspace `cargo fix`
  mis-handles macro/cfg-blind cases — it removes imports/renames params used only inside
  macro invocations (e.g. `RTL_CONSTANT_STRING!(...)`), over-qualifies enum paths with
  unimported modules (`types::types_arch::X`), strips `mut` off a binding reassigned only
  under *another arch's* cfg arm (`let mut OldOffset` in freeldr i386 code), removes a
  `pub use win64::{…}` re-export the crate needs (crt-startup amd64), and drops a
  `use winsrv_sys::nt::*;` glob that on i386 is the *only* source of `SetWindowLongPtrW`
  (bindgen_i386 has just the 32-bit `SetWindowLongW`).
- **The oracle is the 3-arch `-k0` gate BUILD, not `cargo check` (2026-07-16, learned the
  hard way — a batch landed a green-looking commit that failed the gate build on i386+amd64).**
  `cargo check -p X` on the *default* target sees only ONE arch's cfg, so it green-lights
  edits that break a different arch's gated arm. Worse: crates that "don't build standalone"
  under `cargo check` (freeldr/uefildr/winsrv/kdgdb/netstack) are **not** exempt — they build
  fine in the gate via `reactos-link` (a build.rs-driven `cargo build -p X --target …`), so
  EXCLUDING them from validation hides exactly the cfg-blind botches. Do NOT trust "excluded
  standalone failure" as a reason to skip a crate. The good news: the cargo-fix noise class
  (drop `mut` / remove import / remove parens) can only fail by *not compiling* — so a
  `bash ../tools/canon-run.sh ninja -k0 livecd` on each x86 arch (+ the gate's arm64) is a
  **complete** oracle. Recipe when a batch breaks the gate: run `ninja -k0 livecd` per broken
  arch to enumerate the FULL casualty set in one pass (don't whack-a-mole one error at a
  time), then for each broken file either surgically restore the one botched line (`let mut`,
  the re-export) or, if the botch is dense/cfg-fragile (winsrv's whole console frontend),
  revert the *entire crate* to the pre-batch parent — `git checkout <parent> -- <crate>/`.
  **`<parent>` is `HEAD~1` once the batch is committed, NOT `HEAD`** (a `git checkout HEAD`
  restores the committed botch and no-ops).
- **Two per-arch gate checks a single-arch `cargo check` misses entirely (2026-07-16):**
  (1) **the lint baseline-diff runs PER ARCH** — `check amd64` reads `bindgen_amd64.rs`,
  whose line numbers differ from `bindgen_arm64.rs`, so a line-shift re-baseline verified only
  on arm64 can still fail `lint:amd64`. Re-baseline / verify on the SAME arch mix the gate runs
  (or just run `vibeos-cli check` for each arch). (2) **`committed_drift`**: cargo fix will
  happily "clean up" a committed **generated fallback** — an `unused_parens` removal in
  `winreg-stub/data/winreg_s.amd64.rs` (a widl-generated RPC server stub) byte-diverged it from
  regeneration → `lint:amd64` DRIFT. Generated/committed-fallback files (`*_s.<arch>.rs` RPC
  stubs, `cflags/*.flags`, `*.rc.res`, `bugcodes.h`, `ks*.inc`) must NEVER be hand- or
  cargo-fix-edited; revert any the sweep touched to `HEAD~1`. Detector: `python3
  tools/check_committed_drift.py --arch <a>` (13-14 artifacts, byte-compare vs cmake regen).
- **Validate cargo-fix batches with a CLEAN per-arch build, not a warm `-k0` (2026-07-17).**
  A warm/incremental `ninja -k0` recompiles the files you edited but may reuse a cached
  artifact for a crate it deems unchanged — so it can MISS an i386-cfg-`mut` or import
  regression the gate's clean build then catches. Batch 3 shipped a green warm i386 build
  but the gate's clean build failed: cargo-fix-amd64 stripped `mut` off
  `smss-csrss-rust/csrss.rs`'s `let mut Status` (i386 reassigns it in the
  `#[cfg(target_arch="x86")]` IOPL block; amd64 assigns once — the SAME csrss.rs case as
  batch 1). Restore from `HEAD~1` and re-validate with `vibeos-cli build --arch i386
  --clean` (full lint suite + stamp). Do the clean build BEFORE the gate so you don't burn
  a ~12-min gate cycle. This `mut`-strip class is the most common cargo-fix cfg hazard
  after import-removal — grep the batch for `^-.*let mut ` and spot-check any binding
  reassigned under an arch cfg.
- **Never suppress to green.** Removing a real warning by fixing it ✅; re-adding a blanket
  `#![allow]` to silence it ❌. If a warning is a genuine false-positive, `#[allow(...)]` it
  **per-item with a `// reason`**, never crate-root.
- **A fix that changes behavior is a bug, not a cleanup.** `cargo fix` is *supposed* to be
  behavior-preserving, but review every hunk — an "unused" `Result` it drops may be a
  swallowed error (`scan_silent_failures`' domain). When in doubt, keep the value and bind it.
- **Consider a gate at the end** — once a crate is clean, a lint that keeps it clean (deny the
  fixed classes for that crate, or a workspace `-D` for the ones fully eliminated) is what
  makes the burndown *stay* burned down. Design it so it can't be defeated by a new blanket allow.
