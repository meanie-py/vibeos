# Codex handoff — win32k DC-lifecycle hang in comctl32 InitializeClasses (i386)

**Branch:** `worktree-win32k-translation` (worktree `/home/curi/Projects/os-playground/vibeos-wt-win32k`)
**HEAD:** `787820400d9` (the ABI-collision fix described below is already committed)
**Task:** Static cross-model review of the 4 DC-lifecycle Rust translations to find a
**runtime hang** (spin or deadlock) that survived the ud2 fix. We need a *logic* bug,
not a build/link bug — the code compiles + links clean on all 3 arches.

---

## TL;DR of where we are

A C→Rust translation of the win32k DC-object-management tier landed:

| Rust file | C source | what it is |
|-----------|----------|------------|
| `sdk/lib/rust/win32k-rust/src/dclife.rs` | `win32ss/gdi/ntgdi/dclife.c` | DC alloc/init/cleanup, GreOpenDCW, NtGdiOpenDCW, IntGdiCreateDC |
| `sdk/lib/rust/win32k-rust/src/dcobjs.rs` | `win32ss/gdi/ntgdi/dcobjs.c` | NtGdiSelect*, DC_vUpdate*Brush, DC_hSelectFont |
| `sdk/lib/rust/win32k-rust/src/dcstate.rs` | `win32ss/gdi/ntgdi/dcstate.c` | SaveDC/RestoreDC, DC_vCopyState |
| `sdk/lib/rust/win32k-rust/src/dcutil.rs` | `win32ss/gdi/ntgdi/dcutil.c` | Get/Set Bk/Text/Map/GraphicsMode, DPtoLP/LPtoDP, BoundsRect |

These are cohabitation-flipped (C commented out in `win32ss/CMakeLists.txt`, Rust
`#[no_mangle]` symbols take over). The companion **region algebra** translation
(`region.rs`, `win32ss/gdi/ntgdi/region.c`) landed green *earlier* and is NOT the
focus — but it IS on the CreateDC path (visible-region init), see below.

### Already fixed (do not re-investigate)
The DC wave first crashed with **BUGCHECK 0x1E** at the first CreateDC: `GreOpenDCW`'s
entire body was a single `ud2`. Root cause was a **calling-convention collision** —
rustconv declared 10 functions in bare `extern "C"` blocks whose *definitions* are
stdcall / NTAPI(system) / FASTCALL. Within one crate LLVM sees one `#[no_mangle]`
symbol with two conventions → mismatched-CC call is UB → the *caller* folds to `ud2`.
(`EngpGetPDEV` declared `extern "C"`, defined `extern "system"` in `pdevobj.rs` was the
GreOpenDCW killer.) Fixed in `787820400d9`: ud2 count in `win32k.sys` dropped 9 → 2
(the 2 left are `region.rs` unwrap panic-stub allowlist entries, expected).

---

## The remaining bug — a HANG, not a crash

With the ud2 gone, i386 now boots **much** further: kernel → SMSS → csrss → win32k
(`[WIN32K] shell ready`) → winlogon (creates Default/Winlogon/Screen-Saver desktops,
SwitchDesktop OK) → services.exe (PID 152) → lsass.exe (PID 160). **No BUGCHECK.**

It then **hangs** while lsass (PID 160) runs comctl32's DllMain. Exact serial tail
(identical every run, ~175 KB then dead silence for the full 360 s timeout):

```
[LDR] DllMain(comctl32.dll) ATTACH PID=160 Base=7B850000
[comctl32] DllMain PROCESS_ATTACH
[comctl32] before GlobalAddAtomW
[comctl32] after GlobalAddAtomW
[comctl32] before CreateBitmap
[comctl32] before CreatePatternBrush
[comctl32] before RefreshSysColors
[comctl32] before InitializeClasses        <-- LAST LINE, then hang
```

Read the markers as "before X" printed → X runs → next "before Y" printed means X
returned. So `CreateBitmap`, `CreatePatternBrush`, `RefreshSysColors` all **completed**
(basic GDI + brush + color work is fine). The hang is inside **`InitializeClasses`**
(comctl32 `dll/win32/comctl32/commctrl.c`) — which registers the common-control window
classes and, per the pre-fix crash signature, triggers the **first CreateDC**
(`NtGdiOpenDCW`). The pre-fix BUGCHECK was *at that very CreateDC*; now CreateDC no
longer crashes but the path does not return.

### What this means
The DC code now **executes** and one of these is true:
1. A DC function on the CreateDC path **spins** (infinite loop), or
2. CreateDC **returns a malformed DC** and a downstream win32k op spins, or
3. A **lock/semaphore acquired in the DC path is never released** → the next GDI op
   deadlocks. This is a known recurring rustconv defect — it drops the lock-release
   epilogue when inlining (see `feedback_con_api_lock_release_dropped`).

We could NOT get a live backtrace: this sandbox blocks detached QEMU from grabbing
`/dev/kvm` and denies `-gdb tcp` bind, so only foreground-with-timeout boots work (no
concurrent gdb / CPU sampling). Hence this static-review request.

---

## The CreateDC call chain that now runs (review these in order)

`NtGdiOpenDCW` → `GreOpenDCW` → in `GreOpenDCW`:
1. `EngpGetPDEV(pustrDevice)` (pdevobj.rs — not in this wave)
2. `DC_AllocDcWithHandle(LO_DC_TYPE)`  — `dclife.rs` ~161; `GDIOBJ_AllocateObject` +
   `GDIOBJ_hInsertObject` (returns the object **locked**)
3. `DC_vInitDc(pdc, iType, ppdev)`     — `dclife.rs` ~258; the big initializer.
   Calls `IntSysCreateRectpRgn`, **`UpdateVisRgn(pdc)`** (FASTCALL — into clip/region),
   `IntEngInitClipObj`, `PALETTE_ShareLockPalette`, `BRUSH_ShareLockBrush`,
   `PEN_ShareLockPen`, `EBRUSHOBJ_vInitFromDC` ×4, `LFONT_ShareLockFont`.
4. `DC_InitHack(pdc)`                   — `dclife.rs` ~228
5. `DC_bAllocDcAttr(pdc)`
6. **`DC_UnlockDc(pdc)`**               — MUST release the lock taken at alloc. If this
   is a no-op / dropped / wrong, the DC stays locked and the next `DC_LockDc` spins.

### Specific things to check (prioritized)
- **Lock balance:** `DC_AllocDcWithHandle` locks the object; `GreOpenDCW` ends with
  `DC_UnlockDc`. Verify `DC_UnlockDc`/`DC_LockDc` are real (not stubbed) and that every
  `EngAcquireSemaphore((*(*pdc).ppdev).hsemDevLock)` in `dclife.rs` (DC_vFinishBlit /
  DC_vPrepareDCsForBlit, ~883/913/986/1016) has a matching release on **every** path —
  rustconv loses the release when it flattens early-returns.
- **`UpdateVisRgn` arguments / return:** it's the one region call in `DC_vInitDc`.
  Compare the Rust call + the `prgnVis = IntSysCreateRectpRgn(0,0,cx,cy)` that feeds it
  against `dclife.c`. A wrong rect (e.g. a truncated `as i32`, or cx/cy swapped) could
  produce a degenerate region that loops a band-walk in `region.rs`
  (`REGION_*` has many `while`/`loop` band iterators).
- **`DC_vUpdate{Background,Text,Fill,Line}Brush` (dcobjs.rs):** these were also in the
  pre-fix ud2 set; FASTCALL. Diff against `dcobjs.c` for dropped statements / inverted
  conditions. `feedback_rustconv_dropped_fn_calls`: a bare tuple `(a, b);` (missing the
  fn-name ident) is a silent no-op the compiler accepts.
- **Off-by-one / sentinel / truncation:** the classic rustconv defects —
  `>= u64 as i64` inverting an unsigned sentinel (`feedback_u64_sentinel_as_i64`),
  silent `as u8/u16/u32` narrowing of a count/loop bound
  (`feedback_rustconv_truncation_bugs`), `write_bytes::<T>(p, v, count)` passed a
  *byte* count (`feedback_write_bytes_element_count`). Any of these in a loop bound = spin.
- **Self-ref / null list init:** `LIST_ENTRY = {&head,&head}` translated to `{null,null}`
  (`feedback_rustconv_selfref_static`) — a list walk over a {null,null} head loops or
  faults. Check any list heads the DC path touches.

The C sources to diff against are in the same worktree:
`win32ss/gdi/ntgdi/{dclife,dcobjs,dcstate,dcutil}.c`.

---

## How to reproduce / verify a fix (i386)

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-win32k
# build
ninja -C build-i386 win32k.sys && ninja -C build-i386 livecd
# ud2 sanity (expect 2 — region panic stubs only)
.build-tools/llvm-mingw/bin/aarch64-w64-mingw32-objdump -d build-i386/win32ss/win32k.sys | grep -c ud2
# boot (OVMF32 UEFI El Torito; the livecd is EFI-only, legacy BIOS won't boot it)
qemu-img convert -f raw -O qcow2 qemu/OVMF32_VARS.fd /tmp/vars.qcow2
timeout 200 qemu-system-i386 -M pc -accel kvm -cpu host -m 512M \
  -drive if=pflash,format=raw,file=qemu/OVMF32_CODE.fd,readonly=on \
  -drive if=pflash,format=qcow2,file=/tmp/vars.qcow2 \
  -drive id=cd0,if=none,format=raw,file=build-i386/livecd.iso,readonly=on \
  -device ide-cd,drive=cd0,bootindex=0 -vga std -display none -no-reboot \
  -serial file:/tmp/s.log
grep -a "InitializeClasses\|_tWinMain entered\|shell ready" /tmp/s.log
```
**Fixed** = serial gets past `[comctl32] before InitializeClasses` and eventually prints
`[EXPLORER] _tWinMain entered` (the smoke boot marker). Full smoke:
`pytest tests/qemu/test_smoke.py -v --arch i386-uefi --timeout 360 --skip-clean-build`
(needs `qemu/OVMF32_{CODE,VARS}.fd` — copy from a sibling worktree's `qemu/` if absent).

## Hard rules (from AGENTS.md — do not violate)
- **Never revert Rust → C**, never delete a `.rs`, never comment a Rust mod out of the
  build to dodge the regression. Fix in place.
- The translation is the goal; a broken-but-integrated translation visible in smoke is
  the correct state until fixed.
- Match the surrounding Rust idiom; preserve the per-arch ABI (`extern "system"` for
  stdcall/NTAPI, dual-cfg `extern "fastcall"`/`"C"` for FASTCALL, `extern "C"` for cdecl).
