# Static-Lib Translation Audit (cargo drive)

> **Ground-truth re-verified 2026-06-15** against `sdk/lib/rust/*-cargo/build.rs`
> + CMakeLists + code. **CORRECTION (supersedes the 2026-06-14 banner): cmlib is
> NOT in the arm64-gated set — the kernel cmlib body is 100% Rust on ALL three
> arches.** cmlib's cmake target REMOVE_ITEM's all 13 `.c` → a placeholder TU on
> every arch (`sdk/lib/cmlib/CMakeLists.txt:28-59`) and `build.rs` links
> `cmlib-rust` **unconditionally** (`ntoskrnl-cargo/build.rs:282`). The C-free
> clean build is exactly what exposed (then fixed) the `write_bytes`
> element-vs-byte APC bug — see `cmlib-apc-bugcheck-handoff.md` (RESOLVED
> 2026-06-11); i386+amd64 boot C-free.
>
> The genuinely arm64-gated set is the OTHER **five** deep-C cohab libs —
> **cportlib / csq / arbiter / lsalib / rossym**. Their `libX_rust.a` is pulled
> on **arm64 only** (`ntoskrnl-cargo/build.rs:165`); on **i386/amd64 the kernel
> still links the bundled C body** (the build dir's pre-flip `libX.a` still
> carries the symbols — a rust-only `--clean` does not rebuild it). The x86
> ungate of these 5 is *blocked* because adding their archives shifts the kernel
> link LAYOUT and tips cmlib's *latent* layout-sensitive miscompile on the `Hv*`
> hive-write path (cold amd64 HANG at `hivewrt.c`). So cmlib is the *epicentre*
> of the blocker, but cmlib itself is fully flipped ×3 — it's the 5 OTHER
> archives whose x86 ungate is gated. Field manual "Incomplete Rust cohab flip",
> crp `672b9808345`. **Loader-side cmlib is ALSO Rust now**: uefildr/rosload link
> `cmlib-rust --features bldr` (the `-D_BLDR_` read-only surface) — the C
> `blcmlib` is no longer linked by any loader (`da9c72b240d`, 2026-06-11; the
> cmake `blcmlib` target is vestigial). So **cmlib is 100% Rust end-to-end —
> kernel AND loader, all 3 arches**. (`cmlib-loader-swap-handoff.md` is stale; it
> still says OPEN.) The fs-drivers btrfs/ext2/ntfs + fslibs were deleted
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

## Where we are (2026-06-13)

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
i8042prt, basesrv, csrss, csrsrv, smss, **userinit, winlogon, services** (18).
The entire user-mode spine is now `.a`-free; what remains is boot-critical
(hal — cgu-fragile guardrail; uefildr/rosload bootloaders keep genuine
mini_hal+libcntpr), unvalidatable (KD transports — not smoke-covered), or active
other-agent lanes (kernel ntoskrnl/ntkrnlmp 16-.a floor, win32k/winsrv, ntdll librtl).

## Remaining cmake-built `.a` consumed via `.static_libs()` (the live surface)

Distinct cmake-built archives still on a cargo link line, with consumers.
Sorted roughly by how entrenched (consumer count / centrality):

| `.a` | Consumers | Status / path to Rust |
|---|---|---|
| `sdk/lib/crt/liblibcntpr.a` | kernel ×2, hal, win32k, winsrv, framebuf, rosload, uefildr (classpnp + all 4 KD transports dropped it) | **C residual** of the CRT. Rust half is `cnt-rust-lib` (mechanism 5, already linked alongside). NOT a softfloat gap — the float/printf/scanf CORE is already Rust (streamout engine incl. format_float, floatcvt, math, strtod, scanf). What stays C = the **irreducible variadic plumbing** (thin `va_start; streamout(...)` shells sprintf/_snprintf/… + scanf's `cnt_va_next_ptr`); Rust can't DEFINE C-variadics on stable. ELIMINATION (done for ntdll + all 4 KD transports): `.c_from_compile_db("libcntpr", &[shells + cnt-rust scanf_trampoline.c])` then drop the `.a`. |
| `sdk/lib/pseh/libpseh.a` | kernel ×2, win32k (3) | DUMMY_PSEH stub. ntdll/ntdll_vista/classpnp already use the `dummy.c` `c_shim` (mechanism 4); **csrsrv + basesrv + smss + winlogon + services dropped it 2026-06-13; hal + winsrv 2026-06-14** (probe: zero PSEH-runtime refs in the Rust body — dummy.c provides only `_SEH2_Volatile0`/`_SEH2_VolatileExceptionCode`). Remaining (kernel ×2, win32k) can follow — trivial. |
| `sdk/lib/crt/libchkstk.a` | kernel ×2 (i386+arm64), hal (i386), winsrv, win32k (i386+arm64), uefildr | i386/arm64 stack-probe asm. amd64 gets `__chkstk` from clang_rt.builtins; ntdll/kernel i386 already assemble it in-cargo (`asm_archive`). **classpnp + basesrv + smss + winlogon + services dropped it 2026-06-13** (probe: no >4KB-frame refs; uefildr i386 also probes inert but kept — bootloader risk). → `global_asm!` crate or asm_archive everywhere. |
| `sdk/lib/crt/libcntpr_rust.a` | win32k, framebuf, kdgdb, kdvm, kdcom-serial (5) | **Already Rust** (cmake-built from `cnt-rust`). Pure plumbing: swap for `.rust_lib("cnt-rust-lib")` (classpnp/ntdll/kernel/**hal + winsrv 2026-06-14** already do). |
| `sdk/lib/crt/libmemcmp.a` | win32k, bootvid, kdcom, kdgdb, kdvm (5) | One C fn. The 8 driver/DLL modules + **hal (2026-06-14)** already dropped it (compiler-builtins-mem, build-std). Remaining = same one-line drop. |
| ~~`sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a`~~ | ~~userinit, winlogon, services~~ | `_name`→`name` MSVC alias archive. **All cargo consumers dropped it: cabinet/mlang (Push A), lsass + userinit + winlogon + services 2026-06-13** (probe: zero oldname refs ×3). No cargo module reuses it now. |
| `sdk/lib/cportlib/libcportlib.a` | kernel ×2, kdcom, kdgdb, kdcom-serial (5) | Serial/KD port lib. **`cport-rust`/`libcport_rust.a` exists; kernel pulls it on arm64 ONLY** — on i386/amd64 the kernel still links this C body (x86 ungate blocked by the cmlib hive-path layout miscompile, see banner). uefildr uses `cport-rust` via `.rust_lib`. |
| ~~`sdk/lib/3rdparty/libwine/libwine.a`~~ | ~~userinit, winlogon~~ | 3rd-party wine glue (~1.1K LOC). **All cargo consumers dropped it: cabinet/mlang (cohab c_shim), userinit + winlogon 2026-06-13** (probe: inert ×3 — bodies reference none of wine's debug-channel stubs). No cargo module reuses it now. |
| `sdk/lib/vcruntime/libsetjmp.a` | hal (1) | setjmp/longjmp. **✅ Rust (`setjmp-rust`, commit `a0f85f894bb`)** for the kernel — global_asm! save/restore (disasm-identical to the cmake asm) + Rust longjmp. DROPPED from both kernels' static_libs: amd64 links setjmp-rust, i386 (DUMMY_PSEH) + arm64 reference no setjmp (inert). hal still links the C `.a`. |
| `sdk/lib/drivers/arbiter/libarbiter.a` | kernel ×2, hal (3) | HW-resource arbiter. **✅ Rust (`arbiter-rust`, commit `4ebacf613a7`)** — but kernel pulls `libarbiter_rust.a` on **arm64 ONLY**; on i386/amd64 it still links this C body (x86 ungate blocked, see banner). hal still links the C. |
| `hal/{halx86\|halarm64}/…/libhal.a` | kernel ×2 | HAL body the kernel links. The `hal.dll` PE itself is **`hal-rust`** (Rust); this `.a` is the kernel's separate reuse. |
| `sdk/lib/cmlib/libcmlib.a` | kernel ×2 (placeholder TU only) | Config-Manager hive lib. **✅ FLIPPED ×3: kernel cmlib is 100% Rust** (`cmlib-rust`, mechanism 5, `.rust_lib` unconditional). The cmake `libcmlib.a` is a placeholder TU on every arch (all 13 `.c` REMOVE_ITEM'd). The **loader is also Rust** (uefildr/rosload link `cmlib-rust --features bldr`, `da9c72b240d`) → cmlib is 100% Rust kernel+loader ×3. cmlib is still the *epicentre* of the x86-ungate blocker for the OTHER 5 libs: its `Hv*` hive-write path holds a latent layout-sensitive miscompile (cold amd64 HANG at `hivewrt.c`) that any link-layout shift tips. See banner + field manual "Incomplete Rust cohab flip" / "latent hive-path miscompile". |
| `drivers/base/…/libkdcom.a` | kernel ×2 | KD COM transport. `kdcom-rust` is the standalone Rust flip; kernel keeps the `.a`. |
| `drivers/base/…/libbootvid.a` | kernel ×2 | Boot video. `bootvid-rust` is the standalone Rust flip; kernel keeps the `.a`. |
| `sdk/lib/drivers/csq/libcsq.a` | kernel ×2 | Cancel-safe queue. **✅ Rust (`csq-rust`, commit `4ebacf613a7`)** — kernel pulls `libcsq_rust.a` on **arm64 ONLY**; i386/amd64 still link this C body (x86 ungate blocked, see banner). |
| `sdk/lib/lsalib/libntlsalib.a` | kernel ×2 | LSA helpers. **✅ Rust (`lsalib-rust`)** — kernel pulls `liblsalib_rust.a` on **arm64 ONLY**; i386/amd64 still link this C body (x86 ungate blocked). |
| `sdk/lib/rossym/librossym.a` | kernel ×2 | ROSSYM reader (~3,083 LOC). **✅ Rust (`rossym-rust`)** — kernel pulls `librossym_rust.a` on **arm64 ONLY**; i386/amd64 still link this C body (x86 ungate blocked). |
| ~~`sdk/lib/{wdmguid,poguid,ioevent}.a`~~ | ~~kernel ×2~~ | GUID data libs. **✅ Rust (`guids-rust`, commit `8e61fee3e68`)** — 217 deduped GUID statics, `.rust_lib()`'d into both kernels, the 3 cmake `.a` DROPPED from static_libs. cold smoke ×3. C kept on disk for non-kernel cmake consumers. |
| `sdk/lib/rtl/librtl.a` | ntdll (1) | RTL C bodies (~59K LOC). Active track → `rtl-translation-audit.md`; `rtl-rust` umbrella is the algorithm half. |
| ~~`sdk/lib/nt/libnt.a`~~ | ~~csrss, smss~~ | **✅ Rust 2026-06-13** — `NtProcessStartup` ported to `smss-csrss-rust::entry_point` (gated csrss_main_only/smss_exe_only). csrss/smss no longer link it; the host tool keeps the C copy. csrss.exe now ZERO cmake .a. |
| ~~`sdk/lib/smlib/libsmlib.a`~~ | ~~csrsrv, smss~~ | **✅ Resolved 2026-06-13 (`cf7a6caa082`)** — NOT translation debt: smclient.c is already a Rust port (`smss-csrss-rust/src/smlib/smclient.rs`, compiled into the body via `pub mod smlib`); smutils.c is dead (compiled nowhere). The cmake target is a comment-only cohab stub; the body defines all 7 Sm* + SmBaseTag, so it was inert (residue probe ×3). Dropped from both build.rs → csrsrv + smss ZERO cmake .a. C kept on disk. |
| `sdk/lib/uuid/libuuid.a` | winsrv, mlang (2) | GUID/IID constants. Tiny; data. |
| `sdk/lib/cportlib/libcport_rust.a` | kdcom, kdgdb, kdcom-serial (3) | **Already Rust** (cmake-built). Swap for `.rust_lib`. |
| ~~`sdk/lib/dmilib/libdmilib.a`~~ | ~~i8042prt~~ | **✅ Rust 2026-06-13** — `ParseSMBiosTables` ported to `i8042prt-rust::dmi` (also fixed a fabricated SYS_VENDOR/PRODUCT slot-index bug). i8042prt now ZERO cmake .a; the host tool udmihelp keeps the C copy. |
| `sdk/lib/dxguid/libdxguid.a` | win32k (1) | DirectX GUIDs. Data. |
| `win32ss/libwin32ksys.a`, `win32ss/libwin32k_residue.a` | winsrv / win32k | win32k C residue — own track (`win32k-translation-audit.md`). |
| `subsystems/.../libbasesrv.a`, `.../libcsrsrv.a`, `win32ss/.../libconcfg_rust.a` | winsrv | csr server C bodies (+ one already-Rust `concfg`). |

## Status of the original 18 (2026-05-09 → 2026-06-13)

| Lib | Then | Now |
|---|---|---|
| `librtl` | C | C `.a`, ntdll only. `rtl-rust` umbrella = algorithm half. Active rtl track. |
| `liblibcntpr` | C (~59K) | C residual `.a` (classpnp + all 4 KD dropped it) + `cnt-rust-lib` Rust half. Residual = variadic printf/scanf shells (NOT softfloat — the float core is Rust); drop via `c_from_compile_db` (ntdll/KD pattern). |
| `libhal` | C (~24K) | kernel reuse only; the `hal.dll` PE is `hal-rust`. |
| `libcmlib` | C | **Rust ×3 (kernel AND loader)** (`cmlib-rust`); kernel body 100% Rust all arches (cmake `.a` = placeholder TU), loader links `cmlib-rust --features bldr` (`da9c72b240d`). Still the x86-ungate blocker *epicentre* for the OTHER 5 cohab libs (`Hv*` hive-path latent miscompile). |
| `libbootvid` | C | **Rust** (`bootvid-rust`); `.a` kernel-reuse only. |
| `librossym` | C (~3K) | **Rust** (`rossym-rust`); kernel pulls Rust `.a` arm64-only, C on i386/amd64. |
| `libpseh` | C | DUMMY stub; ntdll/classpnp on `dummy.c` c_shim, csrsrv/basesrv/smss/winlogon/services dropped 2026-06-13, 5 still reuse `.a` (kernel ×2, hal, winsrv, win32k). |
| `libsetjmp` | C | **Rust** (`setjmp-rust`) for the kernel (amd64 global_asm!+Rust; i386/arm64 inert, dropped). hal still C. |
| `libcportlib` | C | kernel/KD reuse; `cport-rust` exists (uefildr). Kernel pulls `libcport_rust.a` arm64-only; i386/amd64 still link C (x86 ungate blocked). |
| `libntoskrnl_vista` | C (~780) | **Rust** (`ntoskrnl-vista-rust`) for classpnp; kernel `.a` cohab. |
| `libcsq` | C (~440) | **Rust** (`csq-rust`); kernel pulls Rust `.a` arm64-only, C on i386/amd64. |
| `libarbiter` | C (~406) | **Rust** (`arbiter-rust`); kernel pulls Rust `.a` arm64-only, C on i386/amd64, hal still C. |
| `libntlsalib` | C (~345) | **Rust** (`lsalib-rust`); kernel pulls Rust `.a` arm64-only, C on i386/amd64. |
| `libchkstk` | C/asm | hybrid: classpnp/basesrv/smss/winlogon/services dropped 2026-06-13; i386 ntdll/kernel in-cargo asm; amd64 clang_rt. |
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
- **Kernel deep-C cohab floor** — the 16 `.a`s in `ntoskrnl-cargo`/`ntkrnlmp`.
  Each (arbiter/csq/ntlsalib/rossym/cmlib-C/libhal/setjmp/…) is its own arc.
  The single biggest remaining cmake-built-C surface. **Note:** cportlib/csq/
  arbiter/lsalib/rossym are already Rust-translated and the kernel pulls the Rust
  archive on **arm64 only**; the i386/amd64 ungate (swapping C→Rust there too) is
  blocked by the cmlib `Hv*` hive-path layout-sensitive miscompile — the ungate is
  *not* the right mechanism to chase that (crp `672b9808345`); the C body stays on
  x86 until that miscompile is root-caused. So on x86 these are "translated but the
  flip is layout-gated", not "un-translated debt".

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
5. **Kernel deep-C** (arbiter/csq/ntlsalib/rossym/…): per-lib translation arcs.
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
