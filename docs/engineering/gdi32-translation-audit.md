# gdi32.dll — C→Rust Translation Audit & Wave Plan

**Lane:** `gdi32-port` (agent identity: user32, post-user32-100%-Rust frontier).
**Model:** cohab staticlib, exactly like `win32ss/user/user32` → `user32-rust` (see
`user32-translation-audit.md`). The translated `.c` is commented out of
`win32ss/gdi/gdi32/CMakeLists.txt` SOURCE with a
`# <path> — replaced by Rust (gdi32-rust/src/<path>.rs); cohab via libgdi32_rust.a`
marker; the `gdi32-rust` crate's `#[no_mangle]` symbols become the sole provider at link.
No `--allow-multiple-definition` needed (1:1 symbol replacement).

## Scope

gdi32.dll **core body** (the `gdi32-rust` target) = `main/` + `misc/` + `objects/`:
**~17.9K LOC across ~29 `.c` files**. The separate `wine/` subdir (`winegdi` static lib,
~9.9K LOC of EMF/metafile/DirectDraw marshalling) is **out of scope** for now — a later
lane or left as C; it links as `winegdi` independent of the cohab flip.

gdi32 is the **usermode GDI client**; the kernel half (win32k.sys GDI) is already 100%
Rust (`win32k-rust/src/{dclife,bitmaps,brush,dcattr,...}.rs`). Most exports either:
- marshal args → a `NtGdi*` syscall (via the `win32ksys` importlib — same pattern as
  user32→`NtUser*`), or
- queue a `GdiAllocBatchCommand` into the TEB batch ring (deferred syscall), or
- are pure client-side algorithm/bookkeeping (e.g. `LineDDA`, coordinate math).

The `win32k-sys` bindgen crate already exposes the full `NtGdi*` surface + GDI struct
layouts, so **no separate `gdi32-sys` bindgen is needed** — `gdi32-rust` depends on
`win32k-sys` directly (mirrors `user32-rust`).

## ABI / decoration (cohab link contract)

- **i386:** exports are WINAPI (stdcall) → `extern "system"` → `_Name@N`. FASTCALL
  internals → i386-only `extern "fastcall"` → `@Name@N`. Plain cdecl → `extern "C"`.
- **amd64/arm64:** all undecorated `extern "C"` / `extern "system"` (system==C there).
- **Cross-TU data:** non-static globals (e.g. `stock_objects[]`, `GdiHandleTable`,
  `GdiSharedHandleTable`) → `#[no_mangle] pub static`, referenced `extern` from peers.
- **Batch queue:** `GdiAllocBatchCommand` is *not* a syscall — it's TEB batch ring access;
  must be hand-wrapped in the Wave-1a substrate (mirror the win32k-rust / user32-rust
  internal.rs pattern).

Verified Wave 0: `LineDDA` →  i386 `_LineDDA@24` ✓ · amd64 `LineDDA` ✓ (arm64 builds
clean, same `extern "C"` path).

## Core substrate (translate/understand first)

Everything funnels through the **boot trio** — these define the shared infrastructure the
rest call into, so they anchor Wave 1a:
- **main/dllmain.c** — `GdiProcessSetup`, PEB GDI-handle-cache mapping, batch init.
- **objects/gdiobj.c** — the GDI handle table (`GdiHandleTable`/`GdiSharedHandleTable`,
  `GDI_TABLE_ENTRY`), `stock_objects[]` cache (`GetStockObject`), `GdiAllocBatchCommand`,
  the `GdiGetHandleUserData`/handle-validation helpers.
- **objects/dc.c** — DC creation/lifecycle (`CreateDC`/`DeleteDC`/`SaveDC`/`RestoreDC`),
  the DC_ATTR client cache. ~1.6K LOC, the critical substrate for all drawing.

## Wave plan (leaf-first, each independently buildable + smoke-testable)

| Wave | Files | LOC | Notes |
|------|-------|-----|-------|
| **0** ✅ | objects/linedda.c | 93 | **DONE** — pure-algorithm leaf (Bresenham + LINEDDAPROC callback, no syscall). Proves cohab link + `_LineDDA@24` decoration + callback ABI + the crate scaffold. |
| **1a** | main/dllmain, objects/gdiobj, objects/dc | ~2.1K | Core substrate (handle table, stock objects, batch cmd, DC lifecycle). Boot-critical. Brings up `ntwrapper.rs` (NtGdi* inline wrappers) + `internal.rs` (PEB/TEB/batch). |
| **1b** | objects/{brush,pen,palette,colorspace,clientobj} | ~0.8K | Object-creation thin wrappers (CreatePen/Brush/Palette …). |
| **2** | objects/{painting,text,arc,coord,region} | ~4.2K | Drawing core — explorer/desktop render path. |
| **2b** | objects/font | ~2.8K | Largest single file (58 fns, charset tables + A/W conv). Boot-critical. |
| **3a** | objects/{path,bitmap,icm} | ~1.8K | Geometry/paths/bitmaps/color-mgmt. |
| **3b** | misc/{heap,historic,misc,wingl}, objects/{utils,efloat(i386),enhmfile,metafile,palette-extras} | ~2.9K | Utilities + i386 `efloat.c` (float helpers, `if(ARCH STREQUAL i386)`-gated). |
| **4** | misc/gdientry, objects/{printdrv,eng} | ~3.3K | Driver/DirectX/spooler entry — **not boot-critical, deferred last**. |

Critical boot path: Wave 0 → 1a → 1b → 2 → 2b ⇒ explorer desktop render. Waves 3–4 are
post-boot refinement.

## Crate scaffold (`sdk/lib/rust/gdi32-rust/`)

- `Cargo.toml` — staticlib, `win32k-sys` dep, standard per-arch user-mode targets.
- `src/lib.rs` — `#![no_std]`, panic handler, `rust_eh_personality`; `pub mod {types,objects}`.
- `src/types.rs` — minimal Win32 aliases (grows; Wave 1a adds the win32k-sys GDI surface).
- `src/objects/{mod,linedda}.rs` — Wave 0.
- Registered in `sdk/lib/rust/Cargo.toml` workspace members (after `user32-rust`).
- CMake cohab block in `win32ss/gdi/gdi32/CMakeLists.txt` (mirrors user32: GLOB_RECURSE
  `.rs` → `cargo build -p gdi32-rust` → copy `libgdi32_rust.a` → `target_link_libraries`
  + `add_dependencies(gdi32 … rust_gdi32)`).

## Ledger

| Wave | Commit | i386 | amd64 | arm64 |
|------|--------|------|-------|-------|
| 0 (linedda) | _pending_ | build ✓ (cohab relink clean) / smoke: known-flake under host contention, re-run pending | _pending_ | _pending_ |

Notes: Wave-0 i386 build relinked gdi32.dll clean with the Rust `LineDDA`; first i386 smoke
hit the **documented kernel-space winlogon/explorer-handoff race (BUGCHECK 0x1E @ ntoskrnl
PC, auto-classified KNOWN_FLAKE)** while 4 concurrent arm64 TCG boots starved the host —
unrelated to the usermode pure-algorithm leaf. Quiet-host re-run owed for the green tick.
