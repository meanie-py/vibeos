# Tooling Consolidation Plan

*2026-07-04. Synthesized from four parallel read-only surveys of the tooling estates
(translation / debugging / control-plane+CI / scanners+build-infra). This is a durable
work-list — the diagnosis + a tiered, file:line-targeted execution plan. Line refs are
as-of HEAD `b245730b27a`; **re-verify each target before editing** (the tree moves fast).*

## Framing

The tooling is the **fossil record of the debugging campaigns** — each crisis spawned a
tool (instrumentation-to-fix ratio was ~40–60% per the ghost-tax doc), and almost nothing
was consolidated afterward. This is not "100 bloated tools"; it's **five structural
diseases sitting on a layer of dead weight.** The cure is *one front door per task, one
lint pass, one arch source of truth* + a dead-weight sweep — not deletion for its own sake.

The four estates, roughly:
1. **Translation** — `rustconv`: 26 CLI subcommands + ~55 standalone scripts + `.responses` cache.
2. **Debugging** — bash `qemu/*.sh` (~14) + Python `vibeos-cli gdb` engine + `vibeos-cli debug` + `qemu/gdb-helpers/*.py` decoders + rust-side instrumentation (lockdep/census/flightrec/synctrace).
3. **Control plane / CI** — `vibeos-cli` (14 subcommands) over `tests/qemu/` (vm.py/conftest/triage/backend) + raw `qemu/run.sh` + direct `pytest`; plus the footgun program (health/ledger/pre-push/isolation).
4. **Scanners + build-infra** — scan_ud2 / lint_kernel / verify_sacred_primitives / scan_inline_flag_literals / verify_{constants,abi,ffi,signatures,tables,match_consts} + canon-run / cachecfg / provenance / prune / repro / (retired) dedup.

---

## The five diseases

### D1 — Front-door multiplicity (dominant)
Every core task has 2–8 entry points, and picking the wrong one **silently fails**
(native breakpoints don't fire; the Python engine raises `NotImplementedError`).
- **boot a VM: 8+** — `vibeos-cli smoke` / `boot` / `gate` / `debug start` / `gdb start` / `qemu/run.sh` / `qemu/start.sh` / raw `pytest test_smoke.py`.
- **build: 4** — `vibeos-cli build` / `run.sh --rebuild` (inline `ninja`, bypasses cache/canon/lints) / `conftest._ensure_clean_rust_build` / raw `ninja`.
- **attach GDB: 4** — `gdb` (Python/windows-host) / `debug` (bash/native) / `start.sh --gdb` / `run.sh --debug`.
- **check lints: 5** · **translate: 3** (`submit` / `batch` / `translate`).

### D2 — The verify_* / lint family is fragmented AND unenforced ⚠ (the one correctness bug)
- **Fragmented:** the ABI cluster is a strict-containment ladder run 2–3×:
  `verify_abi ⊂ verify_ffi ⊂ verify_signatures`, + `check_calling_conventions.py` (dead 4th copy).
  `verify_ffi.py` re-implements `verify_abi.py`'s exact extern-C/stdcall check against the *same*
  spec+bindgen catalogs (`verify_ffi.py:104,126,423-426` vs `verify_abi.py:153-180`).
  `verify_constants` + `scan_inline_flag_literals` are two halves of one bug class
  (`lints.py:176-178`). 8 source scanners → ~4 rules under one driver.
- **Unenforced (correctness gap):** *nothing on the land path blocks on the scanner suite.*
  `gate()` = build + smoke only (`gate.py:159-209`); the lints run **warn-only** inside `build`
  (`build.py:748`, `strict=False`); blocking exists only in `check --strict` (`debugcmd.py:98,110`),
  a command a human must remember. The footgun program enforced build/isolation/gate-stamp but
  left the lint arsenal opt-in. On **arm64 there is no `ud2`** (`scan_ud2.py:10-18,260-267`) →
  `lint_kernel` is the *sole* `captures(none)` defense, and it's advisory.

### D3 — No single source of truth for arch/config
- Arch resolved in **4 places**: `archcfg.py:18-29` + `vm.ARCH_CONFIGS`/`get_arch_config` (`vm.py:772-786`)
  + `conftest._base_arch` (`conftest.py:50`) + bash (`run.sh:72-236`, `start.sh`/`lib/manifest.sh`).
  Plus 3 private per-arch *target* tables: `lints.py:227`, `verify_signatures.py:104-126`, `scan_ud2 find_objdump:123-127`.
- `qemu/run.sh:72-236` carries a **drifted** table (still BIOS `-M pc` + KVM, `run.sh:163-168,221-222`) —
  the exact "fast-path diverges from smoke" divergence the CLI exists to kill, still live.
- **9 `VIBEOS_*` env vars across 6 files, two config disciplines**: only the 5 `VIBEOS_QEMU_*`
  route through layered `config.py:68-74`; `VIBEOS_ROOT`/`BUILD_CACHE`/`ARTIFACT_ROOT`/`WHPX_IRQCHIP`
  are read ad-hoc via `os.environ`. (Good pattern exists: `cachecfg.py` and `config.py` *are* single-source.)

### D4 — Dead weight (orphaned/superseded, not removed)
- **rustconv:** 25/35 standalone scripts have 0 references (`freeldr_*` ×13, `fix_ke_*`, `dedup_*`,
  `bindgen_*`, `remove_shadow_defs`, `strip_wdf_externs`, `triage_residual`, `reassemble_from_cache`,
  `restitch_decls`, `preseed_cache`); `submit`/`batch` superseded by `translate`; two guard-migration
  impls (`migrate_guards.py` regex vs `migrate_guards_ts.py` tree-sitter, both orphaned from the CLI);
  stale dirs (`.responses` 41 MB/533 subdirs, `.responses-backup-pre-c2rust/`, `classpnp-gap-staging/`,
  `out/`, `.semdiff/`); legacy model aliases (`__main__.py:119-121`).
- **debug:** `list-threads.py` DWARF-dead on the Rust kernel (`list-threads.py:6-10`, superseded by
  `vibeos-threads`); phantom `walk-procs.py` referenced but nonexistent (`debug_block.py:211`,
  `sync-instrumentation-design.md:65`).
- **scanners/build:** `check_calling_conventions.py` (orphan, 0 refs); dead `_maybe_dedup` body
  unreachable after `return` (`build.py:263-282` + `_DEDUP_INTERVAL`); `tools/dedup-build-cache.sh`
  auto-path dead (manual-only now); det-remap/copy-recipe tombstones in 4 files.

### D5 — Backend-split debug duplication (facet of D1+D3)
bash toolkit (native/KVM) vs Python engine (windows-host/TCG) are **complementary, not redundant** —
neither is a superset, so *you can't retire either* (native/CI-Linux would lose its debugger; the
Python "port" lacks preflight/reap/tail/snapshot/offline-sym). But the newest decoders
(lockdep/census/flightrec/threads) are reachable **only** via Python because bash `inspect.sh:86-94`
never sources `debug_block.py`.

---

## The tiered work-list

### T0 — Mechanical sweep (do first; highest ratio, ~Low risk)
Each is a deletion/archive with near-zero behavioral risk. `git log`-check the orphaned scripts
before deleting (some may be episodically hand-run) — **archive, don't delete**, when unsure.

- [x] **DONE 2026-07-04** — Deleted `tools/check_calling_conventions.py` (orphan; only a docstring
      mention in `verify_signatures.py:5`, which was tidied).
- [x] **DONE 2026-07-04** — Dropped the whole dead `_maybe_dedup` fn + `_DEDUP_INTERVAL` + call site
      (`build.py`); left a one-line tombstone comment at the call site pointing to the retirement rationale.
- [x] **DONE 2026-07-04** — Archived 24 orphaned campaign one-off scripts → `tools/rustconv/archive/`
      (freeldr_* ×13, fix_ke_* ×3, dedup_* ×2, bindgen_flatten_forwarders, remove_shadow_defs,
      strip_wdf_externs, triage_residual, autocast, fix_misc_method_variadic — all ref-count-0). README added.
- [x] **DONE 2026-07-04** — Removed local scratch `tools/rustconv/.semdiff/` (untracked, gitignored).
- [ ] `qemu/gdb-helpers/list-threads.py` — **NOT a clean delete** (plan was wrong): it's wired into
      `gdbengine.py:55,79` (HELPER_FILES + command map) and `inspect.sh:93,155,156`, and the field manual
      (`:360`) treats reviving it (once split-debuginfo lands) as owed. This is an archive-**and**-rewire, or
      leave-as-is. **The phantom `walk-procs.py` refs are PROSE** (`debug_block.py:211`, design-doc:65 mention
      it positively as *retired*), not broken refs — nothing to fix there.
- [ ] Guard-migration impls (`migrate_guards.py` regex vs `migrate_guards_ts.py` tree-sitter): both refs-0
      but documented as a complementary **pair** (scanner + transform) in `rust-kernel-patterns.md:213`.
      Merge is a judgment call, not mechanical — deferred.
- [ ] **HELD — needs user approval (paid artifacts, NOT dead weight — plan's D4 was wrong):**
      `classpnp-gap-staging/` holds 3 paid Rust translations (`ClasspBuildRequestEx.rs`, `class_gap*.rs`);
      `out/` holds a *live* cache (`seh_llm_lower_cache.json`, used by `seh_llm_lower.py:25`) + a deliberately
      `misroute-preserved` paid `api.rs`; `.responses-backup-pre-c2rust/` is a backup of paid responses.
      None may be removed without explicit approval (AGENTS.md translation-preservation rule). Separately:
      `.responses/` (41 MB) still wants a body-hash-keyed GC policy — real, deferred.
- [ ] Drop legacy model aliases (`__main__.py:119-121`); reconcile the 4 backends + Codex path to one list.
- [ ] Prune det-remap/copy-recipe doc tombstones to one; fix the AGENTS.md "Worktree Discipline #3"
      copy-recipe if it's now stale post-canon (`canon-run.sh:17-22`).

### T1 — Within-estate merges (S/M, Low-Med)
> **2026-07-05 re-scope: the tree moved a lot since this plan was drafted (`b245730b27a`);
> several T1 specifics are now STALE.** The rustconv CLI is down to ~15 subcommands with only
> 2 `seh-*` win32k passes (not "10 of 26") — other lanes already trimmed it. Re-verify each
> item against HEAD before doing it; the win32k-namespacing item in particular is now moot.
- [x] **DONE 2026-07-05 (commit e0ddb35efee)** — Fold `verify_abi` into `verify_ffi`. Confirmed
      zero coverage gap (verify_abi's 62 findings ⊆ verify_ffi's 135); dropped the redundant lint
      run + orphaned baseline. verify_abi.py kept (abi_map.py imports its parse fns — it's a library).
- [ ] **Merge `verify_constants` + `scan_inline_flag_literals`** into one constant-value rule (two matchers).
- [ ] **Retire `submit`/`batch` → `translate --batch`** — CAREFUL: touches the paid-translation path;
      verify no gate/script depends on them first. (Deferred — cost-sensitive, low streamlining value.)
- [ ] **Fold `boot` → `smoke --fast/--no-gate`** — LOW VALUE: `boot()`/`smoke()` already share
      `_one_boot` (`smoke.py:432` / `:350`); only the thin wrappers differ (boot adds `--until`/no-gate).
- [ ] **Collapse the 4 cache→crate reassemblers** (`reassemble_from_cache`/`restitch_decls`/`reinject_dropped`/`preseed_cache`)
      into one `rustconv reassemble` subcommand (preserve the drop-recovery behavior). Re-verify they still exist.
- [x] ~~Namespace the win32k passes~~ **MOOT 2026-07-05** — rustconv now has only 2 `seh-*` passes
      (`seh-classify`/`seh-inventory`), not the 10 the plan cited; namespacing 2 commands isn't worth it.

### T2 — Cross-cutting structure (M/L, Med)
- [ ] **One arch source of truth.** Merge build-side `archcfg.py` + runtime `vm.ARCH_CONFIGS`; delete
      `conftest._base_arch`'s private copy; centralize the 3 scanner per-arch tables into `archcfg`; make
      `run.sh` source a generated env from `vm.ArchConfig` (or retire its arch table + `--rebuild` in favor
      of `vibeos-cli boot`/`build`). Kills the D3 divergence class permanently.
- [ ] **One debug front door.** Merge `debug` + `gdb` into one command that dispatches native↔windows-host
      by `qemu.backend` (already available, `gdbengine.py:438-442`); source `debug_block.py` in bash
      `inspect.sh` + register the `vibeos-*` verbs (closes the D5 asymmetry); decide preflight/reap/tail/snapshot
      (port to Python, or explicitly declare bash the shared substrate the Python engine calls).
- [ ] **Route ad-hoc `VIBEOS_*` through `config.py`** `_ENV_MAP` (`config.py:68`): `VIBEOS_ROOT`,
      `VIBEOS_BUILD_CACHE`, `VIBEOS_ARTIFACT_ROOT`, `VIBEOS_WHPX_IRQCHIP` (keep names as aliases).
- [ ] **Collapse `test_smoke.py` ↔ `smoke.py` boot/screenshot dup** (`smoke.py:73-101` mirrors
      `test_smoke::test_screenshot`) into one shared boot-and-capture helper.

### T3 — The correctness fix ⚠ (M, Med) — **DONE + VALIDATED 2026-07-04**
- [x] **Make `gate` run the scanner suite blocking** (invoke `check --strict` per built arch, baseline-diff
      NEW=0), so the arsenal enforces on the land path instead of warn-only. Natural next footgun increment
      (pre-push already requires the gate stamp; add lint-blocking to what mints it). Highest correctness
      value: on arm64, `lint_kernel` is the sole `captures(none)` defense and is currently advisory.

  **2026-07-04 — pre-work analysis done; the wiring is the LAST step, not the first.** Running
  `check --strict` today is RED, but every failure was triaged (proof in this session):
  - **74 of ~75 lint findings = line-drift** against baselines that are weeks stale (Jun 13, NOT the
    "eve regen" — that was an uncommitted ghosts-lane copy). PROVEN content-identical once line numbers
    are normalized: verify_abi 64=64, verify_ffi 139=139, verify_signatures 4=4, lint_kernel 574=574
    write-stmts, **0 genuinely-new**. → **regenerate baselines** (`check --update-baseline`); safe, nothing
    to launder.
  - **1 genuinely-new** = shellcheck SC2164 `catch_bugcheck.sh:14`. **FIXED** (commit dd5c4c1332c).
  - **committed_drift** was crash-masked + canonical-path-blind → 3 fixes **DONE** (dd5c4c1332c): checker
    now skips (not crashes) when the canonical mount is absent; `gen_cflags` templatizes `/tmp/vibeos`→`{REPO}`
    (4 false cflags DRIFTs cleared). Residual: **2 GENUINE widl `.rs` drifts** (svcctl_s/winreg_s) — committed
    stubs predate a `widl_to_rust` codegen improvement (`as u32` casts); benign, cargo-off path only.
    **OWED: regen the committed widl stubs (×3 arches) + validate.**

  **Remaining to land T3 (all consequential — bless current state / expensive gate; confirm before doing):**
  (a) regen the 2 widl committed stubs; (b) `check --update-baseline` (proven safe); (c) wire `gate` →
  `check --strict` per built arch, block the stamp on NEW>0; (d) validate with a full `vibeos-cli gate`.
  Note: `committed_drift`'s `.res`/canonical checks only run truthfully INSIDE a canon-mount — decide whether
  the gate's strict pass runs them (skip outside canon, as now) or the check moves into the build (canon
  context). Risk from the plan ("surfaces baselined-but-real findings") turned out NIL — the arsenal's
  current output is all-accepted; the only real bug it caught was the widl codegen staleness above.

  **LANDED green on master (push `fc794e7c29e..8ed66e12e01`, 2026-07-05).** First full gate was red
  ONLY on the pre-existing arm64 `LpcExitThread`/`0x4E` wedge (non-causal to T3 — tooling-only edits);
  after merging the `flake-arm64-lpcexit` fix (`MiPteToX` self-map recursion), the re-gate was **×3
  green + stamp minted**: build ×3 ✓, **lint phase 1.5 ✓ on all three arches** (`check --strict` clean),
  smoke i386 ✓ amd64 ✓ (net-flake recovered on retry) arm64 ✓. The T3 mechanism is proven live: a NEW
  lint finding on any built arch fails the gate + blocks the push stamp; `--no-lint` opts out. Landing
  also fixed 3 real scanner bugs (committed_drift canonical-path family) + bootstrapped two baselines
  (all deltas triaged: line-drift + tool FPs, zero actionable bugs).
  **Residual (owed): committed_drift's canonical checks (`.res`) still SKIP outside a canon-mount — a
  follow-up should run committed_drift INSIDE the build (canon context) so `.res` drift is actually
  gated, not skipped.**

### Longer-term (after T1)
- [ ] **Unify the FFI cluster on `verify_signatures`** (clang-AST — the strongest oracle) + fold in
      `verify_ffi`'s two unique checks (macro-as-extern, flex-array UB); retire `verify_abi` entirely.
      4 files → 1. (L, Med-High: reconciles spec/bindgen-regex vs clang-AST ground truths + baselines.)

---

## Target end state

`vibeos-cli {build, smoke, gate, check, debug, health, translate?, prune, repro, host-status, ps}`
— **one door per task, backend-dispatch *inside*** (debug), `boot` folded into `smoke`. One `check`
that runs ~4 unified lint rules ([1] FFI-boundary, [2] constant-value, [3] output-ptr-write, [4]
differential-tables) and that **`gate` enforces**. `archcfg` as the lone arch authority feeding the
runtime table, the scanner tables, and `run.sh`. Dead weight archived. Debug: keep both backend
engines but under one dispatching front door with symmetric decoder coverage.

## Sequencing recommendation
1. **T0** (~a day, near-zero risk) — removes the bulk of visible sprawl.
2. **T3** (the correctness gate) — the only item that's a bug fix, not tidiness.
3. **T2 "one arch source of truth"** — highest structural value (kills the run.sh-divergence class).
4. **T1 merges + the rest of T2** — parallelizable per estate.

Parallelizable across the fable fleet by estate; T0 + T3 are good to drive centrally (they touch
shared control-plane files). Re-verify all file:line refs against HEAD before editing.
