# Handoff: arm64 intermittent boot hang at `[freeldr] Switching stack...`

**Audience:** Codex (or any agent picking up the arm64 boot-flake investigation).
**Worktree:** `vibeos-wt-ntoskrnl`, branch `ntoskrnl-cargo-deepen` (even with
`continue-rust-port` as of merge `78b33a87ad1`).
**Status:** ✅ RESOLVED (2026-06-06). Root cause was **H1** below: `bootvid.dll`
(an early-loaded image) pulled external `Rtl*`/`_stricmp` imports that didn't
resolve cleanly in the loader context, stalling image load → UEFI watchdog reset.
Fix made `bootvid-rust` self-contained (local `memcpy/memset/memcmp/_stricmp`,
`core::ptr` intrinsics instead of `Rtl*`), plus arm64 user-callback state fixes
(`usercall.rs`) for a downstream 0x50 and smss loop/init corrections. Verified
6/6 consecutive arm64 boots green + cold i386/amd64. The investigation notes below
are kept as a record. Implemented via Codex CLI; validated independently.

---

## TL;DR

arm64 **intermittently** fails to boot: the loader (`RunLoader`, post
`[freeldr] Switching stack...`) loads a few PE images and then **stalls
mid-load**, after which the **UEFI watchdog (~5 min) resets the machine** — the
smoke harness records this as `HANG_TIMEOUT (~9 min)`. It is **flaky, not
deterministic**: across today's runs the same tree booted green ~8 of 9 times,
and a green boot goes **all the way to `[EXPLORER] _tWinMain entered`** (full
desktop), so arm64 is fully capable when it doesn't trip.

**Two important corrections to the first read of this bug** (which thought it
"hung at the `Switching stack` marker"):
1. It does **not** sit at the marker — `RunLoader` proceeds and loads several
   images (`LI[...]` debugcon lines, alloc counter `na` climbing) before
   stalling. The marker is just the last *human-readable* `[freeldr]` line;
   subsequent progress is in terse debugcon markers.
2. It is **not confirmed warm-only.** A `--clean`-flagged build also reproduced
   it (caveat: that build finished in ~28 s, so it was likely incremental-cached
   underneath — see Step 0; confirming a *true* from-scratch cold repro/avoidance
   is the first open question).

So the `KNOWN_CGU_UD2` "lottery" (codegen-unit-placement-sensitive miscompile) is
**a** hypothesis but no longer the only one — the stall correlates with **UEFI
memory-allocation anomalies** (see Symptom), which points equally at a real
intermittent bug in the arm64 loader's image-load / page-allocation path.

**The job is to root-cause and kill the intermittency**, not to find a clean
revert (there isn't one — it predates and is independent of the recently reverted
cargo-builtin-target migration, see
[`cargo-drive-unification-plan.md`](./cargo-drive-unification-plan.md)).

This is a *hard, real* bug worth deterministic root-causing. Treat the flake rate
as the metric: a fix is "N consecutive boots green," not "it booted once."

---

## Symptom

Actual tail of a failing boot (`/tmp/vibeos-cli/20260606-111308-smoke-arm64/full.log`):

```
[freeldr] Boot devices initialized
[freeldr] Switching stack...
LI[ib=00400000,sz=002B9000,pa1=00000000,pa2=42192000]      <- LoadImage: ntoskrnl (~2.8MB @ 0x400000)
a1a2a3{na=0000007F,fl=0000054A:0000FAB5 ...}a4a5a6a7a8a9     <- alloc trace (na = alloc count)
LI[ib=00000000,sz=00006000,pa1=00000000,pa2=4244B000]      <- next module
a1a2a3{na=00000082,...}a4a5a6a7a8a9
LI[ib=00000000,sz=00005000,pa1=00000000,pa2=42451000]
a1a2a3{na=00000085,...}a4a5a6a7a8a9
LI[ib=00000000,sz=00009000,pa1=00000000,pa2=42456000]
a1a2a3{na=00000088,fl=000005B1:0000FA4E ...}a4a5a6a7a8a9     <- LAST output, stall here
Watchdog Timer resetting system                              <- UEFI watchdog fires ~5 min later
UEFI firmware (version edk2-...)                             <- firmware reboots; smoke -> HANG_TIMEOUT
```

- So the loader **advances** past the marker (`na` counter 7F→82→85→88) then stops.
- **UEFI memory-allocation anomalies are present earlier in the same boot** — these
  are the strongest concrete lead:
  ```
  ConvertPages: failed to find range 140000000 - 14004FFFF
  Mmck[...]CConvertPages: Incompatible memory types, the pages to allocate have been allocated
  ```
  Page-allocation landing on conflicting/already-allocated ranges would explain
  both the stall and the intermittency (UEFI memory map / placement varies run to
  run).
- Verdict: `FAIL HANG_TIMEOUT` (~9m01s) on failing runs; the underlying cause is a
  loader stall + **UEFI watchdog reset**, not a CPU lockup at the marker.
- A screenshot is saved each run (`/tmp/vibeos-cli/<ts>-smoke-arm64/boot0/screen.png`).
- Other arches (i386/amd64) are **not** affected.
- A **green** arm64 boot reaches `[EXPLORER] _tWinMain entered` (full desktop) —
  e.g. `/tmp/vibeos-cli/20260606-045535-smoke-arm64/full.log`.

### Flake evidence (today, same tree)

Tally of recent `/tmp/vibeos-cli/*smoke-arm64*` verdicts:

```
045535: OK        042005: OK        035147: OK        032105: OK
030100: OK        024413: FAIL HANG_TIMEOUT          024808: OK
024249: OK        023721: OK
```

≈1-in-9 failure rate. **Same source tree, same arch, different outcomes** ⇒
nondeterministic ⇒ codegen/placement-sensitive, not a logic regression.

---

## Where it is in the code (precise)

The live arm64 loader entry is still **C** (`uefildr.c`). The Rust translation
(`freeldr-rust/src/uefildr.rs::EfiEntry`) exists but its `#[no_mangle]` is gated
behind the off-by-default `uefildr-rust-entry` feature, so the C body is what
prints the markers and runs.

`uefildr.c` / `uefildr.rs:141-147` — the marker is printed **immediately before**
the stack switch:

```rust
BasicStack = (0x32000 + MmAllocateMemoryWithType(0x32000, LoaderOsloaderStack)) ...;
OutputString("[freeldr] Switching stack...\r\n");   // <-- last thing we see
_changestack();                                       // <-- hang is at/after here
```

The stall happens *inside* `RunLoader` (we see several `LI[...]`/`a1a2a3` markers
from it before the stop), so the path is:

1. **`_changestack` (asm)** — `boot/freeldr/freeldr/arch/uefi/arm64/uefiasm.S`.
   Switches `sp` to `BasicStack`, pushes old SP, `bl ExecuteLoaderCleanly`.
   Uses the documented `adrp + add :lo12: + ldr` pattern (per AGENTS.md the
   `adrp + ldr :lo12:` form generates 8-byte-alignment relocs PE-COFF can't
   satisfy — verify this asm still uses the safe form). *Likely not the culprit*
   now that we know execution reaches RunLoader, but it executes on the freshly
   switched `BasicStack` so a stack-size/placement bug here could corrupt later.
2. **`ExecuteLoaderCleanly`** (`uefildr.rs:153` / C) — sets `UefiServiceStack`,
   calls `RunLoader()`.
3. **`RunLoader`** (`freeldr-rust/src/bootmgr.rs:394`) — **the prime suspect.**
   The loader proper: PE-loads ntoskrnl/hal/boot drivers (each `LI[...]` line),
   allocates pages via UEFI boot services (each `a1a2a3{na=...}` line), then
   `ExitBootServices`, then jumps to the kernel. It stalls partway through this
   load/alloc loop. The `ConvertPages: failed to find range` / `Incompatible
   memory types` errors come from this phase.

**Where exactly does it stall?** The last output is an `LI[ib=0,sz=9000,...]`
followed by `a1a2a3{na=88}` — i.e. it loaded an image and recorded an allocation,
then nothing. Candidates: the *next* UEFI `AllocatePages`/`ConvertPages` call
spins or returns an error the loader doesn't handle (infinite retry); a PE-relocate
loop over bad data; or a `captures(none)`-class miscompile in the load loop. The
UEFI watchdog firing (rather than a CPU exception/bugcheck) means the core is
*alive and looping*, not faulted — consistent with a busy spin or retry loop, not
a wild branch.

**Output caveat:** the `[freeldr]` text lines are EFI `ConOut->OutputString`; the
`LI[`/`a1a2a3` lines are debugcon trace. After `ExitBootServices` ConOut is gone
and arm64 switches to the PL011 UART (`Arm64UartKernelBase`, AGENTS.md "UART Trace
Pattern"). The stall here is **before** `ExitBootServices` (we never see a kernel
marker), so output isn't being lost — the loader genuinely isn't progressing.

---

## Competing hypotheses (rank with Step 1's backtrace)

**H1 — UEFI page-allocation stall (newly surfaced, concrete evidence).** The
`ConvertPages: failed to find range` / `Incompatible memory types` errors are real
and in the failing-boot phase. An unhandled `AllocatePages` failure / unbounded
retry in `RunLoader`'s load loop would stall exactly here and be intermittent
(memory-map placement varies). This is the lead with the most direct evidence.

**H2 — CGU lottery (`KNOWN_CGU_UD2`).** Still plausible:
- Nondeterministic across identical source — the fingerprint of codegen-unit
  placement shifting a latent miscompile in/out of a hot path (memory
  `feedback_parallel_cold_builds_cgu_lottery` / `project_cargo_cutover_endgame_map`).
- The workspace firewall (`lto=false`, `codegen-units=256`) mitigates LLVM's
  `captures(none)` cross-CGU miscompile; arm64 lacks the strong per-arch `ud2`
  scan i386 has, so it's the most exposed.
- *Caveat vs the original framing:* the failure was NOT confined to warm builds
  here (a `--clean`-flagged run reproduced it), which would be unusual for the
  classic warm-only lottery — though that build was suspiciously fast (~28 s), so
  Step 0 must establish a true cold baseline before this is ruled in or out.

**H3 — genuine race / uninitialized memory** (timing-dependent UEFI call,
uninitialized loader buffer) that merely *looks* like the lottery.

H1 and H2 are not mutually exclusive (a miscompile could corrupt the alloc path).
**Confirm with Step 1's backtrace before committing to a fix.**

---

## Recommended investigation path

### Step 0 — reproduce deterministically
The hang is ~10%; you need volume. Loop cold-ish smokes and capture every serial
log + screenshot until you have ≥3 failing captures to compare:
```bash
cd /home/curi/Projects/os-playground/vibeos-wt-ntoskrnl
for i in $(seq 1 12); do ./vibeos-cli smoke --arch arm64 --allow-stale; done
# (each run's artifacts: /tmp/vibeos-cli/<ts>-smoke-arm64/{summary.json,full.log,boot0/})
```
Record: does the failure correlate with warm vs cold builds? Per memory the CGU
lottery is *much* rarer on clean cold builds, BUT in this session a
`--clean`-flagged build still failed — except that build finished in ~28 s, which
is far too fast for a from-scratch ntoskrnl-rust rebuild, so it was almost
certainly incremental-cached. **Do a verified true-cold build first**: nuke the
cargo target dir(s) the arm64 cells use and confirm the rebuild actually recompiles
the big crates (minutes, not seconds), then loop smokes. If true-cold is
consistently green and warm is flaky → CGU placement (H2). If true-cold still
flakes → H1/H3 are in play.

### Step 1 — find the spin (this is the highest-value step)
The watchdog reset (not a bugcheck) means the core is *alive and looping*. Catch
it in the act with the QEMU host gdbstub:
```bash
RUN=$(qemu/start.sh arm64 --gdb)          # boots with gdbstub
# wait until it's deep in RunLoader's load loop (past the marker), then interrupt
qemu/wait-for.sh $RUN --pattern "Switching stack" --timeout 120
# give it ~30-60s to reach the stall, then break + backtrace REPEATEDLY:
qemu/inspect.sh $RUN --command bt 40
qemu/inspect.sh $RUN --command symbolicate-pc <pc>
# break a few times — if PC sits in the same UEFI AllocatePages/ConvertPages
# call or the same RunLoader load-loop block each time, that's the spin.
```
(arm64 freeldr symbolication: `qemu/sym.sh <runtime_addr> <module_base>` —
formula in AGENTS.md "Symbolication". `uefildr` is the module.)
Resolve: is PC in (i) a UEFI firmware AllocatePages/ConvertPages call (→ memory
hypothesis), (ii) a `RunLoader`/PeLdr loop in `bootmgr.rs`/PE-load code (→ loader
logic or CGU miscompile), or (iii) `_changestack`/stack glue?

### Step 2a — memory-allocation hypothesis (NEW, co-equal lead)
The `ConvertPages: failed to find range 140000000 - 14004FFFF` and `Incompatible
memory types, the pages to allocate have been allocated` lines say the loader is
asking UEFI for pages at ranges that are unavailable/already-claimed. If the
loader's response to a failed `AllocatePages` is an unbounded retry (or it ignores
the failure and later spins on the unmapped range), that is the bug and it's
intermittent because the UEFI memory map placement varies per boot. Trace the
allocation sites in `RunLoader` / `MmAllocateMemoryWithType` / the PE loader and
check every UEFI alloc return for an unhandled-error path. The `na`/`fl` fields in
the `a1a2a3{...}` markers are the freeldr heap allocator state — decode them
(grep the marker emit site) to see if the heap/free-list is exhausted or corrupt
at the stall.

### Step 2b — if it's the CGU miscompile
- Compare the `uefildr.efi` (and any RunLoader-path `.a`) between a known-green
  build and a known-hung build: `objdump -d` the relevant functions, diff. The
  lottery typically shows a function the optimizer replaced/elided
  (look for the `ud2`-equivalent on arm64: `brk #1` / `udf`, or a tail that
  falls through to garbage). There is no strong arm64 `scan_ud2` equivalent yet —
  consider extending `tools/scan_ud2.py` to recognize arm64 trap encodings
  (`0xd4200000` BRK family / `udf`) as part of the fix.
- Likely culprits are the same `captures(none)` / output-pointer-elision class
  the i386/amd64 work hit: a `*out = v` that should be `write_volatile`, or an
  `extern "C"` vs `extern "system"` ABI mismatch across an rlib boundary that
  links silently on arm64 (per AGENTS.md "NTAPI vs cdecl ABI"). Run
  `verify_abi.py` / `verify_ffi.py` against the freeldr-rust + RunLoader-path
  crates.
- A deterministic mitigation that has worked elsewhere: force the miscompiled
  function into its own CGU / mark the output write volatile / split the rlib so
  the firewall keeps caller and callee in separate CGUs.

### Step 3 — verify the fix
A fix is **N consecutive green cold boots** (≥10), not one. Re-run the Step 0
loop. Then i386 + amd64 cold smoke to prove no regression on the other arches.

---

## Build / smoke recipe (arm64)

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-ntoskrnl
./vibeos-cli build --arch arm64 --clean     # cold build (nukes rust targets)
./vibeos-cli smoke --arch arm64             # boots ramdisk, idle-detection verdict
```
- **arm64 verdict is idle-detection**, not "shell ready" (the marker x86 uses).
- The cli's arm64 build **skips usermode**; for a usermode change you'd
  `ninja <target>` + `mkramdisk --refresh` manually — but this hang is in the
  *loader*, so the default cli build covers it.
- `summary.json` is the durable verdict; `full.log` is the serial. Don't scrape
  stdout. `qemu/ps` shows live VMs if KVM/TCG is contended.
- arm64 runs under TCG (cortex-a72), so boots are slow (~20-30s to first marker);
  budget timeouts accordingly.

---

## What this is NOT

- **Not** the reverted cargo-builtin-target migration — that's stashed
  (`stash@{0}`) and the baseline reproduces the hang without it. See
  [`cargo-drive-unification-plan.md`](./cargo-drive-unification-plan.md) and
  [`cargo-target-model-builtin-vs-buildstd.md`](./cargo-target-model-builtin-vs-buildstd.md).
- **Not** an i386/amd64 issue.
- **Not** (so far as known) a logic regression from the recent crp merges — the
  intermittency predates them. But a `git log` bisect of the flake *rate* across
  recent crp history is a fair tiebreaker if Step 1 points at a specific function
  that recently changed.

## Constraints (project rules — do not violate)
- Never revert a `.rs` translation to C to dodge this; fix in place.
- Never delete a `.rs` file or re-run a paid API translation without approval.
- Never weaken the smoke test or lower the boot marker to make it "pass."
- The hang being visible in smoke is the correct state until root-caused.
