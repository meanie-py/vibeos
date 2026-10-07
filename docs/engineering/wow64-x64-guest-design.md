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

**Owed:**

1. **Into an x86-64 guest FROM a non-x86-64 process** (the reverse direction): a native or 32-bit
   creator's CreateRemoteThread into a running x86-64 guest starts the native kernel32's
   `BaseThreadStartup`, which that process no longer has, with an x86-64 routine. It wants the
   same divert as WoW64's: the guest process's native loader handing a start that is no native
   image's code to wowx64.
2. **A NEW native arm64 child** of an x86-64 guest: unreachable — the guest's system32 is
   redirected to sysamd64, where an arm64 exe reads back as not-found (WoW64's own-machine
   view) — and refused with `STATUS_NOT_SUPPORTED` if it ever is reached.