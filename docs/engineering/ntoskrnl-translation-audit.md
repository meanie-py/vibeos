# ntoskrnl C→Rust Translation Audit

> **Translation: ~100% of the ntoskrnl body (methodology below).** The shipping kernel links **zero ntoskrnl body C** — the only active `.c` in `ntos.cmake` is the 8-file, 1,037-LOC header-free CRT/SEH/asm-glue shim layer (each augments a Rust impl in a sibling rlib). Methodology: the active-source list in `ntos.cmake` contains 0 body `.c` (all subsystem dirs carry "# replaced by Rust" markers — 42 in `ntos.cmake`/`CMakeLists.txt`); `find sdk/lib/rust/ntoskrnl-rust/src -name '*.rs'` = **406 files / 451,190 LOC** vs an original C body of **268,513 LOC**; the irreducible glue is the 1,037-LOC shim layer + the in-cargo CRT floor. Honest caveat: "100%" = coverage at link (every body symbol resolves to Rust), not idiomation quality — the frontier is Stage-3 (Result/Drop-RAII) + runtime correctness.

> **Re-verified 2026-06-21** against CMakeLists/code — body still 100% Rust at link (zero active body `.c` in `ntos.cmake`). **UPDATE since 2026-06-14: the deep-C cohab libs are now Rust-flipped on ALL arches, not arm64-only.** `ntoskrnl-cargo/build.rs` now links `cport-rust`/`csq-rust`/`arbiter-rust`/`lsalib-rust`/`rossym-rust` **plus `cmlib-rust` and `rtl-rust`/`cnt-rust-lib`** as cargo-native `.rust_lib(...)` for i386/amd64/arm64 alike (lines ~310–327). The old x86 layout-sensitive BUGCHECK-0x50 gate is closed; the empty cmake `libcmlib.a`/`libcportlib.a` placeholders were dropped (cmlib `.a` removed 2026-06-20). So the five deep-C libs + cmlib are **Rust-bodied at link on all three arches** now. (`cmdata.c` remains FLIPPED — `config/cmdata.rs` active.)

## 2026-06-13 — MILESTONE: the ntoskrnl body is 100% Rust at link time

The shipping `ntoskrnl.exe` (and `ntkrnlmp.exe`) links **zero** ntoskrnl C
objects for its body. Verified mechanically:

- `ntoskrnl-cargo/build.rs` pulls in no `libntoskrnl.a` and no per-`.dir`
  `.obj` glob; `reactos-link` has no auto-include of cmake ntoskrnl objects.
  The only `static_libs` are *helper* archives (hal, kdcom, bootvid,
  ntoskrnl_vista) — genuine non-ntoskrnl-body libs — plus the irreducible
  CRT/SEH/asm floor compiled in-cargo (`c_from_compile_db` printf/scanf
  shells + i386 x87 `_CIxxx` math + amd64 SEH personality, `c_shims`).
  The former cohab-placeholder `.a`s (cmlib/cportlib/rossym/arbiter/csq/
  lsalib) are gone — those bodies are now cargo-native `.rust_lib` (see
  the all-arch deep-C nuance below).
- **Deep-C cohab nuance (UPDATED 2026-06-21 — now all-arch).** The five helper libs —
  `cportlib`/`csq`/`arbiter`/`lsalib`/`rossym` — are now built **cargo-native** via
  `.rust_lib("cport-rust"/"csq-rust"/"arbiter-rust"/"lsalib-rust"/"rossym-rust", …)`
  in `ntoskrnl-cargo/build.rs` (lines ~323–327) on **all three arches** — the
  arm64-only gate (and the `bacd1fc64d3` x86-ungate-then-revert dance) is gone.
  The cmake C `.a` placeholders were empty `rust_cohab_stub` TUs (0 real symbols),
  so dropping them was a no-op for the link. **`cmlib` is likewise now Rust on all
  arches** via `.rust_lib("cmlib-rust", …)` (line ~316); its cmake `libcmlib.a`
  (an empty placeholder since all 13 `cmlib/*.c` are `REMOVE_ITEM`'d) was **dropped
  2026-06-20**. `rtl-rust` + `cnt-rust-lib` are also cargo-native `.rust_lib` here.
  Net: every deep-C cohab lib is **Rust-bodied at link on i386/amd64/arm64.** The
  old layout-sensitive BUGCHECK-0x50 concern is resolved (guarded by the cmlib-rust
  no-prepopulate class-guard).
- Every core kernel symbol resolves to Rust: `NtCreateFile`/`IopParseDevice`
  → `iomgr/file.rs`, `MmCreateProcessAddressSpace` → `ARM3/procsup.rs`,
  `KeBugCheckEx` → `ps/psmgr.rs`, `KdpSafeReadMemory` → `kdbg/i386_dis.rs`.
- Subsystem dirs are 1:1: e.g. all 26 `io/iomgr/*.c` have `.rs` twins.

The **337 `.c` files still in `ntoskrnl/`** are inert reference copies (kept
per the never-delete-a-translation rule). The 5 basename "gaps" a naive scan
flags are all false positives: `priv.c`→`priv_.rs`, `i386-dis.c`→`i386_dis.rs`,
`kdmemsup.c`→`i386_dis.rs`, `cmboot.c`→`cmboot-rust` crate, and `pagepae.c`
**is not compiled by cmake at all** (ReactOS i386 builds non-PAE `page.c`→
`page.rs`; PAE is excluded from the build).

**Consequence:** there are no ntoskrnl "C leaves" left to eliminate from the
build. The remaining ntoskrnl translation frontier is *quality*, not coverage
— Stage-3 idiomation (Result/Drop-RAII over the mechanically-transliterated
unsafe Rust) and runtime correctness (the open amd64 taskbar BUGCHECK 0x50).

### Kernel-land status (other PEs), surveyed 2026-06-13 (spot-checked 2026-06-21)

Probed each `*-cargo/build.rs` for residual C (the same way as ntoskrnl).
"CRT floor" = printf variadic shells / i386 x87 math / SEH personality /
scanf trampoline / strsafe shim — the irreducible mingw-CRT residue, not
subsystem body C.

| PE / module        | Body              | Residual C in link                          | State |
|--------------------|-------------------|---------------------------------------------|-------|
| ntoskrnl / ntkrnlmp| ntoskrnl-rust     | CRT floor only (in-cargo)                   | ✅ 100% Rust body |
| smss               | smss-csrss-rust   | none                                        | ✅ |
| csrss / csrsrv     | smss-csrss-rust   | CRT floor only                              | ✅ |
| basesrv / winsrv   | winsrv-rust       | CRT floor + strsafe shim (placeholder .a)   | ✅ |
| lsass              | lsass-rust        | none                                        | ✅ |
| services / winlogon / userinit | *-rust| CRT floor only                              | ✅ |
| ntdll / ntdll_vista| ntdll-rust        | CRT floor (rtl placeholder .a)              | ✅ |
| HAL                | hal-rust          | 42/45 `.c` retired; only a generated empty stub live | ✅ ~100% |
| **win32k.sys**     | win32k-rust       | win32k cmake was flipped OFF (mainline landing); residual DirectDraw/D3D + a few gdi `.c` may remain — re-verify against `win32ss/CMakeLists.txt` (still has cohab markers) | 🔶 mostly Rust — see win32k audit for live-`.c` ledger |
| uefildr / rosload  | freeldr-rust      | boot RTL `.c` (bitmap/compress/crc32/image/largeint/random/unicode/mem) + `libmini_hal.a` + 2 asm | 🔶 boot-loader RTL not on rtl-rust; `systimer.S`-blocked |

The remaining kernel-mode *body* C lives outside ntoskrnl: residual
**win32k** GDI/DirectDraw (track in the win32k audit — `win32ss/CMakeLists.txt`
still carries cohab markers) and the **uefildr/rosload boot RTL** (the
`_MINIHAL_`/`systimer.S` asm-define blocker documented in the freeldr audit).
Everything inside the ntoskrnl body is Rust-bodied.

---

Last updated 2026-06-21. (History below is from the 2026-05-09 milestone session.)

1. **The header-drop arc completed for the 8 shims.** All 8 active C
   shim files are now header-free (no `<ntoskrnl.h>`, no internal
   kernel headers).
2. **`kernel_c_helpers.c` retired.** Rust now supplies the canonical
   `PsProcessInfoClass` / `PsThreadInfoClass` symbols, replacing the
   `ps_i.h` re-include trick that lived in C since the start of the port.
3. **The cargo-driven `ntoskrnl.exe` build PoC for i386 boots to desktop
   and passes 7/7 smoke.** Cargo can now drive the final link without
   cmake; the remaining 30+ static libs are still cmake-built but
   that's the next thread of work, not a blocker.
4. **The cargo PoC compiles its own C shim layer via `cc-rs`.** The
   8 header-free shim translation units (and `io_tables.c` while it
   still existed; later retired in milestone 7) are now produced by
   cc-rs from `build.rs`, not pulled from cmake's `.obj` directory.
   Boot-equivalent. Pre-existing latent bug surfaced and fixed:
   `ntkrnlmp/CMakeLists.txt` was missing `io_rust_shims.c` and
   `kdbg_rust_shims.c` from `NTKRNLMP_PCH_SKIP_SOURCE` — only
   mattered once livecd actually tried to rebuild ntkrnlmp.
5. **The cargo PoC compiles its own asm layer via `cc-rs`.** All 8
   ntoskrnl asm files (`zw.S`, `fastinterlck_asm.S`, `ioport.S`,
   `ctxswitch.S`, `trap.s`, `usercall_asm.S`, `zeropage.S`,
   `prefetch.S`) compile via `cc-rs` with the same kernel-mode flags
   + DEFINES + asm-include dirs cmake uses. The `.text` and `.data`
   sections of every cc-rs-built `.o` are byte-identical to cmake's;
   only `.debug_info` strings differ (path-dependent). 7/7 smoke pass.
6. **The cargo PoC compiles its own resource (`.rc.res`) via
   `windres`.** The new `compile_resource()` helper in `build.rs`
   shells out to `i686-w64-mingw32-windres` with the same
   preprocessor + 136 `LANGUAGE_*` defines + `KERNEL_DEFINES` +
   include chain cmake uses. Output is **byte-identical** to cmake's
   `.rc.res` (61,116 bytes, `cmp` exit 0). The cargo PoC no longer
   pulls a single `.obj`/`.res` file from cmake's output dir — only
   the `.def` file and the ~30 sibling static libs.
7. **`io_tables.c` retired — last `<ntoskrnl.h>` `#include` in the
   active build is gone.** The 8 `Iop*Operation*` tables are now
   `#[no_mangle] pub static [u8/u32; N]` in
   `iomgr/io_tables.rs`, computed from `core::mem::size_of::<>()`
   against ntkrnl-sys bindgen types and `FILE_READ_*`/`FILE_WRITE_*`
   constants from `ntkrnl_sys::constants`. `verify_sizes.py`'s
   compile-time assertions guarantee the bindgen sizes match the C
   structs byte-for-byte. cmake i386 7/7, amd64 7/7, arm64 5/5,
   cargo PoC 7/7. Cargo PoC binary grew 4 KB (one page) for the new
   `.data` statics.
8. **`ks386.inc` generated in-process by `build.rs`.** The new
   `generate_ks386_inc()` helper compiles
   `sdk/include/asm/genincdata.c` to a `.obj` via cc-rs (with the
   full kernel `-I` chain), then a small Rust COFF parser
   (`read_coff_section()`) walks the `.asmdef` section's
   `ASMGENDATA` records (64 bytes each: type, name[55], uint64
   value) and emits `ks386.inc` to `OUT_DIR/asm-gen/`. Output is
   **byte-identical** to cmake's `ks386.inc` (27,151 bytes,
   `diff` exit 0). Drops the cargo PoC's hardcoded reference to
   `build-i386/sdk/include/asm/`.
9. **`bugcodes.h` / `bugcodes.rc` / `bugcodes_MSG*.bin` generated
   in-process by `build.rs`.** `generate_bugcodes_mc()` does the
   ASCII → UTF-16LE conversion in pure Rust (8 LOC, byte-validated
   vs cmake's `native-utf16le` host tool), then shells out to
   `i686-w64-mingw32-windmc -u -A -b -h … -r …` to produce the
   header + .rc + .bin sidecars in `OUT_DIR/mc-gen/`. All three
   outputs are **byte-identical** to cmake's (h: 20,118 bytes; rc:
   210 bytes; bin: identical, all `cmp`/`diff` exit 0). The
   `apply_kernel_includes()` and `compile_resource()` helpers
   take a `mc_gen_dir: &Path` parameter; `KERNEL_INCLUDES` no
   longer contains `build-i386/sdk/include/reactos/mc`. Drops
   another cmake-generated dep.
10. **Surveyed `KERNEL_INCLUDES` and pruned 10 dead-weight paths.**
    Used `clang -H` on the cargo PoC's only three live compile
    inputs (asm, RC, `genincdata.c`) to dump the actual include
    hierarchy and cross-reference vs the path list. Dropped 3
    obviously-dead `build-i386/...` paths (`ntoskrnl`,
    `ntoskrnl/include`, `ntoskrnl/include/internal` — the latter two
    don't even exist on disk), 3 more empirically-dead `build-i386/...`
    paths (`sdk/include`, `sdk/include/dxsdk`,
    `sdk/include/reactos`), and 4 dead source-tree paths
    (`sdk/include/reactos/libs`, `sdk/include/winrt`,
    `sdk/include/dxsdk`, `sdk/include/reactos/drivers`,
    `sdk/lib/drivers/arbiter`, `sdk/lib/cmlib`, `""`). Kept the 12
    paths actually pulled. `ks386.inc` and `bugcodes.h` still
    byte-identical to cmake's after the trim; smoke 7/7 PASS.
11. **`hpp` self-hosted; the cargo PoC has ZERO `build-i386/*`
    build-time dependencies.** The 4 hpp-generated kernel headers
    (`ddk/ntddk.h`, `ddk/ntifs.h`, `ddk/wdm.h`, `psdk/ntdef.h`) are
    now produced in-process: `compile_hpp_host()` builds
    `sdk/tools/hpp/hpp.c` (631 LOC C) with the host compiler via
    `cc::Build::host()`, then `generate_xdk_headers()` runs the
    binary on the 4 `*.template.h` files (CWD set to
    `sdk/include/xdk/` so fragment includes resolve). All 4 outputs
    byte-identical to cmake's; `KERNEL_INCLUDES` no longer
    references any `build-i386/...` path.

    **End-to-end proof:** with `build-i386/sdk/include/{ddk,psdk}`
    renamed away and the cargo build cache nuked, the cargo PoC
    rebuilt the full 3.1 MB `ntoskrnl.exe` cleanly (host hpp +
    in-process xdk gen + windmc + genincdata + cc-rs C/asm + windres
    + final link). All generated files came out byte-identical to
    cmake's; smoke 7/7 PASS on the proof binary. The cargo PoC's
    only remaining cmake-built inputs are the ~30 sibling static
    libs (rtl, hal, cmlib, etc.) and the `.def` file (later retired
    in milestone 12) — no headers, no .obj files, nothing under
    `build-i386/sdk/include/...` or `build-i386/ntoskrnl/CMakeFiles/`.
16. **Remaining 6 asm files pre-processed out-of-band; `KERNEL_INCLUDES`
    deleted entirely.** New helper `tools/gen_preprocessed_asm.py` runs
    `clang -E -x assembler-with-cpp` on each of the 6 source-tree
    `.S/.s` files (`ex/zw.S`, `ex/i386/{fastinterlck_asm,ioport}.S`,
    `ke/i386/{ctxswitch,trap,usercall_asm}.S`) with the kernel `-I`
    chain, expanding `<asm.inc>`, `<ks386.inc>`,
    `<internal/i386/asmmacro.S>`, `<syscalls.inc>`, `<sysfuncs.h>`
    inline. The resulting fully-self-contained `.s` files (~180 KB
    total) are checked in to `data/asm/*-i386.s`. `build.rs`'s
    `compile_asm()` now compiles each via cc-rs with NO kernel `-I`
    chain — only `-isystem sdk/include/crt` + clang's resource dir.
    `KERNEL_INCLUDES` and `apply_kernel_includes()` are both deleted
    entirely. End-to-end proof: with `ntoskrnl/include` renamed away
    and the cargo cache nuked, the cargo PoC rebuilt cleanly; same
    3.08 MB binary; smoke 7/7 PASS in 52s. Re-run the python script
    when an asm source or its includes change.
15. **`ntoskrnl.rc` compile moved out-of-band; 3 RC-only source-tree
    `KERNEL_INCLUDES` paths dropped.** New helper script
    `tools/gen_ntoskrnl_res.py` (~150 LOC) runs `i686-w64-mingw32-windres`
    on `ntoskrnl/ntoskrnl.rc` with the kernel `-I` chain (using
    cmake-built `bugcodes.rc` from `build-i386`), and emits
    `sdk/lib/rust/ntoskrnl-cargo/data/ntoskrnl-i386.rc.res`
    byte-identical to cmake's (61,116 bytes). The cargo PoC's `build.rs`
    now reads that checked-in file via the new `stage_ntoskrnl_res()`
    helper — `compile_resource()` and the 136-entry `LANGUAGES` const
    are both deleted. Drops 3 `KERNEL_INCLUDES` entries: `ntoskrnl`
    (root, only RC pulled it for `inbv/bootanim.rc`),
    `sdk/include/psdk` (only RC after milestone 13), and
    `sdk/include/vcruntime` (same). `KERNEL_INCLUDES` shrinks 5 → 2
    (only `sdk/include/crt` and `ntoskrnl/include` remain). End-to-end
    proof: with `sdk/include/{psdk,vcruntime}` and `ntoskrnl/inbv`
    renamed away and the cargo cache nuked, the cargo PoC rebuilt
    cleanly; same 3.08 MB binary; smoke 7/7 PASS in 52s. Re-run the
    python script when version blocks (`common.ver`, `ntverp.h`) or
    bootanim resources change.
14. **`zeropage.S` + `prefetch.S` translated to `global_asm!`.** The two
    smallest of the 8 i386 asm files (32 + 44 LOC, neither `#include`s
    `<ks386.inc>` heavily) ported to `core::arch::global_asm!` blocks
    in `ntoskrnl-rust/src/ke/i386/zeropage_asm.rs` and
    `src/rtl/prefetch_asm.rs`. Symbol decoration preserved (fastcall
    `@KeZeroPages@8`, `@RtlPrefetchMemoryNonTemporal@8`); force-pull via
    `#[used]` static fn-pointers. Drops 2 of 8 entries from `build.rs`'s
    `asm_sources` list. Binary shrunk 3,117,056 → 3,080,192 bytes
    (-36 KB, more `--gc-sections` reach without the per-asm `.o` linkage).
    Smoke 7/7 PASS. Pattern proven for the remaining 6 asm files.
13. **`genincdata.c` regen moved out-of-band; 5 source-tree
    `KERNEL_INCLUDES` paths dropped.** New helper script
    `tools/gen_ks386_inc.py` (~200 LOC) compiles `genincdata.c` once
    via clang (using cmake-built `xdk-gen` headers as input), parses
    the `.asmdef` COFF section, and emits
    `sdk/lib/rust/ntoskrnl-cargo/data/ks386-i386.inc` byte-identical
    to cmake's. The cargo PoC's `build.rs` now reads that checked-in
    file via the new `stage_ks386_inc()` helper — `genincdata.c` is no
    longer compiled at build time. Drops 5 `KERNEL_INCLUDES` entries
    only `genincdata.c` was pulling: `sdk/include` (root),
    `sdk/include/ddk`, `sdk/include/ndk`, `sdk/include/reactos`,
    `sdk/lib/pseh/include`. `KERNEL_INCLUDES` shrinks 10 → 5.
    End-to-end proof: with `sdk/include/{ddk,ndk}` and
    `sdk/lib/pseh/include` renamed away and the cargo cache nuked,
    the cargo PoC rebuilt cleanly; same 3.1 MB binary; smoke 7/7 PASS.
    Re-run the python script when kernel struct layouts or the
    `ks{386,x}.template.h` files change (validate with
    `python3 tools/gen_ks386_inc.py --check`).
12. **`spec2def` self-hosted; `.def` file generated in-process.**
    The 1,585-line `ntoskrnl.def` (export table for the PE link) is
    now produced by `build.rs`: `compile_spec2def_host()` builds
    `sdk/tools/spec2def/spec2def.c` (1,784 LOC C) with the host
    compiler via `cc::Build::host()` (same pattern as the milestone-11
    `hpp` self-host), and `generate_def_file()` runs the binary on
    `ntoskrnl/ntoskrnl.spec` with cmake's exact flags
    (`--version=0x502 --dbg`). Output is **byte-identical** to
    cmake's (`cmp` exit 0, 1,585 lines). End-to-end proof: with
    `build-i386/ntoskrnl/ntoskrnl.def` renamed away and the cargo
    build cache nuked, the cargo PoC rebuilt cleanly; same 3.1 MB
    binary; smoke 7/7 PASS in 52s. The cargo PoC's only remaining
    cmake-built inputs are now the ~30 sibling static libs.

Treat any prior version of this document as suspect.

## Where we are

**~99.7% of the original ntoskrnl C body of work is now Rust.** The
active C in the default i386/amd64 build is **8 narrow shim files
totalling 1,037 LOC** — and **all 8 compile with only
`<stdarg.h>`/`<stddef.h>`/`<stdint.h>` + a small local typedef
block**, no `<ntoskrnl.h>` chain. **Zero source files in the active
build pull `<ntoskrnl.h>` anymore** as of 2026-05-08.

**Cargo-driven build for i386 works end-to-end.** `cargo +nightly
build -p ntoskrnl-cargo --target ntoskrnl-cargo/i686-kernel.json -Z
build-std=…` produces a 3.08 MB `ntoskrnl.exe` (vs. cmake's 4.5 MB)
that boots in QEMU and passes 7/7 smoke. As of milestone 16, the
PoC's compile inputs are entirely self-contained — no source-tree
kernel headers pulled at every build.

**Scope of this doc:** ntoskrnl source body only (the kernel's own
`.c`/`.rs` files + the 8 shims). The cargo PoC also links ~30
**cmake-built static libs** (`librtl.a`, `libhal.a`, `libcmlib.a`,
etc., totalling ~165K LOC of C) — those are tracked separately in
[`static-libs-translation-audit.md`](static-libs-translation-audit.md).
That's the next "elimination" axis.

> **Note (2026-06-21):** the ~30 cmake-built static-libs axis is now essentially CLOSED — rtl/libcntpr/hal/cmlib **and** the five deep-C cohab libs (cport/csq/arbiter/lsalib/rossym) are all linked cargo-native `.rust_lib` on all 3 arches (`ntoskrnl-cargo/build.rs`). The empty cmake placeholder `.a`s were dropped. Only the genuine non-body helper archives (hal/kdcom/bootvid/ntoskrnl_vista) + the in-cargo CRT floor remain C.

Counts (verified by `wc -l`):

| Bucket | LOC |
|---|---:|
| Original ntoskrnl C body of work (excl. shims) | 268,513 |
| Rust under `sdk/lib/rust/ntoskrnl-rust/src/` (406 files, 2026-06-21) | 451,190 |
| Active C in default i386/amd64 build (excl. shims) | 0 |
| C shim layer (8 files in `PCH_SKIP_SOURCE`) | 1,037 |
| C data-table sister files | 0 (was 1; `io_tables.c` retired 2026-05-08) |

Shim layer files (LOC, header status). All shims are header-free on
every architecture they're compiled for:

| File | LOC | Headers | Role |
|---|---:|---|---|
| `ke/ke_rust_shims.c` | 251 | header-free ✓ | minimal trampoline |
| `ntprintf_trampoline.c` | 170 | header-free ✓ | variadic `va_arg` pull |
| `io/iomgr/io_rust_shims.c` | 154 | header-free ✓ | probes + variadic |
| `mm/mm_rust_shims.c` | 152 | header-free ✓ | ARM64-only PTE/IRQL debug |
| `kdbg/kdbg_rust_shims.c` | 91 | header-free ✓ | `KdbpPrint` + `KdbPromptStr` |
| `lto_workaround.c` | 87 | header-free ✓ | LLVM i386 fat-LTO isolation |
| `variadic_shims.c` | 77 | header-free ✓ | `KdpDprintf` + `RtlStringCbPrintfA` |
| `mm_lock_outlines.c` | 55 | header-free ✓ | C `__FILE__`/`__LINE__` capture |

The historical "sister file" pattern (small `.c` that re-includes
`<ntoskrnl.h>` to materialize a C-side data table whose entries
depend on `sizeof(KernelStruct)`) is now empty. `io_tables.c` was
the last instance and got translated to Rust on 2026-05-08
(`iomgr/io_tables.rs`) using `core::mem::size_of::<>()` against
ntkrnl-sys bindgen types — verified equivalent by `verify_sizes.py`.

Rust LOC is higher than original C because (a) bindgen unions need
explicit accessor paths, (b) every output-pointer write goes through
`write_volatile` or an `opaque_ptr` launder, (c) the augment+isolated-
rlib pattern means C trampolines + Rust impls coexist for a shrinking
set of fns. Net work-done metric is closer to 1:1.

**Workspace crate count: 207** (`find sdk/lib/rust -maxdepth 2 -name Cargo.toml | wc -l`, 2026-06-21). Was 45 on 2026-05-09 — the growth is the kernel-land driver/DLL waves (cdrom/disk/classpnp/uniata/scsiport/atapi/storahci/storport, kernel32, netstack smoltcp, npfs/msfs, cdfs) factored into their own rlibs, not all ntoskrnl-body.

Smoke status (boot verdict is now the single `shell ready` marker on all 3 arches):
- **i386: GREEN** to shell ready (cmake build); cargo PoC binary also boots.
- **amd64: GREEN** to shell ready (cmake build). *Open, ntoskrnl-adjacent:* a
  non-deterministic amd64 taskbar BUGCHECK 0x50 (~1/3) post-`shell ready` via
  win32k taskbar child-control creation — runtime/logic, not an ntoskrnl
  coverage gap; tracked separately.
- **arm64: GREEN** — boots to the ReactOS desktop. The old post-boot fastfat
  `STATUS_INSUFFICIENT_RESOURCES` BUGCHECK is **RESOLVED** (Rust fastfat cohab
  flip landed on mainline 2026-06-15; arm64 reaches winlogon→explorer→taskbar).

## Survey of remaining `build-i386/` deps (2026-05-08)

Methodology: ran `clang -H` on the cargo PoC's three live compile
inputs (asm `trap.s`, RC `ntoskrnl.rc`, `genincdata.c`), unioned
the resulting include hierarchy, and cross-referenced vs the
`KERNEL_INCLUDES` list. After the survey:

| `KERNEL_INCLUDES` entry | Verdict | Notes |
|---|---|---|
| `sdk/include/crt` | LIVE | `-isystem` of mingw user-mode CRT replacement |
| ~~`ntoskrnl/include`~~ | DROPPED | only asm pulled it; gone in milestone 16 |
| ~~`ntoskrnl`~~ | DROPPED | only RC pulled it; gone in milestone 15 |
| ~~`sdk/include/psdk`~~ | DROPPED | only RC pulled it; gone in milestone 15 |
| ~~`sdk/include/vcruntime`~~ | DROPPED | only RC pulled it; gone in milestone 15 |
| ~~`sdk/include`~~ | DROPPED | only `genincdata.c` pulled it; gone in milestone 13 |
| ~~`sdk/include/ddk`~~ | DROPPED | only `genincdata.c` pulled it; gone in milestone 13 |
| ~~`sdk/include/ndk`~~ | DROPPED | only `genincdata.c` pulled it; gone in milestone 13 |
| ~~`sdk/include/reactos`~~ | DROPPED | only `genincdata.c` pulled it; gone in milestone 13 |
| ~~`sdk/lib/pseh/include`~~ | DROPPED | only `genincdata.c` pulled it; gone in milestone 13 |
| ~~`build-i386/sdk/include/psdk`~~ | DROPPED | replaced by in-process `xdk-gen/psdk` (milestone 11) |
| ~~`build-i386/sdk/include/ddk`~~ | DROPPED | replaced by in-process `xdk-gen/ddk` (milestone 11) |
| ~~`build-i386/ntoskrnl`~~ | DROPPED | only build artifacts (`.a`, `.def`), no headers |
| ~~`build-i386/ntoskrnl/include`~~ | DROPPED | dir doesn't exist on disk |
| ~~`build-i386/ntoskrnl/include/internal`~~ | DROPPED | dir doesn't exist on disk |
| ~~`build-i386/sdk/include`~~ | DROPPED | no direct hits (subdirs hit) |
| ~~`build-i386/sdk/include/dxsdk`~~ | DROPPED | no hits |
| ~~`build-i386/sdk/include/reactos`~~ | DROPPED | no hits |
| ~~`build-i386/sdk/include/reactos/mc`~~ | DROPPED | replaced by in-process `mc-gen/` (milestone 9) |
| ~~`sdk/include/dxsdk`~~ | DROPPED | no hits |
| ~~`sdk/include/winrt`~~ | DROPPED | no hits |
| ~~`sdk/include/reactos/libs`~~ | DROPPED | no hits |
| ~~`sdk/include/reactos/drivers`~~ | DROPPED | no hits |
| ~~`sdk/lib/drivers/arbiter`~~ | DROPPED | no hits |
| ~~`sdk/lib/cmlib`~~ | DROPPED | no hits |
| ~~`""` (project root)~~ | DROPPED | no hits |

**Zero `build-i386/...` paths remain in `KERNEL_INCLUDES`** as of
milestone 11. The hpp host tool (`sdk/tools/hpp/hpp.c`, 631 LOC C)
is now self-built from `build.rs` via `cc::Build::host()` and run
on the 4 templates to produce `xdk-gen/{ddk,psdk}/*.h` byte-
identical to cmake's. Empirically validated: the cargo PoC builds
clean with `build-i386/sdk/include/{ddk,psdk}` renamed away.

Live cargo-PoC-generated dirs (live but self-owned, all under OUT_DIR):
- `OUT_DIR/host-tools/{hpp,spec2def}` — host-built tools
- `OUT_DIR/xdk-gen/{ddk,psdk}` — 4 hpp-generated kernel headers
- `OUT_DIR/asm-gen` — `ks386.inc` (staged from checked-in
  `data/ks386-i386.inc`; regenerate with `tools/gen_ks386_inc.py`)
- `OUT_DIR/mc-gen` — `bugcodes.{h,rc,bin}`
- `OUT_DIR/spec-gen` — `ntoskrnl.def` + `ntoskrnl_stubs.c`
- `OUT_DIR/shims` — per-shim/per-asm `.o` files

### Source-tree kernel includes — consumer breakdown

After milestones 13, 14, 15, and 16, **`KERNEL_INCLUDES` is empty**
— the const itself was deleted. The cargo PoC's only `-I` chain is
now `-isystem sdk/include/crt` (mingw user-mode CRT replacement) +
clang's own resource dir, applied directly by each cc-rs caller.

| Path | Pulled by | Notes |
|---|:--:|---|
| `sdk/include/crt` | all | `-isystem` mingw user-mode CRT replacement (universal, comes free with `-nostdinc`) |

`sdk/include/crt` isn't part of the kernel header surface; it's the
mingw user-mode CRT replacement that comes for free with `cc-rs`'s
`-nostdinc` setup. It stays as long as we use clang.

The "kernel header chain" of cmake's original `-I` list (25 entries)
is now ENTIRELY ABSENT from every cargo build — replaced by:
- 4 hpp-generated kernel headers in `OUT_DIR/xdk-gen/{ddk,psdk}` (milestone 11)
- bugcodes.h in `OUT_DIR/mc-gen` (milestone 9)
- ks386.inc staged from checked-in `data/ks386-i386.inc` (milestone 13)
- ntoskrnl.rc.res staged from checked-in `data/ntoskrnl-i386.rc.res` (milestone 15)
- 6 pre-processed asm files staged from checked-in `data/asm/*-i386.s` (milestone 16)
- `.def` file generated by self-hosted spec2def (milestone 12)

Net result: cargo PoC compiles `ntoskrnl.exe` for i386 without
touching ANY source-tree kernel header.

## What's still C

| Bucket | LOC | Notes |
|---|---:|---|
| 8 shim files in `PCH_SKIP_SOURCE` | 1,037 | All header-free. See table above. |
| ~~`cmdata.c` Phase B holdouts~~ | ~~~150~~ | **CORRECTED 2026-06-14 — NO LONGER C.** `cmdata.c` is now commented out in `ntos.cmake` (line 37) and `config/cmdata.rs` (the full ~995-LOC translation incl. `MmConsumedPoolPercentage` + the registry-structure tables) is active in the build. The Phase-B holdout is closed. *(NB: the source-tree comment at `ntos.cmake:31` "cmdata.c kept in C" is itself now stale.)* |

**Translation candidates with no Rust translation in tree: zero.**
Every C file the default i386/amd64 build pulls in either has its Rust
replacement active, or is a shim that augments a Rust impl living in a
sibling rlib.

The historical `cache/` directory (NEWCC alternate cache subsystem) is
gone as of 2026-05-06; the Rust CC subsystem (`ntoskrnl-rust/src/cc/`)
is the only cache implementation now.

## Forward path

The strategic goal motivating shim minimization isn't the LOC count
itself — it's switching ntoskrnl from cmake to cargo as the build
driver. The ntoskrnl-source-body axis is **done**: every shim is
cc-rs-compilable header-free, every asm/RC/header is generated
in-process or staged from a checked-in artifact, and `KERNEL_INCLUDES`
is empty. The next layers all live outside the ntoskrnl source body:

### 1. Replace the ~30 cmake-built static libs

**Largest remaining C surface in the cargo PoC link line** — see
[`static-libs-translation-audit.md`](static-libs-translation-audit.md)
for the full ledger and per-lib forward path. Quick wins available:
move the 17 already-Rust sibling rlibs to cargo (build-system only),
translate the 3 GUID-only libs to tiny `#[link_section]` statics
(~50 LOC), then chip away at the 18 cmake-built C/asm libs
(`librtl.a`, `liblibcntpr.a`, `libhal.a`, `libcmlib.a`, …).

### 2. Cargo PoC build-system follow-ups

In rough priority order:

1. **Replicate per-arch.** Clone `i686-kernel.json` to
   `amd64-kernel.json` and `arm64-kernel.json`; the rest of the PoC
   structure is arch-agnostic.
2. **Audit `pefixup` / `fix_lld_forwarding.py`.** The smoke-passing
   PoC binary skipped both — strong sign that with the modern
   linker/Rust setup neither is strictly required for boot, but
   worth a careful audit (especially: does pefixup adjust the
   export table in ways that user-mode DLLs depend on?).

### 3. Re-probe the FIXME(amd64-misalign-231) workaround

`ex/exintrin.rs::ProbeForRead` carries a workaround that treats
non-{1,2,4,8,16} `Alignment` values as 1, citing a "consistent 0xE7
from a 16-byte UNICODE_STRING-style probe post-shell-ready on amd64."
This has the exact same root-cause smell as the just-fixed info-class
COUNT bug — past-end array reads producing junk Alignment values.
Worth re-probing whether the workaround is still load-bearing now
that `PS_PROCESS_INFO_CLASS_COUNT` and `PS_THREAD_INFO_CLASS_COUNT`
match the actual array sizes.

### Definition of done

`cargo build -p ntoskrnl --target <arch>` produces a working
`ntoskrnl.exe` for all 3 arches without invoking cmake, all 3 smoke
tests pass on the cargo-driven binary, `tools/scan_ud2.py --check` is
clean, and the cmake path can be removed (or kept as a parallel CI
check).

## Cargo PoC, as it stands

Lives at `sdk/lib/rust/ntoskrnl-cargo/`. Run via:

```bash
RUN_SMOKE=1 ./sdk/lib/rust/ntoskrnl-cargo/build-and-test.sh
```

Components:

- **`Cargo.toml`** — `[[bin]]` named `ntoskrnl`. No Rust deps —
  `kernel-rust` is `crate-type = ["staticlib"]` so build.rs links it
  by absolute path.
- **`i686-kernel.json`** — custom target spec forked from
  `i686-pc-windows-gnu` with the user-mode CRT bits stripped:
  `late-link-args` (no `-lmsvcrt` / `-lgcc_eh` / etc.),
  `pre-link-objects` and `post-link-objects` (no `rsbegin.o`/`rsend.o`),
  `crt-objects-fallback` removed. `rustc-abi` set to `x86-softfloat`
  so build-std's compiler_builtins doesn't insist on SSE.
- **`build.rs`** — invokes `cargo build -p kernel-rust` as a
  sub-step (with the same custom target), then runs five
  in-process generators (all byte-identical to cmake's outputs):
  1. `generate_xdk_headers()` — host-builds `sdk/tools/hpp/hpp.c`
     via `cc::Build::host()`, runs it on the 4 xdk templates
     (`wdm`, `ntddk`, `ntifs`, `ntdef`) producing the kernel
     headers in `OUT_DIR/xdk-gen/{ddk,psdk}/`.
  2. `generate_bugcodes_mc()` — UTF-16LE conversion + `windmc` →
     `bugcodes.{h,rc,bin}` in `OUT_DIR/mc-gen/`.
  3. `stage_ks386_inc()` — copies `data/ks386-i386.inc` (regen
     out-of-band via `tools/gen_ks386_inc.py`) into
     `OUT_DIR/asm-gen/ks386.inc`.
  4. `stage_ntoskrnl_res()` — copies `data/ntoskrnl-i386.rc.res`
     (regen out-of-band via `tools/gen_ntoskrnl_res.py`) for the
     link line.
  5. `generate_def_file()` — host-builds `sdk/tools/spec2def/spec2def.c`
     via `cc::Build::host()`, runs it on `ntoskrnl.spec` to produce
     `ntoskrnl.def`.

  Then compiles the 8 header-free C shim translation units and the
  6 pre-processed asm files (`data/asm/*-i386.s`, regen out-of-band
  via `tools/gen_preprocessed_asm.py`) via `cc-rs` with NO kernel
  `-I` chain, and emits `cargo:rustc-link-arg=` directives for: the
  resulting `libkernel_rust.a`, the per-shim/per-asm `.o` files, the
  staged `.res` resource, the generated `.def` file, the ~30
  cmake-built static libs, the clang builtins archive, and the
  kernel-mode PE-COFF link flags.

  The `apply_kernel_base()` helper mirrors cmake's `DEFINES = …`
  and `FLAGS = …` lines exactly. `KERNEL_INCLUDES` and
  `apply_kernel_includes()` are GONE as of milestone 16 — every
  compile input is self-contained or staged from a checked-in
  artifact, so cc-rs only needs `-isystem sdk/include/crt` + clang's
  resource dir.
- **`src/main.rs`** — `#![no_main]` shell with a stub
  `#[panic_handler]` and a `#[used]` anchor referencing `KiSystemStartup`.
- **`build-and-test.sh`** — wrapper script that refreshes cmake
  inputs, runs the cargo build, and (with `RUN_SMOKE=1`) swaps the
  cargo binary into `build-i386/` + reruns smoke.
- **Nightly toolchain required** for `-Z build-std`. The cmake-driven
  build still uses stable; `build.rs` sets `RUSTUP_TOOLCHAIN=nightly`
  for its sub-cargo invocation only.

Notable: the cargo binary is **smaller** (3.1 MB vs. cmake's 4.5 MB)
because `--gc-sections` runs more aggressively without the cmake-side
intermediate static libs accumulating unreferenced helpers.

## Active in Rust (default build)

Counts are *active modules* in `mod.rs`. As of 2026-05-08, every
Rust file in the tree is wired in — parked translations: **0**.

| Subsystem | Active mods | LOC active |
|---|---:|---:|
| KE (kernel) | 67 | ~54,100 |
| ARM3 (mm internals) | 25 | ~46,000 |
| iomgr | 26 | ~42,100 |
| EX (executive) | 32 | ~38,100 |
| SE (security) | 19 | ~35,200 |
| config (registry CM) | 25 | ~34,900 |
| pnpmgr | 20 (incl. arb) | ~26,400 |
| ps (process) | 21 | ~23,700 |
| ob (object) | 12 | ~22,600 |
| fsrtl | 18 | ~21,000 |
| kdbg (KDB) | 12 | ~13,800 |
| mm (mm internals) | 13 | ~11,900 |
| kd64 | 12 | ~12,100 |
| fstub | 5 | ~12,300 |
| cc (cache) | 8 | ~6,200 |
| dbgk / po / inbv / lpc / kd / wmi / vdm / arch dirs / rtl / vf / top-level | ~50 | ~30,000 |
| **TOTAL active (i386/amd64)** | **406 files** | **451,190** (2026-06-21) |

## Verified must-stay set

These have been re-probed and **proven** to require either C-side
execution or a separate compile unit. Not candidates for further
elimination until the underlying constraint moves.

- **Variadic exports** (`KdbpPrint`, `KdpDprintf`, `RtlStringCbPrintfA`,
  KDBG ext printers, `RtlUnicodeStringPrintf_4wZ_shim`,
  `ntprintf_trampoline.c`'s callees). Stable Rust limitation; cargo's
  `#[no_mangle] extern "C-variadic"` is unstable. Already isolated to
  minimal trampolines — fine.
- **`SepInitTokensShim`** (i386, in `lto_workaround.c`). LLVM fat-LTO
  miscompilation: `ud2` inserted after `@ObInitializeFastReference@8`.
  None of `opaque_ptr`, `compiler_fence(SeqCst)`, `core::hint::black_box`
  defeat it. C-side execution outside the Rust LTO scope is the only
  working workaround.
- **`KiSaveXState` / `KiRestoreXState`** (amd64, in `ke_rust_shims.c`).
  KVM bug: `xrstors64` faults on StateSaveArea pages that look valid
  via PTE self-referencing. Force `fxsave64`/`fxrstor64` instead.
- **`MmLockSectionSegment` trio** (`mm_lock_outlines.c`). Uses
  `__FILE__`/`__LINE__` for call-site recording in the underlying
  `_Mm*` primitives.
- **Rust dual-decoration constraint** (i386):
  `Ke386Get{Global,Local}DescriptorTable` has both an `extern "C"`
  cdecl caller (kdbg/kdb_cli.rs) and an `extern "system"` stdcall
  caller (ke/i386/cpu.rs) referencing the same Rust source name.
  Rust forbids two source-level fn definitions of the same name, so
  the cdecl impl lives bare-name in `ke-macros` and the stdcall
  `_X@4` decoration is supplied by a one-line C trampoline calling
  the cdecl `_rs_*` variant.

## Re-probing pattern (carry forward)

Many "Rust crashes / hangs" excuses in shim or DISABLED docs predate
the workspace `lto = false` + `codegen-units = 256` firewall, the
`opaque_ptr` laundering discipline, and `verify_abi.py`. **Re-probe
before treating any classification as a settled answer.**

When picking a parked file, deferred translation, or stay-in-C
classification to revive, the cheapest first step is often: re-enable
it, build, smoke. The bug may already be gone. Recent examples that
fell to a re-probe: every io captures(none) workaround, the trap-exit
chain ("multi-month" estimate that took hours), the `KiEndInterrupt`
"rustc 1.92 ABI bug," the four HAL stdcall bridges, the three Phase 1
substantive translations
(`KiAttachProcess`/`KiThreadStartup`/`Ki386InitializeTss`), and most
recently the `kernel_c_helpers.c` "alignment regression" — which
turned out not to be a Rust-translation issue at all but a pre-existing
latent count bug in `ps/query.rs` that the C-side trick was
accidentally masking.

The pattern: the heavy lifting (callees in Rust, layout structs already
defined, helpers as private fns next to the consumer) is usually
already done in earlier passes, leaving the actual port as a
one-session task. And: the "Rust regression" framing is sometimes
itself wrong — the C trick may have been hiding an upstream bug that
only surfaces after the swap.

## Common bug patterns

Track these — they keep recurring across translations.

1. **NT_ERROR vs NT_SUCCESS.** `NT_ERROR(x)` is `(x >> 30) == 3`, NOT
   `x < 0` (that's `!NT_SUCCESS(x)` and includes warnings).
2. **Bindgen anonymous union offsets.** `__bindgen_anon_1` can point
   to the wrong union member. KTHREAD::ApcState lives inside an anon
   union on every arch.
3. **Wrong enum constants.** Match arms using sequential ordinals
   instead of actual enum values (DeviceProperty, KeyValueInformationClass).
4. **Bindgen flexible-array indexing.** Bindgen renders trailing
   `Foo[]` as `Foo: [T; 1]`; indexing past 0 panics on bounds-check.
   Take the array base via `addr_of!` once, use `.add(i)` from there.
5. **`extern "C"` vs `extern "system"` ABI mismatch.** On i386, NTAPI
   functions are stdcall; declaring them `extern "C"` produces a cdecl
   call-site that blows the stack frame. amd64+arm64 link silently.
   Run `verify_abi.py` or `nm` first when ud2 shows up after a call.
6. **Bindgen enum max ≠ C array length.** When bindgen emits a Vista+
   `MaxXxxInfoClass` and the C array uses an NT 5.x-only re-include,
   they diverge. Hard-code the count to match the C array.
   Recent surface: `PS_{PROCESS,THREAD}_INFO_CLASS_COUNT` in
   `ps/query.rs` were transcribed from the Vista enum maxes (51/56)
   instead of NT 5.x values (47/28). The off-by-many indexed past
   the table end, reading garbage `AlignmentQUERY` bytes that tripped
   `ProbeForRead`'s misalign check. Bug was masked by C-side trailing
   bytes happening to be safe; surfaced when swapping the symbol
   supplier to Rust.
7. **Wide string encoding.** `b"..."` byte strings used as UTF-16. Must
   be `[u16; N]` or use the `wstr!` macro.
8. **Static lifetime on inline arrays.** Inline `[u16; N]` arrays as
   temporaries → dangling pointers; declare as `static`.
9. **Bitfield bit-position errors.** `RAMDISK_CREATE_OPTIONS` (and
   similar union/bitfield C definitions) need the bit layout matched
   exactly; collapsing to a single `: ULONG` field and setting `=1`
   sets bit 0, not the named bit. Use explicit `1 << N` constants.
10. **i386 has 3 decorations, not 1.** Before deleting a "dead" shim,
    grep for `_X` (cdecl), `_X@N` (stdcall), AND `@X@N` (fastcall).
11. **ud2 + folded success path.** Identical symptom for
    `captures(none)` AND `extern "C"` / `extern "system"` ABI
    mismatch on i386. Check ABI first via `nm` or `verify_abi.py` —
    1-second check vs. minutes of captures(none) defenses.
12. **PCH-skipped `static` re-include trick.** A C file that
    `#define static` + re-includes a header to externalize a `static`
    array (the original `kernel_c_helpers.c` pattern) only works when
    the file is PCH-SKIPPED. PCH-included siblings already pulled the
    header with the `static` qualifier, so the re-include creates a
    `static` vs non-static redefinition. Both former users of this
    pattern (`kernel_c_helpers.c` and `io_tables.c`) are now Rust;
    the pattern is gone from the active build. If a future shim
    needs it, either keep the file PCH-skipped (with `<ntoskrnl.h>`
    retained) or translate to Rust.

## Recipe for re-probing a shim

1. Read the in-source comment for the original symptom.
2. Grep for Rust callers across all 3 i386 decorations (`_X`,
   `_X@N`, `@X@N`).
3. If used: drop the C trampoline, declare the underlying Rust fn
   directly with the right ABI. Verify decoration via `nm`.
4. Build all 3 arches. `python3 tools/scan_ud2.py --check build-i386`
   is the canary — unexpected ud2 in a named function is the smoking
   gun.
5. Run smoke. Green on all 3 = ghost. Failure = either a real bug
   surfaced (ABI mismatch, missing volatile, etc.) or a genuine
   workaround that has to stay.

## Verification tools

| Tool | What it checks | Location |
|---|---|---|
| `verify_abi.py` | extern "C" vs extern "system" mismatch | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `lint_kernel.py` | Unprotected output pointer writes | `tools/lint_kernel.py` |
| `scan_ud2.py` | LLVM miscompilation (ud2 in named functions) | `tools/scan_ud2.py` |

`nm` decoration check (1 second, faster than any of the above):

```
i686-w64-mingw32-nm build-i386/ntoskrnl/ntoskrnl.exe | grep <FunctionName>
```

`_X` = cdecl, `_X@N` = stdcall, `@X@N` = fastcall. If the Rust caller's
extern decl produces a different decoration than the symbol's actual
decoration, that's the bug.

## What this audit doesn't measure

- **Boot correctness.** A line-count metric says nothing about whether
  the Rust version actually works under load. Smoke is the source of
  truth; this audit is just the LOC ledger backing the smoke story.
- **Per-arch differentials.** i386/amd64/arm64 all boot green to the desktop
  (`shell ready`). One open *runtime* issue: the non-deterministic amd64 taskbar
  BUGCHECK 0x50 (win32k child-control creation, post-shell-ready) — not an
  ntoskrnl coverage gap.

## How these numbers are computed

```bash
# Total ntoskrnl C body of work (excludes shims):
find ntoskrnl -name "*.c" \
  | grep -v -E '_rust_shims\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | xargs wc -l   # → 268,513 (2026-06-21)

# Active C in default build (excludes shims):
grep -E '^\s+\$\{REACTOS_SOURCE_DIR\}/ntoskrnl/.*\.c' ntoskrnl/ntos.cmake \
  | grep -v '^\s*#' \
  | grep -v -E '_rust_shims\.c|variadic_shims\.c|lto_workaround\.c|mm_lock_outlines\.c|ntprintf_trampoline\.c' \
  | sed 's|^\s*\${REACTOS_SOURCE_DIR}/||; s|).*||; s| .*||' \
  | sort -u | xargs wc -l   # → 0

# Shim layer (8 files, all PCH-skipped):
wc -l ntoskrnl/ke/ke_rust_shims.c \
      ntoskrnl/ntprintf_trampoline.c \
      ntoskrnl/io/iomgr/io_rust_shims.c \
      ntoskrnl/mm/mm_rust_shims.c \
      ntoskrnl/kdbg/kdbg_rust_shims.c \
      ntoskrnl/lto_workaround.c \
      ntoskrnl/variadic_shims.c \
      ntoskrnl/mm_lock_outlines.c   # → 1,037

# Rust LOC across ntoskrnl-rust/:
find sdk/lib/rust/ntoskrnl-rust/src -name "*.rs" | xargs wc -l   # → 451,190 (406 files, 2026-06-21)

# Workspace crate count:
find sdk/lib/rust -maxdepth 2 -name "Cargo.toml" | wc -l   # → 207 (2026-06-21)

# Parked Rust translations (file on disk, not in mod.rs):
for d in sdk/lib/rust/ntoskrnl-rust/src/*/mod.rs; do
  dir=$(dirname "$d")
  diff <(find "$dir" -maxdepth 1 -name "*.rs" ! -name mod.rs \
         | xargs -n1 basename | sed 's/\.rs$//' | sort -u) \
       <(grep -E '^pub mod ' "$d" | awk '{print $3}' | sed 's/;//' | sort -u) \
    | grep '^<' || true
done   # → empty
```
