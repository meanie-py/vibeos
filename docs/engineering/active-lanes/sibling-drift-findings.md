# Sibling-drift scan: findings

Tool: `tools/scan_sibling_drift.py`. No upstream oracle — it never reads ReactOS. It looks for
pairs of Rust functions **in the same file** that are conventionally paired (naming or
`#[cfg(target_arch/target_pointer_width)]` twins, or two loops adjacent inside one function),
normalises away the tokens that are SUPPOSED to differ between the pair, and reports whatever
is left. The four bugs fixed on this lane today (pushlock.rs CAS-value swap, filelock.rs
missing ownership check, ntbitblt.rs unused `ptlSrc`, resource.rs wrong-object lock) are all
this shape, and none was visible to an upstream diff or a test — only to reading both sibling
bodies side by side.

## How it works

1. **Extract functions** from every `.rs` under `sdk/lib/rust/` with a regex + brace-counting
   pass (not a real parser — good enough for rustfmt'd code). Files matching `bindgen_*.rs`,
   `hal_bindgen_*.rs`, `types_*.rs` (mechanically generated, thousands of trivial fns each) are
   skipped outright, and any file whose `//!` banner starts with `ORPHAN-BY-DESIGN:` is skipped.
2. **Generate candidate pairs**, three ways:
   - *Name-paired*: `<Stem>Shared`/`<Stem>Exclusive`, `Acquire`/`Release` substring swaps,
     `Nt<X>`/`Zw<X>`, `<X>`/`<X>Ex` — all in the same file.
   - *Arch-gated twins*: same `fn <Name>` appearing more than once with a `#[cfg(target_arch =
     ...)]` / `#[cfg(target_pointer_width = ...)]` directly above each copy.
   - *Adjacent loops*: two `for`/`while`/`loop` constructs at the same nesting depth inside one
     function (nested loops are explicitly excluded — see false-positive class 4 below).
3. **Normalise** both bodies to a token stream: strip comments, collapse the pair's own two
   names and the generic Shared/Exclusive, Acquire/Release, Get/Set, Nt/Zw word-pairs to a
   single placeholder.
4. **Diff** the normalised token streams (`difflib.SequenceMatcher`) and score the residual
   opcodes: `continue`/`return`/`break`/lock-related keywords score, a `replace` of exactly one
   identifier-for-identifier token scores highest (the "different variable passed to the same
   callee" shape), and a whole `if { ... }` present on one side only ("one-sided guard") scores
   highest and is required (together with a $\ge 0.85$ similarity ratio) for an adjacent-loop
   candidate to be reported at all — see class 4.
5. Findings below `--min-score` (default 12) are dropped; the rest are sorted by score.

**A/W (ANSI/wide) and Get/Set pairs were tried and deliberately excluded** from the
name-pairing rules — see false-positive class 1.

## Candidate/finding counts

| Stage | Count |
|---|---|
| Raw candidates before tuning (first working version) | 1,272 |
| After dropping A/W, Get/Set name-pairs | 659 |
| After requiring ratio ≥0.85 + a whole one-sided guard for adjacent-loop pairs | 137 |
| After excluding nested loops from "adjacent" pairing (a real bug in the first version of the tool — see class 4) | **65** |

Final run: `./tools/scan_sibling_drift.py --top 300` (repo root) → **65 candidates**
(33 name-pair, 23 arch-gate, 9 adjacent-loop). Scan time: ~50s over the whole `sdk/lib/rust`
tree (7000+ generated-file functions skipped; ~50k hand-written functions scanned).

**Hand-verified: all 65.** Every one of the 65 was read against its sibling in the actual
file (not just the diff snippet) before being classified below.

**Confirmed real bugs: 1.**

## The real finding

### `WSPSelect` exceptfds off-by-one — out-of-bounds array access

- **File**: `sdk/lib/rust/msafd-rust/src/misc/dllmain.rs`
- **Sibling A (correct)**: line 1440, inside the `readfds` block:
  ```rust
  if j >= HandleCount {
      ERR!("Error while counting readfds %ld > %ld\n", j, HandleCount);
      ...
      return SOCKET_ERROR;
  }
  ```
- **Sibling B (correct)**: line 1474, inside the `writefds` block — same `if j >= HandleCount`.
- **Sibling C (wrong)**: line 1512, inside the `exceptfds` block:
  ```rust
  if j > HandleCount {
      ERR!("Error while counting exceptfds %ld > %ld\n", j, HandleCount);
      ...
      return SOCKET_ERROR;
  }
  ```
  followed (line 1519 on) by `(*PollInfo).Handles[j as usize].Handle = ...` and
  `.Events |= ...`.

**What diverges**: the inner loop just above each guard (`while j < HandleCount { ... }`,
`j += 1`) searches `PollInfo.Handles[0..HandleCount)` for a matching handle and `break`s on a
hit. If nothing matches, the loop runs to completion with `j == HandleCount` — "not found".
The readfds and writefds arms correctly treat `j == HandleCount` as *not found*
(`j >= HandleCount`) and bail out with `SOCKET_ERROR` before touching `Handles[j]`. The
exceptfds arm uses `j > HandleCount`, which is **false** when `j == HandleCount` — so on a
not-found exceptfds handle, execution falls through the guard and indexes
`(*PollInfo).Handles[HandleCount]`, one past the end of the array that was sized to exactly
`HandleCount` entries (`PollInfo->Handles[i]` is filled for `i` in `0..selectfds.fd_count`,
i.e. `0..HandleCount`). This is a genuine out-of-bounds heap write, reachable from a
user-mode `select()` call whose `exceptfds` set contains a handle not present in the
combined descriptor set built from `selectfds`/`readfds`/`writefds`.

**Which is correct and why**: `j >= HandleCount`, matching the readfds/writefds siblings in
the very same function and the exact same "search then bail on not-found" idiom used three
times in a row.

**Origin note (does not change the verdict)**: the commented-out original-C reference block
starting at line ~5895 in the same file shows this off-by-one is inherited verbatim from the
upstream C (`if (j > HandleCount)` at line ~5975 of the comment, vs `>=` in the two siblings
just above it) — so this predates the Rust port. This tool has no oracle and does not care
where a bug came from: the shipping vibeos Rust binary has an OOB write here regardless of
which codebase introduced it, and it is exactly the sibling-divergence shape this tool exists
to catch. Worth fixing either way.

**Proposed fix** (one-line, no other file touched):

```diff
--- a/sdk/lib/rust/msafd-rust/src/misc/dllmain.rs
+++ b/sdk/lib/rust/msafd-rust/src/misc/dllmain.rs
@@ -1509,7 +1509,7 @@ pub unsafe extern "system" fn WSPSelect(
                 j += 1;
             }
-            if j > HandleCount {
+            if j >= HandleCount {
                 ERR!("Error while counting exceptfds %ld > %ld\n", j, HandleCount);
                 if !lpErrno.is_null() {
                     *lpErrno = (WSAEFAULT as i32);
```

Not applied (read-only per the task's hard rules) — this is a proposed diff for the owning
lane to land.

## Hand-checked candidates that are NOT bugs (representative sample)

Every one of these was read in full context, not just the tool's diff snippet.

- `ObpAcquireDirectoryLockShared`/`Exclusive` (`ntkrnl-sys/src/inline/ob.rs:223,237`) — ratio
  0.96, the highest in the whole run. Perfectly symmetric; the only difference is the
  Shared/Exclusive state-signature constant and which `ExAcquirePushLock*` variant is called.
- `ExAcquireFastReference`/`ExReleaseFastReference` (`ntkrnl-sys/src/inline/ex.rs:385,435`) —
  not actually an Acquire/Release pair of the same operation; `ExAcquireFastReference` has no
  `object` parameter and a different CAS predicate by design (its real release-side
  counterpart is `ExInsertFastReference`, 2-arg). Correct as written.
- `RtlpAcquireSRWLockExclusiveWait`/`SharedWait`, `RtlAcquireSRWLockShared`/`Exclusive`,
  `RtlReleaseSRWLockShared`/`Exclusive` (`rtl-sync/src/srw.rs`) — SRW locks have genuinely
  different wake mechanisms for the shared vs exclusive path (chained wait blocks vs a single
  wake bit); the divergence is the real Windows SRW-lock algorithm, not drift. One is
  explicitly commented "Twin of ... above" while still being intentionally different logic.
- `RtlAcquireResourceExclusive`/`RtlAcquireResourceShared`
  (`rtl-strings/src/cabi/mod.rs:6468,6534`) — a faithful port of a `goto`-driven C state
  machine; the two check different `NumberActive` ranges by design and the code comments
  explain each divergent branch. Not drift.
- `FatAcquireForCcFlush`/`FatReleaseForCcFlush` (`fastfat-rust/src/resrcsup.rs:354,403`) —
  acquire must choose exclusive-vs-shared mode (extra branch), release does not need to;
  asymmetry is structural, not a mistake.
- `ExfAcquireRundownProtection_impl`/`ExfReleaseRundownProtection_impl`
  (`ex-rust/src/rundown.rs:70,126`) — release has an extra branch to handle the
  already-rundown case and wake a waiter; acquire doesn't need it. Correct.
- `KxAcquireSpinLock`/`KxReleaseSpinLock` (`ntkrnl-sys/src/inline/ke.rs:486,538`) — heavily
  commented deliberate asymmetry (an owner-stamp check added to acquire only, explicitly
  NOT mirrored on release, with the incident that would happen if it were). The best example
  in this run of "read the comment before believing the diff."
- `_ExAcquireFastMutex`/`_ExReleaseFastMutex`,
  `_ExAcquireFastMutexUnsafe`/`_ExReleaseFastMutexUnsafe` (`ntkrnl-sys/src/inline/ex.rs`) —
  correctly paired IRQL raise/lower and count increment/decrement with matching boost
  condition (`old < 0` on release). Correct.
- `ExAcquirePushLockExclusive`/`ExReleasePushLockExclusive` (`win32k-rust/src/gdiobj.rs:259,
  276`) — a different (thinner, fast-path) implementation from `ex-rust/src/pushlock.rs`'s
  already-fixed one, but internally consistent and matches its own doc comment's C macro.
- `GetFileSize`/`GetFileSizeEx` (`kernel32-rust/src/file/fileinfo.rs:292,322`) — different
  return-type conventions (`DWORD` + `INVALID_FILE_SIZE`/`GetLastError` vs `BOOL`); the extra
  branch in `GetFileSize`'s error path is required by its own calling convention, not missing
  from `GetFileSizeEx`.
- `DiskIoctlReassignBlocks`/`DiskIoctlReassignBlocksEx` (`disk-rust/src/disk.rs:11182,11389`)
  — the `Ex` ioctl's struct genuinely uses a 64-bit `LARGE_INTEGER` block number where the
  non-Ex one uses a 32-bit `ULONG`; the tool's "different type" flags are the intended type
  change, not drift.
- `DispCallFunc` ×3 arch-gated copies (`oleaut32-rust/src/typelib/dispcall.rs:86,259,418`) —
  see false-positive class 2.
- `BaseInitializeContext` x86_64/aarch64 (`kernel32-rust/src/client/utils.rs:616,679`) — see
  false-positive class 2; this one even has a comment noting it was a *previously fixed* real
  regression (an absent ARM64 branch), now present and correct on both arches.
- `IopFixupResourceListWithRequirements` "sibling" loops (`pnpmgr-rust/src/pnpres.rs`) — this
  candidate came from the pre-fix version of the tool; the two loops are an outer/inner nest,
  not siblings (see false-positive class 4, now fixed).
- `SepAnalyzeAcesFromDacl` "sibling" loops (`se-rust/src/accesschk.rs:222,327`) — two
  same-shaped `while AceIndex < ...` loops, but they run under different `ActionType` branches
  (`AccessCheckMaximum` vs `AccessCheckRegular`) — different features, not a copy-paste pair.
  Ratio 0.88 but no genuine shared invariant to diverge from.

## False-positive classes (why some rule ideas were dropped or tightened)

1. **A/W (ANSI/wide) and Get/Set name pairs are not reliable signal.** Tried first, produced
   351 candidates (of 1272 total) dominated by legitimately-different bodies: ANSI/wide
   functions convert between `char`/`WCHAR` buffers and routinely have different allocation
   sizes, different helper calls, and different error paths — none of that is drift. The
   single highest-scoring candidate in the very first run (`secur32/sspi.rs`'s
   `_makeFnTableA`/`_makeFnTableW`, score 432) turned out to be a perfect mirror once read —
   the false "divergence" was an artifact of both functions referencing *both* `A` and `W`
   symbols (thunking tables), which defeats a simple global A↔W token swap. Get/Set pairs are
   even worse: `GetSrbScsiData`/`SetSrbScsiData` in `disk-rust/src/ddk_shims.rs` take
   genuinely different parameter types (`*mut u8` vs `u8`) because one reads and the other
   writes — the "residual diff" is the entire, intentional shape of the function. Dropped
   from `name_pair_reason()` entirely.
2. **Arch-gated ABI/calling-convention trampolines are expected to differ substantially.**
   `DispCallFunc`'s three arch copies (i386/x86_64/aarch64) each hand-marshal function-call
   arguments per that arch's ABI (stack push-loop vs Win64 register slots vs AAPCS64
   register block) — this is real, necessary, and documented divergence, not drift, and it
   produced the two highest scores (162, 148) in the entire run. `BaseInitializeContext` is
   the same story (x86_64 segment-register setup vs aarch64 X0/X1/SP/PC setup). **This class
   cannot be filtered by a similarity threshold** — the divergence really is large and
   real — so arch-gate findings need a human who knows "is this function ABI-shaped" before
   reading further; the tool cannot tell fast-path ABI code from a genuine cfg-duplicated bug
   (win32k-rust's already-fixed `NtGdiGetPixel` case) without that judgment call.
3. **Faithful ports of an original `goto`/state-machine C function look like drift but
   aren't.** `RtlAcquireResourceExclusive`/`Shared` and several disk/fastfat ioctl handlers
   are deliberately-different state machines with inline comments citing the exact C label
   they correspond to (`// C: goto done`, `// C: goto wait`). A diff-based tool has no way to
   know "this is a documented port of an asymmetric original" vs "this is where the port
   drifted" — only reading the comments settles it.
4. **Adjacent-loop detection originally conflated NESTED loops with SIBLING loops.** The
   first version matched every `for`/`while`/`loop` construct linearly through a function body
   and paired consecutive matches; an outer loop immediately followed (in match order) by its
   own inner loop was reported as a "sibling pair", which is nonsensical — a nested loop's
   body is not supposed to look like its parent's. `IopFixupResourceListWithRequirements`
   (outer `while i < AlternativeLists` / inner `while ii < Count`) was the case that exposed
   this. **Fixed**: the scanner now tracks the byte-span of the last *accepted* top-level loop
   and skips any loop whose start falls inside it, so only loops at the same nesting depth are
   ever paired. This one fix alone dropped adjacent-loop candidates from 859 (or 81 after the
   ratio/guard tightening) to 9.
5. **Plain identifier drift in adjacent loops is expected, not signal.** Two loops iterating
   different collections (`readfds` vs `writefds`, `cffolder` vs `cffile`, `source` vs
   `destination`) will always have several single-token identifier swaps; scoring those the
   same as a true guard omission produced ~600 spurious candidates. Adjacent-loop findings are
   now gated on BOTH a $\ge 0.85$ token-similarity ratio AND at least one whole `if { ... }`
   guard present on exactly one side (`one-sided-guard`) — the actual shape of the
   already-fixed `FsRtlFastUnlockAll` bug.
6. **Generated/bindgen files are unscannable and worthless to scan.** `bindgen_*.rs`,
   `hal_bindgen_*.rs`, `types_*.rs` files have 4,000–7,000 functions each; naive O(n²) name
   pairing over one such file alone is tens of millions of comparisons, and any "pair" found
   there is a coincidence of the C header's naming, not authored code. Skipped by filename
   before parsing.

## Precision assessment

- 65 final candidates, all 65 hand-read, 1 confirmed real bug (a genuine OOB array write),
  ~15 documented as clearly-correct-by-design during triage (see above), the rest read and
  judged not-a-bug for reasons specific to that pair (different return conventions, different
  state machines, ABI-specific arch code, or two loops running under mutually-exclusive
  branches). **Zero candidates are marked INCONCLUSIVE** — every one was legible enough,
  once read in full file context, to call definitively.
- Precision on this run is roughly 1/65 for "actionable bug", but the categories that produce
  the noise (arch-ABI trampolines, faithful `goto`-state-machine ports, and legitimately
  asymmetric Acquire/Release pairs with a documented reason) are now named and explained above
  for the next person who reads this tool's output, which should make the NEXT run faster to
  triage even if the hit rate doesn't improve.

## Is this worth a `vibeos-cli check` gate?

**No — recommend an occasional manual sweep, not a CI gate**, for three reasons:

1. **It has no correctness oracle of its own.** Every "finding" is a heuristic score over a
   text diff; distinguishging a real bug from a deliberately-asymmetric, well-commented pair
   (the majority of this run) requires reading the surrounding code and any comments,
   which is exactly the step a gate cannot do. A gate that fails builds on well-understood,
   correct asymmetric code (the `KxAcquireSpinLock` case, or any arch-ABI function) would be
   false-positive noise that erodes trust in the gate within a day.
2. **The interesting population is small and shrinks as bugs get fixed.** This run found the
   remaining stock of this bug shape in the CURRENT tree at 65 candidates / 1 real bug. Once
   the `WSPSelect` fix lands, a re-run should return the same 65 minus that one (or fewer, if
   related code is touched) — there's no expectation of a steady stream that would justify
   an every-commit check. It's much closer to a periodic (say, per translation "wave") sweep
   run by a human who reads the report.
3. **It IS cheap enough to run occasionally.** ~50 seconds over the whole tree, read-only, and
   the tuned output (65 candidates) is small enough for one person to read end to end in
   well under an hour. That argues for keeping it as a standalone tool invoked manually
   (or from a future lane's checklist) rather than baking it into the always-on gate.
