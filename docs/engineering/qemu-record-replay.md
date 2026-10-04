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
| a drive behind `ide-cd` / `scsi-cd` | QEMU 9.0.2's `blkreplay` stalls the ATAPI path outright — see below. x86 records over usb-storage instead (next paragraph) |

**x86 records AND replays to the desktop (2026-10-04)**, booting its ISO over
usb-storage: `VIBEOS_QEMU_BOOT_TRANSPORT=usb-storage ./vibeos-cli rr record --arch
amd64 --payload none`, then `rr replay <trace>` (replay to `shell ready` in 1m26s,
92% of the trace consumed -- the rest is past the marker). Getting there took one
loader fix, three kernel layers and three harness fixes, each found by the one before:

- freeldr booted `rdisk(N)partition(M)` for any ISO not on a 2048-byte-block device;
  it now asks whether the medium carries ISO9660.
- the kernel then needed the boot ArcName bound to a DISK holding the ISO
  (`IopAssignCdArcNameFromDisk`), partition zero's `VPB_RAW_MOUNT` lifted on that one
  device so cdfs's `\CdfsHdd` can mount it, and a cdfs translation bug fixed (the
  sector cache's no-TOC branch was translated under the wrong cfg, so every
  disk-based ISO raised STATUS_INVALID_PARAMETER).
- `rr record` passes `--no-net`: amd64's smoke runs a second boot for its net suite,
  and under the rr env it overwrote the trace's media after the recording ended.
- `-rtc clock=vm` on x86: the PC RTC ticks on the host clock by default, which filled
  the log with host-clock timer checkpoints. Recordings got ~3x faster, ~5x smaller.
- **x86 replay does not load the `init` snapshot.** Loading it re-armed a
  virtual-clock timer due at time zero (a device post-load hook); in replay mode
  that timer may only run at a recorded `CHECKPOINT_CLOCK_VIRTUAL`, and the
  recording has none before its first instruction. The vCPU then spins in
  `icount_handle_deadline -> qemu_clock_notify` while the main loop idles, at
  instruction count 0, forever (and SIGTERM goes unanswered: the main loop that
  handles it never runs). Recording still creates the snapshot; `prepare_replay`
  reverts every disk to it, so the replay starts from the same disks and the same
  power-on state the recording did. Found by `info replay` on a monitor (count 0),
  `-d exec` (not one translation block), and a backtrace of QEMU run under gdb
  (`ptrace_scope` refuses an attach). arm64 still loads the snapshot.

virtio-blk does not boot yet: the virtio ISO disk is not enumerated when ArcNames
are made, so only the empty scratch disk is scanned.

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

## Replaying with gdb — stop it with `replay-break`, NOT from the debugger

**A replay CAN be stopped and inspected. It just cannot be stopped from
outside.** This section previously concluded the opposite and said rr was "a
camera, not a debugger"; that was wrong, and the fix is a different stop
mechanism rather than a different QEMU.

The distinction is that the gdbstub interrupts the vCPU from outside the replay,
which aborts it, whereas QMP's `replay-break` names an **icount** — a position
in the recorded timeline — and lets the replay arrive there itself.

### The working recipe (verified end to end)

Start the replay with both a QMP socket and a gdbstub, then:

```bash
# 1. where is it now?
{"execute": "query-replay"}
    -> {"icount": 2288513945, "mode": "play", "filename": ".../replay.bin"}

# 2. stop it at a point of your choosing
{"execute": "replay-break", "arguments": {"icount": 2293513945}}
    -> {"return": {}}

# 3. confirm
{"execute": "query-status"}
    -> {"status": "paused", "running": false}

# 4. NOW attach gdb — the guest is already stopped, so nothing interrupts it
gdb-multiarch -batch -ex 'set architecture aarch64' \
  -ex 'target remote 127.0.0.1:<port>' -ex 'output/x $pc' -ex 'info registers'
```

Measured result: `pc=0x7fd1d04c`, registers read, gdb detached, **QEMU still
alive with empty stderr**. No `Playback shouldn't have to iowait`.

**But it is not reliable.** Three attempts, two clean stops and one abort — with
the same error the gdbstub produces:

```
replay-break at cur + 5,000,000   -> paused cleanly, gdb read registers   (x2)
replay-break at cur + 3,000,000   -> qemu-system-aarch64: Playback shouldn't
                                     have to iowait (insn total 1024013292/750000
                                     left, event 17661 is EVENT_INSTRUCTION)
```

### The rule: a break must land on an instruction-chunk boundary

Where the target icount lands is not *approximately* the reason — it is the whole
reason, and the abort message says so if you read the two numbers in it.

A replay executes the guest in the chunks the recording wrote down:
`EVENT_INSTRUCTION + N` means "run N more instructions, then look at the log
again". The failing target above, 1024013292, sits **499,347 instructions into a
1,249,347-instruction chunk** that runs 1023513945 → 1024763292. `1249347 -
499347 = 750000`, which is exactly the `/750000 left` QEMU reported. The two
successes were at 1967263945 and 1972263945, and both are chunk boundaries
exactly. Stopping part-way through a chunk leaves the replay owing instructions
it cannot run without advancing the log, and it dies rather than iowait.

So the set of legal break icounts is a property of the trace, readable offline,
and `./vibeos-cli rr breaks <trace>` prints it:

```
$ ./vibeos-cli rr breaks clean1 --near 1024013292
  1023513292
  1023513945
  1024763292   <- snaps here
  ...
1024013292 is NOT legal: it sits 499347 instructions into a 1249347-instruction
chunk, leaving 750000. Breaking there aborts the replay. Use 1024763292.
```

That is QEMU's own abort arithmetic, derived from the log before anything runs.

**The checkpoint hypothesis was wrong and is worth recording as wrong.** The
obvious guess was that `EVENT_CHECKPOINT` records mark the safe stops. They do
not discriminate anything: a single arm64 boot contains 1,178,159
`EVENT_CP_CLOCK_WARP_ACCOUNT` records alone, so nearly every icount is beside a
checkpoint and being beside one predicts nothing. The variable is the chunk
boundary.

**There is no arithmetic shortcut, so don't invent one.** Chunk sizes in that
same boot run from 1 instruction to 2^28, with a median of 18: 1.07M chunks of
exactly 18 instructions during real execution, and 1.25M-instruction chunks while
the guest idles in WFI. A break picked as "current + a round number" is nearly
always safe during execution and nearly always fatal during an idle stretch —
which is precisely the pattern the three samples showed, and why two successes
looked like a working mechanism. Snap against the trace instead.

**Status of the rule: VERIFIED PROSPECTIVELY.** It was derived retrospectively
from three samples, which is where most of our "rules" stop. This one was then
made to predict. Two fresh replays of `clean1`, in separate QEMU instances so
that an abort in one could not explain the other, with the prediction — including
the exact remainder, computed from the trace before QEMU was started — written
out first:

```
[boundary] icount=1674763945  target=1679763945
[boundary] PREDICTION: 1679763945 IS a boundary -> replay pauses cleanly, QEMU survives
[boundary] t+10s PAUSED cleanly, qemu alive, icount=1679763945

[midchunk] icount=2094763292  target=2100388618
[midchunk] PREDICTION: 2100388618 is 624673 into a 1249347-instruction chunk
           (2099763945..2101013292) -> ABORTS with 'insn total 2100388618/624674 left'
[midchunk] QEMU DIED rc=-6
[midchunk] qemu-system-aarch64: Playback shouldn't have to iowait
           (insn total 2100388618/624674 left, event 24118 is EVENT_INSTRUCTION)
```

Both predictions correct, the abort matching to the digit. `rr breaks` also
reproduces QEMU's own decode of the log independently — same 1,445,912
boundaries and same final icount as `replay-dump.py`, in 1.9s.

So a snapped icount is a rule, not better odds. What remains unmeasured is
whether anything ELSE can abort a break at a legal boundary; nothing has, in
three boundary attempts across two harnesses.

`query-replay` is also the best available read of replay progress — a live
icount, better than inferring position from the rrfile read offset.

### What is still unknown

* Whether `replay-seek` to an EARLIER icount works. If it does, that is reverse
  execution by its real name — QEMU implements `reverse-continue` as
  snapshot-plus-replay-forward, which is exactly what seek does. Give it a
  boundary icount from `rr breaks`; the seek target is presumably subject to the
  same chunk rule as a break, though that is an assumption and not a measurement.
* Whether anything other than a mid-chunk target can abort a break. Nothing has
  in three boundary attempts across two harnesses, but that is a small n.
* Whether `continue` after a `replay-break` pause resumes safely, or whether the
  replay may only be stopped once.
* Whether a gdb breakpoint set while paused survives a resume.

### Wired into the tool: `rr replay --break-at`

**Measured end to end, and one gap found by doing it.** On a freshly recorded
arm64 trace, through the real harness:

```
[rr] break icount 8000000000 is mid-chunk and would abort the replay;
     using the next legal boundary 8139927533 (+139927533)
[rr] replay-break armed at icount 8139927533
```

The snap distance is the point: 8e9 lands inside one of the 2**28-instruction
idle chunks, so the unsnapped request was 139.9M instructions from anywhere
legal. The arming works; the replay then stopped with its serial frozen at 5,789
bytes, which is about where that icount falls, so the break very likely fired.

**"Very likely" is the gap.** Nothing reports that the break FIRED. From outside,
a replay paused at a break and a replay wedged look identical — both have a
frozen serial log, and both leave QEMU burning CPU. The obvious discriminator
does not work: this paused-or-wedged QEMU sat at 87% CPU in `R` state, so
"paused replays idle, wedged ones spin" is false, or at least not reliable.
Confirming a pause needs `query-status`, and the harness owns the QMP socket —
the same reason `--break-at` had to be a hook in the first place.

**Closed.** QEMU does announce the stop — a QMP `STOP` event — and the harness's
client was throwing it away: `_recv_json` has to skip events while scanning for a
command's reply, and it discarded them on the way past. They are retained now
(bounded tail), `QmpClient.saw_event("STOP")` asks the question, and a replay run
with `--break-at` reports on the timeout path:

```
[rr] replay-break FIRED: guest stopped at icount 8139927533
```

or, when it did not, says so and names the next thing to check. That is the
timeout path on purpose: a break that fires IS a quiet serial log followed by the
harness waiting out its boot timeout, so that is where the answer is needed.

A control run of the same trace with no break wedged at a different, later point
(724,992 trace bytes / 36,965 serial bytes, against 229,376 / 5,789 with the
break). One run each, so that difference does not attribute: it is equally
consistent with the break firing early by design and with ordinary run-to-run
variance, which replays through the harness have shown before.

`./vibeos-cli rr replay <trace> --break-at <icount> [--gdb]` arms the break
through the harness's OWN QMP connection, snapping the requested icount forward
to a legal boundary and saying so when it moves.

The mechanism is a single hook on the VM, `on_qmp_ready`, called once with the
live `QmpClient` the moment the handshake completes and before anything reads
serial. `None` by default, so every other boot path is unchanged — the vm.py
diff is a pure addition, no line removed.

**Why a hook rather than a second QMP socket.** Opening our own monitor
alongside the harness's would have been a much smaller change and it is the
wrong one: replay reach is measurably sensitive to how many host-side backends
are attached — the same trace reached 1773 guest lines with a file serial and no
QMP, and 23 lines with a file serial and QMP (see the host-shape entry in the
divergence log). Adding a second monitor to study replay would perturb the thing
being studied, and the resulting numbers would look like findings.

The hook is deliberately not wrapped in `try`/`except`. A swallowed failure
would leave the caller believing a break was armed when it was not, and that
presents as "the break never fired" — indistinguishable from a break landing at
an illegal icount, which is a real and separate failure.

## Superseded: the earlier conclusion that a replay cannot be stopped

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
  over virtio-blk regardless of what rr wants.

  **The blocker is OURS, not QEMU's — corrected 2026-09-13.** This section used
  to end "making x86 recordable therefore needs blkreplay to handle ATAPI — a
  QEMU-side fix, or a newer QEMU". That was inferred from one non-ATAPI transport
  failing. A second one says otherwise.

  `usb-storage` is neither of the two models the transform refuses, and QEMU
  accepts `blkreplay` over it without complaint — a probe recorded 94MB of
  `replay.bin` while OVMF booted the ISO from it:

  ```
  BdsDxe: starting Boot0001 "UEFI QEMU QEMU USB HARDDRIVE 1-0000:00:04.0-1"
  UEFI EntryPoint: Starting freeldr from UEFI
  [freeldr] MachInit done ... Boot devices initialized
  [freeldr] Switching stack...          <- and no further
  ```

  Same wall as virtio-blk, to the line. And the control settles it: **the same
  usb-storage boot with no `-icount` and no `blkreplay` stops at the identical
  `[freeldr] Switching stack...`, same 10 lines.** rr is not involved.

  So the summary for x86 is:

  | transport | blkreplay | boots |
  |---|---|---|
  | `ide-cd` (ATAPI) | **fatal** | yes (7214 lines) |
  | `virtio-blk-pci` | fine | no — `Switching stack...` |
  | `usb-storage` | fine | no — `Switching stack...`, and also without rr |

  Two independent non-ATAPI transports, the same failure point, reproducing
  without rr. That is a freeldr boot-device limitation, not an rr limitation and
  not a QEMU one. Waiting on a newer QEMU would not have helped.

  **DIAGNOSED 2026-10-03, and "get freeldr past `Switching stack...`" was the
  wrong goal: freeldr already gets past it.** The silence after that line is the
  output CHANNEL changing, not a crash — every print from there on went to
  `TRACE`/`WARN`, which the serial log does not carry, while everything before it
  used `ConOut`, which it does. Three permanent `ConOut` probes
  (`freeldr-rust/src/uefildr.rs`, `uefidisk.rs`) now say what happens, and
  `VIBEOS_QEMU_BOOT_TRANSPORT=usb-storage|virtio-blk` (new, `vm.py`) makes the
  non-ATAPI boot reproducible through the harness instead of a hand-rolled QEMU
  line that bypasses the staleness and media guards.

  | | control (`ide-cd`) | `usb-storage` |
  |---|---|---|
  | `[FLDR] arc branch` | `CDROM` | `HARDDISK+partition` |
  | survived `_changestack` | **yes** | **yes** |
  | next line | `[FLDR] phase: system hive load begin` → desktop | `[FLDR] RunLoader RETURNED (no boot config)` |

  So the real failure is `RunLoader` finding **no boot configuration** on
  `multi(0)disk(0)rdisk(N)partition(M)`. `UefiGetBootPartitionEntry` did NOT fail
  (its probe never fired), so the path was computed and simply does not resolve.

  **Root cause: the CD test is a DEVICE-MODEL test standing in for a filesystem
  question.** `UefiSetBootpath` picks the `cdrom(N)` ARC name only when
  `RemovableMedia && BlockSize == 2048`. An ISO served over usb-storage or
  virtio-blk reports 512-byte blocks, fails that test, and takes the hard-disk
  arm — where freeldr then looks for a partition table and a FAT/NTFS volume that
  an ISO9660 image does not have.

  The control that pins it: **arm64 takes `HARDDISK+partition` and boots fine**,
  because its `boot.img` really is a FAT disk. The hard-disk arm is not broken;
  ISO9660-behind-it is unsupported.

  **The fix for x86 rr** is therefore to decide the ARC name by what the medium
  IS rather than how the device presents — detect ISO9660 and use `cdrom(N)`
  regardless of block size — or to teach the rdisk path to mount ISO9660. The
  first is smaller and is where the bug is.

  **Loader half FIXED 2026-10-04; the claim that rr then "follows with no rr changes"
  was wrong.** There were two gates, not one: `UefiSetBootpath` chose the arm, and
  `UefiInitializeBootDevices` registered the `cdrom(N)` device behind a second copy of
  the same 2048-byte test, so the first fix alone produced `arc branch = CDROM` and
  then the identical `RunLoader RETURNED (no boot config)`. Both now follow one
  question -- does the device carry an ISO9660 volume (`UefiMediumIsIso9660`) -- and
  over usb-storage freeldr loads the hive, the NLS files and every boot driver from
  `cdrom(0)`. **The kernel then stops at `0x6B`**: QEMU's usb-storage presents
  "QEMU HARDDISK", so usbstor hands it to `disk.sys`, no CD-ROM device object exists,
  and cdfs (which mounts only on CD-ROM devices) has nothing to mount the system
  volume on. Owed for x86 rr: either a USB CD presentation (`usb-bot` + `scsi-cd`,
  which `blkreplay` refuses -- see above) or ISO9660 mounted from a disk-class device.
  The prediction was made without booting past the loader; the boot that would have
  checked it was one command away.

  **RESOLVED the same day**: usb-storage boots to the desktop and records and replays.
  The kernel and harness layers are listed under "What it refuses" near the top.

  Worth noting how the wrong conclusion was reached, since it is cheap to repeat:
  one transport was tested, it failed, and the cause was attributed to the thing
  that transport had in common with rr's requirements rather than to the thing it
  had in common with the other failing transport. The second sample was one
  command away.

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

  **ANSWERED: the two traces do NOT share a boot prefix that far, and the
  same-guest-point hypothesis loses.** Comparing the two RECORDS line by line,
  they diverge at guest line **1,120** — 900+ lines before either replay stalls:

  | | clean1 | kmt3 |
  |---|---|---|
  | records diverge from each other | line 1,120 | line 1,120 |
  | replay stalls at | line 2,070 | line 2,045 |
  | last line replayed | `user32 ... hIconSmWindows` | `[SERVICES] Initialized, starting auto-start services` |
  | where that line sits in the OTHER record | 1,974 | 1,956 |

  Read the last row: each replay ran PAST the guest point where the other one
  stopped. They stall at different guest work, in different order, having
  diverged 900 lines earlier — and still land within ~1% of each other on every
  measure (2,070 vs 2,045 lines, 122,374 vs 120,380 serial bytes, trace offset
  2,145,086 vs 2,163,558). That is the signature of a structural limit, not of a
  guest instruction sequence replay cannot execute.

  Note the trace offsets are ~2.15MB in both, on traces of 10.7MB and 494MB —
  0.86% apart on files 46x different in size.

  **Which quantity is actually invariant — and icount is the odd one out.**
  Ranked by how close the two runs are:

  | measure | clean1 | kmt3 | spread |
  |---|---|---|---|
  | trace offset | 2,145,086 | 2,163,558 | 0.86% |
  | guest lines | 2,070 | 2,045 | 1.2% |
  | serial bytes | 122,374 | 120,380 | 1.6% |
  | **icount** | **10,513,077,852** | **11,267,275,810** | **7.2%** |

  icount is ~8x looser than the other three. Two samples cannot separate 0.86%
  from 1.6%, so trace bytes / lines / serial bytes remain indistinguishable — but
  the icount spread is different enough to argue AGAINST a virtual-time or
  instruction-budget limit, and in favour of something counted in log bytes or
  output volume. That matters because "clock machinery" was the standing guess
  (`EVENT_CP_CLOCK_WARP_ACCOUNT` being the second most common record), and this
  is the first evidence pointing away from it.

  **Also disconfirmed: it is not block I/O.** The appealing story was that a new
  process image load parks a `blkreplay` completion that never gets released —
  it fits "burning CPU while consuming nothing", and both replays stop right as
  user-mode processes begin spawning. It is wrong. All 2,277 `EVENT_ASYNC`
  records in clean1 sit **before trace offset 162,166**, the first 1.5% of the
  log, and there are none at all after it. This is a ramdisk boot: after the
  initial load the guest does no further block I/O, so there is no parked
  completion to deadlock on. (Checked before writing it down, which is the only
  reason it is here as a disconfirmation rather than as a finding.)

  **What the wedge does correlate with.** Binning both traces by event type at
  200KB granularity puts the stall at the onset of the densest exception and
  interrupt stretch in each — and the structure is the same in both despite the
  guest divergence:

  ```
  offset           INTR   EXCEPT     WARP    CHARW     INSN
  1800000            15      655      318    13871    14830      clean1
  2000000           582     4550     5039    10300    19310   <- wedge
  2200000           583     9410     8573     6703    24105      (exception peak)

  1800000            87     2055     1821    12643    16433      kmt3
  2000000           705     6768     5895     9008    20970   <- wedge
  2200000           542     9385     6796     7467    23107      (exception peak)
  ```

  Exceptions rise 3-7x and interrupts 8-40x in the wedge bin, and both replays
  stop just before the peak. Mechanistically that is the guest's first sustained
  user-mode multitasking: demand-paging faults plus timer preemption. In clean1
  `EVENT_INTERRUPT` and `EVENT_CP_CLOCK_VIRTUAL` are equal to the record (902
  each), i.e. one virtual-clock checkpoint per timer interrupt — which is where
  a clock/interrupt replay bug would live, and `EVENT_CP_CLOCK_WARP_ACCOUNT` is
  the second most common record in the whole log.

  This is a correlation with a plausible mechanism on two samples, not a cause.
  It is, however, a much narrower place to look than "somewhere in the trace".

  **RETIRED: the exception correlation is the WORST fit of any measure, not the
  best.** Counting every candidate quantity at both wedge points — the check that
  should have been run before writing the paragraph above — ranks them:

  | measure | clean1 | kmt3 | spread |
  |---|---|---|---|
  | trace bytes | 2,145,086 | 2,163,558 | **0.85%** |
  | guest lines | 2,070 | 2,045 | 1.21% |
  | serial bytes | 122,374 | 120,380 | 1.63% |
  | EVENT_INSTRUCTION records | 180,516 | 185,952 | 2.92% |
  | total events | 384,921 | 397,601 | 3.19% |
  | icount | 10,513,077,852 | 11,267,275,810 | 6.69% |
  | EVENT_CP_CLOCK_WARP_ACCOUNT | 47,548 | 51,247 | 7.22% |
  | EVENT_INTERRUPT | 6,222 | 6,826 | 8.85% |
  | **EVENT_EXCEPTION** | **14,492** | **18,823** | **23.01%** |

  Exceptions differ by 23% between the two stalls. Whatever stops the replay, it
  is not "after N exceptions" — the density spike is something both boots happen
  to be doing at the point they stop, not the thing being counted. The
  binned-histogram observation above stands as an observation; its suggested
  mechanism does not.

  **What the ranking does say.** The measures split cleanly in two. Log-position
  quantities (trace bytes, lines, serial bytes) cluster at 0.85-1.63%;
  guest-work quantities (events, icount, clock warps, interrupts, exceptions)
  spread from 2.92% to 23%. The limit is positional in the LOG, not a count of
  anything the guest did.

  **`rr wedge <trace>` makes this a one-command check.** The decode above needed
  QEMU's own `scripts/replay-dump.py` fetched from the matching stable tag; the
  in-tree walker now answers the same question. `rr wedge` asks a live stalled
  replay where it stopped and prints the events either side, or takes
  `--at <offset>` for a position recorded earlier. It deliberately prints a
  window and names no culprit, for the reason in the next paragraph.

  **The two offsets quoted above are not the same measurement.**
  `trace_read_position` froze at byte **2,342,912**; the table's "trace bytes"
  figure for the same stall is **2,145,086**.

  **RETRACTED, same day, by the next measurement: the 197,826-byte difference is
  NOT kernel read-ahead.** That is what this paragraph originally said, and a
  fresh trace disproves it. Read positions are block-granular — 2,342,912 is
  exactly 572 x 4096 and the new trace's 798,720 is exactly 195 x 4096 — and on
  the new trace the decode frontier sits **5 bytes** before the fd offset, with
  the straddling `EVENT_CHAR_WRITE` needing 4 more bytes than have been read. So
  read-ahead is bounded by ONE 4 KiB block, and cannot account for 193 KiB. The
  2,145,086 figure must therefore be a different quantity (guest-progress
  derived), and comparing it against an fd offset as though both measured "where
  the replay is" was the error. Note what the block alignment does and does not
  buy: it is guaranteed by the read granularity, so "the stall is at a block
  boundary" is a property of the instrument, not a finding.

  The reason a tool must still refuse to name the event at the read position is
  unchanged, and now better supported: the fd offset is an upper bound on the
  replay's logical position, so `rr wedge` prints a window and labels BEFORE as
  the evidence.

  **The open caveat on "positional in the log": the two tightest measures are
  nearly collinear.** This trace is dominated by serial output, so trace bytes
  and serial bytes are not independent — trace-bytes-per-serial-byte is 17.53
  on clean1 and 17.97 on kmt3, a 2.5% spread. Trace bytes (0.85%) does fit
  better than serial bytes (1.63%), which is real evidence for position over
  volume, but it is two samples separated by less than a factor of two in
  tightness. **The discriminating experiment is a third recording with
  deliberately different serial density** — the same boot with debug printing
  turned down, or up. If the wedge is positional in the log, the trace offset
  stays put and the serial-byte count moves; if it tracks serial volume, the
  opposite. Until that runs, "positional in the LOG" is the better-fitting of
  two correlated explanations rather than an established one.

  Worth stating because it is the cheap half: whatever the measure turns out to
  be, ~2.145MB is not a round number, a power of two, or a multiple of 32 KiB
  or 64 KiB (65.46 and 32.73 respectively), so no obvious fixed-size buffer
  explains it yet.

  **2026-10-03: none of the above reproduces, and the host QEMU is no longer the
  one it was measured on.** Every figure in this section was taken on QEMU
  **9.0.2**. After the machine rebuild the host carries **8.2.2**
  (`QEMU emulator version 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.18)`), which is a
  DOWNGRADE, and `rr info` reports it per trace — check it before comparing any
  number here against a fresh one. One thing is unchanged and worth recording:
  8.2.2 writes the same replay log version **0xe0200c** (v12), so the in-tree
  decoder, `rr breaks` and the instruction-chunk rule all apply untouched.

  Two fresh arm64 `--smp 1 --payload none --shift 3` traces, recorded minutes
  apart off identical media with identical commands, replayed to **two different
  failure modes**:

  | | wedge-shift3 | wedge-b |
  |---|---|---|
  | trace size | 3,113,641 | 3,044,334 |
  | record guest lines | 2,117 | 2,117 |
  | replay reached | 835 lines | 1,103 lines |
  | mode | **WEDGE** — read position frozen at 798,720 (25.65%) for 8 samples over 4m38s at 85% CPU | **ABORT** — `Missing character write event in the replay log`, reproduced on two independent replays at the identical guest line |

  So:

  * **The "fixed trace offset" invariant is gone.** 798,720 is not 2,342,912,
    and it is not the same fraction either (25.65% vs clean1's 22% vs kmt3's
    0.44%). Guest lines do not match either (835 vs ~2,060). Whatever held
    across clean1 and kmt3 does not hold here, and those two shared a guest
    build while this one is 450 commits later on a different QEMU — so the
    change is NOT attributed; either variable could carry it.
  * **A third failure mode exists, and it is the most informative one yet.**
    `Missing character write event in the replay log` is QEMU detecting
    DIVERGENCE: the guest emitted serial output the log has no record of at that
    position. That is not a stall, it is replay going off-script, and QEMU's own
    consistency check caught it. It is deterministic — same message, same guest
    line, two replays.
  * ~~**Both modes implicate character writes**~~ — **RETRACTED by the third
    sample, see below.** The abort does name `EVENT_CHAR_WRITE` and the first
    wedge did sit mid-char-write, but a second wedge sits in an exception burst
    with no char-writes anywhere near it.
  * **Two traces of the same boot failing differently means replay is not
    reliably deterministic here at all**, which reframes the wedge: it may be
    one symptom of a divergence that sometimes stalls and sometimes trips the
    char-write check, rather than a structural ceiling.

  **Third sample (`div-b`, 2026-10-03): a second WEDGE, in a completely
  different kind of event burst — which kills the char-write lead.** Window at
  the frozen offset:

  ```
  @921590  icount 9691286144  EVENT_CHECKPOINT (id 31)
  @921591  icount 9691286619  EVENT_INSTRUCTION
  @921596  icount 9691286619  EVENT_CHECKPOINT (id 31)
  @921597  icount 9691289919  EVENT_INSTRUCTION
  ```

  No `EVENT_CHAR_WRITE` at all — checkpoints, instruction chunks and id 2
  (exceptions). The first wedge sat in a uniform char-write burst, so "the wedge
  happens mid-serial-output" was a property of that one sample, not of the wedge.

  All three samples on QEMU 8.2.2, same `--smp 1 --payload none --shift 3`:

  | trace | trace bytes | mode | frozen offset | % of trace | replay lines / record lines |
  |---|---|---|---|---|---|
  | wedge-shift3 | 3,113,641 | WEDGE | 798,720 | 25.65% | 835 / 2,117 |
  | wedge-b | 3,044,334 | ABORT (missing char write) | — | — | 1,103 / 2,117 |
  | div-b | 7,695,461 | WEDGE | 921,600 | 11.98% | 876 / 4,352 |

  What the third sample settles:

  * **The limit is not a FRACTION of the trace.** 25.65% vs 11.98% on traces
    2.5x apart in size.
  * **It is roughly absolute, and roughly stable:** 798,720 and 921,600 are 15%
    apart (both exact multiples of 4096, which is the read granularity and
    therefore not itself informative). The historical pair sat at ~2.34MB on
    QEMU 9.0.2 with a different guest, 0.86% apart from each other. So the
    pattern is "tight within a (QEMU, guest) pair, moves between pairs" rather
    than a universal constant — which is why the 2.145MB figure did not
    reproduce and why chasing a round number in it was never going to work.
  * **Guest lines reached is just as tight as the offset**: 835, 1,103, 876 —
    a ~14% band, while the records behind them are 2,117 / 2,117 / 4,352 lines.
    A replay gets ~900 lines into the boot regardless of how long the recorded
    boot was. Byte-position and guest-progress cannot be separated on this data.
  * **Two of three wedge, one aborts, same recording command minutes apart.** The
    failure MODE is not a property of the configuration.

  **The `--shift` discriminator does not exist.** The plan was to break the
  trace-bytes/serial-bytes collinearity by recording at a different icount
  shift, keeping guest serial fixed while changing instruction-record density.
  It cannot be run: shift 1 and shift 5 both stall in the first 4,096-byte
  buffer having produced 271 bytes of serial, so only shift 3 replays at all.
  `rr record` warns about this at record time and this document already said so
  — the plan was made without re-reading it. Finding a knob that varies serial
  density at shift 3, during early boot, is still open.

  **The divergence abort, pinned down: the replay is byte-exact right up to the
  moment it fails.** Three measurements on `wedge-b`, which is the most specific
  thing rr has produced so far:

  * The replay's guest serial is **byte-for-byte identical to the record for
    1,100 guest lines** (excluding the two harness header lines), and then stops
    exactly where the record carries on — the record's next line is
    `[IRQL-CHK] PspCreateProcess-after-InitSecurity irql=0`.
  * The log is **provably complete** (the in-tree walk reaches a shutdown event)
    and holds **164,964 `EVENT_CHAR_WRITE` records**. Full mix: 264,783
    `EVENT_INSTRUCTION`, 164,964 char-writes, 74,621 checkpoint id 31, 29,680 of
    id 2, 9,672 of id 29, 7,379 of id 1; 564,976 events total.
  * It is **deterministic**: same message, same guest line, two independent
    replays.

  So `Missing character write event in the replay log` is NOT truncation, NOT a
  short log, and NOT the guest doing something different — the serial matches.
  QEMU is reporting that the guest asked to write a character at a point where
  the log's next event is not a char-write. That reads as **icount alignment
  drift**: the replay arriving at the same guest work at a different instruction
  position than the recording wrote down. That is a far more tractable failure
  than "it stalls somewhere", because QEMU names the invariant that broke.

  **Record and replay must bracket any media rebuild, and this is easy to get
  wrong.** Both traces above became unreplayable when a later `vibeos-cli build
  --arch arm64` (run to warm the land receipts) rewrote `qemu/boot.img` at 11:16
  — the trace's `boot.qcow2` is an OVERLAY over that file, so the guest would
  read bytes that were never recorded. `rr verify` catches it and refuses by
  name, which is the only reason the aborts above are still trustworthy: the
  rebuild landed AFTER both replays (10:41 and 10:47), so the media was
  consistent while they ran. Check the mtimes before believing any
  record/replay pair that straddles a build.

  Unexplained, recorded so it is not mistaken for a result: the same `rr verify`
  run printed `PARTIAL — agreed for 1416 lines; record has 2430, replay 1416`
  alongside its own "not replaying" refusal. Those counts match neither trace
  (`wedge-b` is 2117/1103). Nothing here is built on them.

  **Next experiment:** re-record (required anyway, the media moved), replay, and
  capture the fd offset at the abort so `rr wedge --at` can place it in the log —
  specifically whether the abort lands mid-char-write-burst the way the wedge
  did. If both failure modes sit in the same kind of position, they are probably
  one defect with two presentations.

  Two caveats worth keeping, because they bound how much the top row is worth.
  The three tight measures are near-proportional to one another in this region —
  the log here is a uniform `EVENT_CHAR_WRITE` / `EVENT_INSTRUCTION +3` pattern,
  so trace bytes ~ 14x char-writes ~ 14x serial bytes — so 0.85% versus 1.63% is
  not a real separation on two samples. And neither 2.15MB nor any of the other
  numbers is a round figure or a power of two, so no obvious buffer or window
  size explains it yet.

  **The replay gets FURTHER than the earlier framing suggests.** "22% of the
  trace" is a byte fraction of a file dominated by idle-time clock records, and
  it undersells the result badly. By guest progress, clean1's replay reproduces
  **2,070 of 2,070 lines identical to the record, zero divergence — 65.7% of the
  recorded boot — and it reaches `[WIN32K] shell ready`**, the win32k half of the
  desktop marker. Quote guest lines, not trace bytes, when describing how far a
  replay got.

  Caveat on the byte figures: the frozen read position (2,342,912 = 572 x 4096)
  is a page-aligned READ-AHEAD boundary and runs ahead of execution. The
  execution point, recovered by mapping emitted serial bytes back to their
  `EVENT_CHAR_WRITE` index, is 2,145,086 — ~198KB behind it. Use the read
  position to decide whether a replay is moving; use the char-write mapping when
  you want to know where it actually is.

  **SETTLED by varying the icount shift, and the answer rules out BOTH.** The
  clean discriminator is to record the same guest boot with a different encoding
  density: `--shift 5` produced a 5,999,935-byte trace where `--shift 3` produced
  10,666,166 for the same boot — 44% denser — so a fixed FILE offset and a fixed
  GUEST point predict different stall locations.

  | shift | trace size | stalled at | guest serial | where |
  |---|---|---|---|---|
  | 3 | 10,666,166 | 2,342,912 (22%) | 124,212 | desktop bringup |
  | 5 | 5,999,935 | **4,096 (0.1%)** | **271** | still at the UEFI banner |

  Neither hypothesis survives: 4,096 is nowhere near 2,342,912, and 271 bytes is
  nowhere near 124,212. **Replay reach is a function of the icount shift.**

  And it is not monotonic — a third run settles that, and it is the practical
  result of this whole investigation:

  | shift | trace size | stalled at | guest serial | where |
  |---|---|---|---|---|
  | 1 | 33,571,609 | **4,096** | 271 | UEFI banner |
  | 3 | 10,666,166 | 2,342,912 (22%) | 124,212 | desktop bringup |
  | 5 | 5,999,935 | **4,096** | 271 | UEFI banner |

  **Only the default shift replays at all.** 4,096 bytes is the first read
  buffer: at shift 1 and shift 5 the replay never gets going. That also means the
  stall at 2,342,912 is a *different* phenomenon from the immediate stall at 1
  and 5 — one is a replay that ran most of a boot and stopped, the other is a
  replay that never started.

  **Replay reach also depends on backends the GUEST CANNOT OBSERVE, which
  invalidates hand-built replay command lines.** Same trace, three
  configurations:

  | serial backend | QMP attached | guest lines reached |
  |---|---|---|
  | `file:` | no | 1773 |
  | `file:` | yes | **23** |
  | socket chardev (the recording's own) | yes | 1693 |

  Two orders of magnitude, from a monitor socket and a chardev kind. Confirmed
  with a control: the 23-line run reproduces with QMP attached and *never
  queried*, so it is not the act of querying.

  This is worse than a crash — a replay that quietly under-reaches looks like a
  finding about the guest, and several observations earlier in this document were
  taken from hand-built command lines for exactly that reason.
  `assert_same_shape` now compares the host-side shape (`-serial`, `-qmp`,
  `-monitor`, `-chardev`, `-display`) by backend KIND, ignoring per-run paths and
  ports, and refuses a mismatch. **Replay through the harness that recorded;
  do not hand-roll the command line.**

  (QMP's replay API is worth knowing about regardless: `query-replay` returns the
  live `icount`, which is a direct read of replay progress — better than
  inferring it from the rrfile read position — and `replay-break` / `replay-seek`
  exist, which may be the replay-native way to stop one, where the gdbstub
  cannot.)

  **The traces are identical at the head and diverge in CHARACTER within
  4KB.** Decoding all three: the first thirteen events match structurally,
  including `EVENT_INSTRUCTION(0) + 336314 -> 336314` at the same instruction
  count; only host-clock values differ. But at comparable file offsets the event
  mix has already parted:

  ```
  shift 5, @4000   982:EVENT_CHAR_WRITE(22) 1 -> 1
                   983:EVENT_INSTRUCTION(0) + 25 -> 381953882      (unbroken run)

  shift 3, @4000   784:EVENT_INSTRUCTION(0) + 1249581 -> 407930319
                   785:EVENT_CP_CLOCK_WARP_ACCOUNT(31)
                   786:EVENT_CP_CLOCK_VIRTUAL(34)
                   788:EVENT_INTERRUPT(1)
  ```

  So at shift 3 the early trace carries periodic timer interrupts and clock
  checkpoints separated by hundred-thousand-instruction runs, while at shift 5 it
  is an uninterrupted char-write/`+25` sequence. Recorded as an observation, not
  a mechanism — it says the shift changes what the stream is MADE OF that early,
  which is a more specific fact than "reach depends on the shift" and is where
  anyone chasing this should start.

  The trap this creates is the reason `rr record` now warns on a non-default
  `--shift`: RECORDING succeeds at every value, so the failure is invisible until
  someone tries to replay, by which point the expensive part is already spent. (That also reproduces an observation from the
  first night of this work, when a shift=5 replay sat at the UEFI banner at 271
  bytes for eight minutes and was written off as contention.)

  This points at the clock/timing machinery rather than at a byte limit or a
  particular guest code path, which fits the event census: `EVENT_CP_CLOCK_WARP_
  ACCOUNT` is the second most common record in the trace at 1,178,159. A larger
  shift means coarser virtual time per instruction, so clock-warp accounting
  changes character — and it is the one thing known to differ between these two
  runs.

  Within a single shift it stays reproducible: two different traces at shift=3
  both stalled in desktop bringup, and the detector reports the position, so a
  future fix has a cheap before/after.

  **There is a modest signal, in WHICH measure agrees better** (among the
  shift=3 traces only). Both replays stop
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

## What this means for the chartered case (record a kmtest sweep, replay it)

**Recording works. Replaying does not reach the sweep — it falls about 24KB of
serial short.** Measured on trace `kmt3`, against its own recording:

| marker | byte offset in the recorded serial |
|---|---|
| `[WIN32K] shell ready` | 74,173 |
| `[EXPLORER] _tWinMain entered` | 118,113 |
| **replay stalls** | **122,199** |
| `[KMTEST-SWEEP]` begins | 146,694 |
| first `Starting test ...` | 176,825 |

So the replay gets through firmware, the whole kernel boot, `shell ready` and
explorer's entry — and stops **24,495 bytes before the sweep launches**, out of
1,348,868 bytes recorded. The sweep itself is never replayed at all.

That is the honest bound on this tool for the job it was built for. It also
makes fixing the wedge worth more than "replay is incomplete" suggests: the gap
is small and quantified, not an unknown distance.

Until then the chartered flow is **record-only**: a kmtest sweep IS captured
deterministically and completely (`launched=91 ok=88 bad=3`, 489MB trace), and
that capture is re-runnable up to desktop bringup. What it cannot yet do is
re-run the sweep.

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
