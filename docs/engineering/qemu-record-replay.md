# QEMU record/replay — deterministic re-execution of a vibeos boot

`./vibeos-cli rr` records one boot and replays it instruction-for-instruction,
as many times as you like, with gdb attached.

It exists because of a class of bug we have no other tool for. The arm64 MP boot
faults at `PC=0` about 1 boot in 60. Months of soaking established the *rate* and
nothing else: every boot that crashed was gone by the time anyone could look at
it, and every attempt to look changed the timing. Record/replay turns the crash
into a file. Once you have the recording, the bug stops being rare — it happens
every single time you open it, at the same instruction, with the same memory.

**The tool is the deliverable.** Any particular bug it catches is a bonus.

---

## Quick start

```bash
./vibeos-cli build --arch arm64 --kmtests        # media the sweep needs
./vibeos-cli rr record --arch arm64              # one boot, kept
./vibeos-cli rr list                             # what you have
./vibeos-cli rr info latest                      # will it still replay?
./vibeos-cli rr replay latest                    # that exact boot, again
./vibeos-cli rr replay latest --gdb              # ... halted at instruction 0
./vibeos-cli rr verify latest                    # did it re-execute identically?
```

`verify` is the one to reach for when you doubt a trace: it replays and diffs the
replay's serial against the recording's, so you learn that the guest ran *the
same boot* and **how far it got** — not merely that a VM ran. Both halves
matter. A trace that quietly diverges is worse than no trace, because you debug
the divergence thinking it is the bug; and on current evidence a replay often
stops partway, which `verify` reports rather than letting you assume otherwise.
Read the first entry of the divergence log before planning around a replay.

Traces live under `~/vibeos-rr/` (`$VIBEOS_RR_ROOT`). They are **not** in `/tmp`:
a trace costs a full boot to make and is worth more than the boot that made it.

## What a trace is

```
~/vibeos-rr/20260913-0142-arm64-kmtest/
  manifest.json     arch, commit, QEMU version, icount shift, the exact argv,
                    the pinned VIBEOS_* env, and where each image came from
  replay.bin        the recorded non-deterministic input stream (~10MB for an
                    arm64 boot-to-desktop)
  media/            the images the guest boots from — qcow2, private to the trace
  pristine/         media/ frozen at the END of the recording (see below)
  record-serial.log the transcript `verify` diffs a replay against
```

Nothing in a trace points into a build tree, so a trace outlives the tree that
made it.

`pristine/` is frozen **after** the recording, not before, and the distinction is
the whole correctness of replay: `rrsnapshot=init` writes a vmstate snapshot
*into* the qcow2 during the run, and replay loads it. Media frozen before the
boot would not contain that snapshot. The guest's own writes are not a problem —
loading `init` reverts the disk to the recorded starting point, which is exactly
what it is for. Every replay restores `media/` from `pristine/` first, so replay
#2 sees what replay #1 saw.

## How it works, and why the command line is rewritten

`-icount shift=N,rr=record,rrfile=T` runs the vCPU on a virtual clock derived
from instruction count and logs every non-deterministic input — device reads,
interrupt deliveries, clock reads — into `T`. `rr=replay` feeds those same inputs
back in the same instruction slots.

Two things must hold, and neither is achievable by appending flags:

1. **Block I/O must be deterministic.** A real disk completes asynchronously, so
   a completion lands in a different instruction slot every run. The `blkreplay`
   block filter parks completions and releases them at the recorded virtual
   time. *Every* guest-visible drive has to be wrapped — one missed drive is
   enough to ruin a trace, and nothing tells you at record time.
2. **The guest must start from byte-identical media.** Recording mutates media
   (efivars, the scratch disk, the boot overlay), so a trace owns private copies.

So `tests/qemu/cli/recordreplay.py:transform()` rewrites the whole launch
command: it wraps each drive as `blkreplay` in front of a qcow2 backend,
repoints firmware pflash at the trace's own copy, drops `readonly` (the overlay
absorbs writes, and `rrsnapshot=` needs somewhere to put the vmstate), forces
single-threaded TCG, and appends the `-icount` clause.

The filter deliberately **keeps the id the device refers to** (`-device
virtio-blk-device,drive=scratch` must keep resolving) and the raw backend takes
`<id>-img`. Getting that backwards is not a build error: QEMU starts happily and
the device is attached to the *unwrapped* image.

## What it refuses, and why refusing is the feature

A bad rr config does not error. It produces a trace that diverges halfway
through a replay, and the divergence message names an event number rather than
the config that caused it. So the transform refuses up front:

| Refused | Because |
|---|---|
| `-accel kvm` / `whpx` | hardware virt has no instruction counter to anchor the trace to |
| `-smp N` for N > 1 | MTTCG has no recorded global instruction order across vCPU threads |
| `-netdev` / `-nic` / `-net` | network input needs replay net filters we have not wired |
| an existing `-icount` | we will not record on top of someone else's counting config |
| no `-drive` at all | `rrsnapshot=` needs a snapshot-capable image |
| the `windows-host` backend | media stage into a Windows path; the trace would straddle two path namespaces |
| a drive behind `ide-cd` / `scsi-cd` | QEMU 9.0.2's `blkreplay` stalls the ATAPI path outright — see below. This is why **x86 cannot be recorded yet** |

x86 is wired end to end — the transform, the media staging and the accelerator
demotion below are all tested against the real configs — but it **cannot
actually record today**, because its boot ISO sits behind `ide-cd`. arm64 is the
only arch proven on real media.

One refusal the transform does NOT make is x86's default accelerator. `amd64`
and `i386` boot `-accel kvm -cpu host`, which would fail the first row of that
table on every recording — so the launch hook **demotes them to TCG itself**
(`backend.rewrite_accel`, which also drops the paired `-cpu host`, a KVM
host-passthrough concept TCG cannot honour) rather than handing the operator an
error to decode. arm64 is carved out: it already runs TCG with a real
`-cpu cortex-a72` that the rewrite would strip. Recording x86 is therefore
slower than an ordinary x86 boot by more than the icount overhead alone, since
it gives up hardware virtualization as well.

`--smp 1` **is** accepted, and that matters: on the MP kernel one vCPU is exactly
the UP:=smp1 collapse case the arm64 bug lives in. Refusing all `-smp` would
have refused the only interesting boot.

## Replaying with gdb

```bash
./vibeos-cli rr replay latest --gdb
```

This halts at instruction 0 with the stub open (the port is printed by the boot
harness; it is auto-allocated per run — never hardcode 12001). Inside gdb:

```
target remote 127.0.0.1:<port>
add-symbol-file build-arm64/.../ntkrnlmp.exe <kbase+0x1000>
continue                 # runs to the fault
reverse-continue         # runs BACKWARDS to the previous stop
watch -l *(long*)&thing
reverse-continue         # -> lands on whoever wrote it
```

Reverse execution is the whole point. For the `PC=0` fault, the question is not
where it faulted (we know: instruction fetch from address 0) but *who zeroed the
return address*, and a reverse watchpoint answers that directly.

`--gdb` works through the harness's own gdbstub plumbing (`VIBEOS_QEMU_GDB=wait`),
not a second implementation, so everything that is true of `debug start --gdb` is
true here.

## The icount shift is the one setting that can break a healthy guest

`-icount shift=N` costs the virtual clock `2**N` ns per instruction, which is
the same as fixing the guest's apparent CPU at `2**-N` GHz — shift=3 is a
125 MHz machine, shift=7 a 7.8 MHz one. Two things follow, and they pull in
opposite directions:

* **Too large and the guest times out.** The guest's clock advances at
  `real_MIPS * 2**N / 1000` virtual seconds per real second, so at shift=7 on
  this host virtual time runs roughly **6x faster than real**. Firmware and
  bootloader watchdogs are denominated in that virtual time. Measured: arm64 at
  shift=7 never reaches the kernel — the UEFI 5-minute watchdog expires while
  freeldr is still loading the 290MB ramdisk, and the boot loops
  (`Watchdog Timer resetting system`, five times, `[freeldr] Switching stack...`
  as the last line each round).
* **Too small and it costs real seconds.** A firmware delay loop spins for a
  number of *virtual* ns, so each step down the shift doubles the instructions —
  and the wall clock — it burns.

**shift=3 is the measured default**: arm64 boots to the kernel and through the
kmtest sweep in ~2 minutes of wall clock. `shift=5` also records cleanly and
faster (63s, 6MB vs 10MB), so the ceiling sits between 5 and 7 — but it does not
help the thing that actually hurts (see the throughput entry in the divergence
log), so the default stays at the value with the most headroom. Change it with
`--shift`; a trace records the value it was made at and replay re-uses it,
because replaying at a different shift feeds the recorded input stream to a
differently-clocked guest.

`shift=auto` is unusable here at any value: it calibrates against the host, so
record and replay would be different machines.

## Determinism findings

Divergence on replay is a **finding about our own system**, not an obstruction.
Record it here and keep going; only chase one when it looks load-bearing.

- **`blkreplay` cannot drive an ATAPI CD, which is why x86 cannot be recorded
  (2026-09-13). MEASURED, root-caused, and now refused up front.**

  An amd64 recording never leaves the OVMF screen clear — one line of serial,
  QEMU idle at ~0% CPU (4m42s elapsed, 11s of CPU), no trace growth. Two rounds
  of one-variable A/B, 100s each, same media:

  | config | serial lines |
  |---|---|
  | plain TCG | 3327 |
  | `-icount shift=3` alone (no rr, no blkreplay) | 2124 |
  | rr + blkreplay, `rrsnapshot` removed | **1** |
  | full rr | **1** |

  So neither icount nor `rrsnapshot` is implicated. Wrapping one drive at a time
  then pins the device:

  | wrapped in blkreplay | serial lines |
  |---|---|
  | nothing | 1975 |
  | scratch only (`virtio-blk-pci`) | 2050 |
  | **cd0 only (`ide-cd`)** | **1** |
  | both | 1 |

  `blkreplay` over `virtio-blk` is fine; over `ide-cd` the guest never boots.
  That explains the whole arch split: arm64 is virtio throughout and records
  cleanly, while amd64 and i386 boot their ISO from ATAPI.

  `transform()` now **refuses** a drive referenced by an `ide-cd` or `scsi-cd`
  device rather than recording a dead trace. Leaving that one drive unwrapped
  would have been worse than refusing: it is the BOOT media, so its
  non-determinism would poison everything downstream, silently. `scsi-cd` is
  listed alongside `ide-cd` because it is the same command set on the same block
  path — naming only one would let the identical failure back in the moment
  somebody changed the controller.

  **The shape of the fix** is to attach x86 boot media through virtio-blk under
  rr, which the measurement says would work. That changes the device set the
  guest sees, so it is a deliberate choice rather than something to do silently,
  and it is not needed for the bug this tool was built for (#39 is arm64).
  A test asserts a virtio-backed CD is allowed, so the path stays open.

- **Replay reproduces CONTENT faithfully but does not reliably run to
  completion: it WEDGES, at a point that differs between runs of the same trace
  (2026-09-13, trace `rec5`). OPEN — this is the headline limitation.**

  The good half, and it is real: over the 968 lines one replay produced, **every
  line matched the recording exactly**. Where a replay progresses, it progresses
  correctly.

  The bad half: a second replay of that same trace stopped at **25 lines**, at
  `[FLDR] load ramdisk(0)\reactos\system32\config\SYSTEM` — the first large
  ramdisk read. Two runs, one trace, two different stopping points.

  It is a wedge, not slowness, and one measurement settles that. The replaying
  QEMU's read position in `replay.bin` sat frozen at **167,936 bytes of
  10,266,279 (1.6% of the trace)** across 200 seconds of sampling while the
  process burned **111% CPU**, then died on the watchdog. It is executing
  furiously and consuming no further recorded events.

  To check this yourself in one command — it is the measurement that separates
  "be patient" from "this trace will not replay":

  ```bash
  P=$(./vibeos-cli ps | awk '/<your run dir>/{print $4}')
  for f in /proc/$P/fd/*; do case "$(readlink $f)" in *replay.bin)
      grep ^pos /proc/$P/fdinfo/$(basename $f);; esac; done
  ```

  Sample it twice a minute apart. Frozen position + high CPU = wedged.

  Two earlier readings of this were wrong and are worth recording as such,
  because both were single samples of a busy process. The vCPU was once caught
  at `DebugService+0` (`sdk/lib/rtl/arm64/debug_asm.S:13`), which suggested the
  cost scaled with how much the boot prints; and before that the whole thing was
  written up as a determinism divergence when `hang-cpustate.txt` plainly said
  `VM status: running`. A sampled PC is where the vCPU happened to be, not where
  the time went.

  Untried leads, cheapest first: whether the wedge is specific to the 290MB
  ramdisk drive (x86 boots a smaller ISO — now that x86 records, that is a
  one-boot experiment); whether a trace recorded with a warm page cache behaves
  differently; whether QEMU newer than 9.0.2 fixes it.

  **What it means for the #39 hunt:** a recorded crash is still worth having —
  the fault happens early, inside the window replays do reach — but a replay
  cannot be assumed to get anywhere in particular. Record, then verify the trace
  reaches your fault before building a debugging session on it.

  `--shift` is NOT the lever it looked like, and it is worth being precise about
  that because the reasoning said it should be. `shift=5` records fine and faster
  than `shift=3` (63s vs 119s, 6MB vs 10MB trace — so the watchdog ceiling is
  between 5 and 7, not below 5 as the `shift=7` failure first suggested). Its
  replay is **worse**: still at the UEFI banner, 271 bytes, after eight minutes,
  where `shift=3` had reached 968 guest lines. Whatever bounds replay throughput
  is not the instruction cost of a virtual second.

  **What it means in practice:** a replay is tens of minutes for a full boot to
  desktop. That is still the right trade for a bug you otherwise cannot catch at
  all, and it bites less than it sounds for the case we built this for — the #39
  fault happens early in boot, long before the 3,000-line tail.

  The documented remedy for real-time idle waiting, `sleep=off`, would not have
  helped given the above — and does not work here anyway:

  * **recording** with it never leaves the UEFI firmware — the vCPU sits at a
    firmware PC, serial stops after BdsDxe clears the screen, the run dies on
    the watchdog at 10 minutes and writes no trace at all;
  * **replaying** with it aborts while loading the rrsnapshot:
    `qemu-system-aarch64: migration/vmstate.c:140: vmstate_load_state:
    Assertion 'first_elem || !n_elems || !size' failed.`

  A/B'd on one trace, same media, nothing else changed: without the flag that
  same replay ran fine. So the replay half is a QEMU-side interaction between
  `sleep=off` and `rrsnapshot`, not our configuration, and the flag stays off —
  with a test pinning it off so nobody re-adds the obvious fix.

  Worth revisiting on a newer QEMU — the `vmstate_load_state` assertion in
  particular reads like a bug rather than a constraint.

- **A replay is slower than its recording, and the harness's silence watchdog
  did not know that (2026-09-13, trace `rec1`) — OUR bug, now fixed.** The
  replay's guest serial was byte-identical to the recording's for 54,540 bytes,
  up to the netstack self-test's timed waits, and was then aborted
  `HANG_TIMEOUT` by the 120s no-serial watchdog while the recording had gone on
  to `[EXPLORER] _tWinMain entered` at 240,407 bytes. It looked like a
  determinism failure and was not: the wedge capture shows `VM status: running`
  with the vCPU advancing through an EL0 user PC, i.e. executing normally and
  merely quiet. (The later, harder finding is that a replay can ALSO genuinely
  wedge — see the first entry. Both are real; this one was the harness's fault
  and is fixed, that one is not.) `_apply_record_replay` now scales `boot_timeout` and
  `hang_silence_seconds` for any rr run, the same way `_build_windows_cmd`
  already scales them for whpx.

  Worth writing down because the first reading was wrong in an expensive
  direction: the tool's own output said DIVERGED-shaped things, and the
  cheap check that settled it — reading `hang-cpustate.txt` for whether the VM
  was actually stopped — cost nothing.

- **`Playback shouldn't have to iowait (insn total 1/1 left, event 7 is
  EVENT_INSTRUCTION)`** — seen on a degenerate bare VM (no firmware, no media)
  under `-S` + gdb during the initial spike. On a machine with nothing to
  execute, the replay's first scheduled event is already past the end of the
  instruction stream. Not reproduced on a real vibeos boot; if it reappears
  there, it is a genuine divergence and belongs in a bug entry, not this list.

## Costs and limits

- **Recording is cheap and reliable. Replaying is neither.** Recording is
  measured on arm64 (`smoke --smp 1`, boot to the desktop marker): **~90s under
  record vs ~40s normally**, 11MB `replay.bin`, and it has not failed. A replay
  runs at a small fraction of that pace and, on current evidence, **wedges
  partway through at a point that varies between runs of the same trace**.
  Record freely; treat each replay as an experiment whose reach you confirm
  rather than assume. Neither is a soak.
- A trace directory is the trace plus the qcow2 overlays, twice (counting
  `pristine/`) — tens of MB for arm64, since the big media are overlays over the
  originals rather than copies.
- A trace is pinned to a QEMU build. Device models change between releases and
  the trace does not; `rr info` warns when the versions differ, which is the
  most likely reason an old trace stops replaying.
- A killed QEMU leaves a zero-byte `replay.bin` — the log is flushed on exit.
  Let a recording end on its own, not on a timeout kill.

## Where the code lives

| Piece | File |
|---|---|
| the command-line transform, media staging, trace lifecycle | `tests/qemu/cli/recordreplay.py` |
| the `rr` verbs | `tests/qemu/cli/rrcmd.py` |
| the launch-path hook (`VIBEOS_QEMU_RR=<mode>:<dir>`) | `tests/qemu/vm.py:_apply_record_replay` |
| guards | `tests/qemu/cli/test_recordreplay.py` |

The guards include three that drive the transform with the **real** output of
each `ArchConfig.qemu_args` rather than a hand-written fixture, and assert that
every `-device ...,drive=X` resolves to a blkreplay filter. That is the check
that found the x86 KVM default above, and it is the one that will catch the next
arch-config change rr cannot handle — at test time rather than halfway through
somebody's recording. The x86 media shape is the reason it matters: their boot
media is an `if=none,id=cd0` ISO with a separate `-device ide-cd,drive=cd0`, so
the filter must inherit `cd0` and the raw image become `cd0-img`. Get that
backwards and QEMU starts happily with the CD attached to the unwrapped image.

The hook is opt-in and nothing sets it by default, so every existing boot path
is byte-for-byte unchanged when rr is not requested.
