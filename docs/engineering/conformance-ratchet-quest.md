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
