# Deterministic / Reproducible Builds — Plan + Research Notes

Status: **planning + living research doc** (started 2026-06-20, build-cache lane). This is the
home for everything reproducible-build: the plan, the catalog of nondeterminism sources, and
our accumulated Rust/C best-practice notes. Split out of `build-cache-and-determinism-plan.md`
(determinism graduated from a section to its own lane).

> **Living doc.** Add findings as we learn them — measured leaks, flags that worked/didn't,
> per-file gotchas. Exhaustive is fine (it's pull-context, zero standing cost). Keep the
> *push* channel (AGENTS.md/MEMORY.md) free of this.

## ⚠️ Strategy pivot (2026-06-21): determinism is NOT the disk lever — identical-artifact hardlink-dedup of the *cargo target tree* is

The original priority-#1 framing below ("determinism unlocks cross-worktree dedup from 13 %
→ ~90 %") was **measured and disproven** — but so was the *first* pivot ("share one
`CARGO_TARGET_DIR`"). The corrected, **validated** finding:

### 1. The ~17 % yield was measured on the wrong tree.

A post-fix dedup-yield measurement on a `build-i386` (7.58 GB) found only ~17 % dedup-able. But
`build-<arch>` is **not** the disk hog — the **`sdk/lib/rust/target` cargo tree is**: 9.7–26 GB
*per worktree* (vibeos main 26 G, netstack 25 G, kernel32 23 G, filesystems 20 G …). Across 11
worktrees the caches total **281 GB**, of which the cargo target trees dominate.

### 2. A shared `CARGO_TARGET_DIR` is OFF the table — it's a *correctness* hazard, not a perf trade.

`sdk/lib/rust/reactos-link/src/lib.rs` deliberately gives every cargo-EXE module its **own**
per-crate target dir (`target/<pkg>-<crate>-rustlib`). Quoting its rationale:

> PER-CRATE dir (not shared): a shared dir lets cargo feature-unify across crates and emit
> duplicate symbols with different code that `--allow-multiple-definition` resolves first-match
> to the wrong variant.

So the ~98 per-crate target dirs each recompile build-std (core/alloc/compiler_builtins) and the
shared rlibs — **that redundancy is the deliberate price of per-crate isolation**, and collapsing
to one target dir would mis-link the kernel. Cache *sharing* via a single target dir is ruled out.

### 3. The real, safe lever: hardlink byte-identical artifacts (`hardlink --ignore-time`).

The per-crate build-std rlibs are **byte-identical** (same content, just built at different times),
and post-determinism the per-worktree crates are byte-identical *across* worktrees too. Hardlinking
them preserves per-crate isolation completely (each path still exists; they just share inodes).
**`--ignore-time` is essential** — util-linux `hardlink` defaults to requiring identical mtime,
which hid ~all of it (a default run found 767 KiB; `--ignore-time` found 6.4 GiB in one tree).

**Measured (2026-06-21):**

| scope | before | reclaim |
|---|---|---|
| intra-worktree (one cargo target tree) | 9.7 GB | **6.4 GiB → 3.3 GB** |
| **FULL FLEET cargo target trees (11 worktrees)** | — | **113 GiB (60,899 files)** |

ext4 has no reflink/CoW, so hardlink is the mechanism. **Safety validated end-to-end:** cargo
always emits via tempfile + atomic rename (never in-place mutation), so a later rebuild gets a
*new* inode and the link breaks cleanly — worst case a spurious rebuild, never wrong output.
Proven: dedup → warm i386 build (7m41s, valid 433 MB iso) → **i386 smoke ✓ boot 83 s**. Re-bloat
after a warm rebuild is partial (3.3 G → 4.9 G here) and recovered by re-running the dedup, so it
is a **periodic maintenance pass**, not a permanent build-graph change.

**Tool:** `tools/dedup-build-cache.sh` (dry-run by default; `--apply` to hardlink; auto-discovers
all worktrees via `git worktree list`; skips trees with a recent write to dodge live-build races).

### Determinism's retained value is therefore NOT disk directly

It's payoffs #2 and #3 below (anti-stale de-risk + reproducibility), **plus it is what makes the
cross-worktree hardlink-dedup actually land** — without the path/timestamp normalization the
"identical" crates wouldn't be byte-identical across worktrees and only the (path-independent)
build-std would dedup. The anti-stale value already paid off concretely: the genuinely-cold
`--clean` (rung #41) unmasked two real arm64 bugs (PTE-cache alignment + stale-fastfat) that warm
builds had hidden all session → arm64 now cold-boots green in ~38 s.

The original payoff list is kept below for history; read it through the pivot above.

## Why determinism (three payoffs — ORIGINAL framing; see pivot above)

1. **Disk — the cross-worktree dedup multiplier (priority #1 — DISPROVEN, see pivot).** Measured
   2026-06-20: hardlink-dedup across 5 `build-i386` trees saved only **3.94 G / ~13 %**, because
   "identical" binaries aren't byte-identical — they embed the worktree's absolute path
   (debug info, `__FILE__`) and a PE build timestamp. Strip those and the dedup ceiling rises
   from ~13 % toward the ~90 %-actually-unchanged reality. ~~Determinism is the lever that makes
   the disk plan's dedup real.~~ **Corrected 2026-06-21:** measured post-fix yield is ~17 %, not
   ~90 % — the bulk of a tree is regenerable cache, so cache-SHARING (shared `CARGO_TARGET_DIR` +
   ccache/PCH), not determinism dedup, is the disk lever. (See `build-cache-and-determinism-plan.md`
   → Disk-first reframing.)
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
| 4. build-id / debug-GUID determinism ✅ | identical debug directory | SATISFIED — i386 cold build-twice is byte-identical, so the GUID/build-id is already deterministic (no random/time id); no separate fix needed |
| 5. reproducibility smoke ✅ (gauge built + RUN green i386) | identical bytes across builds/agents, wired as a gauge | `vibeos-cli repro verify --arch X` (cold build-twice → byte-identical); **i386 RUN BYTE-IDENTICAL 2026-06-26**; amd64/arm64 + cross-worktree run pending |

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
  - **cc-rs c_shims path remap ✅ FIXED 2026-06-21** (the one compile path that missed the remap).
    An empirical `strings` audit of a built `ntoskrnl.exe` (i386) found a residual **worktree**
    abs-path leak: `…/<worktree>/ntoskrnl/mm_lock_outlines.c`. Root cause: `reactos-link` compiles
    the kernel C shims/asm/stubs via **cc-rs with the absolute `repo_root.join(rel)` source path**
    and carried **no `-ffile-prefix-map`** — so clang embedded the real path into `__FILE__`/assert
    strings. cmake's own C TUs already remap (compile_commands carries `-ffile-prefix-map=<wt>=` +
    `../=`) and the rustc objects via `--remap-path-prefix=<root>=/vibeos`; the cc-rs compiles were
    the gap. **Fix:** `RosLink::apply_prefix_map()` maps the worktree root → the same `/vibeos`
    token, wired into all four cc-rs sites (c_shims, asm, stubs, asm_archives). **Verified by
    rebuild:** worktree-path leaks in ntoskrnl.exe **1 → 0**; `mm_lock_outlines.c` now embeds
    `\vibeos\ntoskrnl\mm_lock_outlines.c` (constant token, no worktree path).
  - **Empirical cross-worktree path audit of the rebuilt ntoskrnl.exe (i386, 2026-06-21):**
    **0** worktree-specific paths (`vibeos-wt-buildcache` / `/build-i386/`) — source + build-dir
    fully remapped. The remaining embedded abs paths are **home-relative toolchain paths, identical
    in every worktree on this machine** so they do NOT break same-machine dedup: **260** ×
    `/home/curi/.rustup/toolchains/…` (build-std core/alloc sources) + **~1829** llvm-mingw. **Net:
    cross-worktree byte-identity on this machine is now essentially achieved** for the source/build
    paths the remap targets.
  - **2 residual `…/os-playground/vibeos/{arbiter,lsa}.c` (MAIN-path) strings — likely a
    ccache/sccache cache-coherence artifact, NOT a build-rule gap.** This worktree's
    compile_commands.json is fully worktree-correct (0 main refs) and the rebuilt `liblsalib.a` /
    `libarbiter.a` carry 0 main refs — so these two strings come from neither. The build-cache lane
    runs ccache; the most likely source is a cache-HIT object compiled earlier in *main* (whose
    embedded `__FILE__` predates/bypasses the remap). If every worktree hits the same main-cached
    object the string is *constant* (harmless to dedup); a cache-off cold build would confirm/clear
    it. → flagged for the cold build-twice gate (which runs cache-off).
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
- **Rung 5 ✅ gauge built + RUN GREEN on i386** (gauge 2026-06-21; first real run **2026-06-26**,
  commit `eaae890a98f`): `vibeos-cli repro verify --arch i386` cold-builds the arch twice (deleting
  the copied cargo exes between to force a genuine relink — defeats the §stale-cargo / task-#41
  false-identical trap) and byte-compares. **Result: cold build-twice BYTE-IDENTICAL ✓** for all
  three core artifacts:
  ```
  ✓ identical  ntoskrnl/ntoskrnl.exe
  ✓ identical  win32ss/win32k.sys
  ✓ identical  dll/win32/kernel32/kernel32.dll
  → cold build-twice BYTE-IDENTICAL ✓   (3m23s real, cache-accelerated)
  ```
  This is the end-to-end proof that rungs 2a (PE-timestamp zeroing) + 2b (`SOURCE_DATE_EPOCH`) hold
  on a real cold build, not just `repro report`. **Still pending: amd64 + arm64 same-path verify**
  (the fixes are all-arch but only i386 is RUN-proven), and the stronger **cross-worktree
  `diff-dirs`** proof that exercises the rung-1 path remap's cross-tree value (evidence already
  strong: the strings-audit found 0 worktree-path leaks in ntoskrnl.exe, remaining paths are
  home-relative toolchain paths identical on-machine — see Rung 1 notes).
- **Rung 4 (build-id / debug-GUID determinism) ✅ SATISFIED — verified, no work needed.** The
  i386 cold build-twice being byte-identical *proves* the PE debug directory / CodeView GUID /
  build-id are already deterministic (content-derived or absent) — a random/time-based id would
  have differed between the two builds. So rung 4 is closed by the rung-5 result, not a separate
  TODO. (Re-confirm when the amd64/arm64 verify runs.)
- **Pending ×3 cold gate** for the rung-1/2/2b build changes — superseded: these landed on mainline
  via the build-cache merge (`53f453b586e`, ×3 boot-green) and are now in c-r-p. The determinism
  *effect* is now RUN-proven on i386 (above). Remaining determinism TODO: amd64/arm64 verify +
  cross-worktree diff-dirs.

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

## Stale-cargo-binary elimination (the #2 payoff — root-caused 2026-06-20, ✅ FIXED 2026-06-21)

The non-standard cargo-driven kernel link can serve a **stale binary** — the concrete form of
the #2 footgun. Two distinct gaps (NOT one), both now closed:

- **§9.6b "doesn't relink on a `.rs` edit" (staticlib only) — ✅ FIXED.** `ntoskrnl/CMakeLists.txt`
  built the kernel-rust staticlib with `add_custom_command(OUTPUT ${RUST_KERNEL_LIB} … DEPENDS …)`
  carrying a **hand-maintained 200-line `.rs` file list** — necessarily incomplete (ntoskrnl-rust
  alone is ~405 .rs; kernel-rust is an umbrella pulling in every sibling rlib via `extern crate`),
  so editing any `.rs` not listed left ninja thinking the staticlib was current → stale kernel
  masking the fix. (`add_cargo_module` in `gcc.cmake` was already fine — `file(GLOB_RECURSE _srcs
  CONFIGURE_DEPENDS)` + `DEPENDS ${_srcs}`, so the *exe* relinked.) **Fix:** replaced the hand-list
  with `file(GLOB_RECURSE _rust_kernel_srcs CONFIGURE_DEPENDS ${WS}/*.rs ${WS}/*.toml)` +
  `list(FILTER … EXCLUDE REGEX "/target/")` (don't depend on build artifacts). **Verified:** the
  regenerated `build-i386/build.ninja` libkernel_rust.a edge now lists **2032** `.rs` deps (was
  ~204), 0 under `/target/`, incl. cross-crate `mm/mod.rs` + `rtl-rust/src/lib.rs` + deep `ke/`
  files — any workspace `.rs` edit now forces the relink.
- **"wiped target → stale exe" (the `rm -f output` trap) — ✅ FIXED.** cargo outputs are *copied*
  out of the target dir, so nuking the target alone leaves the copy present + newer than the
  `.rs` → ninja "no work" → stale. Worse, the cargo-EXE links build into the **shared workspace
  cache `sdk/lib/rust/target/<stem>`** (i686/x86_64/aarch64-kernel), which lives OUTSIDE
  `build-<arch>` — so `discover_rust_targets` (globs `build-<arch>`) never saw it, and `--clean`
  left cargo serving a warm-cached object (the manual `rm -rf sdk/lib/rust/target/<stem>` agents
  ran by hand). **Fix (`build.py:_clean` + `provenance.py`):** `_clean` now also (a) removes the
  workspace cargo stem via `provenance.workspace_cargo_target(repo, arch)` and (b) deletes the
  copied cargo-module output binaries via `provenance.discover_rust_module_outputs(repo, arch)`
  (the kernel(s) + win32k.sys + kernel32.dll + the arm64 staged set) so ninja MUST re-run each
  cargo link. Source-unchanged modules not in that set rebuild byte-identically anyway (the
  determinism invariant), so a stale copy of one is harmless; the listed set is exactly the
  frequently-edited boot-critical links whose staleness is a false-green. Unit-tested
  (`test_workspace_cargo_target_per_arch`, `test_discover_rust_module_outputs`).

Chosen over a blanket "always-run cargo" (corrosion-style): there are ~48 `*-cargo` modules, so
always-invoking cargo would add 48 no-op cargo checks per `ninja` — too costly. The two targeted
fixes close both gaps without that overhead. Both are build-path changes → **own ×3 cold gate**
(cmake `cmake .` reconfigure of build-i386 verified clean; the full build-twice relink proof is
the gate's job).

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
- ✅ **C abs-path `-ffile-prefix-map` per-worktree-correct (no residual).** The earlier "hardcoded
  to main's path" worry was from reading *main's* generated `build.ninja`. Inspected a freshly-
  configured **worktree** (`vibeos-wt-buildcache`) build.ninja 2026-06-21: the two maps are
  `-ffile-prefix-map=../=` (relative, worktree-safe) and `-ffile-prefix-map=<THIS worktree's abs
  path>=` — i.e. `${CMAKE_SOURCE_DIR}` expands to the worktree's OWN root, mapping it to empty.
  No `…/vibeos`(main) hardcode present. Two worktrees each erase their own root → byte-identical.
  Resolved. (Separate, still-deferred: `DW_AT_comp_dir` records the abs *build* dir
  `<worktree>/build-<arch>`, which neither prefix-map covers → a possible debug-info-only residual.
  Measure with `repro --diffoscope` on a real build-twice; add `-ffile-compilation-dir=.` only if
  it actually shows. Don't widen the blast radius on spec.)
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
