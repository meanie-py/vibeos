# Static-Lib Translation Audit (cargo drive)

> **Translation/cmake-off: ~100% (methodology: per-lib, against the live tree).**
> The kernel `static_libs` vec in `ntoskrnl-cargo`/`ntkrnlmp-cargo build.rs` is now
> **EMPTY** (re-verified 2026-06-21 — the `vec![…]` block is comment-only); the
> cargo-driven `ntoskrnl.exe`/`ntkrnlmp.exe` link references **zero cmake-built `.a`**.
> Every cargo-linked kernel-mode and user-mode-spine module reaches its entire C/asm
> floor via Rust (`.rust_lib`), in-cargo `c_from_compile_db` (irreducible C variadics),
> `asm_archive` (irreducible asm), or `.import_lib` (hal/kdcom/bootvid). The only
> cmake-built `.a` still reused on *any* cargo link line is the bootloader floor
> (uefildr/rosload) + a couple of tiny GUID/uuid data libs (libuuid). **The remaining
> "debt" is cargo-native floor-cleanup (variadic shells / asm leaves), not C→Rust
> translation.** Global fresh facts (2026-06-21): Rust body = 1,992 `.rs` / ~2.87M LOC
> / 207 crates; 594 cohab markers. **The staticlibs/cargo-drive cmake-off arc is
> COMPLETE this session: kernel + the 13-module spine + 25/25 cohab drivers + win32k
> are all cmake-off byte-equivalent (landed on mainline); `tools/cargo_orchestrate.py`
> (target cells + `--spine`, builds each PE cargo-native and post-links it
> byte-equivalent to the cmake archive) is productionized.**
>
> **Ground-truth re-verified 2026-06-21** against `sdk/lib/rust/*-cargo/build.rs`
> + CMakeLists + code. **CORRECTION (supersedes the 2026-06-14 banner): cmlib is
> NOT in the arm64-gated set — the kernel cmlib body is 100% Rust on ALL three
> arches.** cmlib's cmake target REMOVE_ITEM's all 13 `.c` → a placeholder TU on
> every arch (`sdk/lib/cmlib/CMakeLists.txt:28-59`) and `build.rs` links
> `cmlib-rust` **unconditionally** (`ntoskrnl-cargo/build.rs:282`). The C-free
> clean build is exactly what exposed (then fixed) the `write_bytes`
> element-vs-byte APC bug (RESOLVED 2026-06-11); i386+amd64 boot C-free.
>
> **✅ #35 WALL FULLY DISSOLVED (re-verified 2026-06-21).** The five deep-C cohab
> libs — **cportlib / csq / arbiter / lsalib / rossym** — are no longer gated *or*
> cmake-built on any arch: the kernel `build.rs` now pulls them as cargo-native
> `.rust_lib("cport-rust"/"csq-rust"/"arbiter-rust"/"lsalib-rust"/"rossym-rust")`
> uniformly across i386/amd64/arm64 (`ntoskrnl-cargo`/`ntkrnlmp-cargo build.rs`
> ~lines 323-327). This **supersedes** the earlier x86-ungate (`bacd1fc64d3`, the
> `aarch64 || x86 || x86_64` gate that re-added the cmake `libX_rust.a` on x86) —
> that whole block is gone (commit `e8765d21008`, 2026-06-20, ×3-green incl. amd64
> cold), and both halves of each cohab dropped from `static_libs` (5 empty C-body
> stub `.a` + 5 cmake-built `libX_rust.a`). The cmlib-rust no-prepopulate class-guard
> survived the layout shift (0 hivewrt failures, amd64 cold ✓boot ✓net). The blocker
> was NEVER the suspected cmlib
> "layout-sensitive miscompile" on the `Hv*` hive-write path — that was a
> red herring. The real blocker: an **i386 stdcall/cdecl ABI mismatch** in cmlib
> `cmcheck.rs`/`cmheal.rs` (CMAPI fns `HvMarkCellDirty`/`HvIsCellAllocated`/
> `HvGetCellSize`/`CmpCompareCompressedName` declared `extern "C"` but defined
> `extern "system"` = stdcall `_Name@N`; LLD fuzzy `_foo`↔`_foo@N` links it, both
> sides pop → ESP drifts per call → the deep `CmCheckRegistry` tree-walk `ret`s
> into stack word `0x20` → BUGCHECK 0x1E at smss). Fixed by making the decls
> `extern "system"` (i386-only effect; no-op on amd64/arm64). Field manual §"#35
> SOLVED". Follow-up owed: test-remove the cmlib no-prepop guard (kept
> conservatively; the real bug was ABI not captures(none)).
> **Loader-side cmlib is ALSO Rust now**: uefildr/rosload link
> `cmlib-rust --features bldr` (the `-D_BLDR_` read-only surface) — the C
> `blcmlib` is no longer linked by any loader (`da9c72b240d`, 2026-06-11; the
> cmake `blcmlib` target is vestigial). So **cmlib is 100% Rust end-to-end —
> kernel AND loader, all 3 arches**. The fs-drivers btrfs/ext2/ntfs + fslibs were deleted
> (`14a0563b291`) — this doc never referenced them, so no correction needed.
> No softfloat gap: the libcntpr float/printf/scanf core is already Rust.

Created 2026-05-09. **Major refresh 2026-06-13** (cargo drive went fleet-wide;
the original "ntoskrnl-cargo PoC, 18 archives, ~165K LOC" framing below was
~5 weeks stale — it predated the cohab-driver fleet, the cmake→cargo
kernel/driver/DLL/spine cutover, and the Tier-B C-floor work).
Sister docs: `ntoskrnl-translation-audit.md`, `ntdll-cargo-drive-audit.md`,
`cargo-drive-unification-plan.md`, `driver-translation-audit.md`.

> Scope: the **cmake-built C/asm static libs** that the cargo-driven PE links
> (`sdk/lib/rust/*-cargo/build.rs` → `reactos-link`) still consume. This is the
> ledger for "what C floor is left, and how is each module getting it."
> Standalone `.sys` drivers live in `driver-translation-audit.md`; the kernel
> body in `ntoskrnl-translation-audit.md`; RTL bodies in
> `rtl-translation-audit.md`.

## How a cargo module gets its C/asm floor (the 5 mechanisms)

Since 2026-05-09 the link engine grew several ways to satisfy a module's
non-Rust floor. In rough order of "most cmake-coupled" → "least":

1. **`.static_libs(&["…/libX.a"])`** — reuse a **cmake-BUILT** `.a` verbatim.
   The remaining hard cmake tie: cmake must have compiled the archive first.
   This is the surface this ledger tracks (section "Remaining cmake-built .a").
2. **`.c_from_compile_db("X", &[srcs])`** — cargo compiles the C **itself** by
   replaying cmake's flag-exact `compile_commands.json` entry, then `ar`s an
   in-group archive (lazy pull, byte-equiv to the cmake `.a`). Still C, still
   needs the cmake-generated headers + `compile_commands.json`, but **no
   prebuilt `.a`**. (libcntpr printf/scanf residue; blrtl; usersrv shim.)
3. **`.msvcrtex_crt(MsvcrtexKind)`** — the mingw CRT-startup floor. Originally
   (2026-06-13) a packaged `c_from_compile_db` + `asm_archive`; **now**
   (2026-06-13, later) the three Rust staticlibs (crt-startup-rust /
   crt-dllmain-rust / crt0-rust) + the `msvcrtex_asm` archive (the only C-side
   residue is the i386/amd64 stack-probe/integer asm). See "Debt → DONE" below.
4. **`.c_shims(&["x.c"])` / `.asm[_archive](…)`** — cargo compiles **header-free**
   C (or assembles asm) in-cargo. Closest to the "no headers" goal. (pseh
   `dummy.c`; the driver cohab stubs; the ntdll/ntoskrnl `*_rust_shims.c`.)
5. **`.rust_lib("crate", …)` / `.rust_staticlib("crate")`** — a Rust crate, no C
   at all. The actual translation endpoint.

A lib is "done" only when every consumer reaches it via (5) — Rust. (2)–(4)
remove the *prebuilt-`.a`* coupling but are still C/asm = **translation debt**.

## Where we are (2026-06-13 → 2026-06-21)

**2026-06-21 status:** the cmake-off arc is COMPLETE — the kernel `static_libs` vec
is EMPTY (both `ntoskrnl-cargo` and `ntkrnlmp-cargo`), and the spine + 25/25 cohab
drivers + win32k are all cmake-off byte-equivalent and landed on mainline. The
historical narrative below (from the 2026-06-13 fleet-wide-cutover refresh) is kept
as the record of how each lib got there; the per-row "Status" cells were re-verified
2026-06-20/21.

The cargo drive is now **fleet-wide**: 43 `*-cargo` modules (kernel ×2, hal,
ntdll ×2, the csr/spine exes, win32k, the boot loaders, the KD transports, and
the driver/DLL fleet). The early "18-archive ntoskrnl PoC" is one module among
many. The kernel (`ntoskrnl-cargo`/`ntkrnlmp-cargo`) is now **by far the
deepest remaining cmake tie** — it alone still reuses 16 cmake-built `.a`s as
its C-first cohab floor.

**Landed 2026-06-13 (this session):** bootvid + framebuf dropped their last
`.a` (memcmp / libcntpr → cnt-rust-lib); **basesrv reached ZERO cmake .a**
(inert pseh+chkstk dropped after an `nm -u` residue probe); lsass dropped
oldnames. Two genuine C leaf libs were **translated to Rust**: `libdmilib.a`
(`ParseSMBiosTables` → `i8042prt-rust::dmi`, i8042prt now ZERO .a) and `libnt.a`
(`NtProcessStartup` → `smss-csrss-rust::entry_point`, **csrss.exe now ZERO .a**).
The dmilib port also fixed a fabricated SYS_VENDOR/PRODUCT enum-slot bug.
**csrsrv + smss reached ZERO cmake .a (commit `cf7a6caa082`)**: `libsmlib.a` is
NOT translation debt — smclient.c is already a Rust port (`smclient.rs`, compiled
into the body via `pub mod smlib`), smutils.c is dead (compiled nowhere), so the
cmake target is a comment-only cohab stub. The body defines all 7 Sm* + SmBaseTag
itself → libsmlib inert; smss's pseh/chkstk likewise inert (residue probe ×3).
All booted ×3. **userinit + winlogon + services reached ZERO `.static_libs()` .a
(commit `36af6b0eebb`)**: their reused libwine/libpseh/libchkstk/oldnames all
proved inert (residue probe ×3 — no PSEH, no undefined stack-probe, nothing from
wine debug stubs or oldnames CRT aliases). libmsvcrtex stays in-cargo via
`.msvcrtex_crt()` (separate cross-cutting CRT project, not a `.a` reuse).
Modules now fully cargo-native (ZERO cmake `.static_libs()` .a): pci, usbd,
processr, swenum, hidparse, classpnp, cabinet, mlang, bootvid, framebuf,
i8042prt, basesrv, csrss, csrsrv, smss, userinit, winlogon, services,
kdcom, kdgdb, kdvm, kdcom-serial, **hal** (23 — the 4 KD transports confirmed
zero-`.a` on the 2026-06-20 ground-truth re-verify: cargo-native `cport-rust`/
`cnt-rust-lib` via `.rust_lib` + in-cargo printf/scanf shells, commits
`afaba999919`/`d7e2d90bd83`; **hal.dll reached zero `.a` 2026-06-20,
`5f90dce5049`** — dropped inert libsetjmp/libarbiter/libchkstk outright).
The entire user-mode spine is now `.a`-free. **win32k reached near-zero cmake
`.a` (commit `27690a9370f`, 2026-06-20, ×3-green):** dropped libmemcmp
(compiler-builtins-mem), libchkstk (clang_rt.builtins — arm64 cold-relink proved
it covers `__chkstk` on aarch64, the largest stack-probe consumer) and swapped
libcntpr_rust → `.rust_lib("cnt-rust-lib")`; win32k now links only its genuine C
residue (libwin32k_residue + libdxguid), the classpnp fully-cargo-native CRT
pattern. **hal.dll reached ZERO cmake `.static_libs()` `.a` (commit
`5f90dce5049`, 2026-06-20, ×3-green):** its last three reuses
(libsetjmp/libarbiter/libchkstk) were inert vestigial (drop-test EMPTY) and
dropped outright. **Every cargo-linked kernel-mode module is now a fully
cargo-native CRT floor on all 3 arches** (libcntpr/libpseh/libchkstk/libmemcmp/
libcntpr_rust + setjmp/arbiter eliminated from every cargo cut).
**Kernel-floor cohab cleanup (commit `e8765d21008`, 2026-06-20, ×3-green incl.
amd64 cold — the cmlib #35 area held):** the 5 driver/serial cohabs
(cportlib/csq/arbiter/lsalib/rossym) went cargo-native `.rust_lib`, dropping 10
cmake `.a` from the kernel `static_libs` (5 empty C-body stubs + 5 cmake-built
`libX_rust.a`). The cmlib-rust no-prepopulate class-guard survived the layout
shift (0 hivewrt failures, amd64 cold ✓boot ✓net).
**🏆 KERNEL `static_libs` IS NOW EMPTY (2026-06-20): the cargo-driven
ntoskrnl.exe / ntkrnlmp.exe link references ZERO cmake-built `.a`.** The final
clearances this session: libchkstk + libcmlib (placeholder TU) dropped
(`a812475681a`); librtl + libhal (`a009c2ca0af`); libkdcom + libbootvid
self-hosted via `.import_lib` (`dbb83b3b776`); and libntoskrnl_vista dropped as
inert (`97344374dce`). Everything the kernel link needs is now cargo-native:
`.rust_lib` crates, `c_from_compile_db` (irreducible C variadics), `asm_archive`
(irreducible asm), and `.import_lib` (hal/kdcom/bootvid import libs from their
specs). The only non-cargo C left in the *kernel* picture is boot-critical
bootloader C (uefildr/rosload mini_hal+libcntpr) and the winsrv lane — neither
is a `static_libs` `.a` reuse. NB: librtl was **not** a translation lane — the
rtl `*.c` surface is
100% Rust (`rtl-rust`); librtl.a is only the irreducible C-variadic+asm floor
(34 syms i386 / 18 amd64 — see its live-surface row).

## Remaining cmake-built `.a` consumed via `.static_libs()` (the live surface)

Distinct cmake-built archives still on a cargo link line, with consumers.
Sorted roughly by how entrenched (consumer count / centrality):

| `.a` | Consumers | Status / path to Rust |
|---|---|---|
| `sdk/lib/crt/liblibcntpr.a` | kernel ×2, framebuf, rosload, uefildr (classpnp + all 4 KD transports + win32k + winsrv + hal dropped it) | **C residual** of the CRT. Rust half is `cnt-rust-lib` (mechanism 5, already linked alongside). NOT a softfloat gap — the float/printf/scanf CORE is already Rust (streamout engine incl. format_float, floatcvt, math, strtod, scanf). What stays C = the **irreducible variadic plumbing** (thin `va_start; streamout(...)` shells sprintf/_snprintf/… + scanf's `cnt_va_next_ptr`); Rust can't DEFINE C-variadics on stable. ELIMINATION (done for ntdll + all 4 KD transports + kernel ×2 via cntpr_residue): `.c_from_compile_db("libcntpr", &[shells + cnt-rust scanf_trampoline.c])` then drop the `.a`. **2026-06-19: win32k DROPPED** (printf-shell + `_vscwprintf` residue via c_from_compile_db; offline nm drop-test sized it; i386 cold smoke OK) **+ winsrv DROPPED** (ZERO residue — cnt-rust-lib covers its whole surface; i386 cold smoke OK) **+ hal DROPPED** (i386 residue = `sprintf`/`swprintf` shells via c_from_compile_db, amd64 ZERO residue, arm64 never linked it; both x86 flavors cold-built + smoke green incl amd64 ✓net; cgu-fragility was a non-issue — the drop is symbol-clean). **Remaining active: uefildr/rosload** (bootloaders) — **every kernel-mode + spine consumer now libcntpr-free.** **2026-06-20 bootloader residue sized (complete offline, all loader i686 rlibs):** uefildr libcntpr residue = **4** (`__snprintf`/`_sprintf`/`_sscanf`/`_cnt_va_next_ptr`); rosload = **1** (`_cnt_va_next_ptr`) — the irreducible C-variadic/scanf-trampoline shells, droppable-to-`c_from_compile_db` (the ntdll/KD pattern), NOT a hard floor. **`mini_hal` is separately INERT in BOTH loaders** (residue = 0 — `_HalpCalibrateStallExecution@0` unreferenced even by hal-rust[minihal]); the Rust mini-HAL (`hal-rust --features minihal`) fully covers, so the cmake asm-half `libmini_hal.a` is vestigial → **DROPPED 2026-06-20 from uefildr + rosload** (i386 ✓boot through the modified loader, amd64 uefildr.efi link-clean; arm64 full-ninja ramdisk-boot gate pending). |
| `sdk/lib/pseh/libpseh.a` | — (all cargo modules done) | DUMMY_PSEH stub. ntdll/ntdll_vista/classpnp already use the `dummy.c` `c_shim` (mechanism 4); **csrsrv + basesrv + smss + winlogon + services dropped it 2026-06-13; hal + winsrv 2026-06-14**; **✅ kernel ×2 DROPPED 2026-06-19 → `pseh-rust`.** The first attempt (drop to nothing) failed because `ntoskrnl.spec` EXPORTS the PSEH3 runtime `_except_handler2/3`/`_global_unwind2`/`_local_unwind2`/`_abnormal_termination` (-arch=i386,arm) as the driver-facing ABI — libpseh is live (exported), not dead code, so the kernel needed the runtime DEFINED even under DUMMY_PSEH. Solution: `pseh-rust` (`sdk/lib/rust/pseh-rust`) — the i386 `seh.s` runtime as `global_asm!` (disasm-IDENTICAL to the cmake `seh.s.obj` modulo two `push offset`→`mov+push` workarounds for an LLVM push-imm32-symbol reloc limitation) + the 2 `_SEH2_Volatile*` dummy data symbols as Rust statics (needed on ALL arches; referenced by C cohab pseh2.h). Wired `.rust_lib("pseh-rust", &[], true)` into ntoskrnl/ntkrnlmp (all arches; the asm self-gates `cfg(target_arch="x86")`, amd64/arm64 link only volatiles). i386+amd64 cold smoke green; all 5 symbols still exported from ntoskrnl.exe (same ordinals). **✅ win32k DROPPED 2026-06-19** — offline drop-test (undefined across libwin32k_residue + libwin32k_kernel_rust ∩ libpseh defs) = EMPTY on both i386 (full seh.s runtime + volatiles) and amd64 (volatiles-only): win32k's DUMMY_PSEH handlers are dead code and its body never references the `_SEH2_Volatile*` symbols, so the cmake reuse was vestigial → removed the static_lib outright (no pseh-rust/dummy needed). i386+amd64 cold smoke green. **Every cargo-linked module is now libpseh-free.** `seh.s`/`seh_prolog.s`/`dummy.c` kept on disk for any remaining pure-cmake consumer. |
| `sdk/lib/crt/libchkstk.a` | kernel ×2 (i386+arm64), winsrv, uefildr | i386/arm64 stack-probe asm. amd64 gets `__chkstk` from clang_rt.builtins; ntdll/kernel i386 already assemble it in-cargo (`asm_archive`). **classpnp + basesrv + smss + winlogon + services dropped it 2026-06-13; win32k + hal 2026-06-20** (probe: no >4KB-frame refs; uefildr i386 also probes inert but kept — bootloader risk). **2026-06-20: clang_rt.builtins confirmed to provide `__chkstk` on ALL 3 arches** — win32k (the largest aarch64 stack-probe consumer) cold-relinked standalone on arm64 with zero undefined `__chkstk` and booted to explorer, so the `→ asm_archive` plan is moot for the kernel-DLL cargo cuts. Remaining = kernel ×2 + winsrv + uefildr (kernel/uefildr i386 already in-cargo asm; bootloader risk kept). **2026-06-20 uefildr chkstk confirmed INERT** — complete offline residue (libchkstk defs `__chkstk`/`___alloca`/`__alloca_probe{,_16}` ∩ truly-unresolved across ALL uefildr i686 rlibs incl freeldr-rust + hal-rust[minihal] + cport/cnt/cmlib) = **0** (no >4KB-frame refs anywhere in the loader). **DROPPED 2026-06-20 from uefildr** (i386 ✓boot through the modified loader, amd64 uefildr.efi link-clean via clang_rt `__chkstk`; arm64 full-ninja ramdisk-boot gate pending). winsrv keeps it (i386 winsrv-rust references `__alloca`). Remaining = kernel ×2 + winsrv. |
| ~~`sdk/lib/crt/libcntpr_rust.a`~~ | — (no cargo consumers) | **Already Rust** (cmake-built from `cnt-rust`). **2026-06-20 ground-truth re-verify: ZERO cargo modules reuse it** — framebuf (`.rust_lib("cnt-rust-lib")`), kdgdb/kdvm/kdcom-serial (`.rust_lib` + `c_from_compile_db`, commits `afaba999919`/`d7e2d90bd83`), classpnp/ntdll/kernel/hal/winsrv/win32k all on the cargo-native `cnt-rust-lib`. The cmake `.a` survives only for non-cargo cmake consumers. |
| ~~`sdk/lib/crt/libmemcmp.a`~~ | — (no cargo consumers) | One C fn → compiler-builtins-mem (build-std). **2026-06-20 ground-truth re-verify: ZERO cargo modules reuse it** — bootvid (fully zero-`.a`) + kdcom/kdgdb/kdvm (commits `afaba999919`/`d7e2d90bd83`) dropped it alongside the earlier 8 + hal + win32k. The cmake `.a` survives only for non-cargo cmake consumers. |
| ~~`sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a`~~ | ~~userinit, winlogon, services~~ | `_name`→`name` MSVC alias archive. **All cargo consumers dropped it: cabinet/mlang (Push A), lsass + userinit + winlogon + services 2026-06-13** (probe: zero oldname refs ×3). No cargo module reuses it now. |
| ~~`sdk/lib/cportlib/libcportlib.a`~~ | — (no cargo consumers) | Serial/KD port lib. KD transports dropped it (`afaba999919`/`d7e2d90bd83`), uefildr earlier. **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("cport-rust")`; the empty C-body stub left the link too. No cargo module reuses it. |
| ~~`sdk/lib/3rdparty/libwine/libwine.a`~~ | ~~userinit, winlogon~~ | 3rd-party wine glue (~1.1K LOC). **All cargo consumers dropped it: cabinet/mlang (cohab c_shim), userinit + winlogon 2026-06-13** (probe: inert ×3 — bodies reference none of wine's debug-channel stubs). No cargo module reuses it now. |
| ~~`sdk/lib/vcruntime/libsetjmp.a`~~ | — (no cargo consumers) | setjmp/longjmp. **✅ Rust (`setjmp-rust`, commit `a0f85f894bb`)** for the kernel — global_asm! save/restore (disasm-identical to the cmake asm) + Rust longjmp. DROPPED from both kernels' static_libs: amd64 links setjmp-rust, i386 (DUMMY_PSEH) + arm64 reference no setjmp (inert). **hal DROPPED 2026-06-20 (`5f90dce5049`)** — inert there too (drop-test EMPTY; hal references no setjmp/longjmp), dropped outright with no setjmp-rust swap. No cargo module reuses the C `.a`. |
| ~~`sdk/lib/drivers/arbiter/libarbiter.a`~~ | — (no cargo consumers) | HW-resource arbiter. **✅ Rust (`arbiter-rust`, commit `4ebacf613a7`)**. hal dropped it 2026-06-20 (`5f90dce5049`, inert). **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("arbiter-rust")`; the empty C-body stub + cmake `libarbiter_rust.a` both left the link. No cargo module reuses it. |
| ~~`hal/{halx86\|halarm64}/…/libhal.a`~~ | — (no cargo consumers) | NOT a HAL body — the cmake-built **hal.dll IMPORT lib** (spec2def short-import members → hal.dll). `hal.dll` itself is **`hal-rust`** (Rust). **Kernel ×2 DROPPED 2026-06-20 (`a009c2ca0af`)** → kernel self-hosts the import from `hal/hal.spec` via `.import_lib` (the classpnp/win32k pattern). |
| `sdk/lib/cmlib/libcmlib.a` | kernel ×2 (placeholder TU only) | Config-Manager hive lib. **✅ FLIPPED ×3: kernel cmlib is 100% Rust** (`cmlib-rust`, mechanism 5, `.rust_lib` unconditional). The cmake `libcmlib.a` is a placeholder TU on every arch (all 13 `.c` REMOVE_ITEM'd). The **loader is also Rust** (uefildr/rosload link `cmlib-rust --features bldr`, `da9c72b240d`) → cmlib is 100% Rust kernel+loader ×3. cmlib WAS the suspected blocker epicentre, but the "latent hive-path miscompile" was a RED HERRING — the real blocker was an i386 stdcall/cdecl ABI mismatch in cmlib `cmcheck`/`cmheal` (fixed; x86 ungate of the OTHER 5 landed `bacd1fc64d3`). See banner + field manual "#35 SOLVED". |
| ~~`drivers/base/…/libkdcom.a`~~ | — (no cargo consumers) | KD COM transport — the cmake-built **kdcom.dll IMPORT lib** (8 stdcall exports). `kdcom-rust` is the standalone Rust flip. **Kernel ×2 DROPPED 2026-06-20** → self-hosts the import from `drivers/base/kdcom/kdcom.spec` via `.import_lib` (hal pattern). Symbol-identical (the `.a` was itself spec2def-generated). i386+amd64 warm-boot green; arm64 needs the full-ninja cold gate. |
| ~~`drivers/base/…/libbootvid.a`~~ | — (no cargo consumers) | Boot video — the cmake-built **bootvid.dll IMPORT lib** (11 `Vid*` exports). `bootvid.dll` itself is cargo-driven (`bootvid-cargo`). **Kernel ×2 DROPPED 2026-06-20** → self-hosts from `drivers/base/bootvid/bootvid.spec` via `.import_lib`. Symbol-identical; i386+amd64 warm-boot green; arm64 full-ninja cold gate pending. |
| ~~`sdk/lib/drivers/csq/libcsq.a`~~ | — (no cargo consumers) | Cancel-safe queue. **✅ Rust (`csq-rust`, commit `4ebacf613a7`)**. **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("csq-rust")`; empty C-body stub + cmake `libcsq_rust.a` both gone. No cargo module reuses it. |
| ~~`sdk/lib/lsalib/libntlsalib.a`~~ | — (no cargo consumers) | LSA helpers. **✅ Rust (`lsalib-rust`)**. **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("lsalib-rust")`; empty C-body stub + cmake `liblsalib_rust.a` both gone. No cargo module reuses it. |
| ~~`sdk/lib/rossym/librossym.a`~~ | — (no cargo consumers) | ROSSYM reader (~3,083 LOC). **✅ Rust (`rossym-rust`)**. **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("rossym-rust")`; empty C-body stub + cmake `librossym_rust.a` both gone. No cargo module reuses it. |
| ~~`sdk/lib/{wdmguid,poguid,ioevent}.a`~~ | ~~kernel ×2~~ | GUID data libs. **✅ Rust (`guids-rust`, commit `8e61fee3e68`)** — 217 deduped GUID statics, `.rust_lib()`'d into both kernels, the 3 cmake `.a` DROPPED from static_libs. cold smoke ×3. C kept on disk for non-kernel cmake consumers. |
| `sdk/lib/rtl/librtl.a` | ntdll, kernel ×2 | **NOT a translation gap — the whole `sdk/lib/rtl/*.c` surface is Rust** (`rtl-rust` = 5681 syms; rtl-translation-audit.md "ground-truth re-verified 2026-06-14"). **2026-06-20 re-verify: librtl.a is down to 34 syms i386 / 18 amd64 — purely the irreducible floor:** the C-variadic DbgPrint family (`dbgprint_variadic.c`: DbgPrint/DbgPrintEx/wine_dbg_*/ros_dbg_log — Rust can't define C variadics, same class as the printf shells) + per-arch asm (`except_asm.s` SEH dispatch RtlpExecuteHandler*/RtlCaptureContext/RtlRaiseException, `rtlmem.s` RtlCompareMemory/Fill/Move/Zero, `rtlswap.S` byteswaps, `debug_asm.S` DbgBreakPoint/DebugService). Elimination (cargo-native, not translation) = `c_from_compile_db` the variadic shells (cntpr pattern) + route the asm to existing crates (`mem`/`byteswap`) or `global_asm!` (pseh-rust/setjmp-rust pattern) — a floor-cleanup, not a port. |
| ~~`sdk/lib/nt/libnt.a`~~ | ~~csrss, smss~~ | **✅ Rust 2026-06-13** — `NtProcessStartup` ported to `smss-csrss-rust::entry_point` (gated csrss_main_only/smss_exe_only). csrss/smss no longer link it; the host tool keeps the C copy. csrss.exe now ZERO cmake .a. |
| ~~`sdk/lib/smlib/libsmlib.a`~~ | ~~csrsrv, smss~~ | **✅ Resolved 2026-06-13 (`cf7a6caa082`)** — NOT translation debt: smclient.c is already a Rust port (`smss-csrss-rust/src/smlib/smclient.rs`, compiled into the body via `pub mod smlib`); smutils.c is dead (compiled nowhere). The cmake target is a comment-only cohab stub; the body defines all 7 Sm* + SmBaseTag, so it was inert (residue probe ×3). Dropped from both build.rs → csrsrv + smss ZERO cmake .a. C kept on disk. |
| `sdk/lib/uuid/libuuid.a` | winsrv, mlang (2) | GUID/IID constants. Tiny; data. |
| ~~`sdk/lib/cportlib/libcport_rust.a`~~ | — (no cargo consumers) | **Already Rust** (cmake-built). KD transports dropped it (`afaba999919`/`d7e2d90bd83`). **Kernel ×2 DROPPED 2026-06-20 (`e8765d21008`)** → cargo-native `.rust_lib("cport-rust")`. No cargo module reuses the cmake-built `.a`. |
| ~~`sdk/lib/dmilib/libdmilib.a`~~ | ~~i8042prt~~ | **✅ Rust 2026-06-13** — `ParseSMBiosTables` ported to `i8042prt-rust::dmi` (also fixed a fabricated SYS_VENDOR/PRODUCT slot-index bug). i8042prt now ZERO cmake .a; the host tool udmihelp keeps the C copy. |
| ~~`sdk/lib/dxguid/libdxguid.a`~~ | — (no cargo consumers) | DirectX/Direct3DRM GUID *data* (1228 `.rdata` CLSID_/IID_ consts, 0 text). **win32k ×1 DROPPED 2026-06-20**: offline residue (win32k undefined ∩ dxguid-defined) = EMPTY — win32k.sys doesn't reference these user-mode DX GUIDs. Empirical drop-test: i386 ✓boot, amd64 ✓boot+net (win32k.sys relinks clean, no undefined CLSID/IID). Inert, like libntoskrnl_vista. arm64 full-ninja cold gate pending. |
| `win32ss/libwin32ksys.a`, `win32ss/libwin32k_residue.a` | winsrv / win32k | win32k C residue — own track (`win32k-translation-audit.md`). **2026-06-20 residue verified GENUINE (not inert):** proper offline residue (libwin32k_residue defs ∩ win32k-truly-unresolved, subtracting win32k-rlib defs) = **65 live syms** — DIB blitters (`DIB_{8,16,24,32}BPP_BitBlt`/`ColorFill`/`HLine`, C + i386 asm), FLOATOBJ fixed-point math (`FLOATOBJ_*`, C + asm), 50+ `_w32r_*` win32k-rust **cohab shims** (the C↔Rust bridge win32k-rust calls: XFORMOBJ/TEXTOBJ/RtlStringC*/ftGdiGetGlyphOutline/CoordLPtoDP…), and the SSDT (`Win32kSSDT`/`SSPT`/`NumberOfSysCalls`). A real translation lane (entangled with win32k-rust's own cohab), NOT a vista/dxguid-style free drop. |
| ~~`subsystems/.../libbasesrv.a`, `.../libcsrsrv.a`~~ | — (winsrv self-hosts) | NOT bodies — basesrv/csrsrv are 100% Rust DLLs now (kernel32 lane, `8fdbf8a885e`/`9e9b891c244`, landed). These are the cmake-built **IMPORT libs** winsrv links to call them (winsrv imports `BaseSetProcessCreateNotify` + 21 `Csr*` syms). **winsrv ×2 SELF-HOSTED 2026-06-20** via `.import_lib` from `basesrv.spec` (4 exports) / `csrsrv.spec` (35) — the kdcom/bootvid/hal pattern, symbol-identical (cmake .a is spec2def-generated from these). i386 ✓boot + amd64 ✓boot+net (winsrv.dll boot-critical, relinks clean). csrss.exe still `add_importlibs(csrsrv)` — that consumer keeps the cmake import lib until separately swapped. arm64 full-ninja gate pending. |
| ~~`win32ss/.../libconcfg_rust.a`~~ | — (redundant) | cmake-built concfg-rust. **winsrv ×1 DROPPED 2026-06-20**: winsrv-rust already pulls `concfg-rust` as a Cargo dep, so all 50 of libconcfg_rust.a's syms come from the dep (offline residue: concfg_rust defs − winsrv-rlib defs = EMPTY). Same as libcntpr_rust→cnt-rust-lib. i386 ✓boot, amd64 ✓boot+net (winsrv.dll boot-critical, relinks clean). arm64 full-ninja cold gate pending. |

## Status of the original 18 (2026-05-09 → 2026-06-13)

| Lib | Then | Now |
|---|---|---|
| `librtl` | **Rust** | The whole `sdk/lib/rtl/*.c` surface is flipped (`rtl-rust` = 5681 syms; ground-truth re-verified 2026-06-14). `librtl.a` is down to the irreducible floor (34 syms i386 / 18 amd64: C-variadic DbgPrint family + per-arch asm) — see the live-surface row. Translation done; only a cargo-native floor-cleanup remains. |
| `liblibcntpr` | C (~59K) | C residual `.a` (classpnp + all 4 KD + win32k + winsrv + hal dropped it; only uefildr/rosload bootloaders + kernel-reuse framebuf remain) + `cnt-rust-lib` Rust half. Residual = variadic printf/scanf shells (NOT softfloat — the float core is Rust); drop via `c_from_compile_db` (ntdll/KD pattern). |
| `libhal` | C (~24K) | kernel reuse only; the `hal.dll` PE is `hal-rust`. |
| `libcmlib` | C | **Rust ×3 (kernel AND loader)** (`cmlib-rust`); kernel body 100% Rust all arches (cmake `.a` = placeholder TU), loader links `cmlib-rust --features bldr` (`da9c72b240d`). x86 ungate of the OTHER 5 cohab libs landed `bacd1fc64d3` (the suspected hive-path miscompile was a red herring; real cause was an i386 ABI bug). |
| `libbootvid` | C | **Rust** (`bootvid-rust`); `.a` kernel-reuse only. |
| `librossym` | C (~3K) | **Rust** (`rossym-rust`); kernel pulls Rust `.a` ×3 (x86 ungate landed `bacd1fc64d3`). |
| `libpseh` | C/asm | DUMMY stub + i386 SEH runtime. ntdll/classpnp on `dummy.c` c_shim; csrsrv/basesrv/smss/winlogon/services dropped 2026-06-13; hal/winsrv 2026-06-14; **kernel ×2 → `pseh-rust` (global_asm! seh.s + Rust dummy volatiles), win32k dropped outright (vestigial) — 2026-06-19**. No cargo module reuses the `.a`. |
| `libsetjmp` | C | **Rust** (`setjmp-rust`) for the kernel (amd64 global_asm!+Rust; i386/arm64 inert, dropped). hal still C. |
| `libcportlib` | C | kernel/KD reuse; `cport-rust` exists (uefildr). Kernel pulls `libcport_rust.a` ×3 (x86 ungate landed `bacd1fc64d3`). |
| ~~`libntoskrnl_vista`~~ | — (kernel: none; drivers keep it) | ReactOS "Vista+ (NT 6.x) kernel API" static support lib — backfills APIs the XP/2003-era base ntoskrnl.exe doesn't export (PoFx power framework, Etw, FsRtl ECP, IoGet/SetDeviceProperty, node-topology) so newer DRIVERS can statically link them. **Kernel ×2 DROPPED 2026-06-20**: offline residue (vista-defined ∩ kernel-truly-unresolved) = EMPTY both arches, and the empirical drop-test (remove from `static_libs`, build+boot) is i386 ✓boot + amd64 ✓boot+net — vista.a was **inert in the kernel link** (its 35 syms either gc-section'd dead, or the 4 live refs — IoSetMasterIrpStatus/KeGetCurrentNodeNumber/PoQueryWatchdogTime/RtlUTF8ToUnicodeN — already covered by `ntoskrnl-vista-rust`+`utf8`; the 10 PoFx exports are `-version=0x602+`-gated out of this build's ntoskrnl.spec). **`ntoskrnl-vista-rust`** still serves classpnp; the cmake C `.a` stays for the real consumers (nfs/disk/cdrom/wdf/kmtests). arm64 full-ninja cold gate pending. |
| `libcsq` | C (~440) | **Rust** (`csq-rust`); kernel pulls Rust `.a` ×3 (x86 ungate landed `bacd1fc64d3`). |
| `libarbiter` | C (~406) | **Rust** (`arbiter-rust`); kernel pulls Rust `.a` ×3 (x86 ungate landed `bacd1fc64d3`), hal still C. |
| `libntlsalib` | C (~345) | **Rust** (`lsalib-rust`); kernel pulls Rust `.a` ×3 (x86 ungate landed `bacd1fc64d3`). |
| `libchkstk` | C/asm | hybrid: classpnp/basesrv/smss/winlogon/services dropped 2026-06-13; **win32k dropped 2026-06-20** → clang_rt.builtins confirmed to cover `__chkstk` on **all 3 arches** (arm64 cold-relink, largest stack-probe consumer); i386 ntdll/kernel in-cargo asm; amd64 clang_rt. Remaining: kernel ×2 + hal + winsrv + uefildr. |
| `libkdcom` | C (~116) | **Rust** (`kdcom-rust`); kernel `.a` reuse. |
| `libwdmguid`/`libpoguid`/`libioevent` | C | **Rust** (`guids-rust`, 217 GUID statics); DROPPED from both kernels' static_libs (commit `8e61fee3e68`). C on disk for non-kernel cmake consumers. |
| `rtl-rust` (umbrella) | Rust | Rust, via `.rust_lib` (no cmake reuse) in kernel ×2 + ntdll. |

## Translation debt (C that's compiled in-cargo but NOT yet Rust)

Mechanism (2)/(3)/(4) C is off the prebuilt-`.a` hook but is still C — owed
translation for the "all-Rust, no-headers" endpoint:

- ✅ **msvcrtex CRT-startup → Rust (DONE 2026-06-13)** — the mingw runtime
  entry glue (`DllMainCRTStartup`/`[w]WinMainCRTStartup`, `_CRT_INIT` +
  `.CRT$XI*/XC*` `_initterm` walk, `__security_init_cookie`,
  `_pei386_runtime_relocator`, TLS callbacks + key-dtor registry, atexit table,
  PE-section helpers, the gnu top-level exception handler + WIN64 ehandler
  table, `__main`, `_fltused`, and the math/misc/ctype/_invalid_parameter/
  rand_s/mbrtowc/wcrtomb leaves) is now three Rust staticlibs:
  `crt-startup-rust` (the floor; exe/wexe/dll mutually-exclusive features),
  `crt-dllmain-rust` (overridable default DllMain/DllEntryPoint), `crt0-rust`
  (overridable GUI main/wmain→WinMain bridges). `.msvcrtex_crt(MsvcrtexKind)`
  wires them. Only the i386/amd64 MSVC stack-probe + 64-bit integer asm
  (`__chkstk`/`_alldiv`/`_aulldiv`/i386 ceilf/floorf) stays asm (`msvcrtex_asm`
  archive). `dbgrpt.cpp` (`_CrtDbgReport*`) left untranslated — lazy-pulled
  only by a `_DEBUG` libstdc++, unreferenced in release. Boot ×3 validated.
  Key gotchas: `.CRT$X*` `#[link_section]` + `#[used]` boundary arrays
  (lld $-sorts them); msvcrt dllimport DATA (`_acmdln`/`__initenv`/…) via the
  import lib's `__imp_*` slots; **i686-windows-gnu PREPENDS `_` to
  `#[link_name]`** (i386 names carry one fewer leading underscore); `exp2`
  base in a `static mut` (not `black_box`, whose f64 ABI needs SSE).
- **libcntpr i386 float residual** — the no-SSE tail (`nexttowardf`, dead
  `exp2f`/`_invoke_matherr`, `winesup`/`wctype`) → **step 8**. (Possible tie-in:
  the cnt-rust x87-return/`floatbits.rs` softfloat shim.)
- ✅ **Kernel deep-C cohab floor — CLEARED 2026-06-20/21.** The kernel
  `static_libs` vec is now **EMPTY**; the 16-`.a` cohab floor is gone. The five
  deep-C cohabs (cportlib/csq/arbiter/lsalib/rossym) are cargo-native `.rust_lib`
  on all 3 arches (the old "arm64-only Rust, x86 layout-gated" state is obsolete —
  the gate was never needed; the x86 blocker was an i386 stdcall/cdecl ABI bug in
  cmlib `cmcheck`/`cmheal`, NOT the suspected `Hv*` hive-path miscompile, and that
  is fixed). cmlib-C → `cmlib-rust` ×3, libhal/libkdcom/libbootvid → `.import_lib`,
  setjmp → `setjmp-rust`, rtl/cntpr → `.rust_lib` + in-cargo variadic/asm floor.
  Nothing in the kernel link is "un-translated C debt" anymore — see the "KERNEL
  `static_libs` IS NOW EMPTY" note above.

## Forward path (priority)

1. **Pure-plumbing swaps (no translation):** `libcntpr_rust.a` →
   `.rust_lib("cnt-rust-lib")` everywhere; `libcport_rust.a` →
   `.rust_lib("cport-rust")`; `libmemcmp.a` drop (compiler-builtins-mem);
   GUID-only libs → `#[link_section]` statics. Mechanical, fleet-wide.
2. **pseh `dummy.c` c_shim** for the remaining 8 `libpseh.a` consumers.
3. ✅ **step 7 — zlib** (cabinet): DONE 2026-06-13 — cabinet-rust depends on the
   pure-Rust `libz-rs-sys` (zlib-rs), `libzlib.a` dropped; also fixed the dead
   `deflateInit2` macro → `deflateInit2_` (compress path was unresolved/-shared).
4. ✅ **step 8 — libcntpr i386 float tail**: DONE 2026-06-13 for **classpnp** —
   it dropped `liblibcntpr.a` entirely (no C residue needed; cnt-rust-lib +
   compiler-builtins cover it). The other 11 `liblibcntpr.a` consumers remain.
5. ✅ **Kernel deep-C** (arbiter/csq/ntlsalib/rossym/cmlib/…): DONE — all
   cargo-native `.rust_lib`; kernel `static_libs` is now EMPTY (2026-06-20/21).
6. ✅ **msvcrtex CRT-startup → Rust**: DONE 2026-06-13 (crt-startup-rust +
   crt-dllmain-rust + crt0-rust; boot ×3). See the debt note above.

## Definition of done

Every cargo module reaches its entire floor via Rust (`.rust_lib` /
`.rust_staticlib`) or a header-free `.c_shim`/asm — **zero `.static_libs()`
cmake-`.a` entries and zero `.c_from_compile_db` C** on any link line. At that
point the OS image's source-of-truth is `sdk/lib/rust/` (plus genuinely-3rdparty
blobs like wine/zlib only if we keep them as FFI, and the asm leaves).

## What this audit doesn't cover

- The header-free C shims in `ntoskrnl/` — `ntoskrnl-translation-audit.md`.
- Build tooling C (`spec2def.c`, `hpp.c`, …) — runs out-of-band, not linked.
- llvm-mingw's `libclang_rt.builtins-*.a` — third-party compiler runtime.
- win32k / rtl / ntdll bodies — their own `*-translation-audit.md` docs.

## How the LOC numbers were computed (still valid for the unchanged-C libs)

```bash
for d in sdk/lib/{cportlib,cmlib,lsalib,rtl,rossym,crt,vcruntime,wdmguid,poguid,ioevent,pseh} \
         sdk/lib/drivers/{csq,arbiter,ntoskrnl_vista} \
         hal/halx86 drivers/base/{kdrosdbg,bootvid}; do
    find $d \( -name "*.c" -o -name "*.S" -o -name "*.s" \) \
      | xargs wc -l | tail -1 | awk -v d=$d '{print d, $1}'
done
```
