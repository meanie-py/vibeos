# The Ghost Tax — why builds and smoke are this problematic, and the program to fix it

*2026-07-04. Synthesized from five parallel research passes over the field manual
(all 206 documented incidents), the verdict pipeline (`tests/qemu/cli/`), the build
path (`build.py`/`provenance.py`/`canon-run.sh`), the defense inventory (every
`verify_*`/`scan_*`/`lint_*` tool), 60 days of git history, and the live
operational record (288 `vibeos-cli` runs over 36 hours in `/tmp/vibeos-cli`).*

This doc is the systemic answer to "builds and smoke shouldn't be this problematic."
It is not another incident writeup — the field manual already holds 206 of those.
It names the small number of properties that *generate* the incidents, quantifies
what they cost, and lays out the fix program (Tier 1 landed with this doc).

---

## TL;DR

1. **~16–19% of all project effort is ghost tax** — debugging masked or
   misattributed failures, plus building the tools to see them — **and the rate
   is rising** (11.8% → 18.6% of non-merge commits month-over-month). 7 of the
   top-10 churn-hotspot files are build/harness infrastructure, not kernel code.
2. **63% of all documented incidents come from one generator: two copies of the
   truth.** Hand-copied facts (ABI, constants, enums, layouts, signatures) and
   hand-translated logic drift silently between C and Rust. Every class that got
   a *mechanical* single-source-of-truth check stopped producing incidents; every
   class that didn't is still producing them.
3. **Detection exists; enforcement doesn't.** The repo has a remarkable defense
   arsenal, but the North-Star gate ran almost none of it in blocking mode, over
   half of the 206 incidents have documentation-only defenses (a grep someone
   must remember), and the one warning that guards the #1 masking channel (warm
   builds) fired on **100% of builds** in the last 36 h — a dead signal.
4. **The verdict semantics were tuned for the wrong era.** The gate's retry
   policy assumed HANG_TIMEOUT ≈ host contention (true in the native-KVM era);
   today's dominant real failures (service wedge, lost wakeups, kernel panics)
   present *exactly* as HANG_TIMEOUT — so the gate auto-retried the most
   expensive bug class. The verdict marker fired before the shell was up, the
   settle-window serial was discarded unread, and kernel Rust panics never
   reached the verdict at all (debugcon-only on x86, fully silent on arm64).
5. The "ghost" phenomenology — symptom decoupled from cause in space, time,
   environment, or masking — is mostly a *consequence* of 2–4. Triage skill is
   the tax you pay for prevention you don't have. The leverage order is:
   **kill the bug factories → make the verdict honest → then better triage.**

---

## 1. The evidence

### 1.1 The tax, measured (git history, 60 days ≈ the project's active life)

- 1,817 non-merge commits; **~72.7% translation**, ~10.8% harness/tooling,
  ~5.4% regression-debugging, ~11% docs. Core ghost tax (debug + harness) =
  **16.3%**, wide (+ the field-manual ledger) = **19.0%**. Rising: 11.8% (May)
  → 18.6% (June). Within "translation," only ~⅔ is strong forward-only work —
  the rest is debugging freshly-translated code.
- Churn hotspots: `Cargo.lock` (182 touches) + `Cargo.toml` (171) +
  `win32ss/CMakeLists.txt` (84) + the field manual itself (86) dominate; only 2
  of the top 10 are kernel source.
- Campaign archaeology: BUGCHECK `0x50` chased across **four unrelated
  subsystems** over seven weeks; `0x1E` hid ~10 distinct root causes; the
  service-wedge went through **six successive wrong root causes** over days,
  then fell in ~30 minutes once the sync instrumentation filmed it live.
  Instrumentation-to-fix ratio across the big campaigns: 40–60%.
- Every heavyweight defense (freshness nets, canonical path, census/synctrace)
  landed in the **final two weeks**, reactively, after the June false-green
  crisis.

### 1.2 The operational record (288 runs, 36 h, /tmp/vibeos-cli)

- **0 cold builds.** All 113 builds were WARN ("warm — cargo incremental may
  mask"). Nearly every green smoke rode warm artifacts. The warm flag has
  normalized into noise; nothing ever demanded cold.
- **71 amd64 HANG_TIMEOUTs at ~207 s median** (vs ~78 s green) ≈ 4 h of QEMU
  wall-clock on one bug's repro loop.
- **STALE_MEDIA hard-failed 10×** — the guard works, but each hit is a wasted
  round-trip where the harness knew exactly what needed rebuilding and made the
  agent do it by hand.
- amd64's only post-marker health coverage (the net stage) was skipped
  (`--no-net`) on ~113 of 116 runs; i386/arm64 have none at all in the CLI path.

### 1.3 The incident ledger (field manual, 206 incidents)

| Generator class | Count | Structural defense today |
|---|---|---|
| DUP-TRUTH — hand-duplicated C↔Rust facts (ABI ~30, constants/enums ~15, layouts ~15, signatures ~8) | 68 | partial (see §2.1) |
| XLAT — translator-output defects (dropped/stubbed bodies, wrong semantics: `!x`, cast-before-offset, element-vs-byte, missing decay, cfg-of-C-macro…) | 62 | mostly greps in the field manual |
| PROC — stale artifacts / build hygiene / wrong-tree | 32 | largely closed (provenance nets, 2026-06→07) |
| TOOLCHAIN-UB — captures(none), CGU-layout fragility, sacred-primitive elision | 22 | mitigations only, **no kill** |
| ENV — arch/accel/warm-cold/contention divergence | 12 | mostly closed (windows-host TCG, hostlock) |
| OBS — signal existed but nobody watched the channel | 10 | partially closed |

Defense status across all 206: **~28 gate-wired, ~70 manual-tool, ~108
documentation-only.** The largest bucket is "a human must remember a grep."

Worst effort-to-defense ratio: **TOOLCHAIN-UB** (the captures(none)/CGU-layout
family) — weeks of effort, the deep-C x86 cohab ungate still can't land, and the
two durable fixes (split-debuginfo bootable kernel, layout-robustness) are
un-started subprojects. Best ratio: **constants** — comparable cumulative pain
(207 constant/enum bugs across 4 sweeps), now structurally killed by the
whole-workspace `verify_constants` gate + CTL_CODE expander.

---

## 2. The diagnosis — four systemic causes

### 2.1 D1: Two copies of the truth (the dominant bug factory)

DUP-TRUTH (68) + XLAT (62) = **63% of all incidents**. The translation process
hand-copies *facts* and hand-translates *logic*; each copy can drift, and a
drifted copy compiles clean, links clean (lld fuzzy-matches decorations), and
often boots "green" — the first observable symptom lands far from the cause.

The cure is proven, three times over, inside this repo:

- **Struct layout** → generated `offset_check_/size_check_<arch>.rs` compile
  asserts: the whole class now *fails the compile*. Incidents stopped.
- **Scalar constants / CTL_CODE** → `verify_constants` whole-workspace gate:
  class killed (fastfat DISABLE_DIRTY, ramdisk CTL_CODE, LPC windows…).
- **Staleness** → provenance mtime nets + self-healing cohab manifest: the
  "bit six times" arm64 staged false-green class is mechanized away.

The classes still open are exactly the ones without a mechanical check: internal
(non-spec) ABI, enum members (now gated — Tier 1), per-arch constants, and the
long tail of XLAT semantic patterns that live as greps in field-manual §4.

**Principle: every fact that exists in both C and Rust must be either generated
from one source or mechanically compared against it in the gate. A hand copy
without a checker is a scheduled incident.**

### 2.2 D2: Detection without enforcement

The pattern across all five research passes: **every crisis produced a detector;
almost none got promoted to a blocking gate.**

- `vibeos-cli gate` (the North-Star verdict!) ran **no strict lints and no
  blocking binary scans** — it shells to plain `build` + `smoke`, where
  scan_ud2, sacred-primitives, and all nine source lints are warn-only.
  Blocking mode exists only in `check --strict`, which gate never invokes.
- Cold-before-land is prose in AGENTS.md; the repro gauge (`repro verify`,
  proven byte-identical on all 3 arches) is wired into nothing.
- The rc=0-with-no-artifacts build footgun had zero code guard (tribal
  knowledge only) — until Tier 1.
- The `lint_kernel` baseline holds 1,473 accepted findings; a blanket
  `--update-baseline` can silently bless a real regression into the accepted
  set.
- Warm-build WARN on 100% of builds = alarm fatigue by design.

**Principle: a detector that isn't wired into the verdict is a diary entry.
Promote or delete; and any warning that fires on every run must either become a
hard gate on some cadence or carry no gate semantics at all.**

A live specimen from the arm64 campaign (§5.4), because it is the exact shape
of D2 one layer below the documented cases: the **cargo-link dependency graph
did not track kernel asm sources.** The kernel `.exe`/`.sys`/`.dll` are
cargo-linked, and reactos-link assembles `ntoskrnl/ke/<arch>/*.S` (+ cc-rs C
shims) into them — but cmake's `add_cargo_module` `DEPENDS` globbed only the
Rust workspace (`sdk/lib/rust/**`), so an edit to `ke/arm64/trap.S` never
re-invoked cargo and shipped a **stale kernel with zero signal**. This is the
2026-07-04 Rust false-green (`gcc.cmake` comment) recurring against a different
source language: same root (the cargo link is opaque to cmake, so DEPENDS must
be complete), same failure (a "verified-fixed" binary that never rebuilt the
fix). Tier-1's build artifact-check does NOT catch it — media *was* freshly
staged from a genuinely-rebuilt-but-wrong ninja graph. The fix (glob the kernel
asm/`.inc` trees for the kernel module kinds) is landed; the general lesson is
that **any opaque build step must over-declare its inputs, and "I rebuilt and
re-tested" is not proof unless you confirmed the specific source object changed**
(objdump the symbol, or diff the artifact) — the discipline that caught this.

### 2.3 D3: Verdict semantics tuned for the wrong era

The harness's judgment calls encode the threat model of May (KVM contention,
early-boot crashes). The failure landscape moved; the semantics didn't:

- `RETRYABLE` included bare `HANG_TIMEOUT` ("probably contention") — but on the
  default windows-host TCG backend contention false-reds are largely solved,
  while the most expensive modern bugs (service wedge, lost wakeups, kernel
  panics silent on serial) present exactly as HANG_TIMEOUT. A 50% wedge passed
  a 3-attempt gate ~87% of the time per arch. Meanwhile the KNOWN_FLAKES list
  is *empty* (all three entries retired), so every retry was an undocumented one.
- The CLI verdict marker was `[EXPLORER] _tWinMain entered` — before the shell
  is up — and the 15–30 s settle drain *discarded* its serial. A BugCheck during
  taskbar build was a structural false green. (Docs claimed the marker was
  `shell ready`; the code disagreed.)
- Kernel Rust panics emit `PANIC:file:line` to debugcon 0xE9 only (x86) or
  nothing (arm64) → classified HANG_TIMEOUT → *retryable*.
- The kernel's own census/lockdep wedge self-reports (`[CENSUS] STUCK …`,
  landed 2026-07-04) printed to serial on every boot and **no classifier read
  them** — the "11/11 green yet eventlog silently dead every boot" state was
  invisible to the gate that reported the green.
- Boot duration is recorded per run and trended nowhere; a 2× slowdown is
  invisible until it crosses a fixed timeout and becomes a mystery HANG_TIMEOUT.

**Principle: the verdict must consume every signal the system already emits
(serial, debugcon, census, duration), and retry policy must be evidence-based
(retry what is characterized as environmental; block what is characterized as a
wedge), not classification-name-based.**

### 2.4 D4: One class has no structural answer yet — toolchain UB

captures(none)/CGU-layout fragility is the #1 effort sink with defense =
detection-only (scan_ud2 on x86) plus mitigations the field manual itself calls
whack-a-mole (cgu=256 firewall, per-site write_volatile, `#[inline(never)]`).
arm64 is worse: trap-unreachable is off, so the class is *silent* there — no
binary signal at all. The deep-C x86 ungate literally cannot land today because
any link-layout perturbation re-tips a latent miscompile (field manual §2,
R47). This needs a dedicated workstream, not another spot fix (§3 Tier 3).

---

## 3. The program

### Tier 1 — landed with this doc (harness policy corrections, unit-tested)

| # | Change | Closes |
|---|---|---|
| 1 | **Gate retry policy is evidence-based** (`gate.py`): HANG_TIMEOUT retries only when smoke characterized it slow/contended (serial flowing at deadline) AND no kernel wedge self-report; `LAUNCH_FAIL` (env/interop) becomes retryable; everything else blocks | the retried-away-wedge class; the false-red LAUNCH_FAIL hard-fail |
| 2 | **Kernel panics reach the verdict** (`triage.py`): raw debugcon scanned for `PANIC:file:line` → `RUST_PANIC` (never retried), in both the hang path and the post-marker path | kernel-panic-as-HANG_TIMEOUT (x86); the "retry a deterministic panic" hole |
| 3 | **Census/lockdep self-reports are consumed** (`triage.py`, `smoke.py`): on a hang they confirm the wedge (blocks retry); on a green boot they surface as a `health` WARN stage + `census_stuck` in summary.json (trendable) | "11/11 green but threads parked forever" invisibility |
| 4 | **The post-marker false-green window is closed** (`smoke.py`): settle serial is kept and scanned (BugCheck/panic ⇒ FAIL), and green requires `shell ready` after the marker (`SHELL_NOT_READY` otherwise; custom `--until` exempt) | crash-during-taskbar-build false greens; the code-vs-docs marker drift |
| 5 | **Builds self-verify their artifacts** (`build.py`): a "successful" build ends by running the same staleness check smoke uses; missing/stale media ⇒ `BUILD_INCOMPLETE` hard fail | rc=0-no-artifacts (overlay) footgun; stale-cargo-PE caught at build time instead of smoke time |
| 6 | **Enum drift is gated** (`lints.py` + `tools/lint-baseline/verify_constants_enums.txt`): `verify_constants --enums` as a baseline-diff lint — baseline is **empty** (tree is clean today), so it's a strict-zero gate | the AdjustUnwait lost-wakeup class (19 enum drifts found manually in the 07-04 audit; none were gated) |

Also: fixed the pre-existing `check-tooling` breakage on master
(`function_prompt.py` pyright + format), and corrected the CLI README's stale
marker/retry claims.

### Tier 2 — proposed next (cheap, high leverage; some need a policy decision)

1. **Kernel panic → serial on all arches** (kernel change, smoke ×3 to land):
   mirror the panic text to the serial path (and PL011 on arm64, where today a
   kernel panic is fully silent — field manual R122/R123 did this ad-hoc once).
   Debugcon scanning (Tier 1) covers x86; this closes arm64.
2. **Cold-cadence policy** (user decision): `gate --clean` stays opt-in, but
   require **one cold gate per landing** (or per day, enforced by the stamp's
   `cold` bit + a gate refusal when the last cold gate is older than X). The
   repro gauge (`repro verify`) becomes a weekly scheduled check. This converts
   the dead warm-WARN into a real invariant without making every iteration pay
   the ~5 min cold cost.
3. **Gate runs the lint arsenal in blocking mode**: `gate` invokes
   `check --strict` (baseline-diff NEW=0) + binary lints per built arch. The
   baselines make this a no-new-regressions bar, not a boil-the-ocean bar.
   Prerequisite hygiene: a **single-tool** `--update-baseline <tool>` so
   accepting one tool's FP never silently blesses another tool's regression.
4. **verify_abi internal-symbol coverage**: harvest Rust-side `#[no_mangle]`
   defs (the `verify_signatures` harvest already exists) and check every
   def↔decl pair crate-wide, not just spec/bindgen names. This is the #1
   remaining ABI blind spot (field manual R6/R7: ESP-drift bugs that passed
   verify_abi ✓ verify_ffi ✓ scan_ud2 ✓).
5. **Boot-duration trending**: summary.json already carries duration; keep a
   rolling per-arch baseline (e.g. qemu/<arch>_boot_baseline.json) and WARN at
   >1.5×. Turns slow-drift regressions into early warnings instead of timeout
   mysteries, and separates real contention from perf regressions.
6. **STALE_MEDIA auto-heal**: `smoke --rebuild-if-stale` (and make gate use it)
   — the preflight already knows exactly what is stale; 10 manual round-trips
   in 36 h were pure friction.
7. **Net-stage cost fix**: pass `--skip-clean-build` to the folded net suite
   (it currently does a full clean kernel rebuild + second boot inside every
   non-`--no-net` amd64 smoke), and fold a milestones assertion into the CLI
   path for i386/arm64 so all arches get post-boot coverage (partially done by
   Tier 1's shell-ready gate).

### Tier 3 — structural workstreams (each deserves its own plan doc)

1. **Toolchain-UB kill program**: split-debuginfo bootable kernel (make
   miscompiles *nameable*), the layout-robustness experiment, evaluate
   `-Z trap-unreachable=yes` (or equivalent) for arm64 so silent UB gets a
   binary signature scan_ud2 can see, and a reserved-register/xchg-width
   scanner (the class that cost the multi-hour BUGCHECK 0x50 hunt has no
   detector at all).
2. **XLAT lint pack**: mechanize the recurring field-manual §4 greps into one
   baseline-gated `tools/lint_xlat.py` (candidates, all with documented
   incidents: `!fn() != 0`, cast-before-offset, `as u8/u16` narrowing in size
   math, `write_bytes(typed_ptr, _, size_of)`, element-vs-byte counts on wide
   copies, `LIST_ENTRY` self-ref init, `[_;0]` flexible-array indexing,
   `#[cfg(<C macro>)]`, empty arch-cfg stubs, `Length: 0` wide-string globals,
   simulated va_start). Each is a 10-line AST/regex rule; together they cover
   the largest doc-only bucket.
3. **Build-cost structural fix**: build-std is recompiled ~98× per cold build
   by deliberate per-crate isolation (correctness), with only a detached
   daily hardlink dedup (113 GiB reclaimable fleet-wide). Investigate a shared
   sccache-friendly build-std or synchronous dedup on the cold path — this is
   the single biggest lever on cold-build cost, which is itself the reason
   nobody runs cold (D2).
4. **Per-arch constant catalogs**: verify_constants skips
   `ARCH_DEPENDENT_CONSTANTS` entirely; the HIGH_LEVEL=31 → `mov cr8,0x1F`
   class (C1) stays undefended until the catalog is arch-parameterized.
5. **Docs reconciliation**: the two determinism docs contradict each other on
   Rust path-remap (implemented-on-stable vs removed-for-sccache); the CLI
   README's marker claim contradicted the code until today. Stale docs are how
   "outdated lore" (arm64-is-idle-detection) propagates into agent decisions.
   One pass to make `deterministic-builds-plan.md` defer to
   `build-cache-and-determinism-plan.md`.

### What this program is **not**

- It does **not** weaken any smoke test — every Tier-1 change makes green
  *harder* to achieve, not easier (shell-ready requirement, post-marker scan,
  wedge-blocking retries, build self-check).
- It does not replace the field manual — triage skill still matters; the
  program just aims to stop growing §1–§8 at the current rate.

---

## 4. The triage payload (the reusable rules, kept from the ghosts research)

The meta-rules that generalize across all 206 incidents — the part of the
original "ghosts" writeup worth keeping in agents' heads:

1. **The two canaries point opposite ways.** i386 surfaces ABI/decoration bugs
   (`_Name@N`); i386 *hides* 64-bit width/layout/FP bugs. **2-of-3 arches green
   is never exoneration — *which* arch is green tells you the bug class.**
2. **A "shouldn't-matter" change that breaks boot is the trigger, not the
   bug.** Chase the thing that moved (layout/timing/pool-order), not the mover.
3. **Green can mask too.** A green verdict that didn't check services, panics,
   or freshness proves only what it checked. (Tier 1 narrowed this; it can
   never be zero.)
4. **A partially-documented ghost is worse than an undocumented one** — its
   other faces get misfiled as fresh regressions. Document all faces or none.
5. **Names manufacture ghosts.** `arm64_lpcexitthread_race` was a stack
   overflow mislabeled by a catch-all marker for a whole session. Verify a
   label's evidence before inheriting it.
6. **Same symptom, multiple causes — the symptom is a ghost.** "ud2" has ≥4
   root causes; check ABI first via `nm` (1 s) before chasing captures(none).
   BUGCHECK 0x50/0x1E are low-specificity signals, not diagnoses.

Instruments that can lie (check the *current* status before trusting):
link-clean (lld fuzzy-matches decorations, CC-folds ship), cargo-check ×3
(misses per-arch inline-vs-export link breaks), verify_abi ✓ (internal symbols
— open until Tier 2 #4), smoke green (see D3 — narrowed by Tier 1), 2-of-3
arches (rule 1), a lone red repro under contention (check `vibeos-cli ps`), and
the working tree itself (check `git fetch` + `rev-list HEAD..origin/master`
before concluding code doesn't exist).

---

## 5. Day-one field results (2026-07-04, the first hours of Tier 1)

The new verdicts produced three real findings before this doc was even landed:

1. **i386 has silently-stuck threads on every green boot too.** The very first
   smoke through the health stage reported 3 `HOLDER-STUCK(no signal ever)`
   threads (System pid=4, svchost…) on a green i386 boot; the ×3 gate then
   showed `census_stuck: 5` on green i386 AND amd64. The Mm WS self-deadlock
   class was documented as amd64-only; it is not. Trend this number — it must
   drop to 0 when the Mm fix lands.
2. **The triage regexes didn't know the kernel's own BUGCHECK banner.**
   ke/bug.c prints `*** BUGCHECK: 0xNN`; the classifier only matched
   `BugCheck 0xNN`/`*** STOP`, so a live arm64 0xF4 classified HANG_TIMEOUT —
   which the pre-Tier-1 policy would have auto-retried as a flake. Fixed
   (`7930f1f62d8`) within an hour of the new gate refusing to retry it.
3. **arm64 is broken on mainline right now and nobody knew.** The explorer-rust
   merge (`90fe9201f7a`, 12:24) landed with i386-only boot validation
   ("shell boots i386"); no arm64 boot happened between the merge and the
   first ×3 gate through this harness (15:42). Characterized: **~1/5 green**;
   dominant face = silent wedge after `[mpr] wnetInit: EXIT` with
   `[EXPLORER] _tWinMain` never printed (explorer.exe — now 100% fresh Rust —
   dies/wedges during early init); secondary face = smss.exe killed by a wild
   jump to a kernel VA (`PC=0xFFFFF8004219280C`, insn `msr spsr_el1, x1`,
   BUGCHECK 0xF4). The merge's only non-explorer guest surface is
   Cargo.toml/Cargo.lock (workspace shape — the documented CGU-reshuffle
   trigger, §2.4/R50), so the candidates are an explorer arm64 translation bug
   or a layout-tipped latent arm64 miscompile. Either way: this is D1+D3 in
   one specimen — a 2-of-3-arches-validated landing, plus a gate whose old
   retry policy could have kept it green.

## 6. Measuring whether this works

- **Ghost-tax rate**: re-run the commit classification (subject-line buckets,
  §1.1 method) monthly. Success = the debug+harness share falls back below
  ~10% and stays there while translation volume holds.
- **census_stuck trend**: now in every green summary.json. Success = it drops
  to 0 when the Mm WS self-deadlock fix lands — and *stays* 0 (a regression
  becomes visible the day it lands, not months later).
- **Retry honesty**: gate matrix now records hang_kind/wedge_confirmed per
  attempt. Success = zero `recovered` arches whose first failure was
  wedge-shaped.
- **Cold coverage**: % of landings with a cold gate (from the stamp's `cold`
  bit). Success = 100% under the Tier-2 cadence policy.
