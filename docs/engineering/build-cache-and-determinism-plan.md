# Build Cache & Determinism Plan

Status: **implemented + measured on i386, opt-in** (brainstormed + implemented 2026-06-14;
real-tree measured 2026-06-15; lane = `build-cache` branch / `vibeos-wt-buildcache`).
Headline: **sccache 100% Rust hits across the clean-smoke target nuke**, cache proven
codegen-safe. Owner: build-cache lane. Prereq for nothing; unblocks faster iteration.

## Implementation status (2026-06-14)

Landed in `tests/qemu/cli/` (opt-in via `vibeos-cli build --cache` or `VIBEOS_BUILD_CACHE=1`;
**off by default** so a cache bug can't wedge anyone's build):

- **`cachecfg.py`** — the single owner of cache env. `VIBEOS_ROOT` (env, else repo's parent
  dir = the worktree-parent) → `CCACHE_DIR`/`SCCACHE_DIR` + `CCACHE_BASEDIR`/`nohashdir` +
  `RUSTC_WRAPPER`/`CARGO_INCREMENTAL=0`. Pure stats parsers (`ccache --print-stats`,
  `sccache --show-stats --stats-format=json`).
- **`build.py`** — `--cache` reconfigures the tree with `-DENABLE_CCACHE=1` (only if not
  already set; ReactOS already wires `RULE_LAUNCH_COMPILE ccache` behind it), injects the
  cache env into the ninja subprocess, zeroes counters pre-build, and reports per-build
  hit-rate stages (the determinism gauge).
- **8 unit tests** in `test_cli_units.py` (root resolution, env overlay incl. the
  "never set RUSTFLAGS" safety assertion, both stats parsers).

**Validated end-to-end** (real toolchain, the exact env `cachecfg` emits):
- ccache: a second compile of the same TU from a *different worktree path* **hit** the cache
  (cross-worktree, via `CCACHE_BASEDIR`+`nohashdir`) — the core of pain 2.
- sccache: Rust crates **hit** after `rm -rf target` — exactly the smoke-harness nuke (pain 1).

**Not yet measured on the real tree** (needs a `build-<arch>` in the lane): full-build hit
rates (rollout §4) and the warm-fresh-worktree-build vs `cp -a` link-cost comparison (§5).
Those gate retiring the copy recipe and flipping the default on.

Tools installed on this machine: `ccache 4.5.1` (apt), `sccache 0.8.2` (release binary →
`/usr/local/bin`).

## The problem (two pains, one root)

Two complaints, same root cause — **there is no content-addressed build cache**:

1. **Builds aren't cached.** Cold/clean rebuilds redo identical compilation from scratch.
   The smoke harness deliberately *nukes* the Rust target dir each clean run (to avoid
   incremental masking — see "Smoke Tests" in AGENTS.md), so every clean smoke pays a full
   Rust rebuild.
2. **We copy build dirs across worktrees.** Because CMake bakes absolute paths into
   `CMakeCache.txt` / `build.ninja`, a worktree can't share the main build dir (and a
   *symlinked* build dir corrupts main's cache — the documented footgun in AGENTS.md
   "Worktree Discipline" #3). So each worktree gets a `cp -a build-<arch>` + a fragile
   sed-repoint of every absolute path.

The numbers (measured 2026-06-14 on this machine):

| artifact | size | notes |
|---|---|---|
| `build-i386/` | 6.8 G | copied per worktree |
| `build-amd64/` | 7.6 G | copied per worktree |
| `sdk/lib/rust/target` | 35 G | Rust artifacts; nuked every clean smoke |
| ccache / sccache | — | **not installed, nothing wired** |

Both pains dissolve with one shared content-addressed cache: identical objects/rlibs get
reused (pain 1), **and** a fresh `cmake + ninja` in a new worktree becomes cheap on cache
hits, so the copy-and-repoint dance goes away (pain 2).

## Goal

1. Shared compiler cache so repeat/clean builds reuse work — across rebuilds *and across
   worktrees*.
2. Retire the `cp -a build-<arch>` + repoint ceremony (and its corruption hazard) in favor
   of "fresh `cmake + ninja`, fast on cache hits" — **if the measurements support it** (the
   link step is not cached; see Risks).
3. Move materially up the determinism ladder as a side-effect (the path-normalization the
   cache needs *is* a reproducibility property).

## Design

### One root knob — `VIBEOS_ROOT`

A single env var owns worktrees **and** caches. Default `~/.vibeos`; on this machine it is
`~/Projects/os-playground` (where the worktrees already live).

```
VIBEOS_ROOT=~/Projects/os-playground      # default ~/.vibeos
  ├── vibeos, vibeos-wt-fastfat, …         # worktrees
  ├── ccache/                              # CCACHE_DIR
  └── sccache/                             # SCCACHE_DIR   (local; or S3 later)
```

Derived:
- `CCACHE_DIR=$VIBEOS_ROOT/ccache`, `SCCACHE_DIR=$VIBEOS_ROOT/sccache`
- **`CCACHE_BASEDIR=$VIBEOS_ROOT`** — the cache basedir *is* the parent that contains all
  worktrees, which is exactly what makes a build in `vibeos-wt-fastfat` reuse objects from
  `vibeos`. The natural design is also the correct one.

### Two tools, split by language

- **C/C++ (the ReactOS bulk) → ccache.** Its `base_dir` + `hash_dir = false` rewrite
  absolute paths so builds in different worktrees hit the *same* entries — purpose-built for
  pain 2. Wire via `CMAKE_C_COMPILER_LAUNCHER` / `CMAKE_CXX_COMPILER_LAUNCHER = ccache`.
- **Rust (35 G, nuked constantly) → sccache.** ccache can't cache rustc; sccache can
  (`RUSTC_WRAPPER`). Given the nuke-every-clean-smoke workflow this is likely the **biggest
  single win**. Caveats: requires `CARGO_INCREMENTAL=0` (clean builds don't use incremental
  anyway); build scripts + proc-macros are not cached; `codegen-units=256` means many small
  cacheable units.

sccache *can* also do C/C++, but its cross-directory hits need hand-wired
`-ffile-prefix-map`, whereas ccache `base_dir` is automatic — so **ccache for C/C++ +
sccache for Rust** is the chosen split. (If distributed C/C++ ever matters, revisit: move
C/C++ to sccache too and accept the manual prefix-map — see Distributed.)

### The make-or-break detail: path normalization

Cross-worktree hits live or die here. Absolute paths leak into outputs via `-g` debug info,
`__FILE__`, and assertion strings — a different worktree path → cache **miss**.

- ccache: `CCACHE_BASEDIR=$VIBEOS_ROOT` + `hash_dir=false`. **Implemented** — this is the
  automatic, safe half and it gives cross-worktree C/C++ hits today.
- Rust/sccache: needs `--remap-path-prefix=$PWD=/vibeos` so each worktree normalizes its
  *own* root to the same `/vibeos` token. **DEFERRED — cannot go through the `RUSTFLAGS` env
  var here.** `sdk/lib/rust/.cargo/config.toml` carries load-bearing per-target `rustflags`
  (the i386 `-C target-feature=-sse,-sse2` kernel guard), and cargo uses env-flags **XOR**
  config-flags — they never merge. Setting `RUSTFLAGS` would silently *drop* the SSE guard
  and put SSE back in the i386 kernel (a boot-breaking miscompile). So cross-*worktree* Rust
  hits are a follow-up that must thread the remap **through `config.toml`** (per-target, so
  the existing flags survive), not via the env. Until then sccache still wins **same-worktree
  across the target-dir nuke** (pain 1), which is the bigger day-to-day cost. The remap is
  also a genuine reproducibility improvement (path-independent build), so it earns its place
  on the determinism ladder regardless.

### `vibeos-cli` owns the env (decision)

`vibeos-cli build` computes and exports `VIBEOS_ROOT`-derived `CCACHE_*` / `SCCACHE_*` /
`RUSTC_WRAPPER` / remap `RUSTFLAGS`, rather than asking every shell/agent to configure
itself. One owner → consistent across worktrees and agents, and the single place to flip
local↔distributed later. Gated behind an opt-in env/flag for the first landing so a cache
bug can't wedge everyone's build; flip default once hit rates look good.

## Determinism — the virtuous cycle

Caching does not *make* builds deterministic, but it **pays you to become deterministic** and
the work it forces is real reproducibility progress:

- **Forcing function.** Hit rate ∝ input determinism. Chasing cross-worktree hits forces
  path-stripping (remap-path-prefix, basedir) — and path-independence is a reproducibility
  property we want regardless.
- **Free nondeterminism detector.** Build the same commit twice → expect ~100% hits. Any
  miss = something varied. `ccache -s` / `sccache --show-stats` become a determinism gauge.
- **Safe by construction.** Content-addressed: same input → same cached output, always. It
  can never introduce nondeterminism or mask a regression — it *strengthens* the
  deterministic-clean-build guarantee while removing its cost (complements the smoke-nuke).

### The determinism ladder

| rung | buys | work |
|---|---|---|
| path-normalized compiles | cross-worktree cache hits | remap-path-prefix + basedir *(the cache work itself)* |
| `SOURCE_DATE_EPOCH` | kills `__DATE__`/`__TIME__` variance (clang honors it) | one env var + audit |
| deterministic link | byte-identical PE | lld `--no-insert-timestamp` / `/Brepro` (zero PE timestamp) |
| reproducible binary | identical bytes across machines/agents | all of the above together |

Rung 1 *is* the cache task; the rest are cheap and the cache motivates them. **vibeos-specific
payoff:** if any agent on a commit produces a byte-identical kernel, "this exact binary
booted" becomes provable cross-agent, and a byte diff is itself a real bug signal — a natural
future "reproducibility smoke" this infra sets up.

## Distributed / S3 (not now — future-proofing only)

Asked: can we write to S3 to exploit the content hashing, and how much lift to future-proof?

- **sccache is S3-native, ~zero code lift.** Pure env: `SCCACHE_BUCKET`, `SCCACHE_REGION` /
  `SCCACHE_ENDPOINT`, `SCCACHE_S3_KEY_PREFIX` + AWS creds (also GCS / Redis / Memcached /
  Azure / GHA). The same content-hash keys that work locally then read/write S3 instead.
- **ccache is not S3-native.** It has `remote_storage` for `http://` and `redis://` (+ a local
  secondary), so distributed ccache means standing up Redis or an S3-fronting HTTP cache —
  more lift than sccache's one-liner.

**Future-proofing = keep all cache config behind env that `vibeos-cli` computes** (which both
tools already consume). Then local→S3 is a 4-line change, never a rewrite. We stay local
single-machine for now.

## Measurements (2026-06-15, i386, this machine)

First real-tree run: a **fresh** `build-i386` cmake-configured in the build-cache worktree,
built with `vibeos-cli build --arch i386 --cache`.

- **Cache is codegen-safe — the critical result.** The Rust rlib built with sccache +
  `CARGO_INCREMENTAL=0` has byte-identical symbol references to main's normal build
  (`libevtlib_rust.a` undefined-refs are an exact match). sccache changes *nothing* about the
  output; it's a transparent wrapper that runs the real rustc on a miss.
- **Cold populate:** 6293 C/C++ compiles + 73 Rust crate compiles through the cache (0% hits,
  as expected on a cold tree).
- **Warm rebuild** (delete all objects + Rust targets, recompile): **ccache 73% direct / 86%
  incl. preprocessed** (4634/6341); **sccache 100% Rust hits** across the target nuke; 6341
  objects recompiled in **73 s** (vs ~5 min cold). The 73% ccache (vs ~100% steady-state) is
  depressed because the cold build was *truncated* by orthogonal fresh-worktree link failures,
  so 1707 objects got their first-ever compile during the warm pass — legit cold misses, not a
  cache deficiency.
- **Headline:** sccache makes the clean-smoke Rust target-dir nuke ~free (100% hits) — a direct
  hit on pain #1 (every clean smoke currently pays a full Rust rebuild).

**Fresh-worktree friction found (orthogonal to the cache, but it gates §5 "retire the copy").**
A fresh `cmake+ninja` in a new worktree is *not* yet a clean drop-in for `cp -a`, because the
fresh build hits issues the copied (already-built) tree never does:
1. **Parallel-build header-generation race** — `winnt.h` etc. are generated into
   `build-*/sdk/include/psdk/`; a `.rc` can compile before that step runs. Reproduces with
   plain ninja (no cache). Workaround: a header pre-pass, or just re-run ninja.
2. **Gitignored `.build-tools/llvm-mingw`** absent in fresh worktrees (cargo `build.rs` needs
   the i686 clang) → symlink it from main.
3. **C link failures** (`_RtlZeroMemory` &c., a fully-built-tree-only completeness issue)
   cascade into Rust `build.rs` steps that need those libs.
→ So the cache's proven win today is **rebuild + smoke-nuke acceleration inside an existing
build dir**, not (yet) retiring the copy. Keep the copy recipe.

## Disk-first reframing (2026-06-20) — GC, not dedup

User priority clarified: **(1) disk space, (2) the non-standard ntoskrnl/win32k build is
error-prone, (3) determinism is a strong nice-to-have that makes both simpler.** Speed is
*not* a priority (the Ryzen 9 crushes it — though it likely under-uses cores during the
serial-ish cargo-kernel link phase; parked).

Measured the real footprint (2026-06-20, this machine, 16 worktrees):

| thing | size | dedup-able? |
|---|---|---|
| C/C++ objects in `build-i386` | **0.3 G** (5881 `.obj`) | yes, but negligible → **hardlinks are a dud** |
| `build-i386` total | 6.2 G | no — bulk is *linked* `dll/` (2.2 G) + `drivers/` (1.5 G) + generated headers; differ per branch |
| Rust target (`sdk/lib/rust/target`) | **26 G** | no — cargo target isn't content-addressed; **the monster** |
| **Σ build+target across all 16 worktrees** | **~150 G** | — |

**Conclusion that flips the plan: the cache cannot dedup the big artifacts** (objects are
0.3 G; the 26 G Rust target + linked C outputs are per-branch and uncacheable as *files*).
So the disk win is **not** content-addressed file-sharing — it's **garbage-collecting
idle worktrees' build+target trees**, with the shared cache as the *enabler* that makes a
GC'd tree cheap to regenerate (sccache 100% Rust hits + ccache 73–86% C, already proven).

Reframed model:
- **Cache = the durable, shared, content-addressed, concurrent-safe store** (built).
- **Build trees = ephemeral scratch** — GC an idle lane's 26–42 G freely; the cache
  rehydrates on next build. Disk per idle worktree → ~0.
- **De-risks #2 for free:** a GC'd-then-regenerated tree can't be stale — it's rebuilt from
  the cache keyed on *current* source. Determinism makes that regen byte-identical and
  cross-worktree-shareable (the hit-rate multiplier + reproducible-binary proof).

**Hardlinks (`CCACHE_HARDLINK`) dropped** — measured 0.3 G of objects, not worth it.

### Can the *linked* outputs (dll/, drivers/) dedup across worktrees? — measured

Yes in principle (filesystem hardlinks; ext4 here = no reflinks but hardlinks work), but the
yield today is small and determinism-gated. `hardlink --dry-run` across the 5 worktrees that
have a `build-i386` (**31 G** combined): **saved 3.94 GiB (~13%)**, not the ~90% you'd expect
from "most binaries are untouched per lane."

**Why only 13%:** the binaries aren't byte-identical even when logically the same. Linked
outputs embed the **worktree's absolute path** (debug info `-g`/`__FILE__`/assert strings) and
a **PE build timestamp** — so the *same* `advapi32.dll` built in two worktrees differs in those
bytes → different content → won't dedup. These are exactly determinism-ladder rungs 2–3
(`SOURCE_DATE_EPOCH`, `--no-insert-timestamp`, `--remap-path-prefix`).

**Consequence — determinism moves onto the DISK critical path, not just #2.** It's the
multiplier that raises the cross-worktree dedup ceiling from ~13% toward the ~90%-identical
reality (same for the 26 G Rust target — rlibs embed paths too, fixed by the deferred
`--remap-path-prefix`-via-config.toml). So the sequencing is: GC first (big, determinism-
independent) → determinism rungs (cheap, unlock dedup + de-risk #2) → a `hardlink` dedup pass
last, when it's finally worth it. (`hardlink` caveat: shared inode → any in-place post-link
mutation corrupts all copies; safe only because builds write-fresh — gate to leaf artifacts.)

### Disk-first build order

1. **`vibeos-cli prune` (the disk win).** GC build-`<arch>`/ + Rust target dirs for idle
   worktrees. Safety gates (non-negotiable, cross-lane destructive): **dry-run by default**;
   never touch a worktree with a **live QEMU** (`qemu/ps` lock check) or a **running
   ninja/cargo**; LRU/idle heuristic (mtime); never the current worktree without `--self`.
   Reclaims ~110 G of the 150 G today.
2. **Cache default-on** (finish item (c) gate) so regeneration after a prune is cheap.
3. **Fix fresh-worktree friction** (header race, `llvm-mingw` symlink, link cascade — see
   Measurements §1–3) so a *fully* pruned worktree rebuilds clean → also retires `cp -a`.
4. **Determinism rungs** (`SOURCE_DATE_EPOCH`, `--remap-path-prefix` via config.toml,
   lld `--no-insert-timestamp`) → cross-worktree byte-identity + reproducible-kernel proof,
   which is the principled fix for #2.

## Staged rollout (measure first, promise later)

1. ✅ **Baseline.** ccache 4.5.1 + sccache 0.8.2 installed. (Cold-build/copy reference times
   still to be captured against a real `build-<arch>` — see §4/§5.)
2. ✅ **ccache for C/C++.** Wired via `-DENABLE_CCACHE` (→ `RULE_LAUNCH_COMPILE ccache`) +
   `CCACHE_BASEDIR` + `nohashdir`. Cross-dir hit **proven** on a synthetic TU. The
   build-twice hit-rate stage is live and doubles as the determinism reading.
3. ✅ **sccache for Rust.** `RUSTC_WRAPPER` + `CARGO_INCREMENTAL=0` wired; across-nuke hit
   **proven on the real tree: 100% Rust hits** (see Measurements). `--remap-path-prefix`
   deferred (config.toml clobber, see "path normalization") so cross-*worktree* Rust hits are
   not in yet — same-worktree-across-nuke is.
4. ✅ **Real-tree hit rates measured** (i386): ccache 73%/86%, sccache 100%, warm recompile 73 s.
   Cross-worktree ccache is proven *mechanically* (basedir cross-dir hit) but not yet measured
   on two full real trees — that's the remaining cross-worktree gap, lower priority now that
   the same-worktree + smoke-nuke win is demonstrated.
5. ✅ **Link/copy decision: KEEP the copy.** The fresh-worktree friction (Measurements §1–3)
   means a fresh `cmake+ninja` is not a clean drop-in for `cp -a` today — the copy delivers a
   fully-built tree, fresh-configure hits the header race + missing-lib cascade. The cache's
   demonstrated win is **rebuild + smoke-nuke acceleration**, not copy retirement. Revisit once
   the fresh-build issues are fixed (they're ReactOS-build bugs worth fixing independently).
6. ◑ **Default: opt-in + recommended for now.** `--cache` stays default-off (env/flag opt-in);
   proven safe + fast on i386. Flip default-on after amd64/arm64 validation (the cargo-driven
   amd64 kernel relink is the interaction to check). `max_size` capped at 50 G each.

## Host CPU broker (`hostlock.py`) — cross-worktree build/boot arbitration

Same lane, adjacent pain: the shared dev host has **one CPU pool** but many worktrees, and
agents were hand-arbitrating it in `#swarm` ("taking the host" / "host free") to avoid two
classes of corruption-and-false-flake:

1. **CPU starvation → false `HANG_TIMEOUT`.** A cold build (`ninja -j` over all cores) and a
   CPU-bound **TCG** boot (native arm64; or a windows-host boot while WHPX is shelved) run
   concurrently starve each other — the boot misses its marker and reads as a boot regression.
   The documented 2026-06-19 incident burned hours on exactly this.
2. **Same-build-dir corruption.** Two `build --clean` on the same `build-<arch>` clobber each
   other's CMake cache / cargo target (the determinism-verify-vs-gate hazard).

The per-arch QEMU slot cap in `vm.py` only covers *accelerated* same-arch boots; it's blind to
the CPU-bound class. `hostlock.py` closes that gap with two flock locks under a shared `/tmp`
dir (every worktree sees them; the kernel auto-releases on holder death — no stale cleanup):

- **`cpuheavy`** — a counting pool of `VIBEOS_HOST_CPU_LEASES` permits (default 4), used
  **writer-takes-all** so starvation is fixed *without* over-serializing boots:
  a cold **build** / `repro verify` grabs **ALL** permits (exclusive host); a **TCG boot** grabs
  **1** (so up to K TCG boots still parallelize — a ×3 gate keeps its boot parallelism — but a
  build excludes them all). KVM/WHPX boots are hardware-accelerated → take **none** and stay on
  the per-arch QEMU cap.
- **`builddir:<canonical-path>`** — exclusive per-build-dir lock (held under the cpuheavy grant),
  keyed by the resolved path so the same dir from any worktree maps to one lock.

Acquire order is always `cpuheavy → builddir`; a build takes the K permits **in order** (0,1,…)
so two builds can't deadlock. Timeouts **fail soft** (warn + proceed UNLOCKED, never wedge) —
the generous default ceiling (`VIBEOS_HOST_LOCK_TIMEOUT`, 1 h) means hitting it implies a stuck
holder, not real contention. `VIBEOS_HOST_BROKER=0` makes every lock a no-op.

**Wiring.** `build.py` runs the whole build under `hostlock.build_lock(build_dir, arch)`;
`vm.py` `start()` calls `_acquire_cpu_lease()` for a TCG boot (`_boot_is_tcg()`: native arm64
always, native x86 only if `/dev/kvm` absent, windows-host unless WHPX) before launch and
releases it in `_release_slot()`; `repro verify` inherits via its `build --clean` subprocesses.
`vibeos-cli host-status [--json]` shows who holds what (the `--json` feeds the wait-for-quiet
poll guards agents were hand-rolling).

**Status.** Primitive + host-status + writer-takes-all model + build/boot wiring: **landed on
`build-cache`** (`8dd7648a52a`, `be169da8e67`, `000f82c80fc`, `27f7caf389b`). 17 unit tests
(primitive + boot-lease seam) + a cross-process build-excludes-boot check pass; tooling gate
14/14. **Default-ON**, rolling out gradually (the lock-taking code only runs where the new
`vm.py`/`build.py` exists, so it's self-only until the mainline merge propagates — a built-in
safe rollout). Remaining: the real concurrent build-+-arm64-boot starve-test on a quiet host,
then the mainline ×3 gate.

## Risks & open items

- **Link is uncached.** May cap the "no more copying" win; #5 measures it before we promise.
- **WSL2 filesystem.** Keep caches on ext4 under `VIBEOS_ROOT` (never `/mnt/c`).
- **Disk.** Caches grow; cap `max_size` (start ~50 G each). Net win vs copying 7–15 G ×
  N worktrees, but watch the ceiling.
- **First build is cold** (populates cache); wins start on build #2 / other worktrees.
- **Determinism prereqs.** Generated inputs (widl, windmc, `gen_*.py` tables) must be stable
  — they are (committed generators). `__DATE__`/`__TIME__` (ReactOS version resources) and
  the PE-header timestamp are the known reproducibility leaks — rungs 2–3, deferred.
- **sccache Rust caveats.** Build scripts + proc-macros aren't cached; metadata hash includes
  paths (remap fixes it). Confirm hit rates on the actual workspace before celebrating.

## Decisions made (best-judgment, per request)

- `vibeos-cli build` is the single owner that computes/exports the cache env from
  `VIBEOS_ROOT`.
- Measure the link cost (#5) *before* declaring the copy retired — don't over-promise.
  **Measured 2026-06-15 → KEEP the copy** (fresh-worktree friction, not link cost, is the
  blocker; see Measurements).
- Both tools (ccache C/C++, sccache Rust); local now; env-driven so S3 is a later config flip.
- Use ReactOS's `-DENABLE_CCACHE` (global `RULE_LAUNCH_COMPILE ccache`) — verified it does NOT
  break `.rc`/resource compilation once generated headers exist (the earlier `.rc` failures
  were the header race, not ccache). `CMAKE_C/CXX_COMPILER_LAUNCHER` is the RC-only-safe
  alternative if a future RC issue appears.
- Keep `--cache` **opt-in** until amd64/arm64 are validated; i386 is proven safe + fast.

## Success metrics

- Warm 2nd build (no source change): **>90% C/C++ cache hit**, near-100% Rust hit.
- Fresh worktree build (warm shared cache) faster than `cp -a build-<arch>` end-to-end →
  copy recipe retired.
- Clean smoke (Rust target nuked) Rust rebuild time cut by the sccache hit rate.
- Build-twice hit-rate adopted as a standing determinism gauge.
