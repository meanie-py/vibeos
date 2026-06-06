# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, Codex CLI, etc.) when working with code in this repository.

## Project Overview

This is a **multi-architecture port of ReactOS** — an open-source Windows-compatible operating system. Originally forked from `github.com/reactos/reactos` to add ARM64. The OS cross-compiles to **ARM64, amd64, and i386** PE-COFF with zero build failures on all three architectures. All three boot to a graphical desktop in QEMU. The active development branch is `continue-rust-port`; mainline is `master`.

## Translation Philosophy — Rust Is The Goal

**The project's goal is to translate ntoskrnl to Rust.** Getting the binary to build and boot is *important and necessary* but it is not the goal. Translation progress (lines of C → Rust) and boot status are independent axes.

**Rules:**
- **Never revert a Rust translation to C** to make a build link or a smoke test pass. Keep the Rust file. Fix it.
- **Never delete a translated Rust file** without explicit user approval. API translations cost real money; even broken output has value as a starting point for further work. Deletion is destructive and the work cannot be recovered without paying for the API call again.
- **Don't regenerate API translations without user approval.** Always ask before re-submitting a file that's already been translated. Cost matters.
- **Don't exclude a Rust translation from the build to dodge a regression.** Commenting out `pub mod X;` in `mod.rs` (or `lib.rs`) so the smoke tests pass while the C version is enabled is the same anti-pattern as reverting — it hides the regression from the smoke harness, which is *the diagnostic tool* for finding the bug.
- **A broken Rust translation in the tree, integrated and failing, is the correct state** when a regression is unsolved. The smoke harness reports it; you debug from there. The right response to a broken translation is (a) fix it in place, or (b) leave it broken with the failure visible in smoke and ask the user how to proceed — never (c) hide it from the build, and never (d) delete it.

These rules override the smoke-test imperative below when they conflict. If you have to choose between "smoke tests pass today" and "preserve in-progress Rust translation work as a debug signal," preserve the translation. A red smoke result that points at a real translation bug is more useful than a green smoke result that hides it.

## Smoke Tests — The North Star

The QEMU smoke tests (`tests/qemu/test_smoke.py`) are the **single source of truth** for whether the OS works. They boot each architecture in QEMU and verify the system reaches a graphical desktop — the boot marker is `shell ready` (i386/amd64) which proves the full chain (kernel → SMSS → csrss → win32k → winlogon → explorer.exe → desktop render) completed. ARM64 uses idle detection. A screenshot is always saved for visual inspection.

**Rules:**
- **Never weaken a smoke test to make it pass.** If a test fails, fix the OS, not the test.
- **Never lower boot markers** (e.g., from "explorer.exe created" to "ksecdd.sys") to work around regressions.
- **Run smoke tests after any kernel/subsystem change** that touches boot paths (MM, KE, EX, IO, SE, OB, PS, CC, FSRTL, win32k, HAL).
- **All 3 architectures must pass independently.**
- **Always check the screenshot** — the test saves `qemu/<arch>_smoke.png` for visual inspection.

## `vibeos-cli` — The Build/Check/Smoke/Debug Front Door

**Before building, booting, or debugging by hand, reach for `./vibeos-cli` (repo root).** It is the unified control plane over the build, the smoke harness, the lint gate, and the GDB toolkit — built specifically so agents stop reinventing per-arch build incantations, booting stale media, and flooding the transcript with serial logs. Full guide + per-tool motivation: **`tests/qemu/cli/README.md`** (read it once).

```bash
./vibeos-cli build  --arch i386 [--clean]   # warm by default; --clean = cold (nukes rust targets, proves cold-link)
./vibeos-cli smoke  --arch i386             # THE verdict — refuses to boot stale media (build first, or --allow-stale)
./vibeos-cli check                          # source lints, baseline-diff gate (0 NEW vs tools/lint-baseline/)
./vibeos-cli debug  start amd64 --gdb       # wraps qemu/*.sh (POSITIONAL arch); GDB over the real (UEFI) boot
./vibeos-cli ps                             # every QEMU across all worktrees + slot usage
```

Why it matters (the bugs it prevents):
- **No more false-green from stale artifacts.** `smoke` hard-fails on stale/missing media; warm builds are flagged because cargo incremental can mask regressions. Do a `--clean` build before trusting green or handing off.
- **No per-arch divergence.** `archcfg.py` is the single source of truth for build targets + boot media (x86 = `ninja livecd`; arm64 = staged set + `mkramdisk --refresh`). `boot` reuses smoke's exact arch-config so the fast path can't drift from the real test.
- **No token-flooding.** Stdout is a bounded summary; the durable verdict is `summary.json` + `full.log` under `/tmp/vibeos-cli/<ts>-<cmd>-<arch>/`. **Read `summary.json`, don't scrape stdout.**
- **Auto-triage on failure.** Classifies `RUST_PANIC` (file:line) / `BUGCHECK 0xNN` / `HANG_TIMEOUT` / `KNOWN_FLAKE`, symbolicates the faulting PC, captures a screenshot — no manual objdump round-trip.

**No default arch** (always pass `--arch`; `i386`/`amd64` alias to the UEFI boot configs). This is the front door; the raw `pytest tests/qemu/test_smoke.py` path and the `qemu/*.sh` toolkit still work underneath (see `tests/qemu/cli/README.md` → "Relationship to `qemu/`" for which layer to use when).

## Directory Layout

```
vibeos/                          # Git repo root (renamed from reactos/)
├── build-arm64/                 # CMake/Ninja build output (ARM64)
├── build-amd64/                 # CMake/Ninja build output (amd64)
├── build-i386/                  # CMake/Ninja build output (i386)
├── .build-tools/
│   └── llvm-mingw/              # Cross-compiler toolchain
├── qemu/                        # QEMU development environment
│   ├── run.sh                   # Launch QEMU (--arch arm64|amd64|i386, --gui, --debug)
│   ├── sym.sh                   # Symbolicate crash addresses
│   ├── mkramdisk.sh             # Create ARM64 ramdisk + boot images
│   ├── boot.img                 # FAT32 EFI boot disk (ARM64, gitignored)
│   ├── ramdisk.img              # FAT32 ramdisk with ReactOS files (ARM64, gitignored)
│   ├── QEMU_EFI.fd              # ARM64 UEFI firmware (EDK2, gitignored)
│   └── mouse_click.py           # Host-side mouse injection script
├── docs/
│   ├── product/                 # Roadmaps, status updates
│   └── engineering/             # Debug briefs, technical plans
├── ntoskrnl/                    # Kernel source
├── sdk/                         # SDK + Rust crates
├── tools/rustconv/              # Translation pipeline
└── tests/qemu/                  # Smoke test framework
```

## Build Commands

All build commands run from the respective build directory:

```bash
# ARM64
cd build-arm64
ninja              # Build everything
ninja ntoskrnl     # Just the kernel

# amd64
cd build-amd64
ninja              # Build everything
ninja bootcd       # Boot CD ISO (bootcd.iso)
ninja livecd       # Live CD ISO (livecd.iso)

# i386
cd build-i386
ninja              # Build everything
ninja livecd       # Live CD ISO (livecd.iso)
ninja bootcd       # Boot CD ISO (bootcd.iso, for 2-stage install)

# Common patterns
ninja -k0          # Continue past errors
ninja <target>     # Any target — use: ninja -t targets all | grep <name>
```

After modifying a `CMakeLists.txt` or `.cmake` file, re-run cmake before building:
```bash
cmake .            # Re-configure (from any build-*/ dir)
```

## Toolchain

- **Compiler:** llvm-mingw (clang 19, lld 19.1.7)
- **Unified toolchain file:** `toolchain-llvm-mingw.cmake` — builds all 3 architectures
- **Targets:** `aarch64-w64-mingw32`, `x86_64-w64-mingw32`, `i686-w64-mingw32`
- **External dependency:** `windmc` from Homebrew mingw-w64 (message compiler, not in llvm-mingw)
- **16-bit assembly:** Homebrew `i686-w64-mingw32-gcc` for boot sectors/frldr16/MVDM (Clang can't handle `.code16`)
- **Output format:** PE-COFF (Windows executables/DLLs), not ELF

## Key Technical Patterns

### DUMMY_PSEH (All Architectures with llvm-mingw)
PSEH3 (structured exception handling) doesn't work with Clang — ARM64 lacks x86 SEH, and Clang's ESP-relative locals break i386 PSEH filter invocation. All architectures use `USE_DUMMY_PSEH`, which makes `_SEH2_EXCEPT` handlers dead code. Key consequence: must validate pointers before dereferencing in SEH-protected paths, and software exceptions need to be swallowed in user-mode dispatch.

### ARM64 PE-COFF Relocations
Use `adrp + add :lo12: + ldr` (not `adrp + ldr :lo12:`) in assembly. The latter generates `LDST64_ABS_LO12_NC` relocations requiring 8-byte alignment which PE-COFF sections may not provide.

### LLD i386 Forwarder Decoration Bug
LLD prepends `_` to PE export forwarder targets on i386. Workaround in FreeLDR (peloader.c) and ntoskrnl (sysldr.c): strip leading `_` from forwarder DLL name.

### GAS Intel Syntax on i386
In GAS `.intel_syntax noprefix`, `mov eax, label` does a **memory load** (reads bytes at label), NOT an address load. Must use `lea eax, [label]` or `mov eax, offset label`.

### Build System Architecture Conditionals
Architecture checks use: `if(ARCH STREQUAL "arm64")` / `"amd64"` / `"i386"`. ARM64 uses UEFI-only boot. The `USE_DUMMY_PSEH` flag is set for all architectures when using llvm-mingw.

### llvm-rc Limitations
llvm-rc is stricter than MSVC's rc.exe:
- ACCELERATORS: Cannot combine `ASCII` type with `ALT`/`SHIFT`/`CONTROL` modifiers — use `VIRTKEY` instead
- DIALOGEX coordinates must fit signed 16-bit (-32768 to 32767)
- FILEVERSION/PRODUCTVERSION fields must fit 16 bits (max 65535)

## QEMU Development Workflow

```bash
# ARM64 — UEFI ramdisk boot
bash qemu/run.sh --arch arm64                  # Serial console
bash qemu/run.sh --arch arm64 --gui            # GUI with virtio input

# amd64 — LiveCD ISO boot
bash qemu/run.sh --arch amd64                  # Serial console

# i386 — LiveCD ISO boot
bash qemu/run.sh --arch i386                   # Serial console

# Common options
bash qemu/run.sh --arch arm64 --debug          # Pause for GDB on :1234
bash qemu/run.sh --arch arm64 --log            # Enable QEMU tracing
```

### ARM64 Ramdisk Update
```bash
mcopy -o -i qemu/ramdisk.img build-arm64/path/to/file.exe ::/reactos/system32/file.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
# WARNING: Do NOT mcopy uefildr.efi into boot.img AFTER ramdisk — corrupts FAT32
```

### Symbolication
```bash
bash qemu/sym.sh 0xC804 0x50FC    # Look up offsets from UEFI bootloader (uefildr) crash dumps
# Formula: runtime_addr - module_base + PE_ImageBase = PE VA
```

(For ntoskrnl symbol lookups use the kernel debug toolkit's `addr2sym.sh`/`sym2addr.sh` — see next section.)

## Kernel Debugging Infrastructure (qemu/start.sh, inspect.sh, run-until.sh)

**For triaging kernel bugs (BUGCHECKs, hangs, regressions), prefer this toolkit over `qemu/run.sh` + ad-hoc DbgPrint instrumentation.** It is agent-driven: every operation is a single batch command in, structured result out, no interactive GDB sessions to babysit. The scripts are `start.sh` / `stop.sh` / `wait-for.sh` / `tail.sh` / `status.sh` / `inspect.sh` (bt, symbolicate-pc, dump-thread, dump-trapframe, pool-tag, identify-object, triage-bugcheck) / `run-until.sh` / `ps` / `build-index.sh` / `sym2addr.sh` / `addr2sym.sh` / `preflight.sh`.

**Standard recipe (validated end-to-end):**
```
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
```
On the amd64 KiUnwaitThread BUGCHECK flake, this returns the stop code, exception code, faulting PC + symbol, full KTRAP_FRAME, and `identify-object` on the bogus pointer — all in one tool call.

Full toolkit reference + design: `docs/engineering/kernel-debugging-infra.md` and `qemu/README.md`. (`qemu/ps` shows every live QEMU across all worktrees + slot usage; QEMUs are capped via `VIBEOS_QEMU_MAX_CONCURRENT`/`VIBEOS_QEMU_MAX_LIFETIME`; run `qemu/build-index.sh <arch>` after each kernel build for offline sym lookup.)

## Test Suite

### Rust Host Tests (no QEMU needed)

```bash
cd sdk/lib/rust

# Unit tests — algorithm correctness, pack/unpack roundtrips
cargo test -p slist

# Kani model checking — exhaustive verification of state transitions
cargo kani -p slist

# Loom concurrency tests — explores all thread interleavings
cargo test -p slist-loom
```

### QEMU Kernel Tests (28 tests via serial dispatch loop)

```bash
# 1. Build test kernel
cd build-arm64 && cmake . -DREACTOS_TEST_KERNEL=ON && ninja ntoskrnl
cd ..

# 2. Update ramdisk
mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img

# 3. Run tests (boots QEMU, sends commands over serial, checks [TEST PASS])
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -v       # All 28 tests
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -k SList # Just SList
```

### QEMU Smoke Tests (boot verification — see "Smoke Tests — The North Star" above)

```bash
source .venv/bin/activate

# Full test (clean Rust rebuild + boot + static analysis) — ~5 min per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 600
pytest tests/qemu/test_smoke.py -v --arch amd64 --timeout 600

# Fast iteration (skip nuclear rebuild) — ~30s per arch
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300 --skip-clean-build

# Static analysis only (no QEMU boot)
pytest tests/qemu/test_smoke.py -v --arch i386 -k "sacred_primitives or no_unexpected_ud2" --skip-clean-build
```

**IMPORTANT:** Cargo's incremental cache can mask regressions. Always run WITHOUT `--skip-clean-build` for final verification before committing. The clean build nukes `rust-kernel-target/` to ensure deterministic results.

Smoke/CLI harness internals (`vm.py`/`conftest.py`/`serial_client.py`/`qmp_client.py`/`debugcon.py`, boot media, KVM acceleration, resource caps, debugcon boot tracing, test lifecycle): `tests/qemu/cli/README.md`.

## Rust in the Kernel

The Cargo workspace at `sdk/lib/rust/` holds Rust components that compile to PE-COFF static libraries (rlibs/staticlibs) and link into the cmake build. **All crates build for all 3 architectures unless explicitly noted (e.g., hal-rust is x86-only).**

```
sdk/lib/rust/
├── Cargo.toml              # Workspace (resolver v2). lto=false + codegen-units=256 firewall.
├── ntoskrnl-rust/          # The kernel translation. ~405 .rs files, ~447K LOC across all subsystems
│                           #   (ke, mm, ex, ps, ob, se, io, fsrtl, kd, kdbg, kd64, lpc, dbgk, po,
│                           #   wmi, vdm, rtl, cc, fstub, inbv, config, pnpmgr, etc.).
├── ntoskrnl-cargo/         # Cargo-driven ntoskrnl.exe link PoC (replaces cmake-driven link)
├── kernel-rust/            # staticlib umbrella that pulls in all sibling rlibs via extern crate
├── ntkrnl-sys/             # Two-layer FFI crate: hand-written + bindgen (see below)
├── hal-rust/               # x86 HAL Rust translation (i386 + amd64). minihal feature for FreeLDR
├── rtl-rust/               # Umbrella over 18 algorithm rlibs from sdk/lib/rtl/*
├── slist/                   # Lock-free SList (LDXP/CMPXCHG16B/CMPXCHG8B inline asm)
├── slist-loom/              # Loom concurrency tests for SList (std, not linked into kernel)
├── bitmap/                  # RtlBitmap operations
├── avltable/, splaytree/, generictable/  # Generic tree algorithms (used by mm, cm, etc.)
├── mm-avl/                  # Mm-flavor AVL on MMADDRESS_NODE (VadRoot etc.)
├── largeint/, time/, random/, crc32/, encode/, compress/, mem/, byteswap/,
│   utf8/, prefix/, luid/, access/, handle/, interlck/  # smaller algorithm crates
├── ke-inlines/, ke-macros/  # extracted ntoskrnl-rust ke/ helpers as separate rlibs
└── ir-output/              # rustc IR dumping helper
```

The structure isn't an exhaustive list — the workspace has 30+ members today. The principle is: anything that's pure-algorithm with a clean FFI surface gets factored into its own rlib so the workspace `lto=false` + `cgu=256` firewall keeps callers and callees in separate codegen units (which neutralizes LLVM's captures(none) optimization across the boundary).

### Rust Targets (GNU/LLVM, not MSVC)
- **ARM64:** `aarch64-pc-windows-gnullvm`
- **amd64:** `x86_64-pc-windows-gnu`
- **i386:** `i686-pc-windows-gnu`

### Reusable Cross-Platform Pattern
```
component/src/
  lib.rs        — FFI (#[no_mangle] extern "C", cfg for arch-specific conventions)
  algorithm.rs  — Pure logic (no unsafe, no asm, no cfg — Kani-verifiable)
  types.rs      — Shared types (cfg for platform layout variants)
  atomics.rs    — Platform asm (cfg(target_arch), common interface)
```

Principles:
1. **Algorithm is pure** — testable, Kani-verifiable, identical on all platforms
2. **Types use cfg at the boundary** — pack/unpack hides layout differences
3. **Atomics provide a uniform interface** — loop structure is identical per arch
4. **FFI is thin glue** — combines atomics + algorithm, minimal logic
5. **One crate, three targets** — same Cargo.toml for all

### Key Rust/FFI Gotchas
- **LLVM reserves rbx (amd64) and ebx (i386)** — use `xchg` workaround for CMPXCHG16B/CMPXCHG8B inline asm. **On amd64, the xchg MUST be 64-bit** (`xchg rbx, rbx_save`), not 32-bit (`xchg ebx, ebx_save:e`) — a 32-bit register write zero-extends into rbx, silently destroying the upper 32 bits of any caller-held pointer there. Cost a multi-hour BUGCHECK 0x50 debug session in 2026-05; see memory `feedback_cpuid_xchg_32bit_zext.md`.
- **`#[no_mangle] extern "stdcall"`** on i686-gnu correctly produces `_Name@N` decoration
- **`#[export_name]` gets double-decorated** on i686-gnu with stdcall — use `#[no_mangle]` instead
- **i386 SList Pop must be `global_asm!`** — `KiCheckForSListFault` checks exact instruction bytes (0x8B 0x18)
- **NTAPI vs cdecl ABI** matters per arch: i386 NTAPI = stdcall (`_Name@N`), amd64/arm64 NTAPI = cdecl (just `Name`). `extern "system"` resolves to the right thing per arch. Mismatched `extern "C"` vs `extern "system"` declarations across rlibs link silently on amd64/arm64 but break i386. Use `tools/rustconv/rustconv/verify_abi.py` to detect.
- **Cross-DLL data imports** (HAL referencing kernel-data exports like `KdDebuggerNotPresent`, `KeServiceDescriptorTable`): need `#[link(name = "ntoskrnl", kind = "raw-dylib")]` on the extern static block. `postprocess_bindgen.py --hal` adds this automatically for the 41 affected externs.

### ntkrnl-sys — Kernel FFI Declarations Crate (overview)

Dual-layer FFI crate at `sdk/lib/rust/ntkrnl-sys/` providing kernel type and function declarations:
- **Layer 1 — hand-written modules** (`ntkrnl_sys::base`, `::ke`, `::mm`, `::ex`, `::ps`, `::ob`, `::rtl`, `::interlocked`): curated declarations, inline functions, macros, and convenience wrappers bindgen can't generate.
- **Layer 2 — bindgen kernel module** (`ntkrnl_sys::ddk::*`): auto-generated from `ntoskrnl.h` + `mm/ARM3/miarm.h`, covering all internal subsystems (~3,300 extern fns, ~1,850 structs, ~4,400 type aliases per arch, with correct i386 fastcall/stdcall conventions). ~1,500 compile-time `size_of` assertions per arch verify C layout.

Full reference (directory tree + line counts, size-verification `verify_sizes.py`, bindgen regeneration via `generate_bindings.sh`, limitations), plus the CMake build-integration mechanics and the "Adding New Crates" steps: `docs/engineering/rust-in-kernel.md`.

### Cohabitation pattern (HAL)

For HAL Rust translation, each .c → .rs swap is independently revertible: the C file gets commented out in the per-flavor cmake (`hal/halx86/{generic,legacy,pic,acpi,apic}.cmake`) with a `# replaced by Rust (path)` line, and the Rust file's `#[no_mangle]` symbol takes over. `--allow-multiple-definition` allows side-by-side C+Rust during the swap-in phase; lld first-match wins. See `docs/engineering/hal-translation-audit.md` for status and per-file mapping.

## rustconv — C-to-Rust Translation Pipeline

API-driven pipeline that translates ntoskrnl and HAL C files to Rust using LLM (Sonnet batch API by default; also supports Haiku, Opus, Gemini Pro/Flash). Located at `tools/rustconv/`.

Full pipeline internals, post-processor rules, probe/Stage-3 design, and the Codex cross-model path: `docs/engineering/rustconv-pipeline.md`.

### CLI

```bash
source .venv/bin/activate

python -m rustconv scan ntoskrnl/ex/uuid.c            # Parse, print analysis
python -m rustconv resolve ntoskrnl/ex/uuid.c          # Resolve deps → import manifest
python -m rustconv prompt ntoskrnl/ex/uuid.c           # Generate prompt to stdout
python -m rustconv estimate ntoskrnl/ob/               # Token count + cost estimate
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs # Single-file API call
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble  # Translate + write to crate
python -m rustconv translate ntoskrnl/ob/file.c        # Single-file synchronous (better for >1000 LOC; batch silently stubs lazy fns)
python -m rustconv batch ntoskrnl/ob/ -m sonnet        # Batch entire subsystem (smaller files only)
python -m rustconv probe-migrate se/sqos.rs            # Stage 2: ProbeForRead/Write → Result<T>
python -m rustconv trace-catalog                        # Generate debugcon trace catalog from inline asm
python -m rustconv check                                # cargo check ×3 targets
```

> **⚠️ `translate`/`submit` SPEND REAL MONEY — read this before running them.** Multiple agents have wasted >$100 by killing translations mid-run and losing the output. The rules:
> - **NEVER Ctrl-C / kill a `translate` or `submit` mid-call to "adjust something."** Let it finish. `translate` makes one paid call **per function** — kill it at function 35/40 and you re-pay for all 35 on the retry. If you must change something, wait for it to finish, *then* re-run.
> - **Output is now saved as it streams** to `tools/rustconv/.responses/<ts>-<file>/` (gitignored, in-repo, survives kills/reboots). Each function's Rust lands there the instant it returns; the in-flight one is preserved as a `.partial`. That dir is the **canonical copy** — never write translations to `/tmp` (`-o /tmp/...` is flagged; the durable copy in `.responses/` exists regardless). Do not delete a `.responses/` run dir that holds unintegrated work.
> - **Cost estimates are a FLOOR, not a quote.** `estimate` shows the single-prompt cost; `translate` re-sends context per function, so the real bill is routinely **2–5×** higher, and `-m opus` is **full sync price ($5/$25 per 1M), not the batch Sonnet price** in the table below. Watch the live `[running $X.XX]` counter the command prints — that's the truth. Budget with the "sync @5x" column from `estimate`.
> - **Don't re-translate a file that already has output** (in `.responses/` or already integrated) without explicit user approval — you're re-paying for work that exists.

Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`, or full model ID.

`rustconv translate` accepts HAL paths and is HAL-aware (separate `system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is ~70% useful for HAL — review before integrating (memory `feedback_rustconv_hal_target.md`).

The pipeline has multiple stages: Stage 1 (C → Rust, `submit`/`batch`), Stage 2 (probe migration, `probe-migrate` — `ProbeForRead`/`Write` → `Result<T>`; SE+OB done), Stage 3 (Result+Drop-RAII idiomation, planned). See `docs/engineering/rustconv-pipeline.md` for the per-stage design.

### Cost (Sonnet **Batch** API — the cheapest path, NOT what you pay synchronously)

These are **batch Sonnet** figures (50% off, async). The synchronous `translate`/`submit`
path you actually use pays **full list price**, and `-m opus` is **~3.3× the Sonnet rate**
($5/$25 vs $3/$15 per 1M) on top of losing the batch discount — so a "Full ntoskrnl"
Opus-synchronous run with per-function fan-out is an order of magnitude past the bottom row.
Always sanity-check against `rustconv estimate`'s "sync @5x" column and the live cost counter.

| Scope | Est. Cost (batch Sonnet, floor) |
|-------|-----------|
| Single file | ~$0.03 |
| Subsystem (11 files) | ~$0.30 |
| Full ntoskrnl (393 files) | ~$8.70 |
| 5 iterations | ~$43.50 |

### Static Analysis Tools

```bash
# Check for LLVM miscompilation (ud2 in named functions)
python3 tools/scan_ud2.py build-i386              # Report all ud2
python3 tools/scan_ud2.py --check build-i386      # Exit 1 if unexpected ud2

# Check for unprotected output pointer writes (captures(none) vulnerable)
python3 tools/lint_kernel.py                       # Report all violations
python3 tools/lint_kernel.py --fix                 # Show suggested fixes
python3 tools/lint_kernel.py path/to/file.rs       # Single file

# Check sacred primitives survive optimization (KfRaiseIrql, KeAcquireSpinLock)
bash tools/verify_sacred_primitives.sh build-i386

# Check constant values against C headers
.venv/bin/python tools/rustconv/rustconv/verify_constants.py                    # Check all
.venv/bin/python tools/rustconv/rustconv/verify_constants.py --fix              # Auto-fix

# Check ABI mismatches
.venv/bin/python tools/rustconv/rustconv/verify_ffi.py
```

`scan_ud2.py` finds `ud2` (0f 0b) in named functions — LLVM proved the body has UB (almost always captures(none) or an ABI mismatch); it's wired into smoke as `test_no_unexpected_ud2`. `lint_kernel.py` finds `*OutputParam = value` writes in `#[no_mangle]` functions that skip `write_volatile` (a captures(none) target) before compilation. (The smoke harness also nukes `rust-kernel-target/` per session unless `--skip-clean-build`, runs the ud2 scan pre-boot, and auto-symbolicates BSODs — see `tests/qemu/cli/README.md`.)

## ARM64 Specifics

### Memory Layout
```
User space:     0x0000000000000000 - 0x0000FFFFFFFFFFFF
Kernel (KSEG0): 0xFFFFF80000000000
HAL VA:         0xFFFFFFFFFFC00000 - 0xFFFFFFFFFFFFFFFF
```

### QEMU virt
RAM at physical 0x40000000 (1GB). PL011 UART at 0x09000000. Serial via Unix socket at `qemu/serial.sock`.

### UART Trace Pattern (kernel debugging)
```c
extern volatile PVOID Arm64UartKernelBase;
volatile ULONG *u = (volatile ULONG *)Arm64UartKernelBase;
if (u) { while (*msg) *u = (ULONG)*msg++; }
```

### ESR Exception Classes
0x25=data abort, 0x21=instr abort, 0x3C=brk, 0x15=SVC

## Target Hardware

All development and CI happens in **QEMU**: `virt` machine (ARM64), `q35` (amd64), or standard PC (i386). i386 and amd64 use KVM acceleration when `/dev/kvm` is available (WSL2 supports it). Real-hardware boot (e.g., Raspberry Pi for ARM64) is an aspirational target, not a current one.

## Notes
Don't ask the user to run commands or install things after they have told you to debug something, just debug it.

Use the API keys in .env when necessary. Use AI and the process we've developed in rustconv to automate the process of doing translations in bulk. If you have to work on multiple files and there are signficant (50+) number of edits, and it would be good learnings to incorporate into future refactors, then we should automate it. When we already have the AI output of the c->rust or rust->rust translation and you are just integrating it, then go ahead and do it manually.