# Agent Reference — commands, layout, and platform specifics

**This is a PULL doc.** It was split out of `AGENTS.md` on 2026-07-24 because that file is
the PUSH channel (loaded into every agent session, for every agent) and its own Knowledge
Routing rule says to keep it tiny. Nothing here was rewritten — the sections below are the
verbatim reference material that used to live in `AGENTS.md`, so anything that linked to
them still reads true. The behavioural rules stayed behind in `AGENTS.md`.

Read this when you need a command, a path, or a platform detail. Related deeper docs:
`build-system.md`, `rust-in-kernel.md`, `rustconv-pipeline.md`, `kernel-debugging-infra.md`,
`tests/qemu/cli/README.md`, `qemu/README.md`.

---

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

## Landing your work — the merge queue

`./vibeos-cli land` **submits** this lane's tip to the fleet's merge queue and returns in
~2s. You do not merge `master`, do not run the gate, and do not wait. Whichever agent finds
the land lock free becomes the **conductor**: it merges every queued lane onto current
`origin/master` in a dedicated integration worktree, runs the full 3-arch gate **once**, and
fast-forwards `master` to exactly the commit it gated. N ready lanes therefore cost ONE gate.

```bash
./vibeos-cli land                  # submit (and train the batch if nobody else is)
./vibeos-cli land-status --mine    # conductor + queue + MY verdict (landed / rejected + artifact)
./vibeos-cli land --direct         # opt out: gate and push this lane alone
./vibeos-cli land --drain          # train without submitting (orphan recovery)
```

A merge conflict is **bounced back to you** with the conflicting paths — the conductor never
resolves it. A red batch **splits recursively**, so innocent lanes still land and the culprit
is rejected with its gate classification plus its artifact dir. Full design:
`tests/qemu/cli/README.md` → `land`, and the header of `tests/qemu/cli/queue.py`.

## Discarding work safely — the stash, and why index numbers lie

AGENTS.md forbids `git checkout`/`restore` for discarding. Stash instead. The trap is that the
stash stack is **shared by every worktree**, so `stash@{0}` is whatever was pushed most
recently anywhere — a bare `git stash pop` has applied a sibling lane's 1124 files into the
wrong tree. Two consequences:

**Pushing.** `git stash push <pathspec>` **fails outright if any listed path is untracked**, and
paired with `|| true` it creates no entry while looking like it worked. Pass `-u` whenever the
pathspec includes new files, then confirm the entry landed:

```bash
git stash push -u -m "mylane: <what and why>" -- <paths>
git stash list | head -3          # your message must be at the top
```

**Dropping.** `git stash drop <sha>` does not work: drop and apply need a `stash@{n}` ref, so
"drop by hash" is not executable advice. Resolve the index from the message, re-read the message
at that ref, and refuse if it no longer matches — in one command, so nobody can push between the
lookup and the drop:

```bash
IDX=$(git stash list --format='%gd|%gs' | grep -F "<my msg>" | head -1 | cut -d'|' -f1)
MSG=$(git stash list --format='%gd|%gs' | grep -F "$IDX|" | cut -d'|' -f2-)
case "$MSG" in *"<my msg>"*) git stash drop "$IDX" ;; *) echo REFUSED ;; esac
```

**Recovery, if you applied someone else's.** Their work is safe: a conflicted `pop` does not drop
the entry. Back up your own uncommitted files, confirm your commits are pushed
(`git rev-list --left-right --count origin/master...HEAD` shows `0` ahead), then
`git reset --hard HEAD` plus `git clean -fd <your dirs>`, and restore your backups.

## Build Commands

**Builds run at the CANONICAL PATH** (`/tmp/vibeos` — a per-build bind mount, see
`tools/canon-run.sh` and `docs/engineering/build-system.md`): build
dirs bake `/tmp/vibeos/...` paths so caches share across worktrees. `./vibeos-cli build`
handles this automatically (+ mandatory ccache/sccache, provisioning, lints). For raw
ninja iteration, prefix with the wrapper — **plain `ninja` in a build dir fails loudly**
(its baked paths only exist inside the mount):

```bash
# ARM64
cd build-arm64
../tools/canon-run.sh ninja            # Build everything
../tools/canon-run.sh ninja ntoskrnl   # Just the kernel

# amd64 / i386 (same pattern)
cd build-amd64
../tools/canon-run.sh ninja livecd     # Live CD ISO (livecd.iso)
../tools/canon-run.sh ninja bootcd     # Boot CD ISO (bootcd.iso)

# Common patterns
../tools/canon-run.sh ninja -k0        # Continue past errors
../tools/canon-run.sh ninja <target>   # Any target — list: ... ninja -t targets all | grep <name>
```

After modifying a `CMakeLists.txt` or `.cmake` file, re-run cmake before building:
```bash
../tools/canon-run.sh cmake .          # Re-configure (from any build-*/ dir)
```

## Toolchain

- **Compiler:** llvm-mingw (clang 19, lld 19.1.7)
- **Unified toolchain file:** `toolchain-llvm-mingw.cmake` — builds all 3 architectures
- **Targets:** `aarch64-w64-mingw32`, `x86_64-w64-mingw32`, `i686-w64-mingw32`
- **External dependency:** `windmc` from Homebrew mingw-w64 (message compiler, not in llvm-mingw)
- **16-bit assembly:** Homebrew `i686-w64-mingw32-gcc` for boot sectors/frldr16/MVDM (Clang can't handle `.code16`)
- **Output format:** PE-COFF (Windows executables/DLLs), not ELF

### Rust is on PINNED STABLE — write stable Rust (2026-07-12)

**The Rust tree builds on stable rustc** (pinned in `rust-toolchain.toml`). Your new code
must compile on stable. **Do NOT add `#![feature(...)]`, and do NOT add `RUSTC_BOOTSTRAP=1`
to a CMakeLists** — that env var is the stable-rustc escape hatch that silently unlocks
nightly, and it is how nightly crept back in the same day the lane removed it. Both are
now hard-gated by `scan_nightly_surface` (`vibeos-cli check`), so you'll get a loud failure
rather than a surprise.

The **one** sanctioned nightly user is the **i386 kernel** `*-cargo` cell: it needs
`-Z build-std` + `i686-kernel.json` for `rustc-abi: x86-softfloat` (no builtin triple
provides a no-SSE `core`), pinned in `.rust-nightly`. It *retires* with the wow64 lane.
amd64/arm64 build the **builtin triples** with rustup's precompiled std.

If you think you need a nightly feature, you almost certainly don't:
- **`c_variadic`** — needed only to *DEFINE* a C-variadic fn. *Declaring* and *calling* one
  is fine on stable. No-op debug shims (`TRACE(fmt, ...)`) should be a `macro_rules!` (see
  `ole32-rust/src/debug_shims.rs`); to genuinely define one, use the C `va_start` trampoline
  in `sdk/lib/rust/varargs-rust/`.
- **`linkage`** — perma-unstable upstream; it will never stabilize. Restructure.

Two related traps, both real bugs we ate (field manual §9.65):
- **Never translate a C debug MACRO into a no-op `fn`.** An out-of-line no-op is an
  optimization barrier that uncloaks latent ABI mismatches → `ud2` → BUGCHECK. Use a macro.
- **Never hand-write `memcpy`/`memset`/`memmove`/`memcmp` in Rust.** LLVM folds any byte-copy
  loop back into a call to the function itself → infinite self-recursion → silent hang.
  ntoskrnl exports them; just import them.

Full reference: `docs/engineering/build-system.md` §"Cargo target model".

## Key Technical Patterns

### SEH / DUMMY_PSEH (status 2026-07-10 — the ruling doc is `docs/engineering/dummy-pseh-verdict.md`)
`USE_DUMMY_PSEH` is still the default compile mode for **C** on all arches (PSEH3 is incompatible with Clang — ARM64 lacks x86 SEH; Clang's ESP-relative locals break i386 PSEH filters), so `_SEH2_TRY`-protected C must still validate pointers before dereferencing. **But the old "SEH is dead code everywhere, swallow software exceptions" era is OVER** (2026-07):
- **Rust frames have real unwind tables** (`requires-uwtable`): the kernel ships `.pdata`/`.xdata` for every Rust function and the unwinder works (`rtl-except`; field manual §9.35).
- **The exception dispatcher is live infrastructure**: hardware faults and software raises dispatch to user-mode SEH on all three arches (arm64 unwinder fixed 2026-07-10; user-SEH self-test in the gate). Do NOT re-add "swallow it" hacks.
- **Kernel-mode catch is possible** (validated PoC `ntoskrnl/seh_poc.c`); native SEH is a sanctioned per-module tool for the ~22 remaining C files with `_SEH2_TRY`.
- **Rust replaces PSEH semantics idiomatically**, not mechanically: validate-first probes returning `Result` (rustconv Stage 2), `try_*` contract conversions at raise sites, RAII `Drop` guards for `__finally`. See the verdict doc before writing any new SEH-shaped code.

### ARM64 PE-COFF Relocations
Use `adrp + add :lo12: + ldr` (not `adrp + ldr :lo12:`) in assembly. The latter generates `LDST64_ABS_LO12_NC` relocations requiring 8-byte alignment which PE-COFF sections may not provide.

### LLD i386 Forwarder Decoration Bug
LLD prepends `_` to PE export forwarder targets on i386. Workaround in FreeLDR (peloader.c) and ntoskrnl (sysldr.c): strip leading `_` from forwarder DLL name.

### GAS Intel Syntax on i386
In GAS `.intel_syntax noprefix`, `mov eax, label` does a **memory load** (reads bytes at label), NOT an address load. Must use `lea eax, [label]` or `mov eax, offset label`. Also `push offset label` fails to assemble under clang's integrated assembler ("unsupported relocation type") — materialize via `lea eax, [label]` + `push eax` (field manual §9.101).

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

Full toolkit reference + design: `docs/engineering/kernel-debugging-infra.md` and `qemu/README.md`. (`qemu/ps` shows every live QEMU across all worktrees + slot usage; QEMUs are capped via `VIBEOS_QEMU_MAX_CONCURRENT`/`VIBEOS_QEMU_MAX_LIFETIME`; **same-arch i386/amd64 boots run in parallel on the default `windows-host` TCG backend** — the per-arch sub-cap is bypassed there (`vm.py:1295-1299`) and only serializes the accelerator-sharing backends (native+KVM, windows-host+WHPX); see the North Star note + `qemu/README.md` → Resource management); run `qemu/build-index.sh <arch>` after each kernel build for offline sym lookup.)

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
# 1. Build test kernel (canonical wrapper — see Build Commands)
cd build-arm64 && ../tools/canon-run.sh sh -c 'cmake . -DREACTOS_TEST_KERNEL=ON && ninja ntoskrnl'
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

Smoke/CLI harness internals (`vm.py`/`conftest.py`/`serial_client.py`/`qmp_client.py`/`debugcon.py`, boot media, QEMU backends + acceleration (windows-host TCG default / native KVM / WHPX opt-in), resource caps, debugcon boot tracing, test lifecycle): `tests/qemu/cli/README.md`.

## Rust in the Kernel

The Cargo workspace at `sdk/lib/rust/` holds Rust components that compile to PE-COFF static libraries (rlibs/staticlibs) and link into the cmake build. **All crates build for all 3 architectures unless explicitly noted.** The workspace long ago outgrew "kernel algorithms": it spans the bootloader (`freeldr-rust`), the HAL (`hal-rust`, all three arches), win32k (`win32k-rust`), user-mode components (`winlogon-rust`, `explorer`-side crates, server DLLs), and a growing driver fleet (`uniata`, the `usbport`/`usbhub`/HID stack, netstack, `disk`/`classpnp`, …).

```
sdk/lib/rust/
├── Cargo.toml              # Workspace (resolver v2). lto=false + codegen-units=256 firewall.
├── ntoskrnl-rust/          # The (shrinking) kernel-monolith crate — ~130K LOC (was ~456K).
│                           #   Being split per-subsystem (crate-split, ~70% done): what REMAINS
│                           #   here is the dense center — ke, mm/ARM3. The EXTRACTED crates
│                           #   (iomgr, ex, se, config, pnpmgr, ps, ob, fsrtl, kdbg, fstub,
│                           #   kd/kd64, rtlk, cc, po, dbgk, lpc, wmi, inbv, vdm, vf, …) talk to
│                           #   it over the FFI symbol boundary, NOT crate:: paths.
│                           #   See docs/engineering/ntoskrnl-crate-split-plan.md.
├── {iomgr,ex,se,config,pnpmgr,ps,ob,fsrtl,kdbg,fstub,kd,kd64,rtlk,cc,po,dbgk,lpc,wmi,inbv,vdm,vf}-rust/
│                           # Subsystems peeled out of the ntoskrnl-rust monolith (crate-split).
│                           #   Each deps ONLY ntkrnl-sys + dbgprint-rust; forced into the link by
│                           #   kernel-rust's `extern crate` (37 rlibs today). Extract another via
│                           #   the plan's recipe.
├── ntoskrnl-cargo/         # PRODUCTION cargo-driven ntoskrnl.exe link (graduated from cmake
│                           #   add_executable — reactos-link engine; ntoskrnl/CMakeLists.txt:118)
├── ntkrnlmp-cargo/         # SMP-kernel sibling link (ntkrnlmp.exe): kernel asm assembled with
│                           #   CONFIG_SMP. ON MASTER since 2026-07-11 (M0–M3 complete: -smp 2
│                           #   boots to desktop, both CPUs schedule, TLB shootdown + generic-call
│                           #   IPI live; all opt-in via `build --smp-kernel` + `smoke --smp N`,
│                           #   defaults stay UP). Quest log + remaining M4/M5 work:
│                           #   docs/engineering/active-lanes/smp-bringup-quest.md
├── kernel-rust/            # staticlib umbrella that pulls in all sibling rlibs via extern crate
├── ntkrnl-sys/             # Two-layer FFI crate: hand-written + bindgen (see below)
├── hal-rust/               # HAL Rust translation — x86 flavors (i386 + amd64, minihal feature
│                           #   for FreeLDR) AND arm64 (GIC/timer/DMA-adapter/PL031 modules)
├── rtl-rust/               # Umbrella over the algorithm rlibs from sdk/lib/rtl/* (~39 crates)
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

The structure isn't an exhaustive list — the workspace has ~250 members today. The principle is: anything that's pure-algorithm with a clean FFI surface gets factored into its own rlib so the workspace `lto=false` + `cgu=256` firewall keeps callers and callees in separate codegen units. (That firewall's *original* job — blocking LLVM's captures(none) volatile-blind miscompile across CGU boundaries — is **toolchain-fixed on LLVM ≥ 21**: we ship rustc 1.92 / LLVM 21.1.3, which respects volatile stores in attribute deduction (see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"). The firewall *stays* as belt-and-suspenders and because it still firewalls the **toolchain-independent** ABI-mismatch `noreturn`/ud2 cascade; `tools/scan_captures.py` is the tripwire to rerun after each toolchain bump in case a future LLVM regresses it.) The **crate-split** effort (docs/engineering/ntoskrnl-crate-split-plan.md) applies that same principle to the kernel itself for a second reason — cutting the edit→rebuild loop, since rustc's frontend is whole-crate.

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
- **LLVM reserves rbx (amd64) and ebx (i386)** — use `xchg` workaround for CMPXCHG16B/CMPXCHG8B inline asm. **On amd64, the xchg MUST be 64-bit** (`xchg rbx, rbx_save`), not 32-bit (`xchg ebx, ebx_save:e`) — a 32-bit register write zero-extends into rbx, silently destroying the upper 32 bits of any caller-held pointer there. Cost a multi-hour BUGCHECK 0x50 debug session in 2026-05.
- **`#[no_mangle] extern "stdcall"`** on i686-gnu correctly produces `_Name@N` decoration
- **`#[export_name]` gets double-decorated** on i686-gnu with stdcall — use `#[no_mangle]` instead
- **i386 SList Pop must be `global_asm!`** — `KiCheckForSListFault` checks exact instruction bytes (0x8B 0x18)
- **NTAPI vs cdecl ABI** matters per arch: i386 NTAPI = stdcall (`_Name@N`), amd64/arm64 NTAPI = cdecl (just `Name`). `extern "system"` resolves to the right thing per arch. Mismatched `extern "C"` vs `extern "system"` declarations across rlibs link silently on amd64/arm64 but break i386. Detect with `tools/rustconv/rustconv/verify_ffi.py` (the gate's workspace-wide ABI checker; `verify_abi.py` is the narrower calling-convention-only variant it grew out of).
- **Cross-DLL data imports** (HAL referencing kernel-data exports like `KdDebuggerNotPresent`, `KeServiceDescriptorTable`): need `#[link(name = "ntoskrnl", kind = "raw-dylib")]` on the extern static block. `postprocess_bindgen.py --hal` adds this automatically for the 41 affected externs.

### ntkrnl-sys — Kernel FFI Declarations Crate (overview)

Dual-layer FFI crate at `sdk/lib/rust/ntkrnl-sys/` providing kernel type and function declarations:
- **Layer 1 — hand-written modules** (`ntkrnl_sys::base`, `::ke`, `::mm`, `::ex`, `::ps`, `::ob`, `::rtl`, `::interlocked`): curated declarations, inline functions, macros, and convenience wrappers bindgen can't generate.
- **Layer 2 — bindgen kernel module** (`ntkrnl_sys::ddk::*`): auto-generated from `ntoskrnl.h` + `mm/ARM3/miarm.h`, covering all internal subsystems (~3,300 extern fns, ~1,850 structs, ~4,400 type aliases per arch, with correct i386 fastcall/stdcall conventions). ~1,500 compile-time `size_of` assertions per arch verify C layout.

Full reference (directory tree + line counts, size-verification `verify_sizes.py`, bindgen regeneration via `generate_bindings.sh`, limitations), plus the CMake build-integration mechanics and the "Adding New Crates" steps: `docs/engineering/rust-in-kernel.md`.

### Cohabitation pattern (HAL)

For HAL Rust translation, each .c → .rs swap is independently revertible: the C file gets commented out in the per-flavor cmake (`hal/halx86/{generic,legacy,pic,acpi,apic}.cmake`) with a `# replaced by Rust (path)` line, and the Rust file's `#[no_mangle]` symbol takes over. `--allow-multiple-definition` allows side-by-side C+Rust during the swap-in phase; lld first-match wins. (Killing that flag outright is a parked lane — branch `kill-allow-multiple-def`, resume state in-branch.) See `docs/engineering/rust-in-kernel.md` → "HAL ABI contract" for the durable ABI reference, and `docs/engineering/archive/hal-translation-audit.md` for the (closed) per-file mapping.

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
python -m rustconv candidate-check candidates.json --repair  # Batched x3 candidate oracle
python -m rustconv substrate-plan path/to/report.json   # Cluster compiler roots by authority
python -m rustconv crate-repair foo-rust --arches i386 amd64 arm64  # $0 deterministic repair
python -m rustconv.profile_derive dll/win32/foo --emit report       # Derive bindgen island
```

### Deterministic substrate closure

After assembling a candidate, run `candidate-check --repair`, then use the embedded report or
`substrate-plan` to fix distinct compiler roots by authority—not hundreds of individual rustc
sites. Full-module manifest entries should include `"source"` so source-local definitions are
routed correctly. Reuse/promote canonical Rust first, then project bindings, SDK/header
generation, explicit local/bridge generation, and scope repair. Constants, SDK layouts, WIDL
interfaces, vtables, ABI, and imports never go through the LLM. Ambiguous layouts stay red;
same-arity local overloads and trailing C++ default arguments are likewise resolved and
expanded deterministically before payment. If preflight cannot prove one binding, extend the
harness or leave the class red; never ask the model to guess C++ compiler semantics.
Sonnet is considered only after substrate closure leaves a genuine semantic/structural
residual. The complete protocol and stop rules are in
`docs/engineering/rustconv-substrate-closure-protocol.md`.

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

# Silent-failure shapes: bare loop{} spins, unwrap/expect in kernel code,
# swallowed Err arms (AST/tree-sitter; wired into `vibeos-cli check`, 0-NEW gate)
.venv/bin/python tools/scan_silent_failures.py           # sweep no-std workspace
# Suppress a sanctioned loud halt: // nolint: silent-failure (reason)

# Hand-written NUMERIC struct offsets (the MMPFN offset-shift / KTHREAD.Teb-0x1e8
# drift class — verify_offsets.py proves bindgen DECLS, this catches USAGE sites;
# fix = core::mem::offset_of!(bindgen type, field), never a bare number)
python3 tools/scan_magic_offsets.py                      # sweep sdk/lib/rust
# Suppress a verified FP: // nolint: magic-offset (reason)

# Numeric bugcheck STOP CODES (§9.113: the LLM invented the number for 61 of ~115
# translated KeBugCheck(SYMBOL) calls — plausible, self-consistent, and naming the
# WRONG subsystem, e.g. INVALID_AFFINITY_SET -> an IRP bugcheck. Invisible to rustc
# and to every smoke test; it only bites once the kernel is already dying.)
python3 tools/scan_bugcheck_codes.py                     # empty baseline — keep it at 0
python3 tools/gen_bugcodes.py                            # regenerate ntkrnl_sys::bugcodes
# Fix = KeBugCheckEx(ntkrnl_sys::bugcodes::NAME, ...); suppress a deliberate
# non-code (debug marker, DEFAULT_UNREACHABLE) with // nolint: bugcheck-code (reason)

# INVENTED CONSTANTS, generally (§9.113a/b) — two checkers that compose. Use BOTH:
#   verify_constants = precise oracle, but only sees headers in its curated catalog
#   scan_const_contradictions = no oracle, total coverage: "our tree spells one name
#   two ways, so one is wrong". Loop: contradictions finds the family -> resolve vs
#   the real C header -> add that header to verify_constants' catalog -> gated forever.
.venv/bin/python tools/rustconv/rustconv/verify_constants.py   # (--fix to auto-apply)
python3 tools/scan_const_contradictions.py               # baseline = debt + work queue
# DON'T "fix" coverage by sweeping all 2216 headers — tried, 486 findings, nearly all
# false (cross-header name collisions). §9.113b has the evidence.

# Boot coverage map: which functions a real boot EXERCISES (TCG plugin boot)
tools/coverage/run.sh amd64        # -> per-function exercised set + the
                                   #    never-executed-rust.txt landmine registry

# captures(none)-class post-opt IR audit (POISON_ARG committed folds /
# READNONE_OUTPARAM latents / UNREACHABLE_BODY). Class is toolchain-FIXED on
# LLVM >= 21 — rerun this after every rustc/LLVM bump to prove it stayed fixed.
VIBEOS_EMIT_LLVM_IR=1 ./vibeos-cli build --arch amd64   # emit per-CGU .ll
.venv/bin/python tools/scan_captures.py sdk/lib/rust/target/ntoskrnl-cargo-*-rustlib
# Related: VIBEOS_LINK_MAP=1 (lld -Map per cargo module — layout ground truth);
# VIBEOS_LAYOUT_CANARY=1 (deliberately perturbed kernel link — boot tripwire)
```

`scan_ud2.py` finds `ud2` (0f 0b) in named functions — LLVM proved the body has UB (on LLVM ≥ 21 almost always an ABI mismatch; the captures(none) volatile-blind root is toolchain-fixed, see rust-kernel-patterns.md "RESOLVED AT THE TOOLCHAIN"); it's wired into smoke as `test_no_unexpected_ud2`. `lint_kernel.py` finds `*OutputParam = value` writes in `#[no_mangle]` functions that skip `write_volatile` (a captures(none) target) before compilation. (The smoke harness also nukes `rust-kernel-target/` per session unless `--skip-clean-build`, runs the ud2 scan pre-boot, and auto-symbolicates BSODs — see `tests/qemu/cli/README.md`.)

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

All development and CI happens in **QEMU**: `virt` machine (ARM64), `q35` (amd64), or standard PC (i386).

**Acceleration / backend (this misled people — read it):** the default smoke backend is **`windows-host`** — the harness launches the **Windows** `qemu-system-*.exe` on the host via WSL interop (media staged to `/mnt/c/temp/vibeos-qemu`), running **TCG (pure emulation) on all three arches**. This deliberately escapes WSL2 nested-virt: host TCG beats nested KVM, and there's no single-virt-slot contention, so the beast host can run all arches in parallel. **We are *not* on `/dev/kvm`** despite it being present in WSL2. Two alternates exist: `VIBEOS_QEMU_BACKEND=native` (local Linux qemu + Unix sockets + KVM for x86 — the old path, still used by the kernel-debug toolkit's GDB flow), and **WHPX** (Windows Hypervisor Platform, real x86 hardware virt in `qemu-system-x86_64.exe`, `accel: "whpx"`) — **works since 2026-07-10: both x86 arches boot to desktop in ~52s** (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx`; the boot CD rides an ich9-ahci controller under whpx so OVMF DMAs it instead of ~5min of PIO VM-exits — see `docs/engineering/windows-host-qemu-whpx-spike.md` + field manual §9.46 for the six guest-side bugs that bring-up flushed out). Still opt-in (default stays TCG) until whpx accumulates reliability history; same-arch whpx boots serialize (hardware-virt contention, like KVM). arm64 is always TCG (WHPX is x86-only). Config + per-arch resolution: `tests/qemu/cli/config.py` (`qemu.backend`/`qemu.windows.accel`) + `tests/qemu/backend.py` (`resolve_accel`). Real-hardware boot (e.g., Raspberry Pi for ARM64) is an aspirational target, not a current one.

## rustconv — model shortcuts, HAL mode, and pipeline stages

Model shortcuts: `sonnet`, `haiku`, `opus`, `gemini-pro`, `gemini-flash`, `deepseek`, or full model ID.

`rustconv translate` accepts HAL paths and is HAL-aware (separate `system_prompt_hal.md`, hal_ddk imports, auto-CMake edit). Output quality is ~70% useful for HAL — review before integrating.

The pipeline has multiple stages: Stage 1 (C → Rust, `submit`/`batch`), Stage 2 (probe migration, `probe-migrate` — `ProbeForRead`/`Write` → `Result<T>`; SE+OB done), Stage 3 (Result+Drop-RAII idiomation, planned). See `docs/engineering/rustconv-pipeline.md` for the per-stage design.

## Branch history

Work was previously staged on `continue-rust-port` with `master` as a periodically
fast-forwarded mirror; the two were unified onto `master` on 2026-07-03 and
`continue-rust-port` was retired. You may still see the old branch name in stale refs.
