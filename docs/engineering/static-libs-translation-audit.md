# Static-Lib Translation Audit (cargo drive)

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
3. **`.msvcrtex_crt()`** — a packaged `c_from_compile_db("msvcrtex", …)` +
   `asm_archive` for the ~40-file mingw CRT-startup set (entry +
   `_CRT_INIT`/`_initterm` + security cookie + pei386 relocator + i386/amd64
   stack-probe/integer asm). Added 2026-06-13; see "Debt" below.
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

## Remaining cmake-built `.a` consumed via `.static_libs()` (the live surface)

Distinct cmake-built archives still on a cargo link line, with consumers.
Sorted roughly by how entrenched (consumer count / centrality):

| `.a` | Consumers | Status / path to Rust |
|---|---|---|
| `sdk/lib/crt/liblibcntpr.a` | kernel ×2, hal, win32k, winsrv, classpnp, framebuf, kdgdb, kdvm, kdcom-serial, rosload, uefildr (12) | **C residual** of the CRT. Rust half is `cnt-rust-lib` (mechanism 5, already linked alongside). Tail = i386 no-SSE float + a few leaves → **step 8**. |
| `sdk/lib/pseh/libpseh.a` | kernel ×2, hal, csrsrv, smss, basesrv, winsrv, win32k (8) | DUMMY_PSEH stub. ntdll/ntdll_vista/classpnp already use the `dummy.c` `c_shim` (mechanism 4); remaining consumers can follow — trivial. |
| `sdk/lib/crt/libchkstk.a` | kernel ×2 (i386+arm64), hal (i386), smss, basesrv, classpnp (i386+arm64), winlogon, services, winsrv, win32k (i386+arm64), uefildr (11) | i386/arm64 stack-probe asm. amd64 gets `__chkstk` from clang_rt.builtins; ntdll/kernel i386 already assemble it in-cargo (`asm_archive`). → `global_asm!` crate or asm_archive everywhere. |
| `sdk/lib/crt/libcntpr_rust.a` | hal, win32k, winsrv, framebuf, kdgdb, kdvm, kdcom-serial (7) | **Already Rust** (cmake-built from `cnt-rust`). Pure plumbing: swap for `.rust_lib("cnt-rust-lib")` (classpnp/ntdll/kernel already do). |
| `sdk/lib/crt/libmemcmp.a` | hal, win32k, bootvid, kdcom, kdgdb, kdvm (6) | One C fn. The 8 driver/DLL modules already dropped it (compiler-builtins-mem). Remaining = same one-line drop. |
| `sdk/lib/crt/CMakeFiles/oldnames.dir/oldnames.a` | lsass, userinit, winlogon, services, cabinet, mlang (6) | `_name`→`name` MSVC alias archive. Tiny; absorb as c_shim/Rust statics. |
| `sdk/lib/cportlib/libcportlib.a` | kernel ×2, kdcom, kdgdb, kdcom-serial (5) | Serial/KD port lib. `cport-rust` exists (uefildr uses it via `.rust_lib`). |
| `sdk/lib/3rdparty/libwine/libwine.a` | userinit, winlogon, cabinet (3) | 3rd-party wine glue (~1.1K LOC). cabinet/mlang dropped it (cohab c_shim covers their surface); winlogon/userinit still pull it. |
| `sdk/lib/vcruntime/libsetjmp.a` | kernel ×2, hal (3) | setjmp/longjmp. ntdll amd64 assembles it from source asm + Rust glue (no `.a`). |
| `sdk/lib/drivers/arbiter/libarbiter.a` | kernel ×2, hal (3) | HW-resource arbiter (~406 LOC C). Untranslated. |
| `hal/{halx86\|halarm64}/…/libhal.a` | kernel ×2 | HAL body the kernel links. The `hal.dll` PE itself is **`hal-rust`** (Rust); this `.a` is the kernel's separate reuse. |
| `sdk/lib/cmlib/libcmlib.a` | kernel ×2 | Config-Manager hive lib. **`cmlib-rust` is the translation** (linked alongside via mechanism 5); the cmake `.a` lingers as cohab placeholder. |
| `drivers/base/…/libkdcom.a` | kernel ×2 | KD COM transport. `kdcom-rust` is the standalone Rust flip; kernel keeps the `.a`. |
| `drivers/base/…/libbootvid.a` | kernel ×2 | Boot video. `bootvid-rust` is the standalone Rust flip; kernel keeps the `.a`. |
| `sdk/lib/drivers/csq/libcsq.a` | kernel ×2 | Cancel-safe queue (~440 LOC C). Untranslated. |
| `sdk/lib/lsalib/libntlsalib.a` | kernel ×2 | LSA helpers (~345 LOC C). Untranslated. |
| `sdk/lib/rossym/librossym.a` | kernel ×2 | ROSSYM reader (~3,083 LOC C). Untranslated. |
| `sdk/lib/{wdmguid,poguid,ioevent}.a` | kernel ×2 | GUID-only (~20/14/9 LOC). Trivial `#[link_section]` Rust statics. |
| `sdk/lib/rtl/librtl.a` | ntdll (1) | RTL C bodies (~59K LOC). Active track → `rtl-translation-audit.md`; `rtl-rust` umbrella is the algorithm half. |
| `sdk/lib/nt/libnt.a` | csrss, smss (2) | NT user-mode entry lib. Untranslated. |
| `sdk/lib/smlib/libsmlib.a` | csrsrv, smss (2) | SM client lib. Untranslated. |
| `sdk/lib/uuid/libuuid.a` | winsrv, mlang (2) | GUID/IID constants. Tiny; data. |
| `sdk/lib/cportlib/libcport_rust.a` | kdcom, kdgdb, kdcom-serial (3) | **Already Rust** (cmake-built). Swap for `.rust_lib`. |
| `sdk/lib/dmilib/libdmilib.a` | i8042prt (1) | DMI/SMBIOS parse. Untranslated. |
| `sdk/lib/dxguid/libdxguid.a` | win32k (1) | DirectX GUIDs. Data. |
| `win32ss/libwin32ksys.a`, `win32ss/libwin32k_residue.a` | winsrv / win32k | win32k C residue — own track (`win32k-translation-audit.md`). |
| `subsystems/.../libbasesrv.a`, `.../libcsrsrv.a`, `win32ss/.../libconcfg_rust.a` | winsrv | csr server C bodies (+ one already-Rust `concfg`). |

## Status of the original 18 (2026-05-09 → 2026-06-13)

| Lib | Then | Now |
|---|---|---|
| `librtl` | C | C `.a`, ntdll only. `rtl-rust` umbrella = algorithm half. Active rtl track. |
| `liblibcntpr` | C (~59K) | C residual `.a` (12 consumers) + `cnt-rust-lib` Rust half. **step 8** for the i386 float tail. |
| `libhal` | C (~24K) | kernel reuse only; the `hal.dll` PE is `hal-rust`. |
| `libcmlib` | C | `cmlib-rust` is the translation (kernel links both; `.a` = cohab placeholder). |
| `libbootvid` | C | **Rust** (`bootvid-rust`); `.a` kernel-reuse only. |
| `librossym` | C (~3K) | unchanged C. |
| `libpseh` | C | DUMMY stub; ntdll/classpnp on `dummy.c` c_shim, 8 still reuse `.a`. |
| `libsetjmp` | C | kernel/hal reuse; ntdll amd64 in-cargo asm. |
| `libcportlib` | C | kernel/KD reuse; `cport-rust` exists (uefildr). |
| `libntoskrnl_vista` | C (~780) | **Rust** (`ntoskrnl-vista-rust`) for classpnp; kernel `.a` cohab. |
| `libcsq` | C (~440) | unchanged C. |
| `libarbiter` | C (~406) | unchanged C. |
| `libntlsalib` | C (~345) | unchanged C. |
| `libchkstk` | C/asm | hybrid: 11 reuse `.a`; i386 ntdll/kernel in-cargo asm; amd64 clang_rt. |
| `libkdcom` | C (~116) | **Rust** (`kdcom-rust`); kernel `.a` reuse. |
| `libwdmguid`/`libpoguid`/`libioevent` | C (~43 total) | unchanged; trivial GUID-statics flip. |
| `rtl-rust` (umbrella) | Rust | Rust, via `.rust_lib` (no cmake reuse) in kernel ×2 + ntdll. |

## Translation debt (C that's compiled in-cargo but NOT yet Rust)

Mechanism (2)/(3)/(4) C is off the prebuilt-`.a` hook but is still C — owed
translation for the "all-Rust, no-headers" endpoint:

- **msvcrtex CRT-startup — ~3,623 LOC C** (`sdk/lib/crt/startup/*.c` + a few
  `math/misc/stdio/stdlib/wstring` TUs), now `.msvcrtex_crt()`-compiled in-cargo
  for cabinet/mlang/lsass/winlogon/services/userinit. It's the mingw runtime
  entry glue: `DllMainCRTStartup`/`[w]WinMainCRTStartup`, `_CRT_INIT` +
  `.CRT$XI*/XC*` `_initterm` walk, `__security_init_cookie`,
  `_pei386_runtime_relocator`, TLS callbacks, `__main`, `_fltused`, i386/amd64
  `__chkstk`/`_alldiv`/`_aulldiv`. **Debt: port to Rust** (hard — linker-section
  initializer arrays + CRT-init semantics; lower priority, runtime-untested by
  smoke except via the spine exes).
- **libcntpr i386 float residual** — the no-SSE tail (`nexttowardf`, dead
  `exp2f`/`_invoke_matherr`, `winesup`/`wctype`) → **step 8**. (Possible tie-in:
  the cnt-rust x87-return/`floatbits.rs` softfloat shim.)
- **Kernel deep-C cohab floor** — the 16 `.a`s in `ntoskrnl-cargo`/`ntkrnlmp`.
  Each (arbiter/csq/ntlsalib/rossym/cmlib-C/libhal/setjmp/…) is its own arc.
  The single biggest remaining cmake-built-C surface.

## Forward path (priority)

1. **Pure-plumbing swaps (no translation):** `libcntpr_rust.a` →
   `.rust_lib("cnt-rust-lib")` everywhere; `libcport_rust.a` →
   `.rust_lib("cport-rust")`; `libmemcmp.a` drop (compiler-builtins-mem);
   GUID-only libs → `#[link_section]` statics. Mechanical, fleet-wide.
2. **pseh `dummy.c` c_shim** for the remaining 8 `libpseh.a` consumers.
3. **step 7 — zlib** (cabinet): adopt a pure-Rust zlib (`zlib-rs`/miniz_oxide).
4. **step 8 — libcntpr i386 float tail.**
5. **Kernel deep-C** (arbiter/csq/ntlsalib/rossym/…): per-lib translation arcs.
6. **msvcrtex CRT-startup → Rust** (lower priority; the hard CRT-init port).

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
