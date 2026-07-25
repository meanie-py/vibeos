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

### `build --arch X [--clean] [--incremental] [--no-check] [--strict] [--no-cache]`
Compile the boot media for one arch, lint the binary, index symbols, and write a
provenance stamp.

- **Fast local iteration:** `--incremental --no-check` is the inner-loop combo.
  A warm build is dominated by two things that neither caching nor the compiler
  touch: the **source-lint suite** (~53s, led by `scan_inline_flag_literals`
  AST-walking every `.rs`) and **linking**. `--no-check` skips the source lints
  (they're a gate concern, not an every-edit one); `--incremental` turns on rustc
  incremental (a kernel `.rs` edit recompiles the ~13s `ntoskrnl-rust` crate in
  ~5s instead), trading the Rust sccache wrapper (sccache + incremental are
  mutually exclusive) — ccache for C/C++ stays on. **Both are LOCAL-only:**
  incremental artifacts are non-reproducible and the lints gate correctness, so
  the `gate`/CI path uses neither. Measured: warm i386 build ~97s → ~40s with the
  combo (under load; the ratio holds quiet).

- **Canonical build path (2026-07-03):** every build runs with the worktree
  bind-mounted at `/tmp/vibeos` (unprivileged userns — `tools/canon-run.sh`,
  `canon.py`), so compiler argv + embedded paths + CMake caches are identical
  from every worktree. A missing `build-<arch>` is auto-provisioned (fresh
  configure, fast on cache hits); a legacy real-path one is auto-migrated (loud
  `canonize` stage, one-time full rebuild). Raw ninja against a canonical dir:
  `../tools/canon-run.sh ninja <target>` from inside it.
- **Warm by default** (incremental — fast iteration). `--clean` is *cold*: nukes
  **every** discovered rust target dir + every `lib*_rust.a`, then rebuilds. The
  stamp records which mode was used; warm builds are flagged because cargo
  incremental *can mask regressions*.
- **Cache is MANDATORY** (disable with `--no-cache` / `VIBEOS_BUILD_CACHE=0`):
  C/C++ through ccache, Rust through sccache, dirs keyed off `VIBEOS_ROOT` (the
  worktree-parent). With the canonical path, hits are shared **across worktrees
  for both languages** and across the clean-smoke target nuke (measured:
  100%/100% on a `--clean` replay). Reports a per-build hit-rate stage (also a
  determinism gauge). Also fires a detached ≤1/day hardlink-dedup of the cache
  trees (`tools/dedup-build-cache.sh`). Design + status:
  `docs/engineering/build-system.md`.
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

### `smoke --arch X [--allow-stale] [--repeat N] [--until M] [--fail-on RE] [--tripwire CFG] [--timing] [--no-net]`
**The verdict front door.** Staleness preflight, then boot the real media via
the same `QemuVM` the pytest harness uses (slots, reaper, lifetime cap, live-log
all included), then auto-triage on failure.

- **Net suite (amd64) runs by default** and folds into the verdict — a netstack
  regression turns the smoke red like any other, never hidden. It is a *second
  full UEFI boot* (the virtio-net config), kept separate from the plain boot so
  the desktop-reach North Star stays free of slirp/hostfwd flake; that isolation
  is why it's an extra boot. `--no-net` skips it for fast local boot-only
  iteration (an escape hatch, not the gate default).
- **Early-hang watchdog:** a boot that emits serial then goes fully silent past a
  safe threshold (`ArchConfig.hang_silence_seconds` — 120s x86 / 240s arm64) is
  declared WEDGED and fails *fast* instead of burning the full `boot_timeout`
  (saves ~4–5min on a genuinely wedged arm64/x86 boot). A slow-but-progressing
  boot keeps emitting, so it is never false-tripped.

- **Hard-fails on stale or missing media** — it will *not* silently boot the
  wrong binary. If you're bisecting and *want* the current media regardless, pass
  `--allow-stale` (you'll get a loud WARN). Otherwise: `build` first.
- `--repeat N` runs N independent boots to characterize a flake; a partial pass
  (`0 < pass < N`) is reported `FLAKY` and counts as **FAIL** — a flake is a real
  signal, not green.
- `--until` overrides the success marker (default: the arch's boot marker
  `[EXPLORER] _tWinMain entered`, on **all three arches**); `--fail-on RE`
  aborts at the first match of a hard-failure regex.
- **Green requires `shell ready`, not just the boot marker.** After the marker,
  the settle-window serial is scanned (not discarded): a BugCheck / Rust panic
  / kernel debugcon `PANIC:` during settle fails the boot, and `shell ready`
  must follow the marker (else `SHELL_NOT_READY`). A custom `--until` skips the
  shell-ready requirement (early-marker debug boots).
- On failure it captures a screenshot + debugcon triage, classifies
  (`RUST_PANIC` with file:line — including kernel panics that only reach
  debugcon / `BUGCHECK 0xNN` / `HANG_TIMEOUT` with a slow-vs-wedged
  characterization + census/lockdep wedge self-confirmation / `FAIL_MARKER` /
  `SHELL_NOT_READY` / `KNOWN_FLAKE`), and symbolicates the faulting PC via the
  offline index. All in the summary — no manual objdump round-trip.
- On a green boot, kernel `[CENSUS] STUCK` / `[LOCKDEP]` self-reports surface
  as a `health` WARN stage (+ `census_stuck` in summary.json) — the
  "boots green but threads are silently parked forever" signal, trendable
  across runs.
- `--tripwire "SPECIALPOOL=Gla5"` arms per-boot kernel debug knobs via QEMU
  fw_cfg (no media rebuild; kernel side `ntoskrnl-rust/src/tripwire.rs`) —
  special pool turns silent pool corruption into an immediate symbolicated
  bugcheck at the corrupting store. Full recipe: field manual §9.18.
- `--timing` stamps every live-log serial line with elapsed seconds
  (`VIBEOS_SERIAL_TIMESTAMPS=1`, serial_client.py tee — never touches the
  marker-matching buffer) and prints a boot-phase report after the boot:
  phase table (firmware → freeldr → kernel → SMSS/csrss → explorer), the
  largest silent gaps, and per-file freeldr load throughput (the `[FLDR]`
  stamps from `boot/freeldr/freeldr/ntldr/winldr.c`, enabled by the
  `Debug=` line in `boot/bootdata/livecd.ini`). Standalone analyzer:
  `python3 tools/boot_phase_report.py qemu/<vm_arch>_smoke_live.log
  [--sizes build-<arch>]`. This is how the 27s freeldr silent gap and the
  5s menu-countdown tax were found (docs/engineering/
  performance-investigation.md).

### `farm [--arches L] [--boots N | --minutes M] [--tripwire CFG]` / `farm-report [--since TS] [--sha S]`
**The flake farm** — soak independent boots (parallel worker per arch; the
hostlock CPU-lease pool is the throttle) into a durable corpus at
`~/vibeos-flake-farm/corpus.jsonl`: one JSONL record per boot with verdict,
classification, stop code, the self-symbolicating bugcheck banner lines,
tripwire arming, sha and duration; failure artifacts are copied out of
reboot-volatile `/tmp`. `farm-report` clusters failures into ranked
`(arch, classification, stop code, top symbol)` families with sample paths —
a 1-in-50 flake becomes overnight statistics instead of lore. Canonical
overnight soak: `farm --arches i386,amd64,arm64 --minutes 480 --tripwire
"SPECIALPOOL=*"`. Staleness-gated like smoke.

### `gate [--arches i386,amd64,arm64] [--retries N] [--no-build] [--clean] [--no-net]`
**The parallel pre-land verdict** — the North Star "gate ×3 (all arches green)"
done concurrently instead of by a serial human loop. It builds each arch (serial,
still under the exclusive host CPU lease — `--no-build` to skip), then launches
**all arches' smokes in parallel** (wall-clock goes sum→max), then for any arch
that fails retryably launches up to `--retries` (default 2) concurrent fresh
attempts and passes the arch on the first green. Retry-eligible: a *documented*
flake (`KNOWN_FLAKE` / `FLAKY`), an environment failure (`LAUNCH_FAIL`), or a
`HANG_TIMEOUT` that smoke characterized as **slow/contended** (serial still
flowing at the deadline) with **no** kernel census/lockdep wedge self-report.

- **A fresh regression is never retried.** `BUGCHECK` / `RUST_PANIC` /
  `FAIL_MARKER` / `SHELL_NOT_READY` / `STALE_MEDIA` / `UNKNOWN` — and a
  HANG_TIMEOUT that went **silent** (wedge-shaped) or that the kernel's own
  `[CENSUS]`/`[LOCKDEP]` instrumentation confirmed — fail the arch immediately;
  retrying a regression away is exactly what the North Star forbids. (The old
  blanket HANG_TIMEOUT retry dated from the native-KVM contention era and let a
  ~50% service wedge pass a 3-attempt gate ~87% of the time.)
- **Concurrency is safe to over-submit.** Each smoke is a `vibeos-cli smoke`
  subprocess that self-throttles on the host boot-CPU pool (`hostlock` `bootcpu`,
  capacity-derived — 8 on a 24-vCPU box, was a flat 4) and the global QEMU cap
  (16); the per-arch serialize cap is bypassed on the default windows-host TCG
  backend, so same-arch retries run in parallel too.
- Exit 0 only if **every** arch is ultimately green. The summary carries a
  per-arch `matrix` (verdict / classification / attempts / recovered / run_dir);
  each arch's own `summary.json` lives under the gate's artifact dir.
- Auto-push discipline unchanged: a ×3-clean-gate-green landing is still what
  authorizes a push — but drive it through `land` (below), not by hand.

### `land [--direct] [--drain] [--wait] [--retries N] [--clean] [--no-push] [--dry-run]`
**The way to land — it is a MERGE QUEUE.** With no flags, `land`:

1. publishes this lane's tip to the queue (`refs/vibeos/queue/<seq>` + a record
   under `$GIT_COMMON_DIR/vibeos-queue/`) and returns in ~2s — you do **not**
   merge master, gate, or wait;
2. if no other agent holds the land lock, *becomes the trainer*: merges every
   queued lane onto current `origin/master` in a dedicated integration worktree,
   gates **once**, and fast-forwards master to exactly the commit it gated;
3. writes a verdict per submission — `land-status --mine` (or your next `land`)
   shows it.

**N ready lanes cost ONE gate, and no agent blocks.** The trainer is elected by
the lock, so there is no daemon to supervise and nothing that can be "down".

- **Conflicts are bounced, never resolved** — rejected with the conflicting
  paths; the rest of the batch proceeds.
- **A red batch splits recursively**, landing any green half immediately, until
  the culprit is one submission — rejected with its classification + artifact
  dir. Innocent lanes still land. Batch cap `VIBEOS_QUEUE_BATCH` (default 4)
  bounds the worst-case bisect.
- **Guilty or framed?** Before rejecting a lone submission, the trainer gates the
  BASE alone (`origin/master`, nothing merged). If the base is red too, **nobody
  is rejected**: the submissions stay queued, the train HALTS, and the verdict
  says mainline is broken. This is not hypothetical — the very first real batch
  hit it (mainline failed to compile `fltmgr-rust` for i386) and, without this
  check, rejected a docs-only change for a build failure it could not have
  caused. The base gate is lazy and memoized per drain, so a green batch never
  pays for it.
- **A batch is never presumed green from its parts.** Two individually-green
  lanes can be red *together*; the queue catches that before master, where today
  the second lane lands on the first without ever gating the combination.
- **Crash-safe**: a submission is a ref + a JSON record; the trainer's claim is an
  flock that dies with it. Kill a trainer mid-batch and its submissions are still
  queued for the next one. A submission leaves the queue only when it has landed
  or been rejected — repeated work is possible, a silent drop is not.
- The **integration worktree** (`$VIBEOS_ROOT/vibeos-integration`) is created on
  first use and deliberately never auto-pruned: its warm build dirs + cargo target
  are why a batch gates in minutes, and every landing in the fleet would otherwise
  pay for a cold one.
- `--drain` trains without submitting (recovers the microsecond-wide orphan window
  where a submission arrives just as a trainer releases the lock). `--wait` blocks
  for your own verdict if you'd rather not poll.

### `land --direct` — the single-lane path
One serialized critical section, under the same fleet-wide FIFO
lock: `fetch → merge origin/master → gate → push HEAD:master`. Right when you're
the only active agent, or to bypass the queue.

Why it exists: the gate stamp is keyed to an exact **tree hash**, so gate-then-
push by hand is a race. Whoever pushes first wins; every other in-flight lane's
tree is now stale, its stamp is void, and it must merge and **re-gate** — and
that wasted gate was 3 builds + 4 boots of *fleet* load, which lengthens
everyone else's gate and widens the window that voided it. Past a couple of
active lanes it's a congestion collapse: "everyone is always waiting on the gate
and racing to finish". Holding the lock across gate+push removes the race
outright:

- a green gate **always** lands — nobody re-gates because someone else landed;
- the fleet spends exactly **one gate per landing**, not one per attempt;
- waiting is FIFO, bounded and visible (`land-status` / `host-status`) instead of
  a lottery the most patient lane keeps losing;
- the gate runs on the **final** tree — the one actually pushed.

Details:
- **Gate inside the lock; don't pre-gate.** A speculative gate is the thing that
  used to get thrown away. If a fresh (<24h) green full-arch stamp already covers
  the tree being pushed, `land` skips the gate, so the uncontended path is
  seconds.
- **Pushes `HEAD:master`** and never touches the local `master` ref — moving
  `master` from a worktree that doesn't have it checked out leaves that worktree's
  files at the old tree, i.e. a giant phantom staged revert (2026-07-25).
- Refuses a dirty tree or detached HEAD (a dirty tree can't be stamped).
- A conflicting merge is **aborted** and reported rather than resolved while
  holding the fleet's lock.
- The land lock **hard-fails** on timeout (`LandLockBusy`, naming the holder)
  instead of the soft "proceed unlocked" the CPU pools use — proceeding unlocked
  would reopen the very race. `VIBEOS_LAND_LOCK=0` opts out; the `pre-push` hook
  is still the backstop either way.

### `land-status [--mine] [--limit N] [--json]`
The merge queue at a glance: who is training, what is queued behind them (with
wait times), and recent verdicts — a rejected lane's triage pointer lives here.
`--mine` filters verdicts to this lane. The lock holder also appears in
`host-status` as a `LANDING :` line plus the `land` queue.

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

Covers: `shellcheck`, `py-syntax`, `verify_ffi` (the ABI checker — `verify_abi`
was folded into it, T1, and no longer runs as a separate lint), `verify_constants`,
`lint_kernel` (source); `scan_ud2`, `sacred_primitives` (binary, needs a build —
`check --strict --arch X`).

The **binary** lints are judged by their `--check` **exit code**, not baseline-diff:
their output is arch-specific (so a single baseline can't match across arches) and
they already encode the correct per-arch verdict. `scan_ud2` is arch-aware — it
scans x86 `ud2` (LLVM UB trap) on i386/amd64, and **branch-to-self `b .` spins** on
arm64 (which has no UB trap: the captures(none)/UB class is *silent* in arm64
binaries and is caught at source by `lint_kernel` instead — the arm64 binary scan
only catches a function that regressed into an infinite loop, the "idle hang"
signature).

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

### `gdb <verb> ...` — GDB over the **windows-host** backend (native-Python engine)
The `debug` verbs above shell out to `qemu/*.sh`, which are **native-only** (Linux
qemu, KVM — where software breakpoints don't reliably fire). `gdb` is the full
Python port (`gdbengine.py`) that drives the **default windows-host TCG** QEMU,
where breakpoints DO fire. It reuses the same `qemu/gdb-helpers/*.py` verbs (they
run inside gdb, transport-agnostic); only the launch + gdb orchestration is
re-owned in Python. QemuVM emits the gdbstub bound `0.0.0.0` on the Windows stack
(`vm._gdb_args`), reached from WSL via `gdb_host = gateway_ip()` — the same
inbound path as a slirp hostfwd.

```bash
RUN=$(./vibeos-cli gdb start amd64 | grep run-id | awk '{print $3}')
./vibeos-cli gdb wait-for $RUN --pattern IoInitSystem --timeout 200
./vibeos-cli gdb run-until $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck --then "bt 30"
./vibeos-cli gdb inspect $RUN bt 30      # one-shot verb (bt/triage-bugcheck/dump-trapframe/...)
./vibeos-cli gdb ps                      # list debug runs + liveness
./vibeos-cli gdb stop $RUN
```

**CRITICAL gotcha (baked into the engine):** connecting to QEMU's gdbstub *halts
the vCPUs*. So liveness (`status`/`ps`) checks the **Windows process + serial-file
growth**, NEVER a socket connect to the stub — a connect-probe freezes the guest
mid-boot (serial stuck at 0, PC parked in OVMF). Only `run-until`/`inspect` connect
(they intend to halt, then `detach` resumes). The detached QEMU persists across CLI
invocations (a separate Windows process); address it by run-id, kill by uid tag.

### `ps [--json]`
Every QEMU running **across all worktrees** + slot-pool usage. Run it first when
you're unsure what's live or why KVM is contended.

### `prune [--apply] [--self] [--idle-hours N]`
Reclaim disk by GC'ing **idle worktrees' build-`<arch>`/ + Rust target dirs**.
Build trees are ephemeral scratch — the shared cache rehydrates a pruned tree
fast — so an idle lane's 26–42 G can be freed and regenerated on demand (measured
~150 G across worktrees, most of it idle). **Dry-run by default** — prints what
*would* be freed and why each tree is kept; deletes nothing without `--apply`.
Hard safety gates: never a worktree with a **live QEMU** (same lock files `ps`
reads), never one with a **running ninja/cargo/rustc/clang/lld**, never the
**current** worktree unless `--self`. `--idle-hours N` additionally protects any
tree touched in the last N hours (recommended — the live gates are instantaneous,
so an actively-worked lane that just isn't building *this second* otherwise looks
prunable). Only ever removes the build/target dirs, never source or the worktree.

### `repro {report <pe>… | compare <a> <b> | diff-dirs <buildA> <buildB>}`
The reproducible-builds gauge (determinism lane, rung 0 —
`docs/engineering/archive/deterministic-builds-plan.md`). A self-contained PE parser (no
`pefile`/`diffoscope` dependency) answers *are two builds byte-identical, and if
not, where do they diverge?* — `report` dumps each artifact's COFF
`TimeDateStamp` (flags `← LEAK` when non-zero) + sha256; `compare` localizes a
mismatch to the timestamp and/or specific sections; `diff-dirs` runs `compare`
over the staged artifact set (ntoskrnl.exe, win32k.sys, kernel32.dll) across two
build trees — the build-twice-in-two-paths check in one command. `--diffoscope`
shells out for byte-level depth if it's installed (optional).

### Launch backend config (`native` vs `windows-host`)
The smoke/boot runner picks a QEMU **backend** from layered config — committed
`DEFAULTS` (in `config.py`) **<** a gitignored per-machine `<repo>/.vibeos-cli.local.json`
**<** env (`VIBEOS_QEMU_*`). **Default is now `windows-host`** (agents run on a Windows
host — escapes WSL2 nested-virt contention). `native` (local `qemu-system-*` + Unix-socket
QMP/serial + KVM/TCG) is the right choice on real Linux / CI — override via
`VIBEOS_QEMU_BACKEND=native` or a `.vibeos-cli.local.json`. **`windows-host`** (WSL2 only) launches the *Windows* QEMU.exe via interop to
escape WSL2 nested-virt contention (`docs/engineering/archive/windows-host-qemu-spike.md`):
media is staged to a Windows-visible dir, QMP+serial run over TCP (we listen on
`0.0.0.0`, the Windows QEMU dials back `127.0.0.1` via WSL2 localhost forwarding), and
the process is killed by a uid-scoped `Stop-Process`. To enable, copy
`.vibeos-cli.local.example.json` → `.vibeos-cli.local.json` and set
`qemu.backend="windows-host"` (+ `qemu.windows.{qemu_dir,staging_dir,accel,host_addr}`).
`accel` is `tcg` (works today, no reboot — already beats nested-KVM under concurrency)
or `whpx` (needs HypervisorPlatform + reboot; uses `qemu-system-x86_64.exe`). x86 only;
arm64 stays native TCG. Validated under TCG: **i386 + amd64** both boot to `shell ready`
(amd64 via `qemu-system-x86_64.exe`), and **QMP savevm/loadvm** (the snapshot fixture path)
round-trips over the TCP transport. The backend choice is read wherever a `QemuVM` is
constructed — the cli smoke path *and* every pytest path (the folded net stage,
`test_snapshot_apps`, a direct `test_smoke`) via `conftest.py`. The KVM-only idle-cpu gate
(`test_idle_cpu_not_spinning`) skips on windows-host (TCG, and `_proc.pid` is the WSL interop
shim — same reason it skips arm64). The **net suite also runs on windows-host** (`test_net.py`
11/11 validated): the slirp `hostfwd` binds `0.0.0.0` Windows-side and tests dial `vm.net_host`
(the WSL default-gateway IP, not `127.0.0.1` — WSL's loopback is its own VM's), and the
`guestfwd ...-cmd:cat` outbound echo (no `cat` on Windows) is redirected to a harness-run echo
server via a chardev. So a WSL2-no-KVM user on windows-host gets the full smoke — boot North Star
*and* the folded netstack suite.

### `check-tooling [--no-types] [--no-tests]`
The lint/type/test gate for **our own Python** — distinct from `check`, which
lints the *kernel* source. Runs, over the whole vibeos-cli tool surface
(`tools/rustconv` incl. semdiff, this `cli` package, and the standalone
`tools/*.py` scripts): **ruff** (lint, `E/F/I/UP/B/SIM/RUF`), **ruff
format --check**, **pyright** (basic), and **pytest**. One line per check, tails
the failing output, nonzero on any failure. `--no-types`/`--no-tests` for a fast
lint-only pass. Run it before handing off a change to the tooling. Config lives
per-package: `tools/rustconv/pyproject.toml`
(installable sub-pkg) carries its own `[tool.ruff]`/`[tool.pyright]`; the cli +
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
