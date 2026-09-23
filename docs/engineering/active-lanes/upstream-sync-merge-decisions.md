# Upstream-sync: what the big merge decided, and what it left owed

Written 2026-09-20 by the upstream-sync lane, at the moment of the merge. Companion to
`upstream-sync-plan.md` (strategy), `upstream-sync-merge-recipes.md` (the per-file recipes
written before the merge) and `upstream-sync-phase4-review-queue.md` (the 194 refreshed
flipped TUs, which is the *output* of the merge rather than a record of it).

**Everything below is a decision someone made on purpose.** Where a decision was declined
rather than taken, there is a comment at the point of decline in the tree itself, so the next
reader does not "fix" it.

---

## The three silent hazard classes

A conflict is the easy case: git stops and someone reads it. The merge's real risk was in the
changes that arrived with no marker at all, and all three classes fired:

1. **Silent deletes.** Upstream deleted 281 files under `sdk/lib/crt` and they came through as
   a clean auto-merge. Our crt is live — `libcntpr` and every driver link it. Restored all 281.
2. **Silent adds.** Deletions inside a tree we pruned turn into modify/delete conflicts and get
   reviewed; *additions* into that same tree arrive clean. `hal/halx86/pc98` and
   `boot/freeldr/freeldr/arch/realmode` came straight back. So did
   `drivers/storage/ide/atapi_old/`, carrying the Microsoft NT4 DDK EULA we deleted the day
   before and a duplicate `atapi` cmake target.
3. **Silent renames.** `sdk/lib/vcruntime/*` → `sdk/lib/vcstartup/*` moved 18 files and left our
   `vcruntime/CMakeLists.txt` replaced by upstream's, which defines only the `vcruntime` target;
   `dll/win32/ucrtbase` links `vcstartup`, so without `add_subdirectory(vcstartup)` cmake fails
   at configure. The split is coherent and was taken — but the auto-merge also silently dropped
   our `-Wno-builtin-declaration-mismatch` / `-Wno-unused-function` block, restored by hand.

The generic check that caught (2) and (3), worth re-running on any future sync:

```bash
# every source a CMakeLists names, that this merge removed or renamed away
git diff HEAD --name-status --diff-filter=DR | awk '{print $2}' | sort -u > /tmp/gone.txt
# ...then walk every CMakeLists/*.cmake and resolve each listed .c/.cpp/.S against it
# (and the same for .rc resource references)
```

Both came back clean at commit time.

## The census earned its keep

`tools/cohab_census.py` fired **twice in the dangerous direction**, and both arrived by clean
auto-merge with nothing to review:

| | verdict |
|---|---|
| `dll/3rdparty/libtiff/CMakeLists.txt: #tif_jpeg.c` | Safe. Upstream's 4.7.1 drop enables JPEG-in-TIFF and links libjpeg. No Rust twin. |
| `dll/win32/comdlg32/CMakeLists.txt: #itemdlg.c /* Win 7 */` | Safe. Upstream implemented the Win7 IFileDialog. comdlg32 is pure C here — no `comdlg32-rust` exists. |

Review scales to the 8 conflicting build files. Only a census scales to the 80 exposed ones,
and both of these were in the 72 that never conflicted.

## Whole upstream changes DECLINED, and why

| What | Why |
|---|---|
| The all-in-one BootCD (`b2e33f26eb3`) | Reduces `livecd.iso` to a minimal **non-bootable** image. That is the image all three smoke arches boot. Adopting it is a project, not a sync. |
| The CRT restructure (`de7e7074592` + follow-ups) | 281 live files. `crtheap`/`llvm-compat`/`ansi_sync_hacks` landed on disk but are not wired up. |
| `dll/win32/kernelbase` → wine-based `kernelbase_ros` | **The biggest judgment call in the merge.** Ours is a deliberately minimal Vista-scoped shim, documented as such in its own header. See "owed" below. |
| ole32 → `combase`/`coml2`/`pathcch_static` split | Same reasoning; ole32 is Rust-flipped here. |
| libjpeg 9f → 10 | A major API break, and the research lane owns vendor bumps. libpng and mbedTLS were declined because **ours are newer** (1.6.58 vs 1.6.54; 3.6.7 vs 2.16.11). libtiff 4.7.0 → 4.7.1 was taken. |
| pruned trees | pc98, xbox, arm32, BIOS realmode, ext2/btrfs/ntfs/udfs/nfs, NTVDM, ISAPNP, fast486, SAC, x86bios/INT10, legacy tcpip, floppy, fraginator, `untfs`. |

## New upstream TUs declined because a Rust twin owns the symbols

Each has a note at the point of decline. This is the landmine in its subtlest form — several of
these would have entered as an innocuous new source line or, worse, a new static library.

- kernel32: 14 `*ansi.c` splits (`atomansi`, `pathansi`, `procansi`, `synchansi`, `sysinfoansi`,
  `versionansi`, `copyansi`, `diransi`, `fileinfoansi`, `moveansi`, `mntpointansi`, `volumeansi`,
  `localeansi`, `lcformatansi`) — `kernel32-rust` exports both the A and W entry points.
- advapi32: upstream's new `win32_registry` static lib, built from `reg/hkcr.c` + `reg/reg.c`.
  **Both Rust-owned.** Live C beside a Rust twin, arriving as a new library target.
- ws2_32: `src/wine.c` duplicates `inet_addr`, owned by `addrconv.rs`.
- oleaut32: `thunks.c` (asm `DispCallFunc` trampolines) — see "owed".
- ntoskrnl: `ke/x86x64/cpuinfo.c` (both `cpu.c` are Rust).
- rosload: `ntoskrnl/ke/config.c` (cmboot-rust covers it) and `ntldr/advopts.c` (extracted from
  `ntldropts.c` + `options.c`, both Rust).
- syssetup: `addons.c` — its only caller, `install.c`, is Rust.
- win32ss: `gdi/ntgdi/brush.cpp`.
- rtl: `processor.c` — see "owed", this one leaves a real gap.

## Taken live

`kernel32/client/dep.c` (the spec already exported `Get`/`SetProcessDEPPolicy` and
`GetSystemDEPPolicy` with **nothing implementing them**), `sdk/lib/rtl/memres.c`,
`kernel32_vista/threadpool.c`, `explorer/mediabtns.cpp`, `ntdll/compat/amd64/msys2.S`,
`sdk/lib/rtl/rangelist.c`'s new `RtlGetLastRange`, upstream's new `uiautomationcore`, and
`sdk/lib/libwine/dll_canunload.c` — which our own `libwine/CMakeLists.txt` already referenced
and which the libwine move commit had left untracked.

`sdk/include/vcruntime/mingw32/intrin_arm64.h`: applied the owner's ruling. Upstream's 845-line
header is the base (114 symbols vs our 51; all 43 overlaps signature-compatible), plus the six
ours-only intrinsics that have users — `_disable` (75), `__debugbreak` (73), `_enable` (57),
`_mm_mfence` (6), `_mm_lfence` (5), `_mm_sfence` (4). `__readx18` had 0 users and upstream ships
its own `__readx18`/`__writex18` family.

---

## Agent B — drivers (20 build files)

- **Upstream has built a whole AHCI/SATA stack since the fork, and we have zero coverage.**
  Two independent pieces landed in this merge and both are unbuilt on disk:
  - `drivers/storage/ide/atapi/` — a new SATL/port driver (fdo, pdo, satl, scsi, enum,
    dev_config/error/identify/power/timings, smart, wmi, portstate, ioctl). Our `atapi-rust`
    is a translation of the *old* NT4-DDK single-TU miniport, which upstream has retired.
  - `drivers/storage/ide/pciidex/chipset/` — a 16-file chipset quirks framework
    (intel/amd/ati/via/sil680/toshiba/cmd/pctech/svw_pata/svw_sata/pata_*) **plus a full
    software AHCI implementation** (ahci_generic/ahci_hw/ahci_io), and a new `acpi.c`.
    Our `pciidex-rust` came from the pre-fork fdo.c/pciidex.c.
  Likely the single biggest concrete translation opportunity this merge surfaced.
- **pcix**: upstream added `pci/ecam.c` + `pci/express.c` (PCIe ECAM config space). No Rust
  twin; left out of the build. Real feature gap, not a bug.
- `drivers/base/CMakeLists.txt`: our GDB-flag mutual exclusion between kdcom and kdgdb is
  mandatory — both build a cargo target named `kdcom`, so upstream's parallel ARCH-based
  `add_subdirectory` would have collided.
- Folded in from upstream: `add_subdirectory(kdstub)`, pciide's `ENABLE_UNIATA` option gate
  and `pciide_reg.inf`. Declined: atapi's registry inf (configures *their* driver), and the
  `-DDEBUG_MODE` / `_WIN32_WINNT` defines upstream re-added to HID/USB drivers whose C is
  fully flipped — our own cohab commits had removed them deliberately.

## Agent A — kernel / sdk / win32ss (19 build files)

- **`RtlIsProcessorFeaturePresent` and `RtlGetCurrentProcessorNumberEx` have no
  implementation anywhere in the tree, before or after this merge.** All three of
  `ntdll.spec`, `ntdll_vista.spec` and `kernelbase.spec` export them. Upstream's new
  `sdk/lib/rtl/processor.c` is the only definition that exists — but it *also* redefines
  `RtlGetCurrentProcessorNumber`, which Rust already owns
  (`sdk/lib/rust/rtl-strings/src/cabi/mod.rs:7681`), so the whole file was declined rather
  than risk a duplicate symbol. Fix: hand-port the two missing functions into `rtl-strings`,
  or split `processor.c`. A real gap, not a regression from this merge.
- **`win32ss/user/ntuser/job.c` and `user/ntuser/rawinput.c` are new upstream TUs** with no
  entry in our SOURCE list (not even commented) and no Rust twin. Left out entirely, since
  win32k is Rust-only. `job.c` is the win32k half of upstream's new job-object work — it pairs
  with `ntoskrnl/ps/job.c`. Raw input is a genuine functionality gap.
- Added live: `sdk/lib/rtl/memres.c` (`Rtl{Cm,Io}{De,En}codeMemIoResource`,
  `RtlFindClosestEncodableLength`) — no Rust twin, no collision, consumed by `drivers/bus/pcix`.
- `sdk/cmake/gcc.cmake`: kept our llvm-mingw libc++/libcxxabi/libunwind path over upstream's
  `libgcc_eh` addition. Flagged rather than guessed — unclear whether it matters for a
  non-llvm-mingw GCC build.
- Declined: `gdi/ntgdi/brush.cpp` (Rust owns it), the arm32 branch in `except.cmake`, and
  upstream's 5-way split of `arbiter.c` (already 100% translated).

## Agent C — dll / base / boot (25 files)

- **`dll/win32/kernelbase` is the biggest judgment call in the merge.** Ours is a small
  hand-written Vista-scoped shim, documented as deliberate in its own header. Upstream has
  restructured kernelbase into a much larger wine-based `kernelbase_ros.dll`, matching the
  `combase`/`coml2`/`shcore` split that also landed here. Kept ours. **If the new DLLs expect
  a fuller kernelbase, they will not be satisfied by it** — needs a second look.
- `dll/win32/ole32`: declined upstream's `combase`/`coml2`/`pathcch_static` restructure for the
  same reason. Confirm the new DLLs do not expect ole32 to import them.
- `dll/win32/oleaut32`: declined upstream's new `thunks.c` (asm `call_method` trampolines for
  `DispCallFunc`) on the reasoning that `dispatch.rs` replaces it — **inferred, not verified.**
  An arbitrary-arity native call trampoline is usually exactly what stable Rust cannot emit, so
  check `dispatch.rs` actually handles it.
- `dll/win32/ws2_32`: declined upstream's new `src/wine.c` — it duplicates `inet_addr`, which
  `addrconv.rs` owns.
- `dll/win32/advapi32`: declined upstream's new `win32_registry` static lib built from
  `reg/hkcr.c` + `reg/reg.c` — both Rust-owned. That one would have put live C beside a Rust
  twin, i.e. the landmine, arriving as an innocuous-looking new static library.
- Added live: explorer's `mediabtns.cpp`, kernel32_vista's `threadpool.c`, ntdll's
  `compat/amd64/msys2.S`, and upstream's new `uiautomationcore`.
- Declined per pruned-tree policy: `untfs`, the bootsect BIOS revival, rosapps `fraginator`,
  ntdll's arm32 branch, and upstream's ReactOS branding block in `README.md`.

## Agent D — live source (30 files)

- **`ntoskrnl/io/iomgr/iomgr.c` near-miss.** Our ARM64_UART_TRACE hunk declares
  `IopTraceLevel` and `PnpSystemInit`, and the already-auto-merged common text a few lines
  below declares them again. A blind "keep ours" here is a duplicate-symbol compile error.
  Worth telling whoever wrote that macro.
- **`drivers/bus/acpi/osl.c` was MERGED, not picked.** Upstream's fast path checks the
  freeldr-synthesized `AcpiLoaderRsdp` (which we already integrate); our legacy BIOS scan +
  registry-fallback synthesis now runs behind it. Needs a boot check that the fast path does
  not skip something the BIOS scan supplied.
- `ntoskrnl/kd/i386/kdserial.c`: kept our local `BaseArray` rather than upstream's shared
  `cportlib/uartinfo.h` — ours carries the `_M_ARM64` PL011 QEMU branch their header lacks.
  Everywhere else in the serial/debug cluster the shared header was adopted.
- `sdk/include/ndk/arm64/ketypes.h`: kept our real `KeGetCurrentPrcb`; upstream's is
  `UNIMPLEMENTED`.
- `sdk/lib/rtl/rangelist.c`: upstream added `RtlGetLastRange`, which is already in the spec and
  in `rtlfuncs.h` and had no implementation on either side. Added.
- **kernel32_vista.spec, uncertain:** kept our locally-implemented
  `TryAcquireSRWLockExclusive`/`TryAcquireSRWLockShared` over upstream's forward-to-ntdll
  (upstream drops the Shared one entirely). Flagged per "cannot tell, keep ours".
- **msvcrt.spec:** upstream gates `swprintf_s`/`vswprintf_s`/`vsprintf_s` behind
  `-version=0x600+`. Relay to the msvcrt-homebrew lane.

### SHELL32 ORACLE — for the shell32 lane

- New COM objects `CShellLibrary` and `CUserEventTimer`, wired into the object map. Not
  Rust-cohabbed yet.
- New `SHELL_QueryInfoTipAlloc`, and a new `SHELL_GetDetailsOfAsStringVariant(IShellFolder*)`
  overload. No Rust implementation. Currently unreferenced by a live caller, so it is a silent
  gap rather than a broken build. **Note the linkage trap found here:** declaring that overload
  `EXTERN_C` collides with the unmangled Rust-exported `IShellFolder2*` overload of the same
  name — it must stay plain C++ linkage.
- New exports `LinkWindow_RegisterClass` / `LinkWindow_UnregisterClass` (SHELL32.258/259) —
  ordinals were already in the spec with nothing behind them; implemented in `utils.cpp` as
  plain C++.

---

---

## Making it build: twenty rounds, nine shapes

The merge committing is not the merge working. Every failure below was the same shape —
**upstream restructured something, half of it auto-merged, and the other half was ours.**
Recorded because the next sync will produce the same nine.

| # | Shape | Instances |
|---|---|---|
| 1 | **A macro changed signature, we kept the caller.** | `add_link(NAME … PATH …)` became `add_link(name path …)`; only one call site, in `boot/bootdata`, and cmake reported it as a duplicate custom rule on `NAME.lnk`. |
| 2 | **A source list took theirs while the sources stayed ours.** | `sdk/lib/crt/CMakeLists.txt` merged to upstream's, dropping `include()` of conio, direct, mbstring, process, stdio, time, wine and `crt.cmake` — after I had restored all 281 source files. Restoring the tree wholesale was the only coherent state. Same for `minihal`, which lost `add_asm_files(... systimer.S)` and left a target with no compilable source at all (*"CMake can not determine linker language"*). |
| 3 | **A tree-wide define disappeared.** | Upstream's "Stop using non-conforming swprintf / vswprintf" removed `-D_CRT_NON_CONFORMING_SWPRINTFS` from the top-level `CMakeLists.txt` and fixed every call site. **The same commit also dropped the `__cplusplus` arm of the guard in `<stdio.h>`**, so the 2-arg `swprintf` vanished for C++ too — which is what broke hdaudbus. Resolution: take the change, and scope the old declaration to the two TUs that *implement* the count-less form (`printf/swprintf.c`, `printf/vswprintf.c`) with `set_source_files_properties`. |
| 4 | **A header moved to `target_include_directories(INTERFACE)`.** | `evtlib.h`, `arbiter.h`, `infros.h`, `skiplist.h`, `concfg.h`. Upstream deleted each consumer's own `include_directories` line in the same commit. The consumers merged; the publishers did not, because ours are Rust-cohab rewrites. Five separate "file not found" failures with one cause. |
| 5 | **A declined restructure had a piece we actually needed.** | `crtheap` (six heap shims that upstream's new shcore and the auto-merged shlwapi link) and `vcstartup` (`ucrtbase` links it, and cmake fails at *configure* without it). Both live inside the CRT restructure I declined; both are wired up, with the reason written at the point of decision. |
| 6 | **CMake de-duplicated a repeated flag.** | Upstream added a second `-include clsid.h` to urlmon. CMake collapses repeated `COMPILE_OPTIONS`, so the flag was dropped and its path became a second *input* file: `clang: error: cannot specify -o when generating multiple output files`, on **every TU in the module**. |
| 7 | **Vendored trees went Frankenstein.** | Conflicts resolved to ours, clean hunks took theirs. libjpeg ended up with v10 sources against 9f headers (`JPEG_DATA_PRECISION` undeclared); mbedTLS got 2.x files in a 3.6.7 tree; libtiff 4.7.1 turned `JPEG_SUPPORT` on against a libjpeg whose `.spec` exports none of the `jpeg_*` surface. **A vendored tree has to be coherent or it is neither version.** |
| 8 | **A flipped module's `.spec` gained exports the Rust does not implement.** | ~230 of them. The `.spec` drives the `.def`, so the export goes live the moment the merge lands and the link fails with `<root>: undefined symbol`. A pre-flight for this is in the next section. |
| 9 | **A declined restructure arrived anyway through a third door.** | `combase` came in via `dll/win32/CMakeLists.txt` even though I had declined the COM split at ole32, kernelbase and shlwapi. It externs `Ole32DllGetClassObject` and `DestroyRunningObjectTable` from ole32 — names that only exist *after* the split — and our ole32 moniker/oleproxy are Rust, which does not export them. **I nearly papered over it by adding two lines to `ole32.spec`: it would have linked and then misbehaved.** Declined consistently instead: combase, coml2, shcore and uiautomationcore are all commented out in `dll/win32/CMakeLists.txt`. |

Two more, arch-specific, both worth knowing:

- **i386 only.** `@ stub NAME` does not carry an argument size, so on i386 it produces an
  undecorated export and every stdcall caller misses `_NAME@n`. Stubbing an export is not a
  neutral act there. That is what forced `NhGetGuidFromInterfaceName` to be *implemented*.
- **arm64 only.** `advpack/reg.c`, `bluetoothapis/main.c` and `wintrust/{softpub,crypt}.c`
  call `malloc`/`free` with no `<stdlib.h>`; it arrives through another header on i386 and
  amd64 but not on aarch64. Four files, one `#include` each.

### What we implemented rather than deferred

Three functions, all cases of upstream adding something to a file we had translated:

- **`MprConfigServerConnect`** (`mprapi-rust`) — arrived as a *link error*, not a diff: the
  spec drives the `.def`, so the export went live the instant the merge landed.
- **`GetThreadDescription`** (`kernel32-vista-rust`) — upstream moved `SetThreadDescription`
  into a new `ThreadDescription.c` and added its counterpart. Building their C would have
  silently taken their `Set` over our Rust one, so it is ported instead. The retry loop is
  load-bearing: a concurrent `SetThreadDescription` can grow the name between the sizing
  query and the fetch.
- **`CompareStringOrdinal`** (`kernel32-vista-rust`) — upstream calls
  `RtlCompareUnicodeStrings`, which is `-version=0x600+` in `ntdll.spec` while we build ntdll
  at 0x502, so the importlib does not carry it. Comparison inlined rather than widening
  ntdll's export surface.
- **`NhGetGuidFromInterfaceName`** (`iphlpapi-rust`) — see the i386 note above. Upstream's
  `netsh interface ip` build-out (`ifmon/ip.c`, +2409 lines) depends on it.

### Pre-flights worth keeping

Both are cheap and both found real things:

```bash
# 1. sources a CMakeLists still names that the merge deleted or renamed away
git diff HEAD^1 HEAD --name-status --diff-filter=DR | awk '{print $2}' | sort -u > /tmp/gone
#    ...resolve every .c/.cpp/.S in every CMakeLists/*.cmake against it. Also .rc resources.
#    Came back clean at commit time: the rename hazard never fired.

# 2. added .spec exports with no implementation on our side
#    for each changed *.spec: parse '+@ stdcall|cdecl|...' lines that are NOT `stub` and NOT
#    forwarded, then grep the module's own sources and its *-rust crate for each name.
```

The second one is the higher-value of the two and wants to become a real tool — it predicted
the netcfgx, ole32, shlwapi, advapi32 and ntdll_vista link failures before they happened.
Note its two false-positive classes: `@ stub` entries (no implementation is the point) and
forwarded entries (`... ) otherdll.Name`).

### One process note

`tools/cohab_census.py --write` was run **mid-merge**, before the build was fixed, which
recorded upstream's transient state as the baseline. Two entries then read as REMOVED
(`#float/fpecode.c`, `#string/is_wctype.c`) purely because restoring `sdk/lib/crt` put them
back to their pre-merge configuration. Verified against `HEAD^1:tools/cohab-census.txt`,
which has neither. **Only `--write` a state someone has decided on.**

Unrelated observation while checking those two: `__fpecode` has a Rust twin in
`msvcrt-crt-rust/src/msvcrt/math.rs` *and* `float/fpecode.c` is live in our crt. That is the
pre-merge status quo, not something this merge changed, and I have not verified which one
actually wins at link — recorded as something to look at, not as a finding.

## What the lint sweep added to the record

Four scanners had to be answered after the build went green. Two produced findings worth
more than the warning they raised.

- **`verify_translation_literals` is an oracle, not a lint.** It compares the literals in a
  translated `.rs` against the `.c` it came from, so refreshing the dead C beside 194 Rust
  twins lit up every literal upstream had changed. All 77 traced to a named upstream commit;
  four of them are behaviour changes our Rust does not have, and are now in the phase-4
  queue. Baselined (169 total, +77/−0, verified by SET comparison across exactly the seven
  expected files — no foreign debt).
- **`marshalgen_audit` was a tool bug, not tree debt.** `extract_flags` matched any token
  starting `-I`/`-isystem`, but cmake emits `-isystem <path>` as two tokens — so it kept the
  flag and dropped the path. The merge changed one module's spelling and the audit silently
  lost `sdk/lib/pseh/include`, i.e. `<pseh/pseh2.h>`, i.e. every layout. Fixed in the tool.
  A wrong checker that gets baselined is worse than no checker.
- **`vendored_registry`**: upstream's declined COM/shell modules (combase, coml2, shcore,
  uiautomationcore) are Wine/CodeWeavers LGPL code, filed under `vendor.yaml`'s `ours:` half
  rather than `trees:` — **unbuilt is not a reason to shrink the translation denominator.**
  Their winetests likewise. One genuine upstream misfiling found: `AppleGraphInfo.h` is a new
  **ReactOS-authored MIT** header (PR #8706, freeldr UEFI Apple graphics) that upstream
  dropped into the vendored `sdk/include/reactos/edk2/` directory, where a directory-wide
  vendored classification does not fit it. Recorded in `license_extra:` with the reason;
  relocating it is a follow-up for whoever owns freeldr's include paths.
- **`shell32_tail_inventory`**: `CShellLibrary.cpp` and `CUserEventTimer.cpp` are new upstream
  ATL COM leaf classes with all-stub bodies. Classified in `tools/rustconv/data/shell32_cpp_tail.json`
  as `active.untranslated_leaf` — a ledger entry, not a baseline suppression.

And one thing confirmed rather than assumed: **the NT4 DDK EULA has not come back.** The only
hits for it in the merged tree are the comments documenting its deletion.

## The stale-bindgen problem, which the merge turned from theory into a hazard

Upstream fixed several long-standing signature bugs in `ntoskrnl/include/internal/{ps,se,cm,io,lpc,cc}.h`
during this merge — added missing `KPROCESSOR_MODE AccessMode` parameters, widened handle-count
parameters from `ULONG` to `ULONG_PTR`, and changed one `VOID` return to `BOOLEAN` so a caller
can detect a race. Our Rust still carried the old, pre-fix ABI. Those are exactly the bugs this
lane exists to find, and they were all fixed.

But the checked-in bindgen (`ntkrnl-sys/src/bindgen_{i386,amd64,arm64}.rs`) still describes the
OLD shapes and has not been regenerated. So fixing a callee without fixing its callers makes
things WORSE, not better:

> `extern "system"` is **stdcall on i386, where the callee pops the arguments.** A
> nine-parameter callee invoked through an eight-parameter function pointer pops four bytes
> more than the caller pushed. Every object-security callout would corrupt the stack. On
> amd64/arm64 the same mismatch is "only" a garbage register read.

`SeDefaultObjectMethod` hit exactly this: the callee was corrected to nine parameters while all
seven Rust call sites in `ob-rust` still passed eight through the stale `OB_SECURITY_METHOD`
typedef. Every call now goes through **one** helper, `ObpCallSecurityProcedure` in
`ob-rust/src/obsecure.rs`, which transmutes to the real signature and supplies
`ExGetPreviousMode()` — one place to fix when the typedef catches up. The same
transmute-at-the-boundary pattern (precedent: fltmgr's `CloseProcedure`) covers
`OB_CLOSE_METHOD` and netcfgx's `INetCfgComponentPrivate::Unknown1`.

Two more that were caught the same way: `PspRemoveProcessFromJob` and `PspExitProcessFromJob`
lost a parameter upstream, and `ps-rust`'s `kill.rs` was still resolving them through the stale
two-parameter bindgen externs — now imported explicitly from the crate so local definitions win
name resolution.

**Regenerating the bindgen ddk surface is owed debt across the whole tree, not just these
symbols.** Until then, the rule is: when a callee's signature is corrected, fix every call site
in the same change, and prefer one shared helper over scattered transmutes.

The user-mode islands were in the same position. **shell32-rust's island is now regenerated
(2026-09-22) and AUDITED** — the method, so the next crate can reuse it:

1. Regenerate into scratch, not the tree: `OUT_CRATE_DIR=<scratch> bash
   tools/rustconv/gen_driver_types.sh <drv>`, with the SAME bindgen that built the committed
   island (read it off the file's banner — install that version side by side and put it first
   on PATH). Change one variable at a time or the diff mixes header drift with tool drift.
2. Diff STRUCTURALLY, not textually: records/aliases/consts that were added (harmless),
   removed (compile errors — benign), or changed; resolve each changed ALIAS to the record it
   names, because widl's anonymous `__WIDL_*_generated_name_000000NN` types were RENUMBERED
   by the merge (inserted anonymous types shift every later number). All 7 changed aliases
   shell32 uses resolved to identical layouts on all 3 arches — renumbering only.
3. For every changed/removed name, ask whether a crate USES it outside the island.

What it found in shell32: no live layout drift (the changed C++-class renderings are unused —
each Rust port owns its layout), but one LIVE regression the island hid: upstream moved the
About banner into rosbrand.dll and removed IDB_REACTOS/IDB_LINEBAR from shell32, so the Rust
About dialog was loading resources that no longer exist (fixed; arm64 media now stages
rosbrand.dll too). And one CONTRACT change: `SHBindToObject` became a 5-arg stdcall Vista
export upstream while the crate still defines the old 4-arg cdecl internal — unexported at
0x502 and with no live C/C++ caller, so declared by its real contract in the prelude; adopting
the new one is owed.

Four generator defects fell out, each of which would have bitten the next regeneration:
OUT_CRATE_DIR was scanned for the crate's own definitions and _extra.rs (both must read the
crate); amd64 lacked `-mcx16` that every real amd64 TU gets; `intrin_arm64.h` and
`interlocked.h` both defined `_InterlockedAdd64` under the same `!__has_builtin` guard; and the
prelude's declarations of crate-defined functions were hand-copied — now derived by
`tools/rustconv/own_exports_decls.py`, which HARD-FAILS when a crate definition's arity or
calling convention has diverged from the header (it caught SHBindToObject, and caught its own
first draft declaring cdecl functions as stdcall). Other crates' islands are still pre-merge.


## Five tool bugs, found because the merge moved the ground under the tests

The `check-tooling` suite pins fixtures to real upstream C — line numbers, vtable counts,
class field lists. The merge moved all of it, and re-deriving the fixtures turned up five
genuine defects in our own tooling. None of them was upstream's fault and none would have
been found by reading diffs.

| Tool | Bug | Consequence |
|---|---|---|
| `marshalgen.py` | `extract_flags` matched `-isystem` but not the **path token after it** | Lost `sdk/lib/pseh/include`, i.e. `<pseh/pseh2.h>`, i.e. every record layout. Surfaced as `SCANNER_ERROR`. |
| `verify_lost_exports.py` | commented-vs-live `.c` sets compared by raw captured path text, not basename | An explanatory comment mentioning `dep.c` cancelled the live `client/dep.c`, so three DEP exports read as lost when they are built. |
| `verify_lost_exports.py` | `_SPEC_ENTRY`'s `$`-anchored optional forwarder could not tolerate a trailing `# comment` | **The whole line failed to match and the symbol vanished from the audit with no diagnostic.** Fixing it immediately surfaced six genuinely-lost shell32 exports. |
| `spec_parser.py` | no handling of `-import` | spec2def ignores that flag and the LINKER resolves the export from an import library, so no local symbol is needed. Upstream rewrote oleaut32's Get/Set/CreateErrorInfo from `ole32.X` forwarders to `-import` and all three read as MISSING against a tree that links fine. |
| `cpp_com.py` | `_parse_param` stripped only classic SAL, never `_SAL_ANNOTATION` | Modern SAL survived into the parsed TYPE and `render_class_scaffold` emitted it verbatim: `pdo: *mut _In_ IDataObject`. Not Rust. |
| `cpp_com.py` | the "plausible declarator" test was anchored at the end of the **whole statement** | `LPWSTR m_PathTarget = NULL` ends in an identifier and survived; `DWORD m_grfKeyState = 0` ends in a digit and was **silently dropped from the object layout**. No error. Exactly the failure that file warns about everywhere else. |

`cpp_com.py` also learned a fourth internal-activation shape: upstream replaced
`new CComObject<T>()` with a static `T::CreateInstance(CComPtr<T>&)` in the class's own
header that delegates to `ShellObjectCreator`. The class-name template argument the other
three patterns key on is gone, so the call is the only place the name still appears.

Two habits did the work here and are worth keeping:

- **Agents were told explicitly that "the scanner is wrong" was an allowed verdict.** Four of
  the six above came back from agents who took that option instead of re-pinning a fixture.
- **A test pinned to a value the agent believed was wrong was left RED on purpose**, with the
  correct expectation in it, rather than pinned to the broken behaviour. Both `cpp_com` bugs
  arrived that way.

One near-miss worth recording against myself: two `test_cpp_com` failures appeared right
after my own edit to that file and looked caused by it. They were pre-existing — visible only
because the earlier `check-tooling` output had been truncated to its last few lines. Reverting
the edit and re-running took a minute and settled it; guessing would have cost the fix.


## Three boot regressions the build could not see

The merge built clean on all three arches and still did not boot on two of them. Every one of
these was invisible to the compiler, the linker and all 54 source scanners; only smoke found
them, and only a bisect located them. **arm64 passed throughout**, which was itself the clue
in the first case.

### 1. The registered IDE driver was silently swapped (i386 + amd64, BUGCHECK 0x6B)

Upstream is moving to "pciide is the single registered IDE driver", building **uniata AS
pciide** when `ENABLE_UNIATA` is set. The merge therefore:

```diff
-add_registry_inf(uniata_reg.inf)
+# add_registry_inf(uniata_reg.inf)
```

and added `add_registry_inf(pciide_reg.inf)` + `legide_reg.inf` instead. We do not set
`ENABLE_UNIATA`; pre-merge this tree registered **uniata** and pciide not at all. One commented
line unregistered the only IDE bus driver we start.

The failure chain is worth reading once, because nothing in it names IDE:

```
PiEnumerateDevice HTREE\ROOT\0: 1 children      (a good boot has 3)
[CDARC] matched signature, CdRomCount=0
[IO] IopCreateArcNames FAILED
[IO] IopMarkBootPartition FAILED
...
[IO] PsLocateSystemDll
*** BUGCHECK: 0x6B (0xC000003A = STATUS_OBJECT_PATH_NOT_FOUND)
```

No IDE bus driver → no CD-ROM → no ARC name → no `\SystemRoot` → ntdll not found. **arm64
boots from a ramdisk and never touches the IDE path**, which is exactly why one arch passed.

The differential that found it was a *passing* amd64 boot from another worktree: same log,
`3 children` and `CdRomCount=1`. Two log lines settled what an hour of reading diffs would not.

### 2. A `.spec` forwarder pointed at a module we had declined (i386 + amd64, BUGCHECK 0x4C)

Upstream re-pointed two shell32 exports at the COM/shell restructure:

```
176 stdcall SHSetInstanceExplorer(long) shcore.SetProcessReference
@   stdcall SHGetInstanceExplorer(long) shcore.GetProcessReference
```

We declined shcore, so the forwards dangled — `LDR: SNAP FAIL import shcore.GetProcessReference
from browseui.dll`, BUGCHECK 0x4C during csrss startup. `shell32-rust` implements both
locally; the forwarders are reverted. **A declined module is not declined until nothing
forwards to it** — sweep `*.spec` for `<declined>.` after any decline.

### 3. One security method kept the old arity (i386 ONLY, BUGCHECK 0x1E)

This one is mine, and it is the exact hazard described two sections above, walked into from the
other side. Having made `ObpCallSecurityProcedure` pass upstream's new ninth argument to
whatever sits in `SecurityProcedure`, I updated three of the **four** security methods in the
tree. `WmipSecurityMethod` (`wmi-rust/src/guidobj.rs`) still took eight.

`extern "system"` is stdcall on i386 and the **callee** pops: an 8-parameter method reached
through a 9-argument call leaks four bytes of stack *per call*. The boot got all the way to the
system-driver phase and then died at the first heavy registry walk. amd64 and arm64 pass extra
arguments in registers and never noticed.

**The lesson is not "check the callers" — I did that. It is that a function POINTER has no
single callee.** When an indirect-call signature changes, the unit of work is every
implementation that can land in that slot, enumerated mechanically:

```bash
# every security method, with its arity
grep -rn 'SecurityProcedure *= *' --include='*.rs' --include='*.c' sdk/lib/rust/ ntoskrnl/
# ...then check each target's parameter count. Same for CloseProcedure / OkToCloseProcedure.
```

All four are now nine; all close methods are five. That sweep belongs in a scanner.

### What this says about the gate

`check` and `check-tooling` were green for all three of these. The ARC-name one would have been
caught by nothing short of a boot. **Smoke is the north star, and a merge of this size is not
landed until it has passed on every arch** — a green build on three arches proved only that the
symbols resolved.

Also worth recording: bisecting cost four build+smoke cycles and found the answer three times.
Each time I had a plausible theory first, and each time the theory was wrong — the security
methods looked obviously guilty for the ARC-name failure (they were innocent), and the cache
manager's new race guard looked obviously guilty for the i386 hang (also innocent, and a
faithful port).


## The committed artifacts, the difftest harness, and one oracle I broke myself

Three more classes surfaced after the boot was green. The last one is the most instructive and
it is my own mistake.

### Every "correct by construction" artifact went stale at once

The cmake-off cargo build keeps generated inputs committed so it needs no cmake configure. The
merge invalidated all of them simultaneously: the `cflags/*.flags` templates (upstream removed
the tree-wide `-D_CRT_NON_CONFORMING_SWPRINTFS`), all four `.rc.res` blobs per arch, and
`committed-headers/bugcodes.h` (upstream added `DRIVER_POWER_STATE_FAILURE 0x9f`). Regenerated;
`check_committed_drift.py` now reports 14 OK / 0 DRIFT on all three arches.

It also rejected my first swprintf fix, correctly. Scoping the define with
`set_source_files_properties` gave `libcntpr` **two distinct flag sets**, and the cmake-off
build replays exactly one per target. The define moved into the two `.c` files instead, where
the build system never sees it.

**A latent gap found on the way:** `gen_cflags.py` and `gen_committed_res.py` both defaulted to
`i386 amd64` while `check_committed_drift.py` has always checked **arm64**. Regenerating after
any input change therefore left arm64 stale, discoverable only one gate later. Both now default
to all three arches.

### `rtl-difftest` broke four ways, all because it compiles the genuine C

The harness builds `sdk/lib/rtl/*.c` as `ref_*` and diffs it against our Rust — so refreshing
that C is refreshing the oracle:

- `env.c` gained `UNICODE_STRING_MAX_CHARS` / `STATUS_UNSUCCESSFUL` guards and a `_wcsnicmp` call
- `unicode.c` / `sid.c` renamed their `swprintf` call sites to `_swprintf`
- `RtlCompareUnicodeStrings` **moved** from `unicode.c` to `unicode_vista.c`, which the harness
  did not compile, so the test's `ref_` symbol vanished

### ...and then I broke the oracle and read its output as truth

For `_wcsnicmp` I wrote `#define _wcsnicmp wcsncasecmp`. **Host `wchar_t` is 32-bit and `WCHAR`
here is 16-bit** — which is precisely why `refenv_wcschr` already exists a few lines above, a
convention I walked straight past. Every name comparison in the oracle's
`RtlQueryEnvironmentVariable` then mismatched, so the C returned `STATUS_VARIABLE_NOT_FOUND` for
every query.

I reported that as "our Rust is behind upstream". It was not evidence about our Rust at all; it
was evidence about a shim I had written twenty minutes earlier. The agent doing the port caught
it with a standalone repro rather than accepting the harness's verdict.

This is the field manual's own rule — *a tool's account of itself is not evidence about that
tool; re-derive by a route the suspect tool does not mediate* — and the thing that made me miss
it is that **I had just repaired the harness, so I trusted it more than usual.** Having fixed a
measuring instrument is not the same as having verified it.

The other divergence was real and independent: upstream's rewritten `_U` wrapper writes
`Destination->Length` **only on success**, where ours wrote it unconditionally. That one is now
ported, along with the new raw-buffer cores `RtlExpandEnvironmentStrings` /
`RtlQueryEnvironmentVariable` (both in `ntdll.spec` at `-version=0x600+`), the strictly-less-than
capacity test for the query path, and the no-partial-copy-on-too-small behaviour. All 87
rtl-difftest cases pass, and the full difftest suite is green.


## The all-in-one BootCD commit reached four files, not two

`b2e33f26eb3` ("Merge our bootcd and livecd into an all-in-one ReactOS BootCD") is the single
upstream change that cost this merge the most, and it did so by arriving through **four
different files, on four different days of investigation**:

| file | how it was caught |
|---|---|
| `boot/bootdata/**` | by reading, before the merge was committed |
| `boot/boot_images.cmake` | same |
| `drivers/storage/ide/uniata/CMakeLists.txt` | by SMOKE — BUGCHECK 0x6B |
| `sdk/cmake/CMakeMacros.cmake` | by CONFORMANCE — 91 kmtests `launch FAILED err=2` |

The last one is the sharpest. `add_cd_file` took upstream's rewrite, which drops `hybridcd`
and `NOT_IN_HYBRIDCD`, renames `livecd.iso` → `liveimg.iso`, and rewrites every bootcd
destination:

```cmake
string(REGEX REPLACE "^reactos([\\/]+|$)" "${ARCH}\\1" _CD_ARCH_DESTINATION "${_CD_DESTINATION}")
```

So everything staged for `reactos/bin/` landed in `amd64/bin/`. **The guest still booted** — the
livecd path is separate — and nothing in build, lint, difftest or smoke could see it. Only the
conformance sweep, which actually looks for a file by absolute in-guest path, noticed.

`git show --name-only b2e33f26eb3` lists all 30 files and I had that list on the first day. I
checked it against the CONFLICT set, found 8, and moved on. **`CMakeMacros.cmake` merged
cleanly, so it never came back into view.** When declining a structural upstream change, the
unit of work is every file the commit touched, conflicted or not.

### ...and a fifth, from a different commit, in the same investigation

With the CD layout fixed the sweep still reported `launch FAILED err=2` for all 91. Upstream
had also deleted:

```cmake
-set_target_properties(kmtest PROPERTIES OUTPUT_NAME "kmtest_")
```

renaming the binary to `kmtest.exe`. Our conformance harness — `tools/payloads/kmtest/sweep.c`
and the three **committed prebuilt** `kmtest-sweep-<arch>.exe` built from it — launches
`%SystemRoot%\bin\kmtest_.exe` by name. Restored the rename rather than rebuilding three
committed binaries for a cosmetic upstream change.

Result: `launched=91 ok=91 bad=0`, with one ratchet entry to chase
(`IoReadWrite: NEW failing assertion service.c:191` — a service-state wait timeout, on a test
whose 16192 passes are unchanged).

**The pattern across all five:** a structural upstream change is not declined by resolving its
conflicts. It is declined by enumerating its commit and auditing every file, because the
cleanly-merged ones are exactly the ones nobody looks at again.


## The object-callback ABI: three families, three separate discoveries

Upstream moved the whole NT object-callback surface to NT6 form in this merge
(`sdk/include/ndk/obtypes.h`, `ntoskrnl/include/internal/*.h`). Our Rust carried the
pre-fork shapes, and the checked-in `ntkrnl-sys` bindgen still describes those. I found the
three affected families **one at a time, each after the previous fix let the boot get
further** — which is the real lesson here.

| family | header now | what was wrong | how it showed |
|---|---|---|---|
| `OB_SECURITY_METHOD` | 9 params (`+KPROCESSOR_MODE AccessMode`) | callers passed 8 | fixed proactively; then `WmipSecurityMethod` was left at 8 and **i386 bugchecked 0x1E** |
| `OB_CLOSE_METHOD` | 5 params, handle counts `ULONG`→`ULONG_PTR` | width only | silent on i386 (same size), a garbage high half on 64-bit |
| `OB_OPEN_METHOD` | 6 params (`+AccessMode`), `GrantedAccess` **by pointer** | callers passed 5, by value | **i386 BUGCHECK 0x1E write-AV** inside kmtest `ObType`, killing 39 of 91 tests |

The open method is the worst of the three because the mismatch is in both the count *and* a
type: the callee pops one argument too many **and** dereferences what the caller passed as a
value. amd64 and arm64 "passed" throughout — extra arguments ride in registers there, so they
were reading garbage for `AccessMode` and nobody noticed.

It is not an accident that kmtest's `ObType` is what caught it. Upstream rewrote that test in
C++ **specifically to exercise both callback generations**, templating over `NtDdiVersion` with
`OB_OPEN_METHOD_WS03` (5 params) and `OB_OPEN_METHOD_VISTA` (6) side by side. Their test
installs a Vista-shaped method; our kernel called it WS03-shaped.

### The sweep that should have been first, not third

```bash
# every object callback the tree installs, and every implementation's arity
grep -rn 'SecurityProcedure *= *\|OpenProcedure *= *\|CloseProcedure *= *\|ParseProcedure *= *' \
     --include='*.rs' --include='*.c' sdk/lib/rust/ ntoskrnl/
# ...then compare each target against the typedef in sdk/include/ndk/obtypes.h
```

Run across all seven families it now reports agreement:

```
OB_OPEN_METHOD 6 · OB_CLOSE_METHOD 5 · OB_DELETE_METHOD 1 · OB_PARSE_METHOD 10
OB_SECURITY_METHOD 9 · OB_QUERYNAME_METHOD 6 · OB_OKAYTOCLOSE_METHOD 4
```

**When an indirect-call signature changes, the unit of work is the FAMILY, not the call site
that happened to fail.** I wrote that down after the second instance and still went looking
for the third one symptom-first. It belongs in a scanner, so nobody has to remember it.

### ...and a fourth, because arity was only one of the two axes

The sweep above compared *arity*. It reported agreement, and a day later `IoReadWrite` was
still failing on amd64 — `Service = Kmtest-IoReadWrite, state = 4 RUNNING (!= 1 STOPPED)`,
five runs out of five, i386 and arm64 clean.

`OB_CLOSE_METHOD` kept its five parameters but upstream widened two of them, `ULONG` →
`ULONG_PTR` (`obtypes.h`). Every close procedure in the tree is compiled against the new
header; `ob-rust` called through the checked-in bindgen island, which still carries the old
one, and so passed 32-bit arguments. The Win64 ABI leaves the high half of an argument
register undefined, and `IopCloseFile` opens with `if (HandleCount != 1) return;` — so a
garbage high half silently skipped IRP_MJ_CLEANUP. The file's cache map was never torn down,
the driver could not unload, and the SCM left the service in RUNNING until the test's 10 s
timeout expired. i386 is immune because `ULONG_PTR == ULONG` there, which is precisely what
made a plain ABI mismatch read as an arch-specific mystery.

Fixed the way the other three were: `ObpCallCloseProcedure` in `ob-rust/src/obsecure.rs`
transmutes past the stale field type. **The full amd64 sweep also dropped 517 s → 359 s** —
every lingering service had been burning its own 10 s timeout.

So the scanner that section asks for has to check parameter *types*, not just how many there
are. Of the seven families, `OB_CLOSE_METHOD` is the only one upstream widened; the other six
agree on both axes now.

### The cache-manager races: a correct fix that was not this bug

Chasing the same symptom I found `ntoskrnl/cc/view.c` refreshed by +122 lines of upstream
concurrency hardening that `cc-rust/src/view.rs` did not have, and ported four of them:
`CcRosFlushVacb` now honours the `WasMarked` return of `CcRosUnmarkDirtyVacb` instead of
re-marking unconditionally; `CcRosMarkDirtyVacb` returns early on an already-dirty VACB
instead of double-inserting it and leaking a reference; `CcRosDeleteFileCache` moves VACBs to
a private list under the lock so a concurrent `CcRosLookupVacb` cannot take a reference in the
window before they are freed. (`CcRosUnmarkDirtyVacb`'s own guard had already been ported
earlier in the lane — which, on its own, was *half* a five-part fix: it could now return FALSE
to a caller that ignored the result.)

These are real bugs and the ports stand, but they did not move the symptom: the conformance
run after them was byte-identical. Worth recording because the mechanism was plausible enough
that I believed it — a leaked VACB reference also keeps a cache map alive and also blocks
unload. Plausible and true are different, and only the re-run told them apart.

### Two findings that dissolved on re-measurement

Both looked like regressions and neither was:

- **`NpfsFileInfo.c:57`** — reported as a NEW failing assertion on i386. It did not reproduce:
  the test came back at exactly its baseline `pass=50 fail=25`. It was collateral from the
  crashing sweep, not a finding.
- **`MmMdl: pass 594 -> 593`** — with `fail=0` on both sides. Upstream made one assertion
  conditional (`if (MmGetMdlByteCount(Mdl) >= (1UL << 31))`, skipping the 2 GB mapping
  expectation when `MmAllocatePagesForMdl` returns fewer pages) and relaxed another from `==`
  to `<=`. One fewer assertion *executed*, none failing. Re-seeded that single baseline row,
  by hand and only that row.

And the three `NO-END` entries in the i386 sweep (`RtlCaptureContextKM`, `RtlSplayTree`,
`RtlCaptureContextUM`) are byte-identical to the baseline — pre-existing, not new.


## Owed, in rough priority order

1. **`RtlIsProcessorFeaturePresent` and `RtlGetCurrentProcessorNumberEx` have no implementation
   anywhere in the tree** and are exported by `ntdll.spec`, `ntdll_vista.spec` and
   `kernelbase.spec`. Upstream's new `processor.c` is the only definition that has ever existed,
   but it also redefines `RtlGetCurrentProcessorNumber`, which `rtl-strings` owns. Hand-port the
   two, or split the file.
2. **Upstream's AHCI/SATA work.** A new SATL/port atapi driver plus a 16-file chipset quirks
   framework with a full software AHCI implementation in `pciidex/chipset/`. All of it is on
   disk and none of it is built. Probably the largest single translation opportunity this merge
   surfaced.
3. **Job objects.** `ntoskrnl/ps/job.c` gained 2,198 lines of working implementation and
   `win32ss/user/ntuser/job.c` is its win32k half. Ours are stubs — which is exactly why
   `PspCloseJob` is absent from `ps-rust`.
4. **kernelbase.** Decide whether our minimal shim can satisfy `combase`/`coml2`/`shcore` now
   that they exist in the tree.
5. **oleaut32 `thunks.c`.** Declined on the inference that `dispatch.rs` replaces it. An
   arbitrary-arity native call trampoline is usually precisely what stable Rust cannot emit —
   verify `DispCallFunc` actually works without it.
6. **Raw input.** `win32ss/user/ntuser/rawinput.c` is new and has no Rust twin; `user32/misc/stubs.c`
   shows upstream implemented `GetRawInputBuffer` and `GetRawInputDeviceInfoA` too.
7. **PCIe ECAM.** `drivers/bus/pcix/pci/ecam.c` + `express.c`, new upstream, no Rust twin.
8. **`drivers/bus/acpi/osl.c` was merged rather than picked** — boot-check that upstream's
   `AcpiLoaderRsdp` fast path does not skip what the legacy BIOS scan supplied.
9. **`sdk/cmake/gcc.cmake`** — we kept the llvm-mingw libc++ path over upstream's `libgcc_eh`
   addition. Unclear whether it matters for a non-llvm-mingw GCC build.
