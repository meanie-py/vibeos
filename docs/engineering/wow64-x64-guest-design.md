# Running x86-64 guests on arm64: ARM64EC vs. our own BTCpu64 contract

**Status: DECIDED 2026-10-05 — Fork B first. M1 DONE 2026-10-05; M2 DONE 2026-10-06 — the x86-64
ladder is at parity with the 32-bit one (see "Status" below).** Written 2026-09-13 at the
owner's request as scoping, after an audit established that FEX's x64 support is real and every
blocker is on our side (`docs/product/vibeos-roadmap.md`, and the audit summary below).

**The decision (owner, 2026-10-05): build Fork B (our own BTCpu64-shaped contract) first.** Most
of what B needs, A needs too — a 64-bit FEX plugin, x64 CONTEXT conversion, the guest
exception and syscall path — and B runs x64 apps on the amd64 system DLLs we already build,
fully emulated, the same shape as today's 32-bit path. Fork A (ARM64EC) stays the later
speed-up: it runs system DLL code natively, but only once our userland is compiled ARM64EC /
ARM64X. (Not "ARM64EC is opt-in on Windows": on Windows 11 ARM every x64 app runs in an
ARM64EC process. What is opt-in there is a developer porting their OWN code to ARM64EC.)
First milestone: a `hello64` guest that enters FEX in 64-bit mode and makes one syscall.

The question this answers is not "can FEX run x64" — it can, and does, and the recompiler is
already compiled into the `libwow64fex.dll` we ship today. The question is **which OS-side
contract we implement to reach it**, because we have inherited Microsoft's answer without ever
deciding to.

---

## Status — what exists, and what is owed

**M2 (2026-10-06): parity with x86-on-arm64.** The 32-bit ladder's ten guest sources, rebuilt
for x86-64 (`gui64 win64 file64 async64 marshal64 fault64 thread64 selfmod64 reg64 spawn64`,
plus M1's `hello64`), all exit `0x600D` under FEX on arm64 AND natively on amd64, with the
32-bit ladder green on the same boots: arm64 `/tmp/vibeos-cli/20261006-020111-v3xu401s-smoke-arm64`
(24 guests at 0x600D, gui32 at its graded 0x600E), amd64 run dirs in the commit messages. They
run through the guest's OWN amd64 ntdll, loader, kernel32/user32/gdi32/advapi32/ws2_32 — real
Win32, not raw syscalls — so this covers: process init, the import closure, file and overlapped
I/O, IOCP, the registry (incl. the 64-bit view), win32k drawing, window creation with
callbacks into the guest and WM_PAINT, guest SEH through `.pdata` (hardware fault, software
raise, debug print, Get/SetThreadContext), threads (TLS, per-thread SEH, 4 concurrent), runtime
code generation incl. kernel-side writes into code, and process creation incl.
CreateRemoteThread into a running x86-64 child.

**Resume here (2026-10-17).** The full FEX ladder is green at 1 GB on master: 27 guests at
`0x600D`, i386 included once wow64 completed the loader's `int 0x2d` debug services (field
manual, "Every i386 guest under FEX dies at `_DebugService+0x17`"). The land gate does NOT boot
these guests, and that has already cost one red: `ac29134e356` (ProbeForWrite touches pages)
turned selfmod rung 7 red on master from its landing (2026-10-12), because the guest's acceptance test knew only
`STATUS_PARTIAL_COPY` and nobody ran the ladder after the merge. Before landing anything that
touches wow64, wowx64, FEX or Mm, run `./vibeos-cli smoke --arch arm64 --ladder` on `VIBEOS_FEX=1`
media (and `--arch amd64 --ladder` for wow64.dll). Owed, most valuable first:
1. Free-RAM drift: located, and the first half fixed (`docs/engineering/mm-section-cache-standby-design.md`).
   `--tripwire MEMCENSUS=1` (`ARM3/census.rs`) put it in the `ros` column: section pages held by
   their segment after the last view went, 4.4k -> 30.9k pages over a 2G appstorm64 sweep. arm64
   could not free them at all (a bring-up `#ifdef` in `MmPageOutPhysicalAddress`), and the
   balancer trimmed only at 256 free pages. Now: the segment path on every arch, trimming below
   `MmLowMemoryThreshold`, and a starving user fault retries while the balancer makes progress.
   `smoke --vm-memory 1024M` overrides a payload's declared RAM for exactly this sweep. Reclaim
   exposed `MmCheckDirtySegment`'s invented `IoSetTopLevelIrp(0x80000000)` (fastfat 0x23 under
   pressure; field manual, "The LLM invents plausible values"). appstorm64 now passes 54/54 at
   1088M (wedged before) and 1152M; 1024M still runs out. Next is step 2 of the design doc: a
   real standby state, so cached pages count as available.
2. Commit not yet charged: copy-on-write pages of image and file views, and kernel pool
   (`ARM3/commit.rs` header). PEB/TEB pages are charged since 2026-10-17. Owed with it: FEX's
   CPU layer faults when a commit is refused while setting up a new guest thread
   (`20261017-062113-2pae52ry`): rpmalloc's `span_allocate_page` calls the `memory_commit` hook
   (a VIBEOS patch; FEXCore `AllocatorHooks.cpp` `FEX_rp_memory_commit`), which returns `void`,
   and then writes the page header into memory that was never committed. The fix is in
   `vendor/fex`, whose CLAUDE.md forbids AI-written code, so it is for a human or upstream. Seen as an
   apc32 AV when a ladder ran on 1G appstorm64 media, whose ramdisk leaves a 137 MB commit
   limit with appstorm running beside the guests. On plain `VIBEOS_FEX=1` media the ladder is
   green with the charging.
3. The cnt CRT's NUL scans (`cstr_slice`/`wcstr_slice`, under strlen, wcslen and every string
   function that measures its argument) are asm now (`memops.rs`, 2026-10-17). Owed: the
   compare/search loops (`strcmp`, `wcschr`, ...), which are still unoptimised Rust; measure
   first.
4. win32k callbacks cost ~2x native under FEX.
5. An appstorm64 sweep on a quiet host, re-timed against the per-syscall numbers.
6. Fixed 2026-10-17: `notepad-save` (appstorm read the size before notepad had written the
   file; it now waits for EM_GETMODIFY to clear) and `cpl-console` (86 s and PASS after
   `MiFlushTbAndCapture` stopped flushing the whole TLB once per re-protected page on arm64:
   `NtProtectVirtualMemory` cost ~12 ms a call under FEX). Also fixed: wow64's LOADMENU and LPK
   callbacks had no 64->32 marshaller, so no x86 WoW64 window had a system menu
   ("No Menu!!"). Open: iexplore at 1152M once failed to close (a thread that never finished
   exiting after a live HTTPS connection; netstack's IRPs are cancellable, so not that); an
   arm64 BUGCHECK 0xA at high IRQL in a timer DPC, with the fault one page above SP
   (`20261016-233339-xs4f9y8m`), not seen again in 8,000+ reclaims since. Item 4's callback cost
   looks structural: one GetContext, two SetContext and a nested Simulate per callback.

They are graded where the media carries them (`tests/qemu/cli/ladder.py` `STAGED_LADDER`): they
need build-amd64 to exist at all, so an arm64-only tree does not owe them. `hello64` (no
imports) stays in the always-owed `LADDER`.

| piece | where | what |
|---|---|---|
| contract (v3) | `sdk/include/reactos/wowx64/btcpu64.h` (+ `wowx64-rust/src/contract.rs`) | BTCpu-shaped verbs, x86-64 `WOWX64_CONTEXT`, a host-services table. v3 adds `BTCpu64ThreadTerm` for thread-exit cleanup; v2: the CPU layer passes the 4 register args + guest rsp; the OS side reads exactly each service's stack args (v1 copied a fixed 11 words and read off the top of a fresh thread's stack) |
| CPU layer | `tools/fex/wowx64fex/` -> `libwowx64fex.dll` | upstream `WOW64/Module.cpp`'s shape; adds the ReadFile/MemoryDirty notifications for kernel-side writes |
| OS layer | `sdk/lib/rust/wowx64-rust` -> `wowx64.dll` | `host.rs` entry + guest-entry path; `image.rs` maps/rebases the guest ntdll; `syscall.rs` the forwarder and its exceptions; `except.rs` guest exception delivery; `callback.rs` win32k callbacks; `apc.rs` user-mode APCs into guest code; `thread.rs` thread creation, exit cleanup + stack ownership |
| guest DLL set | `qemu/mkramdisk.sh` -> `<SystemRoot>\sysamd64` | build-amd64's full import closure of a stock GUI app (`wow64-closure.py --include-prestaged`) + `lpk.dll` |
| native ntdll | `ntdll-rust` `LdrpLdrData`, `dispatch/mod.rs` `WowX64UserCallbackHook`/`WowX64UserApcHook` | the loader reads its OWN module list/lock (two ntdlls share one PEB); the callback and APC hook slots |
| guests | `tools/fex/hello-compute/*32.c` built twice by `build-hello-compute.sh` | the width-specific parts (`fs:[0]` SEH vs `.pdata`, WoW64-view expectations) are `#ifdef _WIN64`; the i386 builds are byte-identical to before |

**The design, in the places it is not obvious:**

- **Shared PEB and TEB.** The guest's TEB is the thread's TEB; its PEB is the process's. That is
  what makes every kernel-written per-thread field (win32k's `Win32ClientInfo`, last-error) just
  work, and what WoW64 cannot have. The cost is a short list of things each side must keep to
  itself, each found the hard way: the loader's module list and lock (native unwinding looked up
  FEX's `.pdata` in the guest's list), the TEB stack bounds (below), the `InDbgPrint` flag (a
  guest DbgPrint silenced every native trace from the VEH), and the heap block layout (below).
- **One heap format for every 64-bit target.** Both ntdlls reach the process heap through
  `PEB->ProcessHeap`, so their heap code must agree on what a block is. rtl-heap's aarch64
  `HEAP_ENTRY` had its own field order that heap.h never had; it now uses heap.h's `_WIN64`
  layout, as x86_64 always did.
- **TEB stack bounds describe the GUEST stack.** The guest's amd64 `RtlDispatchException` rejects
  any frame outside them; the native stack the CPU layer runs on is fully committed instead,
  so it never needs guard-page growth (which checks those bounds).
- **The system32 redirect is conditional.** A name that exists in `sysamd64` is redirected there;
  any other name stays in system32 (the guest's own exe, data files) — unless system32 holds an
  image for another machine there, which the guest gets as "not found", as WoW64 would give it.
  (Unconditional, `spawn64` could not relaunch itself; with no foreign-image check, gdi32's
  `LoadLibrary("lpk.dll")` mapped the arm64 one.)
- **Threads** (`NtCreateThread`): the guest CONTEXT is parked in the target and the kernel gets a
  native one starting `WowX64GuestThreadStart`; that works into a running x86-64 child too
  (wowx64.dll sits at the same base in every guest process). The initial thread of a NEW x86-64
  process needs no context at all — its native loader diverts to wowx64.

**Done this round (apc64.exe is the oracle — x86-64-only, no 32-bit twin):**

- **User-mode APCs into a guest thread** (`QueueUserAPC`, `ReadFileEx` completion routines,
  `NtQueueApcThread` with a guest routine). The native `KiUserApcDispatcher` asks
  `WowX64UserApcHook` (dispatch.S) first; a guest routine runs the guest's own
  `KiUserApcDispatcher` in a nested simulate (`apc.rs`), exactly as `callback.rs` runs a win32k
  callback. Chained APCs at one alertable wait needed a **kernel** fix: arm64 `NtContinue` exits
  through `KiExceptionExit` and, unlike amd64 and unlike the SVC-dispatch tail, did not deliver a
  user APC that `KeTestAlertThread` had just marked pending — so the loop by which
  `KiUserApcDispatcher` drains more than one APC per wait stalled after the first (field manual,
  "arm64 NtContinue(TestAlert) drops a pending user APC").
- **Per-thread CPU-layer state on thread exit** — `BTCpu64ThreadTerm` (contract v3, ported from
  upstream `BTCpuThreadTerm`) frees FEX's `Threads` entry, the frontend data, the GDT and the
  JIT thread state; `NtTerminateThread` (`thread.rs`) calls it for the calling thread before its
  own termination and, for another thread of the process, after waiting for it to die. The
  native stack we allocated is freed the same way the kernel frees a thread stack
  (`FreeStackOnTermination` + `DeallocationStack`). A **force-terminated** thread still leaks its
  own thread-local CRT/allocator heap — the terminator cannot run the victim's cleanup — exactly
  as on Windows; apc64 rung 7 asserts only that the reclaimable part (JIT + native stack) comes
  back.
- **`NtCreateThreadEx` guard** — still not in the service table (a guest calling it gets
  `STATUS_INVALID_SYSTEM_SERVICE`), but `wowx64-rust/build.rs` now fails the build the day it
  lands without an intercept, so it cannot silently start an arm64 thread from an x86-64 CONTEXT
  (`MUST_INTERCEPT_IF_PRESENT`).
- **An x86-64 guest starting a 32-bit child** (spawn64 rung 9, `syswow64\hello32.exe`): the new
  32-bit process's initial thread is the kernel's to start through its own `Wow64LdrpInitialize`,
  so `NtCreateThread` hands the kernel a harmless native context, as for a new x86-64 child.

**Done in the round after (spawn64 rungs 10-12):**

- **CreateRemoteThread from an x86-64 guest into a RUNNING native or 32-bit process.** Nothing
  of ours runs in the target and nothing has to: the routine is already the target's code. Only
  the CONTEXT's shape is x86-64, so `thread.rs` `CreateForeignThread` translates it into the one
  a native creator builds — kernel32's Win32 shape (`rip` = the guest kernel32's
  `BaseThreadStart`, `rcx`/`rdx` = routine/parameter) into the TARGET's native kernel32
  `BaseThreadStart(routine, parameter)`, found through its loader list and export table; the
  RtlCreateUserThread shape into `routine(parameter)` returning into `RtlExitUserThread`.
  (`BaseThreadStart` is a new kernel32 export of `BaseThreadStartup`, XP x64's name, on every
  arch.) A native target just runs it. A WoW64 target's native loader sees a start that is no
  native image's code and hands the thread to wow64.dll (`ldrinit.rs LdrpDivertWow64Thread` →
  `Wow64StartGuestThreadFrom64`), which gives it a 32-bit stack and runs it as a 32-bit thread
  through the 32-bit loader — Windows' `ThunkStartupContext64TO32`, and the same path a NATIVE
  64-bit creator's remote thread into a WoW64 process now takes too (on amd64 that one used to
  run 32-bit code as 64-bit and take the target down).
- **No native kernel32 in an x86-64 guest process.** arm64 ntdll's `_errno`/`atan2`/`tan` are
  its own now (`ntdll-rust crtexports.rs`; the math the linked CRT's), not forwarders to msvcrt,
  so neither msvcrt nor — through msvcrt's imports — kernel32 is loaded any more; rung 12 checks
  every non-x86-64 image mapped in the process. `errno` lives in `TlsSlots[19]`, reserved in
  `PEB->TlsBitmap` at process init. The first attempt at this was backed out after it crashed
  every 32-bit guest in libwow64fex's rpmalloc, blamed then on its `errno` home; the real cause
  was that **a WoW64 process's fixed-index TLS slots (wow64.dll's 1/10/12, FEX's 15..18) were
  never reserved in the bitmap.** The first `TlsAlloc` in the process gets slot 1 — msvcrt's,
  harmlessly, while the forwarders dragged it in early; libwow64fex's rpmalloc, overwriting the
  CPU area, once they did not. Those slots are reserved now in every WoW64 process (field
  manual, "A WoW64 process's fixed TLS slots").

**Done in the third round (spawn64 rungs 13-14, its native helper `xinject.exe`):**

- **Into an x86-64 guest FROM a native process** (CreateRemoteThread and RtlCreateUserThread).
  The creator built a native CONTEXT for x86-64 code: kernel32's Win32 shape starts in the
  native kernel32's `BaseThreadStartup`, which a guest process does not load, and the Rtl shape
  starts at the routine itself. The guest process's native loader hands any new thread whose
  start is no native image's code to wowx64 (`ldrinit.rs LdrpDivertWowX64Thread` →
  `thread.rs WowX64StartGuestThreadFromNative`), the reverse of `CreateForeignThread`: it
  tells the shapes apart by the native kernel32's `BaseThreadStart` address, gives the thread
  a guest stack and its CPU-layer state, and runs the routine through the GUEST kernel32's
  `BaseThreadStart` (either shape then exits through ExitThread with the routine's value). The
  creator's stack becomes the thread's native stack, freed with it. A Win32-shape start around
  a NATIVE routine is rewritten to routine(parameter) and run natively. A thread can arrive
  before `WowX64LdrpInitialize` has finished (the loader publishes process init just before the
  handoff), so it waits for the guest to be up.
- **A new native arm64 child of an x86-64 guest.** system32 no longer hides an arm64 EXE from a
  guest, only an arm64 DLL (`syscall.rs ForeignDll`), as Windows on ARM does not redirect an
  x64 process's system32 at all. A native child's loader DOES run the initial context, so
  `CreateNativeChildThread` turns the guest kernel32's process start into the native
  kernel32's `BaseProcessStart(entry, PEB)` (`BaseProcessStart` is a new kernel32 export of
  `BaseProcessStartup` on every arch, as XP x64 had).
- **Where the native kernel32's starts are**, in a process that has no native kernel32:
  `image.rs NativeSystemExport` maps `system32\kernel32.dll` as an image, reads the export's
  RVA, and adds the PREFERRED base — every native process loads kernel32 there, since there is
  no ASLR and nothing else is linked at that address. If that ever stops being true, both paths
  above break together.
- **TerminateProcess of a guest that is busy running guest code** — 32-bit and x86-64, rung 14
  — ends it, and its handle signals with the exit code it was killed with. The earlier
  suspicion, recorded after a 32-bit guest hung on a data-section map, that such a process
  outlived its termination is now measured rather than inferred.

**Not owed, by design:** CreateRemoteThread from a 32-bit (WoW64) process into an x86-64 guest
is refused (`STATUS_NOT_IMPLEMENTED`, wow64_manual.c `Wow64pCreateForeignThread`), as Windows
refuses a WoW64 process's remote thread into any 64-bit process.

**Owed:** nothing in the thread/process matrix. `NtCreateThreadEx` is still not in the service
table; the build fails the day it lands without an intercept (above).

**Stock x86-64 programs (2026-10-08, payload `appstorm64`).** Every guest above is one we wrote.
`VIBEOS_FEX=1 VIBEOS_PAYLOAD=appstorm64 ./vibeos-cli build --arch arm64` stages appstorm's
x86-64 build as the StartUp program, plus the stock amd64 build of every app and applet it
launches, with their import closure, into sysamd64 (`guest_sets` in its payload.yaml, computed
by `wow64-closure.py --emit-roots`). appstorm checks each launched process's PE machine against
its own, so a native app cannot be scored as an x86-64 pass. That check caught the first run
doing exactly that: notepad, regedit and explorer also live in `%windir%`, outside the redirect.
Run `20261008-011853-dqktsbut-smoke-arm64`: **25 of the 40 apps pass as x86-64 processes**
(notepad, calc, regedit, taskmgr, cmd, mspaint, wordpad, charmap, the three games, winver, clipbrd,
sndvol32, magnify, osk, eventvwr, devmgmt, servman, msconfig, dxdiag, mplay32, sndrec32, and
explorer opening a folder and My Computer). Getting there fixed:

- win32k gave a system-class window (ScrollBar...) the NATIVE user32's proc, because the first
  user32 fills the global client-proc tables. An amd64 user32 shares the native one's base but
  not its layout. `win32k-rust/src/pfnclient.rs` keeps per-process tables for a user32 whose
  procs differ.
- Two arm64 kernel bits that x86 hardware maintains. The Access Flag: a page the working-set
  trimmer aged faulted forever (`trap.rs ki_set_access_flag`). The dirty bit: a written page
  made read-only paged out as clean and came back from the file, i.e. an unbound IAT
  (`PTE_SOFTWARE_DIRTY`). See the field manual's arm64 fault-loop entries. Neither is FEX-specific;
  x86-64 guests were simply the first workload big enough to make the trimmer run.
- Modules loaded through the registry, which no import walk sees: winmm's `Drivers32` is derived
  from `hivesft.inf`, and the rest are listed in the payload (each one found from wowx64's refusal
  log, which is now never capped). Also the amd64 side-by-side assemblies (comctl32 v5/v6,
  gdiplus), whose absence failed every activation context.

**Then (2026-10-08, later).** The fifteen that failed, run on their own (`APPSTORM_ONLY`):

- rapps and iexplore: both reach https, and secur32's schannel opens its crypto provider by the
  `Image Path` rsaenh writes into the registry at self-registration (`rsaenh.rgs`). No INF names
  it, so `wow64-closure.py` cannot derive it; it is a listed root now. Both then PASS (rapps
  completes its database download in ~110 s).
- appstorm's sampler can read an x86-64 guest thread now. A guest `NtGetContextThread` on another
  thread (`thread.rs RemoteGuestContext`) answers for a thread parked in a syscall, from the guest
  rsp and service each thread publishes in its wowx64 record for the duration of the syscall:
  rip is the stub's return address at [rsp]. A thread running guest code is refused, not guessed.
  (`stackdump.h` also had to learn to read a stack shallower than its 8 KB buffer.)
- control.exe and the applets were never hung: every one the sampler caught was busy, mostly in
  icon loading, class registration and, for sysdm, a logo built with two GetPixel syscalls per
  pixel. They open in 40-80 s on a quiet host, so appstorm, when it is itself the emulated
  process (its machine differs from the shell's), multiplies its waits by 6. Criteria unchanged.

In isolation that took the fifteen from 4 passes to 9-10 per run, the misses all timing. A full
sweep on a host saturated by integration builds (`20261008-114135-_w0p3rum-smoke-arm64`, mspaint
164 s vs 23 s) measured mostly the host. **Owed:** a full sweep on a quiet host (per-syscall cost
under FEX, also owed here, was measured and cut on 2026-10-11, below); memory (a 1 GB VM leaves ~30 MB before the sweep,
so the payload asks for 2 GB; ~1 MB per x86-64 app is kept); the guest user32 reading native procs
from `gpsi` (pfnclient.rs); and remote `NtSetContextThread` on a guest thread.

Memory exhaustion (2026-10-10). The 0x7A was `MiMakeSystemAddressValid` bugchecking on the
`STATUS_NO_MEMORY` its page-table fault returned (P1=1, P2=C0000017, ~8 MB free;
`20261008-003644-ghnmlian`, `-005837-s33duoyg`). It now waits for the balancer and retries, and so
does `MiMakeSystemAddressValidPfn`. User faults still fail on `STATUS_NO_MEMORY` instead of
retrying as the C does: retrying every fault turned exhaustion into a whole-machine stall, because
nothing frees pages until a process exits. Failing the fault kills only the starving app. Measured
at 768 MB (`20261010-075717-s6lsafno`): no bugcheck; rapps and iexplore died `c0000005` at ~18 MB
free; the sweep ran on past them. Owed:
- **explorer wedged afterwards** on a win32k-space gate whose holder never signalled (also in
  `20261010-054207-p5m0hi2a`). Something died or failed holding it. The 2 s user-lock hold trace
  cannot see it, because it reports at RELEASE.
- **Commit accounting: done 2026-10-11, and it was not the fix for this drift.** The bullet that
  stood here said `MmTotalCommittedPages` was never updated; it was, but as pages IN USE (balance.c's
  consumers), and nothing checked the limit. It is now the commit charge (`ARM3/commit.rs`), checked
  at every private, reserve-and-commit and pagefile-section commit, returned on decommit, release,
  segment delete and process teardown, and reported to `GlobalMemoryStatus` and taskmgr. On a 2 GB
  sweep the charge sat at 154 MB after every one of 50 apps while free memory fell 1035 -> 966 MB:
  the drift is in UNCHARGED memory (image pages, cache, pool), still owed.
  Strict accounting then broke the FEX guests. A 1 GB FEX boot has ~161 MB of commit limit (the
  ~714 MB ramdisk is resident), and each FEX process committed ~160 MB it never touched: 16 MB code
  buffers, 4 MB call-ret stacks recommitted at every zeroing, and rpmalloc committing whole 4 MB and
  64 MB pages per malloc. Refused, FEX dereferenced the null and died (`file32`, `appstorm`). Fixed
  in vendored FEX (`vendor-patches/fex.md`, "commit-on-touch"): FEXCore's own allocations and the
  call-ret stack are reserved and committed on first touch through the frontends' OvercommitTracker,
  and rpmalloc commits the OS pages under each block as it hands it out. Peak charge on that boot
  ~160 MB -> under 32 MB. The arm64 build now runs ninja in the out-of-tree FEX builds, so a vendored
  FEX edit reaches the image. Not charged yet: copy-on-write pages of image and file views,
  PEB/TEB pages and kernel pool. The limit (available pages at init minus 1024, plus any
  pagefile) leaves headroom for them. kmtest `ZwAllocateVirtualMemory` now accepts
  `STATUS_COMMITMENT_LIMIT` and checks only its first 50 commit chunks, so its conformance rows
  were re-seeded lower on purpose (the baseline files carry the note).
- Census `STUCK` lines now carry a kernel backtrace (`[CENSUS]   bt`). On arm64 the frame-pointer
  walk past the first two frames is unreliable through Rust code. `ETHREAD.StartAddress` reads back
  garbage there (a wait-path return address, `0x7ff00000000`); that layout is owed a check.

Per-syscall cost (2026-10-11). Measured with the `sysbench` payload (a ladder of operations timed
natively, then by its own amd64 build under FEX, minutes apart on the same VM) and a profile
wowx64 now prints at every x86-64 process exit (`[WOWX64] pid N syscall profile`: kernel,
wowx64, and "before" -- guest code plus FEX's exit from and re-entry into its JIT -- in total
and, past 1000 calls, for the five costliest services). A guest syscall cost 10x native, and
the profile put it on our side: NtGetCurrentProcessorNumber was kernel 5.7 us, **wowx64
45 us**, FEX 1.3 us. wowx64's `Dispatch` zeroed its 1 KB path-redirect buffer on every call, and
ntdll's memset was an unoptimised volatile byte loop (the whole cnt CRT builds with
`-C no-prepopulate-passes`; field manual, "Everything is several times slower..."). memset /
memcpy / memmove are now asm (`cnt-rust/src/cabi/memops.rs`) and the buffer is written only by
the path services. Native / guest per operation (after: `20261011-033055-rtqc8wpb` with the asm
CRT, `20261011-034606-yhfa1enn` with the buffer fix too; a loaded host, so ranges):

| op | before (4 runs) | after (2 runs) |
|---|---|---|
| NtGetCurrentProcessorNumber | 4.2-7.7 / 48-79 us | 3.9-5.3 / 4.3-7.8 us |
| NtQueryInformationProcess | 14-41 / 54-143 us | 16-20 / 10.5-19.6 us |
| NtAllocate + NtFreeVirtualMemory | 29-61 / 183-296 us | 21-38 / 28-31 us |
| GetPixel (memory DC) | 68-109 / 99-209 us | 8.3-16.4 / 11.0-21.8 us |
| CreateWindowEx + DestroyWindow | 1.2-3.8 / 4.7-9.2 ms | 0.57-0.85 / 1.24-1.53 ms |

A plain guest syscall now costs what a native one does, within the noise: FEX's transition is
~1.7 us and wowx64's share under 2 us. What is left is win32k callbacks (about 2x native) and the
CPU layer's Notify brackets on memory calls. Native got faster too: win32k was bound on
byte-at-a-time copies. An empty `PeekMessage` takes ~50 ms on both sides; that is upstream's
`ZwYieldExecution` without `PM_NOYIELD`, giving a quantum to any ready thread, not a defect.
Owed: the CRT compare/search loops (the NUL scans are asm since 2026-10-17); the applet
sweep on a quiet host, re-timed against these numbers.

**Then (2026-10-08, evening): taskmgr.** Two bugs in taskmgr itself, both upstream, both visible
only when the guest is slow:

- A heap corruption (crashed 10 of 20 x86-64 launches): perfpage's `WM_DESTROY` disposed the
  graph controls before stopping the thread that writes them, which stored each CPU-% sample at an
  advancing index into freed memory. Stop the thread first, as procpage and applpage already did.
  0 crashes in 18 launches after.
- The close "deadlock" was neither a deadlock nor a lock holder (win32k's user-lock trace saw no
  hold of 2 s or more). The main window posts `WM_TIMER` to each refresh thread every 500 ms
  whether or not the last refresh has finished. Once x86 DOOM loads the VM, a pass outlasts the
  interval and a backlog queues ahead of `WM_QUIT`. Each pass makes dozens of cross-thread
  ListView `SendMessage`s, which `EndLocalThread` serviced only at its 500 ms polls, because
  `WaitForSingleObject` dispatches no sent message. So every launch after DOOM started timed out,
  even with 300 s allowed. Fix: `EndLocalThread` waits in `MsgWaitForMultipleObjects(QS_ALLINPUT)`,
  and the refresh threads collapse pending `WM_TIMER`s into one pass (`GetRefreshMessage`).
  12 of 12 then pass, 6 of them with DOOM running (`20261008-203258-r8s7qojj-smoke-arm64`); those
  6 still take 86-111 s against ~20 s before DOOM (load, presumably the per-syscall cost owed above;
  not measured).

New and owed: one run (`20261008-194633-hxs3riy7-smoke-arm64`; 0 of the 3 runs before it) killed
appstorm.exe 30 s in with a native undefined-instruction fault (ESR 0x02000000) at
0x7FFFADFD80C4. The address looks like FEX's code cache, but that is not proven. The guest saw it
as `c000001d` on the `ret` after `NtReadFile`'s `syscall`.

Then (2026-10-09). The code-cache attribution is now likely, though still inferred: the same
range takes FEX's LDAR/STLR alignment fixups on guest addresses, which only JIT output does.
LR is PC-0xF8, so the thread returned from the syscall call-out and ran 62 instructions of the
block before the bad word. A `ret` block runs thousands of times, so the code most likely changed
while the thread was blocked. `NtReadFile` is bracketed by `BTCpu64NotifyReadFile`, which
invalidates code and zeroes every thread's call-ret stack (decommit + recommit). Our port matches
upstream. Three kernel suspects were ruled out by reading the code:
- decommit already flushes the TLB process-wide under the PFN lock;
- recommit turns `MmDecommittedPte` back into demand-zero, so the wipe really zeroes;
- user pages never enter the working-set list, so nothing trims them.

Eight more sweeps (`20261009-162243-pc9vk8r4`, `20261009-191037-vsrxcvcf`) did not reproduce it.
An EL0 EC-0 fault now prints a `[UDF]` line: PC, PTE, and the instruction words around PC. The
word tells the causes apart: 0 means lost contents, a valid opcode means a rewrite was seen stale,
anything else is a JIT encoding bug. Owed: the next hit.

Found on the way, then fixed: `MiDecommitPages` overwrote a transition PTE with `MmDecommittedPte`
without detaching the PFN, leaking the standby/modified frame (the C asserted the case away, the Rust
dropped the asserts). It now detaches it as `MiDeletePte` does, and any other unexpected invalid PTE
trips an `mm_guard`. Unreached while user pages are never trimmed.

`osk` sometimes opened and never closed (`close=TIMEOUT exit=0x103`, ~1 in 20 x64 launches;
`20261008-114135-_w0p3rum`, `20261009-191037-vsrxcvcf` payload4). The parked-thread sample showed
`ExitProcess` -> `LdrShutdownProcess` -> msacm32's `DLL_PROCESS_DETACH` -> `RtlFreeHeap`, parked
on the return from `ZwFreeVirtualMemory`, i.e. inside our `NtFreeVirtualMemory` bracket. The
cause: `ExitProcess` first calls `NtTerminateProcess(NULL)`, which kills every other thread
wherever it is, including between a `BTCpu64Notify*` Before (FEX's `ThreadCreationMutex` taken)
and its After. The exiting thread then deadlocks on a lock held by a dead thread. osk's
warning-dialog thread is the usual victim. Upstream's `BTCpuProcessTerm` is empty, so upstream
has the same hole. Fix (contract v4): `BTCpu64ProcessTerm` brackets that call, using FEX's own
fork discipline:
- Before: take `ThreadCreationMutex` and the code-invalidation lock if possible within ~2 s. A read
  bracket can hold both across a blocking `NtReadFile`, so the wait is bounded.
- After: wait for every victim to die, then rebuild both locks in place rather than unlocking. A
  victim killed while queued is still counted as a waiter, and an unlock would hand off to it.

Measured: osk x25 x4 boots (`20261009-214309-hxbaw3uw`), 100/100 closed, and the bracket fired on
every exit (52 per boot). The CPU layer is built out of tree, so a `Module.cpp` edit used to ship
the OLD plugin from a green build; arm64 builds now rebuild it when its sources are newer, or fail
(`wowx64-cpu`, `CPU_LAYER_BUILD_FAIL`).

The same runs found four applets that had never run as x64 guests: openglcfg, wined3dcfg, console
and hotplug, added to appstorm on 2026-10-08. They were missing from the guest set; wowx64
refused exactly those four (`NOT FOUND`). `lpk.dll` was missing too, and gdi32's `LoadLPK` retries
it on every text call: 18929 refusals in one sweep, each one a blocking serial line. With both
added, the four pass, lpk refusals drop to 0, and a sweep takes ~18 min against 33. Most of that
is the four applets no longer timing out; how much lpk saved alone is not measured.
