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

**Correction (verified by reading the bodies):** the *true* substrate root is **misc/misc.c**,
not gdiobj.c. gdiobj.c (only 369 LOC) sits **on top of** the handle-table/batch primitives,
which are defined in misc/misc.c over the PEB-mapped `GdiSharedHandleTable` (`GDI_TABLE_ENTRY`
layout in `win32ss/include/ntgdihdl.h`; client-table defs in `include/gdi32p.h`). So Wave 1a
anchors on:
- **misc/misc.c** (~1.1K LOC, ~42 fns) — the substrate root: `GdiValidateHandle` (712),
  `GdiFixUpHandle` (734), `GdiQueryTable` (756), `GdiGetHandleUserData` (761), `GdiFlush` (44),
  `GdiSAPCallback`, `GdiInitializeLanguagePack`. The PEB-mapped handle-table accessors that
  everything in objects/ calls. **Most error-prone → hand-review the translation.**
- **main/dllmain.c** (140) — `GdiProcessSetup`, PEB GDI-handle-cache mapping, batch init.
- **objects/gdiobj.c** (369) — `stock_objects[]` cache (`GetStockObject`), `GetObjectType/A/W`,
  `DeleteObject` dispatch. Thin layer over the misc.c primitives + `NtGdi*`.
- **objects/dc.c** (~1.6K) — DC creation/lifecycle (`CreateDC`/`DeleteDC`/`SaveDC`/`RestoreDC`),
  the DC_ATTR client cache. The critical substrate for all drawing.

`GdiAllocBatchCommand` (TEB batch ring) is referenced from gdiobj.c/dc.c/text.c but defined
as an inline/helper in the gdi32 headers — must be hand-wrapped in the gdi32-rust substrate.

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
| 0 (linedda) | `0c811db111a` | build+boot ✓ (81s/73s quiet) | (gate) | (gate) |
| 1a (dllmain+misc+gdiobj) | `94e0fd5972a` | build+boot ✓ (73s) | build+boot ✓ (188s, net✓) | cohab-link ✓; gdi32 proven loaded (1st boot reached `mpr wnetInit` late-usermode); smoke blocked by env (KNOWN_FLAKE handoff race, then efivars wrong-X64-image — arm64-love's documented media issue, not gdi32). Gate will verdict on controlled host. |
| 1b (brush+pen+palette+colorspace+clientobj) | `7d40b6e83b2` (landed `76312282832`) | boot ✓ | boot ✓ (net✓) | boot ✓ 33s (genuine Rust, post archcfg-fix) |
| 2 (coord+painting+text+arc+region) | landed `eaae890a98f` | build+boot ✓ 72s | build+boot ✓ 178s (net✓) | build+boot ✓ 33s | **×3 GREEN** (desktop render path) |
| 2b (font) | landed `704590ef795` | build+boot ✓ 1m13s | build+boot ✓ (net flake unrelated — gdi32 has no net surface) | build+boot ✓ 43s (cleared known wnetInit flake on re-run) | **×3 boot-GREEN** (largest single file) |
| 3a (path+bitmap+icm) | `5f5274f26d7` (gating) | build+boot ✓ | build+boot ✓ (net✓) | build+boot ✓ 33s (cleared known wnetInit flake on re-run) | **×3 boot-GREEN** (paths/DIB/color-mgmt) |
| 3b (dc+utils+heap+historic+wingl+efloat+enhmfile+metafile) | landed `1f5868cc4e0` | build+boot ✓ | build+boot ✓ (net✓) | build+boot ✓ 34s (first try) | **×3 boot-GREEN** (DC lifecycle + utils + EMF/WMF + i386 efloat) |
| 4 (gdientry+printdrv+eng) | _pending_ | build+boot ✓ | build+boot ✓ (net✓) | build+boot ✓ 34s (first try) | **×3 boot-GREEN** — 🎉 **gdi32 100% Rust** |

### Wave 4 notes (the final trio — gdi32 → 100% Rust)
3 files (~3.3K LOC, ~89 fns) via 3 parallel opus agents against
`~/vibeos-rust-staging/gdi32-wave4/CONTRACT-4.md`. **gdientry.c is heavy DirectDraw DDI** (the
DDHAL surface callbacks + the 16 GDI entry points). 3 distinct integration fixes:
- **The `GdiEntryN` naming (critical, link-breaking if wrong):** `sdk/include/psdk/ddrawgdi.h`
  `#define`s `DdCreateDirectDrawObject`..`DdSetGammaRamp` → `GdiEntry1`..`GdiEntry15` (and
  `DdSwapTextureHandles`→`GdiEntry16`), so the compiled/exported symbols are `GdiEntryN`. Rust has
  no preprocessor → the 15 fns are **named `GdiEntryN` directly** in gdientry.rs (GdiEntry16 stays
  in historic.rs). The other ~44 `Dd*`/`Dvp*`/`D3d*` HAL callbacks keep their real names. All
  `@N` decorations matched the spec on the first link.
- **The `Unlock` const-pattern footgun:** `DdUnlock`/`DdUnlockD3D` had a param named `Unlock`, which
  collides with the glob-imported `const Unlock: _SECURITY_LOGON_TYPE` → Rust parsed the param as a
  const-pattern (binding nothing) and the body saw a `u32`. Renamed `Unlock`→`pUnlock`. New
  field-manual entry (this is a SILENT bug when the const type is compatible).
- **bitmap-union + cast nits:** `ddsCapsEx.dwCaps4` → `.__bindgen_anon_1.dwCaps4` (union); printdrv
  `ghSpooler = hModWinSpoolDrv as HANDLE` (HINSTANCE→HANDLE).

**🏆 gdi32.dll is now 100% Rust** (29/29 .c translated, cohab-flipped, ×3 boot-green). The usermode
GDI client joins ntoskrnl, win32k, kernel32, user32, cdfs, cdrom, fastfat, usbport, … in Rust.

### Wave 3b notes (substrate + utilities — completes everything bar the Wave-4 driver/spooler trio)
8 files (~3.1K LOC, ~103 fns) via 8 parallel opus agents against
`~/vibeos-rust-staging/gdi32-wave3b/CONTRACT-3b.md`, integrated centrally — only **1 integration
error** (the `GdiBCSelObj` batch-const glob ambiguity in dc.rs → `crate::gdihelp::GdiBCSelObj`):
- **dc.rs** (51 fns) — DC lifecycle + device caps (Create/Delete/Save/Restore/ResetDC,
  GetDeviceCaps, Get/SetDC{Brush,Pen}Color, Get/SetBkColor/Mode/ROP2/PolyFillMode/GraphicsMode,
  GetCurrentObject, SelectObject, bounds-rect, etc.). DC-attr-cache heavy. The boot-critical one.
- **utils.rs** (6 fns) — CalculateColorTableSize, ConvertBitmapInfo, LogFontA2W/W2A,
  EnumLogFontExW2A, LoadLPK + the 4 owned LPK fn-ptr globals (text.rs externs them).
- **heap.rs** (3 fns) — HEAP_strdupA2W(_buf) + the `hProcessHeap` global (font.rs externs the buf).
- **historic.rs** (23 fns), **wingl.rs** (5 pixel-format fns + 2 lazy-loaders), **enhmfile.rs**
  (6 EMF fns), **metafile.rs** (5 WMF fns) — thin client wrappers.
- **efloat.rs** (2 FASTCALL fns, **i386-only** `#[cfg(target_arch = "x86")] pub mod efloat;`) —
  EFtoF/FtoEF. FLOATL is an integer (float-bits), so all-integer ABI, no float-ABI issue.
After this lands, only the **Wave 4** driver/spooler trio remains C: `misc/gdientry.c` (2084 LOC),
`objects/printdrv.c` (944), `objects/eng.c` (264) — driver/DirectX/spooler entry, not boot-critical.

**2 link fixes during 3b integration (both worth a field-manual note):**
1. **A `.spec`-exported symbol provided as a hidden OBJ-LEVEL alias by the flipped .c.** gdi32.spec
   exports ordinal 273 `GdiEntry16` (stdcall, 3 ptr), but no `.c` contains the text "GdiEntry16" —
   the symbol `_GdiEntry16@12` was emitted *into historic.c.obj* as an alias of `DdSwapTextureHandles`
   (the "GDIEntry 16" comment). Flipping historic.c→.rs dropped the alias → `undefined symbol:
   _GdiEntry16@12` at link (the crate compiled fine — it's a link-only break). **Fix:** re-provide
   `GdiEntry16` explicitly in historic.rs as a `#[no_mangle] extern "system"` export forwarding to
   `DdSwapTextureHandles` (all-arch — the spec export is arch-independent). **Detection recipe:** when
   a cohab flip yields `undefined symbol: _Foo@N` for a symbol absent from every `.c`, it was an
   obj-level alias — grep the gdi32.spec/`good api.def` for the ordinal and `nm` the pre-flip `.obj`
   to find the real target. **⚠️ Wave 4 will hit this for `GdiEntry1..15`** (gdientry.c aliases the
   `DdXxx` HAL fns) — each needs an explicit forwarding export.
2. **A blanket "this family is cdecl" ABI assumption that's wrong for one member.** The contract said
   "EMFDC_* are cdecl (`extern "C"`)", but `wine/emfdc.c` declares `BOOL WINAPI EMFDC_GdiComment` —
   stdcall (`_EMFDC_GdiComment@12`). The `extern "C"` decl → undefined `_EMFDC_GdiComment`. **Fix:**
   `extern "system"`. Lesson: verify each cross-file callee's *own* decoration against its C def; don't
   trust a family-wide rule.

### Wave 3a notes (path / bitmap / icm)
3 files (~1.8K LOC, 45 fns) via 3 parallel opus agents against
`~/vibeos-rust-staging/gdi32-wave3a/CONTRACT-3a.md`, integrated centrally. **2 integration
fixes**, both the i386 float-ABI / inline-not-export class (now captured in the field manual
§"i386 no-SSE float ABI"):
- **path.rs `SetMiterLimit`** — an EXPORTED WINAPI fn taking `FLOAT` **by value** (the only such
  export in the wave). i386 `-sse,-sse2` rejects an f32 value across a fn boundary → **per-arch
  cfg split**: i386 takes the float's stack bytes as `u32` (bit-identical to the C stdcall float
  arg) and passes them straight through; amd64/arm64 use the native `f32` sig (float in a vector
  reg). Mirrors the C `gxf_long worker; worker.f=a1; …(worker.l)…` bit-reinterpret.
- **bitmap.rs `RtlULongMult`** — a `__forceinline` from ntintsafe.h (NOT an ntdll export) →
  reimplemented locally as a checked u32 multiply (matches `win32k-rust/src/surface.rs` + font.rs).
- **bitmap.c** is the meaty one (DIB sections, GetDIBits/SetDIBits/StretchDIBits, the FASTCALL
  size helpers); `GdiGetBitmapBitsSize` kept `#[no_mangle] extern "system"` because utils.c (still
  C) links against it. **icm.c** = color-space/ICM-profile thin wrappers + A/W conversion.

### Wave 2b notes (font.c — the largest single file)
ONE file (~2.8K LOC, 70 exported + 11 private fns) via a single opus agent against
`~/vibeos-rust-staging/gdi32-wave2b/CONTRACT-2b.md`, integrated centrally. Font selection
(CreateFont(Indirect)A/W), enumeration (EnumFontFamilies(Ex)A/W + qsort/IntFontFamilyCompare
callback), metrics (GetOutlineTextMetrics, GetTextExtent*, GetCharWidth/ABCWidths incl. the
`*Float` PFLOAT/LPABCFLOAT variants), glyph (GetGlyphOutline/Indices), char placement, and
font-resource add/remove (incl. IntConvertFontPaths). **3 integration fixes** (all the i386
float-ABI class + one inline-not-export):
- **`GDI_ROUND(FLOAT)`** and **`dabs(f64)`** were translated as fns → i386 `-sse,-sse2`
  rejects an f32/f64 *value* crossing a fn boundary (rustc errors at the definition, even
  `#[inline]`). → both converted to `macro_rules!` (math stays in-body, x87). GDI_ROUND made
  **f64-native** (C computes the world-to-viewport transform in `double` throughout, then
  `(int)floor(val+0.5)`; the agent's initial f32 narrowing would've lost precision). Same
  pattern as coord.rs `_fmul!`/`_lrintf!`. **NB: FLOAT *pointers* (PFLOAT/LPABCFLOAT) and
  FLOAT struct fields are fine — only by-value f32/f64 params/returns are forbidden.**
- **`RtlULongMult`** declared `extern` → it's a `__forceinline` from ntintsafe.h, NOT an
  ntdll export → undefined `_RtlULongMult@12` at link. → reimplemented locally as a checked
  u32 multiply (STATUS_SUCCESS / 0xC0000095 STATUS_INTEGER_OVERFLOW), matching
  `win32k-rust/src/surface.rs`.

### Wave 2 notes (desktop render path)
5 files (~4.2K LOC, ~122 fns) via 5 parallel opus agents against `~/vibeos-rust-staging/gdi32-wave2/CONTRACT-2.md`, integrated centrally — only **2 compile errors** across 4.2K LOC (contract pattern dialed in):
- **coord.rs** (28 fns) — viewport/window org+ext, DPtoLP/LPtoDP, World/ModifyWorldTransform, CombineTransform, layout, DC org. DC-attr-cache heavy.
- **painting.rs** (31 fns) — MoveToEx/LineTo/Poly*/Rectangle/Ellipse/RoundRect/Fill·Frame·Invert·PaintRgn/Flood·SetPixel/Blt family (Bit/Pat/Stretch/Mask/Plg/Alpha/Transparent/Gradient). Batch GdiBCPatBlt/PolyPatBlt.
- **text.rs** (33 fns) — TextOut/ExtTextOut(batch)/PolyTextOut/GetTextExtent*/GetTextMetrics/Face/Align/Color, A↔W via Rtl/WideCharToMultiByte.
- **arc.rs** (5 fns) — Arc/ArcTo/Chord/Pie/AngleArc.
- **region.rs** (25 fns) — CreateRectRgn/Elliptic/RoundRect/Polygon/CombineRgn/clip-region ops; **defines `DeleteRegion` (fastcall)** — resolves gdiobj.rs's extern.

Integration fixes: `ExtTextOutW` cchString INT→UINT cast; **i386 no-SSE float-ABI** — coord's `_fmul`/`_lrintf` (both C *macros*) were translated as Rust fns taking `FLOAT`(f32) by value, which on i386 `-sse,-sse2` (i686-CPU-baseline rule) has no ABI register → converted them to Rust `macro_rules!` (faithful; f32 stays inside bodies via x87, never crosses a fn boundary; `FLOATL=DWORD` float-as-bits so `FOtoF`/`EFtoF` were fine). New field-manual candidate: **i386-no-SSE forbids f32 at fn boundaries → translate C float macros as Rust macros, not fns.**

### Wave 1b notes (object-creation wrappers)
5 files (~1K LOC, 38 fns) via 3 parallel opus agents against `~/vibeos-rust-staging/gdi32-wave1b/CONTRACT-1b.md` (addendum to the 1a contract: substrate now in-crate). Object-creation NtGdi* wrappers:
- **brush.rs** (13 fns) — Create{Solid,Hatch,Pattern,DIBPattern}Brush, CreateBrushIndirect, Get/Set/ClearBrushAttributes, BrushOrg (batch GdiBCSetBrushOrg), UnrealizeObject.
- **palette.rs** (11 fns) — CreatePalette, Get/SetPaletteEntries, RealizePalette, ResizePalette, UpdateColors, Get/SetDIBColorTable, AnimatePalette (NtGdiDoPalette).
- **pen.rs** (2 fns) — CreatePen, CreatePenIndirect.
- **colorspace.rs** (7 fns) — GetLogColorSpaceA/W, Get/SetDeviceGammaRamp, CheckColorsInGamut, ColorMatchToTarget, SetColorAdjustment.
- **clientobj.rs** (5 fns) — GdiCreate/Get/Remove/Delete ClientObjLink + Create/Delete ClientObj; **DEFINES `gcClientObj` (resolves gdiobj.rs's extern) + `gcsClientObjLinks` (resolves dllmain.rs's extern)**.

Integration: only 3 errors total (contract reuse paid off) — `GdiBCSetBrushOrg` ambiguous → gdihelp-qualified (the Wave-1a GdiBC* pattern); `pcol` immutable-reassign → `mut`. Compiles + cohab-relinks clean ×3.

### Wave 1a notes (substrate)
3 files via parallel opus agents against `~/vibeos-rust-staging/gdi32-wave1a/CONTRACT.md`, integrated centrally:
- **misc.rs** (29 fns) — the handle-table substrate root + `GdiHandleTable`/`GdiSharedHandleTable`/
  `CurrentProcessId`/`GDI_BatchLimit` globals (`#[no_mangle] pub static mut`), escapes, batch-limit,
  language pack. **gdihelp.rs** — the gdi32p.h FORCEINLINE helpers (`GdiAllocBatchCommand`,
  `GdiGetDcAttr`, `GdiGetRgnAttr`) + `GDI_HANDLE_*` macros + `NtCurrentTeb`, `pub(crate)`.
- **dllmain.rs** (4 fns) — DllMain/GdiProcessSetup + `gbLpk`/`SetStockObjects`/`GdiDevCaps`/
  `GdiHandleCache`/`semLocal` globals.
- **gdiobj.rs** (7 fns) — `stock_objects[]` + GetStockObject/GetObjectType/GetObject(A/W)/DeleteObject.

Integration fixes (440→0): the big root was **E0659 glob ambiguity** (`crate::types::*` vs
`win32k_sys::win32k::*` both export `BOOL`/`INT`/`DWORD`/… ) → dropped the `crate::types` glob in
the files that glob win32k_sys (lean on win32k_sys for scalars, like user32-rust). Then: win32k_sys
`FALSE`/`TRUE` are bindgen `u32` but `BOOL` is `i32` → shadowed with `BOOL`-typed locals in gdiobj.rs;
`GdiBCDelObj` doubly-defined (gdihelp alias + win32k_sys enum_alias) → qualified the use. Link fix:
`METADC_ExtEscape` is plain cdecl in gdi32p.h (`_METADC_ExtEscape`, not `@24`) → moved its extern
decl from the `extern "system"` block to `extern "C"`. Decorations verified: `_DllMain@12`,
`_GetStockObject@4`, `_DeleteObject@4`, `_GdiValidateHandle@4`, cdecl `_GdiGetHandleUserData`,
fastcall `@GdiGetLDC@4`; globals as data syms.

Notes: Wave-0 i386 build relinked gdi32.dll clean with the Rust `LineDDA`; first i386 smoke
hit the **documented kernel-space winlogon/explorer-handoff race (BUGCHECK 0x1E @ ntoskrnl
PC, auto-classified KNOWN_FLAKE)** while 4 concurrent arm64 TCG boots starved the host —
unrelated to the usermode pure-algorithm leaf. Quiet-host re-run owed for the green tick.
