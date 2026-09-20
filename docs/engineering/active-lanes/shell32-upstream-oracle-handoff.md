# shell32: the upstream oracle findings, handed to the shell32 lane

**From** the upstream-sync lane · **written 2026-09-20** · **owner: the shell32 lane**

The shell32 lane is days from 100% Rust. The owner's ruling on the upstream merge was:
**keep the C dead, and fix the Rust semantically** — every hunk upstream touches in
`shell32.cpp` resolves to *ours*, and the upstream change becomes a semantic-port item
against `shell32-rust` instead. See `upstream-sync-merge-recipes.md`.

This document exists so the oracle information survives that gap. It is not a task list for
upstream-sync; it is raw material for whoever does the semantic-port pass.

---

## What "the oracle" means here, and why it is different from your porting work

We forked ReactOS at `5213cf717cff6946945922e6a7e03d87b0a58e4f` (2026-02-11). Where you
translated a `.c`/`.cpp`, you translated the **pre-fix** C. So an upstream fix landing after
the fork on a file you translated is evidence the Rust carries that same bug.

Your semantic-port pass is a **different** question from the one this lane answers, and both
are worth asking:

| question | method | what it finds |
|---|---|---|
| does our Rust carry a bug upstream has since fixed? | diff upstream's post-fork commits against our Rust | inherited bugs |
| does our Rust behave like the C it replaced? | diff our Rust against the C's *behaviour* | translation gaps |

The second is yours and this lane cannot do it. A worked example of why it matters:
`NtUserSBGetParms` (win32k, not shell32) had `_SEH2_TRY` in the **pre-fix** C, and the
translation dropped it *and* never added a probe — so our Rust was **worse than the buggy C**.
Nothing in the upstream-oracle method would have found that; it only showed up because
someone diffed our Rust against `<sha>^` rather than against the fix. Expect that class in
shell32 too, and note `scan_dropped_seh_pairs` structurally cannot see it (it only detects a
kept raise with no catch, not a guard dropped wholesale).

---

## The surface

48 upstream commits touching `dll/win32/shell32` since the fork, across 96 files. Largest by
churn:

```
1676+   evalcmd/elements.cpp        (new subsystem)
 490+/287-  CQueryAssociations.cpp
 392+/51-   CFolderItems.cpp
 384+/191-  shell32.spec
 353+   CShellItemArray.cpp         (new, split from CShellItem.cpp)
 306+   evalcmd/evalcmd.cpp         (new subsystem)
 279+/71-   CShellItem.cpp
 192+/28-   utils.cpp
 181+/27-   CFolder.cpp
```

Most of that volume is **feature work**, not fixes — NT6+ implementations, `FolderItems3`,
`IQueryAssociations::GetKey`, `SHEvaluateSystemCommandTemplate`, the `evalcmd` subsystem. Those
are scope decisions for you, not oracle leads. The fixes below are the part that says something
about code you already have.

---

## CONFIRMED — verified against our Rust, not applied

### 1. `SHGetSetSettings` writes a partially-uninitialised SHELLSTATE to the registry
- **Upstream**: `4fecc908e7d` — "All SHELLSTATE members must be initialized before writing to
  the registry (#9293)". Adds `SSF_ALL`/`SSF_IMPLEMENTED` and changes the pre-read condition
  from `dwMask & ~g_CachedSSF` to `notcached & ~dwMask`, where
  `notcached = ~g_CachedSSF & SSF_IMPLEMENTED` — so on a **write** call every member not
  already cached gets read back from the registry first, because the whole struct is about to
  be written.
- **Ours**: `sdk/lib/rust/shell32-rust/src/wine/shellord.rs:1149-1210`. Line 1208 is
  `if (dwMask & !g_CachedSSF) != 0 { SHGetSetSettings(&mut tempstate, dwMask, FALSE); }` —
  byte-for-byte the pre-fix condition. Neither `SSF_ALL` nor `SSF_IMPLEMENTED` exists anywhere
  in the file.
- **Effect**: any `SHGetSetSettings(..., SSF_SOMEFLAG, TRUE)` whose flag is already in
  `g_CachedSSF` skips the backfill and writes whatever `tempstate`/`gpss` holds for *every
  other member* straight to the registry.
- Proposed diff is in `upstream-sync-flipped-tu-review-wave2.md` §W2-3.

### 2. `CDefView` teardown crash ordering
- **Upstream**: `68a03b2e1ae` — "Prevent crash by unsubclassing DefView during teardown
  (#9305)".
- **Ours**: `sdk/lib/rust/shell32-rust/src/shell32/CDefView.rs:6174-6221`, pre-fix shape.
- Detail in `upstream-sync-flipped-tu-review.md` §18.

---

## INCONCLUSIVE — real upstream fixes, our side not settled

### 3. `SHBindToObject` signature/ABI
- **Upstream**: `a790beb22ba`. Widens it to 5 params + `SHSTDAPI` (stdcall) and removes the
  `SHBindToObjectEx` wrapper.
- **Ours**: `shell32-rust/src/shell32/utils.rs:664-706` still has **both** functions.
- **Why it was not escalated**: a later pass established that all callers are internal to
  shell32 and consistently use the 4-param signature, and `SHBindToObject` is **not in
  `shell32.spec`** — not a public export — so there is no live ABI mismatch reachable from
  outside. It recurs across 6 shell32 files, so it is worth settling once rather than six
  times.

### 4. `6730ec5508f` — CD-ROM eject not refreshing drive text labels in folder view (#8789)
- Twin: `shell32-rust/src/shell32/CDefView.rs`. Flagged by the keyword sweep, never opened.

### 5. `fd683dbba45` — `AddCommasW` uses hardcoded grouping instead of `LOCALE_SGROUPING` (#8814)
- A locale-correctness fix; small and self-contained.

### 6. `02f23141395` — `LockServer` implemented (#9055)
- Noted in the merge recipes as one of the things upstream adds to `shell32.cpp` while we
  retire functions out of it. Feature or fix depending on whether anything calls it.

### 7. `7cba1c80a75` — `E_OUTOFMEMORY` not returned on allocation failure in
  `ASSOCSTR_DEFAULTICON` (#8866). Straightforward error-path fix.

### 8. `26f920582be` — click event not ignored for disabled menu items (#9387).

### 9. `9342ed43d9a` — "Better determine whether to hide or show shell-folder namespace items"
  (#9046), and `39ad1b6824f` — "Set ShellState registry entry when default is needed" (#9037).
  Both touch the same ShellState area as finding 1; worth doing together.

---

## Not shell32, but yours by ownership

`8d44ccdd5f7` — "[EXPLORER][SHELL32] Fix crash when tray icon with active balloon is removed
(#9332)". A real crash fix, but the change lives in `base/shell/explorer/syspager.cpp`, a
200+ line C++/ATL restructuring. `explorer-rust`'s registry entry has `rust_files: []`, so the
twin was never located and this was never verified. Someone has to find that twin by hand.

---

## Build-system item you will hit

`dll/win32/shell32/CMakeLists.txt` conflicts with upstream in 6 commits, and our copy has the
whole `SOURCE` list commented out (~55 entries). The recipe is in
`upstream-sync-build-conflicts.md`. One gap flagged there and **not** resolved:
`CSimpleShellItemArray` — a brand-new class upstream split out into `CShellItemArray.cpp`,
with no counterpart on our side. Zero callers today, so it is a gap rather than a break.

---

## Traps — every one of these cost this lane real time today

Read these before porting any upstream change. They are not hypothetical; each is an incident.

1. **The subject line lies, in both directions.** `e6328fbf1ea` "Rewrite
   NtGdiGetCharABCWidthsW" was a pure readability refactor that fixed nothing, and was ranked
   #1 by a keyword sweep. `06ae7ec2114` "Rewrite GetCharWidth etc." — same word — *was* a real
   fix. **Measured**: subject-only triage was wrong 25–40% of the time, in both directions,
   across two independent samples. One of those misses was a stack-corruption bug dismissed by
   a subject line that literally said "caused stack corruption". Judge only by the diff.
2. **Upstream's fix can itself be buggy.** `e1bf662d9a` routed two early exits to a label that
   returns an uninitialised variable; rustc rejected it outright.
3. **Upstream's fix can be a regression for us.** `e7e1465812` made an optional pointer's copy
   unconditional, turning a legitimately-NULL argument into a failure. We kept our NULL check.
4. **Upstream's fix can be insufficient for us.** A DHCP length guard still overflowed because
   *our* buffer is smaller than upstream's. Check our sizes and widths, never theirs.
5. **The control-flow shape may not transfer.** Upstream's `FsRtlFastUnlockAll` is a `for` loop
   with the advance in its increment clause, so its bare `continue` advances. Our Rust is a
   `while` with the advance at the bottom, where the same `continue` spins forever.
6. **Upstream's history contains its own regressions.** `d09a3600606` fixes a regression
   introduced by `b38db9dd238`, which we never applied. Whether a fix applies to us depends on
   where *we* sit relative to the bug it fixes. Trace the flag before porting.
7. **Our Rust can be worse than the pre-fix C** — see the `_SEH2_TRY` case at the top.

---

## Registry caveat

`tools/cohab-registry.json` maps flipped C to its Rust owner, but entries produced by its
`dir-name-guess` fallback carry `rust_files: []` and *look like an answer*. Three independent
agents lost time to this; every one of 5 files opened in one pass had it. Trust an entry with
non-empty `rust_files`; otherwise grep the **function name** across `sdk/lib/rust/`. The
generator owes a fix (emit *unpaired* rather than guessed).

---

## Where the detail lives

| report | shell32 content |
|---|---|
| `upstream-sync-flipped-tu-review.md` | §18 CDefView, plus the utils.cpp/shellord.c leads |
| `upstream-sync-flipped-tu-review-wave2.md` | §W2-3 SHGetSetSettings, with a proposed diff |
| `upstream-sync-flipped-tu-review-wave3.md` | the ~26 shell32 commits triaged at subject level only — **unverified, see trap 1** |
| `upstream-sync-diff-shape-wave2.md` | rows 23–29, mostly feature classifications |
| `upstream-sync-build-conflicts.md` | the CMakeLists recipe + CSimpleShellItemArray gap |
| `upstream-sync-merge-recipes.md` | the owner's "keep the C dead" ruling for `shell32.cpp` |

**UPDATE 2026-09-20, later: that block is now READ.** See
`shell32-oracle-triage-wave2.md` — all 31 previously subject-only commits opened in full, plus
6 translation/cosmetic ones explicitly logged as skipped.

**The warning above was justified: 6 of the commits characterised as "feature bundle, nothing
to regress" carry real bugs our Rust inherited.** Verdicts: 6 CONFIRMED, 1 INCONCLUSIVE, 6
REFUTED, 10 N/A-feature, 8 N/A-build/spec, 6 skipped.

The six, as *things to check* rather than patches — that lane is rewriting these files, so a
line-level diff would rot before it was read:

| upstream | what to check |
|---|---|
| `a83411cfefe` | `SHRunControlPanel` (`wine/shellord.rs:1986`) has no NULL check before `wcscat`. Reachable from `IShellDispatch::ControlPanelItem` COM automation → crash. |
| `16bb138e325` | `CDefaultContextMenu`: the literal `"open"` verb always returns `E_FAIL` (`DoOpenOrExplore` is a stub) and `MapVerbToCmdId` never searches registry verbs. |
| `763ce84cf55` | `CExeDropHandler` leaks `sPathTarget` on every drop, and `SHAlloc`'s result is used by `copy_nonoverlapping` unchecked. |
| `a5f51269c88` | `CFolderItemVerbs::Item` returns `S_OK` with an uninitialised out-param past bounds; `CEnumIDListBase::AppendItemsFromEnumerator` drops OOM and leaks the pidl silently. |
| `b60f66cbbfa` | `CPrinterFolder::GetUIObjectOf` lacks the `IQueryAssociations` branch `ShellExecute` needs. |
| `d6cc284435e` | `LinkWindow_RegisterClass` returns `TRUE` while registering nothing. |

`93f96e592b2` (CQueryAssociations) is INCONCLUSIVE — a near-total rewrite, too large to bisect
in that pass and left for whoever knows the file.
