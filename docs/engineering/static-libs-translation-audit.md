# Static-Lib Translation Audit (cargo PoC)

Created 2026-05-09. Updated 2026-05-09 (rtl-rust umbrella consolidation).
Sister doc to `ntoskrnl-translation-audit.md`.

> Scope: kernel-linked **static libs**. The `sdk/lib/drivers/` helper libs
> (csq, arbiter, ntoskrnl_vista) and boot bits (bootvid, kdcom) live here because
> they are libraries, not `.sys` drivers. The standalone `drivers/` **driver**
> fleet (pci, pcix, cdrom, classpnp, input, …) is in `driver-translation-audit.md`.

The cargo PoC's `ntoskrnl.exe` link still consumes a fistful of cmake-built
`.a` files. After milestone 16 closed the source-tree-kernel-header
chain, these are **the largest remaining C surface** the cargo PoC
touches — collectively much larger than the ntoskrnl source body.

This doc is the ledger for that surface.

## Where we are

The cargo PoC's link line (in build.rs's `static_libs` array) lists
**18 archives** in cmake link order, inside a `--start-group/--end-group`
pair. Of those:

- **17 are still pure C/asm** (cmake-built from C sources)
- **1 is the `rtl-rust` umbrella staticlib** (`sdk/lib/rust/rtl-rust/`),
  which combines 18 RTL-flavor algorithm rlibs into one `.a` file.
  Before the consolidation, those 18 each shipped as their own
  `libreactos_X.a`; now they are sub-rlibs of one umbrella.

Total cmake-built C/asm LOC under the listed prefixes: **~165,000**.
For context, the ntoskrnl source body is 268K LOC C, so the static
libs are ~62% of the kernel-binary's C surface.

## C/asm libs (18)

Sorted by LOC. Numbers from `wc -l` over `*.c` `*.S` `*.s` under each
prefix as of 2026-05-09; some prefixes contain code for other modules
too (so this is an upper bound on what's linked into ntoskrnl.exe).

| Lib | Path | LOC | Role |
|---|---|---:|---|
| `librtl.a` | `sdk/lib/rtl` | ~59,620 | Run-Time Library (string, list, registry, COM, security primitives) |
| `liblibcntpr.a` | `sdk/lib/crt` | ~59,058 | Kernel-mode CRT subset (string, math, locale) |
| `libhal.a` | `hal/halx86` | ~24,248 | HAL (interrupt controller, timers, ACPI, CPU init) |
| `libcmlib.a` | `sdk/lib/cmlib` | ~8,819 | Configuration Manager (registry hive parser, shared with freeldr) |
| `libbootvid.a` | `drivers/base/bootvid` | ~3,741 | Boot video driver (early VGA framebuffer) |
| `librossym.a` | `sdk/lib/rossym` | ~3,083 | ROSSYM symbol-file format reader (debugger support) |
| `libpseh.a` | `sdk/lib/pseh` | ~2,599 | Pseudo-SEH (PSEH3 macros — most ntoskrnl uses DUMMY_PSEH instead) |
| `libsetjmp.a` | `sdk/lib/vcruntime` | ~1,255 | `setjmp`/`longjmp` for kernel-mode exception unwind |
| `libcportlib.a` | `sdk/lib/cportlib` | ~915 | KD/serial port library |
| `libntoskrnl_vista.a` | `sdk/lib/drivers/ntoskrnl_vista` | ~780 | Vista+ kernel APIs not in ntoskrnl proper (worker threads, etc.) |
| `libcsq.a` | `sdk/lib/drivers/csq` | ~440 | Cancel-Safe Queue helper |
| `libarbiter.a` | `sdk/lib/drivers/arbiter` | ~406 | Hardware-resource arbiter |
| `libntlsalib.a` | `sdk/lib/lsalib` | ~345 | LSA secure-process API helpers |
| `libchkstk.a` | `sdk/lib/crt` (chkstk only) | small | Stack probe for large frame allocations |
| `libkdcom.a` | `drivers/base/kdrosdbg` | ~116 | KD COM-port transport |
| `libwdmguid.a` | `sdk/lib/wdmguid` | ~20 | WDM GUID definitions |
| `libpoguid.a` | `sdk/lib/poguid` | ~14 | Power Manager GUIDs |
| `libioevent.a` | `sdk/lib/ioevent` | ~9 | IO event GUIDs |

GUID-only libs (`wdmguid`, `poguid`, `ioevent`, parts of others)
are trivially `.rs` `static`s with `#[link_section]`.

## Already-Rust libs (1 umbrella + 1 standalone)

The 18 algorithm rlibs that used to ship as 18 separate `libreactos_X.a`
files (avltable, bitmap, slist, compress, crc32, random, encode,
largeint, access, byteswap, interlck, luid, prefix, mem, handle, time,
splaytree, generictable) are now consolidated under one umbrella:

- `sdk/lib/rust/rtl-rust/` — umbrella staticlib (`librtl_rust.a`).
  Sub-crates flipped from `crate-type = ["staticlib"]` to `["rlib"]`.
  Single `#[panic_handler]` lives here (with debugcon emission) instead
  of 18 trivial `loop {}` copies.
- `sdk/lib/rust/utf8/` — standalone staticlib (`libutf8.a`), kept
  separate because it links into `librtl_vista.a` (consumed by the
  small `ntdll_vista.dll` userland module that does not also link
  `librtl.a`; folding it into the umbrella would bloat that DLL).

For the cargo PoC build these are cargo-buildable too — replacing the
remaining cmake glue is a build-system job, not a translation job.

There is also a third Rust crate under the cmake glue:

- `sdk/lib/rust/pool/` — Rust-side pool helpers, consumed as a Rust
  `[dependencies]` rlib by `kernel-rust` and `ntoskrnl-rust`. Not on
  the cmake link line.

## Forward path

In rough priority order:

### 1. Move the rtl-rust + utf8 + pool Rust crates to cargo (build-system only — no translation)

These are already Rust. Hooking them into `ntoskrnl-cargo`'s build
graph as Cargo deps (instead of consuming the cmake-built `.a` files)
drops the last Rust entry from `static_libs[]` and removes that part
of the cmake hand-off. Mechanical. **Step done as of 2026-05-09:**
the 18 sub-rlibs were consolidated under `rtl-rust` so this step is
now "wire one Cargo dep" instead of eighteen.

### 2. GUID-only libs → Rust statics (~50 LOC total)

`libwdmguid.a` (20 LOC), `libpoguid.a` (14 LOC), `libioevent.a` (9 LOC)
are just GUID byte-pattern definitions. Moving to `#[link_section]`
Rust statics in a tiny crate is trivial. Drops 3 more entries.

### 3. Translate `liblibcntpr.a` (~59 K LOC C — the biggest one)

The kernel-mode CRT subset. Standard C library functions (string,
math, locale) — straightforward to translate piecewise. Many already
have Rust equivalents in `core` or `compiler-builtins`; the kernel
wrappers are mostly thin glue.

### 4. Translate `librtl.a` (~59 K LOC C)

Largest non-CRT C surface. Strings, lists, security descriptors,
registry helpers. Already has Rust versions of many components
(SList, Bitmap, AVL trees, etc. in the 17 sibling rlibs above).
The remaining C is mostly hand-rolled algorithms — translatable
piecewise.

> **This is its own active translation track**, tracked in detail in
> **`rtl-translation-audit.md`** (the `rtl-strings` cabi waves: the
> `Rtl*` API bodies — strings/sid/acl/sd/env/process/path/…). The
> algorithm-crate half (the `rtl-rust` umbrella in "Already-Rust libs"
> above) is the boundary: this doc owns the cargo-PoC link surface; the
> rtl audit owns the librtl cohabitation flip status.

### 5. Translate `libhal.a` (~24 K LOC C)

Hardware Abstraction Layer — i386 interrupt controller, ACPI,
APIC, timers, CPU bring-up. Most arch-specific. Higher risk than
the userspace-flavor libs because it's deeply tied to MMIO/PIO.

### 6. Smaller subsystems (libcmlib, librossym, libbootvid, ...)

Each is its own translation arc; some (`libcmlib`) overlap with
work already done in `ntoskrnl-rust/src/config/`.

## Definition of done

`cargo build -p ntoskrnl --target i686-kernel.json` produces a
working `ntoskrnl.exe` for i386 (then amd64 + arm64) **with no
cmake invocation anywhere in the pipeline** and no cmake-built
`.a` files on the link line. At that point, the entire kernel
binary's source-of-truth is `sdk/lib/rust/` plus the 8 narrow
C shims.

## How these numbers are computed

```bash
# Per-lib LOC (upper bound — some prefixes contain non-ntoskrnl code):
for d in sdk/lib/{cportlib,cmlib,lsalib,rtl,rossym,crt,vcruntime,wdmguid,poguid,ioevent,pseh} \
         sdk/lib/drivers/{csq,arbiter,ntoskrnl_vista} \
         hal/halx86 drivers/base/{kdrosdbg,bootvid}; do
    find $d \( -name "*.c" -o -name "*.S" -o -name "*.s" \) \
      | xargs wc -l | tail -1 | awk -v d=$d '{print d, $1}'
done
```

## What this audit doesn't cover

- The 8 C shim files in `ntoskrnl/` itself — those are tracked in
  `ntoskrnl-translation-audit.md`'s shim ledger.
- Build tooling C (`hpp.c`, `spec2def.c`, `genincdata.c`,
  `ntoskrnl.rc`, the 6 source-tree `.S` files) — those run
  out-of-band or are pre-processed once. Not in the link line.
- llvm-mingw's `libclang_rt.builtins-i386.a` — third-party
  compiler runtime, comes free with clang.
- User-mode dependents that link against `ntoskrnl.exe` exports —
  separate translation surface.
