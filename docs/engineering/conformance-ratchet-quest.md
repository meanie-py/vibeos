# Conformance Ratchet — run the in-tree NT test corpus, baseline it, never regress

**Status: QUEST (2026-07-10, lane `conformance-ratchet`, worktree
`~/Projects/os-playground/vibeos-conformance`). Handoff doc — read fully before starting.**

## Mission

Today the project's entire correctness signal is "boots to a desktop" (+28 serial kernel
tests, +15 net tests). That proves roughly one path through each subsystem. Meanwhile the
repo carries a **massive dormant test corpus** we forked from ReactOS and have **never
compiled on our toolchain**: `modules/rostests/` — kernel-mode API tests (`kmtests`),
user-mode API tests (`apitests`), and 132 Wine test modules (`winetests`).

This lane makes that corpus build, run in-guest, and gate:

1. **Build** the corpus on llvm-mingw (kmtests first — highest signal per byte).
2. **Run** it in-guest with results captured host-side.
3. **Ratchet**: record a per-arch baseline of what passes; from then on, newly-failing
   = red, newly-passing gets ratcheted in. Like the lint baselines
   (`tools/lint-baseline/`), but for runtime semantics.

Why it's the highest-value correctness lane: every kmtest failure against a
Rust-translated subsystem is a **pre-packaged mistranslation repro** with a named API and
an assertion message — no crash forensics needed. And it executes translated code the
boot path never touches (see `tools/coverage/` — the never-executed-rust registry).

## Survey findings (2026-07-10 — verified, start from here)

- **`ENABLE_ROSTESTS` is `OFF` by default** (`modules/CMakeLists.txt`). No rostests
  target exists in any build dir; ninja has never seen them. The whole corpus is
  greenfield on llvm-mingw — expect a build-fix grind (see "Expected build hazards").
- **Corpus size:** kmtests = 137 C files (34 use PSEH); winetests = 132 modules;
  apitests = another large tree (ntdll, kernel32, gdi32, advapi32, ws2_32, …).
- **kmtest architecture:** `kmtest.exe` (user-mode controller) creates/starts the
  `kmtest_drv.sys` service, invokes a test by name; kernel side fills a shared
  `KMT_RESULTBUFFER` LogBuffer (`include/kmt_test.h`); the exe writes it to **stdout**.
  Some tests are paired user+driver (`*_user.c` / `*_drv.c`, e.g. NtCreateSection).
- **Test inventory hits our translations dead-on:** `RtlAvlTree`, `RtlSplayTree`
  (our avltable/splaytree crates), `CcCopyRead/Write`, `CcMapData/PinRead`,
  `MmSection`/`MmMdl`/`ZwMapViewOfSection`, `KeApc/KeDpc/KeTimer/KeSpinLock/KeIrql`,
  `IoDeviceObject/IoCreateFile`, `ObReference`/`ObType`, `TcpIpTdi`, `FsRtl*`.
- **`rosautotest`** (`modules/rostests/rosautotest/`) is ReactOS's in-guest runner for
  winetests — it has `add_cd_file(... FOR all)` already. Candidate driver for the
  winetest phase; NOT needed for kmtests.
- **Already done in this worktree:** the stale amd64 kmtests exclusion in
  `modules/rostests/CMakeLists.txt` is removed (upstream excluded it for GCC "new x64
  PSEH" breakage — moot, we're `USE_DUMMY_PSEH` on all arches).

## Recipe

```bash
W=~/Projects/os-playground/vibeos-conformance      # ALL paths $W-prefixed (worktree discipline!)
cd $W && ./vibeos-cli build --arch amd64           # provision build dir (auto, cache-warm)
cd $W/build-amd64
../tools/canon-run.sh cmake -DENABLE_ROSTESTS=ON . # option is cached; sticks for later builds
../tools/canon-run.sh ninja kmtest kmtest_drv      # start here, then widen
```

## Expected build hazards (from prior porting experience — see AGENTS.md)

- **PSEH:** all arches use `USE_DUMMY_PSEH`; `KmtStartSeh`/`_SEH2_TRY` should *compile*
  (dead handlers), but tests that assert "this raises STATUS_X" will **fail or crash at
  runtime**. That is *signal*, not a build problem — baseline them as failing; do NOT
  hack tests to pass. A guest crash mid-suite is expected early; make the runner resilient
  (run tests individually / in order, tolerate reboot, record "CRASHED" as a result).
- **llvm-rc:** stricter than rc.exe — ACCELERATORS `ASCII`+modifiers, DIALOGEX 16-bit
  coords, FILEVERSION 16-bit fields (AGENTS.md "llvm-rc Limitations").
- **MSVC-isms / gcc pragmas** in test sources; `__drv_dispatchType` etc. should be
  no-ops via sdk annotations. Fix forward with minimal shims; keep test *assertions*
  untouched (the upstream corpus is the oracle — a modified assertion is a weakened test).
- Track every source tweak you must make in this doc's progress log.

## In-guest execution + capture (M2 design)

Preferred vehicle: the **payload harness** (`docs/engineering/payload-harness-design.md`)
— YAML → media staging + autostart + host-side test driving; doom/httpd already use it.
Quick-and-dirty alternative for first light: `add_cd_file` kmtest.exe+kmtest_drv.sys,
autostart via the Run-key mechanism, and tee kmtest's output to serial — smallest patch:
in kmtest.exe's result-print path, also `OutputDebugStringA` each line (guard with a
`VIBEOS_KMTEST_SERIAL` define; harness-support additions to kmtest infra are fine, only
assertions are sacred). Kernel-side DbgPrint already reaches serial on our debug builds.

Run tests **one process per test** (`kmtest.exe <name>`) — one hanging/crashing test must
not eat the suite. Emit a per-test start/end marker so the host parser can attribute
hangs: `[KMTEST:BEGIN <name>]` / `[KMTEST:END <name> pass=N fail=M]`.

## The ratchet (M4)

- `tools/conformance-baseline/<arch>/kmtest.txt`: one line per test —
  `name pass_count fail_count skip|crash|hang` at baseline time.
- `./vibeos-cli conformance --arch X`: boot → run suite → parse → `summary.json` with
  `{new_failures: [], new_passes: [], baseline_drift: ...}`. **Report-only** until the
  baseline is stable across ≥3 runs (flaky tests get quarantined into a `flaky.txt`
  list, matched by name, never silently). Then wire as an opt-in gate stage; default
  gate stays as-is until the user opts to include it.
- Triage the initial fail pile into: (a) our-bug (translated subsystem misbehaves),
  (b) never-worked-upstream (ReactOS also fails it — check upstream testman if in doubt,
  or just baseline it), (c) DUMMY_PSEH-structural. Only (a) items become fix work.

## Milestones

- **M1** — `kmtest.exe` + `kmtest_drv.sys` build on amd64 (then i386, arm64). Ship the
  build fixes as their own commit(s).
- **M2** — one kmtest (`Test_Example`, then `RtlAvlTree`) runs in-guest, output captured
  host-side. First real conformance datapoint.
- **M3** — full kmtest sweep per arch, results parsed, initial baseline + triage report
  (the ranked our-bug list is this lane's headline deliverable).
- **M4** — `vibeos-cli conformance` verb + ratchet baselines committed.
- **M5** — widen: apitests (ntdll first — pure syscall surface), then winetests via
  rosautotest. Each is the same shape: build-fix → run → baseline.

## Ground rules (binding)

- AGENTS.md applies in full: worktree path discipline ($W-prefix every path), no push
  without a gate, never weaken smoke, batch tool calls, fail loud.
- **Never weaken or skip a test to make it pass** — baseline it as failing instead.
  Fixing the OS is in-scope for this lane when the fix is small/obvious; for deep
  subsystem bugs, file them in the triage report (M3) rather than rabbit-holing.
- Keep `ENABLE_ROSTESTS` OFF for default builds until the corpus builds clean on all
  arches AND the user opts in — the gate's build time matters. The conformance verb can
  flip it per-build (cached cmake option) or we add a `--rostests` build flag.
- Fixes to *product* code (ntoskrnl/win32k/…) discovered here follow the normal rules:
  Rust stays Rust, smoke ×3 before landing.

## Progress log

- 2026-07-10: Lane opened. Survey done (above). amd64 kmtests cmake exclusion removed.
  build-amd64 provisioned.
- 2026-07-10: **M1 amd64 GREEN.** `ENABLE_ROSTESTS=ON` configures clean; `kmtest_.exe`
  (PE32+ console) + `kmtest_drv.sys` (PE32+ native, 3.8MB, all 137 test files compiled)
  link on llvm-mingw with exactly ONE source fix: `ntos_fsrtl/FsRtlLegal.c` hardcoded the
  GCC pointer-to-pointer import deref `(*FsRtlLegalAnsiCharacterArray)[i]`; switched to
  the SDK's `LEGAL_ANSI_CHARACTER_ARRAY` portability macro (clang uses direct dllimport —
  see `sdk/include/xdk/fsrtlfuncs.h`). Assertion semantics unchanged. NOTE: the user-mode
  exe's target is `kmtest` but its OUTPUT_NAME is `kmtest_` → artifact is `kmtest_.exe`
  (upstream naming, kept). i386/arm64 next.
- 2026-07-10: **Native SEH opted in for kmtests on amd64/arm64** (user call: the FEX-lane
  SEH runtime is real, don't baseline DUMMY_PSEH crashes we can avoid). pseh2.h's
  `_USE_NATIVE_SEH` branch outranks the global `_USE_DUMMY_PSEH`, so a per-target
  `target_compile_definitions` gives kmtests real `__try/__except` (clang lowers it on
  amd64/arm64; `-fms-extensions` already global). Verified `.pdata` + `__C_specific_handler`
  in kmtest_drv.sys. i386 stays dummy (clang's 32-bit SEH lowering is MSVC-env-only —
  dummy-pseh-verdict.md §C) → expect a per-arch crash/fail delta there; that's the baseline
  working as intended. **New toolchain gotcha found + recorded in dummy-pseh-verdict.md §C:**
  clang won't inline callees inside `__try`, and mingw C `__forceinline` is gnu_inline
  (no body ever emitted) → FORCEINLINE SDK helpers called inside `__try` = undefined
  symbols. Shim TU: `kmtests/kmt_native_seh_shims.c` (out-of-line defs, no SDK includes).
  **Expectation setting (from §C):** native-SEH coverage is calls-only on mingw — raise-based
  tests should catch correctly; direct-deref-AV tests inside `__try` may still crash
  (no -fasync-exceptions) → those baseline as CRASH even on amd64/arm64.
- 2026-07-10: **M1 i386 GREEN** — kmtest_all builds with zero errors (dummy PSEH).
  arm64 provisioning + build in flight.
- 2026-07-10: **M1 arm64 GREEN — M1 COMPLETE ×3.** Greenfield arch cost 3 test-file fixes
  + 1 kernel export: (a) `kmtest_drv/structs.c` — 64-bit IO_STACK_LOCATION layout guard
  extended to `_ARM64_` (C_ASSERTs self-verify the shared amd64 layout); (b)
  `ntos_ex/ExSequencedList.c` — upstream's "SLIST_HEADER differs on x64" whole-test skip
  extended to arm64 (same 16-byte header); (c) `ntos_ke/KeIrql.c` — arm64 mirrors the
  amd64 branch (CLOCK1/2→CLOCK_LEVEL, KeRaiseIrql-is-a-macro, no stdcall-export block)
  and CMCI_LEVEL drops from the must-work IRQL array (x86 machine-check concept, no such
  IRQL on ARM64); (d) `ntoskrnl.spec` — `__C_specific_handler` export arch list was
  `x86_64,arm`; added `arm64` (the rtl-except arm64 impl was already in the kernel,
  just unexported — kmtest_drv.sys is its first external consumer). Verified: arm64
  ntoskrnl.exe exports it at a real address. Kernel-touching change → arm64 smoke owed
  before landing.
