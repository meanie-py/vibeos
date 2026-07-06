# Performance Investigation — where the time actually goes

Status: LIVING DOC (started 2026-07-05, branch `performance-investigation`).
Method: measure first, theorize second. Every claim below carries a number and
where it came from. Baseline corpus: 800+ `vibeos-cli` run summaries in
`/tmp/vibeos-cli/*/summary.json` (2026-06-26 → 2026-07-05), plus targeted
experiments on this branch.

## TL;DR (evolving)

Both axes turned out to be full of *specific, fixable* diseases rather than
"TCG is slow":

- **OS axis:** the kernel→desktop chain is fast (~9s); the wall-clock went to
  a silent 27s freeldr load phase (5s of which was a literal menu countdown),
  a dead-on-arrival svchost that costs a 31s SCM timeout every boot (root
  cause: an OS-wide LCMapString translation bug, fixed here), doomed hive
  writes on read-only media, and an unpainted taskbar.
- **Dev-loop axis:** the harness *around* the guest dominates — fixed settle
  waits, a net suite serialized behind the boot it doesn't depend on, the
  same lint arsenal run 6× per gate, serialized builds, and a build graph
  that silently didn't rebuild 16 Rust crates on source edits (false-green
  factory; fixed here).

## Measured baselines (medians over the run corpus)

| Metric | i386 | amd64 | arm64 |
|---|---|---|---|
| `smoke` wall (green) | 82s (74–90) | ~200s (78–484) | 38s (34–38) |
| `gate` stage smoke | 82s | 200s | 38s |
| `gate` stage build (warm) | 155s* | 65s | 78s |
| `gate` stage lint (`check --strict`) | 53s | 52s | 53s |
| desktop_settle (fixed post-marker wait) | 30s | 30s | 15s |

Gate wall median **664s** (n=67; best ~390s green: builds 128s serial + lint
~60s + smokes 204s parallel).

\* i386 build median inflated by cold outliers; same-day warm ~44s.

## Findings

### F1. amd64 "smoke is slow" is NOT a slow amd64 — it's two serial boots
`vibeos-cli smoke --arch amd64` runs the boot smoke, then `_net_stage`
(tests/qemu/cli/smoke.py:247) boots a **second full VM** (amd64-uefi-net) and
runs the 15-test net suite: measured 111–152s. That's ~60% of amd64 smoke
wall. The plain amd64 boot is ~80s — same ballpark as i386.
→ Opportunity: run the net lane concurrently with the plain boot (gate already
over-submits safely), or as a 4th parallel gate lane. Saves ~70–115s off the
gate critical path.

### F2. Every green x86 boot pays a fixed 30s "desktop settle" (arm64 15s)
`_settle_watch` (smoke.py:62) always burns the FULL `desktop_settle_seconds`
window — no early exit when `shell ready` already arrived. Purpose is
screenshot aesthetics (taskbar renders ~15s post-marker). 30s is ~37% of
i386's 82s smoke wall, paid again by the net-config boot.
→ Opportunity: exit the settle early on a taskbar/tray render marker (or
`shell ready` + short grace), or right-size 30→15s to match the documented
render delay.

### F3. The scanner arsenal runs 6× per gate (2× per arch)
Every `vibeos-cli build` runs the full lint arsenal warn-only (~21–40s per
run; measured medians: verify_match_consts 4.4s, verify_tables 3.6s,
verify_signatures 2.4s, verify_ffi 2.1s median / **17s cold**,
verify_constants 2×2.0s, sacred_primitives 1.5s, + ~8 more). Gate then runs
`check --strict --arch X` per arch — the same suite again (~52s each,
parallel). Most scanners are **arch-independent source scans** repeated 3×.
→ Opportunity: skip the warn-only arsenal inside gate builds
(gate lints anyway), and split `check` into source-scans-once +
binary-lints-per-arch. Saves ~60–120s serial per gate.

### F4. Warm no-op build ≈ 42s, of which ninja is ~2s
From commands.jsonl of a no-change amd64 build: ninja livecd 1.2s + primed
targets 0.6s; everything else is scanners + symbol indexing + stats. The
"build" cost people feel in the loop is ~95% lint/verify tooling.

### F5. Gate is a 3-barrier pipeline; per-arch pipelining would cut ~40%
Gate = builds (serial, ~130s warm) → lint (parallel barrier, ~60s) → smokes
(parallel, wall = amd64 ~200s). Nothing starts smoking until ALL arches built
and linted. A green gate's ~390s has a per-arch critical path of only ~250s
(amd64: build 43s + boot 85s + net 115s).
→ Opportunity: launch each arch's smoke the moment its build finishes
(amd64 first), run lint concurrently (it gates the stamp, not the smoke).
Modeled gate wall: ~390s → ~230s warm.

### F6. Fresh-worktree provisioning is a near-cold build per arch
Object files/rlibs are per-worktree; only ccache/sccache are shared. First
`build --arch amd64` in this worktree: >10min (vs 43s warm). ×3 arches to be
gate-capable. (Measurement in progress.)

### F7. Staging-dir litter: 29GB / 70 stale dirs in /mnt/c/temp/vibeos-qemu
Each boot stages media (508MB livecd.iso copy ≈ 3.9s — acceptable) into a
per-run dir that is never cleaned. Not a wall-clock cost, but 29GB of disk.
→ Opportunity: reap stale staging dirs (they're pid-keyed; dead-pid dirs are
safe to delete).

## OS-runtime axis — measured boot anatomy (amd64, timestamped)

First timestamped boot (2026-07-05, `VIBEOS_SERIAL_TIMESTAMPS=1`, amd64 UEFI
livecd, windows-host TCG):

| t (s) | event |
|---|---|
| 0.3 | firmware → freeldr |
| 0.34 | freeldr "Switching stack..." — **then 27.2s of SILENCE** |
| 27.5 | kernel first print (ACPI banner) |
| 30.7 | IoInitSystem done |
| 32.2 | csrss | 
| 34.4 | winlogon |
| 36.5 | explorer `_tWinMain` (boot marker); taskbar child controls at +0.9s |

**The OS kernel+usermode chain is fast (~9s). 75% of boot is the freeldr
"silent gap".** Reproduced at 27.15s on the net-config boot — deterministic.
Decomposition so far: **5s of it was the freeldr boot-menu countdown**
(`livecd.ini TimeOut=5` — set to 0 on this branch; arm64's qemu/freeldr.ini
always had 0). The remaining ~22s = ISO load of kernel/HAL/drivers/hives
(+ early kernel init) — now instrumented with `[FLDR]` per-file/per-phase
serial stamps (winldr.c + livecd.ini `Debug=` line; no-op when debug off).

### OS bugs found wearing a "slow" costume

0. **svchost/DcomLaunch 31s-per-service SCM timeout — ROOT CAUSE (fixed):**
   advapi32-rust `ScConnectControlPipe` queried the `ServiceCurrent` registry
   default value with `Name = b"\0" as PWSTR` — a 1-byte ANSI string cast to
   wide (C: `L""`). As WCHARs that's a garbage non-empty name, the REQUIRED
   query failed, `StartServiceCtrlDispatcherW` returned FALSE, and **every
   svchost exited 0 within 0.4s of spawn** → SCM burned its full ~31s
   connect timeout per hosted service on every boot; DcomLaunch/RPCSS have
   been dead since the translation. (Diagnosed via a loud svchost wmain
   print — cmdline/options were perfect, pinning the dispatcher.) Same
   hand-encoded-UTF-16 class as field manual §; **12+ more latent instances
   found** by `grep -rnE 'b"[^"\\]{2,}\\0"\.as_ptr\(\) as
   (PWSTR|PCWSTR|LPWSTR|LPCWSTR|\*const u16|\*mut u16)'` in samsrv-rust
   (many), msgina ("Shutdown Setting"), winsrv harderror/tuiterm,
   usbaudio-rust — OWED sweep (off boot path, silently wrong at runtime).

1. **LCMapStringW(LCMAP_LOWERCASE/UPPERCASE) has failed OS-wide since the
   2026-06-14 kernel32 locale port.** Translation bug in `lcmap_string`
   (kernel32-rust locale.rs): C `case NORM_IGNORENONSPACE: case
   NORM_IGNORESYMBOLS: case A|B:` became Rust or-pattern `A | B | 0`, whose
   literal `0` swallowed the plain case-mapping path → ERROR_INVALID_FLAGS
   for every LOWERCASE/UPPERCASE map. Loudest victim: svchost
   (`SvchostCharLowerW failed`). FIXED on this branch. (Bug class to watch
   in rustconv output: C bitwise `case A|B:` → Rust or-pattern.)
2. **DcomLaunch svchost dies at spawn → SCM burns its full ~31s connect
   timeout, every boot, on i386 AND amd64** (t=35.7 spawn, t=35.9 svchost
   exit, t=67.2 `connect result=1053`). Serializes/delays every subsequent
   service; RPC/DCOM absent for anything that needs it. The cdrom [GAP] 21s
   "stall" is this wait seen from the disk. Whether the LCMap fix cures the
   svchost death is the first post-fix check.
3. **Registry lazy-flush attempts real hive writes on MININT livecd**
   ("Failed to write the primary hive" ×4/boot, 5–9s gaps): `CmpNoWrite`
   should be TRUE on MININT (`CmpNoWrite = CmpMiniNTBoot` in CmpCmdInit) —
   flush attempts imply the gate isn't holding on some path. OPEN.
4. **Taskbar still unpainted 30s after creation** (screenshot evidence) —
   the known render-completeness issue; controls exist at marker+0.9s but
   paint doesn't complete. Separate lane (ui-render-completeness doc).
4b. **16 cargo-cohab crates never rebuilt on source edits** (build-graph bug,
   dev-loop but correctness-critical): their cmake `add_custom_command`
   DEPENDS listed only `Cargo.toml` + a hand-picked file or two, so ninja
   considered the `.a` fresh after any edit to the other `.rs` files — cargo
   was never invoked. Bit this investigation directly (the diskwmi.rs fix
   linked stale twice). Fixed by scripted sweep to the repo's established
   `file(GLOB_RECURSE <X>_SRCS .../src/*.rs)` idiom (acpi, pl011, disk,
   atapi, pciidex, mountmgr, scsiport, wmilib, conutils, cport, dnslib,
   arbiter, csq, usbdex, lsalib, skiplist); the other 20 cargo-cohab
   CMakeLists already globbed correctly.

5. **132 `unreachable pattern` rustc warnings across the workspace** — the
   LCMapString bug's dead `0 =>` arm was one of them, warned at every build
   and ignored (warn-only, buried in build spew). Each of the 132 is a
   candidate for the same swallowed-case class (msafd-rust dllmain alone has
   dozens). OWED: sweep them; consider promoting unreachable_patterns to
   deny in translated crates once the true-positive rate is known.
6. **i386 COLD build was link-broken on master** (`disk-rust/diskwmi.rs`
   declared cdecl shims `extern "system"` → `_REVERSE_BYTES@8` undefined;
   warm builds masked it via the stale linked disk.sys). FIXED on this
   branch.

## Freeldr load phase — SOLVED decomposition (2026-07-06)

With the restored freeldr debug channel + `[FLDR]` stamps, the x86 "silent
gap" is fully attributed (amd64, TCG, per-file):

- **Every file loads at a uniform ~2.2 MB/s regardless of size** → the CD
  path is per-byte transfer-bound (ATAPI PIO via OVMF Block I/O under TCG),
  NOT per-call overhead. ~45MB of boot files / 2.2MB/s ≈ the whole ~20.5s.
- Top items: **wdf01000.sys 13.2MB = 6.4s (29% of the gap!)**, ntoskrnl.exe
  10.4MB = 4.6s, acpi.sys 3.4MB = 1.6s, cdrom.sys 2.4MB = 1.1s; system hive
  0.15s; everything else <1s.
- arm64 loads the equivalent set from a RAM-backed FAT in 4.8s (its gap);
  its extra costs are EDK2 firmware 6s (TCG) instead.

Ranked fixes:
1. **Demote wdf01000/wdfldr from boot-start** if no boot-critical driver is
   KMDF (instant −6.8s on every x86 boot; verify nothing in the boot set
   needs WDF before PnP).
2. **Attack the 2.2MB/s transfer**: check whether OVMF is stuck in ATAPI PIO
   (vs UDMA) on the i440fx IDE path; a DMA-capable attach (or a QEMU option)
   would multiply throughput. Guest-side constraint: uniata must still see
   the CD for the livecd session (that's why the CD is IDE, not AHCI).
3. **Debug-binary size**: the 10–13MB debug PEs pay real load time; consider
   stripping non-essential debug sections from CD-staged copies of the
   biggest files (keep symbols out-of-band for the debug toolkit).

## Verified wins so far (timed smokes, amd64)

| | baseline | now |
|---|---|---|
| boot-only smoke wall | ~82s | **56s** |
| boot to explorer (guest) | 36.5s | 32.0s |
| freeldr menu | 5s | 0 |
| settle | 30s | 15s |
| amd64 smoke incl. net (gate lane) | ~200s serial | net runs parallel |

## Older section — measurement in progress

Tooling added on this branch:
- `VIBEOS_SERIAL_TIMESTAMPS=1` — the smoke live-log tee now stamps every line
  with elapsed seconds since first serial byte (tests/qemu/serial_client.py).
- `tools/boot_phase_report.py` — buckets a stamped live log into boot phases
  (firmware → freeldr → kernel → SMSS/csrss → winlogon → explorer) and lists
  the largest inter-line gaps (silent stalls).

Open questions being measured:
- Why is a green x86 boot-to-marker ~2× arm64 (~50s vs ~20s) under the same
  TCG backend? Suspects: OVMF firmware phase, freeldr reading a 508MB ISO9660
  over emulated IDE, per-phase kernel stalls.
- Serial chatter cost: boots emit 33–91KB of DbgPrint over a polled 16550;
  quantify per-byte cost under TCG before blaming it.

## Dead ends / disproven folklore (so far)

- "amd64 is the slow arch" — false; its *smoke command* is slow (F1), the
  boot itself is i386-class.
- 508MB ISO copy to /mnt/c per boot — measured 3.9s, not a leading cost.
