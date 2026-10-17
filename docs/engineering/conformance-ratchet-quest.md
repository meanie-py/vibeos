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

## Resume here (2026-10-17)

The survey below is history. The current state and the ordered owed queue are the LAST
progress-log entry, "2026-10-17b — the overnight queue". Re-derive every number in
it from `tools/conformance-baseline/<arch>/kmtest.txt` and a fresh sweep before trusting it.
Windows 11's own results for these suites are `tools/conformance-baseline/win11-amd64/kmtest.txt`.

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
bash ../tools/canon-run.sh cmake -DENABLE_ROSTESTS=ON . # option is cached; sticks for later builds
bash ../tools/canon-run.sh ninja kmtest kmtest_drv      # start here, then widen
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

**The TOTAL assertion count is not a score, and it moves BOTH ways for good reasons.**
Read `fail`, and read it per test; `pass` and `pass+fail` are not correctness metrics.

- **Up.** A test that bailed early now gets further in, so assertions that never executed
  start executing. More correctness, more assertions — the common case, and the one that
  makes a growing total look alarming when it is the opposite.
- **Down.** Fixing wrong *data* can DELETE passing assertions. `VCheckAcl__` iterates the
  ACL's actual `AceCount` but only `va_arg`-compares while `i < expected`, so every surplus
  ACE still earns 4 structural passes (size/alignment) before it `continue`s. When the
  SECURITY hive-root descriptor was corrected from the fabricated 4 ACEs to the right 2
  (task #72), CmSecurity went **906/4 → 900/2**: 8 passing assertions removed, 2 failures
  fixed, strictly more correct. Predicted before the run and confirmed exactly.

So: a falling `pass` is not a regression, and a rising `pass` is not progress. Both are
consequences of how far execution reaches and how much data there is to check. Only the
per-test `fail` count, plus the named assertions behind it, means anything — which is why
the ratchet reports per-test deltas and not a total.

**The corpus cross-checks itself, so run the WHOLE sweep before believing a fix.** A change
that satisfies the test you were chasing can contradict an assertion in a different test, and
that other test is just as much the oracle. Worked example (task #73): `\REGISTRY\USER\S-1-5-18`
failed to open with `STATUS_OBJECT_PATH_NOT_FOUND`, the reparse target `\Registry\User\.Default`
missed the root directory entry `\REGISTRY` on a case-sensitive compare, and the obvious fix was
to give the Key object type `TypeInfo.CaseInsensitive = TRUE` like Directory/SymbolicLink/Device
/File have. It worked — CmSecurity went 900/2 → 1048/1, exactly as predicted — and the same
sweep turned `ObTypes` 765/17 → 764/18, because **`ObTypes.c:217` checks that very field against
WS03 and expects FALSE**. Registry key opens really are case-sensitive at the object-directory
layer; that is *why* NT spells every registry path it builds `\REGISTRY\...` in upper case, and
the actual defect was ReactOS spelling its symlink targets `\Registry\...`. One run, one
contradicted assertion, and a plausible wrong fix died before it could be committed. Never grade
a fix on the test it was aimed at.

- `tools/conformance-baseline/<arch>/kmtest.txt`: one line per test —
  `name pass_count fail_count skip|crash|hang` at baseline time, followed by an indented
  line per failing assertion:

  ```
  IoDeviceObject OK pass=185 fail=2 skip=0
      assert  IoDeviceObject_drv.c:198
      assert? IoDeviceObject_drv.c:206
  ```

  **The floor is the SET of `file:line` identities; the counts are commentary.** `assert?`
  marks a known-bistable identity — not a regression when it fires, not an improvement when
  it does not, and carried forward across a re-seed instead of being re-rolled from one boot.
  The message text is deliberately NOT part of the identity: it carries addresses and
  NTSTATUS values that differ every boot. A row with no `assert` lines has not been seeded
  yet and keeps the old fail-count gate, so seeding proceeds per-arch with no flag day.
- **Re-seed with `--write-baseline --only <TEST>…`, not a bare `--write-baseline`.** The
  bare form rewrites every row from the single sweep in front of it, so the rows we know
  are bistable (`IoDeviceObject` on all three arches, `ExPools` on arm64) get re-rolled
  from one sample and whichever side that boot landed on becomes the permanent floor.
  `--only` replaces just the named rows and leaves the rest at their banked sample; it
  refuses a name the sweep never produced, and announces a name the baseline did not
  carry yet rather than silently growing the file.
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
  rate-limit) before whpx is usable for long-running in-guest suites. (Removed
  2026-10-10: on the OptiPlex it logged every DVD media poll, 190k lines in a day.) Sweep boots use
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
  *(SUPERSEDED — the 0x3 was the LpcExitThread §9.65 `DbgBreakPoint`, fixed+landed `ce89f8d6af1`; see
  the "arm64 SCM fully UP" milestone entry above and the test-2 root-cause below.)*

### 2026-07-16 (later still) — test-2 stall ROOT-CAUSED: an arm64 kernel user-APC-delivery bug (NOT the harness)

Instrumenting `IOCTL_KMTEST_USERMODE_AWAIT_REQ`'s `KeWaitForSingleObject` (the usermode-callback
worker `kmtest/support.c`) showed it returns **`STATUS_USER_APC` (0xC0) *persistently*** — even though
the wait is `Alertable=FALSE`. `STATUS_USER_APC` from a `UserMode` wait means "unwind to user mode to
deliver the queued user APC," but on arm64 that APC is **never delivered/cleared**, so every re-issue
of the wait returns 0xC0 again. The driver's `break`-on-APC then completes the IOCTL with
`Information = 0`; the usermode worker's `ASSERT(BytesReturned == sizeof(RequestPacket))` fires, the
worker dies, and the kernel-side test hangs → sweep stalls after test 1.

* This is the **same arm64 user-APC-delivery family** as the `KiUserModeCallout` issues (field manual)
  — a real kernel bug, well beyond the kmtest harness or the SCM.
* **A speculative "re-wait on APC" patch in `kmtest_drv.c` was WRONG and was REVERTED.** It turned the
  single mis-sized return into an *infinite loop* (spin on 0xC0, flooding serial, hanging the boot) —
  which is itself the proof that the pending user APC is never delivered. Lesson: the fix belongs in
  the arm64 return-to-usermode APC path, not the test harness. (Also: never loop a driver dispatch on
  `STATUS_USER_APC`.)
* **Next step for a full multi-test arm64 baseline** = fix arm64 user-APC delivery (why a pending user
  APC on a `UserMode` `KeWaitForSingleObject` is never delivered on return-to-usermode). Distinct
  kernel lane; owed. Until then the arm64 conformance sweep captures **exactly one** result
  (CmSecurity 300/89/2) before the first callback-using test wedges.

### 2026-07-16 (landed) — arm64 SCM chain LANDED for arch parity (services always-on)

The parked SCM chain (`stash@{0}`) is now on **master** (`76189a16a5d`, merged `d7ff4125020`):
archcfg builds npfs/services/lsass/lsasrv/samsrv/msv1_0 for arm64, mkramdisk stages them,
iomgr adds `Npfs` to arm64 `STAGED_SVCS`, and winlogon's `#[cfg(aarch64)]` branch calls
`StartServicesManager` + `StartLsass`. **User directive: "fix the services and have arch
parity"** — landed **always-on** (not conformance-mode-gated); arm64 now runs services on
every boot, exactly like x86.

**Parity is literal, not aspirational — it's the SAME code path.** Booted amd64 and arm64
side by side: the SCM autostart sequence is **byte-for-byte identical** (EventLog
starts+connects `result=0`, Rpcss/Themes walked, then SamSs). services-rust is arch-independent;
landing the chain just lets arm64 reach it. arm64 boots to desktop in ~44s WITH services (vs
~40s lean — no meaningful slowdown; the "arm64-with-services is slow" worry from the prior
checkpoint was overstated: services start **async after** the shell marker, so `shell ready`
timing is unchanged; the 30s SamSs wait below is background). Gate ×3 green + stamp + pushed.

**Deliberate x86 divergence retained:** arm64 skips `WaitForLsass()` after `StartLsass` (x86
keeps it). services.exe does its own `ScmWaitForLsa`; putting an INFINITE lsass wait on the
boot-owning process (winlogon) would move the hang onto the boot-to-desktop path.

**NOT fixed — SamSs 30s connect-timeout (`connect result=1053`), PRE-EXISTING + SHARED across
ALL arches (amd64 shows the identical timeout), NON-BLOCKING.** The `SUMS(SamSs): CreateProcessW
returned pid=0` is a red herring — `ScmIsSecurityService` (matches `\system32\lsass.exe` in the
ImagePath) deliberately skips CreateProcessW for the security service (lsass is already
running), so `ProcessInformation.dwProcessId` stays 0. The real gap: lsass comes up (signals
`LSA_RPC_SERVER_ACTIVE`, so `WaitForLsass`/`ScmWaitForLsa` return) but lsasrv's `DispatcherThread`
→ `StartServiceCtrlDispatcher` handshake to `\pipe\net\NtControlPipe0` (the fixed security-service
pipe) doesn't complete within `PipeTimeout=30000`, so the SCM times SamSs out and continues
(`ScmAutoStartServices completed` → `SC_AutoStartComplete` fires ~30s later, in the background).
The code is all present + translated correctly: SCM `SetSecurityServicesEvent()` (services.rs:182),
lsass `wWinMain` calls `LsapInitLsa`+`ServiceInit` (lsass-rust:41,53), lsasrv `DispatcherThread`
waits `SECURITY_SERVICES_STARTED` → `I_ScIsSecurityProcess` → `StartServiceCtrlDispatcherW`
(lsasrv-rust/service.rs:237-294). So it's a runtime handshake failure, not a dropped call.
**Next diagnostic (owed, benefits all arches):** instrument lsasrv `DispatcherThread` (traces are
`__WINE_DBCL_TRACE`-gated → invisible) to see whether it clears the `SECURITY_SERVICES_STARTED`
wait and whether `StartServiceCtrlDispatcherW` connects/returns — then trace the SCM side
`ScmWaitForServiceConnect(SamSs)` on NtControlPipe0.

**Gate note for the smp lane (heads-up):** arm64 now runs the full service storm on every boot
(more thread churn). Separately, the pre-existing arm64 **netstack-TDI self-test** silent-wedge
flake still bites: the first gate here hard-failed `HANG_TIMEOUT` at `[NETSTACK] TDI server:
accept 0x00000000` (a genuine wedge, correctly NOT auto-retried — `HANG_TIMEOUT` isn't blanket-
retryable), but arm64 smoked **3/3 green on immediate repeat** and the re-gate was clean. That
flake is upstream of services (driver-init) and unrelated to this change; re-run the gate if it
hits. Full baseline still owed on the arm64 user-APC bug (test-2, above).

### 2026-07-16 (fixed + landed) — SamSs 30s SCM connect-timeout: root-caused + fixed (74c08f68e68)

The SamSs `connect result=1053` timeout (documented above as an owed, shared follow-up) is
**FIXED on all arches**. It was NOT a dropped call — it was a genuine Heisenbug.

**Root cause (by elimination + [SCTRL]/handshake tracing):** lsasrv's `DispatcherThread` gated
the SamSs pipe-connect on `WaitForSingleObject(SECURITY_SERVICES_STARTED, INFINITE)`. That named
event is created independently by BOTH the SCM (services.exe) and lsasrv and is supposed to
resolve to one shared kernel object — but that **cross-process named-object sharing is
timing-fragile in this kernel**: when lsasrv won the create race, the SCM's `SetEvent`
intermittently did NOT wake this waiter, so the dispatcher hung on the INFINITE wait, never
reached `StartServiceCtrlDispatcher`, and the SCM timed SamSs out at 30s. Classic Heisenbug —
**any DbgPrint added to the path perturbed the timing enough to make it "work"** (instrumented
boots connected 3/3; uninstrumented timed out), which is exactly why it was hard to pin. Two
structural fixes FAILED and taught the shape: (1) spawning the dispatcher first (reorder
`ServiceInit` ahead of `LsapInitLsa`) crashed it early (0x3E6); (2) forcing lsasrv to *open* the
event instead of create it still 4/4-timed-out — proving even `OpenEventW` couldn't reliably see
the SCM's object. So the event itself is the unreliable link.

**Fix — sync on the PIPE, not the event.** Every other service connects via its control pipe
reliably every boot, and the SCM creates `NtControlPipe0` *just before* it signals the event
(`ScmCreateNewControlPipe` then `SetSecurityServicesEvent` in database.rs) — so the pipe is the
earlier, reliable rendezvous. (a) lsasrv `DispatcherThread`: the `SECURITY_SERVICES_STARTED` wait
is now best-effort + BOUNDED (3s) + non-fatal; it proceeds to the connect regardless. (b)
advapi32 `ScConnectControlPipe`: for the security process (`bSecurityServiceProcess`), POLL
`WaitNamedPipeW` until `NtControlPipe0` exists (it returns FALSE-fast/FILE_NOT_FOUND for a
not-yet-created pipe, so the poll is cheap) — the SCM creates the pipe lazily as it loads SamSs,
which can be after lsass starts. Non-security services are UNCHANGED (single 30s wait): the SCM
creates their pipe before it spawns them.

**Verified arm64:** `SamSs connect result=0` on 5/5 boots that reached services (was a 30s timeout
on EVERY boot before); `ScmAutoStartServices completed` fires promptly. Gate ×3 green + pushed.
The 2 boots that didn't reach SamSs in the runs were the pre-existing netstack-TDI self-test flake
(upstream of services, unrelated). **Reusable lesson (also in the field manual):** a
`SECURITY_SERVICES_STARTED`-style cross-process named-event handshake is timing-fragile here;
prefer the named pipe (or another reliably-shared object) as the rendezvous.

### 2026-07-16 (fixed) — arm64 user-APC delivery: the test-2 wedge is CLEARED; sweep runs end-to-end

The arm64 user-APC-delivery blocker (test-2 stall, task #2) is **FIXED** (field manual §9.90). Root
cause: the arm64 SVC return path (`KiSvcDispatch → KiTrapReturn → eret`, `ke/arm64/trap.rs`) never
delivered pending user-mode APCs — unlike amd64's `KiSystemServiceExit` (`trap.S:911`, checks
`UserApcPending` → `KiInitiateUserApc`) and i386's trap epilog. `UserApcPending` never cleared, so a
`UserMode` `KeWaitForSingleObject` (the kmtest usermode-callback worker's
`IOCTL_KMTEST_USERMODE_AWAIT_REQ`) re-returned `STATUS_USER_APC` (0xC0) forever, the worker asserted
`BytesReturned == sizeof(RequestPacket)` and died, and the kernel-side test hung → `launched=1
sweep=DID NOT FINISH`.

**Fix:** new `KiDeliverUserApcArm64` at the SVC (`case 0x15`) tail — on return-to-EL0 with
`UserApcPending` set, raise APC_LEVEL, `KiDeliverApc(UserMode, ExceptionFrame, TrapFrame)`, lower.
Mirrors amd64; DAIF-neutral (uses `KfRaiseIrql`, not sti/cli) because on this port the SVC runs
IRQ-masked and `KiTrapReturn` owns the banking window. Arm64-only (`ke/arm64/`); x86 untouched.

**Validated (uninstrumented):** arm64 boot-to-desktop still green (41s); the conformance sweep went
`launched=1 sweep=DID NOT FINISH` → **`launched=81 sweep=done`**. `ExCallback` (former test-2 wedge)
and `ExDoubleList` now pass — the sweep runs to completion end-to-end for the FIRST time.

**Still no clean baseline — the NEXT wedge is `ExFastMutex` (distinct, in-kernel, NOT user-APC).**
Post-fix the sweep reports `ok=3 bad=78`, but that's a **cascade**: `ExFastMutex` TIMEOUTs in-kernel
(`kmtest_drv.c:454 KeWaitForSingleObject timed out!` — the worker's `AWAIT_REQ` returns
`STATUS_TIMEOUT`, not `STATUS_USER_APC`; the kernel test never posts a callback because it's hung),
and being unreapable it poisons the remaining 77 tests with `SET_RESULTBUFFER errors=1` (the same
cascade mechanism as amd64's SeTokenFiltering/TcpIpConnect). Clue: `MmGetSystemRoutineAddress:
'ExiAcquireFastMutex'/'ExiReleaseFastMutex'/'ExiTryToAcquireFastMutex' NOT FOUND` — ExFastMutex
resolves the internal fast-mutex primitives by name and they're unexported on our build. The
`support.c:52` assert is **collateral** (worker times out because the kernel test is stuck), not the
user-APC path. **Next step toward the arm64 baseline:** quarantine `ExFastMutex` in
`tools/payloads/kmtest/kmtest-tests-arm64.txt`, re-sweep to unmask the ~77 cascade victims' real
results, iterate on any further wedgers, then `conformance --arch arm64 --write-baseline`. The
user-APC fix lands on its own (a real, correct kernel fix; unblocks the whole sweep).

### 2026-07-16 (baseline grind, in progress) — arm64 quarantine sweeps after the user-APC fix

With the sweep running end-to-end (user-APC fix §9.90), the arm64-specific wedgers surface
one per boot (the amd64/i386 M3 pattern). The arm64 list started as a copy of amd64's, so
it already carries all amd64 quarantines; these are the NEW arm64-only ones:
- **Sweep 1 — `ExFastMutex` TIMEOUT (in-kernel hang).** `kmtest_drv.c:454 KeWaitForSingleObject
  timed out!` — the usermode-callback worker's `AWAIT_REQ` returns `STATUS_TIMEOUT` because the
  kernel test never posts a callback (it's hung); `support.c:52 BytesReturned` assert is
  collateral. Passes on amd64+i386 → arm64-specific. The `Exi*FastMutex NOT FOUND` spam is
  benign (i386-only fastcall variants; the test `skip()`s them on 64-bit). Quarantined.
- **Sweep 2 — `ExHardError` BUGCHECK 0x1E (0xC0000005) at kbase+0x1c9cf8.** Faults IMMEDIATELY
  at kernel-test start (3 log lines after "Starting test"). Same structural class as i386 (an
  i386-quarantined crasher). NOT the user-APC fix — reproduces on i386 (never runs arm64 code)
  and faults at test-start, not on an APC return. Quarantined (a BUGCHECK kills the guest, so
  it must be quarantined to sweep past it).

Both quarantined in `kmtest-tests-arm64.txt` with negative results recorded; triage owed (both
are pre-existing arm64 structural bugs, distinct from the fixed user-APC path).

**Harness note — the netstack-TDI flake makes the grind expensive.** Sweep 3 hit the known
pre-existing arm64 **netstack-TDI self-test** silent-wedge flake (~1/6 boots, upstream of
services): the boot reached `shell ready` + `ScmAutoStartServices completed` but
`SC_AutoStartComplete` never fired, so the sweep launched 0 tests and the conformance verb
drained its full `--timeout` (~90 min at the 6000s default) waiting for output that never came.
Mitigation for the grind: run `conformance --arch arm64 --timeout 1800` so a boot-flake caps at
~30 min instead of 90 (a real sweep resolves in minutes — a wedger either BUGCHECKs fast or
times-out-then-cascades; only a launched=0 boot-flake drains the whole budget). A proper fix is
the `vm.py MAX_VM_LIFETIME`/early-abort-on-boot-gate-miss harness gap already noted above.
Getting a clean arm64 baseline = keep re-sweeping past the flake, quarantining each wedger, then
`--write-baseline`. NOT yet landed (no baseline file); the 2 quarantines are committed as
progress.

### 2026-07-16 (baseline grind cont.) — sweep 3: the arm64 crasher cluster is the KERNEL-SEH bug (§9.69 / task #10)

Sweep 3 (clean boot, `--timeout 1800`): with ExFastMutex + ExHardError quarantined, the sweep got
2 more OK (ExInterlocked 180/0, ExPools 14/4 → `launched=6 ok=5`) then **ExResource BUGCHECK 0x1E**.
Symbolicated (calibrating off the boot log's `KiIdleLoop@0xfffff800421a9410` vs the PE index):
**the crash PC `0xfffff800421c9cf8` = `RtlRaiseStatus+0x44` (rtl-diag/exception.rs) — the IDENTICAL
PC as ExHardError.** Two exception-raising tests, one instruction ⇒ ONE root cause (like the amd64
memcpy+0x40 cluster).

**Root: the arm64 kernel-mode exception-dispatch bug (§9.69 / task #10).** Both ExHardError and
ExResource deliberately raise a kernel-mode exception expecting their driver's native `__try/__except`
to catch it. On **amd64** native SEH catches it and both PASS (they're NOT amd64-quarantined). On
**arm64** the kernel-mode exception dispatch is broken (`RtlDispatchException`/`RtlVirtualUnwind` for
kernel frames — the same machinery §9.69 found wedging on hardware faults), so the raise goes
unhandled → 0x1E at `RtlRaiseStatus+0x44` (exception code 0xC0000005, single `[RAISE]` print, not
recursion). So the arm64 kernel-SEH gap blocks the **whole exception-raising kmtest class** — at
least ExHardError, ExResource, and very likely RtlExceptionKM/RtlStackKM et al. **Fixing task #10
(arm64 kernel-mode native-SEH dispatch) clears these together** — much higher leverage than
one-by-one quarantining.

**State:** ExResource quarantined (shared-root note). The arm64 baseline is now blocked primarily on
the arm64 kernel-SEH lane (task #10), NOT on the user-APC fix (landed) or the SCM (up). Remaining
options: (a) take on the arm64 kernel-mode SEH-dispatch bug — big, but clears task #10 + a class of
conformance quarantines at once; (b) keep quarantining the exception-raising tests one per boot to
reach a (heavily-quarantined) first baseline; (c) stop here with the user-APC fix as the win. The 3
arm64 quarantines so far (ExFastMutex timeout + the RtlRaiseStatus cluster ExHardError/ExResource)
are committed as progress; no baseline seeded yet.

### 2026-07-16 (M3 arm64 baseline SEEDED) — first arm64 kmtest baseline: 68 OK

The arm64 baseline grind is complete for a v1: `tools/conformance-baseline/arm64/kmtest.txt`
= **68 OK + 3 harness-NO-END, sweep=done, zero crashes**. Started from the amd64 list; the
NEW arm64-only quarantines (all committed with negative results):
- **ExFastMutex** — in-kernel TIMEOUT (arm64-specific; NOT the SEH cluster).
- **The arm64 kernel-SEH cluster (§9.69 / task #10):** ExHardError, ExResource, KeDpc, KeMutex,
  MmSection, RtlIsValidOemCharacter, SeInheritance, ZwAllocateVirtualMemory — each BUGCHECK 0x1E
  (mostly at the SAME PC `RtlRaiseStatus+0x44`; SeInheritance surfaces one frame later in
  `KeWaitForSingleObject`). All raise a kernel exception the native `__try/__except` can't catch
  on arm64; all pass on amd64. **Task #10 clears this whole set at once.**
- **ZwMapViewOfSection** — BUGCHECK 0x23 (fastfat `STUB_BUGCHECK` 0xdead0000): a distinct
  untranslated-fastfat-fn root (maps a file-backed section → fastfat), arm64-only because arm64
  boots the FAT32 ramdisk. Fix = translate the specific fastfat fn (separate lane).

The 3 baselined NO-END (HalPortIo, RtlCaptureContextKM, RtlCaptureContextUM) are all the same
harness limitation `support.c:105 error 0x7b` (INVALID_NAME — a helper binary not found by name),
matching the i386 baseline's RtlCaptureContext entries. Not kernel bugs.

**Parser hardened for baseline stability (`parse_kmtest_serial.py`).** The arm64 sweep exposed two
re-run artifacts that made the raw parse nondeterministic (a random 1-2 tests recorded
None/INCOMPLETE per run despite passing): (1) the sweep exe **re-autostarts** after finishing while
the guest keeps draining → a partial second run clobbered early tests — fixed by **breaking at the
first `done total=`** (the first complete sweep is authoritative; a mid-run BUGCHECK has no DONE so
CRASH handling is unaffected); (2) a **within-run transient re-launch** (e.g. the netstack-TDI
self-test perturbing timing) left an incomplete attempt beside a clean pass — fixed by a **dedup
that keeps the most-complete result per test name**. Both are general (no-ops for single-run x86).
With these, the baseline is stable and the ratchet self-compares green.

**Owed:** the design's ≥3-run stability validation (+ a `flaky.txt` for any residual flapper); the
netstack-TDI flake still costs ~1/6 boots as a launched=0 retry; and the two arm64 roots behind the
quarantines — task #10 (kernel-SEH, clears 8) and the fastfat stub (clears ZwMapViewOfSection) —
are the real fixes that ratchet these back in. Baseline is report-only (not gating) per the M4
design. The conformance verb build is unchanged; the kernel is identical to the landed user-APC fix.

### 2026-07-17 — INDEPENDENT-ORACLE build: kmtest drivers compiled with the clang `-windows-msvc` env (standard on amd64/arm64); i386 spike found the real kernel gap

**Motivation.** A conformance suite compiled by a *different* toolchain environment than the
OS-under-test is a better oracle — a blind spot shared by our clang-mingw toolchain can't hide a
bug from the tests. New knob: **`-DKMT_MSVC_ENV`** (default **ON**) retargets the 17 fault-and-catch
kmtest drivers to `--target=<arch>-pc-windows-msvc` (same clang binary, msvc environment) instead of
the project-default `-windows-gnu`. `kmt_enable_native_seh()` in `modules/rostests/kmtests/CMakeLists.txt`.

**amd64/arm64 — clean drop-in, now the standard conformance build:**
- Retargeted objects add **zero** new undefined refs vs mingw, keep the same `.pdata`/`.xdata`
  table-based SEH model + `__C_specific_handler`, need **no** `__security_cookie` glue. Both arches
  drop only `__imp_RtlAssert` (the `ASSERT` macro lowers differently under `_MSC_VER` — a genuine
  oracle signal for the IoReadWrite assertion-debt count, task #8).
- arm64 needed ONE trivial, provably-safe header fix: `sdk/include/vcruntime/mingw32/intrin_arm64.h`
  guarded `#ifndef __GNUC__ → #error`; widened to `#if !defined(__GNUC__) && !defined(__clang__)`.
  clang-mingw always defines `__GNUC__`, so the `#error` never fired for the normal build → no-op
  there; it only lets the msvc-env clang (which defines `__clang__`, not `__GNUC__`) through. The x86
  sibling shim was already clang-aware; arm/arm64/ppc still had the stale guard.
- End-to-end boot+sweep clean on both: amd64 `launched=87 ok=87`; arm64 `76 OK/3 NO-END` (== baseline).
  Conformance diff vs the mingw baseline is **identical except `IoDeviceObject`**, which wobbles **±1
  assertion in OPPOSITE directions** across the two arch runs (amd64 −1 pass, arm64 +1 pass) ⇒ a
  pre-existing **flaky** assertion in an already-partially-failing test, NOT a codegen divergence.
- **Why so quiet:** amd64/arm64 already use identical table-based SEH in BOTH clang envs, so there is
  no toolchain-shared blind spot for the independent build to expose there. The value was proving the
  mechanism end-to-end (compile → drop-in link → boot → run), which de-risked the i386 application.

**i386 SPIKE — toolchain WORKS, but the kernel-mode catch is a real gap (BUGCHECK 0x1E):**
Coupled flag **`-DKMT_I386_NATIVE_SEH`** (default **OFF**) — on i386, frame-based `__try/__except`
lowers ONLY under `-windows-msvc` (the gnu target errors `L__ehtable$_TestFastIoRead@32 can not be
undefined`), so native SEH is inseparable from the retarget; kept off so the standard i386 build stays
DUMMY_PSEH with its committed baseline.
- ✅ **Compiles** — `--target=i686-pc-windows-msvc` lowers the frame-based SEH the gnu target refuses
  (head-to-head confirmed: gnu → `L__ehtable$` errors, msvc → rc=0).
- ✅ **Links** — the lone runtime dep is `__except_handler3`, resolved from `sdk/lib/crt/wine/except_i386.c`.
  No `_except_handler4`, no `__security_cookie`.
- ✅ **Driver registers correctly** — disasm shows the real `fs:[0]` chain manipulation
  (`mov %fs:0x0,%ecx` / `mov %eax,%fs:0x0` link+unlink) + `__except_handler3` reference.
- ❌ **Runtime catch FAILS** — un-quarantining `IoReadWrite` (a standalone-driver fault-and-catch)
  and sweeping the retargeted i386 build gives `IoReadWrite CRASH … BUGCHECK=0x0000001E`: the
  deliberate fault escapes to the kernel instead of hitting the driver's now-real `__except`.
- **Where the gap is:** the i386 kernel dispatcher `RtlDispatchException`
  (`sdk/lib/rust/rtl-except/src/i386/except.rs`) is **fully implemented** — it walks
  `RtlpGetExceptionList()` (`fs:[0]`), iterates registration frames, invokes handlers. So the driver
  registers and the dispatcher exists, yet the catch doesn't happen. Remaining gap is one of: (a)
  the i386 trap/`KiDispatchException` path not routing a kernel-mode fault into this
  `RtlDispatchException`, (b) a kernel-mode `fs:[0]` exception-list mismatch (KPCR
  `NtTib.ExceptionList` vs what the dispatcher reads), or (c) wine's `_except_handler3` not being
  kernel-safe. (There's also a `RtlIsValidHandler` TODO at except.rs:154, but skipping validation
  would *permit* the handler, not block it.) **Pinning it = a GDB session on the IoReadWrite fault**
  (break `KiDispatchException`/`RtlDispatchException`; confirm it's entered, that
  `RtlpGetExceptionList()` returns the driver's record, and whether `__except_handler3` is called).
- **Net:** this converts the old vague "i386 SEH is MSVC-env-only, needs PSEH_CLANG" blocker into a
  *characterized, bounded kernel bug* — the toolchain half is solved and reproducible via
  `-DKMT_I386_NATIVE_SEH=ON`. Once the kernel-mode frame-based dispatch is fixed, the i386 DUMMY_PSEH
  structural quarantine class (IoReadWrite + the kmtest_drv-hosted set, once kmtest_drv is likewise
  retargeted) can ratchet in — the same payoff native SEH gave amd64/arm64.

### 2026-07-17 (evening) — §9.96 SLAIN: dropped `PAGE_NOACCESS` guard; MmSection + ZwMapViewOfSection ratchet back in on arm64

**The arm64 ZwMapViewOfSection translation-fault storm (and the MmSection "wedge" with it) is
FIXED.** Full writeup: field manual §9.96 (CLOSED). The short version: the Rust
`MmNotPresentFaultSectionView` dropped the C's `if (Region->Protect & PAGE_NOACCESS) return
STATUS_ACCESS_VIOLATION;` (ntoskrnl/mm/section.c:1640), so touching a NOACCESS section view fell
through to `MmCreateVirtualMapping(..., PAGE_NOACCESS, ...)` → protection-mask `MM_NOACCESS(0x18)`
→ `MmProtectToPteMask[0x18]==0` → PTE `pfn|0x441` = bits[1:0]=`0b01` = reserved-invalid at L3 →
CPU faults while bit0-only `MMPTE.valid()` says present → `MmIsPagePresent` early-out returns
SUCCESS → infinite re-fault (the GDB-captured storm). The malformed VALUE was the fingerprint that
named the culprit statically — no writer-hunt boot was needed. On x86 the same dropped guard
silently mapped NOACCESS views readable (wrong, non-fatal) — restored guard is C-faithful on all
arches; amd64 + i386 sweeps came back byte-identical to baseline. Also added a fail-loud tripwire
in arm64 `MmCreateVirtualMappingUnsafeEx`: composing `bits[1:0]==0b01` now bugchecks AT the store
with a `[PAGE] BUG:` banner instead of storming later.

**Ratchet state after the fix (arm64):** MmSection `OK 291/40/12` + ZwMapViewOfSection
`OK 120/14/1` un-quarantined and baselined (stable 4/4 sweeps). Collateral baseline ratchets, all
verified pre-existing or convergence (NOT fix regressions): ZwCreateSection `8/2→7/3`
(deterministic 4/4 — its `FILE_SUPERSEDE` of `kmtest-MmSection.txt` now finds MmSection's leftover
file, `FILE_SUPERSEDED` vs `FILE_CREATED`; identical counts to amd64/i386, where MmSection always
ran); ExPools `14/4→13/5` low-watermark (the `_WIN64` 2GB `ExAllocatePoolWithQuotaTag` is
borderline — flapped on 07-16 too); i386 FsRtlLegal `2504/24→2496/32` + IoDeviceObject
`185/4→184/5` low-watermarks (both flapped before the fix; 2496/32 = arm64's value).

**New exposure (registered, hunt owed):** with the section tests active, ~2/4 arm64 sweeps kill a
LATER kmtest_.exe launch — `0xC000001D` on garbage instruction bytes inside image page
`0x140007xxx` (IoReadWrite, NtSystemDebugControl) or `0xC0000005` at exit (IoDeviceObject). A
shared image-section page corrupted under heavy map/unmap = the latent arm64 PFN/section-teardown
family (same as the quarantined ZwAllocateVirtualMemory 0x4E). Roadmap registry entry added; the
section tests stay active as the canary. This is the next monster.

### 2026-07-24 — i386 pivot: the memcpy+0x40 cluster CLOSES on i386 via a Linux-style exception table, not SEH (field manual §9.123)

**i386 ratchet: 74 → 76 enabled, 23 → 21 quarantined.** `ObSymbolicLink` (OK 154/0) and
`ZwWaitForMultipleObjects` (OK 4/0) un-quarantined; new baseline written after **4
consecutive identical sweeps** (`launched=76 ok=72 bad=4 sweep=done`, the 4 being the
same pre-existing NO-ENDs, byte-identical to the old baseline's).

**What actually blocked i386, and what did not.** The 13 i386-only quarantines are not 13
bugs. Re-validating them against a current build (the recorded PCs were 12 days stale and
symbolicated against a long-gone binary) split them cleanly into two roots:

- **Root A — no fault-tolerant copy on i386** (task #5). `ObSymbolicLink`,
  `ZwWaitForMultipleObjects`, `IoReadWrite` all crashed at the *identical* kernel PC,
  `ntoskrnl.exe+0x14C0` = `memcpy+0x40`. The call sites were already routed through
  `safe_copy_*`; only the i386 *primitive* was a bare `copy_nonoverlapping`. **CLOSED
  today.**
- **Root B — driver-side frame-based SEH** (task #18). `ExHardError` and `ExResource`
  are the shape: the test deliberately passes a hostile pointer *expecting* the caller's
  `__except` to convert the AV to `STATUS_ACCESS_VIOLATION`
  (`ExInitializeResourceLite(NULL)` at ExResource.c:615; `ExRaiseHardError(..., mask=1,
  {1234 / NULL})` at ExHardError.c:162-163). Under i386 DUMMY_PSEH the driver registers
  no handler at all, so there is nothing for `RtlDispatchException` to find. Confirmed
  from source + disassembly, not inferred. **Still open.**

**The 2026-07-17 spike's oracle was mis-chosen.** It used `IoReadWrite` to characterise
task #18 — but `IoReadWrite`'s i386 crash was Root A, in the kernel's own copy, which
would fail there *regardless* of how well frame-based SEH worked. With `KMT_I386_NATIVE_SEH=ON`
it still crashed at `memcpy+0x40`. So the spike's (a)/(b)/(c) narrowing was reasoning
from a test that could not have passed either way. Post-fix, `IoReadWrite` crashes at
**`IoReadWrite_drv.sys+0x21E3`** — the fault finally left the kernel and landed on the
driver's own deliberate fault. *That* is the real task-#18 oracle; use it, not the old PC.

**The fix, and why it did not need SEH.** Both `kmsafecopy.c` and `probe.rs` had listed a
second close for years — "a Linux-extable-style fault-fixup primitive … it needs no
working SEH at all" — and nobody had costed it. It is ~40 lines
(`ntoskrnl-rust/src/ke/i386/safecopy.rs`): `KmProtectedCopyMemory` is `global_asm!` whose
copy is a single `rep movsb`, its address exported as the sole key of an exception table;
`KiTrap0EHandler` compares the faulting `Eip` and redirects to a fixup pad returning 0.
It runs *after* `MmAccessFault` declines, so it keeps amd64's exact semantics (nonresident
pages still page in; only genuinely bad addresses fix up), and it matches one exact PC so
it cannot swallow a fault it does not own. Boot oracle now covers all three arches:
`[KMSAFECOPY] fault-tolerant copy: valid=1 moved=1 invalid_caught=1 (PASS)` on i386.

**Collateral bug fixed** (found reading the same dispatch path, not the root): i386
`RtlpGetStackLimits` used a hand-written `[u8; 512]` for `FX_SAVE_AREA`, which is **528**
bytes — `StackHigh` was 16 bytes too high, so `RtlDispatchException`'s registration-frame
bounds check accepted a 16-byte window past the true top of stack. False-ACCEPT.

**Toolkit gap registered (task #28):** `qemu/run-until.sh --break KeBugCheckEx` does not
work on i386 — GDB reports the symbol undefined and the run then reports `BREAKPOINT NOT
HIT`, which is indistinguishable from "the breakpoint never fired." `qemu/addr2sym.sh` is
also unusable on i386 (matches absolute constant symbols). Symbolicate by hand meanwhile:
`VA = 0x400000 + RVA` from the banner's `P2 = ntoskrnl.exe+0xRVA`, against
`llvm-nm --numeric-sort build-i386/ntoskrnl/ntoskrnl.exe`.

**Next on i386:** task #18 is now the single root under the remaining i386-only
quarantines, with a clean oracle (`IoReadWrite` + `-DKMT_I386_NATIVE_SEH=ON`) and its
kernel-side confound removed.

### 2026-07-25 — the quarantine debt is SHAPED: amd64's list is a strict SUBSET of the other two

Before picking the next bug I ran set operations over the three per-arch sweep lists instead
of reading them one at a time. The result reprioritises the whole remaining quest:

| bucket | tests | entries | which |
|---|---|---|---|
| **shared core (all 3 arches)** | 10 | **30** | IoCreateFile, IoFilesystem, MmMapLockedPagesSpecifyCache, MmReservedMapping, NpfsFileInfo, NtCreateSection, RtlException, RtlExceptionKM, SeTokenFiltering, TcpIpConnect |
| i386-only | 9 | 9 | ExHardError, ExResource, IoReadWrite, KeDpc, KeMutex, MmSection, RtlIsValidOemCharacter, RtlSplayTreeKM, ZwMapViewOfSection |
| i386+arm64, **passes on amd64** | 2 | 4 | SeInheritance, ZwAllocateVirtualMemory |
| arm64-only | 1 | 1 | ExFastMutex |
| | | **44** | |

**There are ZERO amd64-only quarantines.** amd64's entire list is the shared core, so the
three arches are nested, not merely similar.

Two consequences:

1. **68% of the debt is ten tests, and each shared-core fix retires THREE entries.** The
   instinct to attack i386 because it has the most quarantines (21) is backwards: those 9
   i386-only entries pay 1x each, while the core pays 3x. Work the trunk before the tail.
2. **`SeInheritance` and `ZwAllocateVirtualMemory` are quarantined on exactly the two arches
   that had exception-path bugs, and pass on the one that did not.** Both of those bugs (arm64
   kernel-SEH task #10, i386's missing fault-tolerant copy §9.123) are now FIXED. That is a
   falsifiable prediction, not a vibe, and it costs one boot to test.

**Two of the ten are not bugs at all.** `RtlException` and `RtlExceptionKM` are annotated
in-list as expected-fail — "calls-only (no `-fasync-exceptions`), hardware faults in the
guarded body escape … not an OS bug" and "crashes (exit 128) WITHOUT a kernel BUGCHECK and the
sweep continues clean — the good harness-robustness outcome". They will never go green and
they are 6 of the 44 entries. Counting them as debt overstates the real figure (38) and
guarantees the ratchet can never reach zero. They want a separate expected-fail list.

**Re-testing is now mechanised** (`tools/conformance_probe.py`). The reason quarantines rot is
that a bugchecking test truncates the sweep, so enabling all 21 at once yields one verdict; the
tool instead boots candidate-only lists ordered most-likely-fixed-first, harvests every verdict
up to the crasher, drops it and repeats. Worst case one boot per crasher, best case one boot.
It restores the committed list on every exit path, Ctrl-C included.

### 2026-07-25 (later) — the i386-only tail has ONE root, and it is task #18

§9.137 unblocked offline i386 symbolication, so the i386 quarantines stopped being opaque. Two
fresh crashes were traced the same night:

| test | fresh PC | resolves to | root |
|---|---|---|---|
| `RtlIsValidOemCharacter` | `0x80529538` | `_RtlIsValidOemCharacter@4+8` (`rtl-strings/src/cabi/mod.rs:1939`) | **task #18** |
| `SeInheritance` | `0x805510dd` | `_SeAssignSecurityEx@36+61` (`se-rust/src/sd.rs:923`) | distinct, open |

`RtlIsValidOemCharacter` is conclusive. Its test ends with

```c
_SEH2_TRY { RtlIsValidOemCharacter(NULL); }
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status = _SEH2_GetExceptionCode(); }
```

— a **deliberate** NULL deref the `__except` is meant to catch. `+8` is `let mut c = *ch;`, the
deref of that NULL parameter. amd64/arm64 catch it with native SEH; i386 under `DUMMY_PSEH` has a
dead `__try`, so it escapes to `BUGCHECK 0x1E`. Nothing is wrong with the Rust.

**That generalises, and it re-scopes task #18 from one test to the trunk of the i386 tail.**
Classifying every quarantined test by whether its source uses `_SEH2_TRY`:

| i386-ONLY quarantine | `_SEH2_TRY` | reading |
|---|---|---|
| `RtlIsValidOemCharacter` | 1 | task #18 — confirmed |
| `ExHardError` | 2 | task #18 candidate |
| `ExResource` | 1 | task #18 candidate |
| `KeDpc` | 2 | task #18 — **confirmed** |
| `MmSection` | 2 | task #18 candidate |
| `KeMutex` | 0 | distinct bug |
| `ZwMapViewOfSection` | 0 | distinct bug |

So **~5 of the 9 i386-only quarantines are one root** (2 confirmed, 3 by signature), and task #18 is the highest-value i386
fix by a wide margin — it is worth roughly as much as the entire rest of the i386-only tail.

**The discriminator needs both halves.** `IoFilesystem` and `RtlException` also use `_SEH2_TRY`,
but they are quarantined on ALL THREE arches — including the ones where SEH works — so their
failures cannot be the dead-`__try` root. The signature is *i386-only* **and** *uses SEH*. Using
SEH-usage alone would have swept two shared-core bugs into the wrong bucket and made task #18
look like it fixed more than it does.

**Cost of this classification: one grep.** It was available on 2026-07-12 when these nine tests
were quarantined with "triage owed". The blocker was never the difficulty — it was the recorded
claim that triage needed a debugger (§9.137).

**Second confirmation (same night).** `KeDpc` crashed at `_KeInitializeDpc@12+17`
(`ke/dpc.rs:788`) and its test is unambiguous:

```c
_SEH2_TRY { KeInitializeDpc(NULL, NULL, NULL); }
_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { Status = _SEH2_GetExceptionCode(); }
ok_eq_hex(Status, STATUS_ACCESS_VIOLATION);
```

The test **asserts that it receives `STATUS_ACCESS_VIOLATION`** — the fault is the point of the
test, and `+17` is the deref of the NULL `Dpc`. Two candidates predicted from the grep, two
confirmed on inspection. Note what this implies for the ratchet: these tests cannot pass on i386
by fixing kernel code, because the kernel is behaving correctly. They pass when `__try` works.

### 2026-07-25 (later still) — a SHARED quarantine is not a SHARED root; the 3x estimate is an upper bound

The structural finding earlier today — amd64's quarantine list is a strict subset of the other
two, so 10 tests account for 30 of the 44 entries — is still true as *bookkeeping*. But the
inference drawn from it ("each shared-core fix retires three entries") is an **upper bound**, and
the first two shared-core tests probed both undercut it:

| shared-core test | i386 | arm64 |
|---|---|---|
| `IoCreateFile` | **OK** `pass=45 fail=8 skip=1 exit=0` | **CRASH** `0x1E` (AV @ `0xfffff800421c960c`) |
| `IoFilesystem` | **CRASH** `_ExFreePoolWithTag@8+829` (pool) | untested |
| `NtCreateSection` | CRASH `0x50`, PC in **driver** range | CRASH `0x50`, PC in **kernel** range |

`IoCreateFile` is the clean counterexample: quarantined on all three arches, yet it now passes on
i386 while still crashing on arm64. Two arches, two different states, one quarantine entry each —
so "quarantined everywhere" recorded three independent observations that happened to agree at the
time, not one root cause.

`NtCreateSection` is the ambiguous middle: same stop code and same read-fault shape on both
arches, which is suggestive, but the faulting PC is in the kmtest **driver** on i386 and in the
**kernel** on arm64. That is either one bug with two faces or two bugs, and the honest answer
today is that we do not know.

**How to use the subset finding, then.** It is a good *prioritisation* heuristic — the shared
core is still where the entries are concentrated, and a shared root, where one exists, does pay
3x. It is not evidence that a root is shared. Establish that per test, from per-arch verdicts,
before budgeting a fix as a triple win. The cost of getting this backwards is planning a
"clears 6 entries" fix that clears two.

**Corollary for un-quarantining.** `IoCreateFile` passing on i386 is NOT sufficient to
un-quarantine it there yet. `SeInheritance` was flipped because amd64's baseline supplied a
control (`704433/14` vs the observed `704431/16`); `IoCreateFile` is quarantined on every arch,
so no baseline exists to compare against, and a single clean run cannot rule out the
intermittency that would poison the whole i386 sweep. It wants 2-3 confirming runs.

### 2026-07-25 — in fairness: some quarantine notes were exactly right

Two entries today argued that recorded claims decay (§9.137's "symbol-poor" premise; the stale
offsets). That is a real failure mode, but it is not the general case and the record should say
so, because "distrust the notes" is as expensive a heuristic as "trust them".

`RtlSplayTreeKM` re-probed today and reproduced its 2026-07-12 note **exactly**:

```
# QUARANTINED 2026-07-12: BUGCHECK 0x19 BAD_POOL_HEADER (not an AV — pool corruption on
# free). Passes on amd64. Exercises the `splaytree` Rust crate — REAL-BUG candidate
# (i386-specific splay corruption), not DUMMY_PSEH. Repro: SPECIALPOOL=* ...
```

Observed 13 days later: `*** BUGCHECK: 0x00000019 (0x00000005, 0xb0fbb7a8, 0xea, 0xb0fbb780)`.
Same stop code, same class. The note also got the *classification* right in advance — it
explicitly ruled out DUMMY_PSEH, which today's independent SEH-usage discriminator agrees with,
and it named the correct next tool (special pool, since a pool-corruption bugcheck fires at the
victim rather than the culprit — §9.18).

**What actually distinguishes the durable notes from the rotten ones.** The notes that held up
record a *stop code and a mechanism* (`0x19 BAD_POOL_HEADER`, "pool corruption on free",
"exercises the splaytree crate"). The ones that rotted record a *coordinate or a tool claim*
(`ntoskrnl.exe+0x8E3BD`; "offline index is symbol-poor"). Mechanism survives rebuilds; addresses
and tooling assertions do not.

So: when quarantining, write down the stop code, the subsystem and what you ruled out. Do not
lean on an offset, and never record a tool limitation you have not just verified.

For the record, `MmReservedMapping` is the middle case — its note is a genuine mechanism
(`0x101`, pool tag `'KmRM'`, `MmAllocateMappingAddress` path) but today it faults `0x1E` inside
the driver instead, so the mechanism itself moved. Worth re-triaging from scratch rather than
from the note.

### 2026-07-25 — arm64 `SeTokenFiltering` kills the VM SILENTLY, and I nearly discarded the evidence

**Signature.** Two independent probe rounds, same shape:

```
[KMTEST:BEGIN SeTokenFiltering]
(\modules\rostests\kmtests\kmtest_drv\kmtest_drv.c:377) DriverIoControl. Starting test SeTokenFiltering
<serial goes completely silent — no further output, ever>
```

No `[KMTEST:END]`, **no BUGCHECK**, and the QEMU process is *gone* — `vibeos-cli ps` shows zero
QEMUs host-wide while the harness keeps draining a dead VM for its full 5400s budget (task #34).
The parser records `SeTokenFiltering INCOMPLETE`.

**This is worse than a bugcheck, and easy to mis-file.** A bugcheck leaves a stop code, parameters
and a PC. This leaves nothing — the machine simply stops existing. Any triage has to start from
"which test was running", which is only knowable because the sweep announces `BEGIN` before each
test.

**The methodological mistake, recorded because it cut both ways.** On the first occurrence I
unwedged the harness by killing the boot pid, then filed the result as *"CONTAMINATED — my
artifact, not a real failure; must be re-tested; do NOT record it as a crasher."* That was wrong.
The wedge was already in place before I touched anything (serial 29 minutes stale, zero QEMUs);
my kill only ended a drain that had no VM behind it. The second round then reproduced the whole
thing with no intervention at all.

So the caution that correctly caught four *invented* crashers (the probe's attribution bug) also
threw away one *real* one. Being too skeptical is not free — it just moves the error. The
discriminating question is not "did I touch it?" but **"was the failure already determined before
I touched it?"**, and the serial timestamps answered that in one command.

**Hypothesis for the silent death** (untested): the guest reaches a clean VM shutdown rather than
crashing — on arm64 a PSCI `SYSTEM_OFF`/reset terminates QEMU with no diagnostic, which fits
"serial stops, process exits, no bugcheck" exactly. A reset path reached from token filtering
would explain it. Worth confirming with `-no-reboot`/`-no-shutdown` plus QEMU exit-code capture
before hunting in Se.

**Practical:** exclude `SeTokenFiltering` from arm64 probe lists the way `TcpIpConnect` is
excluded — both burn the full drain budget instead of failing fast, and one wedge can consume a
whole probe run.

### 2026-07-25 (later) — amd64 fully re-probed: 9/9 still broken, and the debt is a strict CHAIN

The amd64 probe owed since the tool was written is **done, and it is a clean negative**: every
one of the 9 amd64 quarantines (excluding `TcpIpConnect`, a documented hang) still fails. Zero
un-quarantine candidates. Each test was booted **alone**, one per boot, so no verdict here
depends on any other test's behaviour.

| test | verdict | stop code | note recorded | |
|---|---|---|---|---|
| `NtCreateSection` | CRASH | `0x50` | `0x50` | ✓ |
| `MmMapLockedPagesSpecifyCache` | CRASH | `0x1A` | `0x1000` PFN_LIST_CORRUPT P1=0x401 | **drifted** |
| `IoCreateFile` | CRASH | `0x1E` | `0x1E` | ✓ |
| `IoFilesystem` | CRASH | `0x19` | `0x1E` AV `MiFindEmptyAddressRangeInTree+0x173` | **drifted** |
| `SeTokenFiltering` | TIMEOUT 234s | — | TIMEOUT | ✓ |
| `MmReservedMapping` | CRASH | `0xDA` | `0xDA` SYSTEM_PTE_MISUSE | ✓ |
| `NpfsFileInfo` | CRASH | `0x19` | `0x19` BAD_POOL_HEADER | ✓ |
| `RtlException` | NO-END exit=128 | — | NO-END exit=128 | ✓ |
| `RtlExceptionKM` | CRASH | `0x1E` | `0x1E` | ✓ |

**The quarantine sets are strictly nested: amd64 ⊂ arm64 ⊂ i386.** arm64 is amd64 plus exactly
one test (`ZwAllocateVirtualMemory`); i386 is amd64 plus eleven. *Nothing is amd64-only.* That
makes amd64's list the **shared core** — the arch-independent bugs, where one fix moves all three
arches — and it is the highest-leverage list to work from. It also means a clean amd64 probe
result is the strongest possible evidence that there are no free un-quarantines anywhere: these
same tests fail on the other two arches too.

**On note durability — the fair version.** 7 of 9 notes reproduced their recorded stop code
*exactly*, 15 days later. My earlier "notes decay" framing was too broad. The pattern is sharper
than that: **a note that records a stop code and a mechanism survives; a note that records a
symbol offset or a source line does not.** Both drifted entries misdirect concretely —
`IoFilesystem`'s sent me reading `ARM3/vadnode.rs:370` for a bug that now presents as pool
corruption, and `MmMapLockedPagesSpecifyCache`'s `0x1000` reading is the premise task #7 is built
on, which needs re-checking against `0x1A` before more time goes into it.

**A convergence worth chasing:** `IoFilesystem` and `NpfsFileInfo` now stop with the *same*
`0x19` BAD_POOL_HEADER. Two tests, one stop code — plausibly ONE pool-corruption root rather than
two bugs. The next move on both is `--tripwire "SPECIALPOOL=*"` to fault at the corrupting store,
not more source reading.

**The tool bug this exposed** (fixed, `7a10deea964`). Round 1 ran `SeTokenFiltering` ahead of four
candidates and reported all four as validly re-tested. They were not: a TIMEOUT is an in-kernel
wedge that does *not* end the boot, and the wedged process keeps `kmtest_drv`'s result buffer
registered, so every later test cascade-fails `SET_RESULTBUFFER 0x5` and records a NO-END verdict
about itself that is really a fact about the wedge. Because those look like verdicts, the
"nothing missing → we're done" early-out accepted them. Re-probed with the wedge excluded, all
four had real verdicts of their own.

That is the **second** attribution bug in this tool, and the same mistake in a new costume:
crediting a test with a verdict that says nothing about it. The first one invented four crashers
out of tests that never executed; this one invented four clean re-tests. Both directions are now
covered by controls (`tools/test_conformance_probe.py`) over a pure `split_at_wedge`.

Note the asymmetry that makes NO-END deliberately *not* poisoning: it is how a usermode test
self-crashes without taking the kernel down. The final boot proves the point — `RtlException`
NO-END was followed by `RtlExceptionKM` earning a genuine CRASH, which a broader rule would have
discarded.

**Twice today I read "no QEMUs running" as a wedge and was wrong both times.** `conformance`
front-loads a staging *build* before it boots, so a probe can sit for 15+ minutes holding a
`cpuheavy` lease with no VM in sight — especially with the build broker at 4/4 and a merge-queue
conductor gating. The check that actually discriminates is `host-status` (does it hold a build
lease?) plus whether the run dir has a `full.log` yet — not `vibeos-cli ps` alone.

### 2026-07-25 (evening) — the biggest single movement yet: clang was DELETING our `__except` blocks

**15 quarantines cleared across three arches** (i386 +13, amd64 +2, arm64 +2), from one
toolchain defect that had been misfiled as "not an OS bug" for two weeks.

**What was actually wrong.** Clang's default exception model is CALLS-ONLY: it assumes no
instruction can fault, so a `__try` whose body holds a faulting *instruction* rather than a
*call* gets an EMPTY scope table — and at `-O2` the `__except` block is then deleted outright
as unreachable. The whole of

```c
__try { *(volatile int*)0 = 1; } __except(1){ return 2; }
```

compiles, on our pinned clang 19.1.7, to `movl $1, 0 / xorl %eax,%eax / retq`. **The handler is
not weak or mis-registered. It is gone.** So the AV escaped to the kernel as BUGCHECK 0x1E —
precisely the quarantined symptom. `-fasync-exceptions` restores a real scope table
(`__C_specific_handler` on x64/arm64, `__except_handler3` + `fs:[0]` on i686). It is a no-op
outside `-windows-msvc`, so it only bites where we retarget, and it is purely additive:
callee-faults were covered either way, which is why anything worked at all.

**The note that named its own fix.** `RtlExceptionKM`'s quarantine read: *"mingw native SEH is
calls-only (no -fasync-exceptions) … Expected-fail until async-exceptions story exists; not an
OS bug."* It identified the exact missing flag and concluded it was out of reach. It was two
lines of CMake. **A correct diagnosis with a defeatist conclusion is worse than no diagnosis** —
it closes the question. Prefer "untested hypothesis: try X" over "expected-fail until X exists".

**i386 was the jackpot** (76 → 89 enabled). `KMT_I386_NATIVE_SEH` had sat as an OFF-by-default
spike "until validated end-to-end". Validating it took one afternoon and one shim: retargeting
to `-windows-msvc` left exactly ONE undefined symbol, `_InterlockedAddLargeStatistic`, because
mingw supplies it as an `__INTRIN_INLINE` from a *gnu*-selected header. `_except_handler3`
needed nothing — it resolves from `libpseh.a`'s `i386/seh.s`, a real implementation, and pseh
precedes ntoskrnl on the link line.

**The evidence that this is a fix and not a coincidence** — recovered i386 numbers match the
other arches' committed baselines test-for-test:

| test | i386 (recovered) | other arch |
|---|---|---|
| `KeMutex` | 1122/0 | 1122/0 arm64 |
| `ExResource` | 399/2 | 399/2 arm64 |
| `ExHardError` | 24/6 | 24/6 arm64 |
| `KeDpc` | 316/0 | 316/0 arm64 |
| `RtlIsValidOemCharacter` | 354/0 | 354/0 arm64 |
| `IoReadWrite` | 16192/151 | 16192/151 arm64 |
| `NtCreateSection` | 124/38 | 124/38 amd64 |

A dead `__except` does not fail *selectively*. Restore the handler and these land exactly where
the arches that always had real SEH have been reporting all along.

**Two real OS bugs fell out**, both previously hidden *behind* the quarantine — which is the
whole argument for enabling a test that still has failures:

1. **Usermode stack guard-page fault reports the wrong status** (task #38). `RtlException`'s
   single remaining failure on arm64 is the one assertion `RtlExceptionKM` lacks
   (`#ifdef KMT_USER_MODE`): overflow the stack with `_alloca` and expect
   `STATUS_STACK_OVERFLOW`. We deliver `0xc0000005`, not `0xc00000fd`. The 33/33 shared
   assertions passing on both is what isolates it to that one line.
2. **amd64 usermode exception DISPATCH** (task #37). `RtlException` completes on arm64 but dies
   `exit=128` on amd64 and i386. Codegen is ruled out — the scope table is provably present —
   and `kmtest_.exe` is healthy in the same boot. arm64 is the working oracle to diff against.

**A correction to my own result from this morning.** Commit `8ed5287039b` reported "amd64 9/9
still fail, ZERO un-quarantine candidates." That probe ran *before* this fix; `NtCreateSection`
has since flipped to OK. The conclusion was true of the build it measured and wrong to carry
forward — the same staleness trap that commit warned about in *other people's* notes. **A probe
result is a fact about a BUILD, not about a test.** Re-probe after any fix that plausibly
touches the mechanism, and date the claim.

**What remains is now cleanly separated** — no SEH left in it:

* `MmReservedMapping`, `MmMapLockedPagesSpecifyCache`, `IoFilesystem`, `NpfsFileInfo` crash on
  every arch — genuine Mm/pool bugs. `IoFilesystem` and `NpfsFileInfo` share stop code 0x19
  BAD_POOL_HEADER (task #36).
* `RtlSplayTreeKM` (i386) — pool corruption, as its note always said.
* `IoCreateFile` — now passes on i386, still crashes on amd64 and arm64.
* `ZwAllocateVirtualMemory` — now passes on i386, still crashes on arm64.
* `RtlException` (task #37), `SeTokenFiltering` / `TcpIpConnect` (in-kernel wedges).

### 2026-07-25 (night) — a false-RED gate: probe runs leave the PAYLOAD in the boot media

The gate failed `smoke:arm64` with BUGCHECK 0x19, **deterministically, 3/3** — on a branch
whose entire diff was kmtest build files, conformance lists, and docs. Nothing in the boot path.

**It was not a code regression. The arm64 boot media was still running my last probe's kmtest
sweep.** The serial log says so outright:

```
[EXPLORER] startup: runCmd OK (PID 340): X:\...\StartUp\KMTEST~1.EXE
[KMTEST-SWEEP] start list=X:\reactos\bin\kmtest-tests.txt
[KMTEST-SWEEP] launch NpfsFileInfo        <- the probe's leftover 1-test list
```

`NpfsFileInfo` is a known 0x19 BAD_POOL_HEADER crasher and was the last candidate of the last
arm64 probe. So a *plain* smoke booted it and bugchecked, every time.

**Mechanism.** `vibeos-cli conformance` builds with `VIBEOS_PAYLOAD=kmtest`, which grafts the
sweep into the All Users StartUp folder and stages `kmtest-tests.txt` into the image. The
payload *stamp* (`build-<arch>/payload-selection.json`) is correctly cleared back to
`{"payloads": []}` by a later plain build — the 2026-07-07 doom-clobber guard works. But
clearing the selection does not **remove** what a previous payload build already installed, and
`mkramdisk.sh` has a fast path that skips image re-creation. So `qemu/ramdisk.img` /
`qemu/boot.img` keep the payload indefinitely. Filed as task #39.

**Getting to that answer required a real control, and my reasoning-from-the-diff was not
enough.** I was confident the failure could not be mine — the change surface genuinely does not
touch the kernel — and I was *wrong about the conclusion* even though the reasoning was sound,
because the contamination came from the probe TOOL rather than the committed diff. What settled
it in both directions:

* a worktree at plain `origin/master`, built and smoked: **3/3 PASS** → it *is* my tree;
* deleting `qemu/{boot,ramdisk}.img` and rebuilding **my** tree: **3/3 PASS**, zero KMTEST in
  the log → it is not my *code*.

Two runs, opposite conclusions, and only together do they identify the culprit. "My diff can't
have caused this" is a hypothesis, not a result — and the state a tool leaves behind is part of
your tree even when it is not part of your diff.

**Practical, until #39 lands:** after any `conformance`/probe run on an arch, delete
`qemu/boot.img` and `qemu/ramdisk.img` (both gitignored, both regenerated) before trusting a
plain smoke or gate on that arch. Watch for `[KMTEST-SWEEP]` in a smoke serial log — in a plain
smoke it should never appear.

---

### 2026-07-26 — the 0x19 pair falls to ONE root, and arm64 uncovers a second bug behind it

**`IoFilesystem` + `NpfsFileInfo` shared a single root, and it was not memory corruption in the
usual sense.** `NtQueryInformationFile` (`iomgr-rust/src/iofunc.rs`) validated the info class and
the caller's `Length` only under `if (PreviousMode != KernelMode)`. The C guards a kernel-mode
copy of those two checks behind `#if DBG`, and the translation dropped that arm. `Length` sizes
the `TAG_IOBUF` system buffer, so a kernel `FileAllInformation` query with `Length=0` got a
0-byte pool block; the I/O manager then wrote Access/Mode/AlignmentInformation into it and the
FSD wrote a `FILE_BASIC_INFORMATION` on top, smashing the next block's header with a FILETIME
(the observed `0x01dd1ca3`) → **0x19 subcode 5 POOL_SIZE_OR_INDEX_MISMATCH**.

The decisive control was *internal, not experimental*: our own `NtSetInformationFile` validates
unconditionally, and the C confirms that asymmetry is deliberate upstream — the setter's
kernel-mode branch is a plain `else`, and the query's `#if DBG` is the only one in the entire
file. Two siblings disagreeing about the same invariant is a strong bug smell; chase it before
building repro harnesses.

Result, and note the shape of the evidence:

| arch | IoFilesystem | NpfsFileInfo |
|---|---|---|
| i386 | OK pass=162 fail=142 skip=1 | OK pass=50 fail=25 |
| amd64 | OK pass=162 fail=142 skip=1 | OK pass=50 fail=25 |
| arm64 | still CRASH — different bug, below | OK |

**i386 and amd64 landed byte-identical.** That is the signature a shared-core fix should
produce, and it is much stronger evidence than either arch alone: a coincidence does not
reproduce a 142 *and* a 25 simultaneously on two independent architectures.

**The VAD-tree lead in the 2026-07-10 entries above was stale on every arch** — nobody re-probed
it after the stop code moved, and it cost at least one reader a wasted trip through
`MiFindEmptyAddressRangeInTree`. The list notes now carry the real cause.

**Then arm64 revealed what the pool bug had been hiding.** With 0x19 fixed, `IoFilesystem` got
further and stopped `0x23 (0xdead0000)` — `FASTFAT_STUB_HIT: FatGenerateFileIdFromFcb`. That is
a *macro* (`fatprocs.h:3067`), never a function: the translation emitted an `extern fn` for it,
no symbol existed, and the cohab stub generator "resolved" it into `KeBugCheckEx`. Fixed (with
its enumeration twin, and `SizeOfPackedEa` of the same class) in `fastfat-rust/src/fileid.rs`.
The **next** boot then stopped in `FatOpenVolume` — this one a genuine ~290-line untranslated
function (`create.c:2292`), now hand-translated into `create.rs`.

**Only arm64 exercises fastfat at all** (FAT32 ramdisk = its live system volume; i386/amd64 boot
a CDFS ISO). So an "arm64-only" filesystem crash here is usually *coverage*, not an arch bug.

**The stub queue is computable, and we should stop paying one boot per entry.** Intersecting
`STUB_BUGCHECK` bodies in `rust_cohab_stub.c` with names any `*-rust` module declares gives
"unimplemented AND reachable": fastfat has **19, all reachable**, mostly FSCTL handlers — i.e.
precisely what a filesystem conformance test drives next. Task #45 tracks mechanizing that as a
build-time warning.

**Two tools that were supposed to catch this were themselves broken**, which is the part worth
remembering:

* **Special pool never engaged.** `ARM3/special.rs` had a local
  `const POOL_FLAG_SPECIAL_POOL = 0x10` (that value is `POOL_FLAG_CHECK_DEADLOCK`) silently
  shadowing the glob-imported real `0x20`. The tripwire armed, initialised, printed `ACTIVE`,
  and routed **zero** allocations. Worse: `scan_const_contradictions` had already *found* this
  contradiction — it was sitting in the lint baseline, and nobody traced the consequence.
  Baselining a contradiction about a flag that gates a diagnostic disables the diagnostic. The
  tripwire now allocates a probe through `ExAllocatePoolWithTag` and only claims
  `ACTIVE (routing verified)` if `MmIsSpecialPoolAddress` agrees.
* **The conformance run dir kept no serial log** — it pointed at the *live* per-arch path, which
  the next boot of that arch truncates. A 0x19 with a full `[POOL]` page dump was destroyed that
  way before it could be read. `artifacts.archive_serial()` now snapshots it into the run dir.

Both were fixed *before* the hunt could finish, and both had been silently useless for some
time. **When a diagnostic reports itself healthy, check that it exercises its own output path.**

---

### 2026-09-05 — the ratchet gated on COUNTS, and 33 of 37 shared count-signatures hide a different result (#78)

**The defect was in this document's own design.** "Only the per-test `fail` count, plus the
named assertions behind it, means anything" was written in the M4 section from the start; only
the count half was ever implemented. `compare()` gated on `pass` dropping and `fail` growing,
so the one thing counts cannot express — **fix three assertions, break three others** — passed
the gate in silence.

That is not hypothetical, and the size of it is measurable. 596 complete arm64 sweeps were
sitting in `/tmp/vibeos-cli` from the smp lane's soak:

| | |
|---|---|
| distinct failing assertion identities ever seen | 336 |
| fail in EVERY one of 596 runs (the real floor) | 277 |
| intermittent | 59 |
| distinct whole-run count-signatures | 92 |
| count-signatures shared by >1 run | 37 |
| **…of those, where the failing ASSERTIONS still differ** | **33** |

The worst case is **10 runs with byte-identical pass/fail/skip for all 91 tests** that differ
in 6 assertion identities. The count ratchet accepts every one as the same result. Replayed
through the new comparison, the same pair reports
`IoDeviceObject: NEW failing assertion IoDeviceObject_drv.c:206`.

**92 count-signatures across 596 runs also explains the re-seeding.** A count floor can only
equal one of those 92 faces, so honest runs read as regressions and get re-seeded away — and
each re-seed is a chance to bank a lucky boot. `--only` was built to contain that damage; the
identity floor removes most of the reason to re-seed at all.

**What the identity is.** kmt_test.h's `KmtVOk` prints `<file>:<line>: Test failed: <message>`
per failing assertion, and nothing at all per success (`if (0/*KmtReportSuccess*/)`). So
`file:line` is the identity and the message is not — it carries addresses and NTSTATUS values
that differ every boot. The absence of success lines is also why a **dropped `pass` count stays
count-gated**: there are no per-success identities to compare, and a test that quietly stops
asserting shows up nowhere else. A `fail` count that grew without a new identity is a loop
running longer, and is no longer reported.

**Bistability is marked, not re-seeded away.** 59 of the 336 identities are intermittent, with
frequencies from 594/596 down to 1/596 — `IoDeviceObject_drv.c:198/206/300` and
`ExPools.c:176/178/181` at the top, which is exactly the `IoDeviceObject` / `ExPools` pair this
document already named as bistable, found independently from the logs. Those get `assert?`:
silent whichever way they land, and carried across a re-seed by `_flaky_of` so one boot's face
cannot become the floor.

**Attribution is by LAUNCH, not by END.** A crashed or timed-out test never emits
`[KMTEST:END]`, so attributing failures to the next END bleeds the dead test's assertions into
whichever test ends after it. The soak shows this happening: `ExPools.c` lines surface under
`ExSequencedList` on exactly the runs where the test ahead died.

**The soak was NOT used to seed anything.** Its 610 rounds ran on `smp-bringup`
(`3878df2735d`, `129e388f59d`) — neither is an ancestor of this lane, so it is another lane's
kernel. It is sound evidence about the HARNESS (flake structure, count-blindness are properties
of the corpus and the comparison, not of one lane's kernel) and worthless as a floor for ours.
Seeding is owed, per arch, from `--write-baseline` on our own tree at a known commit; until
then every row keeps its count gate and nothing regresses.

### 2026-09-06 — the ratchet is WIRED INTO THE GATE, and stops being advisory

For its whole life the ratchet computed a correct verdict and nothing was obliged to
act on it. `--gate` existed, and no gate passed it. That is now the default:
`vibeos-cli gate` runs one conformance lane per arch, and a ratchet regression withholds
the push stamp exactly as a dead desktop does. `--no-conformance` opts out.

**Why it reuses the gate's media instead of building its own.** The sweep needs media
built with `VIBEOS_PAYLOAD=kmtest`, which the gate's build did not stamp — a lane running
`--no-build` against a `<none>`-stamped build dir boots fine and reports
`[KMTEST-SWEEP] launch FAILED` minutes later, a layer from the cause. The two options were
a second build per arch, or teaching the gate's build to carry the payload. Measured on a
live fleet, that is not a close call: the host broker sat at **4/4 build permits with lanes
queued 12+ minutes** while the boot pool sat at **1/8**. Builds are the bottleneck; boots
are nearly free. So the gate build now passes `--kmtests` and stamps the payload, and the
conformance lanes boot the same media the smoke does. Cost: three extra parallel boots,
zero extra builds.

**Why the lane is flake-retryable.** This is the part that decides whether the gate is
usable at all. The floor marks the bistables it KNOWS about (`assert?`), but the corpus is
not closed: an intermittent that simply did not fire during the 15 seeding boots is
indistinguishable from a real regression on any single sweep. The merge queue gates ONCE
for N lanes, so a non-retryable miss would reject a whole batch of innocent lanes on a coin
flip — precisely the failure the 2026-08-08 NET_FLAKE incident recorded, where a lane whose
entire diff was three baseline edits and a doc got rejected. So a first-attempt miss is
`CONFORMANCE_RED` (renamed from `CONFORMANCE_FLAKE` 2026-09-26: the label claimed a cause
nobody had established) and retries; a real regression reproduces and ends red. A repeated
experiment that holds the suspected cause fixed falsifies nothing, and the converse applies
here: an experiment that does not repeat has not established anything either.

**What it is still not.** The floor is 702 identities over 91 rows per arch, seeded from 15
boots. It gates what those boots saw. An assertion that fails only under a load the gate
does not produce is not on it, and the retry that protects innocent lanes will also mask a
genuine regression that reproduces less than every other run. The ratchet is a floor, not a
proof.

### 2026-09-06 (same day) — the retry earned its keep on its FIRST real gate, and found a new arm64 bistable

The first gate to run the wired-in ratchet went green with one line under it:
`conformance:arm64 — PASS, 3 attempts, recovered-after-flake`. i386 and amd64 held on the
first sweep; arm64 reported

    ExPools: NEW failing assertion ExPools.c:176
    ExPools: NEW failing assertion ExPools.c:181

failed the same way on the first retry, and held clean on the second.

**This is the case the retry was built for, arriving immediately.** Had the lane been
non-retryable — the obvious first implementation — the very first gate would have been RED,
and in the merge queue it would have rejected every innocent lane in the batch alongside it.
The design note written that morning ("an intermittent that never fired during the 15 seeding
boots is indistinguishable from a regression on any single sweep") was not hypothetical for
even one gate.

**Both identities are now marked `assert?` on the arm64 floor** (bistables there: 4 → 6). The
alternative was leaving every arm64 gate with a real chance of burning all its retries, which
blocks the fleet — a worse failure than absorbing two known-intermittent lines.

**What that mark does NOT settle, and is owed.** `ExPools.c:176` and `:181` sit either side of
`:178`, which is on the floor as a STABLE failure. A cluster of three neighbouring assertions
where one always fails and two fail about two runs in three is not noise-shaped; it looks like
one intermittent defect in arm64 pool allocation, and the `assert?` marks silence the symptom
rather than explaining it. Hit rate observed here is 2/3, which is frequent enough to soak
cheaply: `vibeos-cli conformance --arch arm64 --repeat 20` and read which of the three fire
together. Nothing else in the tree is watching this now that the ratchet is quiet about it.

**DISCHARGED 2026-09-13 — and the soak was not needed, because reading which fire together was
the whole answer.** Four arm64 sweeps were enough: the three lines are not three symptoms but
one, and `:178` is anti-correlated with `:176` by construction rather than intermittent. The
guess above ("one intermittent defect in arm64 pool allocation") was right; the framing that
there were three neighbouring flaky assertions was not. Full root cause, with the serial values
that prove the `USHORT` underflow, in the roadmap's `ExPools` registry entry and in the field
manual under "When the bug is in the TEST, not the kernel". **Superseded within the hour by the actual root cause — kmtest's private `POOL_HEADER`
gates its 64-bit layout on `_M_AMD64` while the kernel gates on `_WIN64`, so on arm64 the test
reads half a header. See the roadmap's ExPools entry. Nothing below about the kernel stands;
the floor question does.** **What it leaves owed is narrower
and is a FLOOR question, not a kernel one:** `:178` and `:181` carry `assert?` marks that record
consequences of `:176` as independent debt, and `:178` in particular passes *spuriously* in the
bad mode, so the quota bit is untested on every run that takes it. Un-marking them makes arm64
gates redder, so it is a deliberate call rather than a cleanup.

### 2026-10-12 — hibernation handoff: x86 on a writable root, four landings, the owed queue

The lane pauses until the next weekend. Everything below was observed on master at
`20b8abeb1468`, with two identical sweeps per arch behind every banked number.

**Run it.** `VIBEOS_PAYLOAD=kmtest ./vibeos-cli build --arch <a>`, then
`VIBEOS_WRITABLE_ROOT=1 ./vibeos-cli conformance --arch <a> --no-build`. Bank a row only after
two sweeps agree, by replacing that row's block in the baseline and never with a blanket
`--write-baseline`, which would clobber the hand-seeded `KeEvent` row and every `assert?` mark.
Put a new dated note BELOW the newest existing one, not directly under the header line: other
lanes add notes there too, and two insertions at the same point bounced this lane CONFLICT
twice in the merge queue.

**Landed this weekend**
- `85a6621a5690`: i386 boots the writable root (`livecd-rw.iso`). fastfat's flush called a
  cdecl helper through a stdcall declaration (DECL_CC_SPLIT, now seen by `scan_cc_mismatch`
  through sys-crate imports), and ExReturnPoolQuota had dropped its special-pool check.
- `a76a2d7eef8a`: `mk_writable_livecd` stages outside `build-<arch>` (it raced amd64's WoW64
  restage and rejected two batches).
- `5c8ff1d7a529`: 119 kernel call sites had translated `ProbeForWrite*` to the range-only
  `probe_user_buffer`; now `probe_user_buffer_write`, and `scan_dropped_probes` reports the
  shape as `WEAK`. IoReadWrite 151 -> 0 on every arch. Also MmSection 30 -> 14 (offset rounding,
  FileObject sections are data sections, SEC_LARGE_PAGES status).
- `20b8abeb1468`: the DBG list poison was behind `cfg(debug_assertions)` (ExSingleList 12 -> 0,
  ExDoubleList 16 -> 0/8), npfs FileAllInformation, and three object-type access masks.

**Failing rows now** (failing assertions per row; amd64 / i386 / arm64 where they differ):
CmSecurity 1, ExPools 1, ExResource 2, ExResourceBounded 2, Example 15, FsRtlExpression 8,
HalSystemInfo 1/1/2, HidPDescription 9, IoCreateFile 7, IoDeviceObject 4, IoFilesystem 3,
KeDeviceQueue 1, MmSection 14, NpfsFileInfo 19, NtSystemDebugControl 20/15/20, ObSecurity 14,
ObTypes 9, RtlStack and RtlStackKM 11/1/11, SeInheritance 14, TcpIpConnect 7, TcpIpTdi 4,
ZwAllocateVirtualMemory 9/2/11, ZwCreateSection 3, ZwMapViewOfSection 6, ExDoubleList 8/0/8.
amd64 only: RtlCaptureContextUM 10. i386 only: KeSpinLock 30, MmReservedMapping 44,
ExInterlocked 4. arm64 only: IoInterrupt 2, RtlMemory and RtlMemoryKM 3.

**Owed, in order**
1. **RtlStack / RtlStackKM** (11 on amd64 and arm64, 1 on i386): the stack walker is a real
   shared defect, and the obvious next target.
2. **The rest of the shared queue, untriaged:** Example 15 (kmtest's sample test; check whether
   its failures are deliberate before treating them as bugs), NtSystemDebugControl (statuses
   0xc0000004 or 0xc0000001 where 0xc0000002 is expected), HidPDescription, FsRtlExpression,
   TcpIpConnect/TcpIpTdi, IoCreateFile, IoFilesystem, IoDeviceObject, ZwMapViewOfSection,
   ZwCreateSection, SeInheritance, the single-assertion rows.
3. **i386 only:** MmReservedMapping 44, KeSpinLock 30, ExInterlocked 4.
4. **arm64 only:** IoInterrupt 2, RtlMemory 3.
5. **SMP baselines** (`kmtest-smp2.txt`) were not re-swept this weekend. Expect IoReadWrite's pass
   count to read as dropped (16192 -> 15563): the bad-buffer calls now fail before the test
   driver, so its per-IRP assertions do not run. That drop is correct; rebank it.
6. **Tooling:** `scan_dropped_probes` WEAK only sees files that carry their C as comments. 61 of
   the 119 sites were found by pairing each Rust probe with its same-named C function in the
   tree (`tools/c_twin.py` is that oracle) and matching the probed pointer by its last
   identifier. Pairing by name alone produced one false positive, IopDeviceFsIoControl's
   METHOD_IN_DIRECT branch, which is a ProbeForRead. Give the scanner that oracle so the
   class stays closed. Its baseline also lists 5 PARTIAL lines it no longer reports.

**Expected, not owed** (the reasons are also in each baseline's header):
- MmSection 14: on Server 2003, closing a file's last handle tears its data segment down; ours
  outlives the close, so the pointer count stays 3 where the test wants 4. A cache-lifetime
  design difference inherited from ReactOS.
- ObSecurity 14: on Windows, SYSTEM's `\??` is a per-LUID directory with a 4-ACE DACL; ReactOS
  has only `\GLOBAL??` with its 6-ACE DACL. Building per-LUID directories is design work.
- NpfsFileInfo 19: the sizeof-4 query fails INFO_LENGTH_MISMATCH in the I/O manager on a checked
  build, as Windows does; the test lacks its KmtIsCheckedBuild guard.
- ExDoubleList 8 on 64-bit: with no Exf variant, both blocks call one function and expect two
  different poison values.
- ObTypes 9: WindowStation cannot keep handle counts without Open/Close procedures (win32k
  supplies them on Windows); Port's 0x7b2 is valid for 2003 SP2 RTM by the test's own comment;
  the Filter* types need fltmgr loaded.
- arm64 HalSystemInfo 0/2: the class lists x86 I/O ports, and arm64 has no port space.

**Mistakes worth not repeating.** Setting MaintainHandleCount on WindowStation crashed boot on
every arch (BUGCHECK 0x32): ObCreateObjectType refuses the flag on a type with no Open or Close
procedure, and I had called the change safe without reading that check. Editing source while
my own build ran cost two rebuild cycles; the build's artifact check caught it both times.

### 2026-10-12 — a Windows 11 reference column (from the research lane)

Windows 11 26300 now runs this lane's own suites: `./vibeos-cli winref kmtest` and
`winref apitest` (needs `./vibeos-cli build --arch amd64 --kmtests --apitests
--no-wow64-stage`; design and caveats in `winref.md`, "The reference columns"). Same
binaries, markers and parser as the guest. The kmtest column is banked at
`tools/conformance-baseline/win11-amd64/kmtest.txt` from two byte-identical runs, and every
run writes `column.txt` against `amd64/kmtest.txt`. It is a reference, not a floor: nothing
gates on it, and banking a guest row is still this lane's two-sweep rule. The apitest column
is NOT banked: two runs agree on 630 of 633 rows, and the three that move (comctl32
`buttonv6`, ntdll `NtQueryInformationProcess`, win32u `NtUserGetThreadState`) want `assert?`
marks first. Run artifacts: `20261012-150918-ci4i6nxq` and `20261012-164618-1ukvsju5`
(apitest), `20261012-160422-w1aacqy3` and `20261012-160425-l5vbpj_0` (kmtest), all
`winref-*-x64` under `/tmp/vibeos-cli/` (gone at the next reboot; the banked file is the
durable copy).

What it says about the owed queue above (observed; the "why" is still yours):
- **RtlStack / RtlStackKM 11** fail the SAME 11 assertions on Windows 11. Item 1 is not a
  defect in our stack walker; it is the test, or how our toolchain builds it.
- **Example 15** fails identically on Windows: deliberate.
- **Clean on Windows, failing in the guest:** MmSection 14, SeInheritance 14, NpfsFileInfo
  19, RtlCaptureContextUM 10, HidPDescription 9, FsRtlExpression 8, ExDoubleList 8,
  ZwMapViewOfSection 6, HalSystemInfo 1, KeDeviceQueue 1. ExDoubleList and NpfsFileInfo were
  filed above as expected; Windows x64 (a free build) passes both, so those two explanations
  want a second look.
- **Shared with Windows:** TcpIpConnect 5, ObTypes 2 (of 9), one IoCreateFile assertion.
- **ObType and RtlRangeList bugcheck Windows 11** (0x3B) every run.

Changes here that touch this lane: `kmtest_.exe` writes its BEGIN/END/ERROR markers to
stdout as well when it has one (none in the guest sweep); `apitest-sweep.exe` does the
same through `emit()`; `parse_kmtest_serial` marks EVERY bugcheck's in-flight test CRASH
(not only the first) and closes a record re-launched without an exit as INCOMPLETE; the
KeIrql kmtest resolves `KeRaiseIrqlToSynchLevel` at run time (Windows x64 does not export
it). A guest sweep after all of it: amd64 99/99, ratchet holds
(`20261012-161729-7jt82k7j-conformance-amd64`).

### 2026-10-12 (evening) — four new kmtests, and MmMdl's floor (from the research lane)

**New tests in all three lists:** `EtwActivity`, `EtwProvider`, `ExPool2`, `FeatureConfig`
(`modules/rostests/kmtests/ntos_ex/ExPool2.c`), for the Windows 10 kernel exports Rung 0 added
(`ExAllocatePool2/3`, `ExFreePool2`, the ETW provider API, feature configuration). Each was
graded on Windows 11 first (`winref kmtest --only ...`, all clean there), then swept here: on
i386, amd64 and arm64 every one is `OK fail=0` with Windows' exact pass counts (5/7/42/2).
They are NOT banked into the guest floors (that is this lane's two-sweep rule to apply);
`compare()` treats them as new tests, not regressions. With the kernel changes set aside they
fail (0/1, 0/3), so they measure the exports rather than passing vacuously.

**MmMdl's amd64 floor lowered 720 -> 714**, by the rule already in `amd64/kmtest.txt` (the
floor is the low face; the same 6-apart pair as 575/569). The low face appeared at 19:42 and
has been the only face since, except one 720 at 20:02. It rejected native-batch@vibeos-gicv3
5f561eb924ec three times as CONFORMANCE_RED. My rounds read 714 with and without this lane's
kernel changes, so it is not them. Ledger rows: `~/vibeos-flake-farm/conformance.jsonl`,
19:42-20:34. If something since 19:40 should NOT have cost one 32 MB mapping, that is worth a
look; the floor change only stops it bouncing innocent batches.

### 2026-10-16 — the Windows 11 column's queue

Worked the rows that are clean on Windows 11 but failed in the guest, on all three arches.
Banked from two byte-identical sweeps per arch; each baseline's `2026-10-16b` note has the
per-row before/after.

**Fixed:** FsRtlExpression 8 (DBCS matcher broke out before counting the end-of-name pass),
KeDeviceQueue 1 plus two more invented object Type literals found by grepping for the class
(Profile 24 -> ProfileObject 23, Controller 3 -> IO_TYPE_CONTROLLER 2), SeInheritance 14,
HidPDescription 9 -> 4, HalSystemInfo:143, arm64 IoInterrupt 2, ZwMapViewOfSection 6 -> 1
(2 on i386). MmFindGap tested the wrong search result (TableEmptyTree for TableFoundNode).
Seven new tests banked, including **MmReservedMappingFixed**: the first use of the "duplicate
the test" rule. Its original stays as a commented expected failure (a PAE-only expectation on
a non-PAE kernel).

**Re-checked, still expected:** ExDoubleList, NpfsFileInfo (PRCB_BUILD_DEBUG: we are checked,
Windows 11 free), MmSection 14 (the test's WS03 column), amd64 RtlCaptureContextUM 10 (TCG's
fxsave), i386 ExInterlocked 296/297 (the test's own SaveState TODO; the export preserves edi).
RtlStack/RtlStackKM fail identically on Windows.

**Owed, in order:**
1. ~~ZwMapViewOfSection:211~~ (done 2026-10-17; the cause given here was WRONG, see below).
2. ~~i386 KeSpinLock 30~~ (done 2026-10-17): every arch's queued spinlocks are the C's plain-lock HACK; only i386's
   test checks. A real MCS lock is boot-risky if anything takes one lock both ways.
3. HidPDescription 4: a ReportIDs entry per report ID (hidclass routes on it, so test against
   a HID device), and PreparsedDataLength in Windows' layout.
4. arm64 RtlMemory 3: NOT memcpy (the export is overlap-safe; disassembled). Each failure is
   the first check after a `_SEH2_TRY { (NULL, NULL, 0) }` block.
5. i386 ZwMapViewOfSection:181 and MmReservedMapping as before; SMP baselines; the scanner
   item from the 2026-10-12 queue.

**When the research lane flips the reported version to 10.0**, 28 kmtests change columns at
once: MmSection and ObTypes' WS03 rows should clear, KeSpinLock's i386 checks vanish (the
queued-lock gap does not), and some 64-bit rows newly fail (ZwMapViewOfSection ZeroBits 1/5
want INVALID_PARAMETER_4 from Win8). Sweep their branch before it lands.

**Tooling:** `conformance --no-build` now refuses a build dir configured without the kmtest
corpus. Pruning recreates build dirs, a fresh configure defaults it OFF, and the sweep then
only failed minutes into the boot. Mistakes: I edited source during my own builds twice more
(stopped and rebuilt both times), and first guessed RtlMemory was inlined memcpy without
looking at the disassembly.

### 2026-10-17 — two owed items, and a cause I got wrong

Banked from two byte-identical sweeps per arch (baselines' `2026-10-17` notes).

**ZwMapViewOfSection:211 done** on all three arches. A file view's reservation may run past
the end of the file, bounded by the user address space (a ViewSize of -1 page-rounded to an
empty "successful" view, :295). The offset rounding's `ViewSize + rem` now saturates instead
of wrapping. amd64/arm64 135/0. i386 trades 211 for 292, LOWERED BY DESIGN: the WS03 column
wants a 4 GB reservation to succeed, and it only ever "passed" through that wrap.

**The cause I gave yesterday was wrong.** The 2026-10-16 entry, the baselines' 16b note and
commit `4f8fc0a3e27` say a reserved view past the end of the file corrupted cached pages and
bugchecked x86. One isolated sweep each on i386 and arm64 (reserve allowance back, nothing
else changed) showed no crash and no corruption. The damage came from the other change in
that same build, the blanket "no CommitSize on a file view" rule. I attributed a two-change
fix to the change I suspected without isolating it. The notes are corrected; the commit
message cannot be.

**i386 KeSpinLock 30 -> 0.** The in-stack queued spinlocks are a real MCS lock
(`ntkrnl-sys` `KxAcquire/ReleaseInStackQueuedSpinLock`) for ntoskrnl's exports, hal's, and the
AtDpcLevel pair, replacing the C's plain-lock HACK (its MCS is `#if 0`, "broken": it exchanges
on `LockHandle->Next`). The audit that made it safe is in the helper's doc comment: no lock is
reached by both the in-stack and a plain API, and the numbered `KeAcquireQueuedSpinLock*`
family (still plain) only takes PRCB LockQueue[] entries.

**Owed, in order:**
1. amd64/arm64 in-stack queued spinlocks are still the HACK. Their kmtest cannot see it (the
   checks are `_M_IX86` only), so this is for correctness under the SMP lane: move them onto
   the same helper (it is `cfg(target_arch = "x86")` today) and sweep `--smp 2`.
2. HidPDescription 4: a ReportIDs entry per report ID (hidclass routes on it, so test against
   a HID device), and PreparsedDataLength in Windows' layout.
3. arm64 RtlMemory 3: NOT memcpy (the export is overlap-safe; disassembled). Each failure is
   the first check after a `_SEH2_TRY { (NULL, NULL, 0) }` block.
4. i386 ZwMapViewOfSection:181 (ZeroBits 21: we say INVALID_PARAMETER_4, Windows NO_MEMORY),
   SMP baselines, the scanner item from the 2026-10-12 queue.

### 2026-10-17b — the overnight queue

Banked from two byte-identical sweeps per arch (baselines' `2026-10-17b` notes).

**In-stack queued spinlocks are the MCS lock on every arch.** The ntkrnl-sys helper lost its
`cfg(x86)`; amd64's ntoskrnl exports, arm64's hal.dll exports and both arches' AtDpcLevel pair
use it. The audit carries over because every in-stack caller is shared code; only where the
exports live differed. No row moves (the kmtest checks are `_M_IX86` only).

**HidPDescription 4 -> 2 (3 skips -> 0).** One ReportIDs entry per report ID with its own
lengths, in declaration order; hidclass picks the collection's longest input report to size
reads, since the first entry can now be a feature-only ID. The C also leaked every built
preparsed context on a failure path; the Rust frees them. Left, EXPECTED: PreparsedDataLength
476/1388 is Windows' own preparsed-data layout, which our HidP_* routines do not read.

**i386 ZwMapViewOfSection:181 done.** `MI_MAX_ZERO_BITS()` was a hand translation: 20 on
32-bit, where the C's i386 header says 21.

**arm64 RtlMemory 3 is EXPECTED, and yesterday's note was wrong.** I said the driver calls the
overlap-safe export. At these three sites it does not: clang inlines the overlapping
`RtlCopyMemory(Buffer + 13, Buffer + 62, 95)` as interleaved 16-byte loads and stores
(`Test_RtlMemory+0x82c`), and an overlap forward-copy through them reads bytes it has
already overwritten (offset 13 reads 0x34, src[111]'s value). "First check after the SEH
block" was a coincidence of layout: each section's first check is its forward-overlap case.
Overlapping memcpy is undefined; RtlCopyMemory's contract forbids overlap.

**Tooling.**
- `scan_dropped_probes` pairs functions with no commented C to their same-module C twin
  (`c_twin`) for the WEAK kind; on the tree it pairs 72 of 80 range-only functions and finds
  nothing, as it should after 2026-10-12. Its baseline lost the 5 stale PARTIAL lines.
- artifact-check: a cohab staticlib that sits beside ANOTHER same-stem binary belongs to that
  binary. `wdmaud.drv` was reported "never relinked" on every arm64 build once the kernel's
  wdmaud-rust rebuilt.
- `--smp N` boots the writable-root ISO, which carries freeldr.ini twice; identical copies
  are now both patched instead of refused.

**SMP baselines rebanked** (`kmtest-smp2.txt`, two identical `--smp 2` sweeps per arch, both
CPUs live on the MP pair). Every row now equals the arch's UP baseline (amd64 MmMdl aside, a
machine-dependent pass count). IoInterrupt's banked MP failures (156 on 64-bit, 306 on i386)
are gone: its CheckSpinLock takes the checked-build branch first, which only wants non-zero,
and the kernel reported a free build when they were banked. Rows for tests an arch does not
build were dropped.

**Seen once, not mine:** i386 `--smp 2` sweep 2 died BUGCHECK 0xF4. csrss took an unhandled
RPC exception (`6be`, then `6ba` on second chance) at 7c505de7. flake-report already holds the
same signature from a UP boot on 2026-10-16 (graded FLAKY, 2 commits), on a tree before this
work. It looks like the field manual's `rpc_try` class (a Rust RPC client that kept only the try
body) reached from csrss. The sweep after it was clean and identical to the first.

**Owed, in order:**
1. ~~The csrss F4 flake~~ (fixed): winsrv `UserServerHardError` -> `ReportEventW` ->
   `ElfrReportEventW_seh`, a native-SEH thunk that is a plain call on i386, so the RPC raise
   killed csrss while eventlog.exe was exiting. advapi32 now uses `rpc_try` for those three calls.
   ole32's rpcss_seh.c and stubmanager_seh.c had the same i386 hole; their i386 branch now
   calls ntdll's RtlSehTryExcept. Smoke-verified only: olewinetest's marshal case (an object
   whose Release faults) has not been run on i386 since (it needs a `--rostests` build).
2. HidPDescription's preparsed-data layout, only if something needs Windows' bytes.
3. The WS03-column rows that move at the version flip (above).
