
# ntoskrnl Translation Status (updated 2026-04-27)

## Smoke Test Status

| Arch | Result | Notes |
|------|--------|-------|
| **i386** | **7/7 PASS** | Full graphical desktop with start menu/taskbar |
| **amd64** | **7/7 PASS** | Explorer regression cleared (was intermittent; clean rebuild + KdProviders init together resolved it) |
| **ARM64** | 4/7 | 2 skipped (objdump path), 1 failing — `[SMSS] _main` never fires (silent failure inside `LdrInitializeThunk` or CRT init) |

ARM64 reaches `[PS] PspUserThreadStartup: returning` reliably (kernel + user-mode dispatch work after the MMPTE soft-PTE bit-layout fix), but `_main` doesn't fire — likely a separate user-mode loader bug.

## Translation Coverage (additions since last update)

### KD — fully Rust on all 3 arches (this session)

| File | C lines | Rust file | Notes |
|------|--------:|-----------|-------|
| kdio.c | 355 | kd/kdio.rs (1,525) | Owns globals KdpDebugMode/KdProviders/DispatchTable/InitRoutines |
| kdmain.c | 563 | kd/kdmain.rs (795) | Defensive `InitializeListHead(&KdProviders)` — Rust static-init can't express the C `LIST_ENTRY KdProviders = {&KdProviders, &KdProviders}` self-reference |
| kdterminal.c | 443 | kd/kdterminal.rs (488) | cdecl ABI (matches C) |
| kdprompt.c | 182 | kd/kdprompt.rs (243) | |
| kdps2kbd.c | 128 | kd/kdps2kbd.rs (250) | x86 in/out via inline asm |
| kdserial.c | 256 | kd/kdserial.rs (80) | Generic wrapper |
| i386/kdserial.c | 115 | i386/kdserial.rs | Used by BOTH i386 + amd64 (cfg `any(x86, x86_64)`) |
| arm64/kdserial.c | 100 | arm64/kdserial.rs | PL011 |
| arm/kdserial.c | 100 | arm/kdserial.rs | PL011 |

**Net: 2,242 lines C → 3,914 lines Rust.** All 3 arches green. Plus a tiny `kd_rust_shims.c` (40 LOC) that bridges Rust → HAL's `KdComPortInUse` dllimport (cross-DLL imports can't be resolved by Rust extern decls on i386 — need `__imp_` thunks the C compiler emits via `NTHALAPI`/`__declspec(dllimport)`).

### MM section/{io,sptab} — Rust on all 3 arches (this session)

`cache/section/io.c` (338 LOC) + `cache/section/sptab.c` (429 LOC) translated to `mm/section_io.rs` (173 LOC) + `mm/section_table.rs` (367 LOC). These were the two unconditionally-compiled cache/ files; the rest is NEWCC dead code. **Wired in `mm/mod.rs:16-17`; C versions commented out at `ntos.cmake:50-51`. Active in build.** `mm_rust_shims.c` gained cdecl outline trampolines for `MiAcquirePfnLock` / `MiReleasePfnLock` (they were FORCEINLINE in `mm.h` with no real symbol — needed for the Rust callers to link).

### kd64 serial passthrough + cache/ NEWCC line-commented (earlier this session)

- kd64: `BREAKPOINT_PRINT` (DbgPrint) now writes to COM1 (0x3F8) directly when no debugger is attached. Plus `int 2c` skip and `0xCC 0xCD 0x2C` (int3+int 2c) auto-skip.
- 11 NEWCC-only cache/* files line-commented as dead code (replaced by Rust `cc/`).

### config/ — first attempt failed (this session)

All 22 `.c` files translated to Rust (~30K LOC) and integrated. Compile-clean across 3 arches, links cleanly. But i386 BSODs very early in boot with `STATUS_ILLEGAL_INSTRUCTION` jumping to near-NULL function pointer — before any DbgPrint flushes. **Stashed in `git stash@{0}`** awaiting Codex-token reset for fresh-eyes debugging. Likely culprits: bindgen `__bindgen_anon_*` accessor mismatch, ABI mismatch on `cm_rust_shims.c` `__asm__("_Name@N")` aliases, or struct-size mismatch on a newly-defined cmsysini.c-owned global (e.g., ERESOURCE).

## Boot-Milestone Harness (from prior session)

`tests/qemu/test_smoke.py::test_milestones` validates that each arch's `required_milestones` list (per `tests/qemu/vm.py`) appears in serial output. Markers:
- `[SMSS] _main entered`
- `[CSRSS] _main entered`
- `[WINLOGON] WinMain entered`
- `[EXPLORER] _tWinMain entered`
- `shell ready`

## Subsystem Coverage Snapshot

281 files, ~360,500 lines translated to Rust active across i386/amd64/arm64. Remaining C in ntoskrnl proper:
- **config/** (~19K lines) — translated, awaiting bisection (stashed)
- **kd64/** (~7K lines) — translated but disabled due to recursive exception flood
- **kdbg/** (~12K lines) — not translated; i386-dis.c (binutils, ~4.4K lines) will stay in C
- **misc shims** (~1.5K lines) — guid.c, kernel_c_helpers.c, *_rust_shims.c — stays in C by design
- **tests/** — kernel test harness, not for translation

## Key Patterns Learned This Session

- **C wide-string `L"..."` literals**: must become `&[u16]`, NOT bytes-cast-to-`*const u16`. Also fixed in fstub session.
- **Cross-DLL imports** (HAL globals like `KdComPortInUse`): need `__imp_` thunk via `NTHALAPI`/`__declspec(dllimport)`. Rust extern decls can't reach these on i386 — route through a tiny C shim file (`kd_rust_shims.c` pattern).
- **C `LIST_ENTRY Foo = {&Foo, &Foo}`** self-referencing static initializer can't be expressed in Rust const init. Two options: (a) call `InitializeListHead(&Foo)` defensively at first-use sites, or (b) Rust ctor pattern via `#[link_section = ".CRT$XCU"]`.
- **C macros mistakenly translated as functions** (UlongToPtr, RemoveEntryList, KdbpGetCharSerial, HalInitPnpDriver): need inline Rust impl, not extern.
- **NTAPI vs cdecl ABI matters**: kd/ has both. Look at the C source — `NTAPI`/`__cdecl` keywords determine the Rust extern ABI. Mismatch → `_Name` undefined vs `_Name@N` undefined.
- **Cargo incremental cache lies**: `find . -name rust-kernel-target -exec rm -rf {} +` is a frequent debug step. Without it, ABI fixes don't take effect even though they're in source.

## What's Next (priority order)

1. **config/ bisection** — fresh-eyes debugging once Codex tokens reset. Stashed translation is mechanically sound; bug is a runtime mismatch in some subset.
2. **ARM64 SMSS reaches `_main`** — debug why `LdrInitializeThunk` or CRT init silently fails. Highest-leverage if we want ARM64 desktop back.
3. **kd64 recursive exception flood** — already translated but disabled. Debug not retranslate.
4. **kdbg/** — translate the 8 .c files (excluding i386-dis.c which stays in C).

## Open Architectural Issues

- **CGU=1 on i386/amd64** — ~15 unexpected ud2s due to LLVM interprocedural noreturn contagion. Workaround `codegen-units=16` + `opaque_ptr` barriers on 2,449 fns shipped, root not yet fixed.
- **kd64/ disabled** — translated but disabled because of recursive exception flood on amd64 (`KdpTrap → KdpReport → KdSendPacket` infinite nesting). Needs the bug debugged, not retranslated.
- **`--allow-multiple-definition` linker flag** — masks dual C/Rust symbol definitions. Should be removed once final C holdouts are translated.
