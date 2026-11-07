# Lane: upstream-sync — ReactOS as a bug oracle

**Worktree** `/home/curi/Projects/os-playground/vibeos-upstream-sync` · **branch** `upstream-sync`
· written 2026-09-20 by the control plane · **owner: the user**

We forked ReactOS at `5213cf717cf` (2026-02-11). They have moved 1,263 commits since. This
lane makes their work reachable without treating it as a patch queue we owe.

> **STATUS 2026-09-20, late. PHASE 3 IS DONE — the merge is IN.** 1,269 upstream commits
> merged (`28a26f3b664`), 302 conflicts resolved, and the tree BUILDS on i386, amd64 and
> arm64 (`842b5458588`). 3,769 files, +464,486 / −163,249.
>
> Read these three in this order:
>
> | doc | what it is |
> |---|---|
> | `upstream-sync-merge-decisions.md` | **every decision the merge made**, what was declined and why, the three silent hazard classes, the nine build-failure shapes, and the nine things still owed |
> | `upstream-sync-phase4-review-queue.md` | **the payoff**: 194 flipped TUs whose dead C the merge refreshed, top 40 ranked with what upstream actually did |
> | this file | strategy and the phase table below |
>
> Phases 0, 1 and 3 are DONE; 2 was retired (measured, not guessed). **Phase 4 is now the
> whole lane** and it is no longer archaeology: every one of those 194 files is an ordinary
> `git diff` against Rust sitting beside it.
>
> The numbers below were measured on the morning of 2026-09-20 and the merge has since
> consumed them. They are kept because the *method* still applies to the next sync — in
> particular "partition by disk existence first", which turned a 302-file wall into 129
> policy lookups, 65 version comparisons and ~110 files needing a human.

**The framing that matters, and the one thing not to lose:** upstream is not a source of
patches for us to apply. It is an **oracle that tells us where the bugs are.** When they fix
`ntoskrnl/ex/sysinfo.c`, that is evidence our `sysinfo.rs` — translated from the *pre-fix* C —
contains that bug, faithfully reproduced. We did not merely diverge from their code; **we
froze their bugs in Rust at the fork point.** Every upstream fix touching a file we translated
is a lead.

---

## RESUME HERE (2026-10-31: the 2026-10-31 sync, 127 commits to upstream `80dec44f649`)

Merge `cd5f05edbc1` (upstream c25e4fd7dec..80dec44f649, 2026-10-15..10-31; 63 conflicts, each
resolution in the commit message) plus follow-up `7b315dd0e56`, on branch `upstream-sync-2`.
The next sync starts from `80dec44f649`. LANDED 2026-11-01 as master `f702ef72441` (batch 65, after the
batch-60 bounce fix). Of the upstream commits after it, two are already handled: 006cf40b7e3 (LSASRV
LsarLookupSids) is hand-ported into lsasrv-rust, and dca105d7490 (BOOTVID:PC98) is a pruned tree.
The git merge of them will still come through as dead C, plus a pc98 tree to keep deleted.

**Status at park (2026-11-01):** submitted to merge-integrator as `upstream-sync-2 @ 1acc6748417`
(queued as its own batch 60, merging master `558865d4c44` onto it, which `merge-tree` shows as 0 conflicts).
It has NOT been smoked locally; the integrator's cold gate is its first boot. Before
resuming, check that it landed (`git merge-base --is-ancestor 1acc6748417 origin/master`) or read
the integrator's classification and artifact dir, and fix forward.

**BOUNCED 2026-11-01, LINT_FAIL; FIXED 2026-11-01 after the migration** (ntkrnl-sys regenerated; the
ndk EXTENDED_DEVOBJ_EXTENSION gained the InterruptCount upstream gave only the public type, so
both are 60/112 bytes). The original note follows. Integrator batch 60
(merged onto master `aa597d19355`, tag `integration-batch60-red`; gate
`/tmp/vibeos-cli/20261101-000723-fkugq9ie-gate-i386+amd64+arm64`, which the migration wipes).
Everything else was green: build, smoke x3 + net + bios, conformance x3, stortest x3. The one
failing row is layout `size_check` DRIFT ("C headers vs committed asserts") on all 3 arches,
because two kernel layouts changed in the headers and ntkrnl-sys was not regenerated:
- `xdk/iotypes.h` `_DEVOBJ_EXTENSION` (upstream 167c0e97ce5) went from the 3-field public stub
  to the full layout. i386 8→60, 64-bit 16→112.
- `ndk/iotypes.h` `_EXTENDED_DRIVER_EXTENSION` (6eaa9a97e7a, KSE) gained
  KseCallbacks/DvCallbacks/VerifierContext. i386 28→40, 64-bit 56→80.

Repro: `.venv/bin/python sdk/lib/rust/ntkrnl-sys/verify_sizes.py --output-dir X`, then diff
`X/size_check_<arch>.rs` against the committed file (those 4 lines are the only diff). Fix:
1. Audit FIRST where iomgr-rust allocates and lays out the device object plus its
   DEVOBJ_EXTENSION, and the driver object plus its extension.
2. Ask whether any C still in the image now reads past what the Rust allocates.
3. Then either port the fields (DEVOBJ_EXTENSION's are real NT layout; KSE's belong with the
   KSE port item) or keep our layout and `#if` the new fields out, giving the reason.
4. Then run generate_bindings.sh, verify_sizes.py and tools/verify_offsets.py, and resubmit.

**Two silent hazards fired, and both arrived as clean merges.** (a) Spec stubs turned into
real exports with no implementation: nine RtlFls*/NtUserPfn lines in `ntdll.spec`, plus
wlanapi's WlanGetNetworkBssList. A link break waiting. Sweep every merged `.spec` for a `-stub`
or `@ stub` that lost the word. (b) A file added into a pruned tree
(`drivers/filesystems/ntfs/lockctrl.c`). Also: a renumbered `resource.h` in a Rust-flipped
module, where the `.rc` compiles against the new header and the island keeps the old numbers
(diskpart, `scan_shadowed_c_define` caught it). Regenerate the island, then `port check`.

Upstream independently landed two fixes we had already made: KiTryThreadLock's inverted fast
path (1b5f5ac0801; ours since 2026-07-13) and diskpart's sizeof(PPARTENTRY) overrun
(b44f5ceb7e4).

**Owed from this sync** (all on disk as oracle C, none built):

| item | upstream | what it needs |
|---|---|---|
| ~~Vista users folder (`C:\Users`, ProgramData, Public)~~ **DONE 2026-11-01** (`7246a5a8d03`) | f0a83320d79 | userenv-rust setup/profile/environment hand-ported together with the hive/livecd/boot_images half (and our own hivesys USERPROFILE/HOMEPATH, livecd_start.cmd). It also fixed a live master regression: the merge took upstream's userenv .rc without its string table, so InitializeProfiles failed on LoadStringW(IDS_PROFILEPATH) on every install. Upstream's table says Music for My Pictures and "Appdata"; corrected here. |
| Kernel Shim Engine (`ntoskrnl/kse/`, ~1.6K lines) — **DONE 2026-11-07** | 6eaa9a97e7a, 1a577262b50, dd91acb1f95, 804d4036c26 | ntoskrnl-rust `src/kse/`: kse/driverscope/ignorepooltag/versionlie translated with rustconv (Haiku, sealed, ~$0.06); ksedb hand-ported with its own bounds-checked SDB reader instead of apphelp `sdbread_km`. Hand-written: `KseRegisterShimEx` (a per-arch `global_asm!` trampoline passes `_ReturnAddress` to the body, which the caller-module check needs), the four DriverScope pool shims (print a NULL caller under KSEP_DEBUG_POOL: no _ReturnAddress on stable) and two call-through wrappers the model kept damaging. Translation defects fixed: every NTAPI hook came back `extern "C"` (cdecl on i386, so a hooked dispatch/completion routine would corrupt the stack); the `MmProbeAndLockPages` SEH was dropped (now `MmTryProbeAndLockPages`); hook tables transmuted fn pointers to integers in a static. Wired as upstream: KseInitialize(0/1) in IoInitSystem, KseDriverLoadImage in the builtin-driver path and MmLoadSystemImage, KseShimDriverIoCallbacks in IoCreateDriver. `RtlFindExportedRoutineByName` became the real NTAPI symbol. Verified x3: drvmain.sdb loads (v2.1), all three shims register, desktop. KSE exports stay `-version=0xA00+` stubs like upstream. |
| ~~RtlFls* (`sdk/lib/rtl/fls.c`)~~ **DONE 2026-11-05** | bf45318bc5b | Hand-ported ($0) to ntdll-rust `rtl/fls.rs` (user-mode only, so ntdll, not the shared rtl), with a real `try_finally` for RtlFlsFree's callback guard. Spec rows un-stubbed at upstream's 0x600+/0xA00+ tiers, so the 0x502 build carries but does not export them; kernel32 does not route (upstream routes only at >=0x600). kernel32-rust's `PEB.FlsCallback` now uses the same `FLS_CALLBACK_INFO` pair layout: upstream's Vista kernel32 keeps a single-pointer BaseRundownFls beside a pair-layout RtlFlsFree. Upstream's RtlFlsGetValue returns ERROR_INVALID_PARAMETER (a success NTSTATUS) with no FLS data; ours returns STATUS_INVALID_PARAMETER. |
| ~~RtlInitializeNtUserPfn / RtlRetrieveNtUserPfn (`ntuserpfn.c`, `ntuserdisp.s`)~~ **DONE 2026-11-05** | 479662b3f0c | ntdll-rust `rtl/ntuserpfn.rs`, the dispatch stubs as `global_asm!` on all three arches (upstream's arm64 is a TODO that leaves NtDllUserStubs undefined). Exported by ntdll_vista.spec, and by ntdll.spec at 0x600+. No caller yet: user32 does not register. |
| PCIX as a full PnP bus driver (~3.7K lines) | 443d9c1de54 | pcix-rust; `pci.h`'s RoutingToken vs our IrqRoute fields is the layout seam |
| USER32 Wine-11.11 controls (button/combo/edit/listbox/static/mdi/DDE, ~4K lines) — **edit DONE 2026-11-06** | 39fee16a692 | user32-rust, hand-ported per control against Wine's own suite. **edit**: counted ReplaceSel and the single-line paste cut, EM_SETRECT validation, CJK margins (`GetCharWidthInfo`/ABC), EM_SETPASSWORDCHAR on multi-line, EM_SET/GETIMESTATUS, ANSI DBCS lead bytes, capture rework (drag timer gone), zero-height guards; user32:edit 21 -> 4 failures of 263152 (the remaining four: one from the stub below, three pre-existing). Upstream's EDIT_EM_PosFromChar lost an `else` (CORE-15780 line discards the measured x): NOT followed, commented. **listbox DONE 2026-11-07**: LBS_NODATA storage (one byte per item, or none for single-select), real `LB_SETCOUNT`, Wine's binary-search `FindStringPos` and wrap-around locale `FindString`, `LB_SETSEL` anchor/caret, `SetSelection` order, multi-column wheel, accessibility events; user32:listbox 287 -> 239 failures, 0 new (A/B on the same sweep; the drop in executed checks is per-item WM_MEASUREITEM callbacks LBS_NODATA no longer fires). What remains is the two rows below, DlgDirList's test-file creation (`listbox.c:1500`, cwd), and the horizontal-extent scrollbar checks (2151-2167), all identical in the baseline. Upstream now counts SM_CYHSCROLL twice in LISTBOX_UpdateSize (its own rect add plus Wine's); ported as is, commented. **static/combo/button/icontitle/mdi/DDE/LoadString DONE 2026-11-07** (A/B on the amd64 WoW64 sweep): static 35 -> 0 (etched lines as WS_EX_STATICEDGE, one WM_CTLCOLORSTATIC per paint, ordinal titles; plus win32k NtUserDefSetText now clears on NULL, which ReactOS's C never did), combo 23 -> 21 (item_height, CBF_NORESIZE; the rest are pre-existing rect/drop-height checks), resource 2 -> 0, dde 24 -> 21 (DdeCreateDataHandle cbOff, plus a translation bug in ddeserver map_W_to_A that dropped the converted command's NUL), MDI (Window-menu refresh, WS_MAXIMIZEBOX, LR_SHARED, and a bounded MDI_GetChildByID: the C loops on nActiveChildren and walks off ci->child), icontitle split A/W. user32 has no button suite (Wine's is in comctl32). spy.c's message-name table is NOT ported: trace output only. user32:win crashed at the same point before and after (10 tests, then 0xc0000005): **fixed 2026-11-07**, the suite called `InternalGetWindowIcon` through a NULL pointer (the spec only had a version-gated stub, so GetProcAddress gave NULL); now implemented Wine's way in defwnd.rs and exported. It now runs to the end and showed 2628+ failures the crash hid (log capped at 256 KB, so an undercount). `test_AWR_flags` (2016 of them) **fixed**: AdjustWindowRectEx is now Wine's `adjust_window_rect` over SPI non-client metrics (the C dropped a minimized window's resize border), and AdjustWindowRectExForDpi uses the metrics for its DPI instead of ignoring it. Now 1039, uncapped; top remaining: main_window_procA WM_WINDOWPOSCHANGED/WM_NCCALCSIZE rect checks (1082-1144), 1484, 2563, 9427; owed. **Still owed**: edit's IME rework (ReactOS composition-window paths; nothing in the suite grades it), user32:win's crash, combo's rect/drop-height checks. The combo `scan_dropped_null_guard` baseline line is this item. Grading note: on amd64 the winlogon winetest sweep runs the SysWOW64 (i386) `user32_winetest`, so it grades build-i386's user32.dll; rebuild i386 first. |
| `NtGdiGetCharWidthInfo` (gdi32 `GetCharWidthInfo`) — **DONE 2026-11-06** | — | win32k-rust `freetype.rs`, Wine 11.11 semantics (hhea min side bearings scaled by ppem/em; 0/0 for bitmap fonts). user32:edit 4 -> 3 (`edit.c:1727` passes). |
| kernel32 collation ignores diacritics — **DONE 2026-11-07** | — | CompareString now sorts the NT way from the shipped `sortkey.nls` + `sorttbls.nls` (nothing read them before): `string/ntsort.rs`. Old Wine's table gave `í` the weights of `i` (it expected a decomposition ReactOS commented out), and CompareStringW/A dropped the LCID, so no tailoring existed. Now: diacritic level (nonspace marks fold into the previous unit), expansions (ß, Æ), per-locale compressions (cs ch, da aa, hr dž) and exceptions (sv Ä/Ö/Ø after Z), French reverse diacritics, word-sort punctuation last. Only the diacritic level reads ahead: Windows stops at the first differing unit and Wine's locale test guard-pages it. user32:listbox native i386 16 -> 8 (left: 3 todo_wine, file/DlgDirList, h-extent). kernel32:locale A/B: CompareStringA/W/Ex failures 27 -> 0; `LINGUISTIC_IGNOREDIACRITIC` accepted. **Owed**: (a) LCMapString NORM_IGNORENONSPACE never strips accents (2840/2860): it was hidden because the test checks it with lstrcmpW, which was accent-blind; (b) LCMapString(LCMAP_SORTKEY) still uses the old Wine table; (c) Hungarian double compression, CJK ideograph files, kana/Jamo specials; (d) kernel32:locale crashes after line 3131 in baseline and now alike. |
| user32:edit's three pre-existing failures — **DONE 2026-11-07, 0 failures of 263171 native i386 AND WoW64** | — | All three were the WoW64 seam or the test, not edit.c: (1) user32's menu code read win32k's 64-bit MENU/ITEM through the 32-bit layout (`wow64.rs` `normalize_menu`, Menu64/Item64 layout-guarded), so TrackPopupMenu in any 32-bit process got garbage (edit.c:2743/2764); (2) NtUserGetMenuBarInfo passed the 32-bit MENUBARINFO straight through: marshalgen refused its bitfields, now folds a same-bits storage unit into one scalar (`Record::marshal_fields`) and generates it; (3) `Wow64Manual_NtUserGetClassInfo` had `if (!W32)` translated inverted, so every WoW64 GetClassInfo failed (test_initialization's ReactOS crash guard was hiding exactly this; removed). Plus edit WM_SIZE returns 1 like Windows, and `--winetests user32` no longer double-defines user32_winetest.exe. |
| user32 `DlgDirSelectEx` clobbers last error — **DONE 2026-11-07** | — | Not user32: win32k `NtUserBuildHwndList` ran `SetLastNtError(Status)` on every exit (so does the ReactOS C), writing ERROR_SUCCESS into the caller on success. It sits under `GetDlgItem`/`WIN_ListChildren`, so every GetDlgItem cleared the last error. Found by probing GetLastError between calls in the guest. Now set on failure only. user32:listbox native i386 849 → 16 failures (10 are the collation row above, 3 are todo_wine). |
| Boot-partition polling (`PollBootPartitionTimeout`, now in the hive at 30 s) | 2b15735e98c | iomgr-rust IopMarkBootPartition. HAZARD: arm64 and MININT boot with no boot device on purpose, and a straight port stalls them 30 s; gate the loop on those first |
| diskpart volume rework (status column, sort, select by letter, CD/DVD size) | ac1ab9e3043, e386343489c, b6f74c0360a | diskpart-rust partlist/UpdateVolumeInfo; the Status column is empty until then. Upstream's PrintVolume default arm assigns nVolumeType for nVolumeStatus |
| shell32-rust shellpath to Wine-11.16 (Vista CSIDL/KNOWNFOLDER defaults), with SHGetKnownFolderItem | fe98aaf5443 | Until it lands, shell32's built-in defaults are XP names, so livecd.inf names EVERY shell folder under HKCU/HKLM `User Shell Folders` (the key shellpath reads). An install is covered because userenv's setup writes them all. The `verify_lost_exports` baseline line is this item. |
| KBDCLASS/MOUCLASS port-device IRP support (14 commits) | 7f754b4e420..2349cb9a3bc | kbdclass-rust / mouclass-rust semantic review |

## Previous sync (2026-10-17: the 2026-10-15 sync LANDED, `7509c4dc796`; follow-ups done)

The 2026-10-15 upstream merge (64 commits to upstream c25e4fd7dec) is on master with its hand
ports, island regen, Haiku translations (pnpprop, cacheinfo $0.10; storport re-ported $0.25)
and conformance re-baselines. Branch `upstream-sync-2` in worktree `vibeos-upstream-sync`
carries the follow-ups: the interface DeviceProperty exports (ntoskrnl, upstream 3cc9aa99292;
ntoskrnl_vista's copies removed, and its `IoDeviceProperty` kmtest in the sweep lists) and
shell32's RegenerateUserEnvironment (c25e4fd7dec), and (2026-10-17) mmc_new's views
(c0f01460800, 857f521a704) plus `CActionWnd.cpp` (88da81d9b8b), re-bought with the owner's
approval ($0.34 Haiku) and flipped. Nothing from the 2026-10-15 sync is owed, tooling
included: a goto-carrying macro (CHK_HR) is now expanded in a fresh TU's C by `port translate`
and never ported as a `macro_rules!` (owner chose not to re-buy mmc_new's 14 hand-banked
bodies; the first real test is the next fresh module that uses one). (`port replay` now refuses a
hand bank whose C changed since it was captured; banks from before 2026-10-17 carry no C
hash until re-captured -- mmc_new's are stamped.)

## Measured, 2026-09-20 (re-derive before trusting — these will drift)

```
fork point            5213cf717cf   2026-02-11  [BOOT][ENVIRON] Remove Environ (#8667)
upstream since fork   1,266 commits            through 2026-09-20
ours since fork       5,389 commits            we are moving ~4x their rate
```

**The conflict set has a half-life: 244 -> 281 in one day (~37/day).** Re-measure before
quoting, and prefer compressing the phases over spreading them across a week — spread out,
you pay the resolution cost twice.

Upstream touched **4,519 files**, which split three ways:

| | files | what it means for us |
|---|---|---|
| Still C/C++ here | ~3,370 | directly mergeable |
| We have translated (HIGH confidence) | **195** | merge the dead C, then review the Rust |
| Not in our tree at all | **956** | see `upstream-sync-absent-files.md` for the 4 buckets |

**Dry-run full merge** (`git merge-tree --write-tree --name-only HEAD upstream/master`,
read-only, touches nothing). 281 conflicts, and the shape is NOT what this plan first
assumed — partition by **disk existence first**:

```
281 conflicts
  126  modify/delete        <- settled by POLICY from the absent-file buckets, not by
                               reading diffs. The single largest block, and it is not
                               merge work at all.
   70  build files          <- the careful set; 8 carry comment-outs (see LANDMINE)
   38  3rdparty/vendored    <- mostly take-theirs per vendor.yaml
   21  live C               <- ours is living code; real per-hunk judgement
   19  headers/IDL/spec
    2  flipped dead C       <- theirs is the fresher oracle
    5  rostests + other
```

**Genuine hand-written work is ~110 files, 8 of them dangerous.** That is a session, not a
week. Note what this does to the reframe below: merging the dead C touches only **2** of the
281 conflicts. The reframe is still right, but its value is entirely in the edits that merge
CLEANLY into flipped files — it is not what makes the conflict set affordable. What makes it
affordable is that 126 conflicts are policy and 38 are vendored.

Two caveats on these numbers, both of which bit me while producing them:

- **The 668 is RESOLVED and was wrong: the real figure is 195.** The stem match collapsed
  `shlwapi/utils.cpp` onto any `utils.rs`. `tools/cohab-registry.json` now holds 1,091 flipped
  TUs (832 HIGH / 255 MEDIUM / 4 LOW confidence), and 195 upstream-changed files hit a
  HIGH-confidence entry. Regenerate with `tools/gen_cohab_registry.py`. Do not quote 668.
  The same error class recurs easily — a later conflict partition called 15 files "flipped"
  when 10 of them were not on disk at all. Check existence before classifying.
- `git merge-tree` prints conflicted paths, a blank line, **then informational chatter**. Parse
  up to the blank line. Reading past it reports 795 conflicts instead of 244 — I published
  795 before catching it.

---

## The reframe: merge the dead C too, everywhere

A cohab-flipped `.c` is **still on disk and commented out in CMake** — e.g. `ntos.cmake:216`
reads `#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c`, and `reply.c`, `utils.cpp`, `sysinfo.c`
are all present. Nothing compiles them.

So merging upstream's change into a flipped `.c` **cannot break the build**, and it buys the
thing that makes this whole burndown affordable: **the C becomes a live oracle again.** Review
stops being archaeology and becomes an ordinary diff against text already in our tree:

```bash
git diff <watermark> HEAD -- ntoskrnl/lpc/reply.c     # what upstream changed
# ...then read sdk/lib/rust/lpc-rust/src/reply.rs against it
```

That is why this is not "merge the C parts and hand-port the Rust parts". It is **merge
everything, then review the Rust against the refreshed C.**

**Conflict resolution is a JUDGEMENT, not a default.** The owner's instruction: for each
conflict the agent reads the upstream diff *and* our corresponding Rust, and writes the patch.
Do not blanket-resolve "ours" because we are 5,308 ahead, and do not blanket-resolve "theirs"
in flipped files because the C is dead. Both are shortcuts that discard the reason the
conflict exists. Note the asymmetry though, because it changes what a correct resolution looks
like: in a file we **compile**, ours is the living code; in a file we have **flipped**, ours is
stale dead text and theirs is the fresher oracle.

---

## THE LANDMINE — read this before merging anything

**Measured properly, the landmine is smaller than this plan first said — and it is in a
different place.** Of the 70 conflicting build files, only **8** contain a commented-out
source, i.e. only 8 can re-enable dead C by a bad resolution:

```
boot/freeldr/freeldr/CMakeLists.txt   dll/win32/kernel32/CMakeLists.txt
boot/freeldr/freeldr/rosload.cmake    dll/win32/shell32/CMakeLists.txt
dll/3rdparty/libtiff/CMakeLists.txt   dll/win32/syssetup/CMakeLists.txt
dll/cpl/sysdm/CMakeLists.txt          ntoskrnl/ntos.cmake
```

Recipes for all 8: `upstream-sync-build-conflicts.md` (one is ASK-OWNER — `rosload.cmake`).

**But the exposure is 80 files, not 8, and that is the real argument for the gate.** Upstream
touched **400** build files; **197** of ours carry a comment-out; the intersection is **80**.
A conflict is not required — a CLEAN three-way auto-merge can drop or revive a comment line
with no marker and no resolution, so nobody reviews it. Review scales to 8. Only a census
scales to 80.

A merge that resolves one of those wrongly **re-enables a commented-out `.c`**. That is a
partial cohab flip — the exact failure that froze the guest on 2026-09-19, where 58 live C++
symbols were silently captured, the taskbar hung inside `OnCreate`, `ProcessStartupItems` never
ran, and the conformance sweep launched **0 of 91** tests. **Smoke passed on all three arches
while the guest was dead**, because the freeze is after the `[WIN32K] shell ready` marker.

Arriving via a merge it would be worse: git is the author, nobody reviewed it as a flip, and
there is no diagnostic.

**DONE — `tools/cohab_census.py`, wired into `vibeos-cli check`** (`1e222cecc52`). 2,067
entries in `tools/cohab-census.txt`. It is a **differ, not a classifier**: the pattern
deliberately also matches prose naming a source file, because stable prose is stable in the
snapshot, so a false positive costs one baseline line while a false negative costs a frozen
guest. Line numbers are not part of the key — a merge shifts every line. After a DELIBERATE
flip, re-run `--write` in the same commit, so the snapshot always records a decision someone
made on purpose. `tools/test_cohab_census.py` asserts it FIRES on the 2026-09-19 shape,
because a gate nobody has watched trip is not a gate.

Related, from the shell32 lane the same night: a stale `ORPHAN-BY-DESIGN:` banner switches
**four** checks off — `scan_extern_arity`, `scan_cc_mismatch`, `scan_silent_failures` and
`verify_lost_exports` all skip a file on sight of the token. If a merge wires a module, that
banner must come off in the same commit or four scanners go quiet on live code.

---

## Provenance: a per-FILE watermark

The unit is the file, not the commit, because the question worth answering is *"is our Rust
reconciled with the current C?"* — and that is per-file.

```json
"ntoskrnl/lpc/reply.c": {
  "upstream": "<sha>",
  "rust": "sdk/lib/rust/lpc-rust/src/reply.rs",
  "reviewed": "2026-09-20"
}
```

One structure, three jobs:

- **Burndown** — count of translated files whose watermark ≠ upstream HEAD. One number, it
  goes to zero, and a scanner can gate it.
- **Review queue** — `git log <watermark>..upstream/master -- <file.c>`, exact and
  re-derivable by anyone, never a typed table. (A typed table is a claim whose age the next
  reader cannot see; `recyclebin-rust`'s Cargo.toml note said "PARKED at 25 errors" and it was
  at 204.)
- **Provenance** — the watermark, plus an `Upstream-Commit: <sha>` trailer on every backport
  commit so `git log --grep` finds them forever.

The big merge sets **every** watermark at once. That is most of why it is worth doing as one
merge rather than a trickle.

---

## Phases

| # | what | why in this order |
|---|---|---|
| **0** | Real cohab registry — **DONE** | `tools/cohab-registry.json`, 1,091 TUs, regenerate with `tools/gen_cohab_registry.py`. Killed the 668. |
| **1** | Comment-out census gate — **DONE** | `tools/cohab_census.py` in `vibeos-cli check`. Nothing else was safe without it. |
| **~~2~~** | ~~Merge `modules/rostests` alone~~ — **RETIRED** | Measured, and it is not the low-risk pipeline proof it looked like. It contributes **1** content conflict, so it proves almost nothing, while upstream's **+172,839 / −45,575 lines** across 461 files there make it a large merge in its own right. And our side is not incidental: 20 commits over 41 files, including the kmtest `-fasync-exceptions` fix and the i386 native-SEH validation — kmtest is our conformance oracle, so a `checkout upstream -- modules/rostests` would silently discard the harness fixes that make the sweep trustworthy. Doing it separately also pays the ~37/day conflict drift twice. Folded into 3. |
| **3** | Full merge — **DONE** (`28a26f3b664` + `842b5458588`) | 302 conflicts, not 281: the drift was real. Partitioned by disk existence — 129 modify/delete by policy, 65 vendored by MEASURED VERSION (which split both ways), 73 build files, 30 sources. Then twenty rounds to make it build. Full record in `upstream-sync-merge-decisions.md`. |
| **4** | Burn down the review queue: refreshed C diff vs our Rust — **LIVE, this is the lane now** | 194 files, listed and ranked in `upstream-sync-phase4-review-queue.md`. Headline: upstream IMPLEMENTED job objects (`ps/job.c` +2198/−536) — which is why `PspCloseJob` was missing from `ps-rust` entirely — and built a whole AHCI/SATA stack we have zero coverage of. The B1 set (63 files we DELETED) still cannot be reached by any merge and remains hand-review only. |
| **5** | `vibeos-cli upstream-sync` — fetch, merge, report the new queue | The owner's ruling: this stays an ONGOING AGENT LANE, not deterministic automation. Don't overinvest in tooling for it. |

**Phase 4 does not depend on phase 3, and the most valuable part of it cannot.** The 63 B1
files are C we *deleted*, not flipped. A merge refreshes dead C we don't compile; it can
never reach code we removed. Those fixes arrive only by someone reading the upstream diff
against our Rust. That is where the first confirmed bug came from
(`4d7ef310adf`), and it is the highest-value output of this lane — above the merge itself.

---

## Rules that apply with extra force here

- **`git merge` only. NEVER `reset --hard`, never force-update `master`.** A `reset --hard` to
  a stale `origin/master` once rewound mainline by ~81 commits to the 2024 fork. A
  `reference-transaction` hook blocks non-fast-forward `master` updates.
- **Never delete a translated `.rs`, never revert Rust to C, never hide a translation from the
  build** to make a merge resolve cleanly. A broken Rust translation, integrated and failing,
  is the correct state. These outrank the smoke-test imperative.
- **`./vibeos-cli land`** — the merge queue. Do not hand-roll gate-then-push. A docs-only batch
  now skips the gate entirely (`tools/docs_only.py`), so documentation lands in seconds.
- **Never broad-kill QEMUs** (`pkill -f qemu` kills your own process tree). Use
  `./vibeos-cli kill`.
- **Path discipline**: resolve `$W=/home/curi/Projects/os-playground/vibeos-upstream-sync` once
  and prefix every path. An absolute path to the main checkout writes to the main checkout.
- **First thing you will hit in this fresh worktree:** `vibeos-cli check` reports
  `verify_signatures SCANNER_ERROR — scanner did not complete`. That is correct, not broken.
  It needs generated headers from **this** worktree and refuses to borrow a sibling's, because
  a verdict that depends on which siblings happen to exist is not a gate (it once moved a
  mismatch count 987 -> 9 with no source change). Fix: `./vibeos-cli build --arch i386`. The
  `land` preflight classifies it as SCANNER_ERROR and warns rather than blocking, so it will
  not wedge a landing.
- The 810 not-in-our-tree files are `modify/delete` conflicts and **each needs a reason, not a
  default**. Keep them out of any automated pass.

## Open questions — ANSWERED by the owner, 2026-09-20

1. **The absent files (956, not 810)** — "we did delete some code, we should eyeball them and
   see if they move." Done: 606 new upstream / 63 deleted-because-translated (**the eyeball
   list**) / 247 deliberate prunes / 40 renames. Full bucketing in
   `upstream-sync-absent-files.md`. One trap found: two `libwine` files look like deletes but
   are a MOVE (`c474d932eb7` → `sdk/lib/libwine/`), so a blanket "stay deleted" would orphan
   upstream's future changes to them.
2. **How much do we want?** — "prefer to KEEP THINGS INCLUDED unless it was deleted; if the
   code was already translated to Rust then figure out what the change was and port to Rust."
   So bucket A is a near-automatic accept, and B1 routes to Rust review rather than to a merge.
3. **Unattended weekly sync?** — "unattended doesn't mean blindly merging, we will always need
   an AI agent to do the work in the sync, so it would be an ongoing lane. we shouldn't
   overinvest in deterministic tooling." Phase 5 is therefore a lane, not a cron job, and the
   deterministic surface stays deliberately thin — the census gate is the one piece that has
   to be mechanical, because it guards a failure smoke cannot see.

**Still needing the owner:** `boot/freeldr/freeldr/rosload.cmake` is marked ASK-OWNER in
`upstream-sync-build-conflicts.md` (upstream adds `ntoskrnl/ke/config.c` to `ROSLOAD_SOURCE`,
and `ntldr/advopts.c` is new code extracted from two already-flipped files).
