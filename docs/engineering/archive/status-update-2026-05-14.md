# Status — 2026-05-14

> **ARCHIVED 2026-07-10** — point-in-time status snapshot (2026-05-14), fully stale (dead "painted taskbar / idle-detection / arm64 C stubs" lore). Live current state: `docs/engineering/rust-port-status.md` (coverage) + `docs/product/vibeos-roadmap.md` (plan + known-bugs) + the root `README.md` (boot status).


## Smoke

All 3 arches boot to a painted taskbar.

| Arch  | Result | Notes |
|-------|--------|-------|
| i386  | 7/7 PASS | Live CD ISO; KVM accel |
| amd64 | 7/7 PASS | Live CD ISO; KVM accel; today's HAL cpuid xchg fix unblocked it (memory `feedback_cpuid_xchg_32bit_zext.md`) |
| arm64 | 5/5 PASS | UEFI ramdisk; idle-detection marker; 2 expected static-test skips |

Boot marker is `shell ready` on x86 (proves kernel → SMSS → csrss → win32k → winlogon → explorer.exe → desktop render). ARM64 uses idle detection. Screenshot saved per arch on every run.

## ntoskrnl translation

- **337 C files** (269K LOC) on disk, **405 Rust files** (447K LOC) in `sdk/lib/rust/ntoskrnl-rust/src/`.
- **8 C files actually compiled into `ntoskrnl.exe`** — all shims:
  - `mm_rust_shims.c`, `io_rust_shims.c`, `ke_rust_shims.c`, `kdbg_rust_shims.c`
  - `variadic_shims.c`, `mm_lock_outlines.c`, `ntprintf_trampoline.c`, `lto_workaround.c`
- Everything else under `ntoskrnl/` is on disk for reference but excluded from the build.
- 14+ sibling rlibs under `sdk/lib/rust/` factor out reusable algorithms (`slist`, `bitmap`, `splaytree`, `generictable`, `avltable`, `mm-avl`, `rtl-rust` umbrella, `ke-inlines`, `ke-macros`, `largeint`, etc.).

Per-subsystem Rust file count (Rust file count vs. C-on-disk file count, indicative not exact):

```
cc      8/7      ex     37/31    fsrtl  18/17    fstub   5/4
inbv    4/3      iomgr 29/26     ke    69/63     kd      7/8
kdbg   12/10     kd64  12/9      lpc    9/8      mm     23/44
ob     13/11     ps    22/16     po     5/4      rtl     4/2
se     20/18     vdm    3/2      wmi    5/4      pnpmgr 20/18
config 26/23     dbgk   3/2
```

mm/ has more C-on-disk than Rust because it contains a lot of small per-arch helpers and ARM3 subdirs that don't 1:1 to Rust files; the kernel build still uses the shim-elimination tracking, not file-count tracking.

## HAL translation

- **47 C files** (19K LOC) on disk, **49 Rust files** (15K LOC) in `sdk/lib/rust/hal-rust/src/x86/`.
- **mini_hal fully Rust** (Phase 2 closed `80ba39c6`): only `halinit.c` + `systimer.S` ship; the 17 other mini_hal C files are retired. Built as a separate `libhal_rust_mini.a` via the `minihal` Cargo feature.
- Live arch dirs in HAL C: `acpi/`, `apic/`, `generic/`, `legacy/{,bus}`, `pic/`. The Rust port covers the i386 surface plus shared portable bits; arm64 HAL stays as C stubs (per `hal-translation-audit.md`).
- raw-dylib infra (`b811836f`) lets Rust HAL files reference 41 kernel-data exports across DLL boundaries.

## Build paths

- **Primary:** cmake/Ninja drives `hal.dll` and `ntoskrnl.exe`. Per-arch dirs `build-{arm64,amd64,i386}/`. `ninja livecd` rolls the boot ISO.
- **Cargo PoC:** `sdk/lib/rust/ntoskrnl-cargo/` produces `ntoskrnl.exe` directly from cargo, reusing cmake-built `.obj`/`.a` artifacts. Goal is to replace the cmake-driven link entirely.
- Workspace firewall: `lto = false` + `codegen-units = 256` neutralizes the LLVM captures(none) class — replaced the older CGU=1 + opaque_ptr hack.

## Dead-platform purge (2026-05-13/14)

Deleted xbox, pc-98, 32-bit ARM, SAC, arm64ec source trees and the SARCH/ARCH cmake conditionals. ~28K LOC removed; xdk template-arch dispatch trimmed to IX86/AMD64/ARM64 only.

## Active threads

- **Shim elimination Phase 4** — Ob/io captures(none) sweep nearly closed; 19 of ~50 io fns landed direct (per memories). Remaining: GLOBAL_ACCESSOR sweeps, `IopInitializeDriverModule` complex-delegation translation, must-stay LLVM_WORKAROUND helpers.
- **HAL Stage 2** — i386 HAL Rust takeover continues; pic_pic + pcibus + bussupp + bushndlr landed this/last session.
- **rustconv multi-stage pipeline** — Stage 1 (C→Rust) and Stage 2 (probe migration) live; Stage 3 (SEH idiomation) planned.

## Open architectural issues

- **`--allow-multiple-definition` linker flag** still in use for HAL cohabitation. Should be removable once C HAL holdouts are gone.
- **769 unprotected output-pointer writes** flagged by `tools/lint_kernel.py` across 133 files. Theoretical captures(none) targets; runtime is clean (`scan_ud2.py --check` passes 0 unexpected ud2). Considering baselining the lint to gate regressions instead of churning the tree.
- **arm64 HAL stays C** — translation-audit explicitly defers it; the HAL Rust crate is x86-only.

## What lives where

- **Live status across sessions:** `~/.claude/.../memory/MEMORY.md` (the "CURRENT STATUS" entry is the freshest).
- **Per-bug postmortems / patterns:** memory files (`feedback_*.md`, `project_*.md`).
- **Long-running plans / audits:** `docs/engineering/{ntoskrnl,hal,freeldr,win32k,static-libs}-translation-audit.md` and the per-subsystem plans (`shim-elimination.md`, `archive/pseh-elimination.md`, `kernel-exception-handling.md`).
- **Reference docs:** `docs/engineering/{kernel-debugging-infra,rust-kernel-patterns}.md`, top-level `docs/{rustconv,testing-strategy}.md`.
