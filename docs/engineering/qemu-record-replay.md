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

## What a recording IS — a conformance or smoke boot, never a gate boot

`rr record` drives `vibeos-cli conformance` (`--payload kmtest`) or
`vibeos-cli smoke` (`--payload none`). It does **not** drive `vibeos-cli gate`.
So anything the gate adds to a boot is outside what a trace captures unless
somebody wires it in on purpose.

This is worth stating because the wrong inference is a short one and its premises
look true. When `9f73d184fea1` made arm64 gates ASK for the WoW64 ladder, the
natural reading was "arm64 gates run the ladder" + "rr records arm64 boots"
therefore "traces now contain nine WoW64 guests that were not there before" —
which would make a trace recorded today incomparable with one recorded
yesterday, and send someone hunting a difference that does not exist.

It is false twice over, and both refutations came from checking rather than
reasoning. A recording is not a gate boot: `want_ladder` is referenced only
inside `smoke.py`, the gate opts in explicitly (`gate.py`), and neither rr path
passes `--ladder`. And the ladder does not actually run on default arm64 media
anyway — it declines in `0.0s` on all three arches, because arm64's WoW64 CPU
layer is the `VIBEOS_FEX=1` opt-in (`mkramdisk.sh`) that default media does not
stage. The 84.7s drain quoted at the time was measured on the one tree in the
fleet with that opt-in enabled.

Adding a gate-only behaviour to recordings is a one-line change to
`_boot_cmd` in `rrcmd.py`. It should stay a deliberate choice rather than
something that arrives by inheritance — a trace's value is that two of them are
comparable, and that holds only while what gets recorded is pinned.

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

## Replaying with gdb — a replay cannot be stopped, so it cannot be debugged

**Read this before planning any debugging session on a replay.** Measured on
aarch64 QEMU 9.0.2, on a fresh warning-free trace, outside the harness so
QEMU's own stderr was visible:

| what | result | replay-specific? |
|---|---|---|
| `--gdb` (halted at instruction 0), connect, read `$pc` | **works** | — |
| `continue` from there | **works** — guest ran 45s, 1691 serial lines, QEMU healthy | — |
| `stepi` | QEMU **aborts** (rc -6) | **yes** — replay-only error, see below |
| `interrupt` while the guest is running | does not stop the target (15s) | **NO** — same on a non-replay control |
| breakpoint (`break` or `hbreak`) at a kernel VA, set at `-S` | never fires | **NO** — same on a non-replay control |
| attach to an already-running replay (stub without `-S`) | QEMU aborted in the one test of this | untested against a control |

**Only the `stepi` row is established as a replay limitation.** The other two
were attributed to replay in an earlier draft of this document and that was
wrong: running the same commands against a *non-replay* boot of the same media
reproduced both. `interrupt` not landing is a limitation of driving gdb from a
batch script.

The breakpoints are now **explained and fixed**, and the cause had nothing to do
with record/replay: `qemu/gdb-helpers/find-kbase.py` was relocating symbols to
the wrong kernel image. Every media carries BOTH kernels and boots MP by default
since the 2026-09-12 promotion, but `VIBEOS_KERNEL` still names the UP image, so
the scan matched `ntoskrnl` at `0xfffff800505e4000` while `ntkrnlmp` was actually
executing at `0xfffff80042190000`. Breakpoints therefore landed at addresses the
guest never executes and silently never fired — which looked exactly like
"breakpoints don't work on arm64". Fixed by preferring the image containing
`$pc`, and failing that the MP image; verified end to end
(`run-until.sh --break ExAllocatePoolWithTag` now reports `=== BREAKPOINT HIT ===`
at `<ExAllocatePoolWithTag+44>`).

That unblocked the real question, which was then answered in the worst way:
**attaching gdb to a RUNNING replay aborts QEMU**, with the same
`Playback shouldn't have to iowait` (`insn total 9616974735/473 left, event
157129`). The replay had booted to the kernel first — 724 lines, correct
`[KRNLBASE]` — and connecting was enough to kill it. That is now two independent
instances of the same rule, and it explains the whole picture:

> **A replay can be started stopped, and run. It can never be stopped again.**
> Attaching at `-S` before the replay runs is fine, and `continue` from there is
> fine. Every later stop — `stepi`, attaching mid-run — aborts.

**A breakpoint hit is also such a stop, and it aborts too.** With the breakpoint
at the verified-hot `ExAllocatePoolWithTag`, set at `-S` and continued into, the
replay died at the hit:

```
qemu_rc = -6
qemu-system-aarch64: Playback shouldn't have to iowait
  (insn total 9507590363/3422 left, event 64901 is EVENT_INSTRUCTION)
```

That closes the question with no exception left, and it is the answer that
decides what this tool is:

> **On QEMU 9.0.2, rr is a camera, not a debugger.** A trace is a deterministic
> *capture* — re-runnable, with serial, screenshots and the harness's own triage
> all working. It is not a deterministic *debugging session*: there is no way to
> stop a replay and look, by any means, because every mechanism for stopping one
> is the thing that kills it.

Two earlier rounds of this experiment reported "the breakpoint never fired", and
both were **my arithmetic**, not a property of replay. `qemu/symbols-*.tsv`
holds LINKED-IMAGE addresses, whose preferred base is `0x400000` — not RVAs. The
true RVA is `addr - 0x400000` (the `PE_PREFERRED_IMAGE_BASE` constant in
`find-kbase.py` is exactly this). Breaking at `kbase + 0x42fac4` instead of
`kbase + 0x2fac4` put the breakpoint 4MB past the function, where nothing ever
executed. Every "set a breakpoint and nothing happened" result in this document's
history traces to that one error; the corrected address hit on the first try.

One loose end, recorded rather than explained: the `rr=record` control leg
produced **zero** serial lines in 1200s with a breakpoint set, where the replay
leg under the same conditions reached 1,378. Recording without a breakpoint is
fast and reliable, so something about breakpoint checking under `rr=record` is
pathological. Not chased — it does not change the conclusion above.

The abort is:

```
qemu-system-aarch64: Playback shouldn't have to iowait
  (insn total 1/336313 left, event 5 is EVENT_INSTRUCTION)
```

Backtraced from the stripped binary: `replay_sync_error()`, on a **worker
thread**, not the vCPU thread. That fits the numbers — a single-step gives the
vCPU a budget of exactly one instruction, and the replay refuses to wait for
in-flight block I/O when the log says the next event is an instruction. So the
collision is between single-stepping and `blkreplay`, which is also the
component that is outright fatal on ATAPI.

**Net effect: you can run a replay under gdb, but nobody has yet stopped one at
a point of interest.** `reverse-continue` and `reverse-stepi` remain untested —
QEMU implements them as snapshot-plus-replay-forward rather than backwards
stepping, so they may well work — but issuing them needs the guest stopped
somewhere useful, and the way to do that has not been established *even on a
normal boot* with the ad-hoc gdb driving used here.

So the next step is not a replay experiment at all: get a breakpoint to fire on
an ordinary arm64 boot first, using `qemu/run-until.sh` (which does this on the
normal debug path), and only then point the same technique at a replay. Testing
two unknowns at once is what produced two retracted findings above.

This matters more than any other limitation here, because reverse execution was
the *point*. For the `PC=0` fault the question is not where it faulted — we know
that — but who zeroed the return address, and a reverse watchpoint would answer
it directly. Today there is no way to get there.

**This section previously described that recipe as working. It was never
tested.** The same error was in the divergence log below, attributed to a
degenerate bare VM and explicitly marked "not reproduced on a real vibeos boot";
that attribution was wrong, and the first real test reproduced it immediately.

`rr replay --gdb` opens the stub and prints a warning saying all of the above,
because the path is genuinely half-working and should stay available — this
reads like a bug rather than a design constraint, and it is the highest-value
thing to re-test on a QEMU upgrade.

**What a trace is still good for meanwhile:** a deterministic, re-runnable
capture of one specific boot, including a crashing one — serial, screenshots and
the harness's own triage all work on a replay. It is the debugger attach that
does not.

### To re-test after a QEMU upgrade

```bash
./vibeos-cli rr record --arch arm64 --payload none --name gdbcheck
./vibeos-cli rr replay gdbcheck --gdb     # note the port it prints
gdb-multiarch -batch \
  -ex 'set architecture aarch64' \
  -ex 'target remote 127.0.0.1:<port>' \
  -ex 'stepi' -ex 'output/x $pc'
```

If that prints a PC instead of `Remote connection closed`, single-stepping works
again and the rest of the recipe is worth retrying. Try breakpoints too — they
are the untested way to reach a point of interest without stepping, and they may
work on the QEMU we already have.

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

  **The obvious fix — attach x86 boot media through virtio-blk instead — DOES
  NOT WORK, and was tested (2026-09-13).** blkreplay is fine over virtio-blk, so
  swapping `-device ide-cd,drive=cd0` for `-device virtio-blk-pci,drive=cd0`
  looked like it would make x86 recordable. It does not, and rr is not the
  reason: measured WITHOUT rr, on plain TCG, one variable changed —

  | boot media attached via | serial lines | reached the kernel |
  |---|---|---|
  | `ide-cd` (control) | 7214 | yes |
  | `virtio-blk-pci` | 11 | **no** — stops at `[freeldr] Switching stack...` |

  Firmware and freeldr start up fine from virtio; the boot then dies where
  freeldr loads the kernel, so vibeos's x86 boot path cannot take its LiveCD
  over virtio-blk regardless of what rr wants. Making x86 recordable therefore
  needs blkreplay to handle ATAPI — a QEMU-side fix, or a newer QEMU — not a
  device-set workaround on our side.

  A test still asserts a virtio-backed CD is *allowed* by the transform, so the
  path stays open for whenever the guest can boot that way.

- **Does the arm64 wedge track the ramdisk drive? UNRESOLVED — and the
  experiment that would answer it needs `rr record`, not a hand-rolled one
  (2026-09-13).**

  The ATAPI result above came from wrapping one drive at a time, so the obvious
  next move was the same A/B on arm64: record and replay with only the scratch
  drive wrapped, then only the boot/ramdisk drive. One run did show the wedge
  reproducing with **only the ramdisk drive wrapped** (record 8,933 lines, replay
  21). But the comparison variants are not trustworthy and the numbers are
  deliberately not quoted here, for two reasons worth knowing before anyone
  repeats this:

  * **The hand-rolled record leg could not finalise a trace.** A recording must
    exit cleanly or QEMU never flushes the rr log. Driving QMP `quit` on a timer
    did not terminate QEMU within 240s in any of three attempts; every resulting
    trace was truncated and replayed as
    `Replay: invalid input log file version`. `rr record` gets this right — it
    shuts down at the boot marker through the harness — so vary the CONFIG and
    let the tool own the recording, rather than reimplementing the record leg.
  * **Unwrapping a drive means restoring its `if=`, not just its id.** arm64's
    boot drive is `if=virtio` with no explicit `-device`: the transform moves
    `if=virtio` onto the filter and leaves the backend `if=none`. Drop the filter
    without putting `if=virtio` back and the guest boots with no disk and stalls
    early — which reads exactly like the wedge under investigation. (x86's drives
    have an explicit `-device`, so the id alone sufficed there and the bug did
    not show.) That silently invalidated a whole variant.

  Because run durations then diverged, the record-throughput comparison between
  variants was measuring different amounts of wall clock and has been discarded
  rather than reported.

- **Replays WEDGE: they stop consuming the trace partway, at a near-deterministic
  point, while burning CPU. Cause unknown. OPEN.** This entry was retracted and
  then un-retracted the same day; both the retraction and the correction are kept
  below, because the reasoning error in the middle is more instructive than
  either verdict.

  This entry stood as the headline limitation for most of a night and was wrong.
  What settled it: replay the same trace twice, sequentially and ALONE — every
  earlier observation had been taken while a gate batch hammered the box, which
  is the confound that produced "varies between runs".

  On a trace recorded BEFORE the finalisation fix, both runs stopped at exactly
  the same place — identical 56-line transcripts and the same abort at the same
  instruction:

  ```
  Missing character write event in the replay log
    (insn total 6497080561/345 left, event 55969 is EVENT_INSTRUCTION)
  ```

  That is a truncated log, not a wedge, and it is perfectly deterministic. On a
  trace recorded AFTER the fix, two sequential replays gave 1802 and 1773 lines,
  `rc=-9` from the harness budget in both cases, **`err=(none)`** — no abort at
  all — and the shorter run's transcript is an exact PREFIX of the longer's
  (common prefix 1773, zero divergence). The 29-line difference is how far each
  got in 900 seconds, nothing more.

  **CORRECTION, same day: the retraction above is WRONG in its main claim.
  Replays DO wedge, on correctly finalised traces too.** The two runs above were
  both ended by my 900s budget, and I read "no abort" as "still progressing"
  without checking whether they were still CONSUMING the trace. They were not.

  A third run with NO timeout reached 1773 lines in 120 seconds and then sat
  frozen there for over ten minutes at 96% CPU, with its read position in
  `replay.bin` stuck at **2,342,912 of 10,666,166 bytes (22%)** across 45s of
  sampling. That is the wedge, on a trace carrying `finalised: true`.

  What the retraction DID establish, and what stands:

  * **Truncated traces are a separate, distinct failure.** They do not wedge,
    they ABORT — `Missing character write event in the replay log`,
    deterministically, at 56 lines. Now prevented at record time, refused at
    replay time.
  * **The wedge point is far more reproducible than "varies between runs".**
    Three runs of one trace stopped at 1802, 1773 and 1773 guest lines of 2772.
    The 25-vs-968 spread behind that phrase was measured across *different*
    traces under *different* box load, which is not a comparison at all.
  * **Replay speed is strongly load-dependent**: 1773 lines took 900s on a busy
    box and 120s on a quiet one — which is what made a fixed wedge point look
    like a moving one.

  **The wedge is at a FIXED TRACE OFFSET, which makes it findable.** Two runs of
  trace `clean1` — one hand-rolled, one through `rr replay` — both froze with the
  read position at **byte 2,342,912 of 10,666,166 (22%)**. The same offset, to
  the byte, through two different harnesses. Guest serial differs slightly
  between them (1773 vs 1693 lines, a capture/flush difference), but the trace
  offset does not move.

  That is a much better handle than "wedges partway". It is not a race and not a
  timeout: replay cannot get past one specific point in the recorded event
  stream.

  **Decoded, with QEMU's own `scripts/replay-dump.py`** (fetched from the
  matching stable-9.0 tag — do not hand-write a decoder for this format):

  * **The trace is complete and well-formed.** The dumper reads 10,666,165 of
    10,666,166 bytes without complaint, so the wedge is not corruption or
    truncation. Its 235,577 `EVENT_CHAR_WRITE` records line up with the 235,754
    bytes of recorded guest serial.
  * **The stall is inside a long, uniform serial-output burst.** Around the
    frozen offset the log is nothing but `EVENT_CHAR_WRITE(22) 1 -> 1` alternating
    with `EVENT_INSTRUCTION(0) + 3`, for thousands of records either side — the
    guest pushing one byte at a time into the UART. Nothing structurally unusual
    sits at the boundary, so this is not "replay chokes on a weird event".
  * **It stops just over halfway through the guest's output.** 128,789 of the
    235,577 char-writes are consumed (~55%), matching the 124,212 serial bytes
    the replay emitted. The 22% figure is of trace BYTES; char-writes are cheap
    records and instruction records are not, which is why the two fractions
    differ.
  * Event census for scale: 1,445,912 `EVENT_INSTRUCTION`, 1,178,159
    `EVENT_CP_CLOCK_WARP_ACCOUNT`, 235,577 `EVENT_CHAR_WRITE`, 43,747
    `EVENT_EXCEPTION`, 13,920 `EVENT_INTERRUPT`.

  Note that 2,342,912 is exactly 572 x 4096: it is a read-buffer boundary, so
  the true stall is the last event inside that buffer rather than the byte
  itself. QEMU stopped asking for more.

  **A second trace stalls in the same neighbourhood.** `rr replay` on a kmtest
  trace 46x larger reported, via the detector added for exactly this:

  ```
  [rr] replay consumed 2,215,936 of 494,129,068 trace bytes (0.4%)
  ```

  2,215,936 against the smoke trace's 2,342,912 — both ~2.2-2.3MB, both
  4096-aligned (541 and 572 buffers), on traces of 10.7MB and 494MB. So the
  stall is NOT proportional to trace length.

  **What that does not yet establish, and the distinction matters:** both traces
  are arm64 boots of the same media, so their early event streams are
  near-identical and only diverge once the kmtest sweep starts. Stalling at a
  similar offset is therefore equally consistent with

  * a structural limit around ~2.2MB (a buffer, a window, a counter), or
  * the same GUEST point being reached in both, which naturally lands at a
    similar offset because the boots are the same up to there.

  Two traces that share a boot prefix cannot cleanly separate those — equal guest
  progress implies equal trace bytes while the content is the same, so both
  hypotheses predict the same observation.

  **There is a modest signal, in WHICH measure agrees better.** Both replays stop
  in the same boot phase, desktop bringup:

  | trace | trace bytes | guest serial | last guest line |
  |---|---|---|---|
  | smoke, 10.7MB | 2,342,912 | 124,212 | `[WIN32K] shell ready` |
  | kmtest, 494MB | 2,215,936 | 122,199 | `[EXPLORER] taskbar: creating tray window` |

  The trace offsets differ by **5.6%**; the guest serial counts differ by
  **1.6%**. A hard trace-byte limit would predict the offsets to be the tighter
  pair. They are not — guest progress is the better-preserved invariant, which
  leans toward "the replay stops at a point in the GUEST" rather than "at a
  point in the FILE". That is a lean, not a proof; two samples, and the boots
  share their media.

  Decisive separation still needs a trace whose early boot differs materially.
  A different arch would be ideal and x86 cannot record; within arm64 the
  candidate is a UP-kernel boot, which executes genuinely different kernel code,
  but `rr record` has no route to select it today (every media defaults to MP
  since the promotion, and the UP entry is not reachable through the rr path).
  Wiring that would be the next concrete step on this question.

  Where it points meanwhile: desktop bringup — explorer, win32k, user32 — is a
  far more specific neighbourhood than "somewhere in the boot", and it is
  consistent with the serial-output finding above, since that phase is the
  noisiest part of the boot.

  So the open question is narrower again: not "which event", but why replay
  stops advancing in a region where every record is trivial. Serial output is
  also where the earliest (and then retracted) sighting pointed — a vCPU caught
  in `DebugService`, the guest's own serial primitive. That reading was
  unsupported at the time and is now, independently, where the trace says the
  replay stops.

  It also retires a confound worth recording: my hand-rolled replays stripped
  `-qmp`/`-chardev` and substituted a file-backed serial for the recording's
  socket-backed one, so they ran a different device set than the recording. That
  was a real methodological hole — `assert_same_shape` only compares DRIVES — but
  it did not cause this: the harness, which reproduces the shape faithfully,
  wedges at the identical offset.

  The `rr replay` WEDGED detector reports exactly this, and also catches the
  truncated-trace abort.

  **Method note, because it is why the bad retraction happened:** "did not
  abort" is not "made progress". The distinguishing measurement is whether the
  rrfile read position moves — tooling I already had and did not apply to the
  runs I drew the retraction from. A killed replay and a wedged replay look
  identical if you only check the exit.

  A third run, through `rr verify`, gives the same shape from the artifacts:
  `record=3149 replay=2070 common_prefix=2070` — the replay's transcript is an
  exact prefix of the recording's, zero divergence, and it stopped only because
  its process was killed (that run was reaped; the verdict above is computed from
  the transcripts on disk, not from the wrapper's output, which the reaper took).


  The earlier measurement — read position frozen at 1.6% across 200s at 111% CPU
  — was taken on `rec5`, recorded before the finalisation fix, so it conflated
  the two failures. It now reads as the truncated-trace case; the 22%-frozen
  measurement above is the wedge proper, on a finalised trace.

  (The original observations — 968 lines one run, 25 the next — were two
  different pre-fix traces under different box load, which is precisely the
  comparison that cannot be made.)

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

- **`Playback shouldn't have to iowait` — RESOLVED as the gdbstub limitation
  above, and the earlier entry here was wrong.** This was first seen during the
  initial spike on a degenerate bare VM (no firmware, no media) under `-S` +
  gdb, and written up as an artefact of that degenerate setup with the note
  "not reproduced on a real vibeos boot". That note was an untested assumption:
  the first time anyone drove gdb against a real trace, it reproduced
  immediately (`insn total 1/336313 left, event 5`). It is not about having
  nothing to execute; it is what happens whenever a replay's guest is stopped by
  the debugger. See "Replaying with gdb" above.

## Recording the kmtest sweep

The chartered case — an arm64 `--smp 1` kmtest sweep, i.e. the MP kernel on one
CPU — records cleanly:

```bash
./vibeos-cli rr record --arch arm64 --payload kmtest --smp 1
```

Measured: 164s wall clock, `launched=91 ok=88 bad=3`, `sweep_done total=91
launched=91 failstart=0 timeouts=0`, `mp_kernel=True`, no bugcheck — and a
**489MB** trace with a 1.3MB serial transcript. It is a much heavier capture than
a plain boot (11MB), because a sweep does far more guest I/O.

**A recording is not over when the boot command returns.** `vibeos-cli
conformance` hands back as soon as its sweep is parsed, while the VM is still
shutting down. Measured on the first attempt at the above: the command returned
at 05:28:38, the guest's serial finished at 05:29:23, and the trace finished at
05:29:39. Finalising at the first of those three froze `pristine/` mid-record,
kept a 1,456-byte transcript of what became a 240,750-byte boot, and reported
"trace 0MB" for what became 10.6MB — three silent corruptions from one early
return, and every one of them poisons something different (a mid-record
`pristine/` may not carry the `init` snapshot every replay loads; a truncated
`record-serial.log` is what `rr verify` diffs against).

`rr record` now waits for `replay.bin` to stop growing before finalising
anything. Smoke never exposed this because its harness waits for its own VM;
kmtest, the payload this lane exists for, did.

## Costs and limits

- **Recording is cheap and reliable. Replaying is neither.** Recording is
  measured on arm64 (`smoke --smp 1`, boot to the desktop marker): **~90s under
  record vs ~40s normally**, 11MB `replay.bin`, and it has not failed. A replay
  runs at a small fraction of that pace and, on current evidence, **wedges
  partway through at a point that varies between runs of the same trace**.
  Record freely; treat each replay as an experiment whose reach you confirm
  rather than assume. Neither is a soak.
- **Traces are big and nothing else reclaims them.** A kmtest capture is ~1.4GB
  (489MB of trace, plus its media twice counting `pristine/`); seven experiment
  traces took this box from comfortable to 89% full in a night. `vibeos-cli
  prune` reclaims build and boot artifacts and does not know `~/vibeos-rr`
  exists, so use `./vibeos-cli rr prune --keep 3` (dry-run; `--apply` to act).
  `--name <trace>` deletes a specific one regardless of age — which is what you
  want for a known-bad capture, since those tend to be the newest.
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
