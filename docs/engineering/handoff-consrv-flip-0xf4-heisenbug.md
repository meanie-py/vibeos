> ## ✅ RESOLVED 2026-05-29 (codex)
> Root cause was NOT a consrv handler — it was a csrsrv **thread-hash off-by-one**:
> `CsrInitializeProcessStructure` (`smss-csrss-rust/src/csrsrv/procsup.rs`) inited
> `0..256` but `NUMBER_THREAD_HASH_BUCKETS=257`; bucket 256 stayed `{null,null}`,
> TID `0x100` hashed there, `InsertHeadList` AV'd, csrss self-terminated → 0xF4.
> Fix: `0..NUMBER_THREAD_HASH_BUCKETS`. The non-returning LPC was **basesrv
> `BaseSrvCreateThread`** (ServerId=1), NOT consrv `SrvGetConsoleInput` (my
> misattribution below — corrected). Also fixed: `ConSrvAllocateConsole`'s
> `NtDuplicateObject` source-process handle `null → -1` (current proc), console.rs.
> Verified i386: 5/5 raw boots clean, smoke 10 passed/1 skip, desktop renders.
> Doc kept for the diagnostic record (Heisenbug-hunting notes still useful).

# Handoff: consrv top-level cohab flip — residual intermittent BUGCHECK 0xF4

**Audience:** Codex (or any agent picking up this user-mode console debug).
**Branch:** `worktree-smss-csrss` (worktree at
`/home/curi/Projects/os-playground/vibeos-wt-smss-csrss`).
**Status:** consrv top-level C→Rust cohab flip is **INTEGRATED + FAILING
(intermittently)**. Links on all 3 arches, reaches `[WIN32K] shell ready`,
but raw boot 0xF4s ~1-in-3 during explorer/svchost bring-up. **Uncommitted.**

> ⚠️ Worktree discipline: edit ONLY under
> `/home/curi/Projects/os-playground/vibeos-wt-smss-csrss/`. Never touch the
> main repo `/home/curi/Projects/os-playground/vibeos/`. Real builds:
> foreground, sandbox disabled. Never revert a `.rs` to C; never delete `.rs`;
> never comment out a `pub mod` to dodge a regression — fix in place.

---

## TL;DR

The console-server (`consrv`) top-level `.c` files were flipped from C to their
Rust translations (cohabitation: commented out of `CONSRV_SOURCE` in
`win32ss/user/winsrv/consrv.cmake`; `libwinsrv_rust.a` provides the symbols via
`--allow-multiple-definition`). `condrv/` and `frontends/` stay C.

A first-order bug was found and fixed: the C `CON_API` macro
(`win32ss/user/winsrv/consrv/api.h`) acquires the per-console lock
(`ConSrvGetConsole(&Console, TRUE)`) at entry and releases it
(`ConSrvReleaseConsole`) at **every** return; rustconv inlined the body but
**dropped the release epilogue**. `coninput.rs` had 5 handlers that acquired and
never released. Fixed → raw boot went from **0/3 to 2/3 clean** (reaches
`[EXPLORER] _tWinMain entered`).

**A residual intermittent 0xF4 (CRITICAL_OBJECT_TERMINATION) remains** (~1/3),
at the same point (console bring-up while a heavy COM/crypto/net process —
PID ~208, the shell/explorer side — is loading). Your job: find and fix it so
raw boot is reliably clean, then pass the i386 smoke harness.

---

## Reproduce

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-smss-csrss/build-i386
ninja winsrv livecd                 # winsrv.dll + roll the ISO (livecd doesn't auto-roll!)
cd ../qemu
rm -f i386_raw.log
timeout 130 qemu-system-i386 -M pc -accel kvm -cpu host -m 512M \
  -cdrom ../build-i386/livecd.iso -serial file:i386_raw.log \
  -vga std -display none -no-reboot >/dev/null 2>&1
grep -a BUGCHECK i386_raw.log | tail -1
grep -ac "_tWinMain entered" i386_raw.log   # 1 = clean boot, 0 = crashed before it
```

Run it **3-5 times** — it's intermittent (~1/3 crash). The crash signature is
stable:

```
*** BUGCHECK: 0x000000F4 (0x00000003, 0xffbXXXXX, 0xffbXXXXX, 0x805adbec)
```

(P1=3 = a PROCESS terminated; P2/P3 = EPROCESS ptrs, vary per boot; P4=0x805adbec
is constant across all runs — a fixed kernel reason/callsite address.)

The smoke harness reproduces it too (and its debugcon triage is MISLEADING here —
it decodes "crashed before KiSystemStartup / 0 milestones", which is bogus for a
user-mode crash; trust the serial log, not the debugcon decode):

```bash
.venv  is at ../../vibeos/.venv  (the worktree has no .venv):
/home/curi/Projects/os-playground/vibeos/.venv/bin/python -m pytest \
  tests/qemu/test_smoke.py -v --arch i386 --timeout 600 --skip-clean-build
```

---

## ⚠️ This is a Heisenbug — read before instrumenting

ANY observation that perturbs timing MASKS the crash:

- **DbgPrint instrumentation in the csrsrv dispatch hot-path made it vanish** (the
  boot then reached `_tWinMain` every time). Adding prints is actively
  counterproductive for catching it.
- **`DbgPrint` is unusable INSIDE winsrv.dll** — winsrv links `libcntpr`, whose
  local `DbgPrint` is a no-op / AVs in user-mode csrss. Only **csrsrv**
  (`smss-csrss-rust`) DbgPrint resolves ntdll's and actually prints (`[CSRSRV]`
  lines in the serial). So you cannot trace winsrv handlers via DbgPrint.
- **gdb breakpoint on KeBugCheckEx did not reproduce it** under TCG either (the
  bp/connection overhead perturbs timing). Note for the toolkit: on i386 the
  symbol is decorated `_KeBugCheckEx@20` and gdb's DWARF table doesn't expose it
  by name — `qemu/run-until.sh --break KeBugCheckEx` fails with "not defined".
  Break by ADDRESS instead: kbase has been stable at `0x80330000`, ntoskrnl
  ImageBase `0x400000`, `_KeBugCheckEx@20` PE-VA `0x4766e0` →
  `--break "*0x803a66e0"` (recompute if kbase differs in `[find-kbase]` output).

It reproduces reliably ONLY on un-instrumented fast boot — **both KVM and TCG**
(so it's a genuine race, not KVM-scheduling-specific). Prefer reasoning from the
code + lock-balance audits over live instrumentation.

---

## What's already fixed (don't redo)

Console-lock-leak fix (the CON_API release epilogue), 0/3 → 2/3:
- `coninput.rs`: 5 handlers (`SrvReadConsole`, `SrvGetConsoleInput`,
  `SrvWriteConsoleInput`, `SrvFlushConsoleInputBuffer`,
  `SrvGetConsoleNumberOfInputEvents`) refactored to the **Impl-split** pattern
  (entry: acquire → fail-return(no release) → `Srv*Impl(...)` → single
  `ConSrvReleaseConsole` → return). This mirrors the already-correct
  `conoutput.rs` handlers (e.g. `SrvGetConsoleScreenBufferInfo`).
- `history.rs`: 5 handlers, inline `ConSrvReleaseConsole` before each post-acquire
  return.

Current per-file console-lock balance (acquire / release):
`coninput 5/5 · conoutput 18/18 · frontendctl 18/18 · console 10/13 ·
history 7/11 · handle 1/2`. Global 67 acquire / 73 release — **no net leak**, but
count-balance ≠ per-path-balance.

---

## Leads for the residual (ranked)

1. **A per-path console-lock leak in a count-balanced handler.** conoutput &
   frontendctl use Impl-split (likely fine). Audit `console.rs` (10/13) and any
   handler with multiple early returns: does EVERY post-acquire return release
   exactly once, and does the acquire-FAIL path NOT release? A leak on a rare
   early-return path would be intermittent.

2. **An over-release / double-release** (73 > 67). If a handler releases on a path
   where it didn't acquire (or releases twice), it corrupts the per-console lock
   state → intermittent crash. Audit the same handlers for the inverse error.

3. **The poll loop.** `SrvGetConsoleInput` (console API #1) is dispatched in a
   tight loop returning `STATUS_SUCCESS` (you'll see many
   `[CSRSRV] LPC_REQ ... ApiId=1 ... dispatch returned st=0x0`). Whatever the
   client (PID ~208) is polling, the crash happens around there. Check the
   `STATUS_PENDING`/wait-block path (`ReadInputBuffer` → `WaitBeforeReading` →
   `ReadInputBufferThread`, which is reached via a
   `core::mem::transmute(fn as *const ())` cast — verify that callback's ABI).

4. **Dual-definition cohab hazard.** condrv stays C, but the Rust `condrv/*.rs`
   functions (e.g. `ConDrvGetConsoleInput` in `condrv/coninput.rs`) are ALSO
   `#[no_mangle]` and sit in `libwinsrv_rust.a`. With `--allow-multiple-definition`
   and link order (consrv C archive before the Rust `.a`), the C condrv SHOULD
   win, but verify the live winsrv.dll resolves condrv symbols to the C copy, not
   the dormant/untested Rust condrv. Use the toolchain's nm
   (`.build-tools/llvm-mingw/bin/*-nm` or `llvm-nm` if present) on
   `build-i386/win32ss/user/winsrv/winsrv.dll`.

5. **Which process dies.** Get it deterministically: break by address at
   `_KeBugCheckEx@20` (see Heisenbug section) and run
   `qemu/inspect.sh $RUN --command triage-bugcheck` / `identify-object` on the
   0xF4 P2/P3 EPROCESS. If it's csrss → a console-server thread faulted; if it's
   the client (explorer/svchost) → a console API returned bad data the client
   choked on. (Catching it live is hard because the bp masks the race — you may
   need to let it free-run under TCG and catch the bugcheck when it does fire.)

---

## Diagnostic facts gathered

- Boot reaches: SERVICES (DcomLaunch), WINLOGON GinaInit, svchost.exe,
  `[WINLOGON] Spawning explorer.exe`, `[WIN32K] shell ready` (×3). Crash is after
  shell ready, before `[EXPLORER] _tWinMain entered`.
- csrsrv dispatch loop is in `sdk/lib/rust/smss-csrss-rust/src/csrsrv/api.rs`
  (~line 448-486). It already DbgPrints `[CSRSRV] LPC_REQ` lines (ServerDll,
  ApiId, DispatchTable, dispatch_fn_slot/target, "dispatch returned st="). Console
  server = ServerDll index 2, ApiBase 0; ApiId is the index into
  `ConsoleServerApiDispatchTable` (`consrv/init.rs`): 0=SrvOpenConsole,
  1=SrvGetConsoleInput, 3=SrvReadConsoleOutput, etc.
- When un-instrumented, the last `[CSRSRV] LPC_REQ ... ApiId=1` line before the
  0xF4 has NO matching "dispatch returned" — but with the lock-leak fixed, API #1
  now returns st=0x0 repeatedly, so the residual crash is no longer a clean
  "handler never returned" — it's a delayed/contended termination.

---

## Build / test recipe

```bash
# winsrv-rust crate compiles for all 3 (cohab flip links clean on all 3):
cd build-i386  && ninja winsrv         # i386 (authoritative — KVM, reaches shell ready)
cd build-amd64 && ninja winsrv         # links; amd64 BOOT is blocked by a separate
                                       #   pre-existing 0x6B (IopCreateArcNames) — ignore
cd build-arm64 && ninja winsrv         # links; boot-verify via ramdisk (see AGENTS.md)

# after any winsrv.dll change you MUST roll the ISO:
cd build-i386 && ninja livecd

# host cargo check (fast, no QEMU):
cd sdk/lib/rust && cargo check -p winsrv-rust \
  --no-default-features --features winsrv_dll_only,ntddi_ws03 \
  --target i686-pc-windows-gnu
```

Goal: raw boot reliably clean (5/5 reach `_tWinMain`), then i386 smoke green
(`10 passed, 1 skipped`), then arm64 ramdisk boot reaches `shell ready`.

---

## Uncommitted changes that ARE this flip (HEAD = `81ae68dd518`)

```
 M sdk/lib/rust/winsrv-rust/src/consrv/condrv/conoutput.rs   # ABI: buffer fns extern "C"
 M sdk/lib/rust/winsrv-rust/src/consrv/condrv/graphics.rs    # ABI
 M sdk/lib/rust/winsrv-rust/src/consrv/condrv/text.rs        # ABI
 M sdk/lib/rust/winsrv-rust/src/consrv/coninput.rs           # lock-leak fix (Impl-split)
 M sdk/lib/rust/winsrv-rust/src/consrv/console.rs            # ConSrv* ABI + ConSrvInitConsole translation + CodePageToCharSet
 M sdk/lib/rust/winsrv-rust/src/consrv/frontends/gui/conwnd.rs
 M sdk/lib/rust/winsrv-rust/src/consrv/frontends/terminal.rs
 M sdk/lib/rust/winsrv-rust/src/consrv/history.rs            # lock-leak fix (inline)
 M sdk/lib/rust/winsrv-rust/src/consrv/mod.rs                # CRT helpers extern "C" split
 M win32ss/user/winsrv/consrv.cmake                          # the flip (13 .c commented) + shim
?? win32ss/user/winsrv/consrv/consrv_rust_shim.c             # IS_FULL_WIDTH/IsListEmpty/_Interlocked*/COBJMACROS
```

ABI reconciliation already done (the #1 i386 crash cause): 6 `ConSrv*` are cdecl
(`extern "C"`), `ConSrvValidateConsole`/`ConSrvInitConsole` are NTAPI
(`extern "system"`); condrv buffer fns + `CodePageToCharSet` are cdecl. See the
commit-message history and `feedback_con_api_lock_release_dropped` /
`feedback_extern_c_abi` memories.

To revert the flip cleanly (keep all Rust, restore green tree) if you need to:
`git stash push -- win32ss/user/winsrv/consrv.cmake` (re-enables C consrv; Rust
stays dormant + compiling). `git stash pop` to re-apply.
