# Delay-Import Support for llvm-mingw — Design & Plan

**Status:** design / spike (2026-06-20). **Owner:** user32 lane (picked up as an independent
toolchain unit; it is *not* user32-specific — see §8 Scope/ownership).
**Lane branch:** TBD (`delay-import` worktree). **Type:** build-system / toolchain.

> This is a **self-contained unit of work** independent of the user32 translation waves.
> It fixes a toolchain gap that currently forces per-module stub workarounds OS-wide.

---

## 1. Problem

`llvm-dlltool` (our import-lib generator, llvm-mingw) **does not support `--output-delaylib`**
(GNU binutils `dlltool -y`). Confirmed: `llvm-dlltool --help` has no delay option; the binutils
`dlltool` we also have is **i386-only** (`-m` machine list: `arm[_interwork], i386, mcore, thumb`
— no arm64, no x86-64), so it cannot rescue the arches that matter.

Consequence (see `sdk/cmake/gcc.cmake`): `add_delay_importlibs()` gates on
`if(CMAKE_DLLTOOL_MACHINE)` — which is set for **all three** llvm-mingw arches
(`toolchain-llvm-mingw.cmake` i386/`i386:x86-64`, `toolchain-arm64.cmake` arm64). On that branch
it:
- creates an **empty** `lib<name>_delayed.a` (gcc.cmake:800–808),
- links the module against the **regular** import lib (`lib<name>`), not `_delayed`,
- **skips linking `delayimp`** (the runtime helper).

**Net effect: nothing delay-loads on any arch.** Every `add_delay_importlibs(...)` silently
degrades to a regular (load-time) import. It "works" only because the target DLLs happen to be
present at boot — at the cost of eager loading + full transitive-dependency closure.

### Why it matters (the damage this causes today)
Because real delay-load is unavailable, modules that *want* lazy/optional dependencies must
**stub them out** instead. Four such workarounds exist, all `#ifdef _M_ARM64` stub TUs:

| Module | Stubbed dep | Lost capability |
|---|---|---|
| `win32ss/user/user32` | libpng | PNG-compressed icons/cursors (`cursoricon.c`) |
| `dll/win32/advapi32` | secur32 | LSA logon functions |
| `dll/win32/rpcrt4` | ole32/oleaut32/wininet | COM/OLE marshalling, HTTP-over-RPC |
| `win32ss/user/winsrv/consrv` | shell32/ole32 | clipboard/OLE |

The `consrv` case is the tell: its comment says regular-importing shell32 *"causes csrss to load
shell32 + 30 transitive deps at startup, overflowing the user-mode stack."* That overflow is a
**direct symptom** of the delay→regular fallback. Real delay-load lets us **delete these stubs**,
not just one.

---

## 2. What already exists in-tree (the good news)

This is **not** a build-the-whole-stack project. Three of the four pieces are already present:

1. **Runtime helper — DONE.** `sdk/lib/delayimp/` builds a `delayimp` static lib providing the
   MS-compatible `__delayLoadHelper2(PCImgDelayDescr pidd, PImgThunkData pIATEntry)`
   (`delayimp.c:58`) + the `pfnDliNotifyHook2`/`pfnDliFailureHook2` hooks. It just isn't linked on
   our builds (the cmake gate skips it).
2. **cmake consumption path — DONE.** The `else()` branches of `add_delay_importlibs` /
   `add_importlib`'s delay section already do the right thing: generate `_delayed.a` via
   `--output-delaylib`, link `lib<name>_delayed` + `delayimp`. We only need to make the
   `if(CMAKE_DLLTOOL_MACHINE)` branch produce a **real** `_delayed.a` instead of an empty one
   (and re-enable the `delayimp` link on that branch).
3. **Descriptor format — DONE / authoritative in-tree.** `sdk/include/psdk/delayimp.h`:
   ```c
   typedef struct ImgDelayDescr {
       DWORD grAttrs;        // 1 = dlattrRva (all fields are RVAs)
       RVA   rvaDLLName;     // → "name.dll\0"
       RVA   rvaHmod;        // → HMODULE slot (one per DLL, init 0)
       RVA   rvaIAT;         // → delay Import Address Table (PImgThunkData[])
       RVA   rvaINT;         // → delay Import Name Table (PImgThunkData[])
       RVA   rvaBoundIAT;    // 0
       RVA   rvaUnloadIAT;   // 0
       DWORD dwTimeStamp;    // 0
   } ImgDelayDescr;
   ```

**The only missing piece is a `--output-delaylib` replacement** — a generator that emits the
delay-import library `.a` from a `.def`, for llvm-mingw COFF (i386 / x86-64 / arm64).

---

## 3. The contract (what the generated `.a` must provide)

From `__delayLoadHelper2` (`delayimp.c`): the per-function thunk calls
`__delayLoadHelper2(pidd, &pIAT[index])` — it passes the descriptor and the **address of this
function's own IAT slot**; the helper derives `index` by pointer subtraction
(`IndexFromPImgThunkData(pIATEntry, pIAT)`), reads `pINT[index]` for the name (an
`IMAGE_IMPORT_BY_NAME` via RVA) or ordinal, `LoadLibraryA(szDll)`, `GetProcAddress`, **patches
`pIAT[index].u1.Function`** to the resolved address, and returns it.

So for a DLL `D` exporting functions `F0..Fn`, the delay-lib must contribute (all in the *final
linked module*, via objects the linker pulls from the `.a`):

- **`rvaDLLName`** → the string `"D.dll\0"`.
- **`rvaHmod`** → one writable pointer slot (init 0) — the cached `HMODULE`.
- **`rvaIAT`** → array of `n+1` thunk-data slots; slot `i` initialized to **point at thunk `Fi`**
  (so the first indirect call lands in the thunk), terminated by a 0 slot.
- **`rvaINT`** → array of `n+1` slots; slot `i` = RVA of `IMAGE_IMPORT_BY_NAME { Hint; "Fi\0" }`
  (or `IMAGE_ORDINAL_FLAG | ord`), 0-terminated.
- **The descriptor** `ImgDelayDescr` (one per DLL) with the four RVAs above + zeros.
- **Per-function public symbol + thunk.** Callers bind to `Fi` (the import stub: `jmp *__imp_Fi`).
  `__imp_Fi` is the IAT slot. The IAT slot starts pointing at a **tailMerge thunk** that:
  loads `pidd` and `&__imp_Fi`, calls `__delayLoadHelper2`, then jumps to the returned address.
  (GNU `dlltool --output-delaylib` emits one shared `__tailMerge_D` + per-fn `__imp_load_Fi`
  trampolines; we can do the same, or a per-fn thunk. Exact factoring decided in the spike.)

**Decoration (the i386 trap):** the public symbol is `_Fi@N` (stdcall) and the IAT slot is
`__imp__Fi@N`; x86-64/arm64 are undecorated (`Fi` / `__imp_Fi`). The `.def` already encodes the
`@N` (it's the same `_implib.def` used for the regular import lib), so we read decoration from
there — do **not** re-derive it.

---

## 4. Implementation approach — generate source, let clang emit COFF

**Do not hand-emit COFF or post-process the linked PE.** The hard parts (machine-code thunks +
base relocations for every RVA) are exactly what a compiler does correctly. So:

```
<name>_implib.def  ──►  gen_delaylib.py  ──►  <name>_delay.c   (descriptor + INT/IAT + names, using delayimp.h types)
                                          └─►  <name>_delay_<arch>.S   (the tailMerge thunk(s))
                                                     │
                                          clang --target=<triple> -c  ──►  *.o (COFF + relocs done by clang)
                                                     │
                                          llvm-ar crs  ──►  lib<name>_delayed.a
```

- **`<name>_delay.c`** — pure data, written against `delayimp.h` structs. The descriptor, the
  `IMAGE_THUNK_DATA` IAT (each entry `= (DWORD_PTR)&thunk_Fi`), the INT (each `= (DWORD_PTR)RVA`…
  but RVAs aren't known at compile — use **symbol addresses + relocations**: declare the
  name/thunk symbols `extern` and let the linker fill the slots; clang emits `IMAGE_REL_*`
  relocs and lld + the loader's base-reloc pass make them image-relative at runtime). This is the
  key trick — express RVAs as `&symbol`, never as literals.
- **`<name>_delay_<arch>.S`** — the tailMerge stub per arch. i386: push descriptor + IAT-entry,
  `call ___delayLoadHelper2`, `jmp *eax`. x86-64: `rcx`/`rdx` set, `call`, `jmp rax` (shadow
  space). arm64: `x0`/`x1` set, `bl`, `br x0`. Small and fixed; ~one stub per DLL.
- **`gen_delaylib.py`** — parse the `.def` (name + `Fi @N` list), emit the two source files, drive
  clang + llvm-ar. Lives in `sdk/cmake/` or `tools/` and is invoked from the
  `add_delay_importlibs` custom command (replacing the empty-archive branch).

**cmake wiring (gcc.cmake):** in the `if(CMAKE_DLLTOOL_MACHINE)` branch, replace the empty
`ar crT` with a call to `gen_delaylib.py --def <implib.def> --arch <CMAKE_DLLTOOL_MACHINE> --out
<name>_delayed.a`; in `add_delay_importlibs`, on that branch, link `lib<name>_delayed` + `delayimp`
(i.e. make both branches behave the same once the generator works). Keep a `VIBEOS_NO_DELAYLOAD`
escape hatch that restores the empty-archive + regular-import fallback (graceful degradation).

---

## 5. Spike (do this first — host-side, ~1 hour, fails cheap)

Smallest end-to-end proof, **i386 only, one DLL, one or two functions**:
1. Pick a trivial delay target (e.g. a 1-function `.def`). Run `gen_delaylib.py` → `lib_delayed.a`.
2. `llvm-ar t` / `llvm-nm` the archive: confirm it exports `_Foo@N`, `__imp__Foo@N`, the
   descriptor symbol, and pulls `___delayLoadHelper2`.
3. Link a trivial test DLL/EXE that calls `Foo`, against `lib_delayed.a` + `delayimp`.
4. **`llvm-objdump -p` / a small `pefile` script** on the output: confirm the **delay-import
   directory** (data dir index 13) is present and well-formed (descriptor RVAs resolve to the
   name string, IAT, INT; IAT slots point into our thunk; `.reloc` covers them).
5. Bonus host-side check: load it under wine or a unit harness and confirm first call triggers
   `LoadLibrary` (or that the structure matches a known-good MSVC-built delay PE byte-for-byte).

If the directory is well-formed and the symbols resolve, the approach is proven → scale to
x86-64 + arm64 (new `.S` stubs only) and wire the cmake. **Only the final OS-boot proof needs
KVM** — everything above is host CPU, no smoke slot.

---

## 6. Validation & rollout

- **Host-side gates (no KVM):** generator unit output (golden `.c`/`.S` per `.def`); `llvm-objdump
  -p` delay-dir well-formedness on a representative set (user32→libpng, advapi32→secur32,
  rpcrt4→ole32, consrv→shell32); a regular `cargo`/`ninja` link of each affected module.
- **Boot proof (KVM, minimal):** re-enable **one** delay-import (start with user32→libpng on
  i386/amd64), boot, confirm the OS still reaches shell-ready AND that the delayed DLL is *not*
  loaded until exercised (check module list). Then arm64.
- **Stub removal (the payoff):** once delay-load works on arm64, delete the four
  `arm64_*_stubs.c` TUs and restore the real delay-imports; re-gate ×3.
- **Cross-cutting risk:** `gcc.cmake`'s delay path touches **every** module using
  `add_delay_importlibs`. The `VIBEOS_NO_DELAYLOAD` fallback + a staged rollout (one module at a
  time, fallback default-on until proven) keeps a regression from breaking the whole build.

---

## 7. Leverage

- Restores **real delay-load semantics OS-wide** (today *nothing* delay-loads on llvm-mingw).
- **Removes 4 arm64 stub workarounds** (libpng/secur32/ole32/shell32) → real PNG icons, LSA,
  COM/OLE, clipboard on arm64.
- **Fixes the root cause of the consrv stack overflow** (eager 30-dep transitive load).
- Shrinks every module's load-time dependency closure → faster, more robust early boot.
- Reusable forever (any future optional dependency).

---

## 8. Scope / ownership / coordination

- This is a **build/toolchain-lane** change, not user32. It edits shared `sdk/cmake/gcc.cmake`
  + adds a generator tool + links `delayimp`. Coordinate with **control-plane (meaniepy)** and the
  **build-cache/determinism lane** before landing; do dev on an isolated worktree.
- **arm64 gate is currently blocked** (the floor-completion train is parked on
  `arm64_lpcexitthread_race`, arm64-love's lane). i386/amd64 dev + proof can proceed now; the
  arm64 stub-removal + full ×3 lands once arm64 is deterministically green. The generator itself
  is arch-parametric, so arm64 support is ready before the gate is.
- **Never reverts/deletes** the existing stub TUs until their real delay-import is proven (keep
  them until §6 stub-removal, then delete in the same commit that re-enables the import).

---

## 9. Open questions (resolve during the spike)
- Per-function thunk vs one shared `__tailMerge_D` + per-fn `__imp_load_Fi` trampolines (match GNU
  dlltool, or simplify)? Decide from what links cleanly + matches `__delayLoadHelper2`'s index math.
- Does lld's GNU driver honor the `IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT` data-dir entry from a
  linked-in descriptor, or must we set it explicitly? (objdump -p in the spike answers this.)
- `__imp_` IAT-slot symbol naming + whether lld auto-creates the import stub `Fi` from `__imp_Fi`,
  or we emit `Fi: jmp *__imp_Fi` ourselves.
- i386 `@N` decoration round-trip through clang asm symbols (the `@` in symbol names).
