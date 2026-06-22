# ReactOS → Rust Port — Status & Timeline

**A multi-architecture port of ReactOS (Windows-compatible OS) being
translated from C to Rust, booting to a graphical desktop on i386, amd64,
and ARM64.**

*Verified 2026-06-04 against the `continue-rust-port` branch. Part 1 is the
current translation status; the project history is in Appendix A.*

> **Read this first — the cohabitation caveat.** Project convention is
> *never delete a translated C file*. When a `.c` is flipped to Rust it is
> **commented out in CMake and kept on disk** as dead source; the Rust
> `#[no_mangle]` symbol takes over the link. Therefore
> `find … -name '*.c' | wc -l` counts **on-disk** C, **not compiled** C.
> Every "C remaining" figure below is **compiled/active C** unless marked
> *on-disk*. The two diverge by a lot (e.g. ntdll has ~0 compiled C but
> ~11.9K LOC of dead C on disk).

---

## TL;DR

- **The kernel proper (ntoskrnl body) is ~99.7% Rust** — the only live C is
  8 header-free shim files (1,037 LOC), most of them irreducible (variadic
  trampolines, one LLVM-LTO workaround, two hardware/KVM shims). By
  "translatable body" the kernel is effectively done.
- **The boot spine is Rust end-to-end**: FreeLDR → ntoskrnl → HAL(x86) →
  ntdll → smss → csrss/winsrv → win32k-GDI all flip to Rust and boot to
  `shell ready` on all 3 arches.
- **Three big C buckets remain**, in priority order:
  1. **win32k USER core** (~74K compiled C) — the genuine active frontier.
  2. **rtl C bodies** (~53K compiled C) — heap, actctx, threadpool, exception engine.
  3. **libcntpr / kernel CRT** (~53K compiled C) — own track, ~10–17% flipped.
- Everything else is small, by-design asm/vendor code, or translated-but-not-yet-wired.

---

## Master component ledger

| Component | Rust LOC | Compiled C remaining | % done | Build status | Arches |
|---|---:|---:|---:|---|---|
| **ntoskrnl** (kernel body) | 453,404 | 1,037 (8 shims) | **~99.7%** | Default | 3× |
| **HAL (x86)** | 21,245 | ~0 (empty stub TUs) | **~100%** | Default (cohab) | i386+amd64 |
| **HAL (arm64)** | 0 | 1,007 (`halstubs.c`) | n/a (stubs) | C stubs | arm64 |
| **FreeLDR** | 74,204 | 14 of 91 .c active | **~85% (files)** | Default | 3× |
| **bootvid** | 5,059 | ~0 | **~100%** | Default (cohab) | 3× |
| **kdcom** (kdrosdbg) | 175 | ~0 | **~100%** | Default (cohab) | 3× |
| **ntdll** | 19,736 | ~0 + 708 asm (by design) | **~96%** | Default + cargo-link PoC | 3× |
| **rtl** (Rtl* bodies) | 15,733 | 53,392 | **~30%** | Default (cohab) | 3× |
| **libcntpr** (kernel CRT) | 6,177 | ~53,000 | **~10–17%** | Default | 3× |
| **win32k.sys** (kernel GUI) | ~140,000 | ~148,000 (USER core) | **~50%** | Default (cohab, 93 .c flipped) | 3× |
| **smss.exe** | in `smss-csrss-rust` | ~0 | **100% flipped** | Default | 3× |
| **csrss / csrsrv / basesrv** | in `smss-csrss-rust` | ~0 | **100% flipped** | Default | 3× |
| **winsrv.dll** | 28,430 | ~0 (cohab stubs) | **100% flipped** | Default | 3× |
| **i8042prt.sys** | 8,454 | ~0 | translated | Default (cohab) | 3× |
| **arm64input.sys** | 2,947 | 0 | **100%** | Default (cohab) | arm64 |
| **kbdclass.sys** | 2,668 | 1,218 (still active) | translated, **not wired** | Workspace only | 3× |
| **mouclass.sys** | 2,543 | 1,172 (still active) | translated, **not wired** | Workspace only | 3× |

*Bindgen FFI substrate crates (`ntkrnl-sys`, `win32k-sys`, `ntdll-sys`,
etc.) are generated, not translation work, and are excluded.*

---

## ntoskrnl detail (the headline number)

- Original C body of work (excl. shims): **268,509 LOC**.
- Rust under `ntoskrnl-rust/src`: **453,404 LOC** across **406 .rs files**.
- **Active non-shim C: 0.** Only 8 header-free shim files compile:

  | Shim file | LOC | Why it stays C |
  |---|---:|---|
  | `ke/ke_rust_shims.c` | 251 | minimal trampoline; amd64 `KiSave/RestoreXState` (KVM xrstors64 fault) |
  | `ntprintf_trampoline.c` | 170 | variadic `va_arg` pull (Rust `extern "C-variadic"` unstable) |
  | `io/iomgr/io_rust_shims.c` | 154 | probes + variadic |
  | `mm/mm_rust_shims.c` | 152 | ARM64-only PTE/IRQL debug |
  | `kdbg/kdbg_rust_shims.c` | 91 | `KdbpPrint` + `KdbPromptStr` variadics |
  | `lto_workaround.c` | 87 | i386 fat-LTO `ud2` miscompile isolation (`SepInitTokensShim`) |
  | `variadic_shims.c` | 77 | `KdpDprintf` + `RtlStringCbPrintfA` |
  | `mm_lock_outlines.c` | 55 | C `__FILE__`/`__LINE__` capture |
  | **Total** | **1,037** | |

On the strict "translatable body" metric the ~1,037 shim LOC are mostly
irreducible, so calling ntoskrnl **~99.7–99.9% done** is fair — the
remaining fraction is language/toolchain limits (variadics, one LTO bug,
two hardware/KVM workarounds), not un-ported logic.

*One open item: `rtl/path.rs` (5,486 LOC) is on disk but not wired into
`rtl/mod.rs` — a parked translation worth wiring or documenting.*

---

## The remaining frontier (what's actually left)

Ordered by size of the compiled-C surface:

### 1. win32k USER core — ~74K C — the active frontier
- **GDI is 100% done**: all 40 NtGdi* files, the graphics engine/DDI
  (37 files incl. `ldevobj.c`), and all 14 per-bit-depth DIB renderers are
  flipped. Includes the long poles: `freetype.c` glue (FreeType vendor
  stays C), `font.c`, `path.c`, `bitblt`, `d3dkmt` dxgkrnl thunks, and
  `brush.cpp` (first C++→Rust flip).
- **USER is ~4/59 files (~7%)**: only `object.c`, `session.c`,
  `useratom.c`, `guicheck.c` flipped. The hard half — window-create chain,
  message queues, hooks, callouts, desktop/winsta — is ~74K LOC still C.
- **Blocker:** the Phase 1.5 typestate callout-safety API
  (`UserLock`/`CoRef`/`CoCallout`) is **not started** and gates the deep
  USER core. This is the single largest remaining translation effort.

### 2. rtl C bodies — ~53K C (~30% flipped)
Still C in `librtl.a` (largest first): `actctx.c` (6,214, SxS), heap family
(`heap.c`/`heappage.c`/`heapdbg.c`/`heapuser.c`, ~7K — deferred to a
behavioral/Kani track, never byte-diffable), `threadpool.c` (3,516),
`unicode.c` (2,852), per-arch exception engine (`except.c`/`unwind.c`),
`registry.c`, `image.c` (PE primitives). The string/sid/acl/sd/path/env
families and all pure-algorithm `.c` are already Rust.

### 3. libcntpr / kernel CRT — ~53K C (~10–17% flipped via `cnt-rust`)
Own track. Large surface, low priority while it boots.

### 4. Smaller static-lib C surfaces
`libcmlib.a` (~8.8K), `librossym.a` (~3.1K), `libpseh.a` (~2.6K),
`libsetjmp`, `libcportlib`, `libcsq`, `libarbiter`, `libntlsalib`. Plus
**3 GUID-only libs** (`wdmguid`/`poguid`/`ioevent`, ~43 LOC total) that are
trivial `#[link_section]` flips.

### 5. Loose ends (small, mechanical)
- **kbdclass/mouclass** — fully translated but **not wired**; their
  CMakeLists still build the C. ~2.4K C compiled needlessly.
- **smlib/csrlib client flip** — Rust exists and server-side consumers use
  it; only the client-side link into ntdll/ntdll_vista is pending.
- **arm64 HAL** — only 1,007 LOC of stubs exist; a real arm64 HAL
  (GIC + Generic Timer + PSCI + SBBR ACPI) is a separate future project.

---

## Deliberately-still-C set (by design — not translation debt)

Language / toolchain / vendor boundaries, **not** translation candidates:

- **Variadic export trampolines** (ntoskrnl shims, `ntprintf_trampoline.c`):
  Rust `extern "C-variadic"` is unstable.
- **`SepInitTokensShim`** (i386): LLVM fat-LTO inserts `ud2`; C-side
  execution outside the Rust LTO scope is the only working fix.
- **`KiSave/RestoreXState`** (amd64): KVM `xrstors64` fault → forced
  `fxsave64`/`fxrstor64` in C.
- **Asm trampolines**: ntdll `dispatch/{arch}/dispatch.S` (kernel→user trap
  entry), win32k `sys-stubs.S` (auto-gen syscalls), HAL `*.S`, slist arch asm.
- **i386 kernel-mode fixed-point math**: win32k `gdi/eng/i386/floatobj.c` —
  no kernel-mode SSE on i386.
- **Vendor code**: FreeType (glue only; library stays C), autogenerated
  PCI-ID tables, bin2c data blobs.
- **BIOS v86 thunks**: FreeLDR `bios.c`/`x86bios.c` (setjmp/longjmp from
  asm-entered frames; dead under UEFI anyway).
- **arm64 HAL stubs** and **DirectDraw/dxg kernel side** — out of scope.

---

## Smoke status

All three architectures boot to `shell ready` (i386/amd64) / idle (arm64)
on the default cohabitation build. Latest smoke screenshots:
`qemu/{i386-uefi,amd64-uefi,arm64}_smoke.png` (May 30 – Jun 1).

Open notes carried from component audits:
- rtl: an amd64 post-`shell ready` services.exe init hang is the open
  blocker in the rtl track (PATH-wave).
- win32k USER half being mostly C is *why* smoke passes — the unflipped
  core is still the live code path.

---

## How the whole-OS picture nets out

The kernel, boot spine, HAL(x86), ntdll, and the smss/csrss/win32k-GDI
user-mode spine are **Rust and shipping in the default build on all 3
arches**. The translation project is no longer about the kernel — that's
done bar irreducible shims. It's now three concentrated C bodies:
**win32k USER (~74K)**, **rtl (~53K)**, **libcntpr (~53K)**, plus mechanical
wiring loose ends. win32k USER is the critical path, gated on the typestate
callout-safety API.

*Per-component detail lives in the area-specific audits under
`docs/engineering/` (`ntoskrnl-`, `hal-`, `freeldr-`, `ntdll-`, `rtl-`,
`static-libs-`, `win32k-`, `smss-csrss-translation-audit.md`).*

---
---

# Appendix A — Project Timeline

*From the ReactOS fork to the Rust port. Reconstructed from git history
(`continue-rust-port`, 902 commits). Dates are author dates.*

> **Fork shape:** this is a *squashed/grafted* fork — it shares exactly one
> commit with upstream ReactOS (`5213cf71`), not the full upstream history.
> Everything after is original work.

## At a glance

```
2026-02-11  ●  Fork from ReactOS  (commit 5213cf71, "Remove Environ #8667")
2026-02-12  │  ── ARM64 PORT BEGINS (lambdax) ──
   …4 days  │  Milestones 1–16: boot → desktop → CreateProcessW on ARM64
2026-02-18  │  Multi-arch: i386 + amd64 boot to desktop (still pure C)
2026-02-19  ●  ── RUST PORT BEGINS ──  SList + Bitmap, 3 arches
2026-02-20  │  rustconv translation pipeline born
2026-02-21  │  ── SUBSYSTEM BLITZ (meaniepy) ──  ob, se, ps …
   …8 days  │  tier1, cc/fsrtl, mm, ex, ke — 8 subsystems, ~216 C files
2026-02-28  │  "WIP bulk fixes — DOES NOT PASS SMOKE"  (integration wall)
2026-03 ─04 │  ── STABILIZATION GRIND ──  get all 3 arches green again
2026-05     ●  ── THE MAY EXPLOSION ──  663 commits: HAL, rtl, ntdll,
            │     win32k-GDI, smss/csrss, winsrv, pnpmgr, freeldr
2026-06     │  Periphery cleanup; win32k USER core is the open frontier
```

Commit cadence by month: **Feb 45 · Mar 89 · Apr 29 · May 663 · Jun 76.**

## Phase 0 — The Fork (2026-02-11)

`5213cf71` — `[BOOT][ENVIRON] Remove Environ (#8667)`, Justin Miller. An
upstream ReactOS commit; the project branched here and squashed the prior
history. The single shared ancestor with `upstream/master`.

## Phase 1 — The ARM64 Port (2026-02-12 → 02-18) — *"the arm port, not a rust port"*

All by **lambdax**, in a ~4-day sprint. Pure-C ReactOS gaining a brand-new
AArch64 architecture, milestone by milestone:

| Date | Milestone |
|---|---|
| 02-12 | Add ARM64 (AArch64) architecture port |
| 02-13 | FreeLDR ARM64 boot fixed; `KiSystemStartup` → `ExpInitializeExecutive` |
| 02-13 | Kernel VA page tables + Phase 0 MM; context switch + Phase 1 dispatch |
| 02-13 | VBAR_EL1 exception vectors; APC + SVC dispatch (Phase 1 complete) |
| 02-14 | Ramdisk boot, process creation, SMSS launch (M7); EL0 user-mode (M8) |
| 02-14 | User APC, RTL exceptions, COW fix, winlogon boot (M9–10) |
| 02-14 | MM session space + win32k.sys loading (M11); user exception dispatch (M12) |
| 02-15 | Full Win32 DLL chain in winlogon (M13–15); user-mode `CreateProcessW` (M16) |

By M16 the ARM64 port booted to a desktop and spawned processes — entirely
in C. This is the era the project was "an ARM port."

## Phase 2 — Multi-Arch (2026-02-18 → 02-19)

The pivot that set up everything after: a **unified llvm-mingw toolchain**.

- 02-18 — `multi-arch: unified llvm-mingw toolchain — i386 and amd64 boot to full desktop`
- 02-19 — `tests: add kernel test infrastructure + multi-arch QEMU smoke tests`

The project became tri-arch **before** the Rust work began — which is why
every Rust component since has had to target all three.

## Phase 3 — Rust Begins (2026-02-19 → 02-20)

- **02-19** — `rust: cross-platform SList + Bitmap kernel components for all 3 architectures` ← **the first Rust in the kernel** (lambdax)
- 02-19 — `rust: port 19 RTL crates + kernel pool allocator to Rust`
- **02-20** — `rustconv: API-driven C-to-Rust translation pipeline + ob/obhandle.rs` ← the engine that powered everything after
- 02-21 — **meaniepy** takes over as primary author (`kernel-rust` umbrella crate)

## Phase 4 — The Subsystem Blitz (2026-02-21 → 02-28)

Eight kernel subsystems translated in eight days via the fresh rustconv
pipeline — ~216 C files:

| Date | Subsystem | Files |
|---|---|---:|
| 02-21 | **ob** (object manager) | 12 |
| 02-21 | **se** (security) | 18 |
| 02-22 | **ps** (process) | 13 |
| 02-22 | **tier1** (lpc, dbgk, po, inbv, wmi, vdm, vf, rtl) | 26 |
| 02-23 | **cc + fsrtl** (cache + FS runtime) | 24 |
| 02-26 | **mm** (ARM3 + mm) | 30 |
| 02-26 | **ex** (executive) | 31 |
| 02-26 | **ke** (kernel) | 62 |

**02-28** — `WIP: bulk fixes across all Rust subsystems — DOES NOT PASS SMOKE TESTS`.
The honest integration wall: everything translated, nothing booting yet.

## Phase 5 — The Stabilization Grind (March – April 2026)

Low commit volume (Mar 89, Apr 29), high difficulty — making the
bulk-translated kernel actually boot green on all 3 arches. Cross-model
translation entered here to catch blind spots:

- 03-20 — `iofunc.rs: complete translation … via Codex (GPT-5.4)`
- 03-22 — `migrate 40+ lock patterns to RAII guards across MM, IO, KE`
- Late Mar → mid-Apr: near-silent (weeks of 4, ~0, ~0, 5 commits) — deep
  debugging, little to commit. This is where the "re-probe before trusting
  any stay-in-C classification" discipline was forged.

## Phase 6 — The May Explosion (May 2026) — 663 commits

With the kernel core stable, every peripheral component got translated,
often via bulk LLM passes. Weekly: W18 49 · **W19 177 · W20 223 · W22 227**.

| Date | Landmark |
|---|---|
| 05-01 | `mm/ARM3/virtual`, `pnpmgr/pnpres`, `iomgr/arcname` full ports on all 3 arches |
| 05-01 | `kdbg/i386-dis` full Rust port (Gemini 3 Pro) |
| 05-09 | `rtl-rust`: 18 algorithm rlibs consolidated under one umbrella |
| 05-14 | **HAL takeover**: pic/pcibus swap; `mini_hal` phase 2 complete (17/17 .c retired) |
| 05-16 | **win32k Phase 1 complete** (USER object/handle table → Rust); **smss-csrss** csrsrv.dll green ×3 |
| 05-24 | FreeLDR boot-debug marathon — captures(none)/ABI ud2 fixes unblocking MmInit |
| 05-29 | Pre-Vista prune: drop NTVDM/MVDM 16-bit subsystem |
| 05-31 | **smss full Rust flip** (Opus 4.8) |

## Phase 7 — Periphery & Cleanup (June 2026 →)

- 06-01 — `userinit` full flip; `rtl PATH wave` complete (27/27)
- 06-02 — `winlogon` leaf set complete (Opus 4.8)
- **Open frontier:** win32k **USER core** (~74K C) — GDI 100% done, USER ~7%,
  gated on the typestate callout-safety API (see Part 1).

## Contributors

| Author | Commits | Role |
|---|---:|---|
| **meaniepy** | 870 | Primary — entire Rust era (Feb 21 →) |
| **lambdax** | 30 | ARM64 port + first Rust components (Feb 12–20) |
| Justin Miller | 1 | Upstream ReactOS fork commit |

*`lambdax` and `meaniepy` are the same developer across git identities.*

## Translation tooling (by commit-message attribution)

The Rust era leaned on multiple models, picked for cross-model blind-spot
coverage — the strategy is legible in the git log:

| Tool / model | Attributed commits |
|---|---:|
| **Opus 4.8** | ~173 |
| **Codex** (GPT-5 / GPT-5.4) | ~52 |
| **Sonnet** | ~42 |
| **Gemini** (incl. 3 Pro) | ~12 |
| Haiku | 1 |

Counts only commits that *name* the tool; the true automated share is higher
(the rustconv batch pipeline default is Sonnet).

## The arc in one paragraph

A squashed ReactOS fork (Feb 11) became a from-scratch **ARM64 port** in a
four-day milestone sprint (Feb 12–15), then went **tri-arch** on a unified
llvm-mingw toolchain (Feb 18). The day after smoke tests landed, the **Rust
port began** (Feb 19) with SList/Bitmap and the rustconv pipeline, followed
by an eight-day **kernel subsystem blitz** (ob→ke, ~216 files) that out-ran
integration — culminating in the candid "DOES NOT PASS SMOKE" commit
(Feb 28). Two months of **stabilization** (Mar–Apr) made it boot again.
Then **May exploded** (663 commits) across HAL, rtl, ntdll, win32k-GDI,
smss/csrss, and freeldr, driven by bulk LLM translation. By June the kernel
is ~99.7% Rust and the work has moved to the win32k USER core — the last
large C body on the critical path.
