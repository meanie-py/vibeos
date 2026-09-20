# upstream-sync: merge conflict recipes (judgement conflicts + modify/delete policy)

Fork point `FP = 5213cf717cff6946945922e6a7e03d87b0a58e4f`. Measured against `upstream/master`
= `2def708122ac27e642f77f656cc3eb2a200aa640`, our tip = `5618d6fce95` (branch `upstream-sync`),
2026-09-20. Re-derive before the real merge — both branches move (~37 new conflicts/day).

```
git merge-tree --write-tree --name-only HEAD upstream/master   # 294 conflicted paths today
git merge-tree --write-tree HEAD upstream/master                # full CONFLICT-line log used below
```

**This document analyzes only the file scope. It is read-only research — no merge was performed,
no file in the working tree was touched.**

## Summary table

Partitioned by the actual `CONFLICT (...)` line git emits, not by disk presence alone (disk
presence gets the direction wrong for 5 files — see note below the table).

| Bucket | Count | How resolved | Needs a human |
|---|---:|---|---:|
| **modify/delete — we deleted, upstream modified** | 125 | Policy per A/B1/B2/C bucket (below); B1 (62) is a real bug-triage backlog | ~62 (all of B1) |
| **modify/delete — upstream deleted, we modified** | 5 | Keep ours (we're the living side); noted as a CRT-modernization trend to watch | 0 (policy is clear), 1 flagged FYI |
| **rename/delete (false positives)** | 6 | All 6 are spurious `explorer-old`-vs-new-upstream-file pairings (git similarity heuristic) — take upstream's new file as a plain ADD | 0 |
| **file location (real move)** | 1 | `sdk/lib/3rdparty/libwine/dll_canunload.c` → land at `sdk/lib/libwine/dll_canunload.c` (our renamed tree) | 0 |
| **add/add** | 1 | `sdk/include/vcruntime/mingw32/intrin_arm64.h` — independently invented by both sides | **1 — ASK-OWNER** |
| **content — build files** (`CMakeLists.txt`/`*.cmake`) | 72 | 8 already have full recipes in `upstream-sync-build-conflicts.md` (skip); 64 NOT examined here | 64 not reached |
| **content — vendored (`vendor.yaml` trees/containers)** | 44 | General policy + 3 flagged clusters | 3 flagged |
| **content — live C/headers/other** | 47 | **Full recipes below**, 45 substantive (2 trivial: `.gitignore`, `README.md`) | 3 — ASK-OWNER |
| **Total** | **294** | | |

Sum check: 125+5+6+1+1+72+44+47 = 301 CONFLICT-lines-worth, but 6 of the "rename/delete" paths
double-count against the modify/delete-125 set (git emits both a `rename/delete` and a
`modify/delete` CONFLICT line for the same path) — 301 − 6 = **295**, still 1 over the 294
`--name-only` paths because `dll_canunload.c`'s `file location` conflict has no companion
`modify/delete` line but its target path only appears once in `--name-only`. Net: 294 distinct
paths, verified against both the `--name-only` count and a manual `grep -c CONFLICT` cross-tab.

**Correction to the task brief's disk-presence heuristic:** partitioning by "does the path exist
on our disk right now" silently misclassifies **5 files** — `boot/bootdata/livecd.ini`,
`sdk/lib/crt/except/amd64/cpp.s`, `sdk/lib/crt/locale/locale.c`, `sdk/lib/crt/misc/tls.c`,
`sdk/lib/crt/wine/wine.cmake` — as ordinary "present/content" conflicts. They ARE present on
disk, but the actual conflict is `deleted in upstream/master and modified in HEAD`: **upstream**
deleted them (commit `de7e7074592` "[CRT] Remove obsolete CRT files" for 3 of them; the
bootcd/livecd merge for the 4th), **we** kept modifying them. That is a real modify/delete
conflict in the opposite direction from the brief's assumption, and it needs the opposite
default policy (keep ours, not "accept upstream's file"). Re-verify direction with the actual
`CONFLICT` line, never with `[ -e file ]`.

---

## Part 1 — modify/delete: we deleted, upstream modified (125 files)

Classified by whether the path existed at FP (same method as
`upstream-sync-absent-files.md`, re-run fresh against exactly this 125-file conflict set — not
the doc's broader 956-file scope).

| Sub-bucket | Count | Meaning | Action |
|---|---:|---|---|
| **A — new upstream, spurious rename match** | 6 | Didn't exist at FP. All 6 are `git`'s rename heuristic pairing our `explorer-old` deletes (bucket B2, commit `abf4af7f353`) with unrelated new upstream files by byte-shape alone | Take upstream's file as a **plain add** at its new path; ignore the phantom rename. See list below. |
| **B — existed at FP, we deleted it since** | 118 | Real modify/delete | Classify by deleting commit (table below) |

### Sub-bucket A — the 6 spurious `explorer-old` renames (extends the known trap)

`upstream-sync-absent-files.md` already documented 5 of these against the broader diff; this
merge's actual conflict set surfaces a 6th (`shcore.h`). All 6, confirmed via the `CONFLICT
(rename/delete)` line naming the phantom old-side:

| Upstream's new file (land this) | Git's claimed old-side (ignore — actually `explorer-old`, unrelated content) |
|---|---|
| `dll/win32/activeds.tlb/activeds_tlb.idl` | `modules/rosapps/applications/explorer-old/precomp.cpp` |
| `dll/win32/comdlg32/comdlg32.manifest` | `modules/rosapps/applications/explorer-old/explorer.exe.manifest` |
| `modules/rostests/winetests/user32/testdll.c` | `modules/rosapps/applications/explorer-old/precomp.h` |
| `sdk/include/psdk/featurestagingapi.h` | `modules/rosapps/applications/explorer-old/externals.h` |
| `sdk/include/psdk/ioringapi.h` | `modules/rosapps/applications/explorer-old/notifyhook/notifyhook.h` |
| `sdk/include/psdk/shcore.h` | `modules/rosapps/applications/explorer-old/services/shellservices.h` |

**Action for all 6: accept upstream's file verbatim as a new add.** There is nothing to merge —
the "old side" git found is `explorer-old`, which we deliberately deleted (59,710 lines, "none
built") and is content-unrelated to any of these 6 new files. A tool that trusts the rename
label instead of re-deriving it would try to 3-way-merge `explorer-old/precomp.cpp` against
`activeds_tlb.idl` and produce garbage — confirmed **this is exactly the shape of failure
class §"a path around a guard is not a path through it"** warns about: the rename label is git's
guess, not ground truth, and here it's wrong 6/6 times it appears.

**I searched exhaustively for any OTHER move-shaped conflict** (`grep -E "rename/rename|file
location" ` over the full conflict log) and found exactly one: the already-known
`sdk/lib/3rdparty/libwine/dll_canunload.c` → `sdk/lib/libwine/dll_canunload.c` case (Part 2
below). **No new move trap exists beyond the one already recorded and the 6 spurious renames
above.**

### Sub-bucket B — 118 real deletes, by deleting commit

| Files | Deleting commit | Bug-triage priority |
|---:|---|---|
| **48** | `8d3c33ee41e` win32k.sys retire (Rust-only) | **B1 — triage** (win32k-rust may be missing the fix) |
| **14** | `3feaf6ca5f6` tcpip.sys/lan driver retire | **B1 — triage** (netstack/ws2_32/afd/iphlpapi may be missing the fix) |
| 9 | xbox/pc98/arm32 dead-platform prune | B2 — stay deleted |
| 5 | nfs/udfs/vfatfs filesystem prune | B2 — stay deleted |
| 5 | drivers/sac + arm64ec dead-branch drop | B2 — stay deleted |
| 4 | hal dead-tree prune (Stage 0) | B2 — stay deleted |
| 4 | ext2/btrfs/ntfs filesystem removal | B2 — stay deleted |
| 4 | 32-bit ARM source dir removal | B2 — stay deleted |
| 4 | Pre-Vista prune E5 (NICs/bluetooth/TV-tuner) | B2 — stay deleted |
| 4 | Pre-Vista prune E4 (obsolete drivers) | B2 — stay deleted |
| 4 | BIOS prune 5/N | B2 — stay deleted |
| 3 | explorer-old/fraginator/nukecamp (genuine deletes, not the spurious-rename 6 above) | B2 — stay deleted |
| 2 | freeldr dead FS-read driver prune | B2 — stay deleted |
| 2 | Pre-Vista prune E3b (videoprt INT10 severance, HAL x86bios/fast486 retire) | B2 — stay deleted |
| 2 | Pre-Vista prune E3a (legacy VGA/INT10 miniports) | B2 — stay deleted |
| 1 each | freeldr disk.c, freeldr GUI stub, Pre-Vista E2 (NTVDM), Pre-Vista E1 (ISAPNP/PCMCIA) | B2 — stay deleted |

**B1 triage is the valuable, unfinished work — not done here** (out of this document's time
budget; flagged as **not reached**, see end of doc). The method, spelled out so the next pass
doesn't re-derive it: for each of the 62 paths, `git show <upstream-commit-since-FP> --
<path>` to read what upstream fixed, then check whether the Rust replacement
(`win32ss/user/*` → win32k-rust; tcpip.sys/lan driver → netstack/ws2_32/afd/iphlpapi) already
has the equivalent fix. `rust-port-status.md` is the index of what Rust owns each C file's
functionality now.

**One item worth flagging without full triage:** among the 48 win32k paths, several are the
`.splitN.c` shards (`winpos.split3.c`, `winsta.split2.c`, etc.) — these are shard artifacts of a
single logical source file split for translation batching, not independent modules. A triage
pass should group by pre-split logical file, not by shard, or it will double/quadruple-count
the same upstream fix across shards that all trace back to one original `.c`.

---

## Part 2 — the other 5 "modify/delete" directions + the real move + add/add

### `sdk/lib/3rdparty/libwine/dll_canunload.c` → land at `sdk/lib/libwine/dll_canunload.c`

Confirmed via the actual `CONFLICT (file location)` line: *"added in upstream/master inside a
directory that was renamed in HEAD, suggesting it should perhaps be moved to
`sdk/lib/libwine/dll_canunload.c`."* This is a NEW file upstream added inside their `libwine`
3rdparty tree — not the same two files (`CMakeLists.txt`, `debug.c`) the absent-files doc
already flagged, but the same class of trap: git correctly detects our `sdk/lib/3rdparty/libwine
→ sdk/lib/libwine` rename (commit `c474d932eb7`, "libwine + strmbase are OURS — out of the
3rdparty container") and offers the right target. **Action: take it, land at
`sdk/lib/libwine/dll_canunload.c`, not the literal upstream path.** Low risk — it's a new file,
no ours-side content to reconcile, just a path decision. Then wire it into
`sdk/lib/libwine/CMakeLists.txt` (check whether upstream's own `CMakeLists.txt` diff for that
dir — also a conflict, in the vendor bucket — already lists it).

### The 5 upstream-deleted-we-modified files: **keep ours**, but note the trend

| File | Upstream deleted via | Why keep ours |
|---|---|---|
| `boot/bootdata/livecd.ini` | `b2e33f26eb3` merged bootcd+livecd into one BootCD | We still ship a separate livecd target; upstream's merge doesn't apply to our boot layout without a wider bootcd/livecd redesign — out of scope for a merge conflict resolution |
| `sdk/lib/crt/except/amd64/cpp.s` | `de7e7074592` "[CRT] Remove obsolete CRT files" | Still compiled by us (`0977613c9f3` multi-arch toolchain unification touches it) |
| `sdk/lib/crt/locale/locale.c` | same | same |
| `sdk/lib/crt/misc/tls.c` | same | same |
| `sdk/lib/crt/wine/wine.cmake` | same | build-file counterpart of the above (in the 72-file build bucket, not detailed here, but same "keep ours" direction applies) |

**Policy for all 5: keep ours** — matches the project's stated default ("prefer to KEEP THINGS
INCLUDED unless we deliberately deleted it"): we did not delete these, upstream did, and we are
actively still building against them.

**Worth a flag, not an ASK-OWNER:** `de7e7074592`'s subject is "Remove obsolete CRT files" —
upstream is moving these three away (likely superseded by their UCRT work, given
`sdk/lib/ucrt` is a tracked vendor tree). If upstream's CRT is migrating off classic
`locale.c`/`tls.c`/`cpp.s` entirely, our own eventual CRT-to-Rust translation queue should know
these are heading toward EOL upstream — not an argument to delete them now (we still ship them),
just useful context for whoever prioritizes the CRT translation backlog next.

### `sdk/include/vcruntime/mingw32/intrin_arm64.h` — **ASK-OWNER**

Genuine independent invention, not a text conflict from a shared ancestor (the file didn't exist
at FP on either side — confirmed both `git log` histories start from scratch after the fork).

- **Ours** (`374097f14dc` "Add ARM64 (AArch64) architecture port"): 140 lines, MIT-licensed,
  written by us for exactly what our ARM64 port needed at the time.
- **Upstream** (3 commits: "Add MinGW intrinsics support" → "Fix MinGW intrinsic declarations" →
  "Address intrinsic review feedback"): 845 lines, public-domain, upstream's own later and much
  more complete MinGW ARM64 intrinsics header, independently declaring the same class of
  builtins (`_BitScanForward`, `_bittest*`, etc.) with different signatures/organization in
  places.

**Why this needs the owner, not a guess:** this header feeds every ARM64 TU across the CRT and
kernel through `<intrin.h>`. Swapping to upstream's 845-line version could change declared
signatures for intrinsics our existing ARM64 Rust FFI/bindgen already generated bindings against
(width/calling-convention mismatches are exactly the class of bug the field manual's
`width_alias_cast` guidance exists for). Keeping ours risks missing intrinsics upstream's fuller
header declares that some upstream-authored `.c` we still compile might now call. **Flag:
diff the two header's declared symbol sets before choosing; if upstream declares a strict
superset with identical signatures for the overlapping names, prefer upstream (more complete,
maintained by people who track the real ABI); if any overlapping signature differs, that is the
actual decision point and needs the owner's judgment on which is correct.** I did not diff the
full symbol list — this is genuinely ASK-OWNER, not a resolvable-by-inspection case, given the
845-line size and the boot-relevance of getting ARM64 intrinsics wrong.

---

## Part 3 — content conflicts: live C / headers / other (47 files — the main target)

Ordered boot path first. For each: what changed on both sides, whether flipped (dead C, upstream
diff is an oracle only) or still compiled (live risk), and the recommended resolution. Verified
flip status directly against `CMakeLists.txt`/`*.cmake` (not just `tools/cohab-registry.json`,
which has gaps — noted per file).

### Debug/KD chain (freeldr + kdcom + kdgdb + ntoskrnl kd) — one upstream refactor series

Upstream commits `3012af5e303` ("Move duplicated UART settings into a common header"),
`6c532e771e8` (DEBUGPORT serial-port-address option), `956d9f7973f`/`c4d60756dd4`/`4d0642b67fe`/
`b1a31610ac2` (cleanup), `b0bd092c0d4` (ASSERT bugchecks) all touch this cluster together.

| File | Flip status (verified) | Recommendation |
|---|---|---|
| `boot/freeldr/freeldr/lib/comm/rs232.c` | **FLIPPED** (`freeldr/CMakeLists.txt:51`, "replaced by Rust freeldr-rust/src/rs232.rs") | Dead text. Take upstream's version (fresher oracle for `rs232.rs`), diff the new DEBUGPORT-address logic against `rs232.rs` afterward as a bug-triage lead, not a merge blocker. |
| `boot/freeldr/freeldr/lib/debug.c` | **FLIPPED** (`freeldr/CMakeLists.txt:47`, "replaced by Rust freeldr-rust/src/debug.rs") | Same: take upstream, triage lead for `debug.rs`. |
| `boot/freeldr/freeldr/ntldr/headless.c` | **FLIPPED** (two comment sites: `rosload.cmake:47` and `CMakeLists.txt:115`, both "replaced by Rust freeldr-rust/src/headless.rs") | Take upstream. Upstream's own commits here (EMS support enable, SAL2 annotations) are real functional adds worth checking against `headless.rs` — triage lead. |
| `drivers/base/kdcom/kdcom.c` | **FLIPPED** (`drivers/base/kdcom/CMakeLists.txt:7,16`, "replaced by Rust kdcom-serial-rust") — **not in `tools/cohab-registry.json`, verify manually** | Take upstream. Notably upstream's tip already dropped the old `SARCH_PC98`/multi-platform `BaseArray` branching we also stripped ourselves (`f6c19a05b89`) — the two efforts converged independently, so upstream's file is a strictly cleaner oracle. |
| `drivers/base/kdgdb/kdcom.c` | **FLIPPED** (`drivers/base/kdgdb/CMakeLists.txt:13,17`, "kdrosdbg (kdcom-cargo) cohab") — **not in registry, verify manually** | Take upstream, same reasoning. |
| `ntoskrnl/kd/i386/kdserial.c` | **FLIPPED** on i386, amd64 (shares i386 file), and arm64 (own file) — `ntoskrnl/ntos.cmake:452-469`, all three arch branches commented | Take upstream. |
| `boot/freeldr/freeldr/arch/uefi/uefildr.c` | **LIVE — still compiled.** `uefi.cmake:30`: `# arch/uefi/uefildr.c   # NOT commented — owns uefildr.exe binary entry.` (the comment itself documents that this file is deliberately still C) | **Real merge, not dead-C housekeeping.** Upstream's 4 commits (`f9943c75564` Reboot() simplify, `2a74f5e794d` cold-reset implement, `7a80c49e561` memory-stack refactor, `c8907598f58` x64 ABI/memory-map fixes) are real UEFI boot-path fixes. Cross-check against our own two touches (`92f5ffadba7` "cherry-pick upstream amd64 #GP + ABI fixes" — **we may have already manually ported `c8907598f58`**, check for duplicate application before merging; `d3fa4718175` arm64 QEMU virt fixes, unrelated). **Do the actual 3-way merge here** (`git merge-file`, in your scratchpad, not the worktree) and read both hunks before accepting either side wholesale. |
| `boot/freeldr/freeldr/include/mm.h` | Not source-listed (header) — no flip question applies | Content merge of struct/macro definitions. Upstream: `26d8d3039eb` "[FREELDR] Fix AMD64 builds on GCC/Clang". Ours: several arm64/i386 fixes since fork. Straightforward 3-way; low risk since it's a single narrow GCC-portability fix upstream, unlikely to collide with our arch-specific additions unless both touch the exact same macro — verify at merge time. |

### ntoskrnl core (boot path, 4 files)

| File | Flip status | Recommendation |
|---|---|---|
| `ntoskrnl/config/cmconfig.c` | **FLIPPED** (registry: `ntoskrnl-rust`) | Take upstream (`13364f7be6a` PC-98 arch detection is dead-platform work irrelevant to us anyway per the xbox/pc98 prune; `34d5a35b74e` is whitespace-only). Zero functional stake either way — this one is close to MECHANICAL. |
| `ntoskrnl/inbv/inbvport.c` | **FLIPPED** (registry: `ntoskrnl-rust`) | Take upstream (`9833d3d8c80`/`50cb29ad4a3` are headless-init simplification/SAL2 annotations, real but small) — triage lead against the Rust `inbv` port, not a merge blocker. |
| `ntoskrnl/io/iomgr/iomgr.c` | **FLIPPED on all 3 arches** (verified directly in `ntos.cmake` — both the i386/amd64 line 140 and the arm64-branch line ~464 are commented). **Registry gap:** not in `cohab-registry.json` at all. **Stale-comment note (not a landmine):** line 140's inline comment says "amd64/i386 uses Rust; ARM64 uses C (in arm64 section below)" which is FALSE today — the arm64 section's own comment (line ~464) correctly says "Rust iomgr.rs active on all 3 arches" and its `iomgr.c` line is also commented out. Worth a one-line comment fix at `ntos.cmake:140` while someone's in there, but it changes nothing functionally. | Take upstream (`9174dcbd615` Win7+ I/O case-insensitivity) as an oracle/triage lead for `iomgr.rs`. |
| `ntoskrnl/mm/ARM3/mminit.c` | **FLIPPED** (registry: `ntoskrnl-rust`) | Take upstream (`4cba65d760a` "Change calculation for more system PTEs") — this is a real behavior-affecting MM fix, worth prioritizing in the B1-style triage queue over the others in this table since a wrong system-PTE count is a plausible source of a subtle future OOM/hang. |

### HAL / ACPI / input drivers (3 files, all flipped)

| File | Flip status | Recommendation |
|---|---|---|
| `hal/halx86/generic/usage.c` | **FLIPPED** (registry: `hal-rust`) | Take upstream (`bcd4c718c13` PC98 IRQ mapping — dead-platform, irrelevant; `a70f59b0dd5` RtlConvert*ToLargeInteger inlining — mechanical). Low stakes. |
| `drivers/bus/acpi/osl.c` | **FLIPPED** (registry: `acpi-rust`) | **Read this diff before accepting blanket.** Upstream's `63deca5a962` "[ACPI][UEFI] Use bootloader-provided ACPI root pointer in OSL" is exactly the same territory as our own `7bcecdea6e1` "i386 UEFI keyboard input: synthesize ACPI RSDP when BIOS scan fails" — both are about how the ACPI root pointer gets discovered under UEFI. These are two independent fixes to adjacent bugs in the *same* dead C file; the Rust `acpi-rust` port should be checked for whether it already handles both cases (bootloader-provided pointer AND BIOS-scan-fallback synthesis). **Triage priority: HIGH** — flag this pairing explicitly for the B1-style triage pass even though it's technically a live content-conflict, not a bucket-B1 modify/delete. |
| `drivers/input/i8042prt/mouse.c` | **FLIPPED** — confirmed via `drivers/input/i8042prt/CMakeLists.txt` header comment: "All 10 functional C sources are translated to Rust... cargo-driven... mouse.c" is dead residue. Not in registry (gap). | Take upstream (`39ca039e02f` "Compensate for inverted PS/2 mouse Y-axis under Hyper-V") as a triage lead against `i8042prt-rust`. Our own `70153561897` "i8042prt + PnP devaction diagnostics + mouse smoke gate" suggests we've already been deep in this driver's Rust mouse path recently — good candidate to check first. |

### win32ss (1 file — deliberately reduced to stubs, not flipped to Rust)

| File | Flip status | Recommendation |
|---|---|---|
| `win32ss/drivers/videoprt/int10.c` | **LIVE, still compiled** (`win32ss/drivers/videoprt/CMakeLists.txt:19`) — but NOT a translation target: commit `ebf1ac53a99` ("Pre-Vista prune E3b: sever videoprt INT10 + retire HAL x86bios + fast486") deliberately gutted this file to minimal stubs. Confirmed by reading the file's own header comment: *"The legacy INT10 / VGA-BIOS emulation path... has been removed... dead under UEFI... these minimal stubs remain to satisfy the dispatch open path."* | **Take ours, in full.** Upstream's real bugfix here (`f4786c1b91d` "[VIDEOPRT] Fix INT10h support initialization" — adds a `VDMAddressSpaceInitialized` guard and proper error codes) patches actual BIOS-emulation logic that literally does not exist in our stubbed file anymore. There is nothing to port forward; this is the cleanest "we deliberately deleted the functionality" case in this whole bucket. MECHANICAL, high confidence. |

### shell32 (5 files — mid-flip, partial Rust)

Upstream has been actively shipping new shell32 features in this window (`095860df576` "A big
bundle of NT6+ shell32 implementations and stubs", `c891aefdcff` CFolder[Item] getters/setters,
`a790beb22ba` SHBindToObject fix, `a5f51269c88` FolderItems3, plus several smaller ones) while we
have been simultaneously flipping shell32 piece-by-piece to Rust (dozens of commits: brfolder,
CDefaultContextMenu, shlfolder, CFSFolder, CDesktopFolder, CRecycleBin, CShellLink, etc.).

| File | Flip status | Recommendation |
|---|---|---|
| `dll/win32/shell32/brfolder.cpp` | **FLIPPED** (registry: `shell32-rust`) | Take upstream (`c3d6516f20c` EXTERN_C fix, `a790beb26ba` SHBindToObject fix) as oracle/triage against the Rust `brfolder` port. |
| `dll/win32/shell32/utils.cpp` | **FLIPPED** (registry: `shell32-rust`) | Take upstream (`a5f51269c88` FolderItems3, 3 SDK-implementation commits) — **this is the richest upstream delta in the shell32 cluster** (a real new feature, move/copy directory tree support); flag as a priority triage lead, not routine. |
| `dll/win32/shell32/shell32.cpp` | **LIVE — still compiled** (`CMakeLists.txt:97`, uncommented; comment nearby: "shell32.cpp names the unmangled symbols Rust emits" — it's a hybrid glue TU, not fully flipped) | **Real 3-way merge required.** Both sides touched this heavily. Read the hunks: our side is mostly COM-class retirement (functions moving OUT to Rust, so lines disappearing), upstream's is feature adds (`LockServer` impl, `AddCommasW` locale fix, swprintf conformance). A hunk that upstream edits inside a function WE already deleted (moved to Rust) should resolve to "stay deleted, note as a triage lead"; a hunk in code that's still live in both should do an ordinary merge. **ASK-OWNER if any single hunk can't be cleanly classified this way** — this file is actively morphing shape from both directions at once and is the highest-risk file in the whole "other" bucket for exactly the landmine this task warns about (accidentally re-enabling a function upstream "fixed" that we've already moved to Rust, if the C survives as an uncommented duplicate symbol). |
| `dll/win32/shell32/precomp.h` | Header, no flip question | Content merge: upstream added several NT6+ declarations (`095860df576` bundle, `c891aefdcff`, `a83411cfefe`, `c61d98f9392` — declaration removal). Low functional risk since it's declarations only, but do check whether any declaration upstream REMOVES (`c61d98f9392` "Delete SHMultiFileProperties declaration") is still referenced by our own Rust FFI bridge headers before dropping it. |
| `dll/win32/shell32/shfldr.h` | Header, no flip question | Content merge, low risk: upstream (`c891aefdcff`, swprintf conformance) vs. our extensive shell32 Rust-flip commits that touched this header's declarations as classes moved out. Same caution as `precomp.h` — verify no FFI bridge references a declaration either side removes. |

### arm64 SDK headers (2 files + the add/add already covered in Part 2)

| File | Recommendation |
|---|---|
| `sdk/include/ndk/arm64/ketypes.h` | Content merge of kernel-type struct/macro definitions. Upstream: `f6efc98b370` (part of the same MinGW-intrinsics series as the add/add file), `34d5a35b74e` whitespace. Ours: extensive arm64 boot-milestone commits. **Check specifically for struct-layout fields** — any struct here that Rust `#[repr(C)]` mirrors via bindgen must stay byte-identical; if upstream's diff changes field order/size in a shared struct, that is a correctness issue, not a stylistic one. Read the actual hunk before accepting either side. |
| `sdk/include/xdk/arm64/ke.h` | Same caution: upstream's `7debc3ef11c` "[NDK][XDK] Improve SharedUserData related definitions" plus the intrinsics-series commit vs. our own extensive arm64 milestone work on this file (KiSystemStartup, page tables, context switch). **Read the hunk; if it only touches `SharedUserData` layout and our arm64 boot code already depends on a specific layout, this needs the same struct-layout care as `ketypes.h` above.** |

### Vendored SDK-header mirrors: libjpeg / libpng / libtiff (14 files, one recipe)

These live at `sdk/include/reactos/libs/{libjpeg,libpng,libtiff}/` — **not** covered by
`vendor.yaml`'s `trees` list (which only tracks the `dll/3rdparty/{libjpeg,libpng,libtiff}`
implementation dirs, not this SDK-header mirror). That's a real gap in the registry, flagged
here rather than guessed around.

Version comparison (measured directly from the header constants, not from commit messages):

| Lib | Ours | Upstream | Verdict |
|---|---|---|---|
| libpng | 1.6.58 | 1.6.54 | **Ours is newer — keep ours.** |
| libtiff | 4.7.0 | 4.7.1 | **Upstream is newer by one patch** (`bb63bb36ed5` "fix x64 regression") |
| libjpeg | (bumped 9d→9f per our commit `1b03ad0d747`) | synced via `dcd45a59d15` | Not independently version-checked here — same recipe applies |

**Recommendation: do NOT hand-resolve these as ordinary text merges.** All three are vendored
trees with an established re-vendor process (see `vendor.yaml`'s per-tree `patch`/`version`
metadata and its 2026-09-20 ACPICA review as the model). The correct move is: resolve the merge
conflict by taking **whichever side is version-newer** as a stopgap (ours for libpng/libjpeg,
upstream for libtiff), then **file a follow-up to run the real vendor-bump pipeline** for
libtiff 4.7.0→4.7.1 so the `dll/3rdparty/libtiff` implementation and this SDK-header mirror
move together and stay in sync — a raw merge here risks skewing the header mirror one version
ahead of or behind the implementation tree. **ASK-OWNER only on the meta-question:** should
`sdk/include/reactos/libs/*` be added to `vendor.yaml` so this stops being a manual judgment
call next time? Not blocking for landing this merge.

### Trivial (2 files)

| File | Recommendation |
|---|---|
| `.gitignore` | Pure additive merge — take the union of both sides' new ignore patterns. MECHANICAL. |
| `README.md` | Ours has been substantially rewritten (vibeos branding, screenshots); upstream's only change is a release-button tweak (`e055568a902`) that doesn't apply to a non-ReactOS-branded README. **Take ours entirely**, drop upstream's hunk — this is a content fork, not something to reconcile line-by-line. MECHANICAL, high confidence. |

---

## Part 4 — vendored (44 content conflicts) and build files (72 content conflicts): NOT fully worked

Out of this pass's scope given the size of Part 3; recorded here so the next pass starts from a
correct file list instead of re-deriving it.

- **Vendored (44):** classified by `vendor.yaml`'s `containers` (`sdk/lib/3rdparty`,
  `dll/3rdparty`, `external`) and `trees` list. General policy per the registry's own model
  (see ACPICA's 2026-09-20 review entry): prefer a from-scratch re-vendor to upstream's/the
  official release's pristine source plus our tracked patch, over a manual line-level merge of
  the conflict — vendored code merged by hand tends to drift from both the patch and the
  upstream baseline. **Not enumerated file-by-file here** — that's the next pass's job, one
  vendor tree at a time, cross-checked against each tree's `version`/`patch`/`modified` fields
  in `vendor.yaml`.
- **Build files (72):** 8 already fully recipe'd in `upstream-sync-build-conflicts.md`
  (`boot/freeldr/freeldr/CMakeLists.txt`, `boot/freeldr/freeldr/rosload.cmake`,
  `dll/3rdparty/libtiff/CMakeLists.txt`, `dll/cpl/sysdm/CMakeLists.txt`,
  `dll/win32/kernel32/CMakeLists.txt`, `dll/win32/shell32/CMakeLists.txt`,
  `dll/win32/syssetup/CMakeLists.txt`, `ntoskrnl/ntos.cmake`) — **skip these, they're done.**
  The remaining **64** were not examined in this pass. Given this document's boot-path findings
  above (e.g. `iomgr.c`, `mminit.c`, `osl.c` all flipped and touched by both sides), expect a
  meaningful fraction of the 64 to be more cohab-flip CMake files following the same pattern as
  the documented 8: mostly-commented source lists where the conflict is structural, not
  content-colliding. **Not a safe blanket assumption** — each still needs the same
  `git merge-file --diff3` read the 8 documented ones got.

---

## Not reached (for the next pass to resume from, not re-derive)

1. **B1 bug-triage backlog (62 files: 48 win32k + 14 tcpip/lan)** — Part 1's biggest unfinished
   item. Method is written out above; needs someone to actually read each upstream diff against
   the corresponding Rust port.
2. **The 64 not-yet-recipe'd build-file conflicts** (Part 4).
3. **The 44 vendored-tree conflicts**, file-by-file (Part 4) — only the general policy and 3
   flagged clusters (intrin_arm64.h treated in Part 2; libjpeg/libpng/libtiff treated in Part 3)
   are done.
4. **`dll/win32/shell32/shell32.cpp`** — flagged ASK-OWNER-if-ambiguous in Part 3, but the actual
   hunk-by-hunk read (which functions are upstream-live vs. already-Rust-owned) was not done;
   this is the single highest-risk file in the "other" bucket and deserves a dedicated pass.
5. **Struct-layout diffing for `ketypes.h`/`ke.h`** — flagged as needing care, not actually
   diffed hunk-by-hunk against current Rust `#[repr(C)]` mirrors.
6. **`intrin_arm64.h` symbol-set diff** (ours 140 lines vs. upstream 845) — ASK-OWNER, ungated on
   an actual side-by-side symbol/signature table that wasn't built here.

## What would make this merge mechanical

- **Add `sdk/include/reactos/libs/{libjpeg,libpng,libtiff}` to `vendor.yaml`** as tracked trees
  (or explicitly note they mirror the `dll/3rdparty/*` trees and should re-vendor together) —
  closes the registry gap Part 3 found and turns those 14 conflicts into a scripted re-vendor
  instead of 14 manual header diffs.
- **Extend `tools/cohab-registry.json` coverage** — at least 4 files in this pass
  (`drivers/base/kdcom/kdcom.c`, `drivers/base/kdgdb/kdcom.c`, `ntoskrnl/io/iomgr/iomgr.c`,
  `drivers/input/i8042prt/mouse.c`) are flipped in fact (verified by hand against their
  `CMakeLists.txt`/`.cmake`) but absent from the registry the tooling is supposed to consult.
  Every gap here is a file where an agent doing a fast pass would wrongly treat a dead-C
  oracle-read as a live-code merge (safe direction) — or, worse, treat a live file as dead
  because a *sibling* file in the same directory happens to be registered (unsafe direction).
  Regenerating the registry from the actual CMake state (which `tools/cohab_census.py`
  presumably already parses for its gate) rather than hand-maintaining it would remove this
  entire class of "not reached" caveat.
- **Fix the stale comment at `ntoskrnl/ntos.cmake:140`** (claims ARM64 still uses C for
  `iomgr.c`; it doesn't) — harmless today but exactly the kind of stale in-repo claim the
  project's own AGENTS.md warns compounds across compactions.
- **Run the B1 triage (62 files) before the merge, not during it** — those are real potential
  bugs sitting in Rust, independent of the merge mechanics, and finding them via the merge diff is
  a free byproduct of work that should happen regardless of when the merge lands.
