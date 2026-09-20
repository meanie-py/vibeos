# Upstream-sync: files upstream changed that are absent from our tree

Fork point `FP` = `5213cf717cff6946945922e6a7e03d87b0a58e4f` (2026-02-11).
Measured against `upstream/master` and our tip `4cb03718d99` on branch `upstream-sync`.

## How this was derived (re-run these to check my numbers)

```bash
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f

# 1. Files upstream touched since the fork (4519 total)
git diff --name-only $FP upstream/master > upstream_changed.txt

# 2. Of those, which are absent from our working tree right now (956)
while IFS= read -r f; do [ -e "$f" ] || echo "$f"; done < upstream_changed.txt > absent.txt

# 3. Bucket A vs B: did the path exist in the FORK-POINT tree?
while IFS= read -r f; do
  if git cat-file -e "$FP:$f" 2>/dev/null; then echo "$f" >> bucketB.txt
  else echo "$f" >> bucketA.txt; fi
done < absent.txt
# bucketA.txt = 646, bucketB.txt = 310  (646 + 310 = 956, checks out)

# 4. Bucket B: find the commit that deleted each path in OUR history
while IFS= read -r f; do
  git log --diff-filter=D --format='%h %s' $FP..HEAD -- "$f" | head -1
done < bucketB.txt
# all 310 resolved to a real deleting commit -> 0 fell through to "no delete commit found"

# 5. Rename check: did upstream RENAME a file whose old name we still carry, making a
#    bucket-A "add" actually just the new name of something already in our tree?
git diff -M --name-status $FP upstream/master | grep '^R' > upstream_renames.txt   # 45 renames
# of the 45 new-names, all 45 land in bucketA; checked whether the OLD name still
# exists in our tree: 40 do (real bucket-C candidates), 5 don't (see caveat below)
```

## Measured counts

| Bucket | Count | Notes |
|---|---:|---|
| **Total absent** (upstream changed, missing from our tree) | **956** | |
| **A — new upstream work** (didn't exist at FP) | **606** | after moving the 40 rename-matches out to C |
| **B — we deleted it** (existed at FP, removed in our history) | **310** | resolved to a deleting commit for all 310; 0 fell into B3 |
| &nbsp;&nbsp;B1 — deleted because translated/replaced by Rust | 63 | bug leads |
| &nbsp;&nbsp;B2 — deleted on purpose, other reason (incl. 2 "moved not deleted") | 247 | |
| &nbsp;&nbsp;B3 — reason not determinable from commit message | 0 | every commit message was explicit enough to classify |
| **C — rename on one side, counterpart already in our tree** | **40** | |
| Sum check | 606 + 310 + 40 = **956** | matches measured absent count |

The task brief's "~810" estimate was low; the real number is 956 (upstream's changed-file
count is also higher than estimated: 4519, not ~3958 — likely drift since the brief was
written, or a different diff arg). Recommend re-running the commands above right before the
actual merge, since both branches keep moving.

### Caveat on bucket C's rename detection

`git diff -M` (default 50% similarity threshold) paired 45 upstream deletes with 45 upstream
adds. 40 of those pairs have their *old* name still present in our tree — solid bucket-C
evidence, from a same-content `R100` match down to a probabilistic `R050`. The remaining 5
pairs (all involving `modules/rosapps/applications/explorer-old/*` as the claimed "old" side)
are almost certainly **spurious pairings**: git's similarity heuristic matched deleted
`explorer-old` files against unrelated new upstream files purely on rough byte-shape (e.g.
`explorer-old/precomp.cpp` -> `base/services/rpcss/irpcss.idl`, `explorer-old/externals.h` ->
`sdk/include/psdk/ioringapi.h`), at low confidence (R052–R074) and with no plausible
name/content relationship. Those 5 were left in bucket A as ordinary new files, not folded
into C.

## Bucket B — full list, grouped by deleting commit (B1 first)

### B1 — deleted because translated/replaced by Rust (BUG LEADS — read these first)

**`8d3c33ee41e` — "win32k.sys: retire superseded kernel-mode C tree — Rust-only (179 files)"**
(49 of those 179 paths are also touched by upstream since the fork — i.e. upstream kept
patching C that we deleted because win32k is now Rust-only. Each one is a candidate: does our
Rust already have the fix upstream just made to the C?)

```
win32ss/user/ntuser/... (and sibling win32ss/user/* files) — see full path list:
```
- win32ss/user/ntuser/*.c / *.h and the `.splitN.c` shards upstream still edits under
  `win32ss/user/ntuser/`, e.g. `winpos.split3.c`, `winpos.split4.c`, `winsta.c`,
  `winsta.split2.c`, plus 45 sibling paths in the same tree (full 49-path list is
  reproducible via the script above with commit `8d3c33ee41e`; omitted here for length —
  every one is under `win32ss/user/`, the exact tree the commit subject names as
  "Rust-only").

**`3feaf6ca5f6` — "net: delete the dead tcpip.sys stack + the orphaned lan driver (~189K lines)"**
(14 files) — `rust-port-status.md` confirms this is a real B1: "the legacy NDIS/tcpip/tdi
stack (dead, superseded by netstack)... from-scratch Rust rewrite (100%, zero C, replacing the
legacy NDIS/tcpip stack)." Any upstream fix to the deleted tcpip.sys/lan driver C is a
candidate bug that may also live in the Rust `netstack`/`ws2_32`/`afd`/`iphlpapi` chain — worth
a diff-and-triage pass, not a blanket ignore.

### B2 — deleted on purpose, other reason (scope narrowing / dead code / vendor reorg)

| Files | Commit | Subject |
|---:|---|---|
| 190 | `abf4af7f353` | rosapps: delete explorer-old, fraginator and nukecamp — 59,710 lines, none built |
| 9 | `fe69a065ec3` | build: delete xbox + pc-98 + arm32 dead-platform support |
| 6 | `4e0d87895ac` | build: delete 32-bit ARM source dirs (16 dirs, 74 files, ~4900 LOC) |
| 5 | `c097b8a9346` | build: drop drivers/sac/ (Server-only SAC) + arm64ec dead branch |
| 5 | `be5f5abf300` | filesystems: delete nfs / udfs / vfatfs — useless non-target FS drivers |
| 4 | `e0cccdc876a` | Pre-Vista prune E5: drop legacy NICs + bluetooth + TV-tuner; fix latent E3b HalpTrap0D |
| 4 | `dc52225e248` | Pre-Vista prune E4: drop obsolete device drivers |
| 4 | `5ca4d482d32` | BIOS prune (5/N): git-rm dead BIOS-boot source files |
| 4 | `457efb25554` | hal: prune dead trees + unused DLL targets (Stage 0 prep) |
| 4 | `14a0563b291` | filesystems: remove ext2, btrfs, ntfs (driver + format-lib + IFS-dll + shellext) |
| 2 | `ebf1ac53a99` | Pre-Vista prune E3b: sever videoprt INT10 + retire HAL x86bios + fast486 |
| 2 | `c474d932eb7`* | vendor: libwine + strmbase are OURS — out of the 3rdparty container |
| 2 | `bdea86df186` | Pre-Vista prune E3a: drop legacy VGA/INT10 video miniports + display |
| 2 | `b5068fbdb54` | freeldr: prune dead btrfs/ext/ntfs FS read drivers + dead directui.c (−5098 LOC) |
| 1 | `f4b81767fb6` | Pre-Vista prune E1: drop ISAPNP + PCMCIA bus drivers |
| 1 | `b186290382e` | Pre-Vista prune E2: drop NTVDM/MVDM 16-bit subsystem |
| 1 | `9341c72da54` | freeldr: remove empty disk/disk.c TU |
| 1 | `6a9ba271daf` | freeldr: prune never-implemented GUI UI stub (ui/gui.c + gui.h) |

\* **`c474d932eb7` is not a true delete — it's a MOVE.** Both of its 2 "absent" paths
(`sdk/lib/3rdparty/libwine/CMakeLists.txt`, `sdk/lib/3rdparty/libwine/debug.c`) are live today
at `sdk/lib/libwine/CMakeLists.txt` and `sdk/lib/libwine/debug.c` (verified: both exist).
Default bucket-B action ("stay deleted") is **wrong** for these two — upstream's changes to
the old path should be re-homed to the new path, not dropped.

Full per-file lists for every commit above are reproducible with the script in the "How this
was derived" section (swap in each `%h`); omitted here since the commit-subject-plus-count
table already answers "why is this gone."

## Bucket A — new upstream work, by top-level directory (606 files)

| Directory | Count |
|---|---:|
| `dll/win32` | 149 |
| `modules/rostests` | 118 |
| `base/applications` | 60 |
| `sdk/include` | 49 |
| `drivers/storage` | 47 |
| `sdk/lib` | 40 |
| `media/themes` | 33 |
| `drivers/wdm` | 24 |
| `modules/wallpapers` | 9 |
| `base/system` | 9 |
| `boot/freeldr` | 8 |
| `dll/shellext` | 7 |
| `modules/rosapps` | 6 |
| `win32ss/user` | 5 |
| `sdk/tools` | 4 |
| `drivers/base` | 4 |
| `base/services` | 4 |
| `hal/halx86` | 3 |
| `dll/branding` | 3 |
| `base/shell` | 3 |
| `base/ctf` | 3 |
| `drivers/bus` | 2 |
| `dll/cpl` | 2 |
| `dll/appcompat` | 2 |
| `base/setup` | 2 |
| everything else (`ntoskrnl/*`, `media/nls`, `drivers/network`, `dll/ntdll`, `dll/3rdparty`, `boot/bootdata`, `.github`, `CMakePresets.json`) | 1 each |

**Surprise worth flagging:** `drivers/storage/ide/` (47 files) is a wholesale new IDE/SATA
stack upstream added in parallel with the old one — `atapi/` (AHCI-capable, `satl.c`,
`dev_*.c` split files) and `pciidex/chipset/*` (per-vendor AHCI/PATA chipset handlers: amd,
ati, intel, sil680, via, toshiba, svw_pata/sata...). This lines up with bucket C's rename hit
`drivers/storage/ide/atapi/license.txt` -> `drivers/storage/ide/atapi_old/license.txt`
(R100): upstream renamed their *old* atapi implementation to `atapi_old` and put the new
AHCI-era one at `atapi/` — the same "flip and keep the old one around" pattern this repo uses
for cohab, just upstream doing it for a driver rewrite instead of a Rust translation. This is
by far the largest coherent new-feature block in bucket A and deserves a deliberate look
rather than blanket accept, purely because of its size and because it will collide with
whatever exists at `drivers/storage/ide/atapi_old/` in our tree today.

Also notable: `ntoskrnl/ke/x86x64/cpuinfo.c` + `ntoskrnl/include/internal/x86x64/ke.h` +
`ntoskrnl/ntdll_nt6.S` suggest upstream is unifying x86/amd64 under a shared `x86x64` tree —
worth a look since it touches the kernel core this project has heavily Rust-translated.

## Bucket C — rename on one side, counterpart already in our tree (40 files, full list)

Format: `<old path in our tree>` -> `<new path upstream uses>` (similarity).

| Old (present in our tree) | New (upstream name, absent) | Similarity |
|---|---|---:|
| `boot/bootdata/autorun-livecd.inf` | `boot/bootdata/autorun.inf` | R050 |
| `dll/win32/msgina/resources/reactos.bmp` | `dll/branding/rosbrand/resources/bmp/brand_banner.bmp` | R100 |
| `dll/win32/msgina/resources/line.bmp` | `dll/branding/rosbrand/resources/bmp/brand_bannerline.bmp` | R100 |
| `dll/win32/msgina/resources/svg/reactos.svg` | `dll/branding/rosbrand/resources/svg/brand_banner.svg` | R100 |
| `dll/win32/msgina/resources/svg/line.svg` | `dll/branding/rosbrand/resources/svg/brand_bannerline.svg` | R100 |
| `dll/win32/avifil32/rsrc.rc` | `dll/win32/avifil32/avifil32.rc` | R097 |
| `dll/win32/ole32/rpc.c` | `dll/win32/combase/rpc.c` | R073 |
| `dll/win32/ole32/stubmanager.c` | `dll/win32/combase/stubmanager.c` | R079 |
| `dll/win32/ole32/memlockbytes.c` | `dll/win32/coml2/memlockbytes.c` | R092 |
| `dll/win32/cryptdll/cryptdll.c` | `dll/win32/winusb/main.c` | R063 |
| `drivers/storage/ide/atapi/license.txt` | `drivers/storage/ide/atapi_old/license.txt` | R100 |
| `modules/rostests/kmtests/ntos_ob/ObType.c` | `modules/rostests/kmtests/ntos_ob/ObType.cpp` | R082 |
| `modules/rostests/kmtests/ntos_ob/ObTypes.c` | `modules/rostests/kmtests/ntos_ob/ObTypes.cpp` | R072 |
| `modules/rostests/winetests/cryptui/testlist.c` | `modules/rostests/winetests/combase/testlist.c` | R055 |
| `modules/wallpapers/Angelus/ReactOS/Angelus ReactOS Dark 16x9 1067x600.jpg` | `modules/wallpapers/Backwoods.jpg` | R100 |
| `modules/wallpapers/Mizu/ReactOS Mizu 4x3 1440x1080.png` | `modules/wallpapers/Mizu (4x3).png` | R100 |
| `modules/wallpapers/Mizu/ReactOS Mizu B 16x9 1920x1080.png` | `modules/wallpapers/Mizu.png` | R100 |
| `modules/wallpapers/ReactOS/ReactOS Nebula 1920x1080.jpg` | `modules/wallpapers/Nebula.jpg` | R100 |
| `sdk/lib/3rdparty/freetype/src/base/ftdebug.c` | `sdk/lib/3rdparty/freetype/builds/windows/ftdebug.c` | R091 |
| `sdk/lib/3rdparty/freetype/src/otvalid/otvmod.h` | `sdk/lib/3rdparty/freetype/src/sfnt/sfwoff.h` | R061 |
| `sdk/lib/3rdparty/freetype/src/otvalid/otvgpos.h` | `sdk/lib/3rdparty/freetype/src/sfnt/woff2tags.h` | R054 |
| `dll/win32/comcat/comcat_main.c` | `sdk/lib/3rdparty/libwine/dll_canunload.c` | R061 |
| `sdk/lib/crt/printf/vswprintf.c` | `sdk/lib/crt/printf/_vswprintf.c` | R091 |
| `sdk/lib/vcruntime/__acrt_initialize_stub.cpp` | `sdk/lib/vcstartup/__acrt_initialize_stub.cpp` | R100 |
| `sdk/lib/vcruntime/__report_gsfailure.c` | `sdk/lib/vcstartup/__report_gsfailure.c` | R100 |
| `sdk/lib/vcruntime/__report_rangecheckfailure.c` | `sdk/lib/vcstartup/__report_rangecheckfailure.c` | R100 |
| `sdk/lib/vcruntime/__scrt_uninitialize_crt.cpp` | `sdk/lib/vcstartup/__scrt_uninitialize_crt.cpp` | R100 |
| `sdk/lib/vcruntime/__security_init_cookie.c` | `sdk/lib/vcstartup/__security_init_cookie.c` | R100 |
| `sdk/lib/vcruntime/__vcrt_init_stubs.c` | `sdk/lib/vcstartup/__vcrt_init_stubs.c` | R100 |
| `sdk/lib/vcruntime/_fltused.c` | `sdk/lib/vcstartup/_fltused.c` | R100 |
| `sdk/lib/vcruntime/_onexit.c` | `sdk/lib/vcstartup/_onexit.c` | R100 |
| `sdk/lib/vcruntime/amd64/__security_check_cookie.s` | `sdk/lib/vcstartup/amd64/__security_check_cookie.s` | R100 |
| `sdk/lib/vcruntime/atexit.c` | `sdk/lib/vcstartup/atexit.c` | R100 |
| `sdk/lib/vcruntime/commonCRTStartup.hpp` | `sdk/lib/vcstartup/commonCRTStartup.hpp` | R100 |
| `sdk/lib/vcruntime/i386/__security_check_cookie.s` | `sdk/lib/vcstartup/i386/__security_check_cookie.s` | R100 |
| `sdk/lib/vcruntime/initializers.cpp` | `sdk/lib/vcstartup/initializers.cpp` | R100 |
| `sdk/lib/vcruntime/isa_available.cpp` | `sdk/lib/vcstartup/isa_available.cpp` | R100 |
| `sdk/lib/vcruntime/mainCRTStartup.cpp` | `sdk/lib/vcstartup/mainCRTStartup.cpp` | R100 |
| `sdk/lib/vcruntime/tlssup.c` | `sdk/lib/vcstartup/tlssup.c` | R100 |
| `sdk/lib/vcruntime/wmainCRTStartup.cpp` | `sdk/lib/vcstartup/wmainCRTStartup.cpp` | R100 |

Reading this cluster: upstream renamed `sdk/lib/vcruntime/` -> `sdk/lib/vcstartup/` wholesale
(18 of the 40, all R100 — a pure directory rename), renamed `dll/win32/msgina/resources/*` ->
`dll/branding/rosbrand/resources/*` (4 files, R100 — branding got its own DLL), and renamed
`modules/wallpapers/<Pack>/<Long Name>.{jpg,png}` -> `modules/wallpapers/<Short Name>.{jpg,png}`
(4 files, R100 — wallpaper pack flattening). The rest are lower-confidence single-file matches
(ole32 split into combase/coml2, cryptdll refactored into winusb, freetype internal reshuffle)
that should be spot-checked rather than trusted blindly, since below ~R080 git's guess gets
speculative.

## Recommended merge posture

- **Bucket A (606, blanket rule OK):** accept upstream's file as-is for the vast majority.
  Carve out **`drivers/storage/ide/`** (47 files, the new AHCI/pciidex stack) for a deliberate
  look before merging — it's large, it's a whole new driver architecture, and it directly
  collides with bucket C's `atapi/license.txt` -> `atapi_old/license.txt` rename.
- **Bucket B1 (63, needs a human/agent pass, not a blanket rule):** for each of the 63 paths,
  diff what upstream changed against the Rust that replaced it
  (`win32ss/user/*` -> win32k-rust; the tcpip.sys/lan driver -> netstack/ws2_32/afd/iphlpapi)
  and decide per-change whether it's a real bug fix to port. This is the valuable set the
  owner asked to eyeball first.
- **Bucket B2 (247, blanket rule OK — stay deleted)** with **one required exception**: the 2
  files under `c474d932eb7` are a MOVE, not a delete — route upstream's changes to
  `sdk/lib/libwine/{CMakeLists.txt,debug.c}` instead of discarding them.
- **Bucket C (40, needs a quick per-file decision, not fully blanket):** the three coherent
  clusters (vcruntime->vcstartup, msgina->rosbrand branding, wallpaper flattening — 26 files,
  all R100) can be handled by a small scripted path-remap rule with high confidence. The
  remaining ~14 lower-similarity matches (R050–R097) should be manually resolved — accept
  upstream's version at the new path, or treat as independent since the "rename" is only a
  content-similarity guess.
