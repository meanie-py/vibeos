# Final bosses: modern drivers, real GPUs and Chrome

The long-range plan for the point where vibeos stops being "ReactOS in Rust" and starts
running **today's Windows binaries unchanged**: Windows 10 drivers on real hardware, a
real GPU, and the latest Chrome. Written 2026-10-05 from a design discussion with the
owner. Product summary: [`../product/vibeos-roadmap.md`](../product/vibeos-roadmap.md) §12.

**Read the confidence markers.** Lines marked *(tree, 2026-10-05)* were checked against
this repo. Lines marked *(unverified)* are recollection about external software (vendor
driver history, Chromium internals, Libreboot payloads). Check each one against the thing
itself before building on it.

---

## Where we start

Our kernel's export surface and driver models are NT 5.2 (Server 2003). Modern drivers
and apps target Windows 10. Most of the gap is **additive** (new exports, new framework
hosts, new class extensions). Only the graphics stack is a genuinely new subsystem.

Already in the tree *(tree, 2026-10-05)*:

| Piece | State |
|---|---|
| KMDF (`vendor/wdf`) | Microsoft's MIT-licensed WDF, **built** as `wdf01000.sys`, KMDF 1.17. Vendored, `translate: planned` |
| StorPort | built (`storport.sys`), Rust port in `sdk/lib/rust/storport-rust` |
| xHCI | **Rust-native** `usbxhci` (~20K lines, written in Rust from the start); SuperSpeed devices work on the Pi 4 |
| HD Audio / PortCls | `drivers/wdm/audio/hdaudbus`, `portcls` |
| D3DKMT thunks | `win32k-rust/src/d3dkmt.rs`: the syscall surface exists, but every adapter call returns `STATUS_PROCEDURE_NOT_FOUND` because nothing populates the dxgkrnl table |
| NVMe | `nvme.sys` (2026-10-06), Rust from the start, all three arches: MSI-X interrupts since 2026-10-07 (the pin as fallback, a 10 ms timer behind both), a queue pair per CPU (up to 8, each on the MSI-X message the HAL delivers to that CPU, since 2026-10-08), 63 commands deep with DMA to the requester's pages (2026-10-07), each namespace a disk.sys disk. A command hung 30 s resets the controller and is failed back for disk.sys to retry (2026-10-08; `--tripwire NVME_LOSE_COMPLETION=<n>` proves it) |
| dxgkrnl / WDDM | **missing** |

[`../product/driver-roadmap.md`](../product/driver-roadmap.md) predates the KMDF and xHCI
work and still lists both as missing. Where they disagree, this doc and the tree win.

---

## Rung 0, shared by everything: the Windows 10 kernel export surface

Every boss below needs the same layer: the kernel APIs that Windows 10-built drivers and
apps import, which a 2003-era kernel does not export. So the first move is a **tool, not a
driver**:

- **The import-gap census.** Point it at a folder of real Windows 10 PEs (a stock
  `System32\drivers`, a few vendor driver packages, `chrome.exe`/`chrome.dll`). It reports
  every import we do not export, ranked by how many binaries need it. That turns "make
  Windows 10 binaries load" into a burn-down agents can grind, the way `port_audit.py`
  did for translation. User-mode binaries also resolve APIs at runtime with
  `GetProcAddress`, which no import table shows. Loader logging of failed lookups during a
  run covers those.
- **Layout drift.** WDK macros compile struct field offsets into drivers (`IRP`,
  `IO_STACK_LOCATION`, `MDL`, `DEVICE_OBJECT`, dispatcher headers). A driver built with the
  Windows 10 WDK assumes Windows 10 layouts, and a mismatch is silent corruption. Extend
  the offset verifiers to check our layouts against a Windows 10 WDK.

**Built (2026-10-10): `tools/win10_surface.py`.** It reads the export tables of a real Windows
install (WSL's `/mnt/c/Windows`: x64 `System32`, x86 `SysWOW64`, `System32\drivers`) and
compares them with our `.spec`s. There are four subcommands:

- `reference` reads the install into a cache.
- `stubs --apply` writes every missing name into its module's `.spec` as a `@ stub`.
- `verify` proves those stubs are dormant.
- `gap <dirs>` ranks what a set of programs or driver packages imports that we don't export.

The stub surface is **lined out, not implemented**: 16,874 stubs plus 124 data exports
across 226 specs. Data exports are listed for an `extern` and get no stub. All of them sit
in a generated block, measured against Windows 10.0.26100. Every line is
`-version=0xA00+`, so the shipping 0x502 build never sees one, and `verify` checks that
rather than assuming it: spec2def's output at 0x502 is byte-identical with and without the
block on all three arches. The tier means "present in that build", not "introduced in
Win10": nothing in an export table dates a name. Edit the hand-written part of a spec and
re-run `stubs --apply`; never edit the block itself.

`gap` labels each missing function with why it is missing:

| Label | Meaning |
|---|---|
| `gated` | A real line exists behind a newer `-version`, so the work may be done |
| `arch` | Exported on other arches only, often as arm64 forwards to `kernel32_vista` / `ntdll_vista` |
| `stub` | Declared, but only as a stub |
| `surface-only` | Only in the generated block |

First runs (measured against Windows 10.0.26100):

- **38 desktop apps, 2,385 images** (7-Zip, Blender, Git, VLC, Chrome, Paint.NET, Steam, …):
  - **474 missing functions**, of which 126 are already in the tree as `gated` or `arch`:
    SRW locks, condition variables, `InitOnce` and `GetTickCount64`, each imported by 11–19
    apps. 92 are stubs and 256 are untouched.
  - The CRT api sets (`api-ms-win-crt-*`) are imported by 15 apps, and we ship no api-set
    DLLs.
  - Missing modules: `ncrypt` (5 apps), Media Foundation (`mf`, `mfplat`, `mfreadwrite`)
    and `wevtapi`.
- **478 vendor driver packages, 1,099 images** (the host's `DriverStore`):
  - **1,457 missing functions**, 1,272 of them untouched. The top entries:
    - `KeInitializeSpinLock` (210 packages; our spec exports it on i386 only)
    - `ExAllocatePool2` (176)
    - ETW registration and writes (~146)
    - the `RtlQueryFeatureConfiguration` / `RtlRegisterFeatureUsageProvider` family (117,
      the WDK's static-library boilerplate)
    - the NDIS 6 NetBufferList and miniport API (~60–90 each)
  - Missing modules: `wpprecorder.sys` (128 packages), `cng.sys` (17), `werkernel.sys`
    (14). KMDF's loader is *not* a gap: our `wdfldr.sys` declares all ten of the Windows
    one's exports.

Imports resolved at runtime through `GetProcAddress` are invisible to `gap`; that half
still needs loader logging.

**Done 2026-10-12: the first three triage steps.** Measured with `gap` before and after,
weighting each missing function by the binaries that import it:

| Corpus | Missing functions | Imports needing a fix |
|---|---|---|
| 38 desktop apps | 474 -> 308 | 1,513 -> 827 (-45%) |
| 479 driver packages | 1,457 -> 1,121 | 11,483 -> 7,108 (-38%) |

1. **Code the tree already had.** `win10_surface.py forwards --apply` forwards every export
   Windows' module has, our default build does not, and a provider DLL implements: kernel32,
   ntdll, advapi32, gdi32 and user32 to their `*_vista` DLLs (159: SRW locks, condition
   variables, InitOnce, thread pools, `GetTickCount64`, the locale-name API, `RegDeleteTree`)
   and kernelbase to kernel32/advapi32 (838). Each forward switches off where the spec's own
   row takes over, the block is generated (`# BEGIN vista forwards`), and
   `forwards --check build-<arch>` follows every forward in the BUILT DLLs to a real body
   (0 problems on all three arches). It found two dead arm64 rows on the way, now fixed.
   A forward is a load-time edge the import walk never saw: SysWOW64 staged advapi32 and
   user32 without their `_vista` siblings, and every 32-bit guest died at load
   (0xC0000139; msvcrt binds `GetDpiForSystem` through user32 now).
   `tools/fex/wow64-closure.py` follows export forwarders since.
2. **Cheap kernel clusters**, in Rust, written from the WDK documentation:
   `ExAllocatePool2/3` + `ExFreePool2` (ex-rust), the ETW provider API with no sessions
   (wmi-rust: registration succeeds, nothing is ever enabled, writes go nowhere), feature
   configuration with nothing configured (rtlk-rust), `KeInitializeSpinLock` and
   `KeGetCurrentIrql` exported on amd64, and `wpprecorder.sys` (amd64/arm64), the WPP
   recorder 128 packages import. Four kmtests cover the kernel calls; Windows 11 graded
   them first (`winref.md`, "New tests: graded by Windows first"), and the guest matches
   Windows' counts on all three arches. One measured divergence from the docs: Windows
   accepts reserved POOL_FLAGS bits, so ours ignores them too.
3. **api sets are not missing.** The loader already redirects `api-ms-win-*` through
   `sdk/lib/apisets/apisets.table.c` (the guest passes the apisets apitest's 4,472 checks,
   as Windows does). `gap` now resolves them through that table, as the loader does, and
   lists names it has no row for: one, `api-ms-win-core-realtime-l1-1-1` (2 apps).

**Done 2026-10-12 (later): the owed tooling and the next kernel export.**
- **`tools/stub_hits.py`, the stub-hit report.** It ranks every stub any run reached, by
  the number of runs, then lines; default input is every run under `/tmp/vibeos-cli`, read
  through each run's `serial.log` so a boot counts once. Each stub gets its KIND from the
  spec (`raises`, `returns 0`, `returns V`) and a DIED count: kernel32's
  unhandled-exception filter prints `Missing function: dll!name` when a raising stub kills
  a process. Stub lines now name their module (`WARNING: calling stub msvcrt.dll!X(`);
  older lines are matched to the one spec stubbing the name, or listed as ambiguous.
  Beside it, the GetProcAddress misses: ntdll's `LdrpGetProcedureAddress` logs
  `LDR: GetProcAddress miss <dll>!<name>` for every by-name or by-ordinal lookup that
  finds nothing. A probed function has a fallback, so a miss is usually the right answer,
  and the list is what stops a stub from being added in its place.
- **`-stub=VALUE` in spec2def**, the non-raising kind: `@ stdcall -stub=E_NOTIMPL F(ptr)`
  logs its first call per process and returns VALUE (`E_NOTIMPL`,
  `STATUS_NOT_IMPLEMENTED`, `FALSE`, `NULL`, or a number). It needs a calling convention
  and the argument list, because i386 stdcall pops its own (`@ stub -stub=…` is refused);
  `tools/test_spec2def_stub_value.py` pins it. Raising stays the default, and NOTHING
  opts in yet: write a row by hand outside the generated block (the generator drops a
  name the spec declares) once the report shows a program reaching that stub and handling
  failure. No last-error variant: stub objects import nothing, so `-stub=FALSE` leaves
  `GetLastError` stale.
- **api-set names resolve ignoring the trailing minor version**, as on Windows (its schema
  hashes a name only up to the last `-`): `api-ms-win-core-realtime-l1-1-1` reaches the
  `-l1-1-2` row. It is a fallback after the exact row, in BOTH resolvers: ntdll-rust's
  `apisets.rs` is the one the loader runs, while `apisets-rust` is the one the apisets
  apitest links, which now has minor- and major-version cases. `gap` applies the same rule
  (`ros_apiset_host`), so no api set is unmapped any more.
- **`MmMapIoSpaceEx`**: `MmMapIoSpace` with the cache type taken from the protection (no
  modifier = cached, `PAGE_NOCACHE`, `PAGE_WRITECOMBINE`; both, or a protection that is not
  readable, is NULL). The access part is checked but not enforced; the view is read-write,
  like every `MmMapIoSpace` view. kmtest `MmMapIoSpaceEx`: Windows 11 10/0 first, then the
  guest 10/0.
- **ntoskrnl_vista's ETW copies are gone** (also classpnp's no-op shims): drivers that
  link the library now import the kernel's real `EtwRegister`/`EtwUnregister`/`EtwWrite`.
  (Corrected 2026-10-13: this entry also said ntoskrnl links ntoskrnl_vista. It has not
  since its cargo link dropped `libntoskrnl_vista.a` in June; only the CMake
  EXTRA_DEPENDS still names it.)
- **`wpprecorderum.dll`** (amd64/arm64): `WppAutoLogStart`/`Stop`/`Trace` and
  `WppAutoLogGetDefaultHandle`, records to nowhere like the kernel half. It is checked as
  a PE (its exports, and ntdll the only import); no UMDF host runs its importers yet.

Measured with `gap` after these, the drivers' weighted gap is 7,108 -> 7,060:
`MmMapIoSpaceEx` removed 72 import sites, and the api-set rule surfaced 7 functions (24
sites) that had been hidden behind unmapped api sets, now counted against their host
(`RegSetKeyValueW`, `VirtualAlloc2FromApp`, shell32's feature-state set, ...).

**Done 2026-10-13: the hidden exports.** `win10_surface.py gated build-i386 build-amd64
build-arm64` lists every row Windows exports that a spec gates behind a newer `-version`,
and for each arch says whether the body is LINKED into the built image, imported from
another DLL, absent, or (for a forwarding row) whether the target is exported.
`--apply` drops the `-version` floor from rows linked (or forwarding to a live target) on
every arch they are gated on. First run: 141 rows exported where the body already was,
none of them new code. `gap`, measured against a checkout of the commit before: apps
309 -> 271 missing functions, 829 -> 671 import sites (-19%); drivers 1,127 -> 1,095,
7,060 -> 6,872.
- kernel32 (46, two of them forwards into ntdll): the `*Ex` locale API (`LCMapStringEx`, `CompareStringEx`,
  `GetDateFormatEx`...), `CreateSemaphoreEx`, the processor-group calls, and the psapi
  set as `K32*` (`K32EnumProcesses`, `K32GetModuleFileNameExW`, ...).
- gdi32 (44): the `D3DKMT*` thunks, which call the WDDM driver's callback or return
  `STATUS_PROCEDURE_NOT_FOUND` with none; d3d11's five forwards to them now resolve.
- shell32 (18): `SHCreateItemFromParsingName`, `SHGetPropertyStoreFromParsingName`,
  `SHGetNameFromIDList`, `SHCreateDefaultContextMenu`...; ntdll (13); ws2_32
  (`inet_ntop`/`inet_pton` and the W forms); advapi32 (`RegSetKeyValue`, `RegCopyTree`...);
  msvcrt's `*printf_s`; crypt32, rpcrt4, pciidex one or two each.
- ws2_32's 23 `Wah*` helpers forward to ws2help (a new `forwards` provider), `-private`
  because ws2help links ws2_32's import library and would collide with its own
  definitions (`LINKS_BACK`). kernelbase's forwards follow kernel32 (45 more).

ntoskrnl's `RtlUTF8ToUnicodeN`/`RtlUnicodeToUTF8N` were "absent" only from the image:
the body is the utf8 crate, which reached the kernel through `libntoskrnl_vista.a` until
that left the link. The kernel's cargo link now takes the crate directly; kmtest
`RtlUtf8` (Windows 11 16/0 first).

Two guards came out of it. A body under the Windows name can be a DIFFERENT function:
shell32's linked `SHBindToObject` is the tree's 4-argument cdecl helper, Windows exports
5 arguments WINAPI, and only i386's link said so (`_SHBindToObject` vs
`_SHBindToObject@20`). `gated` now compares the i386 decoration with the row's argument
size and reports `signature:` instead of `linked`; the export now goes to a WINAPI
`SHBindToObject_Export` over `SHBindToObjectEx`. And 13 linked bodies stay gated in
`NOT_A_BODY` because they do not do the job, so a program keeps its working fallback:
`RtlIsValidLocaleName` (TRUE for anything), `SHCreateItemInKnownFolder` (E_NOTIMPL),
`D3DKMTOpenAdapterFromHdc` (success without an adapter), the clipboard-listener pair
(fails without a last error), and the rest the list names.

**Done 2026-10-13: the first written bodies, top of the app ranking.** 21 new functions,
none of them existing code: apps 271 -> 251 missing functions, 671 -> 541 import sites
(-19%).
- kernel32 (`client/vista_ex.rs`): the `ProcThreadAttributeList` trio (Wine's layout;
  `CreateProcess` accepts `EXTENDED_STARTUPINFO_PRESENT` but does not yet APPLY
  `PARENT_PROCESS` or `HANDLE_LIST` — owed), `CreateEventEx`/`CreateMutexEx`/
  `CreateWaitableTimerEx` A and W, `Get`/`SetThreadErrorMode` (through
  `RtlSetThreadErrorMode`: the TEB holds the RTL_SEM encoding, 0x10/0x20/0x40, not the
  SEM_ values), `RaiseFailFastException`, and `GetLogicalProcessorInformationEx`
  synthesized from the legacy information class (one processor group; the kernel has
  no `SystemLogicalProcessorInformationEx`).
- shell32: `SHGetKnownFolderPath` and `SHGetKnownFolderIDList` over the CSIDL table,
  `SHGetStockIconInfo` (a subset of SIIDs mapped to our `IDI_SHELL_*` icons; the rest
  get the generic document icon).
- ntdll's user-mode ETW (`EtwEventRegister`, `EtwEventWrite*`...): providers register and
  every event is dropped, which is Windows with no trace session; advapi32's `Event*`
  forward to them by their Windows names. iphlpapi's `ConvertInterfaceIndexToLuid` and
  `FreeMibTable`.

`stubs --apply` had silently reverted two of these: an edit made INSIDE the generated
block (advapi32's `Event*` rows, ntdll's `EtwEventWriteEx`) read as finished in the spec
diff and was restored to a stub on the next regeneration. Both `stubs` and `forwards` now
refuse to drop a row from their block that they did not write (`HandEditedBlock`);
move it above the block.

**Done 2026-10-14: the kernel I/O set and the second batch of bodies.** Apps 251 -> 191
missing functions, 541 -> 356 import sites (-34%).
- **Four new system calls, the first added since the fork:** `NtCancelIoFileEx`,
  `NtCancelSynchronousIoFile`, `NtRemoveIoCompletionEx`, `NtFlushProcessWriteBuffers`
  (appended to `sysfuncs.h`/`.lst`, SSDT regenerated by `tools/gen_ssdt.py`, WoW64 manual
  thunks for the three whose structures narrow). kernel32's `CancelIoEx`,
  `CancelSynchronousIo`, `GetQueuedCompletionStatusEx` and `FlushProcessWriteBuffers` sit
  on them.
  - Cross-thread cancel runs as a special kernel APC in each target thread: a thread's
    `IrpList` has no lock and only its owner may walk it.
  - `KeRemoveQueueEx` (exported, as on Windows) and an alertable queue wait: the queue
    wait now makes the same `KiCheckAlertability` test as every dispatcher wait.
  - Test: kernel32 apitest `IoCancelEx`, 42 checks. Windows 11 42/0 first (winref), then
    the guest 42/0 on amd64 and i386. Its first guest run found three bugs of ours: a
    KEVENT initialized in a constructor and returned by value (0x109/0xA; the class is in
    the field manual and `scan_init_then_move` now gates the constructor shape too), a
    user APC queued before an alertable queue wait never ending it, and CancelSynchronousIo
    missing reads on handles opened for synchronous I/O (`FO_SYNCHRONOUS_IO`, not
    `IRP_SYNCHRONOUS_API`). arm64 has no kernel32 apitest: `SetUnhandledExceptionFilter.c`
    does not compile there (owed).
- user32: 26 exports. Most had bodies in `user32_optional_vista-rust` that never reached
  the DLL: a static archive only contributes a member something references, and the gated
  rows referenced nothing, so `gated` reported them "absent" (owed: teach it to look in
  the archives the module links). The Wine stubs among them were made honest for a
  no-touch, one-monitor, 96-DPI machine: one display path from the primary device's
  current mode, touch and gesture handles always invalid, unique power-notification
  handles, the mouse as pointer 1, and a process DPI awareness that `SetProcessDPIAware`
  and `SetProcessDpiAwarenessContext` actually record. Plus 9 DPI exports in
  `user32-rust/src/misc/dpi.rs`.
- iphlpapi: the LUID/name/GUID conversions, `if_nametoindex`/`if_indextoname`,
  `GetIfEntry2`/`GetIfTable2`. GUIDs are our own deterministic encoding, not Windows'.
- kernel32: `QueryThreadCycleTime`/`QueryProcessCycleTime` (CPU time in 100 ns units, not
  cycles), `GetTimeZoneInformationForYear`, `GetDynamicTimeZoneInformation`,
  `GetProcessGroupAffinity`, `GetNumaNodeProcessorMaskEx`, `CallbackMayRunLong`,
  `WerGet/SetFlags`, and `GetNamedPipeServerProcessId`, which fails with
  ERROR_NOT_SUPPORTED because npfs has no `FSCTL_PIPE_GET_PIPE_ATTRIBUTE` (owed).
- Every structure the agents laid out by hand (`MIB_IF_ROW2`, the `DISPLAYCONFIG_*` set,
  `POINTER_INFO`) carries compile-time size/offset asserts measured from the SDK headers.

**Done 2026-10-14 (second batch): WinRT strings, shcore DPI, the Win8/10 kernel32 set.**
Apps 191 -> 125 missing functions, 356 -> 209 import sites (-41%).
- kernel32: 45 private forwards to advapi32 (registry, tokens, `CreateProcessAsUser`):
  `forwards` gained advapi32 as a kernel32 provider, and `LINKS_BACK` is now keyed by
  (module, provider) since advapi32 links kernel32's import library.
- kernel32 (`client/win8_misc.rs`, 24): `SetFileInformationByHandle` (no body existed on
  any arch), `CreateFile2`, `CreateRemoteThreadEx`, Get/SetProcessInformation,
  Set/GetThreadInformation, `GetCurrentThreadStackLimits`, `QueryUnbiasedInterruptTime`,
  `Wow64Get/SetThreadContext`, `GetEnabledXStateFeatures`, the processor-group helpers,
  power requests (an event handle; no power policy is affected) and
  `CompareObjectHandles` over a new system call, `NtCompareObjects` (ob-rust).
  ntdll's status table had no row for STATUS_NOT_SAME_OBJECT, so a correct answer came
  back as error 317; added.
- ole32 (`winrt_string.rs`, `roapi.rs`): the HSTRING API and `Ro*` from Wine's combase,
  in ole32 because combase is not built here and the api-set table routes
  api-ms-win-core-winrt-* to ole32. `RoGetActivationFactory` reads the
  `WindowsRuntime\ActivatableClassId` registry and finds nothing (REGDB_E_CLASSNOTREG, as
  Windows answers for an unknown class); `RoGetApartmentIdentifier` names the apartment
  (Wine's constant made every apartment equal).
- shell32 (`shcore_dpi.rs`): `GetDpiForMonitor`, `GetScaleFactorForMonitor`,
  Get/SetProcessDpiAwareness (over user32's awareness), `Shell_NotifyIconGetRect` (the
  whole notification area). Its error codes are Windows 11's, measured: a bad type is
  HRESULT_FROM_WIN32(ERROR_BAD_ARGUMENTS), a NULL output ERROR_INVALID_ADDRESS, not
  Wine's E_INVALIDARG.
- api-ms-win-eventing-provider now resolves to advapi32 (it was kernel32, which has no
  `EventRegister`; Windows maps it to kernelbase). The generator named in
  apisets_table/mod.rs's header (`rustconv data-port`) no longer exists, so the C table
  and the Rust table were edited together.
- Test: kernel32 apitest `Win8Surface`, 49 checks, Windows 11 49/0 first, guest 39/0 on
  amd64 and i386. Its file block (CreateFile2, SetFileInformationByHandle) SKIPS in the
  guest: the sweep has no writable directory on the live CD (owed: a writable volume
  for the apitest payload, then grade it there).

**Done 2026-10-14 (third batch): every load-time import of the 20-app corpus resolves.**
Apps 125 -> 1 missing function (ucrtbase `_except1`, a stub), 209 -> 1 import site.
Nothing in the corpus fails to load for a missing export of a module we build. Test:
kernel32 apitest `Win10Surface`, Windows 11 95/0 first, guest 96/0 on amd64 and i386.
- Loader: `SetDefaultDllDirectories`/`AddDllDirectory`/`RemoveDllDirectory` and the
  `LOAD_LIBRARY_SEARCH_*` flags (kernel32 `client/dlldirs.rs`, beside
  BaseComputeProcessDllPath, which is where our search path is built; Wine's order and
  error codes). `LoadLibraryExW` used to ignore those flags.
- `GetProductInfo` forwarded to `ntdll.RtlGetProductInfo`, which no target compiled
  (`sdk/lib/rtl/product.c`): ported into ntdll-rust `rtl/version.rs`. `NtOpenKeyEx` lived
  in the arm64/amd64-only WOW64 shim; now every arch. `RtlUTF8ToUnicodeN` and its pair had
  bodies (`utf8` crate) and no rows. `NtGetNextThread` over a process snapshot; the KTM
  calls fail STATUS_NOT_IMPLEMENTED (Wine fakes success) and the TEB's current
  transaction is real.
- api-ms-win-security-base-* now resolves to advapi32 (kernel32 has none of AccessCheck,
  EqualSid & co.).
- kernel32 (`win10_misc.rs`, `pathcch.rs`, `idn.rs`, console): the whole PathCch family
  (Wine's path.c; our api-set host for api-ms-win-core-path is kernel32), XState/context,
  CopyFile2, mitigation policy, AppModel package queries (APPMODEL_ERROR_NO_PACKAGE), Pss
  (ERROR_NOT_SUPPORTED), ResolveLocaleName, SetWaitableTimerEx, UnmapViewOfFileEx,
  IdnToAscii/Unicode/NameprepUnicode (exact RFC 3492; nameprep has no NFKC),
  GetCurrentConsoleFontEx, K32EnumProcessModulesEx.
- psapi: EnumProcessModules and its module lookup read `sizeof(PEB_LDR_DATA)` into a
  pointer variable, overrunning the stack on every call since the Rust port; fixed.
- user32/win32k: AddClipboardFormatListener posts WM_CLIPBOARDUPDATE (a listener list in
  win32k-rust `clipboard.rs`, reached through two NtUserCallHwnd routines numbered outside
  the shared enum, as SETFOREGROUNDWINDOWMOUSE is); pointer/pen queries fail as for an
  unknown pointer id; SetCoalescableTimer.
- shcore.dll is built (against ole32, not the declined combase), its DPI calls forwarding
  to shell32's Windows-graded ones. setupapi/cfgmgr32: SetupDiGetDevicePropertyW, the
  Keys/Set pair and CM_Get_DevNode_Property* over the legacy registry properties
  (DEVPKEY_Device_* pid = SPDRP + 2). iphlpapi `*2` tables/routes/neighbours (IPv4, from
  the legacy tables; change notifications deliver only the initial one), ws2_32
  GetAddrInfoExW (sync+async+cancel), WSAPoll, WSASendMsg; winhttp proxy resolver over
  the module's PAC code; ole32 winrt-error set; advapi32 AddMandatoryAce,
  EventActivityIdControl; esent fails JET_errFeatureNotAvailable; and honest
  not-present answers for AAD join, AppContainer, SHGetPropertyStoreForWindow,
  SHCreateAssociationRegistration.
- Tooling: `pecheck.spec_exports` reported a `-stub -version=` row outside its range as a
  callable stub (it is not exported: a load failure); `win10_surface gap` lists imports
  Windows itself does not export apart from our gap.

**Done 2026-10-15: the missing modules, imported from Wine 11.19.** Every OS module the
corpus imports is built: mfplat, mf, mfreadwrite, evr, wevtapi, ndfapi, dcomp, qwave, tbs
(gap: 9 modules -> 0; functions 5 -> 1, ucrtbase `_except1`). rtworkq and dxva2 are built
at every DLL_EXPORT_VERSION, and rtworkq is synced to 11.19 to match mfplat.
- **Media Foundation has no codecs.** Wine decodes through winegstreamer (a Unix-side
  GStreamer bridge we cannot use), so attributes, media types, buffers, samples, work
  queues, the session, topology loader, sample grabber, sinks and the source reader all
  work, but nothing turns an mp4/AAC/H.264 byte stream into a source or a decoder MFT.
  An app that only plays media through MF loads and then fails at
  CreateObjectFromByteStream (MF_E_UNSUPPORTED_BYTESTREAM_TYPE).
- Found by Wine's suites in the guest (payload `mfwinetest`): our Rust threadpool (a port
  of an older Wine) waited on the caller's handle, so closing one pending wait's handle
  made NtWaitForMultipleObjects fail for the whole wait thread and every other wait in
  the bucket stalled; ported Wine's duplicate-handle + update-serial fix.
- Bodies Wine leaves as stubs: MFCreateMediaBufferWrapper (graded on Windows 11), real
  QoS handles, the tbs context calls, DCompositionCreateSurfaceHandle.
- Guest amd64: wevtapi 10/0, qwave 18/0, rtworkq 176/1 (`flaky_wine`); mfplat completes
  with Wine's own todo_wine lines plus the codec-dependent checks.

**Done 2026-10-15 (later): Media Foundation is Rust, and the corpus is wider.**
- mfplat and mf flipped. Graded on Wine's mfplat, mf and topology suites against the C
  built from the same tree, the two now fail identically (see rust-port-status.md for the
  four translation bugs the comparison found, and the two lints that came out of them).
- The corpus grew from 20 app folders to 34 (`win10_surface.py gap`, 60 apps, 4096 images):
  LLVM, Node.js, qBittorrent, qTox, QEMU, Raspberry Pi Imager, Gpg4win, Docker, Oracle
  VirtualBox, Ubisoft, Rockstar, Logitech Options+ and Microsoft Office. Office alone
  accounts for 219 of the missing functions and most of the 36 missing modules, and
  VirtualBox for 62 (most of them kernel-driver imports). Every other app's imported
  FUNCTIONS now resolve except QEMU's SHCreateShellItemArrayFromIDLists (since ported);
  the modules they need are listed below.
- Bodies for what those apps import: kernel32 GetPhysicallyInstalledSystemMemory,
  GetVolumeInformationByHandleW and five forwards; user32's thread DPI-context set;
  ws2_32 GetHostNameW; iphlpapi NotifyUnicastIpAddressChange and InitializeIpInterfaceEntry
  (measured on Windows 11: 0xFF "unchanged" sentinels, not the zeros the docs describe);
  shell32's explicit AppUserModelID pair (measured too); rpcrt4
  RpcServerInqCallAttributes{A,W}; the bluetoothapis GATT set and bthprops
  BluetoothIsDiscoverable with no-radio errors. ucrtbase `_except1`. `icuuc.dll` (new,
  Rust) for cmake-gui.

- The kernel speaks the NT6 access rights. The process, thread and job types carry the
  Vista+ valid masks (0x1FFFFF, 0x1FFFFF, 0x1F003F) and Windows 11's generic mappings;
  a full information right brings its LIMITED partner on every handle (open, create,
  duplicate), so an NT5-style 0x1F0FFF open is granted 0x1F3FFF as on Windows; and the 48
  process and 22 thread query classes Windows answers through a LIMITED handle accept one
  (measured class by class on Windows 11); likewise the 4 process and 4 thread set
  classes a SET_LIMITED handle may write. This was the NtGetNextThread ACCESS_DENIED: the
  NT5 thread mask silently narrowed THREAD_QUERY_LIMITED_INFORMATION away. Win10Surface
  pins it.
- userenv's DeriveAppContainerSidFromAppContainerName derives the SID as Windows 11 does
  (measured): S-1-15-2 plus the first seven little-endian words of SHA-256 over the
  lowercased UTF-16LE name, 1 to 64 units, the output untouched on error.
- `sechost` and `cryptsp` were never needed as DLLs: the apps import them through api
  sets, and the apiset table pointed 34 contracts at those two hosts, which we do not ship.
  They now go to advapi32 (`update.py` REDIRECT_HOSTS), which exports every function the
  corpus calls through them. `wer` and `virtdisk` are new (Rust), `xinput1_4` is Wine 11's
  HID xinput with XInputGetAudioDeviceIds given a body, and shell32's
  SHCreateShellItemArray / FromIDLists / FromShellItem are ported and exported. That
  closes every function the non-Office, non-VirtualBox apps import, and every module
  except Docker's `mi` and three AMD GPU-driver runtimes a vendor driver installs
  (`amdhip64_6` for Blender, `mantle32`/`mantle64` for Steam).

**Done 2026-10-16: console fonts and condition-variable wakes.**
- GetCurrentConsoleFont, GetCurrentConsoleFontEx and GetConsoleFontSize report the font
  (consrv's frontend vtable gained GetFontSize). As measured on Windows 11: the size is the
  font's unscaled one (Consolas 16 reads 0x16), index 1 fails ERROR_INVALID_PARAMETER, and
  `bMaximumWindow` answers the maximum window size in characters, not a font size.
- A condition-variable wake no longer costs clock ticks on one CPU: the waker released the
  keyed event while holding the CV's lock bit, and the boosted waiter spun on it until its
  quantum ended (field manual, "On ONE CPU, a woken waiter that spins"). A pool work item
  went from 36 ms median / 470 ms worst to ~95 us, which is what mfplat.c:3308's 100 ms
  standard-queue check was catching. `tools/payloads/wakeprobe` measures it. The kernel's
  keyed event now reports a wait that was paired while timing out as satisfied.

**Owed on the export surface:**
- **Modules other apps still need:** `mi` (Docker; WMI's Management Infrastructure, which
  Wine does not have). Office: bring it in when the rest is green, it is its own campaign.
- **d3d11.dll is built only at DLL_EXPORT_VERSION >= 0x600**, so a default build has no
  D3D11CreateDevice at all; Wine's mfplat suite ends on that NULL pointer, and Chrome and
  Office import d3d11. Decide whether to build the d3d family at every version (rtworkq and
  dxva2 already are).
- **mf.c:4553, the presentation clock's callback interval,** reads ~188 ms instead of a tick
  in the second or third mf run of a boot (the first passes), with or without the
  condition-variable fix; the 10 ms periodic pool timer alone runs at 64 Hz in wakeprobe.
  Seen once beside it: mf.c:4493 "got count 0, state 0".
- **mf's converter scenario:** both builds fail mf.c's "SetTopology ... requiring a
  converter" checks (no converter MFT is registered: the codec gap below).
- **Codecs for Media Foundation** (above): native Rust demuxers/decoders, or ports of
  the codec DLLs Windows ships (msmpeg2vdec, msauddecmft, resampledmo). The suites need
  the resampler DMO and the audio endpoint (MMDeviceEnumerator) before they can finish.
- **icuuc under a real Qt:** host-tested only. Run cmake-gui in the guest.
- **Tooling:** `port replay` took 87 minutes on mfplat's 525 functions, and there is no
  single "re-island and reassemble from banks" command. Split island groups are now
  reported by `port island` but still not generated.
- **New partials from the third batch:** iphlpapi change notifications after the
  initial one, IPv6 tables; IDN nameprep without NFKC.
- **Bodies that answer honestly but not fully:** `GetNamedPipeServerProcessId` (npfs has
  no attribute FSCTL); `GetProcessInformation(ProcessMachineTypeInfo)` (no
  NtQuerySystemInformationEx); Get/SetRestrictedErrorInfo stay E_NOTIMPL as in Wine.
- **Then the version flip to 10.0** (Windows 11 is 10.0 too, build 22000+): report it,
  apply Windows' manifest rule in `GetVersionEx` (6.2 unless the manifest declares the
  Windows 10 `supportedOS`; `RtlGetVersion` is never lied to), and audit the ~88 sources
  that branch on the version. Last, because a program that sees 10.0 stops taking its
  XP fallback.
- **Seed the stub-hit report with real programs.** Its first reading (607 boots: smoke,
  kmtest on all arches, and one amd64 appstorm sweep, which opens every stock app and
  applet): exactly one stub is reached, the SEH smoke probe's deliberate
  `msvcrt.dll!_CrtCheckMemory`, and 39 GetProcAddress misses, all probes with a
  fallback (`SvchostPushServiceGlobals` in every svchost service, the optional `BTCpu*`
  hooks in `wow64cpu.dll`, `_set_invalid_parameter_handler` once per CRT process) except
  `kernel32.dll!LCMapStringEx`, which was `gated` (exported 2026-10-13; the appstorm
  sweep after it: 54/54 apps, 38 misses, all probes).
  The ranking only means something over third-party programs (the apitest sweep, Chrome
  once it starts), so opt stubs into `-stub=` from those runs, not before.
- **The next cheap kernel batch is NOT NDIS 6** (this section said it was):
  `PoRegisterPowerSettingCallback`/`PoUnregister…` (61/59 packages),
  `IoGetDevicePropertyData`/`IoSetDevicePropertyData` (38/35) and `strncpy_s` (35) sit
  among the NDIS names. ntoskrnl_vista has bodies for the first four, but they return
  `STATUS_NOT_IMPLEMENTED` (and the kernel does not link that library; this entry said it
  did), so exporting them as they stand would turn a load failure into a start failure. The Po pair want a success that never calls back
  (or calls back once with the current value, as Windows does: grade it there first).
  The NDIS 6 NetBufferList/miniport surface is the big one after that.
- **The gated rows left** (`gated build-amd64`, 2026-10-15: 36 absent, 10 excluded): ntdll's
  hash helpers (`MD5*`, `A_SHA*`) and memory/IO resource codecs (20), ntoskrnl's PoFx
  runtime power (10; ntoskrnl_vista, which the kernel does not link, has NOT_IMPLEMENTED
  bodies), the gdi32 D3DKMT scheduling/adapter calls (7), kernel32's console `*Ex` calls
  (6), shell32's SHCreateItemInKnownFolder and SHEvaluateSystemCommandTemplate (2) and
  activeds' ConvertTrusteeToSid. The `excluded` ones have a body that is not one yet (the
  reason is printed per row).
- **wpprecorder.sys / wpprecorderum.dll have not served a real vendor driver yet.**
- **Re-measure** after any of these: `python3 tools/win10_surface.py reference` (cached in
  `~/.cache/vibeos/win10-surface/`), then `gap` over the same 20 app folders under
  `/mnt/c/Program Files*` and over `/mnt/c/Windows/System32/DriverStore/FileRepository`.

**The oracle for this surface is real Windows 11** (2026-10-11): `vibeos-cli winref` runs
tests built by our toolchain on a Windows 11 VM, so the expected result is measured, not
believed. Its first payload found 104 failures that were all ours (test build and harness),
none Windows'. See [`winref.md`](winref.md).

**10.0.26100 is Windows 11** (the 24H2 base; the measuring host reports 25H2, build 26200,
which shares it). Windows 11 kept NT 10.0 and `_WIN32_WINNT` 0x0A00, which is why the tier
and the tool say "Win10": every number above is the gap to Windows 11, and the Windows 10
gap is a subset that cannot be measured without a Windows 10 install. What export tables
cannot show is the same for both: WinRT activation through `combase`
(`RoActivateInstance` + `.winmd` metadata) for WinUI 3 / Windows App SDK apps, and new
attribute VALUES passed to existing functions (`DwmSetWindowAttribute` for Mica and rounded
corners), which degrade cosmetically rather than fail. Windows 11 has no 32-bit edition, so
i386 is a WoW64 (SysWOW64) target there, not a kernel one.

Known contents, from the design discussion: the newer pool and MDL APIs
(`ExAllocatePool2/3`, `MmMapIoSpaceEx`, no-execute MDL mappings), processor-group calls
(trivially one group), `EtwRegister` and WPP plumbing (stub success first),
`KeSaveExtendedProcessorState`, DMA adapter v3, `PoFx` runtime power (a keep-everything-on
stub first). Real subsystem work, not stubs:

1. **MSI/MSI-X interrupts** through `IoConnectInterruptEx`. Modern GPUs and NICs expect
   them. **MSI-X works (2026-10-07), the Windows way:** a driver opts in with
   `MSISupported` under its device key; pci.sys adds one message requirement per table
   entry (in FILTER_RESOURCE_REQUIREMENTS, since ReactOS queries requirements before
   AddDevice); the HAL translates each into a vector (x86: an APIC vector with no IOAPIC
   pin; arm64: a GICv2m SPI, or on a GICv3 an LPI that the ITS translates from the
   device's requester ID through the IORT) and supplies the address/data pci.sys writes
   into the table; `IoConnectInterruptEx` connects message-based, or falls back to the
   line. The arm64 HAL drives a GICv3 (Graviton's) as well as the Pi's GICv2 since
   2026-10-07: `smoke --arch arm64 --gic 3 [--nvme] [--smp N] [--smmu]`. The ITS backend
   brings up every ITS the MADT lists, uses a two-level device table where the ITS can,
   and spreads each device's messages across the CPUs (one collection per CPU); the
   IORT walk follows a root complex through an SMMU, which the HAL puts in bypass at
   boot since no driver programs an IOMMU. Since 2026-10-08: a device's `Affinity
   Policy` (DevicePolicy, AssignmentSetOverride) rides on its message requirements and
   the HAL places each message on a processor it allows, on all three arches (proved
   by `--tripwire NVME_AFFINITY=<mask>`); arm64 `KeConnectInterrupt` enables on the
   interrupt object's processor, as x86's does, and an ITS LPI enabled elsewhere is
   moved there (MOVI). The PnP manager gives message vectors back to the HAL when a
   device is removed or a translation is abandoned (ITS: DISCARD, then the device
   unmapped), and pci.sys turns MSI/MSI-X off at STOP and REMOVE;
   `--nvme --tripwire NVME_FAIL_START=1` proves that path (nvme.sys connects, then fails
   START, and the boot fails unless the vectors come back). A device with MSI but no
   MSI-X gets a multiple-message block since 2026-10-08, as on Windows: one requirement
   spanning the block, which the HAL translates to consecutive values aligned to its
   size (APIC vectors, GICv2m SPIs, or ITS EventIDs) and halves when it cannot find that
   many together. usbport takes messages for xHCI controllers, so
   `smoke --xhci msix|msi|line` runs the USB rig on each; with `msi` and
   `--tripwire XHCI_MSI_MESSAGES=<n>` the controller gets an MSI block of n. Getting
   xHCI onto MSI-X found pci.sys writing MSI-X tables before memory decode was on (field
   manual). An SMMUv3 is driven for real with `--tripwire SMMU=TRANSLATE` (2026-10-08):
   every device's StreamID gets a stream table entry at START, through one stage-1
   identity map of RAM and the MSI doorbells, and faults are reported
   (`NVME_BAD_DMA=<hex>` proves one is); bypass stays the default, as on Windows
   without DMA remapping. A device whose driver is DMA-remapping compatible (its
   service's `Parameters\DmaRemappingCompatible` = 1, read by pci.sys as Windows does)
   gets a domain of its own (2026-10-09): its own CD, ASID and page tables, which hold
   only what its DMA adapter maps (common buffers, MapTransfer runs, scatter/gather
   lists), refcounted per page and TLB-invalidated on release. nvme.sys takes every
   address from its adapter now and declares itself compatible; `NVME_STRAY_DMA=1`
   proves a DMA to RAM it was never given faults and leaves the page untouched.
   usbxhci (usbport now flushes every transfer it maps), vioblk and netstack's
   virtio-net are compatible too (2026-10-09); the last two find their devices by
   scanning PCI, not through PnP, so they take an adapter for the requester ID from
   the HAL and claim the domain themselves, and pci.sys starting the PDO later keeps
   it. virtio-mmio devices sit behind no SMMU. `CONNECT_MESSAGE_BASED_PASSIVE`
   (2026-10-09): each message's ISR only marks it pending and queues a DPC, which
   wakes the connection's worker thread to call the driver at PASSIVE_LEVEL under a
   mutex that KeSynchronizeExecution takes too; with no messages it falls back to
   the line, masked until the routine has run. `--tripwire NVME_PASSIVE_ISR=1` runs
   nvme.sys on it (KMDF 1.17 connects only `CONNECT_FULLY_SPECIFIED(_GROUP)` and does
   passive handling itself). MOVI is proven by `--tripwire ITS_MISSTEER`, a segment past 0 by
   `--nvme-segment`. The arm64 HAL's PCI config space comes from the MCFG since
   2026-10-08: every bus of every segment, bus numbers being PCI_SEGMENT_BUS_NUMBERs
   (pci.sys keeps the segment through bridges and in the requester ID the ITS routes
   by), and the HAL publishes a root bus for each segment past 0. `smoke --arch arm64
   --nvme-bridge` runs nvme.sys behind a PCIe root port on bus 1, and `--nvme-segment`
   on segment 1 (QEMU's virt machine has one segment, so the HAL presents a pxb-pcie
   expander's buses as a second; `PCI_SEGMENT_SPLIT`).
2. **Write-combined mappings** (`MmWriteCombined` through a correctly programmed PAT).
   Getting it wrong is mostly a performance problem, but it can show up as visible
   corruption.

None of this re-architects ntoskrnl.

---

## Boss 1: modern hardware drivers

Since Vista, Windows drivers are written against **frameworks plus class extensions**, not
raw WDM. Support a framework and a whole category of hardware runs its Windows 10 driver
unchanged.

| Order | Area | What it takes | Notes |
|---|---|---|---|
| 1 | **KMDF wired up** | KMDF is built. What's left: version negotiation (drivers request specific WDF versions; ours is 1.17) and real drivers loaded on it | The master key: most modern drivers are KMDF |
| 2 | **NVMe** | **First version done** (`nvme.sys`, 2026-10-06): a PnP driver that presents namespaces straight to disk.sys, not a StorPort miniport, because our StorPort never completes an SRB (its PDO handler and bus relations are stubs) and scsiport's legacy PCI scan cannot assign resources on arm64. INTx interrupts since the same day, MSI-X and a 63-deep queue with direct DMA since 2026-10-07, a queue per CPU and controller reset on a hung command since 2026-10-08. Owed: finishing StorPort, which vendor NVMe and RAID drivers still need | No modern PC boots without it. The test box has one (below) |
| 3 | **USB 3** | Our xHCI driver exists and works. The remaining question is compatibility with Windows 8+'s USB stack shape (`usbxhci` + `ucx01000` class extension + `usbhub3`) for vendor USB drivers that bind to it | Much further along than the rest |
| 4 | **NDIS 6.x, then NetAdapterCx** | Modern Intel/Realtek NIC drivers are NDIS 6; the newest are NetAdapterCx (Windows 10, on KMDF) | Our NDIS is 5-era |
| 5 | **HD Audio and the Vista audio stack** | Kernel: WaveRT miniports on PortCls (Realtek HDA drivers). User mode: the audio engine (`audiodg`) and WASAPI, which every modern app (Chrome included) talks to | Wine has WASAPI (`mmdevapi`) |
| 6 | **Wi-Fi** | NDIS 6 + Native WiFi / WDI + the `wlansvc` service | Large; it's what makes laptops viable |
| 7 | **Laptop input and sensors** | HID over I²C (`hidi2c`) on `SpbCx` (the I²C/SPI bus) and `GpioClx` (GPIO interrupts). Precision touchpads are this stack | Same KMDF + class-extension pattern |
| — | **UMDF** | User-mode driver host plus its kernel reflector. *(unverified)* The host and reflector were not part of Microsoft's open-source drop, so they would be ours to write | For sensors, some HID, printers |
| — | **ACPI 5.x** | Newer firmware tables; lids, batteries, power buttons, GPIO-signalled devices. Modern Standby stays out of scope; S3 is enough | |

**The pitch:** vibeos boots a modern PC from NVMe, with USB 3, Ethernet, Wi-Fi and audio
all running their own Windows 10 drivers, unmodified.

---

## Boss 2: a real GPU — XDDM, then WDDM 1.1, then WDDM 2.x

### How different WDDM is

| | XDDM (what we have) | WDDM |
|---|---|---|
| Where the driver lives | Mostly kernel: a display DLL inside win32k plus a miniport under `videoprt` | A thin kernel driver, plus a heavy **user-mode driver** loaded into each process (builds GPU command buffers, compiles shaders) |
| GPU memory | The driver manages it alone | **dxgkrnl** manages it: allocations, paging between VRAM and system RAM, GPU virtual addresses from WDDM 2.0 on |
| Sharing the GPU | One owner, no scheduling | dxgkrnl **schedules** work from many processes |
| GPU hang | Bugcheck | **TDR**: dxgkrnl resets the GPU and the OS survives |
| Display setup | Mode-set calls | Display topology management (VidPN) |
| Desktop drawing | GDI draws straight into the driver | GDI draws into memory; a compositor (DWM) presents it. DWM is optional on Windows 7, mandatory from Windows 8 |

**What keeps win32k from a rewrite:** since Windows 7, a **Canonical Display Driver**
(`cdd.dll`) sits in win32k's display-DLL slot and translates the XP-era drawing interface
into dxgkrnl calls. We'd write a CDD-equivalent as an adapter. win32k keeps its interface,
and composition can come later because drivers don't require it.

The rest of the stack: `dxgkrnl.sys` + `dxgmms2.sys`, the `D3DKMT*` bridge (our thunks,
populated), D3D runtimes and DXGI wired to the real path instead of Wine's OpenGL path, and
the Windows 10 kernel exports from rung 0. Our product driver roadmap sizes dxgkrnl at
100K+ lines. *(unverified)* Upstream ReactOS has an experimental dxgkrnl effort; check how
far it got before starting ours.

### The ladder

The test card is a Radeon HD 5450 (AMD, ~19 W, fanless) in the OptiPlex 7010.

1. **XDDM driver: AMD's XP x64 Catalyst.** It uses the driver model we already have.
   amd64 needs the XP x64 driver; the i386 build would take the 32-bit XP one. Old NVIDIA
   cards work the same way with NVIDIA's XP drivers. *(unverified)* Exact last-supporting
   driver versions.
2. **WDDM 1.1: the card's Windows 7-era driver.** *(unverified)* AMD's support for this
   card family ended around Windows 7/8.1 with a WDDM 1.1–1.3 driver. That is a much
   smaller dxgkrnl than WDDM 2.x: no GPU virtual memory, a simpler scheduler. WDDM has
   version negotiation, so the driver declares which versions it speaks and dxgkrnl
   tells it which one it gets.
3. **WDDM 2.x: a modern NVIDIA card.** Adds GPU virtual memory and the newer scheduler.

### Oracles

There's no C to translate here, which is new for this project. These replace ReactOS
C as the correctness reference:

- **The documented interface.** Every dxgkrnl ↔ driver call is public in the WDK
  (`d3dkmddi.h`) and Microsoft's docs, because third parties write the drivers. What isn't
  documented is dxgkrnl's internal behaviour: call order, timing.
- **Real Windows as a black-box oracle.** Run Windows on the same box with the same driver
  and record the call trace at the driver boundary: dxgkrnl's event tracing (what GPUView
  reads) plus kernel-debugger breakpoints on the vendor driver's entry points. Then make
  our dxgkrnl drive the same driver and compare traces: `difftest-contract` at the driver
  boundary. **Clean-room rule: documented interfaces and observed traces only; never
  disassemble Microsoft's binaries.** Where exactly that line sits is the owner's call.
- **Open-source WDDM drivers, both sides visible, in QEMU:** Microsoft's KMDOD sample
  (display-only, the smallest real WDDM driver), then the virtio-gpu and QXL WDDM drivers
  (`viogpudo`, `qxldod`). Most of dxgkrnl's basics can be built and smoke-tested in the
  normal QEMU harness before any real GPU is involved.

### Hardware safety

With a vendor driver on board, our bugs are bugchecks, hangs, black screens and GPU-hang
timeouts, not damage. The vendor driver does all clock, voltage, fan and power
programming, and GPUs throttle and shut down on their own firmware. Rules for agents:

- **Never write firmware:** not the card's VBIOS, not the board's flash. That is the
  only path in this plan that can brick hardware.
- **We host vendor drivers; we do not program GPUs.** Writing our own GPU driver is
  where real risk starts (voltage regulators, clocks). It is out of scope.
- **Scratch disk only.** Bad DMA addresses can scribble over RAM and from there over the
  disk. That's data loss, not hardware damage.
- **LCD monitors only.** A bad mode shows "out of range"; only a CRT could be harmed.

---

## Boss 3: the latest Google Chrome

Wine runs Chrome on Linux, and nearly all of what that takes is user-mode DLLs we already
take from Wine or can: DirectWrite (Chrome's only text renderer on Windows), WASAPI,
`bcrypt`, `winhttp`, `iphlpapi`, `combase`, the `api-ms-win-*` forwarders. What Wine does
not give us is whatever Wine does on the Linux side of its kernel boundary. For us, that's
ntoskrnl work, and it's mostly the sandbox.

### What it needs, in rough order

1. **To think it's on Windows 10.** Chrome 110+ refuses older versions and is manifested
   for Windows 10. Reporting 10.0 is the obvious fix, with side effects elsewhere in the
   OS to manage.
2. **The Windows 10 user-mode API surface**: Vista thread pools and SRW locks, Windows 8's
   `WaitOnAddress` and `GetSystemTimePreciseAsFileTime`, Windows 10's
   `SetThreadDescription`. *(unverified)* PartitionAlloc's use of `VirtualAlloc2` /
   `MapViewOfFile3` (Windows 10 1803). Most of this is in Wine; ReactOS's `*_vista` DLLs
   cover part of it.
3. **The sandbox** (below). The real gap.
4. **Graphics is not a blocker.** Chrome composites with D3D11 through ANGLE, but with
   `--disable-gpu`, or no usable GPU, it uses SwiftShader, its built-in software renderer.
5. **The rest is moderate:** Winsock with I/O completion ports, network change
   notifications, WinHTTP for proxy settings, Crashpad (exception handlers and minidumps),
   TSF for IME input. Chrome brings its own TLS (BoringSSL) and root store.
6. **Skip the installer at first.** Its installer and updater need services, the Task
   Scheduler COM API and a lot of COM. A portable Chromium zip avoids all of it.

### The sandbox: the contract, not Windows' internals

Chrome's sandbox has the same shape on Windows and Linux: a privileged **broker** plus a
restricted **target**, with blocked calls forwarded to the broker. **Chrome writes the
broker on both platforms.** Our share is four kernel primitives, and three of them have a
Linux design to copy.

**Contract only** (one-way restrictions: Chrome checks that setup succeeded, then only
needs forbidden things to fail):

| Windows feature | What Chrome needs | Linux design to copy |
|---|---|---|
| win32k lockdown (`ProcessSystemCallDisablePolicy`) | Renderers' win32k system calls fail | **seccomp**: a per-process filter flag checked in the syscall dispatcher |
| Job objects | One-process limit (renderers can't spawn), UI limits, memory caps | cgroups / `RLIMIT_NPROC` |
| Named-object isolation (AppContainer) | A private object namespace per container | mount and IPC namespaces |
| Integrity levels / restricted tokens | Documented access-check results: low integrity can't write up; restricted tokens are checked twice | **No Linux analogue.** The NT model is its own, but the algorithm is documented and our `Se` access check is already Rust; integrity levels are an addition to it |

**Bring-up:** because these are restrictions, Chrome *works* if the calls succeed before
we enforce them. Phase one accepts them, **loudly marked not enforced** (a log line and a
status flag, never a quiet success). Real enforcement follows. An unenforced sandbox must
never read as a working one.

**Mechanism** (must match Windows exactly):

- **ntdll stub bytes.** The sandbox patches a child's ntdll in memory before it runs:
  it recognises Windows' exact syscall-stub byte pattern and redirects the call to the
  broker. Our stubs for the intercepted calls (`NtCreateFile`, `NtOpenFile`,
  `NtOpenProcess`, `NtMapViewOfSection`, …) must match Windows 10's layout, including the
  `KUSER_SHARED_DATA` syscall-mode check, so that field must exist at the right offset.
  *(unverified)* Whether an unrecognised pattern is fatal: read
  `sandbox/win/src/service_resolver*` in Chromium.
- **Creating the child process.** `CreateProcess` with the extended attribute list
  (inherited-handle list, mitigation policies, AppContainer capabilities). Then the
  parent writes into the suspended child and duplicates handles into it before resuming
  it. These are the cross-process primitives the spawn32 WoW64 ladder guest already tests.
- **Token API details**: specific SIDs and info classes, `NtCreateLowBoxToken`.
- **A separate desktop for renderers**: window-station and desktop object security in
  win32k.

### Oracles

- **Chromium's source** (`sandbox/win/src/`): which byte patterns it accepts, which calls
  must succeed, what it checks afterwards. No reverse engineering needed.
- **Chromium's sandbox test binaries** (`sbox_integration_tests`, `sbox_validation_tests`):
  portable Windows programs that check the OS enforces each restriction. They make a
  ready-made conformance suite for the sandbox rung, like the WoW64 parity guests.

### The ladder

A smoke-harness ladder test, like spawn32:

1. `chrome --headless --no-sandbox --dump-dom file://…`: no UI, no GPU, the smallest real run
2. The same against an `https://` URL (networking, TLS, DNS)
3. A normal Chrome window with `--no-sandbox --disable-gpu`
4. Sandbox on, unenforced first and then enforced. The kernel work lands here
5. GPU on, when Boss 2 gets there

Rungs 1–3 are mostly user-mode Wine plumbing: probably the fastest "wow" in this whole
doc. Chrome also ships native ARM64 Windows builds, so the arm64 port is a real target
too: Chrome on a Pi.

---

## The test bench: OptiPlex 7010 (homelab)

- Has an **NVMe drive on a PCIe adapter**, booted through **Libreboot** (owner flashed
  it). So the NVMe miniport gets real hardware from day one.
- Libreboot's payload is SeaBIOS, not UEFI. **amd64 boots from legacy BIOS again**
  (restored 2026-10-05, gated as `smoke:amd64-bios`; see `bios-boot.md`). That was
  built for the owner's OptiPlex 9020, which runs Libreboot with SeaBIOS.
- First milestone: amd64 boots on it at all, using the integrated Intel graphics through a
  firmware framebuffer. We have only ever booted x86 in QEMU.
- To make it an agent bench like the Pi 4 (`pi4-bench.md`): **remote power** (a smart
  plug), **a serial console** for kernel output (*(unverified)* whether this board has a
  port), and **network boot** so test builds boot without re-flashing anything.

---

## Order of attack

1. Bench bring-up: amd64 boots on the 7010 with firmware graphics.
2. Rung 0: the import-gap census tool and the Windows 10 export surface it ranks.
3. In parallel, by owner priority:
   - **Chrome rungs 1–3** (mostly Wine plumbing; the cheapest big demo)
   - **NVMe**, then **KMDF on real drivers**, then NDIS 6, then audio
   - **GPU rung 1**, the XP x64 driver
4. Chrome sandbox (the four primitives, plus the stub bytes).
5. dxgkrnl, oracle-first: KMDOD in QEMU, viogpu, then the HD 5450 on WDDM 1.1, then 2.x.
6. Wi-Fi, laptop input, UMDF.
