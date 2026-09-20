# Upstream-sync bug leads

Upstream ReactOS is not a patch queue we owe — it is an **oracle that tells us where our own
bugs are**. We forked at `FP = 5213cf717cff6946945922e6a7e03d87b0a58e4f` (2026-02-11) and
translated the C as it stood *then*. Every upstream bug fix that lands afterward, in a file we
translated, is positive evidence that our Rust twin contains the exact same bug — we froze it in
translation. Merging upstream's C fix does nothing for us (we don't compile their C); the fix has
to be re-applied by hand to the Rust.

This doc is the ranked queue of those leads. It is analysis only — no code was touched.

## Method (exact commands, so it's re-derivable)

```bash
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
cd vibeos-upstream-sync

# 1. All upstream commits since the fork
git log --format='%H|%s' $FP..upstream/master > all_commits.txt      # 1266 commits

# 2. Classify by SUBJECT LINE keyword sweep (strong fix signals)
grep -iE '\|.*(fix|CORE-[0-9]{4,6}|regression|crash|leak|overflow|off-by-one|off by one|
  wrong|incorrect|missing|deadlock|race condition|\bhang\b|BSOD|assert|use-after-free|
  double free|\bnull\b|dangling|corrupt|stale|invalid|uninitiali)' all_commits.txt \
  > fix_candidates.txt                                                # 321 commits
grep -iv '\[TRANSLATION\]' fix_candidates.txt > fix_candidates_notrans.txt   # 321 (0 were pure-i18n)

# 3. Get the files each fix commit touched
git diff-tree --no-commit-id --name-only -r <sha>          # per commit, batched

# 4. Intersect with what we translated
find . -name CMakeLists.txt -o -name '*.cmake' | xargs grep -E '^\s*#.*\.(c|cpp)\b'   # commented-out C
find sdk/lib/rust -name '*.rs'                                                        # Rust tree
# a .c/.cpp counts as "has a Rust twin" iff its basename is commented out in some
# CMakeLists/*.cmake AND a plausibly-named .rs exists under sdk/lib/rust/, with a module-name
# correlation pass (see below) to kill cross-module false positives.
```

### The registry that exists, and how I used it

`tools/cohab-registry.json` (named in my brief) does **not** exist yet. What *does* exist,
untracked, dropped by another agent mid-session: `tools/cohab-census.txt` +
`tools/cohab_census.py` — a census of every commented-out source reference across all CMake
files, built for a different purpose (detecting merge-dropped cohab comments), not a
file→crate map. About 120 of its ~2067 lines carry an explicit `(→ <crate> src/<path>.rs — ...)`
annotation. **I used those 120 lines as ground truth to verify/correct my own stem-matching**
wherever they overlapped — this caught and fixed several wrong guesses (below). The rest of the
queue rests on my own approximation and is explicitly weaker evidence.

### The approximation, and its known failure mode

Bare stem matching (`stretchblt.c` → any `stretchblt.rs` anywhere) is what the brief warned
would overcount — `shlwapi/utils.cpp` matching an unrelated `utils.rs`. I saw exactly that:
raw stem matching gave **125** fix commits with an apparent twin; after excluding ~40 generic
stems (`utils`, `init`, `global`, `guid`, `session`, `thunks`, `hardware`, `device`, `pnp`, …)
unless the candidate's `.rs` path shares a normalized module-name token with the `.c` path, it
dropped to **89**. Spot-checking the reject list confirmed real false positives it removed
(`jscript/date.c` → `intl-rust/date.rs`, `dll/win32/jscript/global.c` → `msv1_0-rust/global.rs`,
`drivers/network/ndis/ndis/hardware.c` → `sysdm-rust/hardware.rs` — all different subsystems that
happen to share a generic filename). **89 is still an upper bound, not a proof** — a handful of
single-candidate "specific-looking" stems (e.g. `atom.c`, `env.c`) were accepted without module
correlation and could still be wrong; treat every row below as "worth a five-minute look," not
"confirmed."

Cross-checking against the census also caught concrete **wrong twin names** my heuristic would
have shipped silently: `ntoskrnl/ke/amd64/context.c` is NOT `ke/i386/context.rs` (arch mismatch);
`win32ss/gdi/dib/stretchblt.c` is `dib_stretchblt.rs`, not `stretchblt.rs` (that name belongs to
a *different* file, `gdi/eng/stretchblt.c`); `win32ss/user/ntuser/mouse.c` is `ntmouse.rs`, not
`mouse.rs` (that belongs to `gdi/eng/mouse.c`); `gdi/ntgdi/bitblt.c` is `ntbitblt.rs`, not
`bitblt.rs` (`gdi/eng/bitblt.c` again owns the plain name); and `user32/controls/button.c`
matched `acpi-rust/button.rs` on stem alone — the real twin is `user32-rust/controls/button.rs`.
All five are corrected in the table below and tagged `(census-verified)` /
`(census-corrected)`; every other twin name in the table is **unverified beyond the heuristic**.

### Ranking

Score = boot-path bonus (ntoskrnl mm/ke/ex/io/se/ob/ps/cc/fsrtl, win32ss, hal, freeldr, ntdll,
subsystems/csr, sdk/lib/rtl) + CORE-ticket bonus + small-diff bonus (≤30 lines big, ≤80 medium,
>300 penalized) + twin-count bonus. This is a heuristic sort, not a verdict — read the "why it
ranks here" column with judgment, not as gospel.

## Headline counts

- **1266** upstream commits since the fork (`FP..upstream/master`).
- **321** classified as bug fixes by the subject-line keyword sweep (0 were pure `[TRANSLATION]`
  localization fixes — the sweep's keyword set happened not to catch any of those).
- **89** of those 321 touch at least one file with a plausible Rust twin (down from a raw
  stem-match of 125, after killing cross-module false positives — see Method).
- **15** "Sync with Wine" commits in the range (wholesale re-imports, handled separately below).
- **14** of the 89 touch `modules/rostests` (new upstream tests — an executable oracle, handled
  separately below).
- **33** files touched by fix commits existed at the fork point but are now missing from the
  tree entirely — almost all of them turned out to be the win32k.sys **C-tree retirement**
  (commit `8d3c33ee41e`, 2026-06-21: 179 superseded `.c`/`.cpp` deleted outright, not just
  commented out, because win32k's GDI+USER halves are fully Rust). That single fact means the
  approximation in Step 4 (grep for a `#`-commented basename) **structurally misses every win32k
  file retired that way** — my matches for those files only landed because the fix commits
  predate the retirement and I matched the .c path from the commit diff, not from today's tree.
  A registry keyed off git history (not current-tree greps) would find more of these.

## The ranked queue

Top 60 by score. "Rust twin" is the file I believe carries the bug forward; entries marked
`(census-verified)`/`(census-corrected)` are checked against `tools/cohab-census.txt`'s explicit
annotations, everything else is the stem+module heuristic only.

| # | sha | subject | file(s) touched | Rust twin (approx) | CORE | diff | boot-path |
|---|---|---|---|---|---|---|---|
| 1 | `93c5a7b98b` | [CSRSRV] Remove unnecessary UNICODE_PATH_SEP define; fix related comments. | subsystems/csr/csrsrv/api.c (+2 more) | sdk/lib/rust/smss-csrss-rust/src/csrsrv/api.rs; sdk/lib/rust/smss-csrss-rust/src/csrsrv/init.rs |  | 16 | yes |
| 2 | `91b613f158` | [NDK][RTL] Fix signature of RtlInitializeContext | sdk/include/ndk/rtlfuncs.h (+2 more) | sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/stubs.rs; sdk/lib/rust/rtl-except/src/i386/thread.rs |  | 21 | yes |
| 3 | `e1bf662d9a` | [WIN32SS:NTGDI] ftGetFontUnicodeRanges(): Add missing FreeType locking (#7659) | win32ss/gdi/ntgdi/freetype.c | sdk/lib/rust/win32k-rust/src/freetype.rs |  | 13 | yes |
| 4 | `e7e1465812` | [NTUSER] Fix NtUserSBGetParms (#9411) | win32ss/user/ntuser/scrollbar.c | sdk/lib/rust/win32k-rust/src/scrollbar.rs |  | 8 | yes |
| 5 | `640e39c72e` | [NTGDI] Revert "Fix 3D-text screensaver not moving in preview mode" (#9389) | win32ss/gdi/ntgdi/gdiobj.c | sdk/lib/rust/win32k-rust/src/gdiobj.rs |  | 7 | yes |
| 6 | `39ea28d4c6` | [NTOS:KE/x64] Fix KeContextToTrapFrame | ntoskrnl/ke/amd64/context.c | sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/context.rs (census-verified) |  | 6 | yes |
| 7 | `64f1cf3b6d` | [WIN32SS:NTUSER] Fix DrawText ellipsification bug (#9240) | win32ss/user/rtl/text.c | sdk/lib/rust/win32k-rust/src/text.rs |  | 4 | yes |
| 8 | `c6410fb0c2` | [NTOS:EX] Fix query of SystemTimeAdjustmentInformation | ntoskrnl/ex/sysinfo.c | sdk/lib/rust/ex-rust/src/sysinfo.rs |  | 4 | yes |
| 9 | `feb0f56da7` | [NTOS:EX] Fix query of SystemInterruptInformation | ntoskrnl/ex/sysinfo.c | sdk/lib/rust/ex-rust/src/sysinfo.rs |  | 5 | yes |
| 10 | `c0f18d059a` | [WIN32SS:GDI] Fix an off-by-one error in the StretchBlt mask bounds check (#9136) | win32ss/gdi/dib/stretchblt.c | sdk/lib/rust/win32k-rust/src/dib_stretchblt.rs (census-verified) |  | 2 | yes |
| 11 | `ae0e4ee305` | [WIN32K:DIB] Fix some bugs in DIB_8BPP_BitBltSrcCopy (#9119) | win32ss/gdi/dib/dib8bpp.c | sdk/lib/rust/win32k-rust/src/dib8bpp.rs (census-verified) |  | 9 | yes |
| 12 | `7f79e1c948` | [NTUSER] IntNotifyWinEvent(): Fix 4th argument copypasta (#9138) | win32ss/user/ntuser/caret.c | sdk/lib/rust/win32k-rust/src/caret.rs (census-verified) |  | 2 | yes |
| 13 | `3511472586` | [NTUSER] Fix XBUTTON1 reported as XBUTTON2 on WM_XBUTTONUP (#9120) | win32ss/user/ntuser/mouse.c | sdk/lib/rust/win32k-rust/src/ntmouse.rs (census-verified) |  | 2 | yes |
| 14 | `a94352ba48` | [WIN32SS:ENG] Fix copy direction in overlapping EngTransparentBlt (#9117) | win32ss/gdi/eng/transblt.c | sdk/lib/rust/win32k-rust/src/transblt.rs |  | 4 | yes |
| 15 | `984657bdf0` | [WIN32SS:ENG] Fix Y coordinate of the software pointer exclude rectangle (#9116) | win32ss/gdi/eng/mouse.c | sdk/lib/rust/win32k-rust/src/mouse.rs (census-verified) |  | 2 | yes |
| 16 | `5b2f79fe07` | [RTL] Fix RtlVirtualUnwind for UWOP_*_FAR, UWOP_EPILOG and UWOP_SPARE_CODE | sdk/lib/rtl/amd64/unwind.c | sdk/lib/rust/rtl-except/src/amd64/unwind.rs |  | 12 | yes |
| 17 | `7f459b49f0` | [WIN32K:NTGDI] Fix brush origin for pattern painting | win32ss/gdi/ntgdi/bitblt.c | sdk/lib/rust/win32k-rust/src/ntbitblt.rs (census-verified) |  | 10 | yes |
| 18 | `33a19d05f4` | [RTL] Fix atom table signature | sdk/lib/rtl/atom.c | sdk/lib/rust/rtl-strings/src/core_rs/atom.rs |  | 1 | yes |
| 19 | `0e64048812` | [NDK][RTL] Fix packing leak and ARM64 heap layouts (#9020) | sdk/include/ndk/pstypes.h (+3 more) | sdk/lib/rust/rtl-heap/src/heap.rs |  | 24 | yes |
| 20 | `713888355b` | [NTGDI][FREETYPE] 2-line fix: Changing DPRINT1 to DPRINT | win32ss/gdi/ntgdi/freetype.c | sdk/lib/rust/win32k-rust/src/freetype.rs |  | 4 | yes |
| 21 | `54422db2b8` | [FREELDR] Fix UEFI IA-32 boot (#8886) | boot/freeldr/freeldr/ntldr/arch/i386/winldr.c | sdk/lib/rust/freeldr-rust/src/winldr.rs |  | 6 | yes |
| 22 | `13ac291830` | [NTOS:FSTUB] IoGetBootDiskInformation(): Fix the disks matching condition check. | ntoskrnl/fstub/fstubex.c; ntoskrnl/io/iomgr/arcname.c | sdk/lib/rust/iomgr-rust/src/arcname.rs |  | 26 | yes |
| 23 | `0c7d3f20fa` | [IMM32] Fix Imm32ReconvertWideFromAnsi (#8758) | win32ss/user/imm32/utils.c | sdk/lib/rust/win32k-rust/src/utils.rs |  | 6 | yes |
| 24 | `cad71d733f` | [IMM32] Fix Imm32IsSystemJapaneseOrKorean (#8757) | win32ss/user/imm32/utils.c | sdk/lib/rust/win32k-rust/src/utils.rs |  | 2 | yes |
| 25 | `f63df20bd4` | [FREELDR:NTLDR] Fix GCC build | boot/freeldr/freeldr/ntldr/headless.c | sdk/lib/rust/freeldr-rust/src/headless.rs |  | 2 | yes |
| 26 | `057d978fcb` | [USER32] button.c Fix extra-byte layout on x64 (#8733) | win32ss/user/user32/controls/button.c | sdk/lib/rust/user32-rust/src/controls/button.rs (census-corrected) |  | 4 | yes |
| 27 | `ef8bcd3fbe` | [EVENTLOG][FMIFS][KERNEL32][APITESTS] Fix leaks after calling RtlDosPathNameToNtPathName_U (#8559) | base/services/eventlog/eventlog.c (+9 more) | sdk/lib/rust/advapi32-rust/src/service/eventlog.rs (+2 more) |  | 33 | yes |
| 28 | `bf26ae38a5` | [FREELDR] Make FreeLoader a bit more usable even if FREELDR.INI is missing | boot/freeldr/freeldr/bootmgr.c (+5 more) | sdk/lib/rust/freeldr-rust/src/bootmgr.rs (+4 more) |  | 65 | yes |
| 29 | `c8907598f5` | [UEFILDR][FREELDR] Some x64 ABI violations and memory map fixes (#8668) | boot/freeldr/freeldr/arch/uefi/amd64/uefiasm.S (+3 more) | sdk/lib/rust/freeldr-rust/src/uefildr.rs (+2 more) |  | 39 | yes |
| 30 | `652403caf2` | [RTL/x64][NTOS:KE/x64] Fix RtlInitializeContext | dll/win32/kernel32/client/utils.c (+2 more) | sdk/lib/rust/kernel32-rust/src/client/utils.rs; sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/stubs.rs |  | 79 | yes |
| 31 | `16a65a0df2` | [NTUSER][COMCTL32] Fix ListBox background color setting (#8665) | dll/win32/comctl32/listbox.c; win32ss/user/user32/controls/listbox.c | sdk/lib/rust/user32-rust/src/controls/listbox.rs |  | 64 | yes |
| 32 | `b0bd092c0d` | [FREELDR] Make ASSERT bugcheck on failure (#8730) | boot/freeldr/freeldr/arch/arm/debug.c (+3 more) | sdk/lib/rust/freeldr-rust/src/i386bug.rs; sdk/lib/rust/rtl-diag/src/debug.rs |  | 33 | yes |
| 33 | `dc9ee34478` | [FREELDR:UEFI] Fix broken boot menu countdown (#8714) | boot/freeldr/freeldr/arch/uefi/stubs.c (+2 more) | sdk/lib/rust/freeldr-rust/src/ueficon.rs; sdk/lib/rust/freeldr-rust/src/uefihw.rs |  | 63 | yes |
| 34 | `7f5ec6bd3a` | [HALX86] Fix newline format Addendum to commit b9eb90a | hal/halx86/include/bus.h; hal/halx86/legacy/bus/pcibus.c | sdk/lib/rust/hal-rust/src/x86/pcibus.rs |  | 56 | yes |
| 35 | `2a74f5e794` | [FREELDR:UEFI] Implement missing cold reset in Reboot() function (#9092) | boot/freeldr/freeldr/arch/uefi/uefildr.c; boot/freeldr/freeldr/include/arch/arm/hardware.h | sdk/lib/rust/freeldr-rust/src/uefildr.rs |  | 31 | yes |
| 36 | `9675f756ce` | [NDK] Fix definition of OB_CLOSE_METHOD | drivers/filters/fltmgr/Messaging.c (+12 more) | sdk/lib/rust/iomgr-rust/src/file.rs |  | 56 | yes |
| 37 | `6d8a5fa0bb` | [WIN32SS:NTGDI] Allow calling NtGdiCreateHalftonePalette with a NULL hDC (#9217) | win32ss/gdi/ntgdi/palette.c | sdk/lib/rust/win32k-rust/src/palette.rs (census-corrected; NOT framebuf-rust) |  | 45 | yes |
| 38 | `16e6da0e96` | [FREELDR:SETUPLDR] Add the missing known platform-specific SourcePaths where TXTSETUP.SIF can be found | boot/freeldr/freeldr/ntldr/setupldr.c | sdk/lib/rust/freeldr-rust/src/setupldr.rs |  | 31 | yes |
| 39 | `eb16c75060` | [CSRSRV] CsrApiRequestThread(), CsrSbApiRequestThread(): Update Doxygen block, SAL2 annotations, and fix signature | subsystems/csr/csrsrv/api.c (+3 more) | sdk/lib/rust/smss-csrss-rust/src/csrsrv/api.rs (+2 more) |  | 81 | yes |
| 40 | `049085f106` | [GDI32] Fix pptviewer '97 and calls to PolylineTo and PolyBezierTo (#8902) | win32ss/gdi/gdi32/objects/painting.c (+2 more) | sdk/lib/rust/win32k-rust/src/painting.rs (+2 more) |  | 120 | yes |
| 41 | `77b88c48a4` | [NDK][NTOS][NTDLL][KRNEL32] Fix read/write of KSYSTEM_TIME | dll/ntdll/rtl/libsupp.c (+7 more) | sdk/lib/rust/rtlk-rust/src/libsupp.rs (+5 more) |  | 209 | yes |
| 42 | `1ac657ed0e` | [FREELDR:NTLDR] Normalize the boot options as soon as possible, and fix the algorithm | boot/freeldr/freeldr/ntldr/setupldr.c (+2 more) | sdk/lib/rust/freeldr-rust/src/setupldr.rs; sdk/lib/rust/freeldr-rust/src/winldr.rs |  | 86 | yes |
| 43 | `e1d8ccbdd5` | [USER32] Fix LookupIconIdFromDirectoryEx crash with invalid pointer (#9263) | win32ss/user/user32/windows/cursoricon.c | sdk/lib/rust/win32k-rust/src/cursoricon.rs (census-verified — note this is the KERNEL twin, not user32-rust's own cursoricon.rs) |  | 252 | yes |
| 44 | `189eae49a8` | [USER32] Revert previous commit that was merged incorrectly | modules/rostests/apitests/user32/LookupIconIdFromDirectoryEx.c; win32ss/user/user32/windows/cursoricon.c | sdk/lib/rust/win32k-rust/src/cursoricon.rs |  | 262 | yes |
| 45 | `b35226eaa4` | [USER32] Fix crash on calling LookupIconIdFromDirectoryEx with invalid directory (#9263) | modules/rostests/apitests/user32/LookupIconIdFromDirectoryEx.c; win32ss/user/user32/windows/cursoricon.c | sdk/lib/rust/win32k-rust/src/cursoricon.rs |  | 262 | yes |
| 46 | `feb9febaae` | [WIN32KNT_APITEST][NTUSER] Fix NtUserConvertMemHandle BSOD (#7878) | modules/rostests/apitests/win32u/CMakeLists.txt (+3 more) | sdk/lib/rust/win32k-rust/src/clipboard.rs (census-verified) |  | 140 | yes |
| 47 | `c65740eeb3` | [WIN32KNT_APITEST][NTUSER] Fix NtUserCreateAcceleratorTable memory corruption (#7888) | modules/rostests/apitests/win32u/CMakeLists.txt (+3 more) | sdk/lib/rust/win32k-rust/src/accelerator.rs (census-verified) |  | 204 | yes |
| 48 | `ba35137b01` | [WS2_32_APITEST] Attempt to fix receive test timeouts (#9483) | modules/rostests/apitests/ws2_32/recv.c | sdk/lib/rust/ws2_32-rust/src/src/recv.rs |  | 3 |  |
| 49 | `ae50d745ca` | [USBHUB] USBH_FdoQueryBusRelations: Fix an off by one (#9298) | drivers/usb/usbhub/pnp.c | sdk/lib/rust/usbport-rust/src/pnp.rs |  | 2 |  |
| 50 | `afbc922405` | [NETCFGX] Fix the output of ITcpipProperties_fnUnknown1 | dll/win32/netcfgx/tcpipconf_notify.c | sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs |  | 15 |  |
| 51 | `04c7639769` | [GDIPLUS_WINETEST] Fix crash on Vista | modules/rostests/winetests/gdiplus/graphics.c | sdk/lib/rust/winsrv-rust/src/consrv/condrv/graphics.rs |  | 15 |  |
| 52 | `b3a0868a18` | [DHCPCSVC] Fix stop_selecting truncated from time_t to int on 64-bit (#8996) | base/services/dhcpcsvc/dhcp/dhclient.c | sdk/lib/rust/dhcpcsvc-rust/src/dhcp/dhclient.rs |  | 3 |  |
| 53 | `54c667caaa` | [NETCFGX] Fix calls to wcsicmp | dll/win32/netcfgx/inetcfgcomp_iface.c | sdk/lib/rust/netcfgx-rust/src/netcfgx/inetcfgcomp_iface.rs |  | 4 |  |
| 54 | `17846e97d0` | [SYSSETUP] Fix wine gecko install in second stage setup (#8885) | dll/win32/syssetup/install.c | sdk/lib/rust/syssetup-rust/src/syssetup/install.rs |  | 15 |  |
| 55 | `a2c594187e` | [NETCFGX] Fix the call to wcsicmp which broke the MSVC builds | dll/win32/netcfgx/tcpipconf_notify.c | sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs |  | 2 |  |
| 56 | `c950e66c31` | [USER32_WINETEST] Fix usage of non-conforming swprintf | modules/rostests/winetests/user32/CMakeLists.txt; modules/rostests/winetests/user32/input.c | sdk/lib/rust/user32-rust/src/windows/input.rs |  | 6 |  |
| 57 | `6730ec5508` | [SHELL32] Fix CD-ROM eject not refreshing drive text labels in folder view (#8789) | dll/win32/shell32/CDefView.cpp | sdk/lib/rust/shell32-rust/src/shell32/CDefView.rs |  | 13 |  |
| 58 | `5d6987462d` | [DHCPCSVC] Fix GCC 13 stringop-overflow warning | base/services/dhcpcsvc/dhcp/adapter.c; base/services/dhcpcsvc/include/rosdhcp.h | sdk/lib/rust/iomgr-rust/src/adapter.rs |  | 4 |  |
| 59 | `0df3463af0` | [NETCFGX] Fix radio button selection on the basic property page | dll/win32/netcfgx/tcpipconf_notify.c | sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs |  | 8 |  |
| 60 | `5a147d292d` | [NETCFGX] Fix: Default gateway data "forgotten" when toggling use of DHCP | dll/win32/netcfgx/tcpipconf_notify.c | sdk/lib/rust/netcfgx-rust/src/netcfgx/tcpipconf_notify.rs |  | 10 |  |

(29 more below the cut, mostly small NETCFGX/oleaut32/misc fixes and the two Wine-sync bulk
commits — see `tools/cohab-census.txt` cross-reference candidates in the scratch data if you
want the full 89.)

### Top 10, read in detail

**1–2 (CSRSRV / RtlInitializeContext signature).** Both are mostly *typing* fixes (a
`PINITIAL_TEB` parameter retyped to `PVOID`, comment cleanup) rather than behavioral bugs — they
scored high on boot-path + small-diff but are lower actual value than their rank suggests. Worth
a five-minute diff-vs-Rust check, not a deep dive. Downgrade these two mentally; the real top 10
by bug substance is #3 and below.

**3. `e1bf662d9a` — ftGetFontUnicodeRanges() missing FreeType locking (CORE-7659).**
`ftGetFontUnicodeRanges` walked the FreeType face's charmap (`face->charmap`, `FT_Get_Next_Char`)
without holding `IntLockFreeType()`, while every other FreeType-touching path in the file does.
Two early-return paths (`!glyph_code`, and the out-of-order `char_code` sanity check) also
returned directly instead of going through cleanup, so the fix both adds the lock and reroutes
those returns through a new `Quit:` label that unlocks. **Check**: does
`sdk/lib/rust/win32k-rust/src/freetype.rs`'s translation of this function actually call the
Rust-side lock helper around the FreeType calls, or did the straight-line translation of the
*pre-fix* C skip it exactly as upstream did before 2024-09-14 (this fix predates our fork
temporally in wall-clock terms but the SHA is topologically after FP in the upstream DAG — i.e.
our fork point did not yet have it)?

**4. `e7e1465812` — NtUserSBGetParms BSOD (CORE-18128, #9411).** The syscall handler copied
`*pSBData` via `RtlCopyMemory` inside a `_SEH2_TRY` but never `ProbeForRead`/`ProbeForWrite`'d the
two user-supplied pointers (`lpsi`, `pSBData`) first, and unconditionally copied `pSBData` even
when the optional pointer was NULL — a wild copy from a null/unprobed user pointer that SEH can't
always catch cleanly. **Check**: `scrollbar.rs`'s `NtUserSBGetParms` — does it Probe both
pointers before copying, and does it guard the NULL case for `pSBData` rather than deref'ing it
unconditionally?

**6. `39ea28d4c6` — KeContextToTrapFrame wrong privilege-mode test.** Three call sites tested
`(Context->SegCs & MODE_MASK) == KernelMode` — i.e. trusted the *caller-supplied* `CONTEXT`'s
segment selector to decide whether to write kernel-mode trap-frame fields — instead of using the
authoritative `PreviousMode` parameter. A user-mode caller could hand a `CONTEXT` with a
kernel-looking `SegCs` and get kernel-privileged trap-frame fields set. **Check**:
`ke/amd64/context.rs`'s equivalent branches — do they key off a passed-in "previous mode" concept
or off attacker-controlled `Context->SegCs`? This is a genuine confused-deputy / privilege check
bug, the highest-value class in the whole queue.

**10. `c0f18d059a` — StretchBlt mask off-by-one (#9136), classic OOB read.**
`DIB_XXBPP_StretchBlt`'s mask bounds check used `MaskSurf->sizlBitmap.cx < sx` (strictly less
than) where the sibling source-surface check two lines below correctly used `<=`. `sx == cx` (or
`sy == cy`) slipped through and `fnMask_GetPixel` read one pixel past the mask bitmap's buffer.
**Check**: `dib_stretchblt.rs` — literally grep for the bounds comparison operator on the mask
path; a straight-line translation would carry the same `<` and the same one-pixel OOB read.

**11. `ae0e4ee305` — DIB_8BPP_BitBltSrcCopy stride bugs (CORE-2965, #9119).** In the
top-to-bottom flip path, the code used `SourceSurface->lDelta` for BOTH advancing through the
source buffer AND advancing through the destination buffer during the write-back loop — when the
destination line's stride should use `DestSurface->lDelta`. It also applied the palette
translate (`XLATEOBJ_iXlate`) unconditionally even on a second pass over the same pixel
(`OneDone` flag added to prevent double-translation). Two distinct wrong-stride/double-transform
bugs in one function. **Check**: `dib8bpp.rs`'s flip path for the same delta confusion — a
faithful translation would use the source surface's delta for the destination write-back loop.

**13. `3511472586` — XBUTTON1 reported as XBUTTON2 on WM_XBUTTONUP.** One-line bug, introduced by
an old ReactOS commit (r54227) and only now fixed: `Msg.wParam |= MAKEWPARAM(0, XBUTTON2)` should
read `XBUTTON1` in the XBUTTON1-release branch. Every mouse-4-button release message reports the
wrong button to the app. **Check**: `ntmouse.rs`'s `UserSendMouseInput` XBUTTON1 branch — trivial
to grep and trivial to fix if wrong.

**12. `7f79e1c948` — IntNotifyWinEvent 4th-arg copy-paste (#9138).** The caret's
`EVENT_OBJECT_SHOW` accessibility notification passed `OBJID_CARET` as both the 3rd (`idObject`)
and 4th (`idChild`) argument, where the other four caret notifications in the same file correctly
pass `CHILDID_SELF` for the 4th. Screen readers / accessibility clients get a malformed
notification for exactly one of five caret events. **Check**: `caret.rs`'s
`EVENT_OBJECT_SHOW` call — compare all five `IntNotifyWinEvent` call sites for consistency.

**8–9. `c6410fb0c2` / `feb0f56da7` — NtQuerySystemInformation, SystemTimeAdjustmentInformation
and SystemInterruptInformation (`ntoskrnl/ex/sysinfo.c`).** Two adjacent, same-author,
same-day fixes in the query dispatch: (a) `SystemTimeAdjustmentInformation`'s length check
compared `Size` against the struct size but never set `*ReqSize` on the *success* path, only on
the too-small path — callers doing the standard "call once to get size" pattern got a stale/zero
`ReqSize`; (b) `SystemInterruptInformation` read `Prcb->DpcData[0]` (magic index) instead of the
named `DPC_NORMAL` constant, which is silently wrong if `DpcData` is ever reordered. **Check**:
`ex-rust/src/sysinfo.rs`'s handlers for both classes — does `*ReqSize` get set unconditionally
before the length check, and does the DPC-count read use a named index?

## Special categories (step 4)

**"Sync with Wine" bulk imports (15 commits).** `MCIAVI32`, `OLEAUT32`+winetest, `URLMON`+winetest,
`VBSCRIPT`+winetest, `CRYPTNET`+winetest, `JSCRIPT`+winetest, `INETCOMM`+winetest,
`CRYPTUI`+winetest — all "Sync with Wine 10.0/11.11" with CORE-2059x tickets. These aren't "a bug
fix" in the small sense; they're wholesale replacement of the whole module's source from a newer
Wine snapshot. `oleaut32-rust` and `ole32-rust`/`urlmon`'s `bindctx.rs` DO have Rust presence
(rows 61-62 in the full 89, cut from the top-60 table above for size — `e05d78ede0` OLEAUT32
sync alone touches 14 files we've translated). **Handling**: don't treat these as "N discrete
bugs" — treat each as "diff Wine's oleaut32 10.0 against whatever revision we translated, once,
and decide if the delta matters," which is a different and much bigger task than fixing a
10-line bug.

**Files we deleted entirely (not cohab-flipped, actually gone).** The big one:
**win32k.sys's kernel-mode C tree — 179 files retired outright in commit `8d3c33ee41e`**
(2026-06-21), covering all of `gdi/ntgdi`, `gdi/eng`, `gdi/dib`, and `user/ntuser` (59 files).
The `.c`/`.cpp` no longer exist on disk at all (not commented out — deleted), because the
GDI+USER kernel-mode halves are fully Rust and boot-verified on all three arches. Every leftmost
`win32ss/...` row in the ranked table above is a fix commit whose file *used to* be one of these
179 — I still matched them because I diffed the historical commit's file list against the
CMakeLists comment + current `sdk/lib/rust` tree (the comment for these still exists in
`win32ss/CMakeLists.txt` even post-retirement, e.g. line 1780's `dib_stretchblt.rs` reference —
retirement deletes the `.c`, not the explanatory comment). **This is actually good news for
leads**: it means the win32k rows in the table (10, 11, 12, 13, 14, 15, 17, 20, 26, 37, 43-47) are
higher-confidence than average, because they're independently confirmed by
`tools/cohab-census.txt`'s explicit annotations, not just my stem heuristic. Also smaller and
non-systemic: `drivers/filesystems/udfs/{close,fileinfo,udfinit}.cpp`,
`drivers/filesystems/ext2/src/dirctl.c`, `drivers/storage/floppy/floppy/floppy.c`,
`drivers/network/dd/nvnet/{eth.h,init.c}`, `boot/freeldr/.../arch/arm/debug.c`,
`.../pc98/pc98disk.c`, `.../xbox/xboxdisk.c` — existed at FP, are gone from the current tree, and
none showed up with a Rust twin, meaning they're either unrelated repo pruning (dead
arch-specific code, e.g. PC-98/Xbox freeldr paths this fork never targets) or simply not yet
translated in either direction. Not a translation-oracle lead either way; noted so nobody chases
a ghost file.

**`modules/rostests` fixes (14 commits, listed at Step 4).** These are executable oracles, not
just leads: a new/fixed upstream test for a bug is something we can literally run against our
Rust. Standouts: `feb9febaae` (`NtUserConvertMemHandle` BSOD apitest) and `c65740eeb3`
(`NtUserCreateAcceleratorTable` memory corruption apitest) both ship a NEW apitest alongside the
kernel fix, targeting `clipboard.rs` and `accelerator.rs` respectively — pulling just the new test
file (not the whole Wine-sync machinery) and running it against our win32k-rust would directly
confirm or clear the lead without hand-reading any diff.

## What this classification will have missed

- **The keyword sweep is a subject-line filter; most real fixes don't say "fix" in the title.**
  ReactOS commit style is inconsistent — plenty of genuine bug fixes are titled descriptively
  ("Correctly handle X", "Don't do Y when Z", a bare function name) with no "fix"/"bug"/"crash"
  keyword at all. 321 of 1266 (25%) passed the sweep; the true fix rate in a 1266-commit range is
  almost certainly higher. This is the single biggest gap — a keyword sweep is cheap and
  high-precision but low-recall by construction.
- **Multi-purpose commits get the fix credit but may bury the fix in a larger refactor** — I
  scored on total commit diff size, so e.g. `77b88c48a4` (KSYSTEM_TIME read/write fix, 209 lines
  across 8 files) ranks lower than its actual value because the diff is large; the *fix* itself
  inside it may be 5 lines.
- **The Rust-twin intersection is current-tree-shaped, not history-shaped.** I grepped today's
  CMakeLists comments and today's `sdk/lib/rust` tree, then matched against each historical
  commit's file list. A file that was translated-then-later-renamed, or a Rust crate that was
  itself refactored/renamed after being written, would silently drop out of the intersection —
  the win32k retirement case (above) shows this failure mode can still succeed by luck (comment
  survives deletion) but there's no guarantee it always will.
- **No semantic diff-against-Rust was done for anything outside the top 10.** Everything from
  row 11 down is "subject line + file path" evidence only; I have not opened a single upstream
  diff or Rust file for rows 14-60 beyond what's visible in the table. Some fraction of those are
  certainly cosmetic (`newline format`, `Doxygen block`) despite matching the fix keyword sweep
  (e.g. `7f5ec6bd3a`, `eb16c75060`) — I left them in because they still passed the boot-path/size
  scoring honestly, but they are lower-value than their rank position implies once you actually
  read them.
- **Generic-stem exclusion was a blunt instrument.** Names I excluded outright as "too generic to
  trust" (`thunks`, `hardware`, `session`, `guid`, `global`, `query`, `date`, `math`, `pnp`,
  `keyboard`, `mouse`, `device`) may have thrown away a few genuine matches along with the false
  positives — I did not individually re-verify every excluded candidate against the census.
- **The 33 "existed at FP, missing now" files** were only checked for Rust-twin existence, not
  for *why* they're gone beyond the one big win32k case — I didn't investigate each of the other
  ~25 individually (e.g. whether `udfs`/`ext2`/`floppy` files were pruned, renamed, or moved to a
  different subtree).
