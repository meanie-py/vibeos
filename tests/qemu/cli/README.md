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

### `build --arch X [--clean] [--incremental] [--check] [--strict] [--no-cache] [--extra-target T]`
Compile the boot media for one arch, lint the binary, index symbols, and write a
provenance stamp.

- **`--extra-target T`** (repeatable) also builds a Ninja target the boot DAG does not
  include, keeping the full boot build and its guards — instead of raw `ninja T`.

- **Fast local iteration:** source lint is no longer part of a normal build.
  Use `check` for source analysis or opt into advisory lint with `build --check`.
  The gate runs the complete source inventory separately. `--incremental` turns on rustc
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
  `bash ../tools/canon-run.sh ninja <target>` from inside it.
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
- Runs binary lints (`scan_ud2`, sacred primitives, self-recursion).
  Source lints are opt-in (`--check`); `--no-check` remains a compatibility alias
  for the default. Strict source verification runs through `check --strict` / `gate`.
- Build failures are **classified**: `CMAKE_CONFIGURE_FAIL` /
  `CARGO_CHECK_FAIL` / `LINK_FAIL` (with i386 PE-decoration hints on undefined
  symbols — `_Foo@N` = stdcall, `@Foo@N` = fastcall, `_Foo` = cdecl) /
  `RESTAGE_FAIL`. The classification tells you *which layer* broke without
  reading the log.
- arm64 automatically re-stages into `ramdisk.img`/`boot.img` (the trap that
  silently boots a stale kernel otherwise).

### `smoke --arch X [--allow-stale] [--repeat N] [--until M] [--fail-on RE] [--tripwire CFG] [--timing] [--no-net] [--ladder]`
**The verdict front door.** Staleness preflight, then boot the real media via
the same `QemuVM` the pytest harness uses (slots, reaper, lifetime cap, live-log
all included), then auto-triage on failure.

- **`--arch amd64-bios`** boots the same amd64 media through SeaBIOS (legacy BIOS,
  `freeldr.sys`) instead of OVMF, with the ISO on a USB stick (xHCI) as a Libreboot
  machine would boot it. Every gate that includes amd64 runs it as the
  `smoke:amd64-bios` lane. `VIBEOS_QEMU_BOOT_TRANSPORT=usb-ehci|ide-cd|usb-storage`
  picks the boot device on any x86 arch. See `docs/engineering/bios-boot.md`.

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
- `--repeat N` runs N independent boots sequentially to characterize a flake; a partial pass
  (`0 < pass < N`) is reported `FLAKY` and counts as **FAIL** — a flake is a real
  signal, not green.
- Each `boots[]` entry in `summary.json` includes `timings`: total wall time,
  start, marker watch, desktop settle, and stop. Start also exposes accelerator
  resolution, admission wait, launch-lock wait, and preparation (media plus process
  launch). These components overlap: preparation and waits are part of start;
  total includes all phases. Compare identical smoke options and successful boots.
- **`--ladder` — the WoW64 probe guests, which a plain smoke CANNOT see.** They
  are spawned by winlogon *after* explorer, i.e. after the desktop marker this
  boot already matched, so the ordinary `smoke`/`gate` stops watching before the
  first guest runs. A guest can therefore be RED on one arch indefinitely with
  every gate green — that is not hypothetical, `selfmod32.exe` exited `0xBAD9`
  on amd64 for a day (read-only livecd volume vs. arm64's writable ramdisk) and
  nothing reported it. `--ladder` keeps draining serial after the marker until
  all nine guests report, and **fails the boot** on a wrong or missing verdict.
  Run it on **both** 64-bit arches after touching a ladder guest, `wow64.dll`,
  or a marshaller. arm64 needs media built with `VIBEOS_FEX=1`; a boot with no
  guests staged is reported `LADDER_NOT_STAGED` with the build recipe rather
  than as a regression. Expected-verdict table and rationale:
  `tests/qemu/cli/ladder.py`. Bounded extra watch — with the flag off, the boot
  path is unchanged.
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
- Every smoke boot owns `<run-dir>/bootN/serial.log` (zero-based), including
  `--repeat` runs. `summary.json` lists all results under `boots`; its top-level
  serial archive selects the first failure, or first success if all passed.
  Use those paths for live observation and evidence, not the legacy per-arch log.
- **`VIBEOS_QEMU_HOSTILE=all` (amd64) boots a QEMU stripped down to what a 2018+
  mini PC gives you**, so real-hardware bugs show up at smoke speed. Or name the
  hazards to bisect, e.g. `VIBEOS_QEMU_HOSTILE=q35,serial`:
  - `q35`: ICH9 chipset, AHCI boot media, HPET.
  - `cpu`: `-cpu max`, so TCG enforces the CPU's features.
  - `nops2`: no i8042; input only through USB HID on xHCI.
  - `novga`: GOP from ramfb only.
  - `mem`: 6 GiB.
  - `serial`: no COM1; the console UART sits at I/O 0x5000, named only by an
    injected ACPI SPCR, the way AMT serial-over-LAN or a BMC is on a real PC.

  All of them, plus `VIBEOS_QEMU_SMP=4`, boot green as of 2026-10-03. Opt-in, not
  in the gate. `vibeos-cli gdb start amd64` honours the same knob.

  arm64 takes the same variable, with the ways a Raspberry Pi differs from
  QEMU's default virt (`all` applies them too):
  - `a76`: `-cpu cortex-a76`, the Pi 5's core (ARMv8.2: PAN and friends enforced).
  - `el2`: firmware hands off at EL2, as both Pis' UEFI do, so the loader's drop
    to EL1 runs.
  - `mem`: 6 GiB. This is what caught the 4 GiB Pi 5's silent hang (nonpaged
    pool sizing underflow).

  `a76,el2,mem` boots green on arm64 as of 2026-10-04. amd64 `mem` sees only
  4 GiB until the roadmap's "amd64 RAM above 4 GiB" bug is fixed.
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
  `python3 tools/boot_phase_report.py <run-dir>/boot0/serial.log
  [--sizes build-<arch>]`. This is how the 27s freeldr silent gap and the
  5s menu-countdown tax were found (docs/engineering/
  performance-investigation.md).

### Windows media cache and cleanup

Windows launches automatically reuse content-addressed, read-only media across
worktrees. Every launch hashes its source; a per-run hard link pins the cached
image. Writable firmware variables and scratch disks remain private. Cold fills
use small buffered writes to avoid DrvFS allocation failures. The cache retains
up to 2 GiB plus media pinned by active runs; no opt-in is needed.

`./vibeos-cli windows-cleanup --dry-run` reports abandoned staging paths;
`./vibeos-cli windows-cleanup` removes them. Smoke, gate, farm, and debug launches
also sweep automatically, at most once every five minutes. Cleanup recognizes
only tool-owned paths older than one hour, protects live Linux owners and media
referenced by Windows QEMU command lines, and refuses deletion if Windows process
inspection fails. It shares the launch lock so it cannot race a new launch.
Completed runs still remove their own staging immediately. The sweep recovers
leftovers after interrupted runs and retires old unpinned cache entries.

Each boot directory contains `<arch>_launch.json` (actual backend, accelerator,
argv, admission and preparation timings) and `<arch>_qemu.stderr.log`. Stderr is
a file, so an undrained pipe cannot stall QEMU or hide a host crash. TCG remains
the default accelerator; WHPX is opt-in pending a green network suite and a
measured workload improvement.

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

### `flake-report [--days N] [--arch A] [--source smoke|conformance] [--min K] [--json]`
**Which fleet failures are flakes.** `farm-report` reads only `farm` soaks; the fleet
runs `smoke`/`gate`/`conformance`, so this reads the durable ledger (every run) plus the
preserved evidence under `~/vibeos-flake-farm/` (history from before ledger rows carried
signatures). Each failure is reduced to a signature (`BUGCHECK 0x1E post-marker @ fn`,
`HANG_TIMEOUT wedged | <last serial>`, `assert File.c:N`, the payload/ladder that failed),
then graded at the commit it was seen on: **FLAKE** = present in one run and observed
*absent* in another at the same commit; **DETERMINISTIC** = present in every observing run
(n≥2); **UNGRADED** otherwise. A run only counts as absence if it could have seen the
failure (a lost boot says nothing about an assertion it never reached). Dirty-tree runs
are listed separately and never graded, because their commit does not name the code. The
evidence pointer says so when it pointed into `/tmp` and is gone.

### `gate [--arches i386,amd64,arm64] [--retries N] [--no-build] [--clean] [--no-net]`

The source-scanner suite runs **once in two disjoint partitions**. Five audited
Rust-source checks start with the builds (`source-early`); all other checks wait
for the build matrix (`source-deferred`) so generated inputs belong to this
worktree. New scanners default to deferred. Children report executed inventories,
and the parent rejects missing/incorrect coverage. Standalone `--scope source`
still runs the whole source suite. Cargo-link, committed-drift, binary and
cohab guards remain per architecture. A push stamp requires the shared source
result and every requested architecture's guards, as well as the other gate stages.
The underlying `check --scope source-early --strict`,
`check --scope source-deferred --strict`, `check --scope source --strict`, and
`check --scope arch --strict --arch X` commands are explicitly partial diagnostics,
not standalone full checks. They cannot combine with `--only` or baseline updates;
ordinary `check --strict --arch X` still runs both partitions.

**The parallel pre-land verdict** — the North Star "gate ×3 (all arches green)"
done concurrently instead of by a serial human loop. It submits every arch's build
concurrently (`--no-build` to skip), sharing compiler/linker execution slots, then launches each arch's
smoke as soon as its own build finishes. For any arch
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
  backend, so same-arch retries run in parallel too. **Half that pool is reserved
  for landing**: boots under a conductor's gate (`VIBEOS_JOB_CLASS=integration`)
  queue separately and take the reserved permits first; every other boot — a lane's
  own smokes, payloads, sweeps — fills only the rest, so the job that unblocks the
  fleet never waits behind it. `VIBEOS_LAND_BOOT_RESERVE` resizes it (0 disables);
  `host-status` shows it.
- **`cohab:<arch>` — fresh object coverage.** `tools/verify_cohab_
  coverage.py` asks each flipped C object what it *exports* (`llvm-nm`, after every
  `-D`, `#ifdef` and ABI decoration) and checks that something linking ahead of the
  static library defines it. Unlike standalone checks, it requires objects
  produced by this gate. Source analysis cannot answer: `cargo check` never codegens, so
  `global_asm!` is never assembled and `#[export_name]` never becomes a symbol.
  Eight msvcrt `__stdcall` stubs shipped as `___adj_fdiv_m16i@4@4` — `#[export_name]`
  STACKS with i386 PE decoration rather than replacing it — and built, linked and
  booted green while the C bodies stayed live and the flip did nothing. Needs both
  `--no-lint` and `--no-build` unset: it can only judge artifacts this run produced,
  and it refuses an archive older than the tree rather than report staleness as
  findings. Failure classifies `COHAB_FAIL` and blocks the stamp.
- Exit 0 only if **every** arch is ultimately green. The summary carries a
  per-arch `matrix` (verdict / classification / attempts / recovered / run_dir);
  each arch's own `summary.json` lives under the gate's artifact dir.
- Auto-push discipline unchanged: a ×3-clean-gate-green landing is still what
  authorizes a push — but drive it through `land` (below), not by hand.

### `land [--direct] [--drain] [--hold R] [--release] [--withdraw] [--evict R] [--cold] [--wait] [--retries N] [--clean] [--no-push] [--dry-run] [--skip-lint]`
**The way to land — it is a MERGE QUEUE.** With no flags, `land`:

1. publishes this lane's tip to the queue (`refs/vibeos/queue/<seq>` + a record
   under `$GIT_COMMON_DIR/vibeos-queue/`) and returns in ~2s — you do **not**
   merge master, gate, or wait;
2. if no other agent holds the land lock, *becomes the conductor*: merges every
   queued lane onto current `origin/master` in a dedicated integration worktree,
   gates **once**, and fast-forwards master to exactly the commit it gated;
3. writes a verdict per submission — `land-status --mine` (or your next `land`)
   shows it.

**N ready lanes cost ONE gate, and no agent blocks.** The conductor is elected by
the lock, so there is no daemon to supervise and nothing that can be "down".

- **Conflicts are bounced, never resolved** — rejected with the conflicting
  paths; the rest of the batch proceeds.
- **A red batch splits recursively**, landing any green half immediately, until
  the culprit is one submission — rejected with its classification + artifact
  dir. Innocent lanes still land. Batch cap `VIBEOS_QUEUE_BATCH` (default 4)
  bounds the worst-case bisect.
- **Guilty or framed?** Before rejecting a lone submission, the conductor gates the
  BASE alone (`origin/master`, nothing merged). If the base is red too, **nobody
  is rejected**: the submissions stay queued, the train HALTS, and the verdict
  says mainline is broken. This is not hypothetical — the very first real batch
  hit it (mainline failed to compile `fltmgr-rust` for i386) and, without this
  check, rejected a docs-only change for a build failure it could not have
  caused. The base gate is lazy and memoized per drain, so a green batch never
  pays for it. A base that fails **only** timing-sensitive stages (smoke,
  conformance) gets one warm re-gate before it can hold the line: on 2026-10-04 a
  spawn32 race and an arm64 BUGCHECK each held every lane on a master that later
  gated green. If the re-gate
  passes, the line is not held. A lone submission red only on the stages the base
  flaked on stays queued (`FlakyBase`), neither rejected nor landed.
- **A batch is never presumed green from its parts.** Two individually-green
  lanes can be red *together*; the queue catches that before master, where today
  the second lane lands on the first without ever gating the combination.
- **Crash-safe**: a submission is a ref + a JSON record; the conductor's claim is an
  flock that dies with it. Kill a conductor mid-batch and its submissions are still
  queued for the next one. A submission leaves the queue only when it has landed
  or been rejected — repeated work is possible, a silent drop is not.
- The **integration worktree** (`$VIBEOS_ROOT/vibeos-integration`) is created on
  first use and deliberately never auto-pruned: its warm build dirs + cargo target
  are why a batch gates in minutes, and every landing in the fleet would otherwise
  pay for a cold one.
- The spawned conductor **waits its turn** for the land lock rather than giving up, so
  a submission always gets drained even when a pre-queue lane is holding the lock via
  `--direct`. (It gave up instantly at first, which left submissions with no conductor
  — observed within minutes of shipping.) Nothing is blocked by it waiting, and a
  conductor that wakes to an empty queue exits in ~1s.
- **`check-tooling` runs in YOUR lane before the submission is published.** The
  conductor runs the identical command inside the batched gate, so this can never
  block anything the gate would have passed — it is the same verdict moved from the
  most expensive place in the system to the cheapest. It has to be: a red batch is 3
  builds + 4 boots of fleet load, and the lint that bounced a lane on 2026-07-26 was
  one line 101 characters long. Only a check that *ran and returned nonzero* blocks;
  if the preflight itself cannot run it warns and proceeds, because a broken
  preflight must never be able to wedge the fleet's ability to land. `--skip-lint`
  overrides (report every use). It lints the lane tip, not the tip merged with
  master, so it is an early filter and not a replacement for the gate. The gate's
  **source-lint partitions and host difftests** run there too, by the same rules and
  the same override — the difftests since a lane whose own `ucrt-difftest` could not
  link turned 17 of 18 batched gates red on 2026-09-25.
- `--drain` conducts without submitting; `--foreground` conducts inline instead of
  spawning; `--wait` blocks for your own verdict if you'd rather not poll.

### The train: pipelined gates (`VIBEOS_QUEUE_DEPTH`, default 2)
While one batch gates, the conductor merges the NEXT batch on top of it in a
second integration worktree (`vibeos-integration-2`) and gates that at the same
time. The second batch's verdict is about master + first + second, so it is
used only if the first landed whole; otherwise it is discarded and those lanes
stay queued, neither landed nor rejected. A second batch's merge conflict with
the first is likewise only rejected once the first is on master. Depth 1 is the
serial queue. Why (2026-09-26): one cold ~80-minute gate held nine lanes.

### Whose code conducts, and where its log lives
The lane that wins the election keeps the land lock but does not drain with its own
copy of the queue: it runs the drain as a child in `vibeos-conductor`, a detached
checkout it moves to origin/master first. A fix to the queue therefore governs the
very next drain, whether or not the electing lane has merged it. `VIBEOS_CONDUCTOR_PIN=0`
drains in-process (for testing queue changes before they land). Every conductor line
is also appended to `$GIT_COMMON_DIR/vibeos-queue/journal.log` (one rotation at 8 MB),
which survives the reboots that wipe `/tmp/vibeos-cli`.

When a lone submission fails, the base check re-gates origin/master after
`git clean -fdqx` (keeping only `.build-tools`, `.venv`, tool caches and the arm64
firmware), so master is judged without the previous submission's build dirs.

### `land --withdraw` — step out of line
Takes THIS lane's queued submissions out of the queue (verdict: `withdrawn`),
without touching anyone else's place. A submission already being gated —
`[GATING]` in `land-status` — is left alone: its merge is in the tree under test.
Resubmit with a plain `land`. Use it when you learn your lane is not ready, or
would gate cold; `--hold` is for stopping everyone.

### `land --evict REASON` — clear the line for a fix
Only under your OWN manual hold (`land --hold REASON --preempt` first). Kicks
every other lane's queued submissions back to them with verdict `EVICTED` and
REASON (e.g. "merge origin/master and resubmit"); anything a live conductor is
gating is left alone. Then land your fix with `--direct` and `--release`.

### Warm receipts, and `land --cold`
Every successful `vibeos-cli build` records a receipt for the working tree's
content on that arch. `land` refuses a lane whose tree has no receipt on all
three arches — the queue's gate builds cold and leans on the shared compiler
caches, so code no build has seen compiles from scratch there and can hold the
whole queue for hours. Build the three arches, commit what you built (a
docs-only difference is fine; a docs-only lane needs no build at all), and
`land` again. `--cold` overrides; report every use.

### `land --hold REASON [--preempt]` / `land --release` — stop the line
While a hold stands, no conductor starts a batch, `land` still **queues** your
lane (nobody loses their place, nobody retry-loops), and only the holding lane
may `land --direct`. `--preempt` also stops a *detached* conductor mid-batch; its
batch stays queued, and a `--direct` lander is never killed. `--release` lifts it
and wakes the queue. The hold shows at the top of `land-status` and `host-status`.

A red base holds the line **by itself**: when a train halts on `BASE_RED`, an
AUTO hold is pinned to that master sha, any lane may `land --direct` the fix, and
the hold lapses as soon as master moves. A `land --direct` that lands also wakes
the queue, so submissions never sit under `CONDUCTOR: none` afterwards.

Why (2026-09-25): mainline went red, and every new submission elected a
conductor that re-gated the same red base, all night. The load that generated
turned a 10 ms test-timing assumption into a second, unrelated red, and the fix
lane's gate queued behind the churn for hours. Stopping it took killing
conductors by hand, messaging every lane, and finally `wsl --shutdown`.

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

### `disk-status [--sizes]`

JSON report of filesystem headroom and live disk reservations. `--sizes` also
measures this lane's build/target directories and the shared compiler caches;
it reads metadata only and deletes nothing. CLI builds reserve 32 GiB and tooling
runs 4 GiB, leaving a 20 GiB headroom floor. Gate/land parents check headroom;
their children reserve independently. Host-wide policy lives at
`<HOSTLOCK_DIR>/disk/policy.json`; see the fleet migration plan for its schema.
Admission refusals are infrastructure failures, not code regressions. These are
cooperative estimates, not filesystem quotas: legacy lanes and other writers
remain unaccounted for. `retention --enable` can install bounded hourly cleanup
after a quiet-fleet compatibility check; `disk-status` reports timer activity.

### `host-policy [--apply | --rollback | --pause | --resume]`

Fleet-operator control, not a per-lane tool: with no flag it previews a drained-fleet
build trial; `--apply`/`--rollback` switch between three and two concurrent builds after
the fleet drains, and `--pause`/`--resume` stop and restart CPU admission around a cutover.
Every lane on the host feels it — do not flip it to speed up your own build.

### `host-status [--json]`

Shows active and waiting compiler/linker jobs across worktrees, plus legacy
reservations, boot slots and build-directory owners. New builds reserve no CPU
capacity while waiting for their directory. Individual tools share the host's
execution slots, preferring interactive work with 120-second aging. Queue and
direct-land gates identify their children as integration work.

`--json` includes commands, job class, owning build PID, artifact directory and
submitting launcher. Detached integration work stays visible after its agent
pauses. The old FIFO/shadow display applies only to legacy reservations, not the
new enforced job priorities. See
[host job scheduling](../../../docs/engineering/host-job-scheduling.md).

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

The old strict-check `cargo_link` PoC rebuilds have been retired: ntoskrnl,
ntdll and ntdll_vista are now normal Cargo-linked shipping targets. The regular
build, binary checks and three-architecture smoke gate exercise those links.
Strict lint retains committed-artifact drift checks and does not rebuild those
modules in a competing Cargo target directory.

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

**Breakpoint addresses on i386 come from `ntkrnlmp.exe`, not `ntoskrnl.exe`.** The fleet
boots the MP kernel (`[FLDR] load ...\ntkrnlmp.exe`) and the loader lists it under the
NAME `ntoskrnl.exe`, so `ntoskrnl.exe+RVA` in a banner is an ntkrnlmp RVA. Addresses taken
from `build-i386/ntoskrnl/ntoskrnl.exe` are a different link: the breakpoints are accepted
and silently never fire. Runtime address = `[KRNLBASE]` base + (nm address - 0x400000).
`gdb start`/`probe` attach the same `\TEMP` IDE disk smoke does (`vm.temp_disk_args`).

For MP debugging, prefix `gdb start` with `VIBEOS_QEMU_SMP=1` (or the desired
vCPU count). The launcher selects MP media, passes `-smp`, and records
`ntkrnlmp.exe` as the symbol image. One vCPU still selects the MP kernel.

For rare faults, `VIBEOS_QEMU_SMP=1 ./vibeos-cli gdb probe arm64 --script probe.py
--repeat 80 --timeout 150` starts a fresh VM at reset for each probe. The GDB Python
script gets `VIBEOS_PROBE_TIMEOUT` and `VIBEOS_PROBE_DIR`; it must use a bounded
continue and set `VIBEOS_PROBE_RESULT = {"hit": bool, "checks": positive_int}`.
Count actual probe coverage, not boots. Missing coverage or a script error stops
the soak with exit 2; a hit stops with exit 0; exhausted no-hit runs exit 1.
Each run retains the script, GDB output, kernel hash, result, and serial log under
`qemu/runs/<run-id>/`. Kernel or media changes stop the soak. `gdb stop` also
archives serial before removing VM staging.

**CRITICAL gotcha (baked into the engine):** connecting to QEMU's gdbstub *halts
the vCPUs*. So liveness (`status`/`ps`) checks the **Windows process + serial-file
growth**, NEVER a socket connect to the stub — a connect-probe freezes the guest
mid-boot (serial stuck at 0, PC parked in OVMF). Only `run-until`/`inspect` connect
(they intend to halt, then `detach` resumes). The detached QEMU persists across CLI
invocations (a separate Windows process); address it by run-id, kill by uid tag.

### `checkpoint {capture|run|inspect|verify}` — durable saved execution

Capture before a hot window, then rerun that window without booting again:

```sh
./vibeos-cli checkpoint capture --arch arm64 --smp 1 --name before-window \
  --until '[SMSS] _main entered'
./vibeos-cli checkpoint run before-window --repeat 100 --until 'shell ready'
./vibeos-cli checkpoint verify before-window --pause-seconds 5
```

Bundles live in `~/vibeos-checkpoints`, outside Windows temporary cleanup, with
pinned media and private writable attempt images. CPU topology is preserved.
`run --gdb` restores stopped for one debugger session. Failed windows preserve
logs, CPU state and a failure checkpoint when QEMU remains available. This is
checkpoint retry, not deterministic replay. Capture requires TCG and isolates
outside networking. See [the checkpoint guide](../../../docs/engineering/qemu-checkpoint-debugging.md)
for supported configurations, verification and debugger limitations.

### `host-clock [--seconds 65] [--fix-wsl]` — clock-step evidence

Samples wall time against monotonic time and saves the raw readings. A step over
50 ms fails. `--fix-wsl` disables competing Linux NTP only after proving Hyper-V
implicit synchronization is active; it never changes a native host's time setup.
This measures clock stability, not UTC accuracy. See [host clock notes](../../../docs/engineering/host-clock.md).

### `rr {record|replay|verify|list|info}` — deterministic re-execution of one boot

`rr record --arch arm64` boots once under QEMU's instruction-counted record mode
and keeps a self-contained trace under `~/vibeos-rr/`; `rr replay latest` re-runs
that exact boot instruction-for-instruction. A 1-in-60 crash you have recorded
reproduces every time.

**Do not stop a replay from the debugger.** `stepi`, attaching mid-run and even
a breakpoint HIT all abort QEMU with `Playback shouldn't have to iowait`. Stop it
with QMP `replay-break` at an icount and attach gdb to the already-paused guest
— verified working (registers read, QEMU survives). `rr replay` does not wire
`replay-break` up yet; the recipe is in the "Replaying with gdb" section of
`docs/engineering/qemu-record-replay.md`.

`rr verify <trace>` is the tool's own oracle: it replays and diffs the replay's
serial against the recording's, so you learn that the guest ran *the same boot* —
not merely that a VM ran. Reach for it whenever you doubt a trace. Note the
asymmetry in cost: recording adds about 2x to a boot, but a replay waits out the
guest's idle intervals in real time and runs minutes to tens of minutes. Record
freely; spend replays deliberately.

The interesting flag is `--shift` (icount shift). It is not a tuning preference:
the virtual clock costs `2**N` ns per instruction, which fixes the guest's
apparent CPU at `2**-N` GHz, and firmware/bootloader watchdogs are denominated
in that same virtual time. Too large and a healthy guest times out; too small
and every firmware delay loop costs 16x the real seconds. `rr info` reports the
value a trace was recorded at, and replay re-uses it — replaying at a different
shift feeds the recorded input stream to a differently-clocked guest.

Record/replay refuses rather than producing a trace that silently diverges:
no KVM, no `-smp N>1`, no NIC, no pre-existing `-icount`, native backend only.
`--smp 1` IS accepted and is the interesting case (the MP kernel on one CPU).

`rr prune --keep N` is the only thing that reclaims traces — a kmtest capture is
~1.4GB and `vibeos-cli prune` does not know the trace root exists. Dry-run by
default; `--apply` to act; `--name <trace>` to drop a specific known-bad one.

`rr breaks <trace>` prints the icounts a replay of it can legally be stopped at.
A replay runs the guest in the instruction chunks the recording wrote down, and a
QMP `replay-break` landing part-way through a chunk aborts the whole replay with
`Playback shouldn't have to iowait`. Chunk sizes span 1 to 2^28 instructions in a
single boot, so there is no rounding rule — `--near <icount>` snaps a target to
the next legal one and says how far into a chunk the unsnapped value sat.

`rr record --sample SECS` fingerprints the guest (registers + per-MiB RAM hashes,
the bytes kept deduplicated in the trace) every SECS of wall time; `rr diverge
<trace>` replays it, breaks at each fingerprint, and reports the last icount where
replay and recording agreed and the first that differs, down to the byte. A wedge is
where a divergence becomes visible; this finds where it happened.

Design, gdb recipe and the divergence log: `docs/engineering/qemu-record-replay.md`.

### `ps [--json]`
Every QEMU running **across all worktrees** + slot-pool usage. Run it first when
you're unsure what's live or why KVM is contended.

### `prune [--apply] [--self] [--idle-hours N] [--retain-gib N]`
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

`--retain-gib N` bounds reclamation: select eligible trees largest-first only
until retained private outputs fit N GiB. Its default idle window is 24 hours.
Protected trees can exceed the budget; they are never sacrificed to meet it.
Shared caches are outside this budget. Descendant mtimes count as recent activity.

Apply now requires an exclusive host maintenance lock and an exact pinned
cleanup owner at `<HOSTLOCK_DIR>/maintenance-owner`. Creating
`<HOSTLOCK_DIR>/maintenance-disabled` stops subsequent directory deletions.
Updated artifact-consuming CLI commands hold shared leases on that lock.
**Do not enable the owner until every lane is updated and older/raw jobs are
drained**; they do not honor the lock. No owner is enabled automatically.
See [storage/retention rollout](../../../docs/engineering/rust-output-retention-plan.md).

### `retention [--enable | --disable | --run]`

Install or run hourly generated-output retention from one pinned checkout.
Defaults: 100 GiB fleet-output budget, 150 GiB filesystem headroom, 24 idle hours.
`--enable` accepts `--retain-gib`, `--min-free-gib`, and `--idle-hours` and requires
an idle fleet with compatible maintenance admission in every registered worktree.
It includes idle integration scratch. Active and recent outputs remain protected,
so budgets may remain unmet. `--disable` stops scheduling and sets the global
maintenance kill switch. Plain `retention` prints its configured policy.
See [measurement and rollout](../../../docs/engineering/disk-optimization.md).

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

### `difftest [--only NAME[,NAME]] [--case NAME ... --test-target T]`
`--case` (repeatable, needs exactly one `--only` crate) runs only those exact cases, and
fails unless exactly that many were graded — only crates whose `main` honours `--exact`
(today `cnt-difftest`) support it. For code no crate covers, `rustconv difftest-contract`
generates the harness from a JSON contract instead (`tools/rustconv/data/difftest/README.md`).

Runs every **host differential harness** — the `*-difftest` crates, which compile
the genuine ReactOS C as a host TU with its public symbols renamed `ref_*`, link
that beside the shipping Rust, and fuzz the two for parity in one process. No
QEMU, no `build-<arch>`; ~20s for the whole set on a warm target dir. This is the
only oracle that grades a translation against *the code it replaced* rather than
against someone's reading of it, so `gate` runs it and it gates the push stamp.

Two invariants it enforces, both from real breakage:

- **One cargo invocation per crate.** `cnt-difftest` pulls `cnt-rust` with
  `features = ["cabi"]` and `msvcrt-difftest` deliberately does not (it grades
  msvcrt-crt-rust's own wrappers). In one shared build, cargo's feature
  unification switches `cabi` on for both and cnt-rust's 208 `#[no_mangle]`
  shims collide with the wrappers — `duplicate symbol: mbtowc`. **`cargo test
  --workspace` cannot run these and never could**; that is a large part of why
  nothing ran them.
- **Exit 0 with zero cases compared is a FAILURE.** Every crate emits
  `test result: ok. 0 passed` for its empty lib/doc-test targets, so a harness
  whose cases all vanished still exits 0. The stage reports `graded NOTHING` and
  fails: a suite that cannot fail is worse than no suite, because green stops
  anyone looking again.

Crates are discovered from the `[workspace] members` list, never hardcoded — add
`foo-difftest` to the workspace and it is covered. `--only` restricts the set and
**refuses an unknown name** rather than silently grading a subset.

**A harness may declare its own `--target`.** `msvcrt-seh-difftest` grades
`except_i386.c`, which lives inside `#ifdef __i386__` and whose Rust counterpart
is `#![cfg(target_arch = "x86")]`, so it only exists in a 32-bit binary; cargo
builds one crate for one target per invocation, so that crate says so in its own
manifest:

```toml
[package.metadata.difftest]
target = "i686-unknown-linux-gnu"
```

Read from the manifest for the same reason the crate set is: a driver carrying
the list would silently build the next such harness for the host, where every
`#[cfg]` in it compiles to nothing and it grades zero rows. It needs
`rustup target add i686-unknown-linux-gnu` and a 32-bit libc, and it fails loudly
without them rather than skipping.

**The `drift:*` stages run first, and they check the ORACLE'S OWN INPUTS.**
msvcrt-difftest force-includes seven generated headers — the derived CRT/Win32
constants, the two-compiler layout guardian, and a prototype set per reference TU
— and each has had a `--check` mode since the day it was written that nothing
ran. So a PSDK header that moved a constant, or a reference `.c` whose signature
changed, stayed silent until some row happened to reach it; and a row that
reaches a stale constant does not report "stale", it reports a **divergence in
the Rust**. Wrong file, hours later. These stages re-derive each artifact and
refuse a stale one with `DIFFTEST_STALE_ORACLE`, before the harnesses run, so the
failure names the header and the command that fixes it.

They live here rather than in `check` because re-derivation needs the pinned
cross-compiler *and* `build-amd64` — the prototypes come from the built object,
not the source text, since `_MSVCR_VER` opens and closes whole families. `check`
runs on a fresh worktree with no build dir; `difftest` already hard-requires one.
The artifact set is discovered from `refstub/msvcrt_*_protos.h`: **the header's
existence is the registration**, so a new reference TU is covered the moment it
lands. That set went three → five → seven in two months, once per TU.

`drift:layout:seh-i386` is the eighth and the only one keyed to `build-i386` —
so `difftest` now wants **both** build dirs, and says which is missing rather
than skipping. It
re-derives msvcrt-seh-difftest's transcription of the i386 exception structures
from the real headers. Everything that harness grades is a walk **by offset** —
32-bit SEH is a linked list plus a scope table indexed by trylevel — so a field
that moves does not fail there, it regrades correct Rust as wrong.

---

## The artifact contract (how to read results without flooding context)

Every run prints a bounded summary and writes a durable dir. **Read
`summary.json` for the machine-true verdict; grep `full.log` for detail.** Never
scrape stdout for the result — stdout is intentionally lossy.

```
/tmp/vibeos-cli/<ts>-<unique>-<cmd>-<arch>/
  summary.json     verdict, classification, per-stage status, salient lines, artifact paths
  full.log         raw stage output under === headers ===
  commands.jsonl   every subprocess invoked (cmd, cwd, exit, seconds)
  boot*/screen.ppm, serial tails, debugcon — on failure
```

The unique component prevents concurrent runs from sharing an artifact directory,
even when they start in the same second in different worktrees.

Exit code: `0` for OK/WARN, `1` for FAIL. Stdout glyphs: `✓` ok, `⚠` warn,
`✗` fail, `·` skip.

For performance, read `summary.json.command_timings`: each subprocess has a
`stage`, `start_seconds` offset from run start, `seconds`, and `exit`. The same
fields appear in `commands.jsonl`. These are command timings, not verdict-stage
timings; concurrent durations overlap and must not be summed as elapsed time.
They include subprocess-internal waits, but not fleet acquisition outside the
subprocess. Rust cache statistics also preserve `non_cacheable`,
`non_cacheable_reasons`, `cache_errors`, and `compilations`; zero requests is not
evidence of a cache miss. See [build reuse plan](../../../docs/engineering/build-reuse-plan.md)
for the remaining measurement and invalidation work.

---

## Typical loops

```bash
# targeted source-scanner investigation (explicitly partial; never a landing gate)
./vibeos-cli check --arch amd64 --strict --only scan_orphan_modules,verify_match_consts

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
| Lint gate (NEW-vs-baseline) + binary checks | `vibeos-cli check [--strict]` |
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
