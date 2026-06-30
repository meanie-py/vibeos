# amd64 LiveCD boot-stall — investigation log (arm64-love, 2026-06-28)


## 2026-06-28 — amd64 boot-stall RE-CHARACTERIZED with reliable debugcon capture (arm64-love)

**Supersedes the earlier "lost completion interrupt (IOAPIC/EOI)" theory below — that was measured on a stale pre-LUNEX kernel (§9.6b).** The Rust `ScsiPortIsr` already emits unconditional debugcon markers (`#I#`/`#i#`/`#J#`); amd64 captures debugcon. Added clean probes (`D`=ScsiPortDpcForIsr entry, `P`=completion drained, `t`=ScsiPortIoTimer tick, `T`=30s SRB timeout, `B`=LUNEX busy-retry, `R`=bus reset; all collision-checked vs the trace catalog).

**Capture gotcha (cost ~2 boots):** `vibeos-cli smoke` writes debugcon to a *per-run* Windows staging dir (`/mnt/c/temp/vibeos-qemu/<pid>-<ts>-smoke-amd64-boot0/debugcon.bin`) that is **cleaned after the run**. Reading the flat `/mnt/c/temp/vibeos-qemu/debugcon.bin` gives a **week-old stale file**. Reliable capture = poll-copy the live per-run file (filtered to `*smoke-amd64*`, to skip concurrent net pytests) during the run. Script: scratchpad `capture_boot.sh` + `analyze.py`.

**Validated findings (boots #4, #6 — both hung at read#3336, ~30s gap):**
- **NOT a lost interrupt, NOT a storm.** ISR fires (I≈J≈7900), **DPC runs (D≈J)**, **completions drain (P≈3990 ≈ the ~3336 reads)** — all interleaved to 99% of the stream, zero trailing pure-`#IJ` run. The storage stack works normally throughout.
- **The proximate stall is UPPER-LAYER:** the serial `[GAP] read#3336 gap=30s` is cdrom's `CdromGapLog` (gap between read-IRP *arrivals*), i.e. the cdfs/Cc/Mm layer took ~30s to *issue* the next read — not a scsiport completion failure. `T`=0/`R`=0: scsiport's own SRB timeout never fired.
- **🔑 NEW ANOMALY: the scsiport IO timer (SRB watchdog) fires only ~4× in a 7-minute boot** (`t`=4). It should fire 1/sec (~460×). `IoStartTimer` *is* called (fdo.rs:664) and `IoStopTimer` is only in the unreached `FdoRemoveAdapter` — so scsiport starts it correctly. The watchdog being dead means a stuck request never gets the 30s-timeout-and-reset recovery → a transient stall becomes a **permanent 241s wedge**.

**Unifying hypothesis (next investigator):** the **amd64 periodic-timer cadence** is wrong/coarse — the global NT IO timer (`IopTimerDpc`, 1/sec, drives every device's IO timer) barely fires. That would explain BOTH the dead SRB watchdog AND plausibly the upper-layer ~30s read-issue stalls (a Cc/Mm wait gated on a coarse 1/sec timer). Connects to the documented **amd64 timer-base-skew class** (MEMORY: `38b63f15b74` KeDelay over-wait fix) — likely a residual. **Next steps:** (1) confirm whether the global IO timer / `IopTimerDpc` re-arms 1/sec on amd64 (instrument or check `ntoskrnl-rust` Io timer + the clock that drives it); (2) check the clock-handoff (~4s in?) where `t` stops. Probes live on `arm64-love` (uncommitted scsiport.rs/scsi.rs debugcon markers — revert before landing).

**Narrowed to ke/timer (2026-06-28 cont.):** the global IO timer is a *periodic* KTIMER — `iomgr.rs:1191` `KeInitializeTimerEx(&IopTimer, 1)` + `iomgr.rs:1192` `KeSetTimerEx(&IopTimer, ExpireTime, 1000 /*ms period*/, &IopTimerDpc)`. A periodic timer auto-re-inserts every Period; no manual re-arm. So `IopTimerDispatch` (→ every device IO-timer routine, incl. `ScsiPortIoTimer`) should fire 1/sec forever. It fires ~4× then stops ⇒ **the amd64 periodic-timer RE-INSERTION is broken** — almost certainly the timer-base-skew class: after firing, the kernel re-inserts the periodic timer at `now + Period`, and on amd64 that lands in a hand-just-passed bucket (or never, due to the InterruptTime/KeTickCount base), so it waits ~a full table revolution or never re-fires. **The fix site is `ntoskrnl-rust` ke/timer periodic re-insertion (KiTimerExpiration / KiInsertTimerTable) on amd64**, sibling to `38b63f15b74` (ke/time.rs base-zeroing, which only covered relative one-shots). A 1-line/region fix once the exact re-insert bucket math is confirmed. This is the likely SINGLE root of the whole amd64 boot-stall + the ~30% gate flake: every 1/sec watchdog/poll (IO timer, and plausibly Cc/Mm balance/lazy-writer periodic waits) barely runs, so transient stalls never recover.

**DECISIVE (boot #7, clock-tick counters in ke/time.rs):** `c`=15255 (clock interrupts), `C`=15284 (KeTickCount advances) — both fire continuously to 99% of the stream, ~1:1. `t`=4 (IO timer). **So the clock cadence AND KeTickCount/sweep-hand are HEALTHY** (sweep covers all 512 buckets ~30×). This **rules out clock-died and KeTickCount-frozen.** The bug is purely in the **timer-table insert/sweep bucket alignment**: the periodic IO timer (`IopTimer`, 1s) is re-inserted into a bucket the sweep never *expires* it from, despite visiting it. Suspect triangle: `KiComputeDueTime`/`KiComputeTimerTableIndex` (ntkrnl-sys inline/ke.rs:1212,1244) · `KiInsertTimerTable` (timerobj.rs:115) · `KiTimerExpiration` Index/Limit catch-up (dpc.rs:196-235) · `KiCheckForTimerExpiration` (time.rs:73). All math looks canonical on read; a numeric detail (bucket.Time bookkeeping / Inserted flag / off-by-one) is wrong. NEXT: hand probe at the periodic re-insert (sweep_hand vs ins_hand vs DueTime−InterruptTime) — 4 re-inserts then it stops, so the 4th's values + why no 5th = the answer.

---

## 2026-06-29 — KiTickOffset partial fix LANDED + remaining root identified (arm64-love)

**Tested a candidate fix and got a decisive negative-but-narrowing result.** Three-way source review (arm64-love + kernel32 + meaniepy) found that `38b63f15b74`'s first-tick base reset zeroes `InterruptTime` + `KeTickCount` but leaves **`KiTickOffset`** at its mid-accumulation residual, so `KeTickCount`'s first advance happens after only `residual/Increment` ticks instead of the full `KeMaximumIncrement/Increment` — a constant phase skew between the KeTickCount advance cadence and the 0-based InterruptTime base. Fix: seed `KiTickOffset = KeMaximumIncrement` in the same first-tick reset block (`ke/time.rs`, amd64-gated). **Committed to `arm64-love`.**

**Boot #10 (this fix, fresh full-relink kernel, ke/time.rs `x`/`e` counters added in KiTimerExpiration):**
- `x`=13295 (KiTimerExpiration *runs*), `e`=14973 (timers *expire*) — **general periodic timer expiration is now active and healthy.** The KiTickOffset fix did restore the tick cadence / expiration machinery.
- **BUT `t`=4 STILL** (ScsiPortIoTimer/IopTimer watchdog still dies after ~4 fires) and **the boot STILL HANGS** (`HANG_TIMEOUT`, `[GAP] read#3336`, ~228s silent). Unchanged from boot #7's pre-fix `t`=4.

**⟹ The KiTickOffset skew was real but is NOT the root of the watchdog death / hang.** General timers expire ~15k× yet `IopTimer` specifically never re-fires.

**Refined root (well-supported — the `t`=4 is *exactly* 4 every boot, deterministic, not a phase-drift over-wait):** `IopTimer` is armed during **`IoInitSystem`** (`iomgr.rs:1191-1192` `KeInitializeTimerEx`+`KeSetTimerEx`, period 1000ms) which runs **before** the first clock tick that does the base reset. So IopTimer's initial `DueTime` is computed against the **pre-reset (huge, ~2.8e14) InterruptTime**. When the first clock tick zeroes the time base, IopTimer's huge DueTime is **stranded** relative to the new 0-based InterruptTime — it never satisfies `DueTime <= InterruptTime` again, so it never re-fires (the ~4 early fires happen around/before the reset). Every timer armed *after* the reset works (hence `e`=14973); only the early-boot timers (IopTimer, and any other pre-reset `KeSetTimerEx`) are stranded.

**The real fix (handed off — deeper than a one-liner):** establish the 0-based time base at **clock-init** (HAL clock setup / `KeStartAllProcessors`), *before* any `KeSetTimerEx` runs — OR re-base/re-insert all already-armed timers right after the reset — OR (cleanest) prevent the bogus large pre-seed of InterruptTime/KeTickCount at the source so no reset is needed at all. The "reset on first clock tick" approach is fundamentally racy: it strands every timer armed during Phase-1 init. Candidate owner: meaniepy / whoever has the amd64 clock-init (HAL `HalpInitializeClock` / `KeSetSystemTime` boot path) context.

**Status of artifacts:**
- `ke/time.rs` KiTickOffset fix: **committed on `arm64-love`** (amd64-gated, restores tick cadence, harmless on i386/arm64 — they don't take the reset path). Real correctness improvement; does NOT make amd64 green on its own. Comment in-code points here.
- All debugcon probes (scsiport `D/P/t/T/B/R`, ke `c/C/z/x/e`): **stripped** (reverted scsi.rs/scsiport.rs/dpc.rs to HEAD; removed the `c/C/z` markers + `_flake_e9` helper from time.rs). The pre-existing `ScsiPortIsr` `#I#/#i#/#J#` markers in scsiport.rs are NOT mine and were left intact.
- Capture tooling (`capture_boot.sh`, `analyze.py`): scratchpad only (throwaway).

---

## 2026-06-29 (cont.) — RESOLVED: meaniepy's clock-init fix BOOTS amd64 + a mechanism correction (arm64-love confirm-boot)

**meaniepy implemented the real fix** (`b73c7efd46e` on c-r-p): establish the 0-based time base at **clock-init** in `KeSetTimeIncrement` (`ke/clock.rs`, amd64-gated, Phase-0 HAL `HalpInitializeClock`→`KeSetTimeIncrement`, *before* `IoInitSystem` arms any timer) — zero KUSER `InterruptTime`+tick + `KeTickCount` right where `KeMaximumIncrement`/`KiTickOffset` are already set — and **delete the racy first-tick `CLOCK_BASE_RESET`** from `ke/time.rs`. Supersedes both `38b63f15b74` and the `3262dfd1018` KiTickOffset partial. Pre-seed mystery confirmed (kernel32 + arm64-love grep): the only writers of KUSER InterruptTime in ntoskrnl-rust+hal-rust are ke/time.rs, so the ~2.8e14 is **loader-left page garbage**, not accumulation — present at IoInitSystem before the first tick.

**Confirm-boot (arm64-love warm harness, merged b73c7efd46e + throwaway `t`/`c`/`C` markers, fresh relink — ntoskrnl.exe 20:03, scsiport.sys 20:02):**
- ✅ **BOOT REACHED shell ready** — `summary.json` boot stage `status:"ok", detail:"reached"`. Where boot #10 hung (HANG_TIMEOUT), it now boots. **The amd64 boot-stall is FIXED.** Markers present (`c`=2157, `C`=2186) confirm the kernel was rebuilt with the fix.
- ⚠️ **MECHANISM CORRECTION — `t`=4 STILL** (watchdog did NOT revive to ~460) yet the boot succeeded. **So the stranded-IopTimer watchdog was NOT the boot-blocker.** The fix unblocks the boot via the **relative-timer cadence on the Cc/Mm/cdfs read path** (the original ~6.3s coarse read-stall — relative KeWait-with-timeout no longer over-waits a full ~512-tick table revolution), not by reviving the SRB-timeout IopTimer. The earlier "stranded IopTimer → dead watchdog → permanent wedge" causal chain was the wrong mechanism — the watchdog death (t=4) is a real but **non-boot-fatal latent issue** (impaired storage error-recovery; the IopTimer may indeed still be stranded). *(Caveat: confirm-boot debugcon capture truncated to ~first 33s — the per-run staging file is cleaned mid-run by the harness; t=4 is a partial-window count but anomalously low for any multi-sec window, so the watchdog clearly didn't reach ~460.)*

**Reusable lesson:** "stall + a dead 1/sec watchdog" does NOT imply the watchdog death *causes* the stall — they can be two faces of one upstream timer-base bug (here: garbage time-base → BOTH relative-timer over-wait on the read path AND early-boot periodic-timer stranding). Fixing the base cleared the boot-fatal face (read-path over-wait) while leaving the non-fatal face (IopTimer stranding) — because IopTimer is armed pre-reset and isn't re-based. Don't over-fit the causal chain to the most visible anomaly.

**Open follow-ups (non-blocking):**
- ~~The IopTimer/ScsiPortIoTimer SRB watchdog (`t`=4) — still likely stranded~~ → **CORRECTED (meaniepy, 2026-06-29): the watchdog death is NOT timer-stranding — it's an empty-stub dispatch.** `iomgr.rs:1190` arms `IopTimerDpc` to `IopTimerDispatch`, but that name resolves to a **local empty stub** `iomgr.rs:176` (`unsafe extern "system" fn IopTimerDispatch(..) {}` — body literally `{}`) which **shadows the real `pub` dispatch at `iotimer.rs:53`** (the one that walks `IopTimerQueueHead` and calls each registered device `IoTimer` routine). So the IO-timer DPC fires 1/sec but does nothing → `ScsiPortIoTimer` (SRB watchdog) and every other device IoTimer routine never run. My "stranded IopTimer / DueTime-vs-pre-reset-base" hypothesis was WRONG — IopTimer fires fine; the dispatch is a no-op. (The `t`=4 I measured — rather than 0 — is incidental: ScsiPortIoTimer gets a few init-path calls and/or the capture was truncated; immaterial to the root.) **Fix = remove/rename the `iomgr.rs:176` stub (or qualify the arming to `iotimer::IopTimerDispatch`) so the DPC binds the real dispatch.** meaniepy has a fix stashed (compiles ×3, but did NOT resolve the net residual below, so it's a separate latent fix needing ×3 validation). A duplicate-symbol/local-shadow class bug — sibling to other translation-grind name collisions.
- **amd64 net suite failed (14 errors) on the confirm-boot, but HEAVILY confounded** (`-accel tcg` fallback = no KVM → timing-sensitive TCP tests fail under TCG slowness; + debugcon-marker spam). Confirmed by meaniepy a **pre-existing host-harness flake** (windows hostfwd/serial; amd64 net was already 14/14-failing on pre-fix mainline `53727b3483a`), NOT the clock fix. **netstack owns it.**

---

## Debug surface — how to instrument the storage-timing stall (for the net-residual hunt)

The amd64-net residual stall (net VM wedges at `Rpcss` with a 30s `[GAP]` under the heavier virtio-net boot load) is the **same storage-timing surface** as this boot-stall — the coherent-0-base clock fix mitigated the lighter smoke path but not the net config's load. Reusable instrumentation, banked so the next hunt doesn't re-derive it:

**1. The proximate stall signal — cdrom GAP log.** `CdromGapLog()` (decl `cdrom.rs:50`, call `cdrom.rs:2789`) emits serial `[GAP] read#N gap=<us>` = the gap between **read-IRP arrivals** at cdrom. A 30s gap = the upper layer (cdfs/Cc/Mm) took 30s to *issue* the next read — NOT a scsiport completion failure. This is the headline "are we stalled and where" signal; grep the serial log for `[GAP]`.

**2. debugcon (port 0xE9) byte markers — sub-millisecond, no serial-flush cost.** `WRITE_PORT_UCHAR(0xE9 as *mut u8, b'X')` (scsiport, in scope via `use crate::inlines::*`) or `core::arch::asm!("out 0xe9, al", in("al") b, ...)` (ntoskrnl `_flake_e9` helper pattern). Capture **gotcha**: `vibeos-cli smoke` writes debugcon to a **per-run** Windows staging file `/mnt/c/temp/vibeos-qemu/<pid>-<ts>-smoke-amd64-boot0/debugcon.bin` that is **cleaned after the run** (the flat `/mnt/c/temp/vibeos-qemu/debugcon.bin` is a stale week-old file — reading it cost me 2 boots). Reliable capture = poll-copy the live per-run file during the run, filtered to `*smoke-amd64*` (skip concurrent net pytests). Harness:

```bash
# capture_boot.sh OUT SMOKE_LOG — boot amd64 smoke + reliably grab the per-run debugcon
set -uo pipefail; cd <repo>
OUT="$1"; SMOKE_LOG="$2"; : > "$SMOKE_LOG"; : > "$OUT"
START=$(date +%s); STROOT=/mnt/c/temp/vibeos-qemu
./vibeos-cli smoke --arch amd64 --allow-stale > "$SMOKE_LOG" 2>&1 &  SMOKE_PID=$!
last=""
while kill -0 "$SMOKE_PID" 2>/dev/null; do
  live=$(find "$STROOT" -mindepth 2 -name debugcon.bin -newermt "@$START" -path '*smoke-amd64*' \
         -printf '%T@ %p\n' 2>/dev/null | sort -rn | head -1 | cut -d' ' -f2-)
  [ -n "$live" ] && [ -f "$live" ] && cp "$live" "$OUT" 2>/dev/null && last="$live"
  sleep 5
done
[ -n "$last" ] && cp "$last" "$OUT" 2>/dev/null; wait "$SMOKE_PID"
```
Count markers: `python3 -c "from collections import Counter as C; print(C(open('OUT','rb').read().decode('latin1')))"`. **Marker-collision-safe bytes** (clean vs the trace catalog `tests/qemu/debugcon.py`): `D t T B R c C x z` are free; `e` is a hex digit (minor noise); AVOID `I J K L M N O P Q i j k l m o p q r` (catalog milestones). The pre-existing `ScsiPortIsr` `#I#/#i#/#J#` markers (scsiport.rs:1970-1991) emit unconditionally — leave them. **Capture caveat:** for a boot+net run the per-run file rotates/cleans mid-run, so a single poll-copy can truncate to the first ~30s — for a clean full-boot count, do a **boot-only** run (no net stage) or grab the file before the net stage starts.

**3. §9.6b — the amd64 cargo kernel does NOT relink on a `.rs` edit** (field-manual line ~1344). `ninja livecd` / `vibeos-cli build` can silently ship a STALE kernel. Force a real relink: remove the staged exes (`rm -f build-amd64/ntoskrnl/{ntoskrnl,ntkrnlmp}.exe`) AND, if incremental still looks stale, nuke the cargo target (`rm -rf sdk/lib/rust/target/x86_64-kernel`). **Self-validation trick (used here):** put a debugcon marker in the SAME crate as your fix — if the marker appears in the boot, that crate was genuinely rebuilt with your change (markers in `ntoskrnl-rust` validate any `ntoskrnl-rust` fix; e.g. `c`/`C` in ke/time.rs co-validate a ke/clock.rs change). After an edit, confirm `stat -c %y build-amd64/ntoskrnl/ntoskrnl.exe` is newer than the edit.

**4. Coordinate amd64 boot windows** — concurrent host load perturbs the stall rate (a quiet host is the clean measurement). `qemu/ps` / `./vibeos-cli ps` shows live QEMUs; same-arch boots auto-serialize but cross-contaminate timing. Don't double-boot amd64 with another agent mid-measurement.
