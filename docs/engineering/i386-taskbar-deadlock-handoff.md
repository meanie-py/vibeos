# i386 taskbar deadlock — handoff

**Status:** root cause localized to a kernel-wait deadlock inside explorer's
tray-window creation; exact lock/fault NOT yet pinned. The font/watermark
regression that shared this task is **fixed** (separate commit). This doc hands
off the *taskbar* deadlock.

**Worktree:** `/home/curi/Projects/os-playground/vibeos-wt-win32k`
**Branch:** `worktree-win32k-translation`
**Relevant commits (top of branch):**
- `024e359e8d4` winlogon-rust: synchronous font load (the fixed watermark regression)
- `0429194c5bd` DBG-TB instrumentation (all the probes below; **revert before merge**)

The probes are committed and **left in place on purpose** so this is reproducible.
The one *behavior* change I tried (bypassing `NtQueryDefaultLocale`) is already
reverted — only inert trace probes remain.

---

## TL;DR of the bug

On **i386 only** (amd64 gets a partial taskbar, arm64 a full one), explorer
hangs while building the taskbar. The hang is a **genuine kernel wait**
(deadlock), not a crash and not a spin.

Chain (all confirmed by serial markers, in order):
```
explorer StartWithDesktop
  -> CreateTrayWindow                       [traywnd.cpp:3726]
     -> CTrayWindow::Open -> Create(...)    [traywnd.cpp:2150]  (ATL CreateWindowEx, WS_POPUP tray)
        -> win32k NtUserCreateWindowEx
           -> KeUserModeCallback -> user-mode WM_CREATE -> CTrayWindow::OnCreate [traywnd.cpp:2305]
              -> m_StartButton.Create(m_hWnd)             [traywnd.cpp:2314]
                 -> CStartButton::Create                  [traywnd.cpp:260]
                    -> LoadStringW(hInst, IDS_START=595)  [user32 resources.c:21]
                       -> FindResourceW                   [kernel32 wine/res.c:176]
                          -> find_resourceW               [kernel32 wine/res.c:109]
                             -> LdrFindResource_U          [rtl-misc/src/res.rs:238]
                                -> find_entry              [ntdll-rust/src/rtl/libsupp.rs:645]  <-- HANGS HERE
```

`OnCreate` is running as a **win32k user-mode callback** (KeUserModeCallback,
invoked from inside `NtUserCreateWindowEx`), so **win32k holds its global lock
across the callback**. That context is the whole story (see "ruled out").

---

## Evidence — what is and isn't true

### Confirmed
1. **It's a kernel wait, not a spin.** With the VM stuck, QMP `info registers`
   sampled 12/12 times shows the CPU in the idle loop (`EIP=0x80203c75`, `HLT=1`,
   `CPL=0`). A user-mode spin would show `CPL=3` with a userland EIP. So the
   blocked thread is parked in `KeWaitForSingleObject`-style wait.
2. **The exact same call works *outside* the callback.** I added a pre-warm:
   `LoadStringW(hInstance, IDS_START, …)` in `StartWithDesktop` *before*
   `CreateTrayWindow` (i.e. outside any win32k callback). It **completes fine**
   — all `[FE]` markers print and `find_entry` returns. The *identical* call
   *inside* `OnCreate` then still hangs. => the bug is the **callback context**,
   not the call itself.
3. **explorer uses BASE kernel32.dll + ntdll.dll**, not the `_vista` variants
   (confirmed: only base `kernel32.dll` carries the `[FRW]` markers and they
   fired). So the active `find_entry` is `ntdll-rust/src/rtl/libsupp.rs` (the C
   `dll/ntdll/rtl/libsupp.c` is commented out of `rtl_um`, which is now a rust
   stub — its `[FE]` markers are DEAD, never compiled).

### Ruled out
- **Resource-data page faults.** Pre-warming makes explorer's `.rsrc` resident;
  the in-callback load still hangs. Not the resource pages.
- **`NtQueryDefaultLocale`** (the syscall in `find_entry`'s lang-neutral path).
  Bypassing both calls (`if false && …`) did not help, *and* the hang happens
  before `find_entry`'s first marker even registers. (Bypass now reverted.)

### The remaining contradiction (this is the crux for you)
Inside the callback, **not one `[FE]` marker prints** — so `find_entry`'s very
first step doesn't register — yet it's a kernel wait with resource pages
resident and no reached syscall. The only ways to kernel-wait that early:
- a **page fault on a page pre-warm didn't cover** — prime suspect is the
  **callback's own user stack** (KeUserModeCallback runs on a *distinct* user
  stack from the normal one the pre-warm call used; a stack-growth fault there
  may not be able to complete with the win32k lock held), or a code page; **or**
- an **AB-BA lock deadlock**: explorer's main thread holds the win32k lock
  (via the callback) and blocks acquiring the loader lock (or heap/PEB lock) in
  the early `LdrFindResource_U`/`find_entry` path, while another explorer thread
  holds the loader lock and is blocked waiting on the win32k lock. (Explorer
  spawns worker threads in `FileIconInit`/`OleInitialize` before this.)

Note: `[FE]` DbgPrints DO print from the *pre-warm* (outside-callback) call, so
ntdll `DbgPrint` works in general — but may itself be suppressed/deadlock-prone
*inside* the callback, which is why you can't rely on adding more DbgPrints in
`find_entry` to see the in-callback path. Use a memory sentinel or the kernel
stack instead (below).

---

## How to reproduce (exact)

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-win32k
./vibeos-cli build --arch i386          # ~5-7 min; probes already in tree
RUN=$(bash qemu/start.sh i386 --gdb | tail -1)   # boots KVM; SLOW (~3 min to kernel)
# wait for the stall:
until grep -qa "StartBtn::Create enter, LoadStringW" qemu/runs/$RUN/serial.log; do sleep 15; done
# confirm it's settled (no further [FE] for the in-callback call):
grep -an '\[FRW\]\|\[FE\]\|StartBtn::Create enter' qemu/runs/$RUN/serial.log | tail
```

The serial log shows TWO `find_resourceW` calls: the **pre-warm** one (completes,
full `[FE]` sequence + "done OK" + `[FRW] LdrFindResource_U returned`), then the
**in-callback** one that stops dead at `[FRW] names resolved, calling
LdrFindResource_U` with no `[FE]` after it.

**i386 boots are SLOW under KVM** — the serial log sits at `[freeldr] Switching
stack...` (10 lines) for ~3 min, then jumps to thousands of lines. That's normal,
not a hang. Sample line count twice to tell advancing from hung.

Confirm kernel-wait vs spin with QMP (`qemu/runs/$RUN/qmp.sock`, `human-monitor-command`
"info registers", look for `CPL`/`HLT`). 12 samples idle = wait.

Stop a VM: `bash qemu/stop.sh <run-id>` (needs the `bash` prefix — script isn't +x).
List VMs: `python3 qemu/ps --json`.

---

## The probe markers (what each means / where)

| Marker prefix | File | Purpose |
|---|---|---|
| `[EXPLORER] DBG-TB: SWD …`      | `base/shell/explorer/explorer.cpp` | StartWithDesktop step-by-step + the `pre-warm IDS_START` experiment |
| `[EXPLORER] DBG-TB: CTW/Open …` | `base/shell/explorer/traywnd.cpp`  | CreateTrayWindow / Open / Create split |
| `[EXPLORER] DBG-TB: OnCreate …` | `base/shell/explorer/traywnd.cpp`  | per-child-control steps; hangs right after `StartButton.Create` |
| `[EXPLORER] DBG-TB: StartBtn… / Init…` | `base/shell/explorer/traywnd.cpp` | CStartButton::Create + Initialize split |
| `[LSW] …`  | `win32ss/user/user32/misc/resources.c` | LoadStringW: FindResource/Load/Lock split |
| `[FRW] …`  | `dll/win32/kernel32/wine/res.c`        | find_resourceW: get_res_nameW + LdrFindResource_U call |
| `[FE] … / FE_PROGRESS` | `sdk/lib/rust/ntdll-rust/src/rtl/libsupp.rs` | find_entry steps; **DbgPrints unreliable inside the callback** — `FE_PROGRESS` is a `static mut` sentinel set 1..9 |
| `[FE] …` (DEAD) | `dll/ntdll/rtl/libsupp.c` | NOT compiled (rtl_um is a rust stub). Ignore. |
| `DBG-IPD` | `sdk/lib/rust/win32k-rust/src/{desktop,freetype,font}.rs` | font-render probes from the (now-fixed) watermark bug; orthogonal |

`IDS_START = 595` (`base/shell/explorer/resource.h:105`). In `find_entry` the
gated probes fire when `info.Type == 6 (RT_STRING)` and `info.Name == 38`
(`(595>>4)+1`).

---

## The blocker to finishing, and how to beat it

You need the **blocked thread's kernel stack** — that names the exact wait
(which lock / which fault). Two obstacles, with fixes:

1. **`qemu/inspect.sh <run> --command list-threads` returns 0 threads.** It
   resolves `PsActiveProcessHead` via DWARF, but this kernel has a **+0x291000
   Rust symbol shift** (see memory/field-manual §9): DWARF addresses for the
   ntoskrnl Rust image are offset from runtime. The walk reads a bogus head and
   yields nothing. Fix `qemu/gdb-helpers/list-threads.py` (and `find-kbase`) to
   add the shift, or walk `PsActiveProcessHead` manually in gdb with the shift
   applied, match `_EPROCESS.ImageFileName == "explorer"`, then walk
   `ThreadListHead -> _ETHREAD`, read `KTHREAD.State/WaitReason`, and dump the
   kernel stack (`dump-thread`) to see the wait frame.

2. **`FE_PROGRESS` sentinel can't be read** — the `#[no_mangle] static mut` is
   stripped from the PE symbol table, so I couldn't find its address. Options:
   give it a value you can scan for, or read it via the cargo build's symbol
   info, or (simplest) skip it and go straight for the kernel stack.

Boot **with `--gdb`** (already in the repro) so the gdb stub on
`tcp:127.0.0.1:120NN` is available; the port is printed in
`qemu/runs/$RUN/qemu.launch.log`.

---

## Suggested attack plan

1. Get the kernel stack of explorer's blocked thread (fix the +0x291000 shift in
   `list-threads.py` first). The wait frame answers everything:
   - if it's `MmAccessFault` / `MiWaitForInPageComplete` -> it's the stack-fault
     hypothesis; look at how KeUserModeCallback sets up the callback stack on
     i386 and whether faulting it deadlocks with the win32k lock.
   - if it's `KeWaitForSingleObject` from `RtlpWaitOnCriticalSection` /
     `ExpWaitForResource` -> it's a lock; identify which (loader lock = PEB
     `LoaderLock`, or the win32k `gpresUser`/USER lock) and find the other
     thread holding it (walk all explorer threads, find the AB-BA pair).
2. Cross-check against amd64 (which gets further): why does the same path not
   deadlock there? The difference will point at the i386-specific bug
   (callback-stack setup, lock acquisition order, or an IRQL/APC-region detail).
3. The real fix is almost certainly in win32k callback handling or the
   `KeUserModeCallback`/stack path, NOT in explorer or the resource walker —
   explorer's code is stock ReactOS and works on real ReactOS.

---

## Cleanup owed (before this merges)

- Revert commit `0429194c5bd` (all the DBG-TB/[LSW]/[FRW]/[FE]/FE_PROGRESS/DBG-IPD
  probes). The user wants to keep the **tray-window milestone** markers
  (`OnCreate entry`, `child controls created`, `entering TrayMessageLoop`) as
  permanent boot markers for an expanded smoke test — re-add those cleanly
  (without the `DBG-TB` debug naming) rather than keeping the whole probe set.
- Keep `024e359e8d4` (the font fix).
