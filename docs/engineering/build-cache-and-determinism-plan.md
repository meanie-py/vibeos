# Build Cache & Determinism Plan

Status: **planning** (brainstormed 2026-06-14; lane = `build-cache` branch / `vibeos-wt-buildcache`).
Owner: build-cache lane. Prereq for nothing; unblocks faster iteration for every lane.

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

- ccache: `CCACHE_BASEDIR=$VIBEOS_ROOT` + `hash_dir=false`.
- Rust/sccache: `RUSTFLAGS=--remap-path-prefix=$PWD=/vibeos` — each worktree normalizes its
  *own* root to the same `/vibeos` token, so all worktrees emit path-identical rlibs.
  **Without this, every worktree is a 100% Rust cache miss** — it is the whole ballgame for
  the Rust win, and it is also a genuine reproducibility improvement (path-independent build).

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

## Staged rollout (measure first, promise later)

1. **Baseline.** Install ccache + sccache. Record a cold i386 build time + the current
   per-worktree copy time as the reference to beat.
2. **ccache for C/C++.** Wire `CMAKE_<LANG>_COMPILER_LAUNCHER` (opt-in via env) +
   `CCACHE_BASEDIR` + `hash_dir=false`. **Build-twice probe:** the 2nd build's hit-rate is
   simultaneously the speedup number *and* our first determinism reading.
3. **sccache for Rust.** `RUSTC_WRAPPER` + `CARGO_INCREMENTAL=0` + remap-path-prefix. Probe
   across a target nuke (the smoke workflow) *and* across two worktrees.
4. **Cross-worktree validation.** Build in main, then a *fresh* `cmake+ninja` in a worktree;
   measure C/C++ + Rust hit rates. This is the gate for pain 2.
5. **Measure the link cost.** Compiles cache; the final ~7 G link does **not**. Time
   "warm-cache fresh worktree build" end-to-end vs the `cp -a` copy. Only if the former wins
   do we rewrite the AGENTS.md copy recipe to "fresh `cmake+ninja`, cache makes it cheap"
   (keep copy as a documented fallback).
6. **Flip default + document.** Default the cache on, set `max_size` caps, update AGENTS.md
   (Worktree Discipline + Build Commands) and `tests/qemu/cli/README.md`.

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
- Both tools (ccache C/C++, sccache Rust); local now; env-driven so S3 is a later config flip.

## Success metrics

- Warm 2nd build (no source change): **>90% C/C++ cache hit**, near-100% Rust hit.
- Fresh worktree build (warm shared cache) faster than `cp -a build-<arch>` end-to-end →
  copy recipe retired.
- Clean smoke (Rust target nuked) Rust rebuild time cut by the sccache hit rate.
- Build-twice hit-rate adopted as a standing determinism gauge.
