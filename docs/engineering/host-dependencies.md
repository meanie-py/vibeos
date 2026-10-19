# Host dependencies — what this box needs, and how each one fails

Written 2026-09-19, after rebuilding WSL from scratch. **Fourteen external dependencies
were discovered one crash at a time over two days**, because no list existed and nothing
checks for them. This file is the list. It exists so the next rebuild is an afternoon, not
an archaeology dig.

**The failure signature column is the point.** Almost none of these fail by saying the tool
is missing. They fail as a compile error in an unrelated file, a scanner that silently finds
nothing, or a test asserting the wrong number — so the cost is never the install, it is the
hours spent debugging the wrong thing.

## The list

| Need | Probe with | How it fails when absent |
|---|---|---|
| rust toolchain (pinned stable) | `rustc --version` | everything |
| pinned nightly (`.rust-nightly`) | `rustup run $(cat .rust-nightly) rustc -V` | i386 `-Z build-std` cell only |
| **rustfmt**, for BOTH toolchains | `rustfmt --version` | **silent**: bindgen emits an unformatted token stream, `postprocess_bindgen.py`'s regexes match nothing, `bindgen_common.rs` is never written, `test_bindgen_generation` fails on a missing file |
| rust target `i686-unknown-linux-gnu` | `rustup target list --installed` | `msvcrt-seh-difftest`: "can't find crate for `core`" |
| rust targets `i686-pc-windows-gnu`, `x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm` | same | the arch will not build |
| cmake · ninja · ccache · sccache | `--version` | build |
| qemu (Windows-side too) | `qemu-system-x86_64`; Windows build 11.1.0 extracted to `C:\qemu-11.1.0` (`qemu.windows.qemu_dir`), with Windows Firewall ALLOWING inbound for each `qemu-system-*.exe` | smoke |
| bindgen + libclang | `bindgen --version` | `-sys` crate generation |
| bison · flex | `--version` | build |
| **`windmc`** | `x86_64-w64-mingw32-windmc` — **NOT bare `windmc`** | `committed_drift` bugcodes.h check |
| **GNU `windres`** (binutils-mingw-w64, same package as `windmc`) | `x86_64-w64-mingw32-windres --version` must say **GNU windres** — llvm-mingw ships a `windres` of the same name that is llvm-rc in a trench coat, so the name alone proves nothing | `--rostests` only: `user32_winetest` is skipped with a configure WARNING. Its `resource.rc` uses DIALOGEX control creation data, which llvm-rc has never implemented (`expected identifier, got {`, on 18 and 19), and that one file also carries the suite's accelerators/strings/menus |
| mtools · dosfstools · xorriso | `mcopy`, `mkfs.fat`, `xorriso` | media staging |
| shellcheck | `shellcheck -V` | `vibeos-cli check` |
| **`gcc-mingw-w64-i686`** | `i686-w64-mingw32-gcc` | **silent fallback**: `CreateBootSectorTarget` quietly uses clang instead, surfacing as three `unexpected offset operator` errors against a months-old file |
| **ripgrep** | `command -v rg` — **not `rg --version`** | 16 rustconv tests die on `PermissionError: 'rg'`; worse, `substrate_closure`/`cpp_core` treat a bad rg exit as "no matches", so a *broken* one yields a silently EMPTY substrate |
| **c2rust 0.22.1** | `c2rust --version` | every `--c2rust` translate hard-fails (by design — it refuses to degrade). `cargo install c2rust --locked`, needs llvm-18-dev + libclang-18-dev, ~2 min |

**The cargo registry cache must match the committed lock** — `cargo fetch --locked` in every
directory holding a tracked `Cargo.lock` (`sdk/lib/rust`, `tools/host-jobs`). Not a package,
and the worst failure of the fourteen because it disables a guard instead of breaking a
build. The rebuilt box's cache had been filled from a *different* resolve than the lock:
lock pinned `aho-corasick 1.1.4`, cache held 1.0.1 and 1.1.5. So `cargo metadata --offline`
exited 101, and `queue.settle_cargo_locks` — the preflight that exists because a lock-rewrite
once wedged three lanes for an hour each in an unbreakable re-gate loop — **skipped itself on
every conductor batch since the rebuild**. It logs when it skips, which is the only reason
this was noticeable at all; the batches still gated green, so nothing failed and the
protection was simply absent. Probe it the way the preflight does:

    for d in $(git ls-files -- '*Cargo.lock' | xargs -n1 dirname | sort -u); do
        (cd "$d" && cargo metadata --offline --format-version 1 >/dev/null) || echo "STALE: $d"
    done

Also non-obvious, not a package: `VIBEOS_DISK_BACKING_MOUNTS=/mnt/g` must be exported, or
disk admission cannot see the volume the guest actually consumes.

## Two traps this audit hit itself

**Probe by the name the build uses, not the bare tool name.** A bare `windmc` probe reports
MISSING on a box where `x86_64-w64-mingw32-windmc` is installed and working. This file nearly
shipped that false claim.

**`rg --version` succeeds when ripgrep is not installed.** Claude Code defines `rg` as a shell
FUNCTION, so an interactive check passes while every `subprocess` call fails. Use
`command -v`. This is the general shape — *a tool's own account of itself is not evidence
about that tool* — and it is why the probe column above is spelled out rather than left to
taste.

## This should be a command

Fourteen dependencies, most failing in disguise, discovered by running things until they
broke. That is the detection event for a missing mechanism: there should be a
`vibeos-cli check-host` that probes this table and names what to install, run as a
`check-tooling` preflight. Until it exists, this file is a rule that someone has to remember
— which AGENTS.md is explicit is a rule that will be broken.
