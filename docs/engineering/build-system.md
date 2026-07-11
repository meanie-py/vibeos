# Build System — canonical path, shared cache, determinism, cargo-driven link

The durable as-built reference for how vibeos builds. Consolidates four earlier lane docs
(all in `archive/` for the deep detail + war stories): `build-cache-and-determinism-plan`,
`deterministic-builds-plan`, `cargo-drive-cmake-off-roadmap`, `build-tools-self-hosting`,
`cargo-target-model-builtin-vs-buildstd`. Front door for all of this is **`./vibeos-cli
build`** (guide: `tests/qemu/cli/README.md`); the mechanics live in `tools/canon-run.sh`
and `tests/qemu/cli/{build,canon,cachecfg}.py`.

## The canonical build path (the keystone)

**Every worktree builds at ONE absolute path: `/tmp/vibeos`.** `vibeos-cli build` (via
`tools/canon-run.sh`) bind-mounts the worktree there inside an unprivileged user+mount
namespace and runs cmake/ninja/cargo inside. Compiler argv, embedded paths, `getcwd()`,
and CMake caches become **worktree-independent by construction** — which is what makes the
shared cache and cross-worktree determinism work at all.

Consequences you'll actually hit:
- **Provisioning a worktree's `build-<arch>` is just a fresh `cmake` configure** (fast on
  cache hits). `vibeos-cli build` auto-provisions a missing dir and auto-migrates a legacy
  real-path one (loud one-time `canonize` rebuild that repopulates the cache). The old
  `cp -a` + sed-repoint ceremony is **retired** — never symlink or hand-copy a build dir.
- **Raw `cd build-<arch> && ninja` no longer works** — the baked paths only exist inside
  the mount. Use `../tools/canon-run.sh ninja <target>` or `vibeos-cli build`. Plain
  `ninja` fails loudly by design.
- Copy the gitignored arm64 boot assets (`qemu/QEMU_EFI.fd efivars.fd`) into a fresh
  worktree for smoke/debug; `boot.img`/`ramdisk.img` are rebuilt by the mkramdisk step.

## Shared compiler cache (mandatory)

**ccache (C/C++) + sccache (Rust) are on by default** (`--no-cache` /
`VIBEOS_BUILD_CACHE=0` to disable) and shared across all worktrees. Because everything
builds at the canonical path, **sccache hits cross-worktree with zero `--remap-path-prefix`
flags** (measured 100% on the same crate from a second worktree). The old det-remap
(`--remap-path-prefix=<root>=/vibeos`) is **removed** — it silently marked every rustc
invocation non-cacheable in sccache (0.8.2: `Non-cacheable reasons: --remap-path-prefix`),
disabling all Rust caching while it was live; canonical paths give the same
path-independence without it. The C `-ffile-prefix-map` stays (its value is now the
canonical path).

Sharp edges (all handled in `canon-run.sh`/`canon.py`):
- **sccache server namespace affinity** — a server resolves paths in the namespace it was
  spawned in, so canon-run stops any live server on entry (race-free under hostlock).
- `.build-tools/llvm-mingw` is a relative symlink that dangles under the mount —
  `canon.ensure_toolchain()` heals it to absolute and provisions it in fresh worktrees.
- Fresh trees exposed generated-header dep rot (RC/cc-rs racing `psdk`/`xdk` winnt.h gen,
  masked for months by copied build dirs) — `build` primes `host-tools asm psdk xdk
  bugcodes errcodes` as a barrier stage before the main ninja.

### Disk sharing is delete-only — hardlink-dedup is BANNED

**Do NOT hardlink-dedup build/cargo trees.** The 2026-07-04 fleet-isolation incident
(AGENTS.md § Worktree Discipline, rule 9) proved it corrupts both ways: in-place writers
(spec2def, `cmake -E copy`, ccache-miss clang) truncate shared inodes under a sibling's
running `ld.lld` mmap (SIGBUS), and shared mtimes silently defeat cargo invalidation
(false-green). Sharing belongs ONLY in content-addressed caches (ccache/sccache).
`vibeos-cli build` enforces this with an **`ISOLATION_FAIL` preflight** (refuses a tree
with shared inodes: build-* at links>1; cargo target at links>2 — cargo hardlinks
deps↔final internally). Fleet disk maintenance is delete-only (`vibeos-cli prune`).
*(The archived build-cache doc still advertises an automated `dedup-build-cache.sh` — that
tool is retired; this section is the current truth.)*

## Determinism / reproducible builds — essentially done

Byte-identical kernel artifacts across worktrees on all three arches (gauge:
`vibeos-cli repro`). The rung ladder is effectively complete:
- **Rung 1** (path independence) — landed *as the canonical build path* above.
- **Rung 2a** (PE timestamp, `-Wl,--no-insert-timestamp`) + **2b** (`SOURCE_DATE_EPOCH`,
  derived by `vibeos-cli build`) — implemented, *gate-pending* (not yet enforced).
- **Rung 3** (build.rs/generator date-audit) — clean. **Rung 4** (build-id/debug-GUID) —
  satisfied all 3 arches. **Rung 5** (byte-identity gauge) — built + green ×3.
- Loose ends only: wire 2a/2b into the gate, an optional cross-worktree
  `diffoscope`/`diff-dirs` proof, and the C `trim-paths` add. Tracked in the roadmap
  Check-back ledger.

## Cargo-driven kernel link (cmake-off progress)

`ntoskrnl.exe`/`ntkrnlmp.exe` already link via **cargo with zero cmake-built static
libraries** (`static_libs` is an empty `vec![]`) — the link graduated from cmake
`add_executable` to the `reactos-link` engine (`ntoskrnl/CMakeLists.txt`). "Cmake-off" =
removing cmake as the *orchestrator/producer* too; a quality/hermeticity goal, not a boot
goal (same binaries). What cmake still provides to the cargo build:

| Coupling | Mechanism | Cmake-off blocker? |
|---|---|---|
| `compile_commands.json` | `.c_from_compile_db()` replays clang for the irreducible C floor (~50 files / 10 cells) | **YES — biggest** |
| Generated asm includes | `ks{386,amd64}.inc`/`ksarm64.h` (cmake `native-geninc`) | **YES** |
| Resource `.res` | `gen_ntoskrnl_res.py` needs cmake-built `bugcodes.rc`/`.bin` (windmc) | YES (narrow) |
| ninja orchestration | `add_cargo_module()` sequences the cargo cells | YES |
| bindgen headers | `ntkrnl-sys` FFI | NO — committed |

**Host-tool self-hosting** (needed for cmake-off; legend ✅ self-hosted / 📦 committed
+ drift-guarded / 🔧 in toolchain / 🌐 external): 12 ✅, 4 📦, 3 🔧, 1 🌐 (`windmc`, Homebrew
mingw-w64). `spec2def`/`native-hpp` are self-hosted; `windres`/`widl` ride committed
fallbacks (drift-guarded by `tools/check_committed_drift.py` / `vibeos-cli check --strict`).

## Cargo target model — builtin triple vs `-Z build-std`

Two ways rustc gets `core`/`alloc`/`compiler_builtins`:
- **Builtin triple** (`x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`) — rustc uses
  *prebuilt* std for the exact triple. Used for **amd64 + arm64**.
- **`-Z build-std` + custom `.json`** — rustc compiles std from source for a bespoke
  target. Used for **i386** (the `i686-kernel.json` no-SSE **softfloat** kernel target;
  no prebuilt std exists for it). `compiler_builtins` supplies what LLVM *calls but
  doesn't define* (softfloat `__addsf3`/`__floatdisf`; arm64 outline atomics).

## The `--allow-multiple-definition` link flag

The kernel/driver/DLL link still passes `-Wl,--allow-multiple-definition` (lld first-match
wins) — load-bearing for the C↔Rust cohab swap-in phase and for structural runtime dup
symbols (compiler-rt/libgcc + Rust unwind/panic/i128 that each independent staticlib graph
re-emits). **Killing it categorically is a parked lane** (`kill-allow-multiple-def`
branch): a validated `objcopy --redefine-sym` runtime-rename quarantine replaces the flag
for the structural dups, leaving a small genuine-dup tail to fix in source. Resume state +
the flag-off censuses are on that branch; ratchet tool is `tools/scan_duplicate_symbols.py`.
