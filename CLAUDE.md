# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **multi-architecture port of ReactOS** — an open-source Windows-compatible operating system. The source is a fork of `github.com/reactos/reactos` on the `arm64-port` branch. The OS cross-compiles to **ARM64, amd64, and i386** PE-COFF with zero build failures on all three architectures. All three boot to a graphical desktop in QEMU.

## Directory Layout

```
os-playground/
├── reactos/                    # ReactOS source tree (git repo, arm64-port branch)
├── build-arm64/                # CMake/Ninja build output (ARM64)
├── build-amd64/                # CMake/Ninja build output (amd64)
├── build-i386/                 # CMake/Ninja build output (i386)
├── qemu/                       # QEMU development environment
│   ├── run.sh                  # Launch QEMU (--arch arm64|amd64|i386, --gui, --debug)
│   ├── sym.sh                  # Symbolicate crash addresses
│   ├── mkramdisk.sh            # Create ARM64 ramdisk + boot images
│   ├── boot.img                # FAT32 EFI boot disk (ARM64)
│   ├── ramdisk.img             # FAT32 ramdisk with ReactOS files (ARM64)
│   ├── QEMU_EFI.fd             # ARM64 UEFI firmware (EDK2)
│   └── mouse_click.py          # Host-side mouse injection script
└── llvm-mingw-20250114-ucrt-macos-universal/  # Cross-compiler toolchain
```

## Build Commands

All build commands run from the respective build directory:

```bash
# ARM64
cd /Users/lambdax/Projects/os-playground/build-arm64
ninja              # Build everything
ninja ntoskrnl     # Just the kernel

# amd64
cd /Users/lambdax/Projects/os-playground/build-amd64
ninja              # Build everything
ninja bootcd       # Boot CD ISO (bootcd.iso)
ninja livecd       # Live CD ISO (livecd.iso)

# i386
cd /Users/lambdax/Projects/os-playground/build-i386
ninja              # Build everything
ninja bootcd       # Boot CD ISO

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
- **Unified toolchain file:** `reactos/toolchain-llvm-mingw.cmake` — builds all 3 architectures from macOS
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

# i386 — BootCD ISO (2-stage install: CD→HDD, then HDD boot)
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
bash qemu/sym.sh 0xC804 0x50FC    # Look up offsets from crash dumps
# Formula: runtime_addr - module_base + PE_ImageBase = PE VA
```

## Test Suite

### Rust Host Tests (no QEMU needed)

```bash
cd reactos/sdk/lib/rust

# Unit tests — algorithm correctness, pack/unpack roundtrips (20 tests)
cargo test -p reactos-slist

# Kani model checking — exhaustive verification of state transitions (9 proofs)
cargo kani -p reactos-slist

# Loom concurrency tests — explores all thread interleavings (3 tests)
cargo test -p slist-loom
```

### QEMU Kernel Tests (28 tests via serial dispatch loop)

```bash
cd reactos

# 1. Build test kernel
cd ../build-arm64 && cmake . -DREACTOS_TEST_KERNEL=ON && ninja ntoskrnl

# 2. Update ramdisk
mcopy -o -i ../qemu/ramdisk.img ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
mcopy -o -i ../qemu/boot.img ../qemu/ramdisk.img ::/ramdisk.img

# 3. Run tests (boots QEMU, sends commands over serial, checks [TEST PASS])
cd ../reactos
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -v       # All 28 tests
.venv/bin/python -m pytest tests/qemu/test_kernel.py --test-kernel -k SList # Just SList
```

### QEMU Smoke Tests (boot verification)

```bash
cd reactos && source .venv/bin/activate
pytest tests/qemu/test_smoke.py -v --arch arm64 --timeout 300
pytest tests/qemu/test_smoke.py -v --arch amd64 --timeout 300
pytest tests/qemu/test_smoke.py -v --arch i386 --timeout 300
```

## Rust in the Kernel

The Cargo workspace at `sdk/lib/rust/` holds Rust components that compile to PE-COFF static libraries and link into the C build. **Both SList and Bitmap build for all 3 architectures.**

```
sdk/lib/rust/
├── Cargo.toml              # Workspace (resolver v2)
├── slist/                   # Interlocked SList — all 3 architectures
│   ├── src/lib.rs           # FFI entry points (cfg per arch for calling convention)
│   ├── src/header.rs        # Bit-field pack/unpack (cfg per pointer width)
│   ├── src/algorithm.rs     # Pure state transitions + Kani proofs (no cfg, no unsafe)
│   └── src/atomics.rs       # Atomic primitives (LDXP/STXP, CMPXCHG16B, CMPXCHG8B)
├── bitmap/                  # RtlBitmap operations — all 3 architectures
│   ├── src/lib.rs           # FFI entry points
│   └── src/ops.rs           # Bitmap algorithms
└── slist-loom/              # Loom concurrency tests (std, not linked into kernel)
```

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
- **LLVM reserves rbx (amd64) and ebx (i386)** — use `xchg` workaround for CMPXCHG16B/CMPXCHG8B inline asm
- **`#[no_mangle] extern "stdcall"`** on i686-gnu correctly produces `_Name@N` decoration
- **`#[export_name]` gets double-decorated** on i686-gnu with stdcall — use `#[no_mangle]` instead
- **i386 SList Pop must be `global_asm!`** — `KiCheckForSListFault` checks exact instruction bytes (0x8B 0x18)

### ntkrnl-sys — Kernel FFI Declarations Crate

Dual-layer FFI crate providing kernel type and function declarations for Rust kernel components:

```
sdk/lib/rust/ntkrnl-sys/
├── Cargo.toml
├── generate_bindings.sh       # Runs bindgen for all 3 architectures
├── postprocess_bindgen.py     # Post-processes raw bindgen output
├── verify_sizes.py            # Generates compile-time struct size assertions
└── src/
    ├── lib.rs                 # Re-exports both layers
    ├── base.rs                # Hand-written: NTSTATUS, PVOID, LIST_ENTRY, etc.
    ├── ke.rs                  # Hand-written: KSPIN_LOCK, KeAcquireSpinLock, etc.
    ├── mm.rs                  # Hand-written: POOL_HEADER, POOL_DESCRIPTOR, etc.
    ├── ex.rs, ps.rs, ob.rs    # Hand-written: pool constants, process, object
    ├── rtl.rs, interlocked.rs # Hand-written: RTL functions, interlocked ops
    ├── bindgen_common.rs      # Generated: shared helper types
    ├── bindgen_arm64.rs       # Generated: ~82K lines, ~3,300 extern functions
    ├── bindgen_amd64.rs       # Generated: ~83K lines, ~3,350 extern functions
    ├── bindgen_i386.rs        # Generated: ~81K lines, ~3,360 extern functions
    ├── size_check_arm64.rs    # Generated: 1,522 compile-time size assertions
    ├── size_check_amd64.rs    # Generated: 1,542 compile-time size assertions
    └── size_check_i386.rs     # Generated: 1,525 compile-time size assertions
```

**Layer 1 — Hand-written modules** (`ntkrnl_sys::base`, `ntkrnl_sys::ke`, etc.): Curated declarations for the pool allocator and existing Rust kernel components. Includes inline functions, macros, and convenience wrappers that bindgen can't generate.

**Layer 2 — Bindgen kernel module** (`ntkrnl_sys::ddk::*`): Auto-generated from the full `ntoskrnl.h` master header + `mm/ARM3/miarm.h` via bindgen. Covers ALL internal kernel subsystems — MM (Mi*), OB (Ob*/Obp*), SE (Se*/Sep*), PS (Ps*/Psp*), KE (Ke*/Ki*), IO (Io*/Iop*), EX (Ex*), CC (Cc*), CM (Cm*/Cmp*), LPC, plus ~300 Nt* and ~190 Zw* syscalls. Provides ~3,300 extern functions, ~1,850 structs (including EPROCESS, ETHREAD, OBJECT_HEADER, MMPTE, MMPFN, MMVAD, TOKEN, HANDLE_TABLE, POOL_HEADER), and ~4,400 type aliases with correct calling conventions (including i386 fastcall/stdcall).

#### Size Verification

~1,500 compile-time `const _: () = assert!(size_of::<T>() == N)` assertions per arch verify every bindgen struct matches C layout. Generated by `verify_sizes.py` which cross-compiles `sizeof()` expressions to LLVM IR via clang and extracts values.

```bash
# Regenerate size assertions (after bindgen regeneration)
cd reactos/sdk/lib/rust/ntkrnl-sys
python3 verify_sizes.py

# Verify assertions are up to date
python3 verify_sizes.py --check
```

#### Regenerating Bindings

```bash
# Prerequisites: cargo install bindgen-cli
cd reactos/sdk/lib/rust/ntkrnl-sys
bash generate_bindings.sh    # Regenerate bindgen output
python3 verify_sizes.py      # Regenerate size assertions
```

The script runs bindgen against `ntoskrnl.h` + `miarm.h` for all 3 targets (`aarch64-pc-windows-gnu`, `x86_64-pc-windows-gnu`, `i686-pc-windows-gnu`), then post-processes the output to extract shared helpers and add module headers. Key wrapper header defines:
- `#define FORCEINLINE` — prevents inline function body expansion
- `#define _INTRIN_H_` — skips compiler intrinsics (not available in libclang)
- `#define __int64 long long` — type alias needed by various headers
- `#define __wfi()/__yield() ((void)0)` — ARM64 intrinsic stubs

#### Limitations

- **FORCEINLINE functions** (e.g., `KeGetCurrentIrql` on amd64) appear as extern declarations but have no exported symbol — must be hand-written or provided via overlay
- **Bindgen type naming** uses `_LIST_ENTRY` (with underscore prefix) vs hand-written `LIST_ENTRY` — the DDK module includes both via type aliases
- **Two skip types** in verify_sizes.py: `_GENERAL_LOOKASIDE_POOL` (complex union, bindgen disagrees with C) and `_LONGDOUBLE` (x87 long double, platform-specific padding)

### CMake Integration
`sdk/lib/rtl/CMakeLists.txt` invokes `cargo build --release` per architecture and links the `.a` files into librtl.

### Adding New Crates
1. Create directory under `sdk/lib/rust/`
2. Add to workspace `members` in `sdk/lib/rust/Cargo.toml`
3. Wire up `add_custom_command` + `target_link_libraries` in `sdk/lib/rtl/CMakeLists.txt`

## rustconv — C-to-Rust Translation Pipeline

API-driven pipeline that translates ntoskrnl C files to Rust using LLM (Sonnet batch API). Located at `tools/rustconv/`.

### Pipeline

```
C source → [scanner] tree-sitter parse → [resolver] match vs ntkrnl-sys catalog
→ [prompt_builder] system prompt + C + imports + exports → [batch_runner] LLM API
→ [postprocess] safe mechanical fixes → [assembler] write crate → [checker] cargo check ×3
```

### CLI

```bash
cd reactos
source .venv/bin/activate

python -m rustconv scan ntoskrnl/ex/uuid.c         # Parse, print analysis
python -m rustconv resolve ntoskrnl/ex/uuid.c       # Resolve deps → import manifest
python -m rustconv prompt ntoskrnl/ex/uuid.c         # Generate prompt to stdout
python -m rustconv estimate ntoskrnl/ob/             # Token count + cost estimate
python -m rustconv submit ntoskrnl/ex/uuid.c -o out.rs   # Single-file API call
python -m rustconv submit ntoskrnl/ex/uuid.c --assemble  # Translate + write to crate
python -m rustconv batch ntoskrnl/ob/ -m sonnet      # Batch entire subsystem
python -m rustconv check                              # cargo check ×3 targets
```

Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`, or full model ID.

### Key Files

```
tools/rustconv/
├── rustconv/
│   ├── scanner.py         # tree-sitter C parser (functions, globals, SEH, arch ifdefs)
│   ├── catalog.py         # Index ntkrnl-sys symbols (~9,700 per arch)
│   ├── resolver.py        # Match identifiers → imports + gap warnings
│   ├── spec_parser.py     # Parse ntoskrnl.spec (1,592 exports)
│   ├── prompt_builder.py  # Assemble system + per-file prompts
│   ├── batch_runner.py    # Anthropic + Gemini API client
│   ├── postprocess.py     # Safe mechanical fixes (type names, imports, enums)
│   ├── assembler.py       # Write to ntoskrnl-rust/src/{subsys}/{file}.rs
│   ├── checker.py         # cargo check ×3 targets → structured errors
│   └── config.py          # Paths, model config, token budgets
├── templates/
│   └── system_prompt.md   # Translation rules (~5K tokens, prompt-cached)
└── data/
    └── gap_functions.toml # FORCEINLINE/macro functions not in bindgen
```

### Post-Processor Safety

`postprocess.py` only does context-free transforms that are always correct:
- Type name underscore-prefixing (`OBJECT_HEADER` → `_OBJECT_HEADER`)
- Import fixes (`ddk::*` warning, pool type dedup)
- Enum variant name prefixing (`ObCreateHandle` → `_OB_OPEN_REASON_ObCreateHandle`)

Union field access (`__bindgen_anon_N`) is NOT done by the post-processor — it's context-dependent and handled by the system prompt teaching the LLM to generate correct accessors directly.

### Probe Migration Stage (`probe-migrate`)

Second pipeline stage that migrates bare `ProbeForRead`/`ProbeForWrite` calls in already-translated Rust code to use the Rust-native probe API (`ntkrnl-sys/src/probe.rs`). Operates file-by-file with preprocess→LLM→postprocess pattern.

```bash
python -m rustconv probe-migrate se/sqos.rs           # Single file
python -m rustconv probe-migrate se/ -m haiku          # Batch subsystem
python -m rustconv probe-migrate se/sqos.rs --scan-only  # Show probe sites
python -m rustconv probe-migrate se/sqos.rs --dry-run    # Preview without writing
```

Key files:
- `rustconv/probe_migrate.py` — scanner, preprocessor, postprocessor, runner
- `templates/probe_migrate_prompt.md` — 10 migration rules for LLM
- `sdk/lib/rust/ntkrnl-sys/src/probe.rs` — Rust-native probe API

The probe API returns `Result<T, NTSTATUS>` instead of raising exceptions:
```rust
probe_read_user::<T>(ptr, mode) -> Result<T, NTSTATUS>
probe_write_user::<T>(ptr, value, mode) -> Result<(), NTSTATUS>
probe_user_buffer(buf, len, align, mode) -> Result<(), NTSTATUS>
probe_capture_unicode_string_header(ptr, mode) -> Result<UNICODE_STRING, NTSTATUS>
```

**Status:** SE (12 files, 62 probe sites) and OB (6 files, 35 probe sites) fully migrated. All 3 arches build + smoke tests pass.

### Multi-Stage Pipeline Architecture

The translation pipeline has multiple stages, each refining the previous output:

```
Stage 1: C → Rust        (rustconv submit/batch)   — bulk transliteration
Stage 2: Probe migration (rustconv probe-migrate)  — ProbeForRead/Write → Result<T>
Stage 3: SEH idiomation  (planned)                  — full SEH → Result/RAII/match
```

Stage 3 will operate **function-by-function** (not file-by-file) using paired C+Rust function context, a dependency graph (SQLite cache), and Rust-native API surface for RAII guards, safe user-mode copies, and Result patterns. The LLM sees the original C (with SEH blocks visible) alongside the current Rust translation, plus callees and type context.

### Cost (Sonnet Batch API)

| Scope | Est. Cost |
|-------|-----------|
| Single file | ~$0.03 |
| Subsystem (11 files) | ~$0.30 |
| Full ntoskrnl (393 files) | ~$8.70 |
| 5 iterations | ~$43.50 |

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

Primary target is **Raspberry Pi** (ARM64). Development/debugging uses **QEMU** `virt` machine (ARM64), `q35` (amd64), or standard PC (i386).
