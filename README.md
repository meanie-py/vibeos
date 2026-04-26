# vibeos

> *An operating system for nobody.*

An AI-vibecoded fork of [ReactOS](https://reactos.org/) that ports the kernel to ARM64 and rewrites most of it in Rust — for fun, for science, and for absolutely no practical purpose.

> [!CAUTION]
> **Do not use this for anything. Ever.**
>
> This is an experimental research toy generated largely by LLMs (Claude, Codex, Gemini) running automated translation pipelines against the ReactOS C source. The kernel is held together by static analyzers, smoke tests, and `#[no_mangle]` incantations. **It is significantly less production-ready than upstream ReactOS, which is itself alpha-quality and explicitly warns against use on real hardware with real data.**
>
> If ReactOS says "may corrupt your hard disk," vibeos says "will absolutely brick your VM, eat your data, summon undefined behavior, and generate ud2 instructions in functions LLVM has decided are unreachable." Run it in a throwaway QEMU and nowhere else.

---

## Screenshots

> [!NOTE]
> The i386 and amd64 screenshots reflect the current state. The ARM64 screenshots are **historical** — captured Feb 16, 2026 (M29/M30/M31 milestones). Subsequent kernel-Rust translation work regressed ARM64 user-mode boot; as of late April it reaches SMSS user-mode dispatch reliably but doesn't get to a desktop yet. They're kept here to document what has *previously* been achieved on ARM64.

| i386 LiveCD desktop | amd64 LiveCD desktop |
|---|---|
| ![i386 desktop](docs/screenshots/i386_desktop.png) | ![amd64 desktop](docs/screenshots/amd64_desktop.png) |

| ARM64 desktop *(Feb 16, M31 — historical)* | ARM64 Notepad *(Feb 16, M29 — historical)* |
|---|---|
| ![ARM64 desktop](docs/screenshots/arm64_desktop.png) | ![ARM64 notepad](docs/screenshots/arm64_notepad.png) |

ARM64 was the original target — bringing up an NT-style kernel on a non-x86 architecture from a codebase that had never built for one before. The Notepad screenshot is from milestone M29 (interactive input over serial); the desktop screenshot is from M31 (real virtio-input devices via `qemu --gui`). Both worked at the time and will be the goal to recover.

---

## What this is

A multi-architecture port of ReactOS — an open-source Windows-compatible operating system — with two parallel goals:

1. **ARM64 port.** Cross-compile the entire NT-compatible kernel and userspace to AArch64 PE-COFF using `llvm-mingw`. Boot under QEMU `virt` to a Win32 desktop with virtio mouse/keyboard. Also retains amd64 and i386 builds; all three target zero build failures and pass smoke tests independently.
2. **Rust in the kernel.** Translate the C source of `ntoskrnl` to Rust subsystem-by-subsystem using an automated LLM pipeline (`tools/rustconv/`), then link the result into the kernel as a static library. Currently around **80% of the kernel** (~344K lines) is Rust — including the full `ob`, `se`, `ps`, `ex`, `ke`, `mm`, `cc`, `fsrtl`, `lpc`, `dbgk`, `po`, `wmi`, `vdm`, `vf`, and `rtl` subsystems, plus the pool allocator and the SLIST / bitmap primitives.

The translation pipeline produces transliterated Rust that mostly preserves the original C structure, with subsequent passes for probe migration, RAII guards, and idiomatic NTSTATUS handling. It is held together by:

- ~1,500 compile-time struct layout assertions per architecture (bindgen-generated).
- A custom static-analysis suite (`scan_ud2.py`, `lint_kernel.py`, `verify_constants.py`, `verify_ffi.py`, `verify_abi.py`) for catching LLVM miscompilations and FFI mismatches.
- QEMU smoke tests (`tests/qemu/test_smoke.py`) that boot each arch end-to-end and require a graphical desktop to count as passing.

## What this is *not*

- A usable operating system.
- A drop-in replacement for ReactOS — most of the value here is in the experimental scaffolding (translation pipeline, multi-arch toolchain, smoke harness), not the resulting binaries.
- An attempt to compete with, replace, or upstream into ReactOS. Branding, donation links, and project infrastructure from upstream have been stripped because none of it applies to this fork. The original code is GPL-2.0; see [`COPYING`](COPYING).
- Affiliated with or endorsed by ReactOS, the ReactOS Foundation, Microsoft, or anyone else with a legal department.

## Status

> [!NOTE]
> **Active development is intermittent.** The repo is generally kept in a state where all three architectures build cleanly and pass the smoke harness, but the smoke harness's bar varies a lot by arch (see below). It may go quiet again at any time.

Current per-arch boot state (HEAD as of late April 2026):

- **i386 — full graphical desktop with start menu and taskbar.** This is what the i386 screenshot above actually shows today. LiveCD ISO boots through the full chain (kernel → SMSS → CSRSS → win32k → winlogon → explorer → desktop with shell). Smoke tests pass 7/7 (including milestone validation through `[EXPLORER] _tWinMain entered`).
- **amd64 — full graphical desktop, regression fixed.** Previously broken (desktop wallpaper paint but no taskbar) for several weeks; the explorer-not-starting bug turned out to be downstream of an fstub wide-string translation bug (`L"\\Device\\Harddisk%d\\Partition0"` translated as bytes-cast-to-`*const u16`, producing garbage paths every other character). Fixed late April; smoke tests now pass 7/7.
- **ARM64 — kernel + SMSS user-mode dispatch reliable, no UI yet.** Kernel boots through to user-mode SMSS dispatch (an MMPTE soft-PTE bit-layout fix landed late April). SMSS gets dispatched but doesn't reach `_main` — silent failure inside `LdrInitializeThunk` or CRT init. The historical M29/M30/M31 screenshots above (full desktop, virtio mouse, Notepad with serial keyboard) regressed in March and have not been recovered. Smoke tests 4/7 (2 skipped on objdump, 1 fails on milestones — useful diagnostic that points at the user-mode loader path).

Other status:

- **Build:** All three arches build cleanly from Linux/WSL2 with the unified `llvm-mingw` toolchain. Zero build failures.
- **Rust coverage:** ~80% of `ntoskrnl` translated. Remaining C: `config/` (registry CM, ~19K lines — last major holdout), `kd64/` (translated but disabled due to a recursive-exception bug on amd64), and a few KDBG/KD debug helpers. **`fstub/` (disk partitioning) was completed late April** — disksup, fstubex, halstub, translate — bringing the kernel to ~85% Rust.
- **Known active bugs:** LLVM `captures(none)` interaction with kernel volatile semantics is the single largest source of bugs, mitigated by `opaque_ptr` barriers on ~2,500 `#[no_mangle]` functions and `codegen-units=16`. CGU=1 still produces ~15 unexpected `ud2` instructions due to interprocedural noreturn contagion; the workaround is documented but not yet fixed at the root.
- **Smoke harness:** Now includes a `test_milestones` check that scans serial output for per-arch boot markers (`[SMSS] _main entered`, `[CSRSS] _main entered`, `[WINLOGON] WinMain entered`, `[EXPLORER] _tWinMain entered`, `shell ready`). Catches silent regressions in user-mode rendering — found and fixed the amd64 explorer regression as soon as it was added.

For a deeper look, see:

- [`docs/product/status-update.md`](docs/product/status-update.md) — IO subsystem status board
- [`docs/product/vibeos-roadmap.md`](docs/product/vibeos-roadmap.md) — what this could become
- [`docs/product/vibeos-roadmap-complete.md`](docs/product/vibeos-roadmap-complete.md) — what's already shipped
- [`docs/engineering/`](docs/engineering/) — debug briefs, translation audits, kernel-Rust patterns
- [`AGENTS.md`](AGENTS.md) — the long-form context document used by AI agents working on this repo (also the best single-page summary of how everything fits together)

## Building (for the curious)

Requires `llvm-mingw` (Clang 19+), `windmc` from Homebrew `mingw-w64`, `cargo` with the `aarch64-pc-windows-gnullvm`, `x86_64-pc-windows-gnu`, and `i686-pc-windows-gnu` targets, plus `qemu-system-{aarch64,x86_64,i386}`. See [`AGENTS.md`](AGENTS.md) for the full setup.

```bash
# ARM64 (UEFI ramdisk boot)
cd build-arm64 && ninja
bash qemu/run.sh --arch arm64 --gui

# amd64 (LiveCD ISO)
cd build-amd64 && ninja livecd
bash qemu/run.sh --arch amd64

# i386 (LiveCD ISO)
cd build-i386 && ninja livecd
bash qemu/run.sh --arch i386

# Smoke tests (requires Python venv)
pytest tests/qemu/test_smoke.py -v --arch i386
```

## License

The original ReactOS code is licensed under [GNU GPL 2.0](COPYING). All modifications in this fork inherit that license.

The vibecoded slop is provided as-is, with no warranty of any kind, and in particular no warranty that the code does what its function names suggest.
