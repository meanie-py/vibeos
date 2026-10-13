# winref: real Windows 11 as the conformance oracle

ReactOS's rostests encode what someone believed Server 2003 did. For the Windows 10/11
surface (`modern-windows-compat-roadmap.md`, Rung 0) the expected result comes from Windows
itself: build a test with our toolchain, run it on a real Windows 11, and grade the vibeos
guest against that. `vibeos-cli winref` (`tests/qemu/cli/winref.py`, guest scripts in
`qemu/winref/`) is the VM; commands are in `tests/qemu/cli/README.md`.

## Shape

- **Host:** the Windows host's QEMU (`C:\Program Files\qemu`), images on `D:\vibeos-winref`.
  No network device: no updates, no account, no drift.
- **Base image:** one unattended install (`autounattend.xml` + first-logon `setup.ps1`):
  test signing on, Windows Update off, autologon as `winref`, and an elevated logon task
  running `C:\winref\boot.ps1`.
- **Run:** a qcow2 overlay of the base boots with the payload directory as a CD. The
  runner copies it to `C:\winref\payload` (tests write beside their own .exe), runs
  `run.cmd` elevated, relays every line to COM1 (`OUT: ...`), prints `WINREF: exit=<n>`,
  powers off. The overlay is discarded. ~1.5 min on WHPX.
- **Maintenance:** `winref maint <script.ps1>` boots the base itself with the script as a
  payload; `--keys` types it into Win+R over QMP when the runner itself is broken.
  Snapshot first. `post-install` is the pristine result of the install.
- **Evidence:** `winref logs` reads `C:\Windows\Panther\UnattendGC\setupact.log`, the
  runner task's XML and `C:\winref\` off a stopped disk (7-Zip + `qemu-img dd`; no nbd,
  no admin). Every run keeps a downscaled screenshot timeline in `shots/`.

Reference so far: Windows 11 Pro 26H2, build 26300.9457, from Microsoft's public
multi-edition ISO, unactivated (the generic Pro install key selects the edition).

## What it cost to get here (each is fixed in the tool)

| Symptom | Cause |
|---|---|
| `WHPX: Unexpected VP exit code 4` mid-Setup, after `Ignoring request for interrupt vector 0` (AMD Ryzen 9 7950X3D) | QEMU's WHPX on this host; not the CPU model, USB or irqchip. Install under TCG (~2 h); installed systems run fine on WHPX |
| Setup's account `winref` never became an admin (`0x8007056b`), so FirstLogonCommands never ran | computer name == account name: the group add resolved the MACHINE. Computer is `WINREF-VM` |
| FirstLogonCommands skipped | deprecated `SkipMachineOOBE`/`SkipUserOOBE` (removed) |
| Setup hung forever | `schtasks /create /ru` without `/rp` prompts for a password |
| VM sat at the desktop after setup | `Stop-Computer` from FirstLogonCommands came back as a RESTART; the launcher presses the ACPI power button once the guest prints its finish line |
| Runner never fired | task XML in `System32\Tasks` but unknown to Task Scheduler; setup and maint now verify `Get-ScheduledTask`. Not a battery issue: Windows reports line=Online |
| 20 GB snapshot copy: `ENOMEM` | DrvFS `sendfile`; copies go through Windows `Copy-Item` |

## What the first payload found

kernel32 `sync`/`version` and ntdll `rtl` winetests, built by our toolchain, run green on
Windows 11 only after fixing three things on OUR side, none of them Windows behaviour:

- **59 `sync` failures, error 161:** our build writes `__FILE__` as
  `\modules\rostests\...` (clang + `-ffile-prefix-map` on a Windows target), and object
  names may not contain `\`. The test's MSVC-only guard now covers every ReactOS build.
- **31 `sync` failures in `NtWaitForMultipleObjects`:** the test declared the 4-byte
  `WAIT_TYPE` as `BOOLEAN`; on x64 the kernel read undefined upper bits of r8.
- **14 `version` failures:** the test writes a manifest beside its .exe; the payload CD is
  read-only, so it measured an unmanifested process. The runner now copies payloads.

Each would have failed the same way on vibeos and been filed as an OS bug.

## The reference columns (2026-10-12)

`winref kmtest` and `winref apitest` run the conformance ratchet's own suites on Windows 11:
the guest's binaries from this tree's `build-amd64`, the guest's markers, the guest's parser
(`tools/conformance/parse_*_serial.py`), so a Windows row and a guest row mean the same
thing. Banked with `--write-baseline` under `tools/conformance-baseline/win11-amd64/`, after
two runs agreed. It is a reference, not a floor. `column.txt` sorts every disagreement with
the guest's amd64 floor by what was observed: "Windows clean, guest fails" (look for a guest
bug there first) and "fails on Windows 11 too" (an expectation Windows does not meet, a test
that is timing-sensitive in a VM, or one our toolchain builds differently from its author's).

How each suite gets there:

- **apitest** runs the guest's `apitest-sweep.exe` unchanged. Its lines go to stdout as well
  as DbgPrint (`emit()` in `sweep.c`), because DbgPrint reaches nothing on Windows without
  a kernel debugger. All 64 suites, no quarantine (that list protects the guest). ~5.5 min.
- **kmtest** runs `qemu/winref/kmtest-sweep.ps1`, which prints the guest sweep's markers;
  `kmtest_.exe` now also writes its BEGIN/END/ERROR markers to stdout when it has one.
  The drivers are signed in the VM per run with a throwaway self-signed certificate:
  `testsigning` admits TEST-SIGNED drivers on x64, not unsigned ones. A bugcheck reboots
  Windows and the sweep resumes from `C:\winref\kmtest-progress.txt` (written through to
  disk: the cache dies with the bugcheck), reporting `*** BUGCHECK` from event 1001 against
  the test in flight. If QEMU itself dies, the launcher boots the same overlay again (up to
  10 times); a test the VM is lost under twice without a logged bugcheck is NOT RUN. ~41 min.

A row the harness could not run is NOT RUN, never a result; a run where nothing ran fails.

What the first columns found (Windows 11 26300.9457):

- **Our amd64 headers made every kmtest driver unloadable on Windows.** `xdk/amd64/ke.h`
  declares `KeRaiseIrqlToSynchLevel` as an import, because our UP kernel's SYNCH_LEVEL
  differs from MP's; but Windows x64 has no such export (the WDK inlines it), so
  `kmtest_drv.sys` failed StartService with ERROR_PROC_NOT_FOUND and all 99 rows were
  harness errors. The KeIrql kmtest now resolves it at run time. Any amd64 driver of ours
  that calls it has the same problem on Windows.
- **The apitest column** (two runs, NOT banked): 633 subtests in 64 suites, 443 clean.
  630 rows identical across the runs; three move by count (comctl32 `buttonv6`, ntdll
  `NtQueryInformationProcess`, win32u `NtUserGetThreadState`) and want `assert?` marks
  before a bank, which is the conformance lane's call.
- **Our C++ runtime ties every C++ test to ReactOS.** libc++ through `stdc++compat` imports
  SRW locks from `kernel32_vista.dll` and `snprintf` from msvcrt (Windows' msvcrt.dll only
  has `_snprintf`), so `atl`, `browseui`, `shell32` and `zipfldr` apitests do not load on
  Windows (NOT RUN). Owed to the C++ substrate lane.
- **SEH-protected faults kill the test on Windows.** 59 apitest subtests exit
  0xC0000005, 27 of them ntdll's, including `strlen`, `_wcsicmp`, `setjmp`, `StackOverflow`
  and `UserModeException`: subtests that fault on purpose inside a `__try`, and the process dies with
  the fault unhandled. Unconfirmed: whether it is how our toolchain emits SEH, or the
  tests. That is the first thing to look at in this column.
- **The kmtest column** (two 41-minute runs in parallel, byte-identical tables; banked):
  97 of 99 complete, 59 clean. `ObType` and `RtlRangeList` bugcheck Windows 11 (0x3B, every
  time); the guest runs both. Ten rows are clean on Windows and fail in the guest: MmSection
  14, SeInheritance 14, NpfsFileInfo 19, RtlCaptureContextUM 10, HidPDescription 9,
  FsRtlExpression 8, ExDoubleList 8, ZwMapViewOfSection 6, HalSystemInfo 1, KeDeviceQueue 1.
  Two of those the conformance lane had filed as expected (ExDoubleList "on 64-bit",
  NpfsFileInfo "a checked build"), and Windows x64's free build passes both. Shared with the
  guest, assertion for assertion: RtlStack and RtlStackKM 11 (the lane's owed item 1, filed
  as a stack-walker defect, fails identically on Windows, so it is the test or how our
  toolchain builds it, not our kernel), Example 15 (deliberate), TcpIpConnect 5, ObTypes 2.
  The rest fail only on Windows; KeMutex 517, KeEvent 108, KeApc 92 and ExFastMutex 82
  are probably timing in a VM rather than semantics (not checked).
- WHPX: QEMU dies (`Unexpected VP exit code 4`) while Windows reboots after a bugcheck,
  which is why the relaunch exists.

## New tests: graded by Windows first

`winref kmtest --only <Test>...` runs just those tests (and refuses `--write-baseline`: a
subset is not the column). That is the loop for a test written against the new surface:
it must pass on Windows 11 before its result in the guest means anything. The first four
(`ExPool2`, `EtwProvider`, `EtwActivity`, `FeatureConfig`, in
`modules/rostests/kmtests/ntos_ex/ExPool2.c`, for the Rung 0 kernel exports) were wrong
twice before Windows passed them, both times in a way our kernel would have hidden:

- **`ok_eq_hex(Call(...), x)` calls `Call` twice**: `ok_eq_print` passes its value both to
  the comparison and to the format. `EtwUnregister` ran twice on one handle and Windows
  bugchecked (0x139, parameter 3: list-entry corruption); our no-op unregister would have
  passed. Put every call in a variable before an `ok_eq_*`.
- **Documented is not measured.** "Invalid POOL_FLAGS fail the call", but Windows accepts
  `POOL_FLAG_RESERVED2`, and probing the other reserved bits bugchecked it. The test no
  longer touches reserved bits, and `ExAllocatePool2` ignores them as Windows does.
- **Look exports up, never import them** (`MmGetSystemRoutineAddress`): one hard import a
  kernel lacks makes all of `kmtest_drv.sys` unloadable, where a lookup fails one assertion.

Then in the guest, all four pass with Windows' counts on all three arches.

The rig, hardened by the same runs: the sweep reports a bugcheck's parameters, not just its
code (0x139's first parameter is which check failed), and the base boots with
`recoveryenabled No` / `bootstatuspolicy IgnoreAllFailures` (`setup.ps1`, `maint-runner.ps1`).
Without them, three bugchecks in one run left Windows in Automatic Repair, waiting for a
key press until the run's timeout.

## State on disk (2026-10-12, evening)

`D:\vibeos-winref\` (Windows path; `/mnt/d/vibeos-winref` from WSL):

| Item | What |
|---|---|
| `base.qcow2` + `base-vars.fd` | the live base = `runner-v4`: `runner-v3` plus recovery off (a crash reboots into the runner, never Automatic Repair) |
| `snapshots/runner-v4` | = current base |
| `snapshots/post-install` | pristine result of the install (runner task NOT registered: needs `maint`) |
| `snapshots/runner-v2` | task re-registered, old runner (runs off the CD) |
| `snapshots/runner-v3` | payloads from `C:\winref\payload`, recovery still on |
| `logs/<ts>/` | `winref logs` extractions |

Unactivated retail Pro (public multi-edition ISO, generic install key): no expiry to plan
around, unlike an evaluation image. A reinstall is `winref install --iso <iso>` (~2 h, TCG).
Last green payload: `winref run` of kernel32 `sync`/`version` + ntdll `rtl`, run
`20261011-211134-zx72269t-winref-run-x64` (0 failures, 1m26s, WHPX).

## Owed

- **Banking the columns and wiring them into `conformance`.** The verbs exist and run; the
  conformance lane decides when a column is stable enough to bank (two agreeing runs, as for
  the guest), and whether `vibeos-cli conformance` should print the Windows row beside each
  regression. The winetest sweep (`build --winetest-sweep`) has no Windows column yet.
- **The `win_skip` meter.** Our winetests are a Wine 10.0 sync and already probe the
  Vista-11 APIs at runtime: 1,937 `win_skip` sites skip on vibeos today. Counting skips per
  guest run turns the export gap into a coverage number for free; skips become passes or
  real failures as stubs from `win10_surface.py` get bodies.
- **Generated tests, graded by Windows first.** For gap functions Wine does not cover: a
  model writes a Wine-style test, it must PASS on `winref` (a test Windows fails is a wrong
  test, discarded) and FAIL against a planted mutant, and only then joins the ratchet.
- **Drivers.** The VM runs test-signed drivers (`testsigning` on, Secure Boot off):
  kmtests for the Win10/11 kernel surface (`ExAllocatePool2`, ETW, feature configuration)
  are graded here. virtio-win drivers (`netkvm` NDIS 6, `viostor`/`vioscsi` StorPort,
  `vioinput` KMDF HID, `viogpudo` WDDM) run on the same QEMU devices vibeos uses, so
  Windows-vs-vibeos is a same-driver comparison. Physical hardware (GPU, Wi-Fi) needs real
  Windows on the OptiPlex instead; there is no Windows 11-capable GPU for it yet.
- **Defender exclusions** failed at setup (`0x80010005`). Harmless so far; fix in
  `maint` if Defender ever quarantines a test binary.
- **QMP sometimes does not connect** on a run (20261011-210309); the run still completes
  and is graded from the console. Unexplained.
- **rostests build**: `./vibeos-cli build --arch amd64 --rostests` is green as of
  2026-10-12 (user32_winetest's `monitor.c` compiles alone with Wine's headers first, the
  combase winetest is off with combase itself, menuaccels' table is raw RT_ACCELERATOR data).
  i386 and arm64 were not rebuilt with `--rostests`.
- **apitest's SCM gate** waits 2 min on Windows for `SC_AutoStartComplete`, which Windows 11
  does not create; harmless, but a minute or two per run.
