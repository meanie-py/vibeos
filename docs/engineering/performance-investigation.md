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

### F4b. [FIXED 2026-07-06] host-tools BUILD_ALWAYS phantom-dirtied every warm build — no-op ninja 87s → 1s
F4's "ninja ~2s" stopped being true as cargo modules accumulated: by
2026-07-06 a literal no-op `ninja livecd` took **87s**. Root: upstream's
`BUILD_ALWAYS TRUE` on the host-tools ExternalProject omits the build stamp,
so the phony host-tools chain was dirty on EVERY build; the dirt reached
~53k graph nodes, and the ~40 cargo-module custom commands (which re-RUN on
phony-dirty deps — restat can't save a custom command) each re-invoked cargo
concurrently, serializing on cargo's package-cache lock at 33–51s per edge.
Fix (`sdk/cmake/host-tools.cmake`): build-once + file-level
`ExternalProject_Add_StepDependencies` on the sub-build's real source tree;
BUILD_BYPRODUCTS + restat prune downstream when tools come out unchanged.
Verified: no-op ninja 87s → 0–1s; tool-source edit still propagates (6s
targeted rebuild); warm `vibeos-cli build` ~40s, now genuinely lint-dominated
(verify_ffi 17s is the top stage). This also invalidated the "cargo module
links are 35s each" reading of the ninja log — those were phantom re-runs.

### F4c. Link speed is DEAD LORE — measured 2026-07-06
The remembered "link is the worst offender" no longer holds (canonical-path +
shared-sccache work likely fixed it): kernel cargo link (reactos-link → lld)
is seconds; a warm no-op `cargo build -p ntoskrnl-cargo` is 5.7s; a REAL
one-line kernel .rs edit → rebuilt+relinked ntoskrnl.exe in **12.6s**; a tiny
cargo module (beep) warm no-op is 0.12s, forced re-link 1.4s. The cold-build
critical path is the single-crate `ntoskrnl-rust` rustc (~400s class) — the
module-factoring lane is the right attack. Cold-adjacent facts: sccache marks
`crate-type: staticlib` non-cacheable (38/396 requests last build), so the
~50 per-driver staticlib shells never cache (keep bodies in rlibs); those 50
per-driver `rust-*-target` dirs also each recompile the ntkrnl-sys world
(~90MB each, ~4.5GB/arch) — consolidation into the shared workspace target
(as the cargo modules already do) would cut cold time and disk.

### F5. [FIXED 2026-07-06] Gate was a 3-barrier pipeline; per-arch pipelining landed
Was: builds (serial, ~130s warm) → lint (parallel barrier) → smokes
(parallel); nothing smoked until ALL arches built and linted, ~390s wall.
Now (`gate.py`): each arch's smoke + strict lint + the amd64 net lane launch
the moment ITS build lands (amd64 builds first — it feeds two lanes); builds
stay serial among themselves (exclusive cpuheavy lease). Safe because
windows-host boots throttle on the separate `bootcpu` pool (Windows
scheduling domain) — build ∥ boot is the hostlock contract.
Measured 2026-07-06 (with F4b's no-op-ninja fix compounding): full gate
green ×3+net+stamp in **73s** on an unchanged tree; a real-edit warm gate
lands in the 2–4 min class (vs 8–13 min at the start of the day).

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
   hand-encoded-UTF-16 class as field manual §; **[SWEPT 2026-07-06]** the
   owed workspace sweep converted ~140 sites in 121 files to aligned `w!()`
   arrays (3 mechanical passes + hand fixes for phase-broken strings: SAM
   %08lX ×5, msgina Control key, cmboot System32\Drivers\, partmgr VOLMGR).
   Full class writeup: field manual "ANSI byte string cast to a WIDE
   pointer". Post-sweep the oracle grep returns zero non-comment hits.

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
3. **[FIXED 2026-07-06] Registry hive writes on MININT livecd**
   ("Failed to write the primary hive" ×4/boot, 5–9s gaps): root cause was a
   TRANSLATION bug, not upstream-inherited as first classified. C
   CmpInitializeHiveList: `if (CmpShareSystemHives) HHiveFlags |= HIVE_VOLATILE`
   (marks all machine hives volatile on MININT so CmpLoadHiveThread and every
   write path skip them). The Rust translation tested `CmHive2 != null`
   instead — always false at that point (CmHive2 is set later BY the load
   threads), so MININT hives were never volatile and CmpLoadHiveThread spent
   ~5.5s failing to write CD-backed hives. Fix: cmsysini.rs restores the
   CmpShareSystemHives condition. Verified: stall gone, amd64 log end
   23.1s → 17.4s. (The upstream `CmpNoWrite = FALSE` clobber in
   CmpInitializeHiveList is real but harmless once volatile marking works —
   kept faithful to C.) Bug-class: mistranslated condition variable — same
   family as the or-pattern/`!DBG`-stub entries in the field manual.
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
1. **[DONE 2026-07-06] Strip DWARF from CD-staged binaries — landed as
   `tools/strip_cd_media.py` + a pre-mkisofs pass in the livecd target**
   (boot/boot_images.cmake). Rewrites the graft list to point PE files at
   debug-stripped twins (llvm-objcopy --strip-debug, cached by mtime+size in
   build-<arch>/boot/stripped/, PE CheckSum recomputed — validated against
   pefile). Measured: 675 PEs, 466MB→172MB grafted; ISO 508MB→214MB; **amd64
   kernel entry 24s→6.3s, i386 →7.6s; amd64 explorer entry 32s→15.6s guest
   time**. Caveats resolved: ReactOS `LdrVerifyMappedImageMatchesChecksum` is
   `#if 0` (always TRUE) and all build PEs carry CheckSum=0 anyway; provenance
   hashes build-dir binaries + whole media separately, never assumes CD bytes
   == build bytes. Build tree keeps full DWARF (sym tools unaffected).
   Follow-ups: bootcd/regtest lst + reactos.cab not yet stripped (installer
   path, not smoked); arm64 ramdisk staging could take the same pass.
   Historical detail preserved: wdf01000.sys is 13.5MB of which ~12.5MB is
   .debug_* (code+data ≈ 0.8MB); wdf01000 cannot be demoted from boot-start —
   cdrom.sys is KMDF (wdf01000_reg.inf).
2. **[DONE 2026-07-06] The "slow transfer" was freeldr's own 4KB read
   chunking, not PIO bandwidth.** `UefiInitializeBootDevices` set
   `DiskReadBufferSize = EFI_PAGE_SIZE` — every 4KB of the ~35MB boot set
   paid a full firmware ReadBlocks/ATAPI round trip (~9000 calls/boot),
   which is what capped mid-size files at ~4MB/s and looked like "2.2MB/s
   PIO". Fix: 64KB read buffer (uefidisk.rs; 1MB HANGS OVMF's ATAPI path at
   the first read — bisected). Measured: amd64 kernel entry 6.1s → **2.3s**,
   ntoskrnl reads at 46MB/s, wdf01000 at 103MB/s, serial log end 15.4s.
   arm64 uses the same uefidisk path (RAM-backed FAT) and benefits too.

## Verified wins so far (timed smokes, amd64)

| | baseline | now |
|---|---|---|
| boot to explorer (guest, amd64) | 36.5s | **13.0s** |
| pre-kernel phase (amd64) | ~24s | **2.3s** (DWARF strip + 64KB reads) |
| amd64 serial log end (boot complete) | ~40s | **15.4s** |
| arm64 boot to explorer | ~40s | **11.8s** (EDK2 firmware now 5.9s of it) |
| livecd.iso | 508MB | **214MB** |
| SCM service starts (DcomLaunch etc.) | 31s timeout each | connect=0 instantly |
| MININT hive-write stalls | 5–9s ×4/boot | gone |
| freeldr menu | 5s | 0 |
| settle | 30s | 15s |
| amd64 smoke incl. net (gate lane) | ~200s serial | net runs parallel |
| warm full gate | 664s median | **~120-130s** |

Remaining boot profile (amd64, 15.4s): EDK2+freeldr 2.3s · kernel init 3.9s
(1.36s = uniata secondary-IDE probe, the top single remaining gap) · csrss→
explorer ~6.8s (flat ~0.5s gaps: comctl32 actctx, winsrv load) · post-marker
tail ~2.4s. arm64: EDK2 5.9s dominates; post-shell TDI self-test CONNECT
timeout (0xC00000B5, 9.2s, post-marker — flagged for the net lane).

**[DONE 2026-07-06] uniata IDE-probe gap: 2.35s → ~120ms.** Attributed with
permanent `[UNIATA]` DbgPrint phase markers (FindBusMasterController /
FindDevices / CheckDevice / SoftReset / IssueIdentify / HwInitialize —
uniata-rust, live; mirrored in the out-of-build C). Two VM-blind waits in the
(upstream-inherited) UniATA code, both fixed by gating on the driver's own
`g_opt_VirtualMachine` detection (QEMU is detected via PCI subsys 1af4:1100):
1. `AtapiSoftReset` waited up to 100ms for BUSY to *assert* after ATAPI
   RESET — QEMU (same IDE lineage as Bochs, which already had the `i=100`
   shortcut) completes the reset instantly and never asserts BUSY, so all
   10000×10µs iterations burned every time: ~167ms real under TCG × 12 soft
   resets/boot ≈ 2.0s. Fix: extend the Bochs shortcut to `VM_QEMU`.
2. `AtapiHwInitialize__` ran an unconditional 500ms (5000×100µs, ~880ms real
   under TCG) "get the device ready" mechanical-settle stall per ATAPI lun.
   Fix: skip when `g_opt_VirtualMachine > VM_NONE`.
Verified: FindDevices→"Found device" now 4ms; explorer entry 13.0→~10.1s.
Class writeup: field manual "Driver hardware-settle delays are pure loss in
a VM". i386 shares the same driver+path; arm64 has no IDE.

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
