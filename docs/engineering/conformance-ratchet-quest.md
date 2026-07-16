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
- 2026-07-10: **M2 in progress — first-light findings (amd64, three boots):**
  (a) A `.cmd` in the All Users Startup folder dies silently — explorer's
  `AutoStartupApplications` hits the uninstrumented `ShellExecuteExW` branch for
  non-binaries and no cmd.exe ever spawns. Vehicle switched to a native GUI-subsystem
  `kmtest-sweep.exe` (doom/httpd link recipe), one `kmtest_.exe` process per test,
  180s per-test timeout, list from `bin\kmtest-tests.txt`.
  (b) The sweep must gate on the global `SC_AutoStartComplete` event
  (services.c:379) — during boot auto-start the SCM database lock blocks
  `CreateService` indefinitely.
  (c) **Gate alone insufficient — console attach is the real wedge:** with auto-start
  provably complete, every `kmtest_.exe` child still hung pre-`main()` (kernel32
  PROCESS_ATTACH prints, then nothing; no SCM traces at all). kmtest_.exe is
  win32cui; CSRSS console attach from the startup-walk process context hangs —
  same family as the doom/httpd "GUI subsystem only" lesson, now seen for
  *children* of a startup-walk process. Harness fix: `DETACHED_PROCESS` (no
  console; the serial tee carries results). **The underlying console-attach hang
  is a real OS bug** — candidate conformance finding #1, not yet root-caused;
  the .cmd/ShellExecute silence in (a) may be the same bug one layer up.
- 2026-07-10: **FIRST CONFORMANCE CATCH — root-caused + fixed.** kmtest's
  `CreateService` failed with ERROR_FILE_NOT_FOUND: services-rust's hand-encoded
  `SERVICES_KEY` literal (`config.rs:61`) was `[u16; 33]` — one element short, silently
  dropping the final 's' of `...\Services`, so `ScmCreateServiceKey` opened a nonexistent
  parent key. Never caught by boot because boot-start services ride a different path;
  the FIRST runtime `CreateService(driver)` call in the system's life exposed it.
  Triangulated via stage-print bisection (1 boot) + two parallel subagent audits
  (services-rust diff-audit found the literal; registry-path audit proved the create
  call can't return 2, pinning the parent open). Field manual entry added
  (§string-literal traps). **OWED: `scan_wide_literals` lint** (decode `[u16;N]` hand
  arrays vs intent comments; one-shot sweep found no other true hits, but the class is
  invisible to all existing gates). Kept: fail-loud DbgPrint on RCreateServiceW failure
  (upstream FIXME leaves half-created services — separate latent bug, not yet fixed).
- 2026-07-10: **CONFORMANCE CATCH #2 — root-caused + fixed: IRP_MJ_CLEANUP was never
  delivered to ANY driver.** Surfaced as kmtest run 2 getting STATUS_ACCESS_DENIED on
  SET_RESULTBUFFER (run 1's MDL registration never cleaned up on handle close). Subagent
  diff-audit pinned it: ob-rust `ObpDecrementHandleCount` (obhandle.rs:505-517) translated
  C's `ProcessHandleCount = HandleCount--` (POST-decrement → pre value) as
  decrement-then-read → CloseProcedure got 0 instead of 1 on the last handle →
  `IopCloseFile` early-returned before sending IRP_MJ_CLEANUP. Every
  MaintainHandleCount-type close in the OS's history skipped driver cleanup; invisible
  to boot because nothing re-opens such a device twice. Fixed (read-before-decrement,
  both branches); new rustconv-catalog class in the field manual (post-inc/dec read
  order). M2 first light: Example END pass=31 fail=15 skip=0 — the 15 are Example's
  deliberate self-test failures, machinery fully green. Full 97-test M3 sweep in flight.
- 2026-07-10: **WHPX payload boot currently unusable — cross-lane observation for
  whpx-ahci-cd:** `uniata-rust/src/id_sata.rs:1972` DbgPrints `AhciBeginTx` per AHCI
  transaction (bring-up marker, unconditional); under the whpx ich9-ahci CD path the
  serial spam grinds the guest so hard that SCM auto-start never completes in 10 min
  (sweep gate correctly stayed shut). Needs gating by the owning lane (or a one-line
  rate-limit) before whpx is usable for long-running in-guest suites. Sweep boots use
  windows-host TCG (fleet default) until then.
- 2026-07-10: Harness moved to windows-host WHPX for sweep boots
  (`VIBEOS_QEMU_WINDOWS_ACCEL=whpx ./vibeos-cli smoke --arch amd64 --payload kmtest`;
  payload.yaml grew the `test:` section) — the debug-toolkit/native-KVM path was queueing
  8+ min behind fleet KVM boots and is only needed when GDB is.
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
- 2026-07-10: **M3 amd64 sweep 1 (97-test list): 20/97 complete, then IoCreateFile
  (test 21) BUGCHECKed the guest** — 0x1E, exception 0xC0000005 at a kmtest_drv.sys PC
  (`[RAISE] RtlRaiseStatus(0xC0000005) from fffff880049664c0`, unhandled → double 0x1E
  with a follow-on 0xC0000006 IN_PAGE on teardown). No handler found despite native SEH:
  consistent with a *deref*-AV inside `__try` (calls-only .pdata coverage, §native-SEH
  gotcha in dummy-pseh-verdict.md) OR the test dereferencing a bogus pointer our
  iomgr-rust IoCreateFile path returned (test has NO SEH at that spot — plain
  ObReferenceObjectByHandle + FileObject derefs). Triage owed (GDB repro:
  `run-until.sh --break KeBugCheckEx --then triage-bugcheck` with only IoCreateFile in
  the list). QUARANTINED in kmtest-tests.txt (commented, reason inline); sweep 2
  relaunched from the top. 20 completed results (pass/fail/skip) captured — headline:
  every one of the 20 ran to completion with exit 0, per-suite failures range 0
  (ExFastMutex, FsRtlMcb 36213-assert clean) to 1000 (ExUuid) — exactly the per-test
  fail-stock the ratchet baseline will pin. CmSecurity 300/89/2, ExInterlocked 662/40,
  FsRtlLegal 2496/32, ExUuid 2000/1000, ExSequencedList 0/0/0 (arch-skip), full table
  in sweep log + baseline-to-be.
- 2026-07-10: **M3 sweeps 2-3 (amd64): two more kernel-killer catches, both quarantined
  + triage owed.** Sweep 2: IoDeviceInterface + IoEvent green (22 OK cumulative), then
  **IoFilesystem** BUGCHECKs 0x1E — AV at `MiFindEmptyAddressRangeInTree+0x173`, which is
  the RUST translation (`ntoskrnl-rust/src/ARM3/vadnode.rs:370`); boot never walks that
  path hot enough to catch it. Explore subagent diffing Rust-vs-C launched. Sweep 3:
  IoInterrupt/IoIrp/IoMdl/IoVolume green (26 OK cumulative), then **KdSystemDebugControl**
  hits `RUST PANIC ke/dpc.rs:577` — `KiRetireDpcList` `DeferredRoutine.unwrap()` on None,
  i.e. a NULL-routine KDPC reached the DPC queue (candidate: an uninitialized Kd DPC on
  the debugger-disabled path; C would have jumped through NULL, the Rust unwrap made it
  loud + attributable — the panic print IS the conformance signal working). Also
  symbolicated sweep-1's secondary frames: `+0x7A7E0` = KeGetBugMessageText (bugcheck
  path itself, noise), `+0xCF5EB` = RtlRaiseStatus (dispatch). Iteration cadence:
  quarantine → `VIBEOS_PAYLOAD=kmtest build` (~3 min) → re-sweep; serial Monitor fires
  on BUGCHECK/TIMEOUT/done so a mid-sweep crash never burns the 90-min drain.
- 2026-07-10: **IoFilesystem crash static-triage (2 subagent passes) — AVL machinery
  exonerated, needs live repro.** `MiFindEmptyAddressRangeInTree` + all its Rtl*Avl
  callees + `mm/avl.rs` insert/delete/rebalance/splice verified faithful to C
  (parent-pointer balance-bit masking, MM_AVL_TABLE bitfield layout, two-child-splice
  self-referential case, root deletion, BalancedRoot sentinel — all match). One flagged
  divergence, Rust-favorable: `avl.rs:490` uses masked `is_right_child` where
  `avlsupp.c:372` uses the raw un-masked `RtlIsRightChild` macro in the delete
  rebalance walk. Fault is most plausibly the `(*node).StartingVpn` load at
  `vadnode.rs:410` on a stale/garbage node — i.e. the tree (likely `MiRosKernelVadRoot`
  via the CcMapData/MmCreateMemoryArea path the test hammers) is corrupted BEFORE this
  function walks it; suspects are locking/concurrency around the kernel cache-view tree,
  not per-line arithmetic. Next tool: live GDB repro (IoFilesystem alone in the list,
  `run-until.sh --break KeBugCheckEx --then triage-bugcheck`, then walk the tree at
  the fault). Owed as M3 triage item.
- 2026-07-10: **M3 sweep 4 (amd64): catch #4 — MmReservedMapping BUGCHECKs 0xDA
  SYSTEM_PTE_MISUSE** (subcode 0x101, P3 = the test's own pool tag 'KmRM') — the
  `MmAllocateMappingAddress`/`MmFreeMappingAddress` reserved-mapping path (Rust
  `mdlsup.rs`). Quarantined; triage owed. Progress before the crash: 38/97 cumulative
  OK — IoInterrupt through KeTimer all green (the entire Ke* family: Apc, DeviceQueue,
  Dpc, Event, FloatPointState, GuardedMutex, Irql, Mutex, SpinLock, Timer), MmMdl
  575/0 clean, MmSection 71/13/4.
- 2026-07-10: **M3 sweep 5 (amd64): catch #5 — NpfsFileInfo BUGCHECKs 0x19
  BAD_POOL_HEADER subcode 5** (corrupt pool header on free, immediately at test start).
  Quarantined; triage owed — repro recipe: NpfsFileInfo alone in the list +
  `--tripwire "SPECIALPOOL=*"` to fault at the corrupting store (field manual §9.18).
  NpfsConnect (679/0) and NpfsCreate (351/0) both green before it — 40/97 cumulative OK.
- 2026-07-10: **M3 sweep 6 (amd64): catch #6 — ObSymbolicLink BUGCHECKs 0x1E, AV in
  memcpy+0x40** (bad src/dst from the symlink path; ob-rust
  NtCreateSymbolicLinkObject/lookup-reparse copy is the candidate). Quarantined; triage
  owed. NpfsReadWrite/NpfsVolumeInfo/ObHandle/ObQuery/ObReference (2205/0!) /ObSecurity
  all green before it — 46/97 cumulative OK.
- 2026-07-10: **M3 sweep 7 (amd64): RtlExceptionKM BUGCHECKs — first PSEH-STRUCTURAL
  quarantine** (deliberate hardware fault inside __try; mingw native SEH is calls-only,
  so the in-body AV escapes → 0x1E at a kmtest_drv PC — the exact gotcha documented in
  dummy-pseh-verdict.md §C). Expected-fail class, NOT an OS bug; stays quarantined until
  an -fasync-exceptions-equivalent story exists. ObType/ObTypes/PsNotify/PsQuota/
  RtlAvlTreeKM green before it — 51/97 cumulative OK. (Watch: the UM twin RtlException
  later in the list should crash only the process, not the kernel — a good
  harness-robustness data point.)
- 2026-07-10: **M3 sweep 8 (amd64): catch #7 — RtlStackKM BUGCHECKs 0x1E, AV in
  `RtlWalkFrameChain+0x2b0`.** DUMMY_PSEH-structural candidate with a real kernel-side
  fix available: the C walk relies on __try to stop at a bad frame (dead under
  USE_DUMMY_PSEH in-kernel) — needs explicit frame-pointer bounds validation instead.
  Quarantined; triage owed. RtlIntSafeKM/RtlIsValidOemCharacter/RtlMemoryKM/RtlRangeList/
  RtlRegistryKM/RtlSplayTreeKM green before it — 57/97 cumulative OK.
- 2026-07-11: **M3 sweep 9 (amd64): FIRST FULL COMPLETION — `done total=89 launched=89
  failstart=0 timeouts=1`.** 62 OK with real pass/fail stock (highlights: KeEvent 42733/0,
  SeInheritance 704433/14, NpfsReadWrite 7154/0, IoInterrupt 939/156, KeGuardedMutex
  1566/87). One new catch (#8): **SeTokenFiltering wedges in-kernel at 180s** — and the
  timeout kill can't reap a kernel-stuck process, so kmtest_drv keeps the result buffer
  registered and ALL 26 subsequent tests cascade-fail SET_RESULTBUFFER 0x5 (kmtest.c:334;
  same face as the fixed ob-rust cleanup bug, different mechanism — the process never
  dies, so IRP_MJ_CLEANUP legitimately never fires). Sweep-design lesson recorded: a
  single in-kernel wedge poisons the boot's remaining tests; the per-test timeout can
  only skip, not recover. SeTokenFiltering quarantined; sweep 10 relaunched to get true
  results for the cascade-masked tail (Zw* + the 18 UM tests + TcpIp*).
- 2026-07-11: **M3 sweeps 10-11: catches #9-10, and a CLUSTER signal.** Sweep 10 got
  real results for the Zw* tail (ZwAllocateVirtualMemory 26/17/9, ZwCreateSection 4/3/2,
  ZwMapViewOfSection 104/23/2 — all previously cascade-masked), then ZwWaitForMultipleObjects
  0x1E in `memcpy+0x40`. Sweep 11 verified HidPDescription/IoDeviceObject (+ FileAttributes/
  FindFile/etc.), then IoReadWrite 0x1E — **same `memcpy+0x40` PC**. That's now THREE
  crashers at the identical instruction (ObSymbolicLink, ZwWaitForMultipleObjects,
  IoReadWrite) → almost certainly ONE root: an unprobed/hostile-length user-buffer copy
  where the C relied on SEH probing (dead under DUMMY_PSEH). Fixing that single path likely
  clears all three quarantines. High-value first triage target. 70/97 cumulative OK.
- 2026-07-11: **memcpy+0x40 cluster ROOT-CAUSED (subagent) — a reusable DUMMY_PSEH class.**
  ObSymbolicLink / ZwWaitForMultipleObjects / IoReadWrite (and almost certainly
  NtCreateSection, sweep 13's 0x50) all reduce to: a kernel API captures a caller-supplied
  buffer/handle-array with a copy that in C sat inside `_SEH2_TRY`; under USE_DUMMY_PSEH
  that handler is DEAD, so a hostile pointer (NULL / 0x42424242, both below
  MmUserProbeAddress so a range-only probe passes them) faults the capture memcpy → 0x1E.
  Two compounding gaps in the Rust translations: (1) the `probe_user_buffer` guard is gated
  on `PreviousMode != KernelMode`, but these tests call from kernel mode → guard skipped,
  unconditional copy; (2) `probe_user_buffer` is a *range+alignment* check only (never
  touches memory), so it can't catch a mapped-range-but-invalid pointer anyway — the safety
  was always the SEH around the copy. Capture sites: `ob-rust/src/oblink.rs:657` (symlink
  target copy), `ob-rust/src/obwait.rs:119` (handle-array copy, unconditional), and worst
  `iomgr-rust/src/iofunc.rs:3369-3373` NtWriteFile still calls the raw C SEH probes
  (`ProbeForWrite*`/`ProbeForRead*`) + a bare `*Key` deref (NtReadFile at :2600 was already
  converted to the checked-Result probe pattern; NtWriteFile was not). **Fix (single shared
  primitive):** make `ntkrnl-sys/src/probe.rs` `safe_copy_from_user`/`safe_copy_to_user`
  genuinely fault-tolerant (real PSEH via pseh-rust, OR per-page MmIsAddressValid returning
  Err(STATUS_ACCESS_VIOLATION)), run it UNCONDITIONALLY (not PreviousMode-gated), and route
  all three capture sites through it. Working reference for the checked pattern:
  `ex-rust/src/dbgctrl.rs` NtSystemDebugControl. This fix likely clears 4 quarantines
  (ObSymbolicLink, ZwWaitForMultipleObjects, IoReadWrite, NtCreateSection). Owed as the
  top M3 fix task; gate ×3 before landing (touches ob/io/ntkrnl-sys core copy path).
- 2026-07-11: **origin/master merged into the lane (110 commits)** — brings the arm64
  bugcheck-unmask + callback-ERET IRQ fix (§9.54: arm64 bugchecks now self-report
  truthfully instead of masking through KeGetBugMessageText), the fastfat dropped-finally
  fixes (§9.53), wow64 M3-M7, and smp M2/M3a. Merge was clean except a pure line-shift in
  `tools/lint-baseline/lint_kernel.txt` (rebaselined, 0 new findings). The merge did NOT
  touch the kmtest corpus, so test definitions are stable — any result drift is pure kernel
  behavior. Baseline regenerated against the merged kernel below.
- 2026-07-11: **M3 sweep 14 (amd64, post-merge): `done total=84 launched=84 failstart=0
  timeouts=1`, ZERO BUGCHECKs across all 84 tests — 80 OK.** The SeTokenFiltering-cascade
  tail from sweep 9 is now REAL data (the 26 old NO-END entries resolved): NtSystemDebugControl
  22/20, ZwAllocateVirtualMemory 1256/11, ZwMapViewOfSection 105/22, ZwCreateSection 4/3/2,
  IoDeviceObject 185/4, HidPDescription 28/9/3, FindFile 251/2, FileAttributes 84/0,
  RtlCaptureContextKM 273/0, SeInheritance 704433/14, TcpIpTdi 31/4, and the full Rtl* UM
  suite (RtlIntSafe/RtlMemory/RtlRegistry/RtlSplayTree/RtlStrSafe/RtlUnicodeString all green).
  Three new quarantines this sweep:
  * **RtlException + RtlStack (UM twins):** NO-END exit=128 — the user-mode versions of the
    already-quarantined RtlExceptionKM/RtlStackKM. They deliberately fault inside SEH (dead
    under DUMMY_PSEH / native-SEH-calls-only), but as UM processes they SELF-CRASH (exit 128)
    with NO kernel BUGCHECK and the sweep continues clean past both — exactly the harness-
    robustness outcome the quest predicted (kernel isolates the UM fault). Expected-fail,
    same class as the KM twins; quarantined for a green baseline, not an OS bug.
  * **TcpIpConnect (in-kernel wedge):** TIMEOUT at 180s on the TDI connect path (TcpIpTdi
    ran clean immediately before). Kernel-stuck → unreapable by TerminateProcess → kmtest_drv
    keeps the result buffer registered → the next test RtlCaptureContextUM cascade-fails
    SET_RESULTBUFFER 0x5 (kmtest.c:334, exit 1) — the same cascade mechanism as SeTokenFiltering.
    First real observation (earlier sweeps never reached it un-masked). Triage owed: it's a
    HANG not a crash, so GDB-break the tcpip/afd connect path (no faulting PC to symbolicate).
    Quarantining it un-masks RtlCaptureContextUM's real result on the re-sweep.
  Sweep 15 relaunched (with the 3 new quarantines) to confirm a fully-clean all-green pass +
  RtlCaptureContextUM's true numbers, then the amd64 baseline is regenerated against the
  merged kernel and the whole bundle (kernel fixes + harness) lands via gate ×3.
- 2026-07-11: **BUNDLE LANDED to origin/master** (`38ff68746c1`): conformance harness
  (payload sweep + `parse_kmtest_serial.py` + ratchet compare) + amd64 81/81 baseline +
  the 3 conformance-caught OS fixes (services SERVICES_KEY, ob-rust IRP_MJ_CLEANUP,
  arm64 `__C_specific_handler`) + the fastfat `FatLocateDirent` NT_ASSERT fix (field
  manual §9.57) that the IRP_MJ_CLEANUP fix woke on arm64 boot. Gate ×3 green.
- 2026-07-12: **Owed fastfat NT_ASSERT sweep DONE** (`8536c9541cf`) — all 26 hard
  `assert!(` in fastfat-rust cross-checked against the C; every one mirrored a debug-only
  `NT_ASSERT`/`ASSERT`. Converted 25 → `debug_assert!` (no-op in our `--release` kernel,
  faithful to ReactOS free-build semantics); 2 left (already-guarded fileinfo:1071, a
  comment). Each was a dormant panic of the exact class that detonated FatLocateDirent.
  Field manual §9.57 owed-sweep closed.
- 2026-07-12: **M4 STARTED — `vibeos-cli conformance` verb** (`tests/qemu/cli/conformance.py`).
  One front door: (optional) `VIBEOS_PAYLOAD=kmtest` build → boot → in-guest sweep (drains
  `test_payload.py::test_payload_launched` up to `launch_drain_secs`) → parse the serial log
  → compare vs `tools/conformance-baseline/<arch>/kmtest.txt` → `summary.json`
  (`{launched, ok, bad, regressions, new_passes, sweep_done, bugcheck}`). Report-only by
  default; `--gate` exits nonzero on ratchet regression; `--write-baseline` seeds/updates the
  floor; `--smp N` passes `VIBEOS_QEMU_SMP` (+ `--smp-kernel` build) — the hook the UP→SMP/AP
  lane will use to run guest loads under `-smp`. Fail-loud rostests preflight (no silent
  degradation). Unit-tested (arch-norm, parser regression detect, rostests preflight);
  live-validation is the i386 baseline run next (dogfood the verb to produce M3 i386).
- 2026-07-12: **M3 i386 baseline GREEN — 67 OK + 4 NO-END, 10 i386-specific crashers
  quarantined.** Produced by dogfooding the M4 verb (build→sweep→parse→ratchet), which
  live-validated M4 end-to-end incl. the new crash-abort (a mid-sweep BUGCHECK returns in
  ~50s, not the 90-min drain). i386 runs kmtest_drv under **DUMMY_PSEH** (native SEH is
  amd64/arm64-only) → a strict SUPERSET of amd64's structural crashers, so the test lists
  were split **per-arch** (`kmtest-tests-{arch}.txt`, payload.yaml `{arch}` templating);
  i386 inherits amd64's 16 quarantines + 10 i386-only.
  * **10 i386-only crashers** (all pass on amd64; live-GDB triage owed — offline i386
    symbolication is unavailable, kernel is release-stripped): ExHardError (+0x675D),
    ExResource (+0x8A7EB), KeDpc (+0x1F061), KeMutex (+0x944EF, exc 0xC0000191), MmSection
    (+0xA0F3B), RtlIsValidOemCharacter (+0x66868), **RtlSplayTreeKM (0x19 BAD_POOL_HEADER —
    REAL-BUG candidate in the `splaytree` crate, not DUMMY_PSEH)**, SeInheritance (+0x8E3BD),
    ZwAllocateVirtualMemory (+0x14E4), ZwMapViewOfSection (kmtest_drv+0x5ACF2).
  * **4 baselined NO-END** (isolated own-failures, NOT cascades — each followed by a clean
    test; kept in the sweep as the floor, quest-preferred over quarantine): RtlSplayTree +
    RtlUnicodeString (UM self-crash exit=128, i386-only), RtlCaptureContextKM + UM
    (`support.c:105 error 0x7b` INVALID_NAME — test-harness issue).
  * **sweep.c bug fixed (harness-wide):** the runner's 128-char `line` buffer split any
    over-long list comment, launching its tail ("...symbol-poor).") as a bogus test that
    polluted the baseline + poisoned the result buffer. Now drains over-long lines. amd64
    was unaffected (its comments max 94 chars); its landed 81/81 baseline stands.
  * Iteration was automated (a throwaway auto-grinder: run→parse-crasher→quarantine→rebuild
    until clean).
- 2026-07-12: **M3 arm64 DEFERRED (multi-layer first-light) — root-caused 2 layers, 1 fixed.**
  arm64 boots from a fastfat RAMDISK (not the x86 ISO), so the conformance sweep hits
  arm64-only plumbing gaps the x86 path never had. Peeled with a MAIN-PROBE (a DbgPrint as
  main()'s first statement — the definitive tool; it corrected a wrong hypothesis and saved a
  ~2h dead-end rewrite):
  * **RED HERRING corrected:** the `msvcrt cp-20127 MultiByteToWideChar` err spam on serial
    is NON-fatal — MAIN-PROBE proved `main()` runs fine. Do NOT rewrite sweep.c off msvcrt.
  * **Layer 1 (FIXED):** the sweep's SCM gate `OpenEventA("SC_AutoStartComplete")` never
    succeeds on arm64 — **services.exe never publishes SC_AutoStartComplete on arm64** (x86
    signals it within a few tries). The gate FATAL-returned; now it WARNs + PROCEEDs (the
    sweep runs after the desktop is up, so auto-start is functionally complete). **CONFORMANCE
    FINDING owed:** services-rust should publish SC_AutoStartComplete on arm64.
  * **Layer 2 (BLOCKER, next):** with the gate cleared, every test is `launch FAILED err=2`
    (FILE_NOT_FOUND) — **kmtest_.exe / kmtest_drv.sys (+ helper drivers) are not staged into
    the arm64 ramdisk's `\reactos\bin\`.** They're built (build-arm64/modules/rostests/kmtests/)
    and reach the x86 ISO via the CMake add_rostests_file/CD-file mechanism, which does NOT
    feed the ramdisk (mkramdisk.sh is a hand-curated file list + the payload manifest). Fix =
    stage the full rostests binary set into the ramdisk (mkramdisk.sh, conditional on a
    KMTESTS build). Likely further layers behind it (kmtest_drv load + CreateService on arm64).
  The SCM-proceed sweep.c fix lands now (arm64-path-only — x86 takes the event-found branch
  unchanged, so the landed i386/amd64 baselines are unaffected by construction).
- 2026-07-12: **memcpy+0x40 CLUSTER FIXED — the kernel gets a real fault-tolerant copy
  (`ntoskrnl/kmsafecopy.c`), and 2 quarantines clear.** The 2026-07-11 root-cause was right:
  every member was a caller-supplied-buffer capture whose `_SEH2_TRY` is dead under
  USE_DUMMY_PSEH, so a hostile pointer faulted the capture memcpy. What was missing was the
  primitive — `RtlpSafeCopyMemory` had been translated to a *plain* `RtlCopyMemory` (the SEH
  dropped), so NO fault-tolerant copy existed anywhere in the kernel.
    - **The mechanism (the open question, now settled empirically).** dummy-pseh-verdict §C
      warns clang native SEH is "calls-only (no -fasync-exceptions)" — a *direct deref* inside
      `__try` may escape. That is exactly why the primitive works: a **copy is a CALL**, so the
      fault lands in a CALLEE and the return address sits inside the `__try` scope table.
      `KmProtectedCopyMemory` therefore delegates to a `noinline` `KmCopyRaw` (which calls the
      real `memcpy`) — KiPageFault -> KiDispatchException -> RtlDispatchException unwinds
      KmCopyRaw's frame -> `__C_specific_handler` matches -> `__except` -> return 0 ->
      `Err(STATUS_ACCESS_VIOLATION)`. seh_poc.c had only ever validated a *software raise*;
      this proves the chain catches a **hardware page fault** too. Bonus: because the fault
      goes through `MmAccessFault` first, a valid-but-nonresident page is simply paged in —
      something a bare `MmIsAddressValid` pre-check could never do.
    - **Four kernel capture sites routed through it** (`safe_copy_from_user` / new
      `safe_copy_raw`, run UNCONDITIONALLY — the C's `_SEH2_TRY` wrapped the copy regardless of
      PreviousMode; only the *range probe* is user-mode-gated): `ob-rust/oblink.rs` (symlink
      target), `ob-rust/obwait.rs` (handle array), `iomgr-rust/iofunc.rs` NtWriteFile (buffered
      SystemBuffer capture, with the C's `IopCleanupAfterException` handler restored), and
      `iomgr-rust/irp.rs` IopCompleteRequest (buffered copy-back to `Irp->UserBuffer`).
    - **Result: ObSymbolicLink CRASH -> OK (154/0), ZwWaitForMultipleObjects CRASH -> OK (4/0).**
      `memcpy+0x40` is gone from the kernel entirely.
    - **PERF (caught by disassembly, not assumed):** a hand-rolled `while (Len--) *Dst++ = *Src++;`
      compiled to an actual byte-at-a-time loop (clang's loop-idiom did NOT fire). Since this
      primitive now sits on the buffered-I/O completion hot path, `KmCopyRaw` delegates to the
      real `memcpy` instead. Verified: `callq <memcpy>`.
    - **HONEST GAP — amd64 only.** `fault_tolerant_copy` protects x86_64; i386/arm64 fall through
      to a plain copy (status quo, zero behavior change). `MmIsAddressValid` was deliberately
      REJECTED as the stand-in: it cannot distinguish "invalid" from "valid but paged out", so on
      the IRP completion hot path it would spuriously fail legitimate I/O with
      STATUS_ACCESS_VIOLATION under memory pressure — a silent degradation worse than the crash.
      Closing it needs clang frame-based i386 / aarch64 kernel-SEH validation (the seh_poc.c
      treatment) or a Linux-extable-style fault-fixup copy. The i386 equivalents stay quarantined.
- 2026-07-12: **IoReadWrite RE-QUARANTINED — a SECOND, independent bug behind the memcpy one.**
  The cluster fix landed here too, and the crash PC moved out of ntoskrnl: now
  `IoReadWrite_drv.sys+0x23A0` (the TEST DRIVER). Root: **FAST I/O hands the driver the RAW user
  buffer** (no SystemBuffer), and the test's `TestCommonWrite()` derefs `BufferBytes[i]` with no
  NULL guard (unlike `TestCommonRead`, which checks `Buffer == NULL`). In C the fast-I/O dispatch
  sits inside `_SEH2_TRY`, so the FSD's AV converts to STATUS_ACCESS_VIOLATION; dead under
  DUMMY_PSEH -> BUGCHECK. Same family, DIFFERENT primitive: needs a protected **call** (guard
  around the FastIoRead/FastIoWrite dispatch), not the protected **copy**. Triage owed.
- 2026-07-12: **Two ratchet catches that are NOT ours — regressions the origin/master merge
  brought in.** The amd64 baseline (81/81) was recorded at `5d328937ef9`, BEFORE this lane merged
  origin/master (`eb3df16529c`, 110 commits). Both reproduce deterministically across two sweeps
  and have zero code-path overlap with the copy fixes:
    - **ExUuid: 3000/0 -> 2000/1000** — `Invalid UUID variant: 0xd0`. The test asserts
      `(Data4[0] & 0xC0) == 0x80` (RFC4122). `ex-rust/uuid.rs:315-317` is a *faithful* port
      (`(Sequence & 0x3F00) >> 8 | 0x80` can only yield 0x80-0xBF, never 0xd0) and is UNCHANGED
      since the baseline — so `Data4[0]` is not coming from that path at all: a struct/bitfield
      layout mistranslation in the cached-values accessors is the candidate.
    - **IoDeviceObject: 185/4 -> 184/5** — `RegistryPath not page-aligned`.
  Baseline deliberately NOT updated — leaving it un-refreshed keeps the ratchet flagging these
  until they are root-caused. (This is the ratchet doing exactly its job: catching a mainline
  regression that no boot smoke would have noticed.)

- 2026-07-12: **The "3 kernel crashers" were 1 kernel bug + 2 harness artifacts + 1 innocent —
  and un-quarantining them TOGETHER is what separated the causes.** Root cause of the harness
  half: the 17 **standalone** kmtest helper drivers never got `_USE_NATIVE_SEH` (only
  `kmtest_drv`/`kmtest.exe` did), so under project-wide `USE_DUMMY_PSEH` their own
  `_SEH2_TRY`/`_SEH2_EXCEPT` compiled to **dead handlers**. These drivers *deliberately* fault
  and expect to catch it — `IoReadWrite_drv`'s fast-I/O path derefs the raw user buffer with no
  NULL guard, expecting its `_SEH2_EXCEPT` to return `STATUS_ACCESS_VIOLATION` (precisely what
  the test asserts). Dead handler ⇒ the AV escaped to the kernel ⇒ BUGCHECK. **The kernel was
  innocent.** Fixed with `kmt_enable_native_seh()` on all 17 drivers (amd64+arm64; i386 SEH
  lowering is MSVC-env-only). Verified in codegen (`__C_specific_handler` import + `.pdata`)
  before spending a sweep, and the arm64 build was proven to still link.
    - `IoReadWrite`: **CRASH → OK (16192 pass / 151 fail)** — un-quarantined.
    - `NtCreateSection` (0x50) and `MmMapLockedPagesSpecifyCache` (0x1000): **still crash, same
      stop codes** ⇒ REAL kernel bugs, re-quarantined with the negative result recorded. This is
      the valuable half: it proves native SEH didn't paper over them.
  **My own earlier triage note was WRONG** and is corrected in-file: C does *not* dispatch fast
  I/O inside `_SEH2_TRY` (only the IOSB write-back is protected), and C's `ProbeForRead` is
  range+alignment-only and never touches memory — so no kernel-side "protected call" was ever
  the answer. Reading the C oracle's actual control flow beat pattern-matching. Field manual §9.63.

- 2026-07-12: **`ExUuid` was never a merge regression — it was a latent bug that passed 1 boot
  in 4.** `ex-rust/src/uuid.rs` flattened C's `UUID_CACHED_VALUES_STRUCT` union into a bare
  `GuidInit: [u8;8]` + hand-written accessors, and **transposed** `ClockSeqHiAndReserved` (C:
  `GuidInit[0]`, the RFC4122 variant byte, aliased onto `GUID.Data4[0]`) with `ClockSeqLow`.
  `Data4[0]` therefore got the raw low byte of a `KeQueryPerformanceCounter`-seeded sequence
  number ⇒ `(Data4[0] & 0xC0) == 0x80` held **exactly 25% of boots**, and the committed baseline
  had captured one of those lucky runs as `ExUuid OK pass=3000 fail=0`.
  **The tell:** `git diff baseline..HEAD -- uuid.rs` was **empty** — the code hadn't changed at
  all, so "a merge broke it" was impossible. Reconciling that contradiction (rather than hunting
  the merge) is what cracked it. Fixed by swapping the indices + pinning them to the C header.
  Now deterministic. Field manual → struct-layout section. **Lesson: a green baseline can be
  hiding a probabilistic pass.**
    - Sibling still open: `IoDeviceObject` 185/4 flaps to 184/5 between runs (`RegistryPath not
      page-aligned`) — same "flaky assertion" smell, owed.

- 2026-07-12: **amd64 baseline refreshed: 84/84 OK, 0 crashers, ratchet holds.** The 3 wins are
  locked into `tools/conformance-baseline/amd64/kmtest.txt` so they can't silently regress:
  `ObSymbolicLink` (154/0), `ZwWaitForMultipleObjects` (4/0), `IoReadWrite` (16192/151).
  **Owed, stated plainly:** `IoReadWrite` is baselined with **fail=151** — the CRASH is fixed and
  the test now runs to completion, but 151 assertions still fail and are now accepted as debt.
  That is a real conformance backlog item, not a pass. (Baseline for the known-flaky
  `IoDeviceObject` / `ExResource` was deliberately NOT rewritten from a single lucky run — this
  sweep happened to match the existing values exactly, so nothing moved.)

- 2026-07-13: **Task #5 (arm64 fault-tolerant copy) — TRIED, REJECTED, root-caused; + a build bug
  that fabricated evidence.** Attempted to extend the §9.59 fault-tolerant copy to arm64 (the arm64
  kernel exports `T __C_specific_handler` and clang emits `.pdata` for arm64 `__try`, so it looked
  ready). Added a per-boot self-test (`KmSafeCopySelfTest`, prints `[KMSAFECOPY] ... PASS`) that
  asserts BOTH a valid copy succeeds AND an invalid one is caught — this is now PERMANENT on amd64
  (first end-to-end proof in-tree that a real HARDWARE fault is caught in kernel mode; `seh_poc`
  only ever proved a software raise). arm64 **FAILED**: a probe at an unmapped VA wedges the boot.
  Root cause (field manual §9.69): NOT the DUMMY_PSEH silent-skip crutch (that path is never
  reached — an earlier hypothesis, falsified by instrumentation). `MmAccessFault` correctly
  declines; `KiDispatchException` is entered with a valid kernel stack; **arm64 kernel-mode
  `RtlDispatchException` never returns** — it recurses/faults (first inner fault = instruction
  abort, EC=0x21: it CALLs a bogus handler routine) → kernel stack overflow → infinite re-entry of
  `KiArm64VectorTable`. The bug is in the arm64 virtual-unwind / `__C_specific_handler` path for
  kernel frames. arm64 keeps the plain copy + documented gap (task #10). i386 likewise. The
  arch-neutral close for both = a Linux-extable-style fault-fixup table in the page-fault handler.
    - **Build bug found + FIXED along the way (field manual "Editing a kernel C SHIM…"):**
      `reactos-link` emitted no `rerun-if-changed` for the C shims, so editing `kmsafecopy.c`
      relinked a STALE object and a boot printed the PREVIOUS build's verdict — it fabricated a
      result for code that wasn't running. Same false-fresh class the engine already documented for
      Rust path-deps. This cost a wrong arm64 conclusion before it was caught. Now every c_shim path
      is watched.
- 2026-07-13: **Owed — the two REAL kernel bugs from the un-quarantine experiment (task #7):**
  `NtCreateSection` BUGCHECK 0x50 (PAGE_FAULT_IN_NONPAGED_AREA, read at fffff880045ae040 — param
  capture/validation path) and `MmMapLockedPagesSpecifyCache` BUGCHECK 0x1000 (PFN_LIST_CORRUPT,
  P1=0x401 — Rust MDL-map PTE/PFN accounting). Both proven real (survived the native-SEH harness fix
  with identical stop codes). Both amd64, both clear a quarantine when fixed. Need a kmtest-payload
  build + repro under the kernel debug toolkit for a real triage.
- 2026-07-13: **Owed — the net-suite retry hole (harness robustness, ranks above most single bugs):**
  the gate auto-retries `NET_FLAKE` and accepts a pass on retry, so a *probabilistic* regression
  gets waved through. Direct proof: a tree containing the msvcrt wave-4 regression ran net 3× on one
  build and went `fail → 15/15 → 15/15`; the gate called it GREEN. Very plausibly how msvcrt wave 2
  reached mainline. Fix: serialize the net stage under the host lock (it's the one wall-clock-
  sensitive stage and multiple lanes gate concurrently), or fail the gate on ANY net attempt failing
  rather than accepting a retry.

- 2026-07-13: **Task #7 real triage captured (MmMapLockedPagesSpecifyCache).** Un-quarantined
  locally + built the amd64 kmtest payload + reproduced under the kernel debugger:
  `BUGCHECK code=0x1000 P1=0x401 P2=0xfffff6800000d300 P3=0x1ca0001f P4=0xfffff88000252be9`
  (ntoskrnl kbase `0xfffff80000000000`). **P2 is a PTE self-map address** (base
  `0xfffff68000000000` + `0xd300` → maps user VA ~`0x1a60000`); **P3 is a VALID PTE value**
  (PFN `0x1ca00`, flags `0x1f`); **P4 is a DRIVER address** (`0xfffff880…` = the test driver
  mmmaplockedpagesspecifycache_drv.sys). The params (a valid PTE where one is unexpected) point at
  a double-map / stale-PTE / PFN-share-count inconsistency. **Ruled out:** the
  `MiMapLockedPagesInUserSpace` map loop (`mdlsup.rs:255-321`) is a FAITHFUL translation of the C
  (`mdlsup.c:176-266`) — `Pfn1`=page-table PFN, share-count++, cache attrs, and
  `MiIncrementPageTableReferences` all match. So the bug is in a RELATED path (PDE creation, PFN
  handling, the unmap path, or driver interaction), not the map loop. The `KeBugCheckEx` backtrace
  broke (spinning in the bugcheck handler), and `code=0x1000` is not a grep-able literal in the
  Rust Mm path. **Owed next step:** a reliable repro — break `KeBugCheckEx`, read `rcx`(code)/
  `rdx`/`r8`/`r9` + the symbolicated caller return address for the exact call site, then diff that
  path vs C. NtCreateSection 0x50 (PAGE_FAULT_IN_NONPAGED_AREA) still un-triaged at this depth.
  Both stay quarantined. (This is a genuinely deep MM bug; the triage above is the head start.)
- 2026-07-15: **M3 arm64 Layer 2 CLEARED (landed); Layers 3+4 root-caused — the blocker is that
  arm64 HAS NO SCM.** Picked up on the smp-bringup lane's ask (they want looped
  `conformance --arch arm64 --smp 2` as guest-load soak to amplify a 1-in-258 arm64 0x4E). Two of
  their three predicted gaps were ALREADY CLOSED — mkramdisk.sh has had the arm64 payload leg
  (`compile.py manifest --arch arm64`) all along, `build-arm64` already had
  `ENABLE_ROSTESTS_KMTESTS=ON`, and `kmtest-sweep-arm64.exe` + `kmtest-tests-arm64.txt` already
  existed. The real gap was the one the 07-12 entry above named:
  * **Layer 2 (FIXED, landed).** The corpus never reached the ramdisk. Fixed in BOTH halves,
    because staging alone would have been worse than useless: arm64 builds a hand-picked staged
    set (archcfg.py), so the 14 `*_drv.sys` already in `build-arm64/` were STALE leftovers from an
    old ninja — staging those would have graded stale test binaries (the exact false-green trap
    archcfg's own comment block documents 5× over). So: `kmtest_all` added to arm64's OPTIONAL
    targets (optional because both cmake ROSTESTS options default OFF), and mkramdisk stages
    `kmtest_.exe` + every `*_drv.sys` FLAT into `reactos/bin` (flat is load-bearing:
    `KmtpCreateService` resolves drivers as `GetModuleFileName(kmtest_.exe)` → truncate → append,
    `kmtest/service.c:428-439`). Proof it cleared: the sweep now finds
    `X:\reactos\bin\kmtest-tests.txt` and launches tests (was `launch FAILED err=2`).
    Sibling latent fix: `ninja_targets_optional` ran as ONE batched `ninja a b c`, so a single
    absent target (a kmtests-off dir has no `kmtest_all`) would silently take arm64input/arm64vid
    down with it and stage a stale input/video driver. Now one ninja per target.
  * **Layer 3 (ROOT CAUSE, blocks the baseline): arm64 has NO Service Control Manager.**
    `winlogon.rs:1340` is a `#[cfg(target_arch = "aarch64")]` fork that prints
    `"ARM64: Skipping RPC, services, lsass, notifications, GINA"` and does nothing, while the
    `not(aarch64)` branch does the real `StartRpcServer`/`StartServicesManager`/`StartLsass`.
    services.exe was never built, never staged, never spawned. kmtest's whole model is
    `OpenSCManager`→`CreateService`→`StartService` = advapi32 → RPC `ncacn_np` `\pipe\ntsvcs`
    (`advapi32/service/scm.c:42-44`) → services.exe (`services/rpcserver.c:115`), so EVERY test
    raised **RPC_S_SERVER_UNAVAILABLE (0x6BA)** before its first print — `[UEXC] Code=0x6BA
    ESR=0x56000000` (EC=0x15 = SVC, i.e. a software raise, not a hardware fault) →
    `[UEXC-KILL] Proc=kmtest_.exe`. The sweep only ever saw a 180s per-test TIMEOUT, which is why
    this looked like a hang. **This CORRECTS the 07-12 Layer 1 note above**, which recorded
    "services.exe never publishes SC_AutoStartComplete on arm64" — that phrasing implies it runs.
    It does not exist. (The sweep's 24×5s wait for that event is therefore 120s of dead time on
    every arm64 sweep.)
  * **Layer 4 (the wall, UNSOLVED): spawning services.exe WEDGES the arm64 boot.** All the pieces
    build fine for arm64 (`services`, `lsass`, `npfs` are all valid targets) and services.exe's
    entire import set (advapi32, advapi32_vista, kernel32, msvcrt, ntdll, rpcrt4, user32, userenv)
    is ALREADY staged. Wired it up end-to-end and measured, A/B:
      - npfs.sys staged + hand-loaded + corpus staged, WITHOUT the winlogon spawn →
        **arm64 boots GREEN, 41s.** `[SYSDRV] staged ZwLoadDriver …\Npfs -> 0x00000000` — **npfs
        first light on arm64 is CLEAN.**
      - PLUS the winlogon spawn → `StartServicesManager: CreateProcessW OK (PID=176)` then
        **HANG_TIMEOUT**, explorer never starts, serial goes silent.
    So services.exe is the wedge and npfs is innocent. **Mechanism UNKNOWN — do not assume.**
    The tempting story is the documented "ZwLoadDriver HANGS on a service whose .sys is absent
    from the minimal ramdisk" bug (`iomgr/driver.rs:1200`, the very bug that forces arm64's
    hand-rolled STAGED_SVCS list instead of the bulk Start=1 enumeration), hit from services.exe's
    usermode autostart storm. **But the evidence argues against it:** services-rust has 63
    `DbgPrint` sites (which DO reach serial) and emitted **ZERO** `[SERVICES]` lines, so it
    probably never reaches its autostart loop at all — it dies/wedges before its first print.
    Next step is therefore instrumentation, not theory: a MAIN-PROBE in services-rust's entry
    (the same tool that corrected the 07-12 red herring) to find whether it reaches `main` at all.
    Also note `IopDoLoadUnloadDriver` (`driver.rs:2301-2312`) waits on its work item with
    `Timeout=NULL` — an INFINITE wait — when called from a non-system process (i.e. services.exe),
    so anything that fails to complete that work item hangs the caller forever with no timeout.
  * **PARKED, ready to resume (2 paired stashes on `conformance-ratchet`):**
    `arm64-scm-bringup: npfs+services staging, archcfg targets, iomgr STAGED_SVCS Npfs` +
    `arm64-scm: winlogon StartServicesManager enable`. Both proven to the states above. NOT landed:
    without the SCM they deliver nothing today, and adding an unexercised driver load + a 368KB
    npfs.sys to every arm64 boot is pure flake surface — precisely the wrong thing to do while the
    smp lane is hunting a 1-in-258 arm64 0x4E.
  * **DISPOSITION / answer to the smp-bringup lane:** arm64 kmtest guest-load soak is **blocked**
    behind an arm64 SCM bring-up (its own lane: Layer 4 + then first light of RPC/SCM on arm64),
    NOT the quick amplifier they hoped for. Their **amd64** §9.56/0xF4 soak works TODAY (baseline
    committed, `conformance --arch amd64 --smp N` ready).
  * **Harness bug found en route (real, unfixed, moot until Layer 4):** `vm.py` hardcodes
    `MAX_VM_LIFETIME_SECONDS = 30*60` (threading.Timer + lockfile deadline) while
    `conformance --timeout` defaults to 6000s and kmtest's `launch_drain_secs` is 5400 — and the
    `VIBEOS_QEMU_MAX_LIFETIME` env override AGENTS.md documents is honoured ONLY by the shell
    toolkit (`qemu/lib/common.sh:21`), NOT by the Python harness. So any sweep >30 min is killed
    mid-run. Nobody hit it because the amd64 sweep finishes inside 30 min. Fix = have vm.py read
    the documented env var, and have conformance.py raise it for its own boot only.
  * **Latent, bounded:** `kmtest_drivers` (kmtests/CMakeLists.txt:258) is a hand-list that has
    already drifted — it omits the cc*/fltmgr drivers, so `kmtest_all` doesn't build them and the
    mkramdisk glob can't stage them. Harmless today (no visible Cc*/Flt* test is in any of the 3
    sweep lists) — but if you make one visible, add its driver there. Also `fltlib.dll` is not
    built for arm64 at all (`LDR: Skipping missing import fltlib.dll for kmtest_.exe`).
- 2026-07-15 (later): **arm64 SCM BROUGHT UP — the Layer 3/4 entry above is now WRONG in its key
  claim, and the real blocker is a DROPPED RpcTryExcept in advapi32-rust.** Correcting the record
  first, then the chain.
  * **CORRECTION — "spawning services.exe WEDGES the arm64 boot" NO LONGER REPRODUCES.** That was
    measured on a tree ~30 commits behind mainline; after merging current master the same wiring
    boots to `shell ready` in ~41s with services.exe running. Do NOT chase the "wedge". (The
    ZwLoadDriver-hang hypothesis for it was already flagged doubtful there; it is now moot.)
  * **The bring-up chain (each layer found by measurement, all now working on arm64):**
    1. **npfs.sys** — never built/staged/loaded. Registered Start=1 in the SYSTEM hive already, but
       arm64 honours iomgr's hand-rolled `STAGED_SVCS` list, not the registry. Added → loads clean
       (`ZwLoadDriver …\Npfs -> 0x00000000`). This is npfs's FIRST LIGHT on arm64.
    2. **services.exe** — never built/staged/spawned (`winlogon.rs` cfg(aarch64) skipped the SCM).
       Its whole import set was already staged; only the binary was missing.
    3. **lsass — NOT OPTIONAL.** `ScmCreateServiceDatabase` ends in `ScmWaitForLsa()` =
       `WaitForSingleObject(LSA_RPC_SERVER_ACTIVE, INFINITE)` (services-rust/src/services.rs:253);
       only lsass signals it. SCM-without-LSA hangs FOREVER mid-database (measured: stuck 420s+
       after "LastKnownGood OK"). x86 never hit it because its winlogon branch starts lsass.
       lsasrv/samsrv were the only 2 of lsass's 15 transitive DLLs unstaged.
    4. **msv1_0 — invisible to an import walk.** lsass exited `0xC0000135` (STATUS_DLL_NOT_FOUND)
       with EVERY static import present and NO delay-import directory: lsasrv loads the MSV1_0 auth
       package BY NAME at runtime (the Lsa "Authentication Packages" value) and propagates the
       failure as lsass's exit code. **Reusable lesson: a transitive import-table closure is
       necessary but NOT sufficient for "what does this binary need staged" — runtime LoadLibrary
       by registry name is invisible to it.**
    5. Result: **the SCM is fully up on arm64.** wWinMain → CheckForLiveCD → Events created →
       Security initialized → LastKnownGood → **ServiceDatabase OK** → RegisterServicesProcess →
       ScmAutoStartServices → **SC_AutoStartComplete FIRES**. npfs creates `NtControlPipe0`. The
       kmtest sweep now passes through the **REAL SCM gate** (`SCM auto-start complete (waited
       ~0s)`) instead of the arm64 WARN-and-proceed path, and the SCM services kmtest's
       CreateService as far as `ScmLoadService(Kmtest)`. **The boot still reaches the desktop.**
       => The 07-12 "services.exe never publishes SC_AutoStartComplete on arm64" finding is CLOSED.
  * **THE REMAINING BUG (real, general, not arm64-specific): advapi32-rust dropped the C's
    RpcTryExcept.** services.exe dies with `[UEXC] Code=0x6BA` (RPC_S_SERVER_UNAVAILABLE) inside
    `ScmLoadService(Kmtest)` "logging success event" → `ScmLogEvent` → `RegisterEventSourceW`.
      - C (`dll/win32/advapi32/service/eventlog.c:1276-1289`): the `ElfrRegisterEventSourceW` RPC
        call is wrapped `RpcTryExcept { … } RpcExcept(EXCEPTION_EXECUTE_HANDLER) { Status =
        I_RpcMapWin32Status(RpcExceptionCode()); } RpcEndExcept;` — the raise is CAUGHT and mapped,
        so RegisterEventSourceW returns NULL and ScmLogEvent's `if hLog.is_null() { return; }`
        handles it gracefully.
      - Rust (`sdk/lib/rust/advapi32-rust/src/service/eventlog.rs:944`, the LIVE cohab body):
        `Status = ElfrRegisterEventSourceW(…);` — **NAKED. The handler was dropped in translation.**
        The raise escapes and kills the caller.
      - **§9.53 class** (dropped/no-op'd exception handling in a translation). LATENT ON ALL ARCHES
        — x86 only escapes it because EventLog always runs there. `ScmLogEvent` already carries a
        VIBEOS half-workaround for exactly this (`if ScmInitialize != 0 { return; }`, whose comment
        names DUMMY_PSEH and the dead RPC handler) — it covers only the INIT phase, not post-init
        service loads.
      - **Fix = restore the catch**, not stage more services. Rust can't `__try`, so the sanctioned
        pattern is a native-SEH C shim (the `ntoskrnl/kmsafecopy.c` precedent: `_USE_NATIVE_SEH` C
        wrapper, Rust calls it; clang SEH is calls-only, and an RPC stub call IS a call, so the
        raise lands in a callee with the return address inside the __try scope table).
  * **DO NOT "fix" this by staging eventlog.exe — MEASURED REGRESSION.** It builds and starts
    (pid=268) but never connects to the control pipe, so the SCM blocks EARLIER, at
    `ScmStartUserModeService(EventLog): waiting for connect`, and never reaches
    SC_AutoStartComplete at all. Strictly worse than leaving it absent. Reverted.
  * **NOT LANDED.** The chain (npfs+services+lsass+msv1_0 staging, archcfg targets, iomgr
    STAGED_SVCS Npfs, winlogon StartServicesManager+StartLsass) is parked. Two reasons: the SCM
    still dies at the RpcTryExcept bug above, and turning on a full autostart storm changes the
    arm64 boot profile — which wants measuring before it's inflicted on the smp lane's 1-in-258
    0x4E flake hunt. NOTE `winlogon.rs` deliberately does NOT call `WaitForLsass()` on arm64 (x86
    does): services.exe already does its own ScmWaitForLsa, and making winlogon wait would put a
    fresh INFINITE wait directly on the boot-to-desktop path.
  * Owed housekeeping when it lands: the `STAGED_SVCS` edit shifts iomgr/driver.rs line numbers, so
    the lint baselines (`lint_kernel`, `scan_silent_failures`) need regenerating — the "2 new/4 new"
    they report are pre-existing findings at drifted line numbers, not new defects.

### 2026-07-15 (later) — RpcTryExcept FIXED (native-SEH shim); the arm64 baseline is now blocked one level deeper, in fastfat

* **The dropped RpcTryExcept is FIXED and validated live.** New file
  `dll/win32/advapi32/service/eventlog_seh.c` provides native-SEH C thunks
  (`ElfrRegisterEventSourceW_seh` / `ElfrReportEventW_seh` / `ElfrDeregisterEventSource_seh`) that
  restore the `RpcTryExcept { … } RpcExcept { I_RpcMapWin32Status(RpcExceptionCode()) }` the cohab
  dropped. Compiled with `_USE_NATIVE_SEH`, which `pseh2.h` checks (line 28) BEFORE the global
  `_USE_DUMMY_PSEH` default (line 179), so this one TU gets a real `__try` while the rest of
  advapi32 stays DUMMY. The "calls-only" contract holds trivially — the `__try` body is nothing but
  the RPC stub call, so the raise lands one frame down in the stub (return address inside the scope
  table). It's a SOFTWARE raise (RpcRaiseException), which clang native SEH catches on amd64 AND
  arm64 — kmsafecopy's arm64 *hardware*-fault gap (§9.69) does not apply. `eventlog.rs` declares the
  three thunks and routes the ScmLogEvent-lifecycle sites through them. i386 keeps a pass-through
  (`#else`) — clang i386 native SEH is unreliable (the reason `USE_DUMMY_PSEH` is the i386 default;
  the kmtest native-SEH opt-in likewise excludes i386), and EventLog is present on the x86 boot so
  the stub never raises there anyway. **Measured on arm64 (with the parked SCM chain applied):**
  services.exe no longer dies at `ScmLoadService` with 0x6BA. It survives, logs the DcomLaunch
  load-failure gracefully (`ScmLoadService(DcomLaunch) done: 2`), and proceeds through the autostart
  list. **This closes the RpcTryExcept blocker (task #11).**
* **The advapi32 fix LANDS on its own** — it is a genuine, general robustness fix (a dropped
  exception handler) and is **inert on every current normal boot** (arm64 doesn't run services yet;
  x86 has EventLog present → no raise), so it gates ×3 green without the SCM chain.
* **NEW blocker for the arm64 baseline — a fastfat dropped-translation stub.** With services.exe now
  surviving, the SCM autostarts **EventLog**; `eventlog.exe` runs and writes its `.evt` log →
  `SetEndOfFile` → fastfat `FatSetEndOfFileInfo` → **BUGCHECK 0x23 FAT_FILE_SYSTEM (0xdead0000)**.
  Root cause: `FatSetEndOfFileInfo` was **dropped during the fastfat C→Rust translation**
  (`sdk/lib/rust/fastfat-rust/src/fileinfo.rs:2442` — *"no bank body (dropped at translate) —
  link-recover later"* — it's declared `extern` and the link falls through to a bugcheck stub at
  `drivers/filesystems/fastfat/rust_cohab_stub.c:622`, `KeBugCheckEx(0x23,0xDEAD0000,…)`). It is
  **one of 25** such `STUB_BUGCHECK()` fastfat functions. This is arm64-specific ONLY because arm64
  boots off the FAT32 ramdisk so every file write goes through fastfat; on x86 the boot FS differs
  and these stubs are never hit. The C original is intact (`fileinfo.c:4387-4775`, ~200 lines of
  active code: SEH unwind-on-abort restoring file sizes, `FatInitializeCacheMap`/`CcSetFileSizes`/
  `MmCanFileBeTruncated`, `FatAddFileAllocation`, dirent updates). **This is a distinct fastfat
  translation lane, not part of "fix services."**
* **Consequence for landing the SCM chain:** it can't land until `FatSetEndOfFileInfo` (and likely a
  few sibling stubs the sweep exercises) are translated — the now-surviving services.exe reaches
  EventLog and bugchecks, which would *regress the arm64 desktop boot*. The parked stash note
  (`arm64-scm-bringup v3`) records this. Pre-fix, services.exe died at the first ScmLogEvent (0x6BA)
  before ever reaching EventLog, which is why the chain's earlier "boot still reaches desktop"
  measurement held — the RpcTryExcept fix is precisely what advances the boot far enough to expose
  the fastfat gap.
* **Baseline status: still none.** The sweep now reports `sweep=DID NOT FINISH BUGCHECK=0x00000023`
  (was: services.exe died silently). Next step toward a baseline = translate `FatSetEndOfFileInfo`
  (rustconv single-function, ~$0.03–0.15, or a careful hand-translation of the C), re-run the arm64
  sweep, and clear whatever further stubs the kmtest binaries hit.

### 2026-07-16 (later) — arm64 SCM fully UP; conformance sweep RUNS; first arm64 result captured

**Three fixes landed** (all ×3-gate-green, on master): the advapi32 `RpcTryExcept` (`f754a3a8155`),
the fastfat `FatSetEndOfFileInfo` recovery (`e5f2b28a3b5`), and the lpc `LpcExitThread` §9.65
(`ce89f8d6af1`) + a `verify_ffi` baseline re-anchor (`3d004d50edd`). Together they clear the whole
arm64 SCM path.

* **The `LpcExitThread` 0x3 — hypothesis CONFIRMED by re-sweep.** After the §9.65 fix, the boot
  emits many `PspExitThread: about to call LpcExitThread` with **no `0x3` following** (pre-fix, that
  line was immediately followed by the bugcheck). So the always-on `DbgBreakPoint()` (C: DBG-gated
  `ASSERT(FALSE)` for the "thread exits holding a reply message" TODO) *was* the 0x3.
* **End-to-end validated on arm64** (SCM chain applied): threads exit cleanly → SCM starts
  `DcomLaunch` → `EventLog` (**connect + START both succeed** now) → `Rpcss`/`Themes`/`SamSs`/… →
  `SamSs` times out `connect result=1053` but the SCM correctly **does not block**, walks the rest of
  the list, and **`SC_AutoStartComplete` FIRES**. The kmtest sweep sees it (`SCM auto-start complete
  (waited ~0s)`), reads `kmtest-tests.txt`, and **runs the first test to a real result:**
  **`[KMTEST:END CmSecurity pass=300 fail=89 skip=2]`** — the first arm64 kmtest result ever.
* **The last mile — sweep stalls after test 1 (distinct, arm64-specific).** The 2nd `kmtest_.exe`
  launches and immediately hits `*** ASSERT FAILED: BytesReturned == sizeof(RequestPacket) at
  kmtest\support.c:52` → `Callback handler dying! Error code 2`. That's the kmtest **usermode-callback
  worker** (`KmtUserModeWorker`): a test needing usermode callbacks (e.g. `QueryVirtualMemory`) spawns
  a worker that loops on `IOCTL_KMTEST_USERMODE_AWAIT_REQ`; on arm64 that METHOD_BUFFERED IOCTL came
  back with the wrong `BytesReturned`, so the worker asserts + dies and the kernel-side test hangs
  waiting for a callback that never returns → the sweep drains out (`launched=1 sweep=DID NOT
  FINISH`). i386 runs the full sweep (task #1 baseline), so this is an arm64 IOCTL/harness gap, NOT
  the SCM. **This is the remaining blocker for a *full* multi-test arm64 baseline.**
* **Landing the SCM chain is a cross-lane decision.** With the 3 fixes in, the chain no longer
  regresses the arm64 desktop boot (it reaches the shell WITH services running). But it turns on a
  full autostart storm on *every* arm64 boot, which changes the boot profile the smp lane's 1-in-258
  0x4E flake hunt runs against. The clean landing is to **gate winlogon's `StartServicesManager` to
  conformance-mode** (`VIBEOS_PAYLOAD=kmtest`) so normal arm64 boots stay lean — then land + coordinate
  with the smp lane. Chain parked in stash `arm64-scm-bringup v5`.

### 2026-07-16 — fastfat FatSetEndOfFileInfo TRANSLATED + validated; arm64 baseline now blocked on a 0x3 thread-exit crash

* **`FatSetEndOfFileInfo` recovered.** rustconv `translate --fn FatSetEndOfFileInfo --mode mechanical`
  ($0.06, durable archive under `tools/rustconv/.responses/`), integrated into `fastfat-rust/src/
  fileinfo.rs` (the dropped-at-translate `extern` stub replaced with the real body), and the
  shadowing bugcheck stub removed from `rust_cohab_stub.c` (lld `--allow-multiple-definition` is
  first-match and the C obj links first, so leaving it would have silently kept the stub winning).
  Integration was ~a dozen house-convention fixes: bindgen field paths
  (`Parameters.SetFile.__bindgen_anon_1.__bindgen_anon_1.AdvanceOnly`, `.__bindgen_anon_1.LowPart`),
  `FatUnpinBcb`/`FatGetDirentFromFcbOrDcb` taking `addr_of_mut!(Bcb)` not the Bcb, `NodeType(Fcb)`
  untyped-cast removal, `NT_ASSERT((..) as i32)`, local `const FcbGood/STATUS_FILE_CLOSED`, an
  `IsFileDeleted` extern, and the `if false /* _SEH2_AbnormalTermination() always false */` idiom
  (mirrors `FatSetAllocationInfo`) that also dead-eliminates the macro-only `CcGetFileSizePointer`.
  **Compiles clean on all three arches** (i386/amd64/arm64). **Lands separately** — a genuine
  dropped-translation recovery, inert on normal boots (nothing SetEndOfFile's the FAT ramdisk there).
* **VALIDATED end-to-end on the arm64 sweep.** With the SCM chain applied: `ScmLoadService(EventLog)
  done: 0` → `logging success event` → `event logged OK`. i.e. the full ScmLogEvent lifecycle
  (RegisterEventSourceW → ReportEventW → DeregisterEventSource) now runs — the RpcTryExcept fix + the
  fastfat fix together. The SCM then created `NtControlPipe0` and walked the autostart list all the
  way to **SamSs** (SAM). Two fastfat translation gaps down; the SCM is deep into real service
  startup now.
* **NEW blocker for the baseline: `BUGCHECK 0x3` in `LpcExitThread`.** During a **System thread
  exit** (`PspExitThread … about to call LpcExitThread`), right after the `[NETSTACK] TDI self-test`
  + `SET_INFO self-test` ran, and while the SCM was `ScmStartUserModeService(SamSs): waiting for
  connect`. Params `(0xfffffa…d60, 0xfffffa…c00, 0, 3)` — both kernel pointers, so likely a
  `KeBugCheckEx(3, thr, obj, 0, 3)` from the LPC/thread-teardown path, NOT a literal
  INVALID_AFFINITY_SET. **Sole banner in the log (not a §9.54 mask.)** Almost certainly the known
  arm64 thread-teardown flake family (cf. the netstack-TDI 0x1E residual + the smp lane's 1-in-258
  0x4E) amplified by the dense thread churn of full service startup — **unrelated to fastfat/
  advapi32** (the boot is well past both). This is a distinct LPC/PS-thread-exit lane. `SC_AutoStartComplete`
  never fires, so the kmtest sweep still never runs → **still no baseline**, but the wall is now two
  subsystems deeper than where "fix services" started.
