# Win32k C→Rust Translation Audit

> **Ground-truth re-verified 2026-06-14** against `win32ss/CMakeLists.txt` + on-disk `.rs`: BOTH halves are fully flipped — all 59 `user/ntuser/*.c` are commented `# replaced by Rust … cohabitation` (only `main_rust_shims.c` stays active = irreducible syscall tables); GDI (ntgdi+eng+dib) and DirectDraw (reactx/ntddraw, all 9 .c) are 100% flipped. The only active C is irreducible glue: 5 `*_rust_shims.c`, `main_rust_shims.c`, i386 `floatobj.c`/`floatobj.S` + i386 dib `.s`, gendib generated tables, and the dead `USE_DIBLIB` path (`dib_new.c`/`bitblt_new.c`, FALSE — not compiled). The "USER 3/59" snapshot in the older sections below is corrected inline.

> **2026-06-14 — MILESTONE: win32k.sys default-build body is 100% Rust**
> (commit translating `safe_user_io.c` → `safe_user_io.rs`, cold `--clean` smoke
> GREEN ×3). `safe_user_io.c` was the last real subsystem C leaf — SEH-free
> validate-only user-buffer range/alignment check (`SafeProbeForRead`/`Write`,
> NTAPI `_Safe…@12`). Pure arithmetic (no variadics/SEH/touch) → full C
> elimination, zero trampoline residue. Did NOT delegate to
> `ntkrnl_sys::probe::probe_user_buffer` (it `read_volatile`-touches the page,
> which this must avoid). `MmUserProbeAddress` cross-DLL data import via class.rs's
> arch-split pattern (i386/arm64 raw-dylib +verbatim; amd64 `__imp_` + deref).
> **What remains in win32k.sys is irreducible glue, not subsystem code:**
> 5 `*_rust_shims.c` (cdecl wrappers for FORCEINLINE/macro helpers + the
> `Win32kSSDT` syscall dispatch tables), `gdi/eng/i386/floatobj.c` (i386 FLOATOBJ
> math), and the `if(USE_DIBLIB)` dead path (`dib_new.c`/`bitblt_new.c`, FALSE
> default — not compiled; `dib.rs` covers it).

> **2026-06-14 — user/rtl flipped: `image.c` + `text.c` → Rust (DrawText)**
> (commit `68028a7c719`, cold `--clean` smoke GREEN ×3).
> `image.c` (`RtlGetExpWinVer`) hand-translated → `image.rs`. `text.c` (1402 LOC
> Wine DrawText: `DrawTextExWorker`/`UserExtTextOutW` + 8 ellipsify/wordbreak/
> prefix statics) via rustconv (sonnet, $0.25) → `rtl_text.rs` (named to dodge the
> gdi/ntgdi `text.rs`). Both retired from `win32k_residue`.
> Integration fixups (raw sonnet→×3): NT_SUCCESS/pool imports; `CR` const +
> `USERTAG_RTL` ('trsU'); C `assert()` macro → local no-op (translator emitted
> mixed 1-arg/4-arg calls + a bogus `extern fn assert`); raw-ptr-write
> immutability hacks (`lh`) → `let mut`; uninit bindings initialised;
> HPEN/HRGN→HGDIOBJ + RECT→RECTL `.cast()`; INT/UINT width casts; `KinsokuClassA`
> `[u16;70]`→`[u16;78]` (translator miscounted); `DrawTextExWorker` →
> `#[no_mangle] extern "system"`.
> **`dib_new.c` needs NO flip** — it's behind `if(USE_DIBLIB)` (FALSE by default,
> never compiled); the active path is `dib.c` → already `dib.rs`, which also
> covers dib_new's table/Dummy_*/DIB_DoRop. win32k.sys default-build residue is
> now just **`safe_user_io.c`**.

> **2026-06-13 — DirectDraw fully flipped: `reactx/ntddraw` is 100% Rust**
> (commits `5825b175fba` + `438a847c168`).
> *Phase 1* (`5825b175fba`): retired 8/9 banked-Opus `ntddraw_*.rs` (ddraw, dd,
> ddsurf, d3d, d3dkmt, dvp, mocomp, eng) from `win32k_residue` — Rust symbols win.
> *Phase 2* (`438a847c168`): translated `dxeng.c`'s 42 `DxEng*` bodies via
> rustconv (sonnet, $0.12) — the table-only `ntddraw_dxeng.rs` became the full
> translation, and `dxeng.c` was retired too.
> **Lesson — silent drop the c2rust anchor would have caught**: c2rust panics on
> win32k's header AST (`CDeclId`-not-found, c2rust 0.22.1), so this ran plain
> sonnet; the per-function scanner skips **pointer-return-type** fns, dropping
> `DxEngEnumerateHdev` (HDEV*) + `DxEngAltLockSurface` (SURFOBJ*) — hand-completed.
> Integration fixups (raw sonnet → compiles ×3): gaEngFuncs table → `transmute`
> to PFN; `InterlockedIncrement` import; bindgen anon-union
> (`.__bindgen_anon_1.DriverFunctions`); FLONG i32-vs-u32 mask casts; cross-crate
> `_ERESOURCE`/HGDIOBJ→HDC `.cast()`; BOOL→BOOLEAN `as u8`; amd64 DWORD_PTR(u64)
> match-scrutinee + ULONG field casts.
> **Cold `--clean` smoke GREEN ×3 (i386/amd64/arm64 — ✓ staleness ✓ boot).**
> ~~win32k.sys now has **4 live C files**: `safe_user_io.c`,
> `user/rtl/{image,text}.c`, `gdi/dib/dib_new.c`.~~
> **CORRECTED 2026-06-17 (rollup ground-truth):** all four are now
> `# replaced by Rust` in `win32ss/CMakeLists.txt` — none are live. The only
> active C is irreducible glue (see the 2026-06-14 header at top: `*_rust_shims.c`,
> `main_rust_shims.c`, i386 `floatobj.c`, gendib tables; `dib_new.c` is the dead
> `USE_DIBLIB=FALSE` path, not compiled).

> **SUPERSEDED 2026-06-09 / re-verified 2026-06-14 — the ntuser flip is now COMPLETE: all 59 `user/ntuser/*.c` are cohabitation-commented in `win32ss/CMakeLists.txt`; the "3/59" counts below are the obsolete 2026-06-04 snapshot, kept only for history.**

Last updated **2026-06-04** *(status re-verified 2026-06-14)*. **Status:
Phase 1 (handle tables) DONE; Phase 2 (GDI half) ✅ DONE — the entire GDI
half (ntgdi + eng + dib) is Rust and smoke-green ×3; Phase 3 (USER half)
✅ DONE — all 59 ntuser files flipped (was "3/59" at the 2026-06-04
snapshot; completed over the wave-1→38 saga, re-verified against cmake
2026-06-14).** The "bulk translation not yet started" premise of the
2026-05-16 draft is long obsolete — the full kernel-mode win32k body is
now Rust. See **"Landed reality"** immediately below; the planning
sections that follow are kept for history and annotated where superseded.

This audit defines the keep-set, the (corrected) prune-set, the
translation order, and the rustconv configuration for the bulk
translation phase.

Companion to:
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — kernel translation, ~99.7% done
- [hal-translation-audit.md](hal-translation-audit.md) — HAL translation, COMPLETE
- [freeldr-translation-audit.md](freeldr-translation-audit.md) — FreeLDR + boot DLLs, COMPLETE
- [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md) — **the C-side de-risking phase that gates this one — now landed**
- [pseh-elimination.md](pseh-elimination.md) — overall PSEH strategy
- `qemu/~temp-chat-consolidated.md` — callout-layer design notes. **⚠ This
  is a gitignored scratch file in `qemu/`. Phase 3 (the hardest half)
  hinges on it — it must be promoted into `docs/engineering/` before
  Phase 3 starts, not referenced as `~temp-chat`.**

### Pretranslation: what actually landed (corrects the old premise)

The original draft of this audit assumed pretranslation would hand
rustconv *rewritten C* full of magic macros (`Status = SafeCapture*(…)`,
`WITH_GDI_LOCKED(…)`). **That is not what was built, by deliberate
design.** What actually exists:

- **Probe lowering is a real source rewrite**: 65 files now use
  `SafeProbeForRead`/`SafeProbeForWrite` (status-returning, in
  `win32ss/safe_user_io.{c,h}`). There is **no** `SafeCapture*`.
- **GDI locks / USER critsec / callouts are extract-only JSON
  sidecars**, NOT source-rewrite macros. There is **no**
  `WITH_GDI_LOCKED`. Pass 2 (`*.win32k_gdilocks.json`), Pass 2b
  (`*.win32k_userlocks.json`), Pass 3 (`*.win32k_callouts.json`) +
  Pass 0 SEH triage feed the rustconv prompt-builder, which injects
  per-function ground truth + `win32k_translation_prompt.md` rules.
- **`win32k-sys` FFI crate is built** (`sdk/lib/rust/win32k-sys/`,
  3 arches, cargo-clean) — the ntkrnl-sys analog. Resolves ~82% of
  the win32k symbol surface (incl. FreeType — 46 FT_ symbols are
  already in it via `pch.h`). This is the substrate; see below.
- **Calling convention is deterministically enforced**, not
  LLM-inferred (`abi_map.py` + `postprocess.enforce_win32k_abi`):
  the #1 i386 crash cause is taken out of the LLM's hands entirely.
- **Giant files were `#include`-back split** (20 → 32 fragments) so
  no file is an oversized prompt — this *inverts* the old "prioritize
  the largest files" guidance.
- **Post-`shell ready` smoke** (`test_post_shell_ready_input`, i386 +
  amd64 QMP input injection) landed; more harness work in progress.

See [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md)
for the full landed inventory. The rest of this document is updated to
match that reality.

## Landed reality — 2026-06-04 *(re-verified 2026-06-14)*

This section is the **source of truth for what's translated**; the
planning sections below (TL;DR, Phasing, Forward path) predate the work
and are now annotated/superseded where they disagree.

### Headline numbers

| Metric | Value |
|---|---|
| Rust modules in `win32k-rust/src/` | **166** (`.rs` on disk, 2026-06-14) |
| Rust LOC | **~140,000+** |
| C files cohabitation-flipped (commented in `win32ss/CMakeLists.txt`) | **all subsystem C** — only irreducible glue stays active |
| Smoke status | **green ×3** (i386/amd64/arm64), 0 unexpected ud2 |

Flipped by subsystem (re-verified against cmake 2026-06-14): **gdi/eng
100%, gdi/dib 100%, gdi/ntgdi 100%, reactx/ntddraw 100% (all 9 .c),
user/ntuser 100% — all 59 .c commented `# replaced by Rust … cohabitation`,
only `main_rust_shims.c` (syscall dispatch tables) stays active.**

### Phase status (vs the original plan)

- **Phase 1 — handle tables: ✅ DONE.** `gdiobj.rs` (GDI handle table,
  the dependency hub) and `object.rs` (USER object/handle table,
  `user/ntuser/object.c` flipped under "Phase 1b") are both live and
  smoke-green. GDI handle lifecycle is Rust.
- **Phase 2 — GDI half: ✅ DONE.** The **entire GDI half is Rust**.
  - `gdi/ntgdi/` — **100%.** Every NtGdi* syscall file flipped. The
    long poles all landed: `freetype.c` (7,917 LOC glue, wave 21,
    `bbdabf00a69` — ft-sys retired, FreeType folded into win32k-sys),
    `font.c`, `path.c`(+splits), `bitblt.c` (→ `ntbitblt.rs`),
    `d3dkmt.c` (56 dxgkrnl thunks), `brush.cpp` (the **only C++**,
    wave 23 — first C++→Rust flip), and the debug residue
    `gdibatch.c`/`gdidbg.c`/`gdikdbgext.c` (waves 24–26).
  - `gdi/eng/` — **100%.** Graphics Engine / DDI surface is Rust;
    `ldevobj.c` (the last straggler) flipped + fixed (`1ba3a5ffaa9`).
  - `gdi/dib/` — **100%.** All per-bit-depth renderers (1/4/8/16/24/
    32bpp + alphablend / floodfill / stretchblt / transblt) are Rust.
  - **What's still C in `gdi/` — and why it's not translation work:**
    `gdi/eng/i386/floatobj.c` (i386 fixed-point math, kernel mode lacks
    SSE — stays C/asm **by design**); `gdi/dib/dib_new.c` +
    `gdi/eng/bitblt_new.c` (both inside the `if(USE_DIBLIB)` branch, and
    the tree sets `USE_DIBLIB FALSE` → **dead code, never compiled**;
    their live equivalents are already in `bitblt.rs`/the dib renderers,
    so a `*_new.rs` would just duplicate `#[no_mangle]` symbols). Net
    translatable GDI C remaining = **0**.
- **Phase 1.5 — typestate API (`UserLock`/`CoRef`/`CoCallout`): ⏳ NOT
  STARTED.** Wasn't needed for GDI or the trivial USER leaves; it gates
  the *deep* USER work (callouts/messages/hooks). Still owed — design
  lives in `qemu/~temp-chat-consolidated.md` §6 (must be promoted into
  `docs/engineering/` before the message/hook/window-create chain).
- **Phase 3 — USER half (`user/ntuser/`, ~73K LOC): ✅ DONE (re-verified
  2026-06-14).** All 59 `user/ntuser/*.c` are cohabitation-commented in
  `win32ss/CMakeLists.txt`; only `main_rust_shims.c` (the `Win32kSSDT`/
  `Win32kSSPT`/`NumberOfSysCalls` syscall dispatch tables) stays active as
  irreducible glue. The saga ran wave 1 (atom-table `session.c`/`useratom.c`
  against the Rust `gAtomTable`) through ~wave 38 (`keyboard.c`) plus the
  "frontier" set (`window.c`/`winpos.c`/`message.c`/`menu.c`/`defwnd.c`/
  `painting.c`/`sysparams.c`/`kbdlayout.c`/`main.c`). The hard core
  (callouts, message queues, hooks, the window-create chain) is flipped.
  The "3/59 / 56 remain" text below is the obsolete 2026-06-04 snapshot.

### Wave naming as it actually happened

GDI commit tags used **Phase 2a (eng) → 2b (dib) → 2c (ntgdi)** —
done **bottom-up**: engine + renderers first, syscall-level ntgdi last.
The 2c ntgdi work ran as ~26 "geometry/object/closeout waves": bezier/
dcattr → arc/stockobj/xformobj → drawing/line/fillshap → rect/polyfill
→ DC-lifecycle → region algebra → pen/coord → palette → cliprgn+bitmaps
→ dibobj → font → ntbitblt → freetype → d3dkmt → brush.cpp → debug
residue (gdibatch/gdidbg/gdikdbgext). The **ntuser saga** then opened
its own wave series; wave 1 = the session/useratom atom-table pair.

### Remaining USER work (Phase 3) — ✅ COMPLETE (history kept below)

> **Re-verified 2026-06-14: there is NO remaining USER work.** All 59
> `user/ntuser/*.c` are flipped. The leaf-first ordering below is retained
> only as a record of how the saga was sequenced; every file listed is now
> cohabitation-commented in `win32ss/CMakeLists.txt`.

**~~56 active~~ 0 remaining `user/ntuser/*.c` files.** Original suggested
leaf-first order (low-dependency leaves before the reentrant core), with
the hard core last:

- **Easy leaves (flip next, same playbook as GDI):** `guicheck.c`,
  `metric.c`, `prop.c`, `caret.c`, `focus.c`, `ghost.c`, `misc.c`,
  `windc.c`, `vis.c`, `usrheap.c`, `monitor.c`, `power.c`,
  `accelerator.c`, `hotkey.c`, `ntstubs.c`.
- **Mid tier:** `class.c`, `cursoricon.c` (both already consume the
  Rust `gAtomTable`/`IntAddAtom` from wave 1), `clipboard.c`, `timer.c`,
  `windc.c`, `display.c`, `draw.c`, `nonclient.c`, `painting.c`,
  `scrollbar.c`/`scrollex.c`, `sysparams.c`, `keyboard.c`/`kbdlayout.c`,
  `mouse.c`, `event.c`, `dde.c`, `ime.c`, `layered.c`, `csr.c`,
  `console.c`, `simplecall.c`, `security.c`, `shutdown.c`, the
  `misc/{file,rtlstr,copy,registry}.c` helpers.
- **Hard core (needs Phase 1.5 typestate API first):** `desktop.c`
  (boot marker `co_IntShowDesktop` lives here), `winsta.c` (the 50ms
  HACK), `message.c`/`msgqueue.c` (SendMessage/PostMessage), `hook.c`
  (14 hook variants, highest SEH density), `callback.c`/`callproc.c`
  (the `co_*` callout straddle), `window.c`/`winpos.c` (window-create/
  position chain), `menu.c` (~6,690 LOC monster), `input.c`, `defwnd.c`,
  `ntuser.c`, `main.c`.

Methodology is **fully proven** on every GDI shape; the easy/mid USER
leaves are the same bulk-rustconv + cohabitation-flip + smoke-gate loop.
The hard core is the genuine new work (typestate API + callout safety).

### Cleared stale annotation

The old `device.rs` "INTEGRATED + FAILING (i386 smoke hangs pre-`shell
ready`)" cmake tag is **resolved** — fixed by `20bc456b6d0`
(DC_bIsBitmapCompatible flexible-array index = the InitializeClasses
hang) + the FillSolid/EngEraseSurface i386 dup-ABI ud2 fix; i386 smokes
green with `device.rs` live. <!-- VERIFY: as of 2026-06-14 the
`gdi/eng/device.c` line in win32ss/CMakeLists.txt STILL carries the stale
"INTEGRATED + FAILING (i386 smoke hangs pre-`shell ready`)" comment. Smoke
is green ×3, so the annotation is wrong — downgrade it on the next
CMakeLists touch. -->

## Notable learnings (banked across the GDI waves — apply to USER)

These are the techniques that made the GDI half tractable. They carry
straight into the USER half.

1. **`nm` is the ABI oracle.** Run `llvm-nm` (in `.build-tools/llvm-mingw/
   bin`) on the built `.obj`/`win32k.sys`: `@Name@N` = FASTCALL (emit a
   dual-cfg `extern "fastcall"` on i386 / `extern "C"` else), `_Name@N` =
   stdcall (`extern "system"`), `_Name`/`Name` = cdecl (`extern "C"`),
   `D`/`d` = data global. rustconv frequently mislabels these; **the
   binary is ground truth, not the LLM.** Calling-convention mismatch is
   the #1 i386-only crash class (links clean on amd64/arm64 where
   cdecl==system, breaks i386).
2. **rustconv STUBS big SEH/switch bodies.** Anything with a large
   `_SEH2_TRY` or a giant `switch` comes back as a ~10-line stub
   (GdiFlushUserBatch: 350-line switch → 13-line stub). **Hand-translate
   those.** Detection: the `.rs` fn body is suspiciously short vs the C.
3. **Wide-string byte-literal misalign** (`feedback_byte_literal_wchar_
   misalign`): `b"...".as_ptr() as *const u16` reinterprets 1-byte chars
   as 2-byte WCHARs = garbage. Cost the ldevobj NtUserInitialize
   0xc0000001. Fix = a `wide_str!` macro emitting a NUL-terminated
   `[u16;N]` static. USER code is full of wide strings — watch for this.
4. **Orphan-cfg stitcher bug**: the stitcher sometimes emits a
   contradictory `#[cfg(target_arch="x86")]` + `#[cfg(not(...="x86"))]`
   PAIR before a fn → it compiles on NO arch → silently absent from the
   `.a` → `@Name@N` undefined at link. Detect: grep for the two cfg
   attrs on consecutive lines. Fix: delete the orphan pair.
5. **C-shim for FORCEINLINE / macro / arch-divergent helpers**: when
   `nm` shows no symbol for a called helper (it's FORCEINLINE, a macro,
   or arch-divergent like FLOATOBJ stdcall-asm-i386/float-macro-amd64),
   **don't reimplement the math in Rust** — write a plain-cdecl
   `w32r_<name>(...)` wrapper in a `*_rust_shims.c` (added to the win32k
   SOURCE list) that `#include`s the header and calls the C original;
   sed the `.rs` call/decl sites. Recompiles per-arch → free arch
   divergence. (path/font/ntbitblt shims established this.)
6. **rustc-JSON-span cast auto-fixer** (`/tmp/fix_casts.py` pattern):
   `cargo build --message-format=json`, collect E0308 primary spans,
   wrap each offending expr `(<bytes> as _)` — pointer/numeric/ref
   mismatches only (skip struct-value, `as` can't cast structs → use
   `transmute` for layout-identical POINT↔POINTL). Cleared 100+ casts in
   seconds on the geometry waves.
7. **arm64 cfg trap**: `#[cfg(not(target_arch="x86"))]` matches **arm64
   too**, not just amd64. x86 inline asm (`gs:`/`fs:`) under that gate
   breaks the arm64 build. Use portable `NtCurrentTeb()` (from
   `ntkrnl_sys::inline::ke`) instead. **Always build all 3 arches** when
   integrating — agents/translations that only build x86 let arm64 cfg
   bugs survive.
8. **arm64 build mechanics** (`feedback_arm64_cli_build_skips_usermode`):
   `vibeos-cli build --arch arm64` does **not** relink `win32k.sys` even
   for a kernel-mode flip (staged set is ntoskrnl+hal+uefildr). Manually
   `cd build-arm64 && ninja win32k`, then mcopy fresh `win32k.sys` into
   `ramdisk.img` + `boot.img` before smoke (else a fast green = booted
   STALE media = false positive). The arm64 `ar` is out-of-tree → prefix
   `.build-tools/llvm-mingw/bin` to PATH + `dangerouslyDisableSandbox`.
9. **Agent worktree-isolation leak** (`feedback_agent_worktree_isolation_
   build_leak`): `isolation:"worktree"` subagents get a fresh checkout
   with NO `build-*` dirs, so their `cd build-i386` resolves to the REAL
   trees and edits leak (lib.rs/CMakeLists conflicts). Have agents write
   `.rs` to `/tmp` only and do all wiring+build+smoke centrally.
10. **C++ "C with classes" flips cleanly** (brush.cpp, wave 23): a
    `class X : public BASEOBJECT` with no vtable (every method a
    non-virtual inline over a C-ABI helper) → `*mut _X` (bindgen struct,
    per-arch-correct size for the alloc); base fields via a `as POBJ`
    cast; only the `extern "C"` API gets `#[no_mangle]`. The C++ file
    lives in `add_library(win32k MODULE ...)` directly, not the SOURCE
    list.
11. **USER groundwork is proven** (ntuser wave 1): win32k-sys already
    carries the USER object model (`PTHREADINFO` w/ `rpdesk: *mut
    _DESKTOP`, `_WND`/`_DESKTOP`, RTL_ATOM/ATOM); the
    `PsGetCurrentThreadWin32Thread() → pti->rpdesk` thread-info pattern
    works; a cross-module `#[no_mangle]` global (`gAtomTable`) shares
    cleanly between two Rust files AND still-C consumers via the `__imp_`
    thunk. The ntgdi→ntuser jump is the same cohab playbook.
12. **Always `git show HEAD:<path>.rs | wc -l` before translating** an
    ntgdi/ntuser file — a pre-existing partial `.rs` may already be
    committed (the bitblt trap); `rustconv translate` without `-o`
    OVERWRITES it. Use `translate <c> -o /tmp/out.rs` to dodge overwrite
    + auto-mutation of lib.rs/CMakeLists.

## TL;DR

- **Win32k.sys is the largest single translation target after
  ntoskrnl.** ~157K LOC C in the keep-set after pruning.
- **Prune ~14K LOC + the SARCH/legacy display variants** before
  translation. Same shape as HAL/FreeLDR pruning.
- **Three semi-independent halves:** GDI (~73K), USER (~73K), reactx
  (~5K — could prune to zero). The split is real; cross-links are
  through GDI handle table + EPROCESS->Win32Process.
- **Translation order: GDI-first as methodology bake-in, then USER.**
  GDI is more deterministic (pixel math, well-defined PE/DDI surfaces,
  fewer arch-conditional paths). USER is harder (callouts, message
  queues, hooks, the open amd64-arm64 desktop edge cases) — let the
  GDI translation prove out the methodology first.
- **freetype.c (~7,943 LOC) glue stays around an FFI boundary**;
  freetype itself stays C (3rd-party vendor blob, same handling as
  ACPICA in HAL). The FT_* FFI surface is **already in `win32k-sys`**
  (46 symbols, via `pch.h`) — no separate `freetype-sys` needed.
- **brush.cpp is the only C++** in the tree (605 LOC, verified).
  Translate to a Rust struct as a one-off.
- **i386 has unique kernel-mode FLOATOBJ_* asm** (`gdi/eng/i386/floatobj.S`)
  — fixed-point math because i386 kernel mode lacks SSE. Stays asm or
  becomes Rust `global_asm!`; either works.
- **Giant files are already split** — do NOT prioritize "largest
  files"; that guidance is obsolete. `rustconv translate` chunks
  function-by-function and every file is now under the prompt cliff.
- **Translation cost via rustconv: budget ~$25–50 lifetime** (≈170
  files × ~$0.05 × 3–5 iters). The earlier "$15–25" figure elsewhere
  in this doc was inconsistent and is superseded by this number;
  win32k-sys *reduces* per-file token cost (no per-file bindgen
  lookups) but the file-split *increases* file count — net wash.
  Multi-month calendar even with parallelism because regressions in
  win32k are highly visible (every one breaks the desktop).
- **The crown jewel is GDI handle table + USER handle table being
  Rust.** Once both are Rust, win32k handle lifecycle is Rust
  end-to-end and the rest is mostly leaf code.
- **Smoke risk: high.** Win32k bugs visibly break the desktop. Smoke
  must be enhanced *before* translation begins to catch post-`shell
  ready` regressions (cursor, taskbar, window-create cycle).

## Today's win32k.sys

The kernel-mode binary alone:

```
build-i386/win32ss/win32k.sys      16.9 MB
build-amd64/win32ss/win32k.sys     17.9 MB
build-arm64/win32ss/win32k.sys     17.6 MB
```

Source layout:

```
win32ss/
├── CMakeLists.txt           269 lines (top-level: builds win32k.sys MODULE)
├── win32k.spec              248 export entries
├── w32ksvc32.h              738 SVC_() entries (i386 syscall table)
├── w32ksvc64.h              746 SVC_() entries (amd64+arm64 syscall table)
├── sys-stubs.S              syscall trampolines, generated by cpp from w32ksvc*.h
├── win32k.h, win32kp.h, napi.h, pch.h, win32k.rc
│
├── gdi/                     109,804 LOC — GDI graphics
│   ├── ntgdi/                41,471   ← NtGdi* syscall implementations + freetype glue
│   ├── eng/                  19,192   ← Graphics Engine (DDI for display drivers)
│   ├── dib/                   5,426   ← DIB renderers per bit-depth
│   ├── diblib/                3,456   ← Alternate DIB lib (USE_DIBLIB FALSE — DEAD)
│   └── gdi32/, gdi32_vista/  35,104   ← USERMODE — out of this audit's scope
│
├── user/                    184,389 LOC — USER + sibling DLLs
│   ├── ntuser/               73,166   ← NtUser* syscall implementations
│   ├── rtl/                   1,448   ← Shared RTL helpers (kernel + user)
│   ├── user32/               61,576   ← USERMODE
│   ├── user32_vista/          3,879   ← USERMODE
│   ├── imm32/                13,241   ← USERMODE (IME)
│   └── winsrv/               27,180   ← USERMODE (CSRSS server)
│
├── reactx/                    6,787 LOC — DirectDraw shim
│   ├── ntddraw/               3,193   ← Kernel-mode DD (mostly stubs)
│   ├── dxg/                   1,884   ← Kernel-mode graphics dispatch
│   ├── dxapi/                              ← (empty/stub)
│   ├── dxgthk/                             ← (empty/stub)
│   ├── ntdxvista/                          ← (empty/stub)
│
├── drivers/                  30,711 LOC — Kernel-mode drivers
│   ├── videoprt/              9,492   ← Video Port (driver framework)
│   ├── miniport/             10,011   ← Display miniport drivers (varies)
│   ├── displays/              7,790   ← Stub display drivers (vga, vga_new, framebuf)
│   ├── font/                  3,181   ← Font driver
│   └── watchdog/                237   ← TDR-style watchdog
│
├── printing/                 28,600 LOC — print spooler (winspool, etc.) — out of scope
├── win32u/                       47   ← Tiny stub
└── include/                   5,345
```

**Win32k.sys composition** (what actually compiles into the .sys):

*(Translation-status column re-verified against cmake 2026-06-14.)*

| Component | LOC C | Files | Status (2026-06-14) |
|---|---:|---:|---|
| `gdi/ntgdi/` | 41,471 | 39 | ✅ flipped (Phase 2) |
| `gdi/eng/` | 19,192 | 40 | ✅ flipped (Phase 2; i386 `floatobj.c` stays C by design) |
| `gdi/dib/` | 5,426 | 13 | ✅ flipped (Phase 2; i386 `.s` + dead `dib_new.c` stay C) |
| `user/ntuser/` | 73,166 | 59 | ✅ flipped (Phase 3; only `main_rust_shims.c` syscall tables active) |
| `reactx/ntddraw/` | 3,193 | 9 | ✅ flipped (all 9 .c → `ntddraw_*.rs`, DirectDraw flip) |
| `reactx/dxg/` | 1,884 | 9 | ⬜ STILL C — separate `dxg.sys` MODULE (`reactx/dxg/CMakeLists.txt`), NOT part of win32k.sys; all 9 .c uncommented. Out of the win32k.sys core. |
| `gdi/ntgdi/brush.cpp` | 605 | 1 (C++) | ✅ flipped (one-off, wave 23) |
| **win32k.sys total** | **~145K** | **170** | — |

Plus sys-stubs.S (auto-generated syscall table from `w32ksvc{32,64}.h`)
and the i386-only `gdi/eng/i386/floatobj.S` + `gdi/dib/i386/dib*_hline.s`
fixed-point asm (~6 files, small).

**Build artifact:** single `win32k.sys` (kernelmodedriver). Imports:
`ntoskrnl`, `hal`, `ftfd` (FreeType font driver). Links: `${PSEH_LIB}`,
`dxguid`, `libcntpr` (kernel-mode CRT subset).

## Today's smoke coverage

Per `tests/qemu/test_smoke.py` and the existing harness:

- **Boot marker:** `shell ready` (i386/amd64). Reaches `co_IntShowDesktop`
  in win32k. So everything up to and including win32k initialization +
  desktop handoff is exercised by every smoke run.
- **What's NOT tested today:** anything *after* shell ready. The
  amd64 taskbar regression that just got fixed (~30 commits ago, not
  yet in MEMORY.md before this session) was visible because of the
  saved screenshot, not because the smoke test caught it directly.
- **Implication for this translation:** smoke must be enhanced before
  bulk win32k translation. Otherwise post-init regressions will land
  silently. See [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md)
  step 7 — adding cursor-movement / window-create-cycle / taskbar-paint
  assertions.

## Prune-set

> **⚠ CORRECTED 2026-05-16.** The original prune table was written from
> path-name guessing and is substantially wrong. The pretranslation
> phase verified the *actual build graph* (`add_subdirectory` +
> explicit `list(APPEND SOURCE …)` + unconditional `#include`). Lesson
> banked as `feedback_prune_verify_build_graph`: "vibeos prune tables
> lie; verify the build graph before git-rm." Corrected status below.

### 1. Already gone (prior dead-platform purge — no action)

`drivers/miniport/xboxvmp/`, `drivers/miniport/pc98vid/` and their
printing references were **already deleted** in the 2026-05-13/14
dead-platform purge (xbox + pc-98 + 32-bit ARM + SAC + arm64ec, ~28K
LOC repo-wide). Do not list them as pending prune work.

### 2. NOT safely prunable (verified load-bearing — keep)

| Path | Why the old "prune" call was WRONG |
|---|---|
| `gdi/diblib/` | NOT "whole dir compiled out". `gdi/dib/dib_new.c` `#include`s `DibLib.h` / `DibLib_interface.h` **unconditionally** → load-bearing. |
| `reactx/dxg/`, `reactx/ntddraw/` | Built via `add_subdirectory(reactx)` in `win32ss/CMakeLists.txt` and in the win32k.sys SOURCE path. Not "trivial stubs to delete now". |
| `drivers/displays/vga/` | Reached via `add_subdirectory(drivers)`; pruning needs build-graph proof, not the smoke-usage guess. |

### 3. Actually safe to prune (verified)

Only the genuinely-orphaned subset survives scrutiny — per the
pretranslation audit, ~the two orphaned `drivers/displays/vga_new`
files (~6.3K LOC) were the *only* safely-removable set, and that prune
already landed (smoke-gated) during pretranslation. Treat any further
prune as: prove it's not in the build graph first, surface
contradictions, never git-rm to hit a LOC number.

**Net:** the "~14K LOC prunable" subtotal is fiction. The real,
verified, *already-done* prune is the vga_new orphans. Do not re-plan
a pruning PR around the obsolete table.

### 3. Cannot prune (load-bearing)

| Path | LOC | Why kept |
|---|---:|---|
| `gdi/ntgdi/freetype.c` | 7,943 | Glue to FreeType (3rd-party at `sdk/lib/3rdparty/freetype/` — 208K LOC, NOT translated). **✅ GLUE TRANSLATED** (wave 21, `bbdabf00a69`); vendor FreeType stays C. |
| `drivers/videoprt/` | 9,492 | Video Port — kernel framework that display drivers use. Even framebuf-only mode goes through videoprt. **KEEP, translate** (not started — out of the win32k.sys core, separate driver). |
| `gdi/ntgdi/brush.cpp` | 605 | The only C++ file. **✅ TRANSLATED** (wave 23, `cba2a2538fd`) → `*mut _BRUSH` + Rust fns. |

## Keep-set landscape

After pruning, ~145K LOC of kernel-mode source across three halves:

### GDI half — ~66K LOC

```
gdi/ntgdi/    41,471 LOC, 39 files   — NtGdi* syscalls + GDI handle table + freetype glue
gdi/eng/      19,192 LOC, 40 files   — Graphics Engine, DDI for display drivers
gdi/dib/       5,426 LOC, 13 files   — DIB renderers (1/4/8/16/24/32 bpp)
```

**Largest files** — ⚠ **OBSOLETE TABLE.** The giants below were
`#include`-back split during pretranslation (`region.c` 4,144→1,292,
`path.c` 3,109→1,131, etc.; remainder in `*.split2/3/4.c` fragments).
Do **not** "prioritize the largest files in rustconv batches" — that
premise is inverted: every file is now under the prompt cliff and
`translate` chunks function-by-function regardless. `gdiobj.c` (the
GDI handle table) still matters as a *dependency hub* (Phase 1), not
because of size. Kept for historical context only:

```
gdi/ntgdi/freetype.c   7,943   ← FreeType glue (NOT split — single TU
                                  by design; translate via win32k-sys
                                  FT_* surface, not a freetype-sys crate)
gdi/ntgdi/region.c     →1,292   (was 4,144; split)
gdi/ntgdi/path.c       →1,131   (was 3,109; split)
gdi/ntgdi/gdiobj.c     1,669    ← GDI handle table — Phase 1 hub (by role)
```

**Translation difficulty: medium.** Pixel math is deterministic and
well-bounded; the GDI handle table (`gdiobj.c`) has lifecycle
complexity but it's a known shape (similar to ntoskrnl's ob/ handle
table that's already Rust). FreeType glue is just FFI calls; the
hard part is shape-matching FT_Face/FT_Glyph types to the existing
GDI font objects.

**i386-specific bits** in this half:
- `gdi/eng/i386/floatobj.S` + `floatobj.c` — fixed-point math
  because i386 kernel mode runs without SSE. **Stays asm** (or
  `global_asm!`) — translating fixed-point to Rust would just reskin
  the existing implementation.
- `gdi/dib/i386/dib*_hline.s`, `dib32bpp_colorfill.s` — i386 hand-asm
  hot paths for horizontal-line / color-fill blits. **Stays asm.**

### USER half — ~73K LOC

```
user/ntuser/  73,166 LOC, 59 files   — NtUser* syscalls + window manager
user/rtl/      1,448 LOC, ~6 files   — Shared RTL (also linked to user32)
```

**Largest files** — ⚠ **OBSOLETE (post-split).** `window.c`
4,727→1,089, `winpos.c` 4,063→1,064, etc.; `sysparams.c`/`defwnd.c`
additionally had their giant dispatch switches tree-sitter-extracted
into helper fns (`SpiGetSet` 1,029→shell+139, `IntDefWindowProc` +11).
`menu.c` ~unchanged (6,690) — a remaining genuine monster the
translator handles via `translate` chunking. The role annotations
still hold and are what matters now:

```
ntuser/menu.c        6,690   ← Win32 menu system (NOT split — translate-chunk)
ntuser/desktop.c     ~3,5xx  ← Desktop/station init (boot marker lives here)
ntuser/message.c     split   ← SendMessage/PostMessage
ntuser/winsta.c      ~2,2xx  ← Window stations (50ms HACK lives here)
ntuser/hook.c        ~1,7xx  ← Win32 hooks (highest SEH density)
ntuser/callback.c    split   ← co_IntClientLoadLibrary lives here (Pass-3 anchor)
```

Sequencing is now by **dependency depth + Pass-2b/3 sidecar shape**
(CLEAN_BRACKET first, REENTRANT/REVIEW last), not by raw LOC.

**Translation difficulty: high.** This is where:
- The 17 `co_*` callout functions live → typestate API
  (`UserLock` + `CoRef<T>` + `CoCalloutContext`, designed in
  `~temp-chat-consolidated.md` §6).
- The amd64 + arm64 desktop edge cases exist (the just-fixed taskbar
  bug; the existing 50ms HACK in `winsta.c`).
- Cross-references through `EPROCESS->Win32Process` → `THREADINFO` are
  densest.
- Hook chains (14 hook variants in `hook.c`) need the most careful
  ABI work.

### reactx half — kept-set after prune

If we drop ntddraw + dxg, reactx is empty. **Probably correct.**
DirectDraw kernel-side can be re-added if we ever support DD apps.

### win32k.sys top-level (small but glue-critical)

| File | LOC | Role |
|---|---:|---|
| `pch.h` | small | Precompiled header |
| `napi.h` | small | NTAPI table for the win32k SVC trampolines |
| `win32k.h`, `win32kp.h` | small | Internal headers |
| `sys-stubs.S` | gen | Syscall trampolines from `w32ksvc{32,64}.h` |
| `win32k.rc` | small | Version resource |

`sys-stubs.S` is auto-generated by cpp processing the SVC_() table.
**Stays asm** — translation makes no sense; the trampolines are by
definition just register-marshalling per the syscall ABI.

## ABI surface

Two facets:

**1. Kernel→user export contract** — `win32k.spec`, 248 entries:

| ABI category | Count | Notes |
|---|---:|---|
| `Eng*` (graphics engine, DDI) | ~150 | Engine functions display drivers call into |
| `BRUSHOBJ_*`, `CLIPOBJ_*`, `PATHOBJ_*`, `XLATEOBJ_*`, `SURFOBJ_*` | ~50 | OBJ accessor methods (DDI-side) |
| `FLOATOBJ_*` | ~25 | i386-only fixed-point math (declared `-arch=i386`) |
| `*WindowProc`, `Hook*`, etc. | ~20 | USER-side exports |

ABI must be preserved byte-for-byte through translation: display
drivers (videoprt + framebuf, both kernel-mode in our keep-set) and
user32/gdi32 link against this surface.

**2. User→kernel syscall surface** — `w32ksvc{32,64}.h`:
- 738 / 746 SVC_() entries (i386 / amd64+arm64)
- ~434 `Gdi*` syscalls (NtGdi prefix)
- ~303 `User*` syscalls (NtUser prefix)
- ~1 `Dx*` syscall

These syscall implementations are the bulk of `gdi/ntgdi/` and
`user/ntuser/`. Each syscall has a sibling `Nt<Name>` function that
the trampoline routes to. The Rust translation must preserve every
syscall's signature byte-for-byte; the win32u stub crate assembles
the user-side call.

## Build artifact decision

> **⚠ CORRECTED 2026-05-16.** The original "cdylib + custom
> `*-win32k.json` target spec" plan **contradicts both** the proven
> project pattern *and* this document's own "Smoke continuity
> strategy" §(which correctly describes augment-link cohabitation).
> `ntoskrnl-rust` is `crate-type=["rlib"]` → `libkernel_rust.a` →
> linked into `ntoskrnl.exe` by the cmake build with the C
> commented-out (cohabitation); `hal-rust` is the same staticlib
> cohabitation. No custom JSON target spec, no cdylib, anywhere in
> the proven tree. **win32k follows the same cohabitation pattern.**
> The cdylib/custom-target section below is superseded.

### Corrected: cohabitation staticlib (proven — ntoskrnl-rust/hal-rust)

`sdk/lib/rust/win32k-rust/` is an **rlib** pulled into a staticlib
umbrella (mirror `kernel-rust`/`hal-rust`): `crate-type=["rlib"]`,
workspace `lto=false`+`cgu=256` firewall, `#![no_std]`. Built per-arch
with the **existing** Rust targets (`i686-pc-windows-gnu`,
`x86_64-pc-windows-gnu`, `aarch64-pc-windows-gnullvm`) — *not* custom
`.json` target specs. Each translated `.c`→`.rs` swap is independently
revertible: the `.c` is commented out in `win32ss/CMakeLists.txt` (or
the relevant `add_subdirectory` cmake) with a `# replaced by Rust`
line; the Rust `#[no_mangle]` symbol takes over; `--allow-multiple-
definition` covers the swap-in window (lld first-match wins). Depends
on `win32k-sys` (built) + `ntkrnl-sys`; **no `freetype-sys`** (FT_* is
in win32k-sys). `extern crate win32k_rust;` in the umbrella `lib.rs`
or the symbols won't reach the staticlib (`feedback_extern_crate_for_staticlib`).

The historical cdylib proposal (kept below, struck) was never how any
vibeos kernel-mode Rust ships:

~~**One Rust crate: `win32k-rust` (cdylib), three target specs.**~~

```
sdk/lib/rust/win32k-rust/
├── Cargo.toml              ← crate-type = ["cdylib"]
├── build.rs                ← compiles per-arch asm via cc-rs (sys-stubs, floatobj, dib hline)
├── i386-win32k.json        ← custom kernel-mode target spec
├── amd64-win32k.json
├── arm64-win32k.json
└── src/
    ├── lib.rs              ← #![no_std], panic_handler, syscall entry roots
    ├── gdi/
    │   ├── mod.rs
    │   ├── ntgdi/          ← NtGdi syscalls
    │   ├── eng/            ← Graphics Engine
    │   └── dib/            ← DIB renderers
    ├── user/
    │   ├── mod.rs
    │   ├── ntuser/         ← NtUser syscalls
    │   ├── lock.rs         ← UserLock typestate (per ~temp-chat §6)
    │   ├── coref.rs        ← CoRef<T> (per ~temp-chat §6)
    │   └── callout.rs      ← CoCallout builder (per ~temp-chat §6)
    └── handle/
        ├── gdi.rs          ← GDI handle table
        └── user.rs         ← USER handle table
```

Imports against existing `ntkrnl-sys` crate (kernel types, syscall
helpers, probe API). Adds dependency on `freetype-sys` (new crate
that bindgens FreeType — sibling of HAL's `acpi-tables-sys`).

**Custom target spec** mirrors the planned HAL/FreeLDR shape: PE
cdylib, no SSE (i386 + amd64 kernel mode lacks SSE for FX state
preservation), no userland CRT, kernel-mode runtime hooks. Probably
share a workspace-level profile across `kernel-rust`, `hal-rust`,
`win32k-rust`, etc.

## Substrate crate — DONE (collapsed from 3 planned to 1)

> **✅ LANDED 2026-05-16.** The 3-crate plan
> (`freetype-sys`+`win32k-types`+`gdi-ddi-sys`) collapsed to the single
> `win32k-sys` the doc itself recommended as the alternative. It is
> **built and cargo-clean on all 3 arches.**

`sdk/lib/rust/win32k-sys/` — bindgen of `<win32k.h>` (which pulls
`pch.h`: ntifs, winddi, winuser, `ntgdi.h`, `ntuser.h`, `win32kp.h`,
**and FreeType**). Per arch: ~2,800 structs (USER object model
`_WND`/`_THREADINFO`/`_DESKTOP`/`_THRDESKHEAD`, GDI
DC/REGION/PATH/SURFACE), ~2,700 fns, ~20K constants, ~7K type aliases.

This **subsumes all three planned crates**:
- `freetype-sys` → unnecessary: 46 FT_* symbols already in win32k-sys
  via `pch.h`'s `ft2build.h`/`FT_FREETYPE_H`.
- `win32k-types` → that *is* win32k-sys (the THREADINFO/PROCESSINFO/
  handle types).
- `gdi-ddi-sys` → the DDI/eng types come through `winddi.h` in the
  same bindgen.

rustconv resolution wired (`config.py` win32k Target, `catalog.py`
scan, `detect_target`): win32k files now resolve ~82% (simplecall.c
233→41). Gotchas banked in `feedback_win32k_sys_bindgen`
(`-fms-extensions` yes / `-fms-compatibility` no; `--allowlist-var
'.+'`; tagged-enum de-prefix). **No further substrate work needed
before translation.**

## Translation strategy

### Phasing

The pretranslation phase (separate audit) lands first. Then this
phase, in three subphases:

**Phase 1 — substrate + handle tables (1–2 sessions):**
- bindgen `freetype-sys`, `win32k-sys`, `gdi-ddi-sys`.
- Translate `gdi/ntgdi/gdiobj.c` (1,669 LOC, GDI handle table) — this
  is the dependency hub for the rest of GDI.
- Translate `user/ntuser/object.c` or equivalent (USER handle table).
- Land both as Rust static libs alongside C; smoke gates the swap.

**Phase 2 — GDI half (~66K LOC, ~5–8 sessions):**
- Bulk rustconv over `gdi/ntgdi/`, `gdi/eng/`, `gdi/dib/`.
- Order within: leaf renderers first (`dib/`), engine plumbing
  (`eng/`), then syscall implementations (`ntgdi/`).
- `freetype.c` glue last in this phase (depends on `freetype-sys`).
- `brush.cpp` → Rust struct as a one-off.
- i386-only asm files stay as `global_asm!`.

**Phase 3 — USER half (~73K LOC, ~5–8 sessions):**
- Build the typestate API first (`UserLock`, `CoRef<T>`,
  `CoCalloutContext`, `CoCallout` builder). 1 session of design + impl.
- Translate `co_IntClientLoadLibrary` (the simplest representative)
  end-to-end as methodology bake-in.
- Bulk rustconv on the rest of `user/ntuser/`, sequenced by
  dependency depth (leaf modules first: focus, scrollbar, clipboard,
  …; finally the message-queue + hook + window-create chain).

**Total: ~12–18 sessions of LLM-driven work**, plus 5+ for
infrastructure / spike / smoke-enhancement / debugging. Multi-month
calendar even with parallelism, because every regression needs to be
diagnosed against a smoke run that already passed up to `shell ready`
plus the new post-shell assertions.

### Rustconv pipeline configuration

The pipeline already exists (`tools/rustconv/`). What needs to be
added:

1. ~~win32k-specific system prompt extending `probe_migrate_prompt.md`~~
   **✅ DONE differently.** `templates/win32k_translation_prompt.md`
   exists and is **sidecar-driven**, injected by `prompt_builder` only
   when the Pass-0/2/2b/3 JSON sidecars are present next to the C file
   (ntoskrnl/hal prompts unaffected). It teaches: residual SEH
   handling (probes already → `SafeProbeFor*`), GDI-lock RAII shapes
   from `*.win32k_gdilocks.json`, USER-critsec `UserExclusiveGuard`
   from `*.win32k_userlocks.json` §2.5, callout straddle from
   `*.win32k_callouts.json` §3. There is **no `SafeCapture*` / no
   `WITH_GDI_LOCKED`** — the input contract is sidecars + lowered
   probes, not magic macros (see header correction).
2. **Cost-budget:** ≈170 files × ~$0.05 × 3–5 iters ≈ **$25–50
   lifetime** (this is the canonical figure; the TL;DR was reconciled
   to it). win32k-sys removes per-file bindgen-lookup tokens.
3. **Verification gates** per file:
   - **ABI is now *enforced*, not just checked** —
     `postprocess.enforce_win32k_abi` overwrites the LLM's `extern`
     with the `abi_map` ground truth (win32k-sys bindgen + win32k.spec)
     for `target_name=="win32k"`. `verify_abi.py` (now folding
     win32k-sys + win32k.spec) is the *gate* behind it.
   - `verify_constants.py` — bindgen constant drift.
   - `scan_ud2.py` — LLVM miscompilation canary.
   - `lint_kernel.py` — captures(none) defenses.
4. **Per-arch build gates** — all 3 arches must build before we move
   to the next file (same as ntoskrnl).

### Smoke continuity strategy

The "broken Rust translation in tree, integrated and failing" rule
from CLAUDE.md applies. Specific to win32k:

- **Augment-link pattern.** Each translated file lands as a Rust
  static lib symbol with the same name; linker preserves Rust over C
  via `--whole-archive`. C version stays buildable as a fallback.
- **Per-file flip on smoke green.** N consecutive smoke passes with
  the Rust symbol in play → delete the C file from `win32ss/CMakeLists.txt`.
- **Post-`shell ready` smoke gates** (added in pretranslation
  phase 7) — these are what catch the post-init regressions.
- **Symbol-presence verification.** After each file flips, `nm` the
  resulting `win32k.sys` and confirm every exported symbol is still
  present at the expected decoration. The 248-entry spec file is the
  contract.

## Forward path

Mirroring the HAL/FreeLDR audits' forward-path shape:

1. ~~**Pretranslation phase.**~~ **✅ DONE** — probe-lowering (65
   files), Pass-0/2/2b/3 sidecars + prompt-builder multiplier, giant
   file-split, switch-dispatcher extraction, post-`shell ready` smoke.
2. ~~**Pruning PR.**~~ **Mostly moot.** xbox/pc98 already purged;
   diblib/dxg/ntddraw verified load-bearing (NOT prunable); vga_new
   orphans already pruned (smoke-gated). No separate pruning PR — the
   obsolete plan is superseded by the corrected prune-set above.
3. ~~**Substrate generation.**~~ **✅ DONE** — single `win32k-sys`
   crate (subsumes freetype-sys/win32k-types/gdi-ddi-sys),
   cargo-clean ×3, rustconv-wired (~82% resolve). Plus **ABI
   enforcement** (`abi_map.py`+`enforce_win32k_abi`) — calling
   convention is now deterministically *enforced*, not just
   `verify_abi`-checked post-hoc.
3a. ~~**REMAINING GATE: smoke-harness enhancements.**~~ Translation
   proceeded; per-file smoke-gating (the cohabitation flip + `shell
   ready` marker + screenshot) has been the working safety net across
   waves 1–9. Further post-`shell ready` assertions remain *desirable*
   but are no longer a hard blocker.
4. ~~**Phase 1 — handle tables.**~~ **✅ DONE.** `gdiobj.rs` (GDI table)
   + `object.rs` (USER table, `ntuser/object.c` flipped) live + green.
5. ~~**Phase 2 — GDI half.**~~ **✅ DONE.** The entire GDI half (ntgdi +
   eng + dib) is Rust and smoke-green ×3. `freetype.c` (long pole),
   `bitblt.c`→`ntbitblt.rs`, `d3dkmt.c`, the debug residue, and `ldevobj.c`
   all landed. Only non-translation C remains in `gdi/` (i386 `floatobj.c`
   by design; dead `USE_DIBLIB` `*_new.c`).
6. ~~**brush.cpp port.**~~ **✅ DONE** (wave 23, `cba2a2538fd`) — the only
   C++ file, flipped to `*mut _BRUSH` + Rust fns. First C++→Rust flip;
   pattern banked (learning #10).
7. **Phase 1.5 — typestate API design.** ⏳ **STILL OWED — gates the
   *deep* USER core** (callouts/messages/hooks), not the easy leaves.
   Implement `UserLock`, `CoRef<T>`, `CoCalloutContext`, `CoCallout` per
   `~temp-chat-consolidated.md` §6 (promote that file into
   `docs/engineering/` first). Spike on `co_IntClientLoadLibrary`.
   **2–3 sessions.**
8. ~~**Phase 3 — USER half (`user/ntuser`, ~73K LOC).**~~ ✅ **DONE
   (re-verified 2026-06-14)** — all 59 ntuser .c flipped, only
   `main_rust_shims.c` syscall tables stay C. The hard core (callouts,
   message queues, hooks, window-create chain) is flipped along with the
   leaves. <!-- VERIFY: Phase 1.5 typestate API (UserLock/CoRef/CoCallout)
   was declared "STILL OWED" as the gate for the deep USER core, yet the
   deep core flipped anyway — confirm whether the typestate API actually
   landed or whether the core was translated without it. -->.
9. **Per-arch verification.** All 3 arches green on smoke. **Continuous.**
10. **Cleanup.** Retire the C `win32ss/` kernel-mode tree; flip the
    `--allow-multiple-definition` cohabitation to Rust-only. Confirm the
    stale `device.rs` "FAILING" cmake annotation is downgraded. **1 session.**

**Remaining estimate: ~9–13 sessions** (Phase 1.5 ~2–3, Phase 3 USER
~6–9, cleanup ~1). The GDI half — the methodology bake-in — is **done**;
**USER is the bulk of what's left.** User-mode DLLs (user32/gdi32/imm32/
winsrv) remain out of scope (Phase N+1).

## Definition of done

> **Progress (2026-06-01):** the build/smoke/ud2 criteria below already
> hold for the **flipped subset** — `win32k-rust` builds ×3, smoke is
> green ×3, `scan_ud2 --check` is clean. They are not yet "done" for the
> *whole* binary because Phase 2 GDI has ~16 files left and Phase 3 USER
> is untouched. The list is the end-state bar; treat unchecked items as
> "true for what's flipped, pending for the rest."

- `cargo build -p win32k-rust --target <arch>` produces a working
  `win32k.sys` for all three arches. — **✅ holds today (flipped subset).**
- All three smoke tests pass on the cargo-driven binary, including
  the new post-`shell ready` assertions (cursor, taskbar paint,
  window-create cycle).
- `tools/scan_ud2.py --check` is clean.
- `tools/verify_abi.py`, `tools/verify_constants.py`,
  `tools/lint_kernel.py` are all clean against the Rust output.
- The cmake-driven `win32ss/` C tree (kernel-mode portion) can be
  deleted (or kept as parallel CI check).
- Smoke-test enhancement post-`shell ready` is landed and stable.
- Per-arch differentials documented (i386 floatobj asm path, amd64
  GDI batch flush quirks, arm64 desktop edge cases).

## What this audit doesn't measure

- **Boot correctness.** Smoke is the source of truth.
- **Per-arch differentials.** i386 has FLOATOBJ + dib_hline asm.
  amd64 + arm64 don't. arm64 has the most live-fix history (M11–M31
  per MEMORY.md). Translation will surface arch-specific bugs as it
  goes; no good way to predict in advance.
- **User-mode DLLs.** user32/gdi32/imm32/winsrv/etc. = ~140K LOC
  combined. **Phase N+1**, separate audit, separate Rust crate
  workspace. ABI is preserved through translation, so userland calls
  Rust kernel-mode through the syscall layer with no special handling.
- **DirectDraw / Direct3D / GDI+.** All out of scope. We don't smoke
  any DirectDraw apps. If/when we do, re-add `reactx/` and translate.
- **Print spooler.** `win32ss/printing/` is 28,600 LOC of userland
  spooler stuff. Out of scope.

## Open questions

1. **ftfd (FreeType Font Driver) target.** It's listed as an
   `add_importlibs(win32k ntoskrnl hal ftfd)`. Is `ftfd` a kernel
   driver we keep, a userland thing, or part of the freetype tree?
   Worth verifying during phase 0 substrate work.
2. **Per-arch translation order.** Ntoskrnl was translated arch-agnostic
   with `#[cfg(target_arch)]` gates. Win32k has fewer arch differences
   (16 i386-ifdef, 5 arm64-ifdef, 0 amd64-specific in `*.c`). Can
   stay arch-agnostic. **Recommendation: same pattern as ntoskrnl.**
3. ~~**brush.cpp specifically — Rust struct or fold into Rust impl?**~~
   **RESOLVED (wave 23).** `class BRUSH` had no vtable → modeled as
   `*mut _BRUSH` (bindgen struct) with file-internal C++ methods as
   private Rust fns and only the `extern "C"` API `#[no_mangle]`'d. See
   learning #10.
4. ~~**GDI handle table = port-as-trait or port-as-struct?**~~
   **RESOLVED.** Went port-as-struct (`gdiobj.rs`), mirroring the
   ntoskrnl ob handle table. Live + smoke-green.
5. ~~**freetype-sys: full or subset?**~~ **RESOLVED — moot.** No
   separate freetype-sys crate. `win32k-sys` already contains the FT_*
   surface (46 symbols) because `pch.h` pulls `ft2build.h`/
   `FT_FREETYPE_H` into the win32k bindgen. The `freetype.c` glue
   translates against `win32k_sys::win32k::FT_*`.
6. **The 50ms HACK in `winsta.c`.** Documented in MEMORY.md as a known
   i386 boot fix. Translation moment is the right time to actually
   diagnose + fix, vs. preserving the HACK byte-for-byte. **Recommendation:
   translate WITH the HACK preserved**, then file a separate ticket
   to root-cause it. Don't fix during translation.
7. ~~**Smoke-test clicker on amd64 + i386.**~~ **PARTLY RESOLVED.**
   `test_post_shell_ready_input` lands i386 + amd64 input via QMP
   `input-send-event` (keyboard qcode + relative mouse) — landed in
   pretranslation. Remaining harness enhancements are the only gate
   before Phase 1 (step 3a in Forward path); finish those, then this
   question is fully closed.
8. **What about `win32u/`?** It's a 47-line stub directory. May
   become the user-mode syscall stub library when we tackle
   user-mode DLLs. **Recommendation: leave alone for now; it's not
   in win32k.sys.**
9. **Per-arch `floatobj.S` — full Rust translation possible?** i386
   kernel mode lacks SSE, hence the fixed-point math. Could reproduce
   in pure Rust with `i64`/`u128` operations. ~$1K of LLM cost
   relative to ~5 LOC saved. **Recommendation: keep as `global_asm!`,
   not worth the spend.**

## Cross-references

- For the C-side rewrite gating this phase: [win32k-pretranslation-audit.md](win32k-pretranslation-audit.md).
- For the typestate API design: `qemu/~temp-chat-consolidated.md` §6.
- For the trap-chain analysis: [pseh-elimination.md](pseh-elimination.md).
- For the cargo build pattern (cdylib + custom target spec):
  [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) §
  *Cargo PoC, as it stands*.
