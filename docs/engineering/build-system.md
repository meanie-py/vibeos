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
  the mount. Use `bash ../tools/canon-run.sh ninja <target>` or `vibeos-cli build`. Plain
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
- **Builtin triple** (`x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`) — rustc links
  the *prebuilt* std that rustup ships for the exact triple. Used for **amd64 + arm64**,
  by BOTH the library crates and (since 2026-07-12) the 42 `*-cargo` link cells. **These
  arches are entirely on pinned STABLE rustc** (`rust-toolchain.toml`).
  i386's user-mode cells (KIND win32dll / win32exe) joined them on `i686-pc-windows-gnu`
  on 2026-10-14.
- **`-Z build-std` + custom `.json`** — rustc compiles std from source for a bespoke
  target. The **only** remaining user is the **i386 kernel kinds** (`i686-kernel.json`, the no-SSE
  **softfloat** kernel target — `rustc-abi: x86-softfloat` has no builtin triple, so no
  prebuilt std can exist). That cell alone needs nightly (pinned in `.rust-nightly`), and
  it is *retired* by the wow64 lane rather than fixed. `compiler_builtins` supplies what
  LLVM *calls but doesn't define* (softfloat `__addsf3`/`__floatdisf`).

**The policy lives in `gcc.cmake:add_cargo_module`** and is mirrored in three re-runners
that must stay in sync: `tests/qemu/cli/lints.py`, `tools/cargo_orchestrate.py`, and
`sdk/lib/rust/cargo-cell-args.sh` (the dev helper scripts source that last one).

Moving the cells onto the builtin triples means the target spec is rustc's, not ours, so
three things it used to give us are restored/neutralised at build+link time instead:
- **`-C force-frame-pointers=yes`** — the retired json said `frame-pointer: always`; the
  builtin triples default to may-omit (x86_64) / non-leaf (aarch64). Load-bearing: the
  kernel debug toolkit walks the frame chain (`qemu/inspect.sh bt`).
- **`-C link-self-contained=no`** (amd64 only) — x86_64-pc-windows-gnu declares
  `crt-objects-fallback: mingw`, which would prepend the user-mode CRT startup objects
  (crt2.o/dllcrt2.o) to a kernel module. aarch64-pc-windows-gnullvm *rejects* the flag and
  needs no equivalent.
- **Empty CRT stub archives** (`reactos-link`) — both triples name the user-mode CRT in
  `late-link-args` (`-lmsvcrt -lgcc -lkernel32 …`) and rustc offers no way to suppress
  them, so an empty `libmsvcrt.a` etc. on a preceding `-L` shadows every one. This is a
  deliberate HARD WALL: a symbol we fail to provide becomes a LOUD undefined-symbol error
  naming it, instead of being silently satisfied by the user-mode CRT (which would graft
  an msvcrt.dll import onto a kernel driver).
- **`libmemcmp.a` fallback** — dropping build-std loses
  `-Z build-std-features=compiler-builtins-mem`, and the precompiled `core` calls `memcmp`.
  ntoskrnl.exe exports memcpy/memset/memmove on every arch but `memcmp` only on x86_64, so
  reactos-link links `sdk/lib/crt/libmemcmp.a` LAST in the link group — a pure fallback, so
  any cell that already defines memcmp keeps its own.
  **Never hand-write mem\* in Rust to plug this** — it self-recurses; see field manual §9.65.

Trade-offs measured on the 2026-07-12 cutover (amd64): ntoskrnl `.text` **+0.4%** (flat),
most modules **−16…−29%**, but small modules that only *touch* `core` grow sharply (bootvid
+115%, hal +77%) because `x86_64-pc-windows-gnu` sets `function-sections: false`, so
`--gc-sections` cannot strip unused `core` functions and one reference drags in a whole
codegen unit — build-std used to compile `core` with our `codegen-units=256`, giving
fine-grained dead-code elimination for free. Net ISO impact **+0.2%**. arm64 has the mirror
image (`function-sections: true`) and *shrinks*: hal −85%, csrss −80%, ntoskrnl −2.9%.
`-Z function-sections` is nightly-only, so amd64's coarse granularity is not fixable on
stable — accepted.

## spec2def stubs

`spec2def()` (gcc.cmake) writes each module's stubs as `${file}_stubs.rs`, a `#![no_std]`
file that one `rustc --emit=obj` compiles to `${file}_stubs.rs.obj`. There is no cargo
involved, and the bodies are the C emitter's: print the call, raise EXCEPTION_WINE_STUB for a
bare `@ stub`, return 0. Modules still list `${file}_stubs.c` in their sources; a
`cmake_language(DEFER)` at the end of the calling directory swaps it for the object, and gives
a stub-only DLL `LINKER_LANGUAGE C`. A module in another directory naming the `.c` fails at
generate time.
- **amd64/arm64**: pinned stable, builtin triple, prebuilt core.
- **i386**: stable `i686` cannot be used, because it requires sse2 (disabling it warns of a
  future hard error) and the C is `-march=pentium`. The stubs use the pinned nightly and
  `tools/spec-stubs-core/i686-kernel.json` (the pentium json the user-mode cells used
  before they moved to the builtin triple), compiled against a sysroot that
  `tools/spec-stubs-core` builds once per build dir with `-Z build-std`. A float argument is
  printed as its bit pattern there, because the f32→f64 promotion would call
  compiler-builtins' `__extendsfdf2`, which a stub object cannot link.
- `WITH_RELAY` modules keep the C emitter: relay-tracing thunks have no Rust form, and
  `spec2def --with-tracing -s=x.rs` refuses. No module uses `WITH_RELAY` today.
- msvc.cmake's `spec2def()` is untouched.
- Every stub's line names its module, `WARNING: calling stub <dll>!<name>(...)`, which is
  what `tools/stub_hits.py` ranks. `-stub=VALUE` on a typed row (`@ stdcall
  -stub=E_NOTIMPL F(ptr)`) is the non-raising kind: it logs the first call per process and
  returns VALUE (`E_NOTIMPL`, `STATUS_NOT_IMPLEMENTED`, `FALSE`, `NULL` or a number).

## The `--allow-multiple-definition` link flag

The kernel/driver/DLL link still passes `-Wl,--allow-multiple-definition` (lld first-match
wins) — load-bearing for the C↔Rust cohab swap-in phase and for structural runtime dup
symbols (compiler-rt/libgcc + Rust unwind/panic/i128 that each independent staticlib graph
re-emits). **Killing it categorically is a parked lane** (`kill-allow-multiple-def`
branch): a validated `objcopy --redefine-sym` runtime-rename quarantine replaces the flag
for the structural dups, leaving a small genuine-dup tail to fix in source. Resume state +
the flag-off censuses are on that branch; ratchet tool is `tools/scan_duplicate_symbols.py`.

**Per module, it is off where the module is ready (`link_strict(<target>)`, gcc.cmake).**
The flag is a generator expression on the target's `VIBEOS_STRICT_LINK` property, and a
strict module also drops `libclang_rt.builtins`. lld fetches clang_rt for an intrinsic
whenever a Rust member that references it is parsed before the whole-archived
`compiler_builtins` member that defines it, which is a duplicate of a symbol Rust already has.
A module qualifies when it is one Rust graph with no C library: one `core`, one panic runtime,
one `compiler_builtins`. Code that used to come from separate Rust staticlibs goes into that
graph as an rlib. pseh-rust, setjmp-rust and delayimp-rust are split so that is possible:
`*-rt` holds the code, and the `*-rust` facade is the standalone staticlib that adds the panic
runtime. Strict since 2026-10-04: msvcrt, msvcrt20, msvcrt40, crtdll, ucrtbase. Anything the
graph lacks becomes visible: arm64 `__chkstk` (compiler_builtins has none) went to ntdll's
import library until crt-shared-rust grew one. So diff the import table before and after
flipping a module, and use `tools/link_pulls.py --arch A --target T` (lld's own record of
which archive member it loaded, and for which symbol) to see what a link really pulls.
