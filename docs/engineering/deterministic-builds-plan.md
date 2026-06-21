# Deterministic / Reproducible Builds — Plan + Research Notes

Status: **planning + living research doc** (started 2026-06-20, build-cache lane). This is the
home for everything reproducible-build: the plan, the catalog of nondeterminism sources, and
our accumulated Rust/C best-practice notes. Split out of `build-cache-and-determinism-plan.md`
(determinism graduated from a section to its own lane).

> **Living doc.** Add findings as we learn them — measured leaks, flags that worked/didn't,
> per-file gotchas. Exhaustive is fine (it's pull-context, zero standing cost). Keep the
> *push* channel (AGENTS.md/MEMORY.md) free of this.

## Why determinism (three payoffs, ranked by our priorities)

1. **Disk — the cross-worktree dedup multiplier (priority #1).** Measured 2026-06-20:
   hardlink-dedup across 5 `build-i386` trees saved only **3.94 G / ~13 %**, because
   "identical" binaries aren't byte-identical — they embed the worktree's absolute path
   (debug info, `__FILE__`) and a PE build timestamp. Strip those and the dedup ceiling rises
   from ~13 % toward the ~90 %-actually-unchanged reality. Determinism is the lever that makes
   the disk plan's dedup real. (See `build-cache-and-determinism-plan.md` → Disk-first reframing.)
2. **De-risk the non-standard ntoskrnl/win32k build (priority #2).** The cargo-driven kernel
   link doesn't go through the normal cmake DAG (field-manual §9.6b: `ninja livecd` doesn't
   relink the cargo kernel on a `.rs` edit), which is a perennial "did the right thing
   actually build / is this stale?" footgun. A content-addressed/deterministic build makes
   *stale* impossible by construction: the binary's identity **is** its input hash, so you
   physically cannot boot a kernel that doesn't match the source.
3. **Reproducible-kernel proof.** If two agents on the same commit produce a byte-identical
   `ntoskrnl.exe`, then "this exact binary booted" becomes provable cross-agent, and a byte
   diff is itself a real bug signal — a natural future "reproducibility smoke."

Determinism is also a **free nondeterminism detector**: build the same commit twice → expect
100 % cache hits / identical bytes; any miss = something varied. `ccache -s` / `sccache
--show-stats` and a build-twice-diff become standing gauges.

## The determinism ladder

| rung | buys | mechanism |
|---|---|---|
| 0. measurement harness ✅ | know what differs today | `vibeos-cli repro` (built-in PE parser; `diffoscope` optional) |
| 1. path normalization | cross-worktree byte-identity + dedup | C: `-ffile-prefix-map` ✅ (in source); Rust: `--remap-path-prefix` via cargo `--config target.'cfg(all())'` ✅ (stable; joins, doesn't clobber the SSE guard) |
| 2. timestamp elimination | kills `__DATE__`/`__TIME__` + PE-header time variance | `SOURCE_DATE_EPOCH`; lld no-timestamp; deterministic `ar` |
| 3. build-script / generator audit | the cargo-link + widl/windmc/gen tables are path/time/order-free | audit the ~48 `*-cargo/build.rs` + generators |
| 4. build-id / debug-GUID determinism | identical debug directory | content-hash (not random) build-id |
| 5. reproducibility smoke ✅ (gauge built) | identical bytes across builds/agents, wired as a gauge | `vibeos-cli repro verify --arch X` (cold build-twice → byte-identical assertion); cross-worktree run pending |

Rung 1 is the highest-leverage (it's the dedup unlock *and* most of the disk win). Rungs 2–4
are cheap and mop up the residual; rung 5 is the payoff gauge.

### Progress (2026-06-20, build-cache lane)

- **Rung 0 ✅ landed** (`f3e4abe9640`): `vibeos-cli repro` harness + validated baseline.
- **Rung 1 (Rust) ✅ IMPLEMENTED on stable** (2026-06-21 — the "toolchain-blocked" wall was
  surmountable after all). `trim-paths` IS unstable on cargo 1.92 (needs nightly + a manifest
  `cargo-features` opt-in, which poisons every *stable* consumer of the shared workspace manifest —
  that's why the manifest change broke the build and was reverted). But `--remap-path-prefix`
  delivered via **cargo `--config "target.'cfg(all())'.rustflags=[…]"`** does the same job on
  stable, and the earlier "stable fallbacks don't fit" verdict was wrong on two counts:
  - **The per-worktree-path problem dissolves**: cmake already knows `${REACTOS_SOURCE_DIR}` per
    worktree at configure time (it's what the C `-ffile-prefix-map` uses). So we `configure_file` a
    per-worktree `det-remap.toml` and pass it via `--config`. Each worktree maps its OWN root to a
    constant `/vibeos` token.
  - **The SSE-guard clobber doesn't happen**: env `RUSTFLAGS` and `[target.<triple>].rustflags`
    clobber, but `target.'cfg(all())'.rustflags` is **JOINED** with the per-triple rustflags
    (cargo combines target.<cfg> + target.<triple>). **Verified empirically** (both inline `--config
    key=val` and file `--config <path>` forms): a build carrying the i386 SSE guard
    `-C target-feature=-sse,-sse2` AND the injected `--remap-path-prefix` shows BOTH on the rustc
    cmdline. SSE guard intact.
  Wired in `sdk/cmake/gcc.cmake:add_cargo_module` (the single cargo invocation funneling all KINDs
  — kernel exe, win32k.sys, ntdll, every cargo DLL/driver), so all 40 cargo modules ×3 arches get
  it. **Verified end-to-end:** real i386 `beep.sys` builds with the remap + SSE guard on the rustc
  cmdline; and at the binary level a panic-bearing crate's embedded `…/<worktree>/src/lib.rs`
  becomes `/vibeos/src/lib.rs` (worktree path gone). std/registry paths (`~/.rustup`, `~/.cargo`)
  are home-relative = identical across worktrees on a machine, so remapping just the source root is
  sufficient for the cross-worktree dedup goal.
  - **Verified on ALL 3 arches** (2026-06-21): the `cfg(all())` join preserves the per-target
    rustflags AND `--remap-path-prefix` rewrites the embedded source path `<worktree>/src/lib.rs`
    → `/vibeos/src/lib.rs` on **i686-pc-windows-gnu** (sandbox + real `beep.sys` via the custom
    `i686-kernel` target, SSE guard intact), **x86_64-pc-windows-gnu**, and
    **aarch64-pc-windows-gnullvm**. Injection is the single `add_cargo_module` cargo cmd shared by
    all three kernel stems → all 40 cargo modules ×3 get it. The clobber risk was i386-only (the
    SSE guard); amd64/arm64 carry less and still verified.
  - **Remaining:** the full cross-worktree *kernel* build-twice byte-identity on
    ntoskrnl/win32k/kernel32 (`repro verify` / `diff-dirs`) — deferred until build slots free.
- **Rung 1 (C) ✅ already correct in source** — no change needed. `CMakeLists.txt:97` maps
  `-ffile-prefix-map=${REACTOS_SOURCE_DIR}=`, which a *fresh per-worktree configure* expands to
  that worktree's own abs path → empty; main's generated `build.ninja` only *looks* hardcoded
  because it shows main's expansion. The plan's earlier "hardcoded to main" was reading main's
  generated ninja, not the source. Residual `DW_AT_comp_dir` (the abs *build* dir, which neither
  prefix-map covers) is a possible remaining C leak → measure with `repro --diffoscope` post-
  timestamp-fix; if it shows, add `-ffile-compilation-dir=.` (clang 19 supports it). Deferred
  until measured (it touches every C TU → don't widen blast radius on spec).
- **Rung 2a (PE timestamp) ✅ implemented, gate-pending**: `-Wl,--no-insert-timestamp` added in
  BOTH link paths — the 679 cmake DLL/driver/exe links (`toolchain-llvm-mingw.cmake`
  `*_LINKER_FLAGS_INIT`) and every cargo-driven link (`reactos-link` common PE-flag block, one
  place covering all 42 `*-cargo` modules incl. ntoskrnl.exe/win32k.sys). This kills the only
  leak measured live on all 3 arches. Verify post-build: `repro report` → `0 (deterministic)`.
- **Rung 2b (SOURCE_DATE_EPOCH) ✅ implemented, gate-pending**: `vibeos-cli build` now derives
  `SOURCE_DATE_EPOCH` from the HEAD commit date (`git log -1 --format=%ct`) and sets it in the
  build env (`build.py:source_date_epoch` + `os.environ.setdefault`), so every subprocess (cmake
  configure → version.cmake `string(TIMESTAMP)`, ninja → clang `__DATE__`/`__TIME__` in
  xipdisp.c) inherits it. Commit-date (not wall-clock) → byte-identical across worktrees on the
  same commit. `setdefault` lets a CI-pinned value win. No version.cmake source edit needed (the
  env var pins it). Unit-tested (commit-`%ct` derivation + non-git → None).
- **Rung 3 (build.rs/generator audit) ✅ done**: clean — see the date/time audit section.
- **Rung 5 ✅ gauge built** (2026-06-21): `vibeos-cli repro verify --arch X` cold-builds the
  arch twice and asserts the artifact set is byte-identical (`repro.py:_cmd_verify`). It deletes
  the cargo output exes between builds to force a genuine relink (defeats the §stale-cargo /
  task-#41 trap where `--clean` leaves the copied exe newer than its `.rs` inputs → ninja skips →
  false "identical"). Unit-tested at the parser/dispatch level; the actual cold build-twice RUN
  (and the stronger cross-worktree `diff-dirs` proof that validates the rung-1 Rust remap end-to-
  end) is **pending** until build slots are free (mainline just recovered; lanes are mid-gate).
- **Pending ×3 cold gate** for the rung-1/2/2b build changes (can't ride a held gate; arm64
  boots rebuilt + restored but the slot is contended by other lanes as of 2026-06-20). Determinism
  *effect* (timestamps→0, date pinned) is verified separately with `vibeos-cli repro report` after
  a clean build — the smoke gate only proves no boot regression. Rungs 4–5 still TODO.

## Sources of nondeterminism — the catalog (research notes)

### Cross-cutting
- **Embedded timestamps** — `__DATE__`/`__TIME__`/`__TIMESTAMP__` macros, archive member
  mtimes, PE/ELF header timestamps. Fix: `SOURCE_DATE_EPOCH` (clang honors it for the date
  macros), deterministic archiver, linker timestamp zeroing.
- **Embedded absolute paths** — debug info (CodeView/DWARF `DW_AT_comp_dir`), `__FILE__`,
  assertion strings, Rust panic messages / `file!()`. Fix: path-prefix remapping (below).
- **Ordering** — directory-read order in globs, `HashMap` iteration (random seed), parallel
  writes to symbol/string tables. Fix: sort inputs; `BTreeMap`/sorted iteration in generators.
- **Environment leakage** — `LANG`/`LC_*`/`TZ` affecting sort/format; env vars copied into
  outputs. Fix: pin `LC_ALL=C`, `TZ=UTC` for the build; don't embed env.
- **Toolchain version** — different clang/rustc → different codegen. We pin llvm-mingw clang 19
  + rustc (rust-toolchain). Record the toolchain hash alongside any reproducibility claim.

### C / clang (the ReactOS bulk)
- **`-ffile-prefix-map=OLD=NEW`** (clang ≥10) — one flag = `-fmacro-prefix-map` (`__FILE__`) +
  `-fdebug-prefix-map` (DWARF). Inject via CMake using the per-worktree source dir, so each
  worktree maps *its own* root to the same token:
  `-ffile-prefix-map=${CMAKE_SOURCE_DIR}=/vibeos`. CMake knows its own source dir at configure
  time → the mapping is automatically correct per worktree (no `$PWD` interpolation problem).
- **`-fdebug-compilation-dir=.`** / **`-ffile-compilation-dir=.`** — normalize the recorded
  compilation directory.
- **`SOURCE_DATE_EPOCH`** — clang sets `__DATE__`/`__TIME__`/`__TIMESTAMP__` from it.
- **`-Wdate-time`** — clang *warning* that flags every `__DATE__`/`__TIME__`/`__TIMESTAMP__`
  use — the audit tool to FIND the leaks (the ReactOS version resources are a known offender).
- **Deterministic archives** — `llvm-ar` deterministic (`D`) mode (zeroes member mtime/uid/gid);
  verify import libs from `llvm-dlltool` are stable too.

### Rust / rustc
- **`trim-paths` (THE clean path remap)** — Cargo profile setting, stabilized 1.81; we're on
  rustc **1.92**, so it's available. `[profile.release] trim-paths = "object"` (or `"all"`)
  auto-remaps the workspace + registry + std paths embedded in objects, **per build with the
  real absolute path** — so it sidesteps both problems we hit:
  - the per-worktree `$PWD` problem (config.toml can't interpolate env, but `trim-paths`
    computes the remap itself), and
  - the **rustflags XOR clobber** (see Landmines): `trim-paths` is a *profile* option, not
    rustflags, so it does NOT touch the load-bearing `[target.*].rustflags` SSE guard.
  → **Recommended Rust approach.** Fallback if a flag is needed instead: add the remap via a
  `[target.'cfg(all())'].rustflags` entry (cargo *joins* `target.<cfg>` with `target.<triple>`
  rustflags, so it adds without clobbering the per-triple SSE guard) — but `trim-paths` is
  cleaner; prefer it.
- **`CARGO_INCREMENTAL=0`** — incremental compilation embeds paths and is nondeterministic.
  Already set by the build-cache (cache mode); keep it for any reproducibility build.
- **Build scripts / proc-macros** — NOT cached by sccache, and they *are* our cargo-driven
  link (~48 `*-cargo/build.rs`). Prime nondeterminism risk: reading `SystemTime`/env/abs-paths,
  `HashMap` iteration order, emitting absolute `-L`/`-l` paths into the link. Audit (rung 3).
- **`-Cmetadata` / symbol-name hash** — includes crate version + path; `trim-paths` stabilizes
  the path component.

### PE-COFF (our output format)
- **PE header `TimeDateStamp`** — ✅ RESOLVED 2026-06-20. We use the **gnu `ld.lld` driver**
  everywhere (`-fuse-ld=lld` + gnu-style `-Wl,…` flags on all 679 cmake DLL/driver links; the
  cargo `*-kernel` targets pin `*-w64-mingw32-clang` → same gnu driver). `ld.lld` **defaults to
  `--insert-timestamp`** (confirmed via `-Wl,--help`) — that is the wall-clock source measured.
  **Fix = `-Wl,--no-insert-timestamp`** (NOT MSVC `/Brepro`, which is the `lld-link` driver we
  don't use). Apply in two places: the cmake link flags (679 links carry none today) and the
  cargo links via `-Clink-arg=-Wl,--no-insert-timestamp` (ntoskrnl-cargo/build.rs etc. pass no
  timestamp flag today).
- **Debug directory / CodeView PDB GUID + Age** — can be content-derived (deterministic) or
  random; verify. Tie to a content hash if random.
- **`--build-id`** — use a content-based id (`sha1`) or `none`; never random/`uuid`.
- **Export/import table ordering** — generally deterministic; watch `.def`/spec2def ordering.

### Build system (CMake / Ninja)
- CMake bakes absolute paths into `compile_commands.json`, generated config headers, and
  `RULE_LAUNCH_*`. Only what lands *in the binary* matters for byte-identity (the compile
  flags above handle that); the ccache `basedir` already handles cache keys.
- Generated inputs — `widl`, `windmc`, the `gen_*.py` NLS/data tables — must be stable. They
  are (committed deterministic generators), but any new generator must sort + avoid time/paths.

## Date/time-macro audit (rung 2b — completed 2026-06-20)

Swept the tree for `__DATE__`/`__TIME__`/`__TIMESTAMP__` + cmake `string(TIMESTAMP)` +
build.rs `SystemTime`/`Instant`/`rand` + generator time/unsorted-iteration. Findings:

- **`sdk/include/reactos/version.cmake:4`** — `string(TIMESTAMP KERNEL_VERSION_BUILD %Y%m%d UTC)`
  bakes the *configure date* into `buildno.h` → every module's version resource (`.rsrc`). The
  PE-timestamp fix (rung 2a) does NOT cover this (it's section content, not the header). **CMake's
  `string(TIMESTAMP)` honors `SOURCE_DATE_EPOCH`** (≥3.8) → setting it pins this automatically.
  `COMMIT_HASH` (git rev-parse) + `REVISION` (git describe) in the same file are deterministic
  per commit already — not leaks.
- **`ntoskrnl/ex/xipdisp.c:78-79`** — `__DATE__`/`__TIME__` compiled **into the kernel**. clang
  honors `SOURCE_DATE_EPOCH` for these macros → same lever closes it.
- Off-boot-path hits (not in core artifacts; lower priority): `drivers/network/dd/netkvm`,
  `drivers/bus/acpi/acpica` apps, `modules/rosapps/*`, and the host tools `sdk/tools/wpp` +
  `dll/directx/wine/d3dcompiler_43` (these *define* `__DATE__` for files they preprocess —
  host-tool determinism, separate concern). `SOURCE_DATE_EPOCH` in the build env covers the
  clang-compiled ones uniformly.
- **build.rs: clean** — no `SystemTime`/`Instant::now`/`rand` in any `*-cargo/build.rs` or
  `reactos-link`. **Generators: effectively clean** — dict-ordered + sorted; one `set()` in
  `tools/gen_global_barrier.py:25` is a membership `seen` set (verify it doesn't drive output
  order; low risk).

**Net rung-2b fix = one lever:** export `SOURCE_DATE_EPOCH` (commit-date-derived,
`git log -1 --format=%ct`) for cmake configure **and** the compile env. Commit-date (not
wall-clock-0) makes it byte-identical across worktrees/agents on the same commit *and* keeps a
meaningful date. Must be present at BOTH configure (version.cmake) and compile (xipdisp.c
`__DATE__`); the natural home is the `vibeos-cli build` front door's subprocess env. Gate-pending
(build change) — bundle with the rung-1/2 increment.

## vibeos-specific landmines

1. **The `[target.*].rustflags` XOR clobber.** `sdk/lib/rust/.cargo/config.toml` carries
   load-bearing per-target rustflags — the i386 kernel SSE guard
   `-C target-feature=-sse,-sse2` (on `[target.i686-pc-windows-gnu]` and `[target.i686-kernel]`).
   Cargo does **not** merge `RUSTFLAGS` env or `[build].rustflags` with `[target.<triple>]`
   rustflags — the first matching source wins. So **setting `RUSTFLAGS` would silently drop the
   SSE guard → SSE back in the i386 kernel → boot-breaking miscompile.** This is exactly why
   the cache lane deferred env-based remap. `trim-paths` (profile setting) avoids the whole
   trap — use it.
2. **The non-standard cargo-driven kernel link.** ~48 `*-cargo/build.rs` (ntoskrnl-cargo,
   win32k-cargo, ntkrnlmp-cargo, hal-cargo, …) drive the link outside the cmake DAG. These are
   the most likely determinism breakers (they run linkers, compute paths) AND the least
   covered. Rung 3 audits them.
3. **ReactOS version resources** embed build date/revision (`__DATE__`/`__TIME__` + a buildno)
   — a real, known leak. Needs `SOURCE_DATE_EPOCH` + possibly a pinned/derived buildno.
4. **Three arches, GNU/LLVM targets**, PE-COFF — every fix must hold on i386 + amd64 + arm64
   (and the i386 SSE guard must survive, see #1).

## Measured baseline (2026-06-20, against main's build dirs)

First real inspection — what's already done vs. what leaks today:

- **PE `TimeDateStamp` is a LIVE LEAK on all 3 arches.** Built `ntoskrnl.exe` carries the
  wall-clock build time (i386 `0x6a36a56e` = 14:36:30, amd64 14:39:48, arm64 15:14:20). Two
  worktrees building the same commit get *different* stamps → instant byte-difference → no
  dedup. The link has **no** `--no-insert-timestamp` / `/Brepro` today. → rung 2 confirmed.
- **C path-remap is PARTIALLY DONE already.** `build-i386/build.ninja` compiles carry
  `-ffile-prefix-map=../=` **and** `-ffile-prefix-map=/home/curi/Projects/os-playground/vibeos=`.
  The relative `../=` map is the cross-worktree-safe one (the relative path from `build-<arch>/`
  to a source is identical in every worktree). BUT the absolute entry is **hardcoded to main's
  path** — in a worktree (`…/vibeos-wt-buildcache`) it only prefix-matches and leaves a
  `-wt-buildcache/…` residual, so any *absolute* source ref still diverges per worktree. → rung
  1 (C) is mostly there; the fix is to make the abs entry `${CMAKE_SOURCE_DIR}`-derived (or
  confirm every ref goes through `../`). **Verify** with a two-path build diff.
- **Archives already deterministic** — `llvm-ar` defaults to `[D]` (zero timestamps/uids/gids).
  Open question resolved: no archive work needed.
- **Rust: no remap yet** — no `--remap-path-prefix`/`trim-paths` in the build; rung 1 (Rust) is
  the clean `trim-paths` add (above), still TODO.

Net: the *biggest* live leak is the **PE timestamp** (rung 2, all arches) + **Rust paths**
(rung 1 via `trim-paths`); the **C paths are largely handled** (verify the worktree abs-map
residual). That's a smaller surface than feared.

## Stale-cargo-binary elimination (the #2 payoff — root-caused 2026-06-20)

The non-standard cargo-driven kernel link can serve a **stale binary** — the concrete form of
the #2 footgun. Two distinct gaps (NOT one):

- **§9.6b "doesn't relink on a `.rs` edit" (staticlib only).** `ntoskrnl/CMakeLists.txt:78`
  builds the kernel-rust staticlib with `add_custom_command(OUTPUT ${RUST_KERNEL_LIB} … DEPENDS
  ${WS}/kernel-rust/src/lib.rs)` — ninja's *only* declared input is `lib.rs`, so editing any
  other `.rs` leaves ninja thinking it's current. (`add_cargo_module` in `gcc.cmake:608` is
  fine — it `file(GLOB_RECURSE _srcs CONFIGURE_DEPENDS)` + `DEPENDS ${_srcs}`, so the *exe*
  relinks on a `.rs` edit.) **Fix:** widen the staticlib `DEPENDS` to the same recursive glob.
- **"wiped target → stale exe" (the `rm -f output` trap).** cargo outputs are *copied* out of
  the target dir, so nuking the target alone leaves the copied exe present + newer than the
  `.rs` inputs → ninja "no work to do" → stale. And `vibeos-cli build --clean` (`build.py:_clean`)
  nukes target dirs + `lib*_rust.a` but **not the cargo-module output exes** — so `--clean` is
  not guaranteed clean for cargo modules (it relies on the deleted-`.a`→dep-chain to force a
  relink, which is fragile). **Fix:** `_clean` also removes the cargo-module output binaries
  (ntoskrnl.exe, ntkrnlmp, win32k.sys, kernel32.dll, the `*-cargo` outputs) so ninja *must*
  relink — automating meaniepy's manual `rm -f` and making `--clean` genuinely cold.

Chosen over a blanket "always-run cargo" (corrosion-style): there are ~48 `*-cargo` modules, so
always-invoking cargo would add 48 no-op cargo checks per `ninja` — too costly. The two targeted
fixes close both gaps without that overhead. Both are build-path changes → own ×3 cold gate,
**queued as the next increment after the build-cache bundle lands** (can't ride the locked gate).

## Verification methodology

- **`vibeos-cli repro`** — the rung-0 harness (built 2026-06-20, `tests/qemu/cli/repro.py`).
  Self-contained PE parser (no `pefile`/`diffoscope` dependency), so it runs anywhere and is
  unit-tested in `test_cli_units.py` (5 tests). Three verbs:
  - `repro report <pe>…` — dumps the COFF `TimeDateStamp` (flags `← LEAK` if non-zero) +
    sha256 + section count. The baseline-capture command.
  - `repro compare <a> <b>` — byte-identity; on a mismatch, **localizes**: does the
    TimeDateStamp diverge, and *which sections* differ (so a leak is pinpointed, not just
    "bytes differ"). `--diffoscope` shells out to diffoscope if installed for byte-level depth.
  - `repro diff-dirs <buildA> <buildB> [--artifacts …]` — runs `compare` over the staged
    artifact set (ntoskrnl.exe, win32k.sys, kernel32.dll by default) across two build trees:
    the build-twice-in-two-paths verification + dedup-yield measurement in one command.
- **Build-twice-in-two-paths.** Build the same commit in two different worktree paths (or the
  same path twice after a clean), then `repro diff-dirs`. Identical = reproducible.
- **`diffoscope`** — optional deep localizer (unpacks PE, disassembles). NOT installed;
  `pip install diffoscope` enables `repro --diffoscope`. The built-in parser covers the
  timestamp + per-section localization without it.
- **Cache hit-rate as gauge** — a build-twice with <100 % hits points at a varying input.
- **Reproducibility smoke (rung 5)** — wire `repro diff-dirs` into a harness check; a non-repro
  binary turns it red, like any North-Star regression.

### ✅ REAL-KERNEL build-twice byte-identity confirmed (i386, 2026-06-20)

Two independent `vibeos-cli build --arch i386 --clean` runs in the **same** worktree path
(outputs removed between to force genuine relinks — sidesteps the §41 stale-output trap),
with rung-2a (`--no-insert-timestamp`) + rung-2b (`SOURCE_DATE_EPOCH`) active:

- `repro report` build #1: ntoskrnl.exe / win32k.sys / kernel32.dll all **`TimeDateStamp 0
  (deterministic)`** (was `0x6a37…` wall-clock pre-fix).
- `repro compare` build #1 vs build #2: **`✓ identical`** on all three (kernel + cargo DLL +
  cmake-C-link DLL). Fresh relink timestamps confirm they were genuinely rebuilt, not stale.

So the build is reproducible at a fixed path. **Caveat:** this does NOT yet prove *cross-worktree*
byte-identity (the dedup payoff) — Rust objects still embed the absolute worktree path, which is
identical in a same-path rebuild but differs across worktrees. That last gap needs rung-1
(`trim-paths`), toolchain-blocked on cargo 1.92. Net: timestamp + date determinism is **done and
proven**; cross-path remap is the one remaining (deferred) piece for full dedup.

### Mechanism proof + rung-4 resolution (build-twice on the real toolchain, 2026-06-20)

Verified the fix end-to-end with `repro` + a minimal DLL through the actual llvm-mingw
`x86_64-w64-mingw32-clang` (`-shared -nostdlib -fuse-ld=lld`):

- **`--no-insert-timestamp` works**: TimeDateStamp `0x6a37394e` (wall-clock) → `0 (deterministic)`.
- **build-twice (same output name, pinned timestamp) = BYTE-IDENTICAL** ✓. The whole approach
  produces reproducible bytes on our toolchain.
- **Rung 4 (build-id) RESOLVED — no flag needed.** lld's default `.buildid` is *content-derived*;
  it only diverged across builds because the build-id hash includes the PE timestamp. Once
  `--no-insert-timestamp` pins the timestamp, a same-name build-twice with the **default** build-id
  is byte-identical. So `--no-insert-timestamp` is load-bearing twice over (header *and*
  transitively the build-id). `--build-id=none` is unnecessary.
- **Which artifacts carry `.buildid`**: only the **cargo/rust-linked** modules (kernel32.dll,
  cdfs.sys — recognizable by the rustc `/4 /18 …` COMDAT sections). Pure-cmake-C links (win32k.sys,
  ntdll.dll) and the kernel (ntoskrnl.exe, all 3 arches) carry none. Either way, rung 2a covers it.
- **Filename caveat (test-only)**: different *output names* diverge in `.rdata` regardless —
  `--enable-auto-image-base` hashes the output name into the image base, and the export directory
  embeds the DLL name. Moot for real artifacts (each always links to the same name at a fixed
  base; the kernel uses `--image-base 0x400000`). Don't mistake this for a real leak when
  build-twice'ing into differently-named files — build into the *same* name (copy between).

**Net: rungs 1 + 2a + 2b are sufficient for byte-identity on real artifacts** (same name/path,
which the build always uses). Remaining: a real-kernel build-twice (via `repro diff-dirs`) to
confirm on ntoskrnl/win32k/kernel32, gated alongside the ×3 cold smoke.

### Rung-0 baseline confirmed by the harness (2026-06-20)

`repro report` on the three built kernels reproduces the measured PE-timestamp leak exactly —
each arch carries a *different wall-clock* TimeDateStamp (i386 00:50:17Z, amd64 00:41:39Z,
arm64 00:49:56Z on a 2026-06-21 build), so two worktrees building the same commit get
different bytes → 0 % dedup on the kernel. This is the rung-2 target, now machine-checkable:
after the `--no-insert-timestamp` fix, `repro report` must show `0 (deterministic)` on all 3.

## Staged plan

0. **Measurement harness** — install `diffoscope`; script `build twice in two paths → sha256
   → diffoscope on mismatch`. Capture the current baseline (what differs today, per arch).
   *Acceptance:* a report listing every nondeterministic section/source for ntoskrnl + a couple
   of DLLs, per arch.
1. **Path normalization** — C: add `-ffile-prefix-map=${CMAKE_SOURCE_DIR}=/vibeos`
   (+ compilation-dir) in the toolchain/cmake flags. Rust: `[profile.release] trim-paths`.
   *Acceptance:* two-path build of a sample TU/crate is byte-identical in the debug/path bytes;
   hardlink-dedup yield jumps measurably above the 13 % baseline. **i386 SSE guard intact**
   (smoke i386 still boots).
2. **Timestamp elimination** — the live leak. (a) Add **`-Wl,--no-insert-timestamp`** to the
   cmake link flags (679 links carry none today) **and** `-Clink-arg=-Wl,--no-insert-timestamp`
   to the cargo links → PE `TimeDateStamp` → 0. (b) Set `SOURCE_DATE_EPOCH`; audit `__DATE__`/
   `__TIME__` with `-Wdate-time`; fix the version-resource leak. (Archives already deterministic.)
   *Acceptance:* PE `TimeDateStamp` reads 0/constant across builds on all 3 arches;
   `-Wdate-time` clean (or remaining uses SOURCE_DATE_EPOCH-fed).
3. **build.rs / generator audit** — sweep the ~48 `*-cargo/build.rs` + widl/windmc/gen_*.py for
   `SystemTime`/env/abs-path/`HashMap`-order. *Acceptance:* none emit time/unmapped-abs-path/
   unsorted output.
4. **build-id / debug-GUID determinism** — pin content-based build-id; verify debug directory
   is stable.
5. **Reproducibility smoke** — wire build-twice-diff as a gauge; **stretch goal: a byte-
   identical `ntoskrnl.exe` across two worktrees on the same commit**, all 3 arches.

## Open questions / to verify

- ✅ **Archives deterministic?** YES — `llvm-ar` defaults to `[D]`. Resolved 2026-06-20.
- ✅ **Which lld driver / where's the wall-clock stamp?** gnu `ld.lld` everywhere
  (`-fuse-ld=lld`), which defaults to `--insert-timestamp` → fix is `-Wl,--no-insert-timestamp`
  (cmake links + cargo `-Clink-arg`). Not `/Brepro`. Resolved 2026-06-20.
- The **C abs-path `-ffile-prefix-map` is hardcoded to main's path** — does a worktree build
  still byte-match main (i.e. does everything route through the relative `../=` map, leaving no
  absolute residual)? Verify with a two-path diff; if residual, switch the abs entry to
  `${CMAKE_SOURCE_DIR}`.
- ✅ `trim-paths = "object"` vs `"all"` — picked `"object"` (the byte-identity requirement;
  `"all"` also cleans diagnostics, not needed). Resolved 2026-06-20.
- ✅ **CodeView/build-id content-derived or random?** Content-derived — becomes deterministic
  once the PE timestamp is pinned (no `--build-id` flag needed). Resolved 2026-06-20 (build-twice).
- ✅ **Buildno strategy?** Derive from `SOURCE_DATE_EPOCH` = HEAD commit date (not 0 — keeps a
  meaningful, per-commit-stable date). Implemented in `vibeos-cli build`. Resolved 2026-06-20.
- ✅ **Where's the wall-clock PE stamp from?** The gnu `ld.lld` driver default `--insert-timestamp`
  (confirmed by build-twice: TimeDateStamp = wall-clock without the flag, 0 with). Resolved.

## References

- reproducible-builds.org — the canonical catalog of nondeterminism sources + fixes.
- `SOURCE_DATE_EPOCH` spec (reproducible-builds.org/specs/source-date-epoch).
- Cargo `trim-paths` (The Cargo Book → profiles) — stabilized 1.81.
- clang `-ffile-prefix-map` / `-Wdate-time` (clang docs).
- Sibling: `build-cache-and-determinism-plan.md` (the cache + disk-GC lane this split from).
