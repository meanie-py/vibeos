# `vibeos-cli` — the build / check / smoke / debug front door

**If you are an agent about to build, boot, or debug VibeOS: start here.** This
one CLI is the front door to the whole loop. It exists so you don't have to
remember which arch needs `ninja livecd` vs `mkramdisk --refresh`, whether your
boot media is stale, what the boot marker is, or how to attach gdb to a UEFI
boot. It does the bookkeeping; you read a one-line verdict and a log path.

Invoke it via the repo-root wrapper (worktree-aware — finds the right Python):

```bash
./vibeos-cli <subcommand> --arch <arch> [opts]
```

`<arch>` is one of `i386 amd64 arm64` (or the explicit `i386-uefi`/`amd64-uefi`
aliases — x86 boots UEFI now, BIOS is dead). **There is no default arch on
purpose** — agents bisecting across arches must say which one, every time.

---

## Why this exists (read this once)

Three classes of bug kept biting us, and all three are *invisible* without
tooling:

1. **False green from stale artifacts.** Cargo's incremental cache and stale
   thin-archives (`ar crT` never removes members) let a build "pass" while the
   binary on the boot media is the *old* one. An agent edits Rust, the smoke
   goes green, and the change was never actually tested. → **Provenance &
   staleness gating** (`provenance.py`). Smoke *refuses to boot stale media*.
2. **Per-arch build divergence.** x86 is `ninja livecd`; arm64 is a staged
   binary set + `mkramdisk --refresh` (and forgetting the re-stage means you
   boot last week's `ntoskrnl`). The "q35 vs pc" / "PAE vs non-PAE" class of
   divergence between the fast path and the real smoke. → **One source of truth
   for build+boot facts** (`archcfg.py`), shared by `build`, `smoke`, and
   `boot`.
3. **Token-flooding boot logs.** Dumping a full serial log into the transcript
   burns budget and buries the verdict. → **Bounded summary to stdout, full log
   to a file** (`artifacts.py`). Every run writes
   `/tmp/vibeos-cli/<ts>-<cmd>-<arch>/` with `summary.json` (machine-true,
   re-readable — never scrape stdout), `full.log`, `commands.jsonl`, plus any
   screenshot/serial/debugcon captured.

The CLI is also a *control plane*, not a reimplementation: the `debug` verbs
**wrap the proven `qemu/*.sh` toolkit** rather than re-porting it. Use whichever
layer fits — see "Relationship to `qemu/`" below.

---

## Subcommands — what each is for, and the strategy

### `build --arch X [--clean] [--no-check] [--strict]`
Compile the boot media for one arch, lint the binary, index symbols, and write a
provenance stamp.

- **Warm by default** (incremental — fast iteration). `--clean` is *cold*: nukes
  **every** discovered rust target dir + every `lib*_rust.a`, then rebuilds. The
  stamp records which mode was used; warm builds are flagged because cargo
  incremental *can mask regressions*.
- **Strategy:** iterate warm; do a `--clean` build before you believe a green
  result or hand off. The clean build is the only one that proves a cold-link
  state. (The smoke harness's clean-build does the same nuke for the same
  reason.)
- Auto-runs the binary lints (`scan_ud2`, sacred primitives) and source lints
  (advisory) so you can't forget them. `--no-check` skips source lints;
  `--strict` turns NEW lint findings into failures.
- Build failures are **classified**: `CMAKE_CONFIGURE_FAIL` /
  `CARGO_CHECK_FAIL` / `LINK_FAIL` (with i386 PE-decoration hints on undefined
  symbols — `_Foo@N` = stdcall, `@Foo@N` = fastcall, `_Foo` = cdecl) /
  `RESTAGE_FAIL`. The classification tells you *which layer* broke without
  reading the log.
- arm64 automatically re-stages into `ramdisk.img`/`boot.img` (the trap that
  silently boots a stale kernel otherwise).

### `smoke --arch X [--allow-stale] [--repeat N] [--until M] [--fail-on RE]`
**The verdict front door.** Staleness preflight, then boot the real media via
the same `QemuVM` the pytest harness uses (slots, reaper, lifetime cap, live-log
all included), then auto-triage on failure.

- **Hard-fails on stale or missing media** — it will *not* silently boot the
  wrong binary. If you're bisecting and *want* the current media regardless, pass
  `--allow-stale` (you'll get a loud WARN). Otherwise: `build` first.
- `--repeat N` runs N independent boots to characterize a flake; a partial pass
  (`0 < pass < N`) is reported `FLAKY` and counts as **FAIL** — a flake is a real
  signal, not green.
- `--until` overrides the success marker (default is the arch's boot marker —
  `shell ready` on x86, idle on arm64); `--fail-on RE` aborts at the first match
  of a hard-failure regex.
- On failure it captures a screenshot + debugcon triage, classifies
  (`RUST_PANIC` with file:line / `BUGCHECK 0xNN` / `HANG_TIMEOUT` /
  `FAIL_MARKER` / `KNOWN_FLAKE`), and symbolicates the faulting PC via the
  offline index. All in the summary — no manual objdump round-trip.

### `boot --arch X --until M [--fail-on RE]`
Raw fast boot, **no staleness gate**, but reuses smoke's *exact* arch-config so
the fast path can't diverge from the real smoke (the q35-vs-pc misdiagnosis
class). Use it to watch a specific marker quickly; use `smoke` for verdicts.

### `check [--arch X] [--strict] [--update-baseline]`
Source lints with a **baseline-diff gate**: the bar is **zero NEW findings vs
the checked-in baseline** (`tools/lint-baseline/<tool>.txt`), *not* absolute
zero — the tree carries known pre-existing findings. Advisory by default;
`--strict` blocks on NEW. `--update-baseline` re-captures current findings as
accepted (review the diff before committing — you're declaring them OK).
Baselines are stored repo-relative so they match across worktrees.

Covers: `shellcheck`, `py-syntax`, `verify_abi`, `verify_ffi`,
`verify_constants`, `lint_kernel` (source); `scan_ud2`, `sacred_primitives`
(binary, needs a build).

`--strict` additionally runs the **`cargo_link`** guard: it re-runs the
cargo-driven link PoCs (`sdk/lib/rust/{ntoskrnl,ntdll,ntdll_vista}-cargo` via
the shared `reactos-link` engine — the long-term replacement for the cmake
links) and asserts each still produces a valid PE. These live off the default
build path, so this is what keeps them from silently bit-rotting. Defaults to
i386 (the only arch all three share — `ntoskrnl-cargo` is i386-only); pass
`--arch amd64`/`arm64` to guard ntdll/ntdll_vista there. Each link self-skips
if its target json, `build-<arch>`, or nightly+rust-src is absent. Gated to
`--strict` (not every `build`) because each relink re-compiles a Rust staticlib
(~20s) and would tax the hot loop.

The `cargo_link` guard is the **LINK** half of the cargo-drive gate (PE-valid,
fast, no boot). Its **RUNTIME** half is `cargo-swap` (below) — a link can relink
clean yet boot broken, and only a boot catches that.

### `cargo-swap --arch X [--module all|ntdll|ntdll_vista|ntoskrnl]`
**The runtime gate for the cargo-driven modules.** Boot-swaps each cargo-built
module into the real livecd and boots it to the smoke marker via `vibeos-cli
smoke` — proving the cargo link is runtime-equivalent to cmake's, not just the
same PE shape. Wraps each module's `boot-swap-smoke.sh` (which owns the build /
swap / cmp-or-sha-assert that the iso carries the cargo bytes / restore-cmake-on-
exit mechanics) and aggregates one verdict + `summary.json`. Covers:
- **ntdll / ntdll_vista** (x86) — `sdk/lib/rust/ntdll-cargo/boot-swap-smoke.sh`.
- **ntoskrnl** (i386 only) — `sdk/lib/rust/ntoskrnl-cargo/boot-swap-smoke.sh`,
  which also applies the kernel post-link (`native-pefixup --kernel` + the i386
  forwarder fix) the standalone cargo link doesn't.

x86 only — arm64 has no livecd for the swap (skips cleanly); ntoskrnl skips on
amd64 (i386-only PoC). Slow (boots QEMU per module) — run it before trusting a
cargo-drive handoff, not in the hot loop.

### `debug <verb> ...` — wraps the `qemu/` kernel-debugging toolkit
Run-id-scoped GDB-over-the-real-boot debugging. Verbs pass through to the proven
shell tools (`start` → `start.sh`, `inspect`/`bt`/`triage-bugcheck` →
`inspect.sh`, `run-until`, `wait-for`, `tail`, `status`, `stop`, `profile` →
`sample-pc.sh`). **`debug diff <runA> <runB>`** is CLI-native: a normalized
(hex/pid-masked) serial diff that surfaces arch-unique lines — built for
arch-only-bug differentials.

The standard kernel-bug recipe (now UEFI, mirroring smoke's boot path). NOTE:
`debug` verbs pass args **verbatim** to the wrapped `qemu/*.sh` tools, which take
a **positional** arch (`start.sh <arch>`), not `--arch`:
```bash
RUN=$(./vibeos-cli debug start amd64 --gdb | tail -1)
./vibeos-cli debug wait-for $RUN --pattern "ACPI Timer" --timeout 60
./vibeos-cli debug run-until $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```

### `ps [--json]`
Every QEMU running **across all worktrees** + slot-pool usage. Run it first when
you're unsure what's live or why KVM is contended.

### `check-tooling [--no-types] [--no-tests]`
The lint/type/test gate for **our own Python** — distinct from `check`, which
lints the *kernel* source. Runs, over the whole vibeos-cli tool surface
(`tools/rustconv` incl. semdiff, this `cli` package, the standalone `tools/*.py`
scripts, and `agent-swarm`): **ruff** (lint, `E/F/I/UP/B/SIM/RUF`), **ruff
format --check**, **pyright** (basic), and **pytest**. One line per check, tails
the failing output, nonzero on any failure. `--no-types`/`--no-tests` for a fast
lint-only pass. Run it before handing off a change to the tooling. Config lives
per-package: `tools/rustconv/pyproject.toml` + `agent-swarm/pyproject.toml`
(installable sub-pkgs) carry their own `[tool.ruff]`/`[tool.pyright]`; the cli +
`tools/*.py` use the repo-root `pyproject.toml`. Policy is identical across all
four so the surface lints under one standard.

---

## The artifact contract (how to read results without flooding context)

Every run prints a bounded summary and writes a durable dir. **Read
`summary.json` for the machine-true verdict; grep `full.log` for detail.** Never
scrape stdout for the result — stdout is intentionally lossy.

```
/tmp/vibeos-cli/<ts>-<cmd>-<arch>/
  summary.json     verdict, classification, per-stage status, salient lines, artifact paths
  full.log         raw stage output under === headers ===
  commands.jsonl   every subprocess invoked (cmd, cwd, exit, seconds)
  boot*/screen.ppm, serial tails, debugcon — on failure
```

Exit code: `0` for OK/WARN, `1` for FAIL. Stdout glyphs: `✓` ok, `⚠` warn,
`✗` fail, `·` skip.

---

## Typical loops

```bash
# edit Rust → fast warm check it still boots i386
./vibeos-cli build  --arch i386 && ./vibeos-cli smoke --arch i386

# before believing green / handing off: cold build, all three
for a in i386 amd64 arm64; do ./vibeos-cli build --arch $a --clean; done
for a in i386 amd64 arm64; do ./vibeos-cli smoke --arch $a; done

# chase a flake
./vibeos-cli smoke --arch amd64 --repeat 5

# a boot is hanging early — watch a specific marker, fail fast
./vibeos-cli boot --arch i386 --until "KiSystemStartup" --fail-on "BugCheck"

# kernel BUGCHECK — gdb over the real (UEFI) boot (debug verbs take POSITIONAL arch)
RUN=$(./vibeos-cli debug start amd64 --gdb | tail -1)
./vibeos-cli debug run-until $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```

---

## Relationship to `qemu/` (when to use which layer)

| Need | Use |
|------|-----|
| A pass/fail verdict that won't lie about stale media | `vibeos-cli smoke` |
| Build the right media for an arch (warm/cold, classified failures) | `vibeos-cli build` |
| Watch a specific boot marker fast | `vibeos-cli boot` |
| Lint gate (NEW-vs-baseline) + cargo-link rot guard | `vibeos-cli check [--strict]` |
| Prove a cargo-driven module boots (ntdll DLLs / ntoskrnl) | `vibeos-cli cargo-swap` |
| Interactive/ad-hoc single GDB op on a live run-id | `qemu/inspect.sh` (or `vibeos-cli debug inspect`) |
| Raw interactive launcher (serial/GUI/`--debug`) | `qemu/run.sh` |
| What's live across worktrees | `vibeos-cli ps` (== `qemu/ps`) |

`qemu/README.md` indexes the shell toolkit; `docs/engineering/qemu-cli-port-plan.md`
is the design rationale and section references (`§N`) cited throughout the code.

---

## Gotchas worth internalizing

- **No default arch** — always pass `--arch`. Aliases: `i386`/`amd64` resolve to
  the UEFI boot config; `base_arch()` keys the build dir, `vm_arch()` keys the
  boot config.
- **Warm build flag is a warning, not a pass** — cargo incremental can mask a
  regression. `--clean` before you trust green.
- **`smoke` won't boot stale media** — that's the feature. `build`, or
  `--allow-stale` for a deliberate bisect.
- **arm64 must re-stage** — `build` does it; if you boot arm64 by hand without
  `mkramdisk --refresh`, you're testing the old kernel.
- **Read `summary.json`, not stdout** — the durable verdict is the file.
