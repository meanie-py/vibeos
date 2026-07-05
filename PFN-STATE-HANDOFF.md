# Codex handoff — complete the ARM3 PFN state-tracking translation

**Worktree:** `/home/curi/Projects/os-playground/vibeos-flake-arm64lpc` (branch `flake-arm64-lpcexit`).
**Owner of this brief:** meaniepy (control-plane Claude). Do this work **in this worktree**.
**Goal:** finish the Rust translation of ARM3 PFN (Page Frame Number) state tracking so it is
*complete and correct* — every page-state transition is tracked exactly as the C does — and
**remove the "soft fixup" masks** that currently paper over PFN-state corruption. Do this
**regardless of whether it turns out to be the root of the boot wedge below** — the masks are
technical debt that hide real bugs and must go.

---

## Why this matters (motivation, not a constraint)

While root-causing the ~50% amd64 boot wedge (full writeup: `docs/engineering/translation-debugging-field-manual.md`
§9 "x86 boot HANG_TIMEOUT at service startup"), the deadlock bottoms out in `MiDecrementShareCount`
(`sdk/lib/rust/ntoskrnl-rust/src/ARM3/pfnlist.rs`). That function's `prototype_pte()!=0` branch
dereferences a **pageable prototype PTE** and page-faults *while the working-set lock is held*,
which self-deadlocks the fault handler. It is reached because PFN state (`page_location`,
`prototype_pte`, share/reference counts) is not being tracked faithfully — the code even admits it:

```rust
// pfnlist.rs:752
// Soft fixup: PFN state tracking has incomplete coverage in the Rust
// translations. Force page_location to ActiveAndValid rather than bugcheck.
// TODO: Find and fix the root cause of PFN state corruption.
```

**Whether or not fixing PFN state fully closes that wedge, completing it is correct on its own
merits** — the soft-fixups suppress `ASSERT`/bugcheck signals that exist precisely to catch this
class of corruption, so leaving them in blinds every future MM debugging session.

---

## Definition of done

1. **No `page_location`/PFN-state "soft fixup" masks remain** in `ARM3/pfnlist.rs` (and the ARM64
   variants). Where the C would `ASSERT`/`KeBugCheckEx`, the Rust asserts/bugchecks too — the
   state must be *correct* so the assert never fires, not *masked* so it can't.
2. **The MMPFN accessors and state-transition logic provably match the C.** Every path that
   changes a PFN's `page_location`, `PrototypePte`, `ShareCount`, or `ReferenceCount` mirrors the
   ReactOS C exactly (see the file's own embedded C comment blocks — each translated fn keeps the
   original C at the bottom).
3. **All 3 arches still build and smoke green ×3** (`./vibeos-cli gate`), and the amd64 wedge repro
   (below) no longer deadlocks inside `MiDecrementShareCount` with masks removed.
4. If you find a genuine layout bug (see "OriginalPte" below), fix it and note it.

---

## Where the work is

Primary: **`sdk/lib/rust/ntoskrnl-rust/src/ARM3/pfnlist.rs`** — the PFN list manager
(`MiDecrementShareCount`, `MiDecrementReferenceCount`, `MiInsertPageInFreeList`,
`MiInsertPageInList`, `MiRemovePageByColor`, `MiUnlinkPageFromList`, `MiInitializePfn`, …).

Accessors (hand-written, offset-based — **verify these**):
**`sdk/lib/rust/ntkrnl-sys/src/mm.rs`** — `impl MMPFN` (`page_location`, `set_page_location`,
`prototype_pte`, `share_count`, `reference_count`, `pte_address`, `original_pte`, `mmpfnentry`).

Adjacent state-setting sites (audit for the same masking pattern):
`ARM3/pagfault.rs` (lines ~743, 760, 1068, 1079), `ARM3/mminit.rs` (~610, 637, 667, 736),
`ARM3/contmem.rs` (~100), `ARM3/section.rs` (~731), `ARM3/helpers.rs` (~986, 1020 — real ASSERTs).

The soft-fixup sites to eliminate (grep `"soft fixup\|rather than bugcheck\|instead of bugcheck\|
PFN state\|state tracking"` under `sdk/lib/rust/ntoskrnl-rust/src/ARM3/`).

---

## What I already verified (don't redo — build on it)

The MMPFN bitfield accessors were checked against the authoritative bindgen `_MMPFN` (amd64) and are
**correct**, so the corruption is NOT a wrong bit/offset in these:

| accessor | hand-written (mm.rs) | bindgen `_MMPFN` (amd64) | verdict |
|---|---|---|---|
| `PteAddress` | offset 8 | `PteAddress` @ 8 | ✅ |
| `ShareCount` (u2) | offset 16 | `u2` @ 16 | ✅ |
| `ReferenceCount` (u3) | offset 24 (u16) | `u3.e3.ReferenceCount` @ 24 | ✅ |
| `MMPFNENTRY` (u3.e1) | offset 24+2 = 26 (u16) | `u3.e1` after `ReferenceCount` | ✅ |
| `PrototypePte` | bit 3 of mmpfnentry | bindgen `PrototypePte` bitpos 3 | ✅ |
| `page_location` | bits 8-10 | bindgen `PageLocation` bitpos 8, width 3 | ✅ |

**⚠️ One thing to double-check: `OriginalPte`.** `mm.rs` reads `OriginalPte` as an `MMPTE` at
offset **32** (`pfn_offsets_orig_pte::ORIGINAL_PTE = 32`), but bindgen `_MMPFN` shows
`UsedPageTableEntries: ULONG` at offset 32 and `OriginalPte` living inside a union (`u4`/an anon
member). Confirm the real `OriginalPte` offset in the C `_MMPFN` (it's a `MMPTE` in a union with
`AweReferenceCount`/`u4`) and fix `ORIGINAL_PTE` if 32 is wrong — a bad `OriginalPte` read feeds
`mmpte_software_protection()` in the exact `MiDecrementShareCount` branch that faults.

Because the *field layout* is (mostly) right, the corruption is in **state-transition coverage** —
some path fails to set/clear `page_location`/`PrototypePte`/counts the way the C does, leaving a PFN
in a state (`prototype_pte` set, or non-`ActiveAndValid`) the delete path doesn't expect. Diff each
translated `pfnlist.rs` fn against its embedded C and find the missing/wrong transition(s).

---

## Reproduce + verify the wedge (host GDB — fast)

The wedge is amd64, ~50%, on the default `windows-host` TCG backend. GDB over that backend
(`vibeos-cli gdb`, merged 2026-07-04) is the fast path:

```bash
source .venv/bin/activate
./vibeos-cli gdb start amd64
RUN=<run-id it prints>
./vibeos-cli gdb wait-for $RUN --pattern 'SUMS\(EventLog\): before CreateProcessW|_tWinMain entered' --timeout 400
#   'before CreateProcessW' + serial goes silent  => WEDGED (inspect it)
#   '_tWinMain entered'                            => green, retry
./vibeos-cli gdb inspect $RUN raw "source $PWD/qemu/gdb-helpers/walk-procs.py"   # offset-free process list
./vibeos-cli gdb run-until $RUN --break KeSignalGateBoostPriority --timeout 40   # NEVER fires when wedged (deadlock)
./vibeos-cli gdb stop $RUN
```

The stuck thread's real stack (KTHREAD offsets + unwind recipe in field-manual §9):
`ObpDeleteObject → LpcpDeletePort → MiUnmapViewOfSection → [WS lock EXCLUSIVE] →
MiDeleteVirtualAddresses → MiDeletePte → MiDecrementShareCount → (#PF) → MmAccessFault →
MiLockProcessWorkingSetShared → KeWaitForGate`. With PFN state correct, `MiDecrementShareCount`
should not take the `prototype_pte` branch on that page / not fault.

Final gate (all 3 arches, no masks):
```bash
./vibeos-cli gate            # builds serial, smokes all arches in parallel, ×3
```

---

## Guardrails (project rules — do not violate)

- **Never revert a Rust translation to C** to make it build/pass, and **never delete a translated
  `.rs` file.** Fix in place. (`AGENTS.md` → "Translation Philosophy".)
- **Smoke is the north star** — never weaken/lower a smoke marker; fix the OS, not the test.
  A broken translation left visible in smoke is the correct state until fixed.
- **Don't hide regressions from the build** (no commenting out `pub mod` / excluding files).
- **Worktree discipline:** work only in this worktree's paths; don't touch other lanes' dirty files.
  Use `git stash push -m "<scope>"` (never `checkout`/`reset --hard`) to set aside changes.
- **Don't commit/push without being asked** beyond the work itself; **never force-update `master`.**
- Builds go through `./vibeos-cli build --arch <a>` (canonical path + ccache/sccache). Judge build
  success by artifact existence, not reported rc.

## Suggested approach

1. Read `MiDecrementShareCount` + its embedded C (pfnlist.rs) side by side; list every
   `page_location`/`PrototypePte`/`ShareCount`/`ReferenceCount` transition the C makes.
2. Repeat for the sibling list-manager fns (`MiInsertPageInFreeList`, `MiInsertPageInList`,
   `MiUnlinkPageFromList`, `MiInitializePfn`, `MiInitializeCopyOnWritePfn`, the free/standby/modified
   list movers) — a missing transition upstream is what leaves a PFN mis-stated downstream.
3. Fix `OriginalPte` offset if wrong (see ⚠️ above).
4. Remove each soft-fixup mask; where C asserts, assert. Rebuild amd64, run the wedge repro — if a
   real assert now fires, that assert *is* the bug's fingerprint; trace back to the transition that
   left the PFN in that state and fix *that*.
5. `./vibeos-cli gate` ×3 green on all arches. Report which transition(s) were missing/wrong.
