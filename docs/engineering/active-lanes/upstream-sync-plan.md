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

## RESUME HERE (2026-11-22, Sunday wind-down; branch `upstream-sync-2`)

**Where it stopped.** kernel32:locale on i386 is **66 failures** (281 on 2026-11-21), run
`/tmp/vibeos-cli/20261122-164010-sLlDPO-smoke-i386` (tmp; the next migration wipes it, so
re-run a baseline first). Every commit is landed or queued: verify with
`git merge-base --is-ancestor <sha> origin/master` for `aaf45ec3114` + `817941ff18a`
(sortdefault steps 1-2, integrator batch 204 as `9eb9da8bcfc`) and the step-3 commit
after them. If one bounced, read the integrator's verdict and fix forward.

**How to grade.** `VIBEOS_PAYLOAD=k32winetest ./vibeos-cli build --arch i386 --winetests kernel32`
then `./vibeos-cli smoke --arch i386 --payload k32winetest`; result in
`<run>/payload/i386-uefi_serial.log`. A/B by `locale.c:<line>` counts (`uniq -c` + `join`).
The payload's build.sh regenerates its suite list after a master merge.

**Owed, next first.** All 66 but 2 are test_unicode_sorting's sort keys (locale.c:3892, 64).
Design and progress: the next section. Steps 1-2 done (92 -> 69); step 3 so far: weightless
units inside a mark run (72-74 fixed, 69 -> 66). The MULTIPLEWEIGHTS 3-byte primary is
implemented per the table but changed nothing measurable; the ~25 CJK entries (en-US and
ja-JP: 35-40, 203-225, 241-243) still fail, so ideographs need something else. Check what
SM/AW/DW the table gives each pair before guessing. Then: kana/halfwidth (~16, 122-159),
JAMOSORT (Hangul, 1143/1152/1196 and d1xx), SORT_DIGITSASNUMBERS (259/260), the N'Ko and
symbol singles. Also left: test_GetNLSVersion could report real sort GUIDs (fr is
00000003-..., the test's guid_fr) now that sortdefault.nls has them; NormalizeString /
IsNormalizedString / RtlNormalizeString unwired (unorm-rust has what they need); the
remaining 2 non-sort lines are 2469 (key length) and 7064 (todo_wine size).

**The source table.** `tools/codegen sortdefault` fetches it to `~/.cache/vibeos-nls/`
(outside the repo; sha256-pinned). The committed media/nls/sortdefault.nls is pinned too
(`OUTPUT_SHA256`), so `tools/codegen sortdefault --check` verifies offline. Regenerating to
a different file needs the pin updated in the same commit.

**Other owed (unchanged):** test_csparentdc's last 10; edit's IME rework; the amd64 magenta
icon block, the arm64 MM concurrent-commit fault and the i386 cdrom 0x1E one-off
(roadmap registry).

## Earlier RESUME HERE (2026-11-08, fleet wind-down; superseded, kept for its history)


**Where it stopped.** Working down the owed list in the USER32/kernel32 rows of the table below.
Everything is committed; the last commit (GetCurrencyFormatW) was sent to merge-integrator at
park. Before resuming, check each landed: `git merge-base --is-ancestor <sha> origin/master` for
`4c3f548046e` (win32k: msg SetParent bugcheck), `0bab223ab44` (locale runs to the end),
`443c329c1b9` (calendar enum / SetLocaleInfoW), `8f8872fe71c` (NLS casing), `99de3a9d6c8`
(GetNumberFormatW + cdrom roadmap entry) and the GetCurrencyFormatW commit after it. For any
that bounced, read merge-integrator's verdict and its artifact dir, and fix forward.

**How to grade kernel32:locale.** It is not on the winlogon sweep (user32 only). Use the
payload: `VIBEOS_PAYLOAD=k32winetest ./vibeos-cli build --arch i386 --winetests kernel32`, then
`./vibeos-cli smoke --arch i386 --payload k32winetest`. The result is in
`<run>/payload/i386-uefi_serial.log` (`locale: N tests executed (... F failures)`). A/B by
`locale.c:<line>` counts against the last run, `/tmp/vibeos-cli/20261108-154504-yoXPba-smoke-i386`
(281 failures), which the next migration wipes, so re-run a baseline first.

**Owed, next step first.**
- kernel32:locale **66** (2026-11-22, `/tmp/vibeos-cli/20261122-164010-sLlDPO-smoke-i386`):
  every small cluster is done; what is left is two projects, largest first:
  1. **Sort keys, 90 + 1**: locale.c:3892 checks LCMAP_SORTKEY only. (CORRECTION: the
     CompareString half of test_unicode_sorting is compiled only at DLL_EXPORT_VERSION >=
     0x600, so it never runs in this 0x502 build; 612ed202d2d's message and an earlier
     line here said it passed, which nobody measured.) Sort keys from ntsort's XP-era
     weights took **145 -> 90 (2026-11-22, `ntsort::nt_sort_key`)**, with the short-buffer
     fill (2790-2801) and 3913. The 90 left are Windows-10 weights XP's sortkey.nls does
     not have, triaged from `/tmp/vibeos-cli/20261122-120532-ngDttQ-smoke-i386`: CJK
     ideograph order (~25, tests 35-40, 203-225, 241-243), kana/halfwidth (~20,
     122-159), scripts and symbols newer than the table (Tai Tham, N'Ko 07f4/07f5,
     Ethiopic, Yi a0xx, dingbats 27xx), and Windows 10's handling of combining marks
     (26/27, 72/74, 230-239, 253-255). That is the Windows-10-era table plus Wine's newer
     compare; design first. Building the CompareString half at 0x600 would show how many
     of the same pairs ntsort's compare gets right. **Blocked on the owner**: the tree has only XP's
     sortkey.nls/sorttbls.nls (media/nls); Wine builds its sortdefault.nls from Microsoft's
     published "Windows Sorting Weight Table" files, so this needs that data downloaded
     and committed, which is the owner's call (asked 2026-11-22, same question as item 2's
     Unicode data).
  2. **Unicode normalization**: DONE as a library (2026-11-22). UCD 15.0.0 vendored
     (vendor/unicode-ucd, owner-approved), `tools/codegen unorm` -> `unorm-rust`
     (NFC/NFD/NFKC/NFKD over UTF-16, no alloc), cross-checked against Python's
     unicodedata 15.0.0 on 1,136,856 strings x 4 forms with 0 mismatches.
     FoldString(MAP_COMPOSITE/PRECOMPOSED/FOLDCZONE) uses it (4065/4066 fixed), and IDN
     nameprep does too: **DONE (5767, 5864/5869)**, all normalization failures cleared.
     Still unwired (no test of ours drives them): NormalizeString / IsNormalizedString
     (kernel32 -stub 0x600+, normaliz-rust stubs) and RtlNormalizeString; unorm-rust has
     what they need.
  SetThreadUILanguage is **DONE** (2026-11-22, 167 -> 165): a TLS slot in kernel32-rust
  (client/thread.rs), GetThreadUILanguage moved beside it and exported by kernel32 at every
  version, kernel32_vista.spec forwarding there. The NLSVersion,
  LocaleNameToLCID, ResolveLocaleName and GetStringTypeW groups are DONE. After a master
  merge the build regenerates the payload's suite list itself
  (tools/payloads/k32winetest/build.sh, keeping the list's --only restriction).
- `LCMAP_SORTKEY` still uses the old Wine table; Hungarian double compression, CJK, kana/Jamo.
- test_csparentdc's last 10; edit's IME rework; the amd64 magenta icon block, the arm64 MM
  concurrent-commit fault and the i386 cdrom 0x1E one-off (roadmap registry).

## Design: Windows-10 sort weights for ntsort (2026-11-22, before any code)

**Why.** kernel32:locale's last big group (90 at locale.c:3892) is sort keys for characters
and rules XP's sortkey.nls/sorttbls.nls do not have: CJK ideograph order, kana/halfwidth,
scripts newer than XP, Windows 10's combining-mark weights. ntsort.rs already implements
the NT weight model (SM/AW/DW/CW, nonspace marks adding DW, expansions, per-locale
compressions and exceptions, reverse diacritics), so the job is Windows-10 DATA plus the
few rules the data needs, not a new algorithm.

**Source and licence** (owner-approved 2026-11-22 via control-plane, Wine's route):
Microsoft's "Windows 10 Sorting Weight Table.txt",
`https://download.microsoft.com/download/C/F/7/CF713A5E-9FBC-4FD6-9246-275F65C0E498/Windows%2010%20Sorting%20Weight%20Table.txt`,
21,156,033 bytes, Last-Modified 2025-02-14, sha256
`81fcfa1e5ed3e3a94d329959ff7d97d522ddf9d653d2c4d6ddcccc5cd4df663f`. No open licence, so the
raw file is **never committed**: fetched at generation time into `~/.cache/vibeos-nls/`
(outside every worktree), checksum-pinned in the generator, and only our generated output
is committed, with the URL, checksum and "generated; raw not redistributed" in its header
and in vendor.yaml.

**What the file holds** (UTF-8 with BOM, CRLF): `SORTKEY` DEFAULT, 60,536 BMP code points
as `0xXXXX SM AW DW CW`, then `SORTTABLES`: NLSVERSION; 131 SORTGUIDs, which 453
LOCALENAMEs map onto; shared EXPANSION and COMPATIBILITY; per-GUID COMPRESSION (TWO..EIGHT
code points), EXCEPTION, DOUBLECOMPRESSION (Hungarian), REVERSEDIACRITICS,
MULTIPLEWEIGHTS, INVERSECASING, JAMOSORT. The header documents the key layout (UW 01 DW 01
CW 01 01 00) and the rules: SM 1 nonspace marks add DW; East Asian "3-byte weights" when
192 <= SM <= 239 (DW folds into the primary); CW bits (0x01 full width, 0x10 upper).

**Generator: `tools/codegen sortdefault`** (tools/vibeos-codegen/src/sortdefault.rs). Fetch
if not cached; refuse on a failed fetch or checksum mismatch, with no silent fallback to the
XP tables. Parse every section; refuse unknown section names, so a format change fails loudly.
Emit `media/nls/sortdefault.nls` in OUR documented binary layout (version word, BMP key
array, GUID table with locale-name index, per-GUID slices, shared expansion table): a
runtime file like sortkey.nls, loaded by mapping, so kernel32 does not grow by megabytes.
**Not in `--all`**: `tools/codegen sortdefault --check` verifies the committed file
against a fresh generation when the cache is present (exit 0/1). Without the cache and no
network it prints `UNVERIFIED: <url> not cached and not fetchable` and exits 3, never 0,
so offline check-tooling is not made red by a network dependency. Fixtures are a few
hand-written lines in each section shape.

**Runtime (ntsort.rs).** Prefer sortdefault.nls when present, else today's XP tables
(the existing fallback chain, logged once). Locale to sort GUID by LOCALENAME
(LCIDToLocaleName, already linked from kernel32-vista-rust). New rules, each behind the
data that needs it: 3-byte weights (SM 192..239); COMPRESSION up to 8 code points (today
2-3); DOUBLECOMPRESSION (owed item (c), Hungarian); MULTIPLEWEIGHTS; INVERSECASING; JAMOSORT
(Korean Jamo). Supplementary code points: the DEFAULT table is BMP-only, so a surrogate
pair keeps today's handling unless the file says otherwise; check before deciding.

**Progress.** (1) DONE `aaf45ec3114`: sortdefault.nls, 3,803,508 bytes (owner OK'd the
size), deterministic, tamper -> exit 3, MAX_OUTPUT_BYTES 4.5 MB. Expansion keys are
synthesized as SM 2 + 16-bit index (the source lists expansion code points only under
EXPANSION; 728 of 730 are absent from DEFAULT), compatibility keys copy their target's.
(2) DONE: ntsort prefers sortdefault.nls (XP tables as logged fallback); locale -> GUID by
name with parent fallback, exceptions/compressions binary-searched, compressions up to 8.
**92 -> 69** (3892: 90 -> 67), no other line moved. Win10 keeps XP's SM assignments except
digits (12 -> 13); SM_FIRST_LETTER-based accent stripping was unaffected in the A/B.
(3, in progress at the wind-down) Weightless units inside a mark run no longer end it:
**69 -> 66** (tests 72-74). MULTIPLEWEIGHTS 3-byte primaries implemented, no measured
change (CJK entries still fail; see RESUME HERE).

**Order and measure.** (1) generator + fixtures + cached-run determinism (two runs
byte-identical); (2) ntsort reads the new layout with the XP rules only: locale.c:3892 A/B
must not rise; (3) the new rules one at a time, each A/B'd on 3892 plus the full locale
count; (4) build the CompareString half of test_unicode_sorting at 0x600 as a second
measure (it never ran here). Stop and report if any step raises a line that was passing.

## Earlier RESUME HERE (2026-10-31: the 2026-10-31 sync, 127 commits to upstream `80dec44f649`; superseded, kept for its history)

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
| USER32 Wine-11.11 controls (button/combo/edit/listbox/static/mdi/DDE, ~4K lines) — **edit DONE 2026-11-06** | 39fee16a692 | user32-rust, hand-ported per control against Wine's own suite. **edit**: counted ReplaceSel and the single-line paste cut, EM_SETRECT validation, CJK margins (`GetCharWidthInfo`/ABC), EM_SETPASSWORDCHAR on multi-line, EM_SET/GETIMESTATUS, ANSI DBCS lead bytes, capture rework (drag timer gone), zero-height guards; user32:edit 21 -> 4 failures of 263152 (the remaining four: one from the stub below, three pre-existing). Upstream's EDIT_EM_PosFromChar lost an `else` (CORE-15780 line discards the measured x): NOT followed, commented. **listbox DONE 2026-11-07**: LBS_NODATA storage (one byte per item, or none for single-select), real `LB_SETCOUNT`, Wine's binary-search `FindStringPos` and wrap-around locale `FindString`, `LB_SETSEL` anchor/caret, `SetSelection` order, multi-column wheel, accessibility events; user32:listbox 287 -> 239 failures, 0 new (A/B on the same sweep; the drop in executed checks is per-item WM_MEASUREITEM callbacks LBS_NODATA no longer fires). What remains is the two rows below, DlgDirList's test-file creation (`listbox.c:1500`, cwd), and the horizontal-extent scrollbar checks (2151-2167), all identical in the baseline. Upstream now counts SM_CYHSCROLL twice in LISTBOX_UpdateSize (its own rect add plus Wine's); ported as is, commented. **static/combo/button/icontitle/mdi/DDE/LoadString DONE 2026-11-07** (A/B on the amd64 WoW64 sweep): static 35 -> 0 (etched lines as WS_EX_STATICEDGE, one WM_CTLCOLORSTATIC per paint, ordinal titles; plus win32k NtUserDefSetText now clears on NULL, which ReactOS's C never did), combo 23 -> 21 (item_height, CBF_NORESIZE) -> **0 (2026-11-08)**: CB_GETDROPPEDCONTROLRECT spans field + list, CB_GETITEMHEIGHT(-1) returns the set height, first focus sends EM_SETSEL(0, INT_MAX), and the dropped list shows whole items (all checks Wine marks todo_wine), resource 2 -> 0, dde 24 -> 21 (DdeCreateDataHandle cbOff, plus a translation bug in ddeserver map_W_to_A that dropped the converted command's NUL), MDI (Window-menu refresh, WS_MAXIMIZEBOX, LR_SHARED, and a bounded MDI_GetChildByID: the C loops on nActiveChildren and walks off ci->child), icontitle split A/W. user32 has no button suite (Wine's is in comctl32). spy.c's message-name table is NOT ported: trace output only. user32:win crashed at the same point before and after (10 tests, then 0xc0000005): **fixed 2026-11-07**, the suite called `InternalGetWindowIcon` through a NULL pointer (the spec only had a version-gated stub, so GetProcAddress gave NULL); now implemented Wine's way in defwnd.rs and exported. It now runs to the end and showed 2628+ failures the crash hid (log capped at 256 KB, so an undercount). `test_AWR_flags` (2016 of them) **fixed**: AdjustWindowRectEx is now Wine's `adjust_window_rect` over SPI non-client metrics (the C dropped a minimized window's resize border), and AdjustWindowRectExForDpi uses the metrics for its DPI instead of ignoring it. Now 1039, uncapped. **1039 -> 495 -> 230 (2026-11-07)**: DefWindowProc(WM_NCCALCSIZE) no longer ORs in `WVR_VALIDRECTS` (Windows returns only the class redraw bits; ~350 checks), GetClientRect no longer fakes SM_CX/CYMINIMIZED for an iconic window (Windows: empty), ArrangeIconicWindows is Wine's (MINIMIZEDMETRICS corner/direction/gaps, move only, returns the count), CreateWindow corrects top-level styles and hides WS_EX_LAYERED only after the WH_CBT hook, and a message-only window ignores HWND_(NO)TOPMOST; WM_WINDOWPOSCHANGED now carries parent-client coordinates, not screen (81); co_WinPosMinMaximize returns (x, y, cx, cy) from every branch (minimize and restore-to-maximized returned a rect, so minimizing to -32000 asked for a -31834 x -31976 size; 46). a minimized window keeps its (empty) client rect across SWP_NOSIZE moves instead of reseeding it from the window rect (48); the default maximized frame in co_WinPosGetMinMaxInfo is Wine's adjust_window_rect, the same formula user32's AdjustWindowRectEx uses (MDI child 2px short, 38); NtUserLockWindowUpdate is Wine's single-slot lock instead of a FALSE stub; CreateWindowEx(WS_EX_MDICHILD) with a non-MDIClient parent fails as on Windows. Now 230. **230 -> 197 (2026-11-07)**: WM_SETICON generates ICON_SMALL2 (a SM_CXSMICON copy of the big icon in the SysICS prop, flagged WNDS2_SMALLICONFROMWMQUERYDRAG so WM_GETICON(ICON_SMALL) hides it), and CopyImage now resizes in-memory icons (it ignored the size, so class hIconSm generation kept the source size; test_icons fully passes, 14); the WM_NCCALCSIZE client rect is kept as returned, larger or inverted, with ReactOS's FixClientRect clamp gone (3175/3187 plus 1089, 7); BeginPaint of a CS_PARENTDC child whose update region covers its client keeps the parent clip (12). **197 -> 178 (2026-11-08)**: the DC brush updates tested literals the translation invented in place of the C's DIRTY_* bits (DC_vPrepareDCsForBlit re-realized the text brush on DIRTY_BACKGROUND, which overwrote the color SetPixel lends it, so SetPixel painted black; fillshap and DIRTY_DEFAULT had the same problem, 12); LockWindowUpdate now clips the locked window's DCs and GetPixel returns CLR_INVALID outside the clip (7). **Owed**: a vibeos-lint rule for that class (a `ulDirty_`/flag-word mask written as raw hex where the C names DIRTY_*/DC_* constants; verify_constants cannot see literals); test_csparentdc's last 10, of which 6 are the Win8+ 166-wide main-window variant (we report NT 10, so the test picks those expectations) and 4 are a partial update, where Windows keeps the parent's region unclipped to the child (Wine todo too). **Still owed**: edit's IME rework (ReactOS composition-window paths; nothing in the suite grades it); user32:msg crashes with 0xC0150010 (STATUS_SXS_INVALID_DEACTIVATION) after 370 failures and overruns the 256 KB log cap around msg.c:5752, so everything after test_messages (button/static/listbox/combobox sequences, paint, interthread) is ungraded (seen identically with and without the combo change, 2026-11-08). The combo `scan_dropped_null_guard` baseline line is this item. Grading note: on amd64 the winlogon winetest sweep runs the SysWOW64 (i386) `user32_winetest`, so it grades build-i386's user32.dll; rebuild i386 first. |
| `NtGdiGetCharWidthInfo` (gdi32 `GetCharWidthInfo`) — **DONE 2026-11-06** | — | win32k-rust `freetype.rs`, Wine 11.11 semantics (hhea min side bearings scaled by ppem/em; 0/0 for bitmap fonts). user32:edit 4 -> 3 (`edit.c:1727` passes). |
| kernel32 collation ignores diacritics — **DONE 2026-11-07** | — | CompareString now sorts the NT way from the shipped `sortkey.nls` + `sorttbls.nls` (nothing read them before): `string/ntsort.rs`. Old Wine's table gave `í` the weights of `i` (it expected a decomposition ReactOS commented out), and CompareStringW/A dropped the LCID, so no tailoring existed. Now: diacritic level (nonspace marks fold into the previous unit), expansions (ß, Æ), per-locale compressions (cs ch, da aa, hr dž) and exceptions (sv Ä/Ö/Ø after Z), French reverse diacritics, word-sort punctuation last. Only the diacritic level reads ahead: Windows stops at the first differing unit and Wine's locale test guard-pages it. user32:listbox native i386 16 -> 8 (left: 3 todo_wine, file/DlgDirList, h-extent). kernel32:locale A/B: CompareStringA/W/Ex failures 27 -> 0; `LINGUISTIC_IGNOREDIACRITIC` accepted. (a) LCMapString NORM_IGNORENONSPACE now strips accents **DONE 2026-11-07** (`ntsort::nonspace`: the base letter is the one with the same SM/AW/CW and diacritic weight 2; kernel32:locale 132 -> 128, exactly 2840/2860, nothing new). **Owed**: (b) LCMapString(LCMAP_SORTKEY) still uses the old Wine table; (c) Hungarian double compression, CJK ideograph files, kana/Jamo specials; (d) kernel32:locale crashed after line 3131 **DONE 2026-11-08**: it now runs to the end for the first time (2,301,592 checks, 1309 failures, i386 k32winetest payload). Three stops, in order: RtlLcidToLocaleName ignored AllocateDestinationString and wrote through the caller's 0xCC Buffer (inherited from the C); ntdll exported no Rtl{System,User,Thread}PreferredUILanguages at 0x502 and the test calls them unchecked (now in ntdll_vista, forwarded as RtlLcidToLocaleName is); kernel32's NLS_EnumCalendarInfo was an abandoned translation with an exitless loop, so EnumCalendarInfo hung (rewritten from the C; new lint `scan_abandoned_translation` for the model's give-up narration). **1309 -> 432 (2026-11-08)**, nothing new: EnumCalendarInfo hands callbacks the string even for CAL_RETURN_NUMBER (Wine strips it) and walks LOCALE_IOPTIONALCALENDAR as the single number it is (the C read past it, ERROR_BADDB), skipping "0" and the primary; SetLocaleInfoW accepts exactly Windows' settable types and fails the rest with ERROR_INVALID_FLAGS, and can set sShortTime. **432 -> 342**: LCMapString cases through l_intl.nls (ntdll Rtl{Up,Down}caseUnicodeChar) instead of the CRT, with Windows' U+0130/U+0131 and Turkic (tr, az) rules under LCMAP_LINGUISTIC_CASING; test_SpecialCasing 90 -> 0. **342 -> 291**: GetNumberFormatW is Wine's get_number_format/format_number (multi-level and zero-repeat groupings, leading zeros dropped, defaults read fresh instead of from the NLS_GetFormats cache SetLocaleInfo never invalidated). **291 -> 281**: GetCurrencyFormatW is Wine's get_currency_format on the same format_number. **281 -> 213 (2026-11-21)**, nothing new: GetNLSVersion/Ex and IsNLSDefinedString were stubs that said TRUE and wrote nothing (now Wine's; sort GUID is GUID_NULL as on Win7, since our collation is not sortdefault.nls); neutral names resolve as Windows does when neutral names are not allowed (es -> 0x0c0a, ga -> 0x083c, zh-Hans -> 0x0804, sr -> 0x241a, which the C's table lacked), and LCIDToLocaleName maps LOCALE_NEUTRAL/the custom defaults to the user locale and a neutral LCID to its default; ResolveLocaleName resolves neutrals ("en" -> "en-US"); GetLocaleInfoEx keeps its neutral special cases under LOCALE_NOUSEROVERRIDE; IsValidLocaleName(NULL) is FALSE; Thai/Mongolian marks are C1_ALPHA (Wine's make_unicode list, applied in kernel32's get_char_typeW because the CRT shares the table). **213 -> 203**: LCMapStringW honours its LCID (the C ignored it: a bad LCID mapped, a Turkic one got no Turkic casing); LCMapStringEx rejects an unknown name for locale-dependent maps only (casing, sortkey; kana/width succeed under any name, as on Windows); LCMAP_HALFWIDTH counts both units of a two-unit mapping, so a short buffer is ERROR_INSUFFICIENT_BUFFER instead of a truncated count and a one-past-the-end write. **203 -> 195**: Get{System,Thread,User}PreferredUILanguages treat only a NULL buffer as a size query (a real buffer with size 0 is ERROR_INSUFFICIENT_BUFFER); ConvertDefaultLocale resolves neutral "ga" to ga-IE (SUBLANG_DEFAULT is gd-GB) and, as on Win7+, leaves a neutral LCID with a non-default sort unchanged; GetGeoInfo(GEO_NATION) is 0 for a region. **195 -> 187**: EnumSystemLocalesA/W honour dwFlags (LCID_ALTERNATE_SORTS alone lists only Windows' alternate-sort LCIDs, with INSTALLED/SUPPORTED both) and stop when the callback returns FALSE. **187 -> 186**: hivesys.inf no longer lists my-MM (0x0455) and bn-BD (0x0845) under Nls\Locale; kernel32 carries no data for either, so they enumerated as invalid locales. **186 -> 184**: GetCalendarInfoA no longer refuses Unicode-only locales (dv-MV): their ANSI code page is 0 = CP_ACP, as on Windows. GetDateFormatA/GetTimeFormatA/GetNumberFormatA/GetCurrencyFormatA still carry the same refusal (lcformat.rs NLS_IsUnicodeOnlyLcid callers): check each against its own test before removing. **184 -> 167**: GetDateFormatA/GetTimeFormatA format fully in UTF-16 and return the ANSI byte count from the conversion (DBCS strings are longer than their UTF-16; the C returned the WCHAR count); IntGetCodePageEntry restores the caller's last error after loading a code page (its CreateFileW fallback cleared it to 0, so the first conversion in 1251/932 reset a successful call's error). **167 -> 165 (2026-11-22)**: SetThreadUILanguage sets a per-thread UI language (TLS slot in kernel32-rust; the NT 5.2 TEB has no field for it) and GetThreadUILanguage reads it; GetThreadUILanguage now lives in kernel32 at every export version and kernel32_vista forwards to it, so setter and getter share state (calling into kernel32-vista-rust's object instead pulled its duplicate CompareStringOrdinal into kernel32.dll, which link_guard refuses). **165 -> 97**: LCMapString(LCMAP_SORTKEY) builds NT sort keys from ntsort.rs's weights (primary SM/AW, 01, diacritic trimmed and reversed for French, 01, case trimmed, 01, 01, word-sort punctuation, 00), so a key compares byte-wise exactly as CompareString orders; a short buffer gets the sections that fit, cut after a separator. Old Wine table only if sortkey.nls cannot be loaded. **97 -> 95**: FoldStringW's MAP_PRECOMPOSED/MAP_COMPOSITE/MAP_FOLDCZONE are NFC/NFD/NFKC/NFKD via unorm-rust (the C left the first two as FIXMEs); FoldStringA converts plainly and passes the flags through, as current Wine (its old MB_COMPOSITE + forced MAP_FOLDCZONE workaround made every A call NFKC once FOLDCZONE meant NFKC: 75 regressions in the first run). **95 -> 92**: IDN nameprep is RFC 3491 (B.1/B.2 map from Wine's StringPrep tables, NFKC via unorm-rust, then A.1 unassigned / C.* prohibited / section 6 bidi checks); kernel32 compiles normaliz-rust's generated nameprep/mod.rs by #[path] in place of its one-element placeholder symbols. IdnToUnicode rejects unassigned code points unless IDN_ALLOW_UNASSIGNED. **Owed**: see RESUME HERE (2026-11-08) above. |
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
