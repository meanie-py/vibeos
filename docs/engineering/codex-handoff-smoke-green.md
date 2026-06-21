# Handoff: get all 3 smoke tests passing (i386 / amd64 / arm64)

**Audience:** Codex CLI (or any agent) continuing this work.
**Worktree:** `/home/curi/Projects/os-playground/vibeos-wt-win32k`
**Branch:** `worktree-win32k-translation` @ `471402209c1` (the commit that first activated the Rust `rtl-heap`).
**Author of handoff:** Claude (prior session), 2026-06-09.

---

## 0. Mission

Get **`./vibeos-cli smoke --arch <i386|amd64|arm64>` to pass on all three architectures**, by
finding and fixing whatever bugs block each one. The smoke test boots the OS in QEMU and
requires reaching a graphical desktop (`shell ready` marker on x86; idle detection on arm64).

This is a translate-C-to-Rust project (ReactOS → Rust). **Read `AGENTS.md` at the repo root first** —
its rules are binding and override convenience. The most important ones for you:

- **Never revert a Rust translation back to C** to make a build/boot pass. Fix the Rust in place.
- **Never delete or re-generate a `.rs` translation** without explicit user approval (they cost real money).
- **Never weaken a smoke test or lower a boot marker.** Fix the OS, not the test.
- **Never hide a regression** by commenting `pub mod X;` out of the build.
- **Do not `git commit` without asking** the user first. **Do not `git checkout`/`restore` to discard** — use `git stash`.
- A broken-but-integrated Rust translation that fails smoke is the *correct* state to debug from.

---

## 1. Current state of the three arches

| Arch  | State |
|-------|-------|
| **i386** | **BLOCKED** on a win32k desktop/session-heap corruptor (BUGCHECK 0x1E). Boots all the way to `shell ready`, then crashes. Deeply diagnosed below — this is the main task. |
| **amd64** | Needs verification. Last known issue was completing the win32k flip (~20 undefined fastcall symbols). **Run a smoke first to get ground truth before assuming anything.** |
| **arm64** | Needs verification. **Caveat: `vibeos-cli build --arch arm64` does NOT rebuild user-mode cohab targets**, so a fast green arm64 smoke can be a *false positive* from stale media — manually `ninja` the user-mode targets first, or do a clean build. |

**Step 1 for you: establish ground truth.** Run a clean smoke on each arch and read `summary.json`
(do not scrape stdout). Don't trust this table — measure.

---

## 2. CRITICAL environment gotchas (these cost the prior session many hours)

### 2.1 KVM does NOT fire gdb breakpoints OR hardware watchpoints
On this host, under `-accel kvm`, **neither software breakpoints (`break`) nor hardware
breakpoints (`hbreak`) nor watchpoints (`watch`) fire.** They are silently ignored and the
guest runs to completion. **You MUST use TCG** for any gdb breakpoint/watchpoint work:

```bash
VIBEOS_NO_KVM=1 qemu/start.sh i386 --gdb     # forces -accel tcg; breakpoints + watchpoints WORK
```

KVM is fine for plain boots and for reading memory/registers via gdb (memory reads work under KVM).
It's only *breakpoints/watchpoints* that need TCG.

### 2.2 Boots are SLOW — do not mistake slowness for a hang
i386 KVM boots take **~80–150s**; the serial log sits at **581 bytes** ("`[freeldr] Switching
stack...`") or even **87 bytes** for a long time and then *races* through to the bugcheck/shell-ready
in a few seconds. **Wait at least 150s (KVM) / several minutes (TCG) before concluding a boot is
wedged.** The prior session repeatedly killed healthy boots at 50s thinking they'd stalled. To tell
slow-from-hung, sample the serial byte count twice ~15s apart, or read `$pc` via gdb (a PC in
`~0x1f000000` is freeldr/UEFI = still booting; `~0x80xxxxxx` spinning = kernel).

### 2.3 ninja does NOT always detect a `.rs` change
Editing a Rust file and running `ninja ntoskrnl`/`ninja win32k`/`ninja livecd` sometimes leaves the
binary **unchanged** (cargo/ninja dependency gap). **After editing any `.rs`, `touch` it before
building**, and verify the output timestamp moved:

```bash
touch sdk/lib/rust/rtl-heap/src/heap.rs       # or win32k-rust/src/window.rs, etc.
cd build-i386 && ninja ntoskrnl livecd
ls -la livecd.iso ntoskrnl/ntoskrnl.exe       # confirm the mtime updated
```

### 2.4 Stale `librtl.a` can mask Rust-heap edits
The Rust heap is compiled into `build-<arch>/sdk/lib/rtl/librtl.a`. If your heap edits don't change
runtime behavior, `rm -f build-i386/sdk/lib/rtl/librtl.a` before rebuilding to force a lean regen.

### 2.5 Clean up QEMUs
Background boots leak QEMU processes. `pkill -9 -f qemu-system-i386` between runs; `qemu/ps.sh`
lists live VMs across worktrees. Don't run many boots concurrently (KVM/socket contention).

---

## 3. The i386 corruptor — full diagnosis (this is where to start)

### 3.1 Symptom
After `[WIN32K] shell ready`, **BUGCHECK 0x1E** (`0xc0000005, 0x8056e8XX, <trapframe>, 0`).
The faulting PC is `RemoveEntryList+11` in ntoskrnl, reached via
`RtlAllocateHeap → RtlpRemoveFreeBlock → RemoveEntryList`. A heap free-list entry's forward link
(`flink`) has been overwritten with a non-pointer value, so walking the free list dereferences garbage.

The corrupt block is deterministically **`0xbc951398`** on the **GlobalUserHeap / session heap
(base `0xbc950000`)**. It is a *legitimately-free* "split-remainder" block (created when a neighbouring
buffer at `0xbc951378` was carved out of a larger free block). **A win32k write overflows the buffer
at `0xbc951378` and clobbers `0xbc951398`'s flink at `0xbc9513a0` (= `0xbc951378 + 0x28`, i.e. 8 bytes
past a 32-byte buffer).**

### 3.2 Root mechanism — CONFIRMED via TCG hardware watchpoint
The winning hunt (reproduce it if you need to re-confirm or catch other corruptors):

```bash
VIBEOS_NO_KVM=1 qemu/start.sh i386 --gdb      # note the gdb port (usually 12000)
# gdb batch:
target remote 127.0.0.1:12000
hbreak *0xf7442ca0                            # NtUserCreateWindowEx (runtime) — fires after the
continue                                      #   session heap is mapped
delete
watch *(unsigned int*)0xbc9513a0 if ((*(unsigned int*)0xbc9513a0)>>24)!=0xbc   # stop when flink stops being a heap ptr
continue                                      # catches the corrupting instruction
printf "PC=%#x val=%#x\n", $pc, *(unsigned int*)0xbc9513a0
info registers
x/48xw $esp
```

This caught **`NtUserDefSetText → memmove`** (win32k) copying 40+ bytes into the 32-byte
`strName` buffer at `0xbc951378`, value `0x80000` (WS_SYSMENU). That is a real overflow.

### 3.3 What makes it hard: the corruptor VARIES per boot
- One boot: `NtUserDefSetText` overflow (above).
- Another boot: a `USERTAG_DCE` ('Usdc') structure overwrote the block.
- Some boots: BUGCHECK with **no** `NtUserDefSetText` overflow at all.

So `NtUserDefSetText` is **one** corruptor, not the consistent first-cause. The strName logic in
`NtUserDefSetText` (win32k-rust `window.rs:6295`) and `DefSetText` (`window.rs:6219`) is **faithful
to the C** (`win32ss/user/ntuser/window.split5.c:167`), and all `strName` mutation sites keep
`MaximumLength` consistent with the buffer. For `NtUserDefSetText`'s FITS-path to overflow, the
buffer block's header `Size` must have been corrupted *smaller* by an **earlier** overflow.

**Leading theory: a CASCADE.** A first overflow shrinks block `0xbc951370`'s header `Size`
(e.g. 9 → 5), making the buffer *look* 32 bytes; then `NtUserDefSetText`'s FITS path trusts the
still-large `MaximumLength` (which equals the *original* allocation size) and overflows the now-undersized
buffer. The true first-cause is an *earlier* overflow not yet pinned.

### 3.4 Theories already RULED OUT (don't re-litigate these)
- **Rust heap-impl bug** — 4 instrumented validation points; corruption only appears at the victim's
  alloc-entry, never introduced at any heap alloc/free *exit*. `RtlpSplitEntry` (heap.rs:1890),
  `RtlpInsertFreeBlockHelper` (493), `RtlpRemoveFreeBlock` (650) all verified faithful to C `heap.c`.
- **Heap-API race** — forcing the session heap to serialize (ignore per-call `HEAP_NO_SERIALIZE`)
  did **not** fix it. So it's not two threads racing inside `RtlAllocateHeap`/`RtlFreeHeap`.
- **Under-sized alloc from FreeHintBitmap desync** — a safety-walk in the alloc hint path did **not**
  help (and was reverted). `RtlpSplitEntry` correctly sets the allocated block `Size = Index`, so
  `RtlAllocateHeap` does not return under-sized blocks.
- **Simple double-free** — `RtlFreeHeap` already rejects freeing a non-busy block.
- **UAF stray-write being the *direct* corruptor** — it's an overflow of an adjacent buffer, not a
  write through a dangling pointer to the freed block itself.

### 3.5 RECOMMENDED next step: catch the FIRST overflow
Enable **`HEAP_TAIL_CHECKING`** on the session heap to find the first-cause:
- In the Rust heap (`rtl-heap/src/heap.rs`), force `HEAP_TAIL_CHECKING_ENABLED` into `(*Heap).Flags`
  for session heaps (`heap >= 0x8000_0000`) early. `RtlAllocateHeap` already fills the 8-byte tail
  with `HEAP_TAIL_FILL` when that flag is set (see the `HEAP_TAIL_CHECKING_ENABLED` block ~line 2171).
- Add a check on every session-heap free/realloc/validate: if a busy block's tail no longer matches
  `HEAP_TAIL_FILL`, that block was overflowed — print its address, size, and the allocating caller
  (capture the win32k return address; see the existing stack-scan helpers). The block whose tail dies
  *first* is the first-cause; its **under-sized** allocation (request vs actual content) names the bug.

Alternative: repeat the TCG watchpoint several times — under TCG (deterministic), check whether the
*first* corruptor is always `NtUserDefSetText` or genuinely varies. If deterministic, fix that site's
root (why its buffer is undersized) with confidence.

### 3.6 Symbolication (relocs pinned by byte-matching live memory)
- **ntoskrnl reloc `0x80091000`** — `addr2sym.sh i386 --reloc 0x80091000 <runtime_addr>` resolves
  both C *and* Rust symbols. (The older `0x7fe00000` value in some notes is WRONG.)
- **win32k.sys reloc `0xf73df000`** (load base `0xf73ef000`). win32k lives in **session space
  `0xf74xxxxx`**, NOT `0x80xxxxxx` — stack-scan filters that only look at `0x80200000–0x80900000`
  will MISS all win32k frames. **Re-derive this reloc after ANY win32k rebuild** (offsets shift):
  read the bytes at a known stack return address via gdb and grep them in
  `build-i386/win32ss/win32k.sys` (`.text` starts at file VA 0x11000 / file offset 0x1000), then
  `reloc = runtime_addr - found_file_VA`. Build a fresh symbol table with
  `nm build-i386/win32ss/win32k.sys`.
- `qemu/build-index.sh i386` builds `qemu/symbols-i386.tsv` for `addr2sym.sh`.

### 3.7 Reliable repro
```bash
unset VIBEOS_NO_KVM
RUN=$(qemu/start.sh i386 | tail -1)           # no --gdb, KVM, fast
# wait — poll qemu/runs/$RUN/serial.log for "BUGCHECK" or "shell ready"; allow ~150s
grep -aE "GHDESYNC|BUGCHECK" qemu/runs/$RUN/serial.log
```

---

## 4. Debug instrumentation currently in the tree (uncommitted)

`git diff` shows three modified files. **All `[DEBUG]`/`GHDESYNC`/`GHTRACK`/`DEFSETTEXT-BAD`
instrumentation must be removed before any commit** (the translations should have zero net debug
diff). It is left in deliberately so you can keep using it.

- **`sdk/lib/rust/rtl-heap/src/heap.rs`** (+163 lines, all `#[cfg(target_arch="x86")]` debug):
  - `dbg_caller_scan()` / `dbg_capture_alloc_ras()` — stack-scan for ntoskrnl (`0x80200000–0x80900000`)
    AND win32k (`0xf7000000–0xf7700000`) return addresses. Prints `ra=`/`allocRA=`.
  - `dbg_validate_freelist(Heap, ctx)` — walks the session-heap free list (capped 24 entries), prints
    `[GHDESYNC]` with the corrupt entry + a content dump of the block and its predecessor, plus the
    last alloc/free pointers and caller stack. Called at alloc-entry (`ctx=A110`), free-entry (`F1EE`),
    free-exit (`F1FE`). **Only `A110` ever trips** — the corruption is a raw write, not introduced at a
    heap op exit.
  - `GHTRACK` hooks in `RtlAllocateHeap` (dedicated return) and `RtlFreeHeap` that print the caller
    when block `0xbc951398` (user `0xbc9513a0`) is specifically allocated/freed. (It's a split-remainder,
    so these rarely fire — useful to confirm.)
  - **The FreeHint safety-walk was already reverted** — that part of the file is clean.
- **`sdk/lib/rust/win32k-rust/src/window.rs`** (+66 lines):
  - A `DbgPrint` extern declaration near the top.
  - `[DEFSETTEXT-BAD]` dump in `DefSetText` + `NtUserDefSetText` (×3 sites incl. the cfg'd-out amd64
    copy): prints `buf / blkusable / maxlen / newlen / curlen / wnd` when `MaximumLength > actual
    block usable` OR a copy would overflow. Capped at 24.
- **`sdk/lib/rust/win32k-rust/src/ime.rs`** (+7/-1): **a REAL fix, KEEP it.** Line ~3288:
  `co_IntCreateDefaultImeWindow` passed `widestring_default_ime().as_ptr()` — a pointer to a
  *temporary* stack array — into `RtlInitLargeUnicodeString`, leaving `WindowName.Buffer` dangling
  before `co_UserCreateWindowEx` reads it. Fixed by binding to a local `ime_name` first.

When you reach a clean fix: strip all the `[DEBUG]`/GH*/DEFSETTEXT instrumentation, keep the `ime.rs`
fix, keep your real corruptor fix, run a `--clean` smoke on all 3 arches, then ask the user before committing.

---

## 5. Tooling reference

- **Front door:** `./vibeos-cli` (read `tests/qemu/cli/README.md` once).
  - `./vibeos-cli build --arch i386 [--clean]` — `--clean` nukes rust targets (proves cold-link).
  - `./vibeos-cli smoke --arch i386` — the verdict; refuses stale media. Read
    `/tmp/vibeos-cli/<ts>-smoke-<arch>/summary.json`, not stdout.
  - `./vibeos-cli check` — lint + baseline-diff gate (0 NEW lints).
- **Kernel debug toolkit:** `qemu/start.sh`, `stop.sh`, `wait-for.sh`, `tail.sh`, `status.sh`,
  `inspect.sh` (bt / symbolicate-pc / dump-trapframe / triage-bugcheck), `run-until.sh`,
  `addr2sym.sh` / `sym2addr.sh`, `build-index.sh`, `ps.sh`. Docs: `qemu/README.md`,
  `docs/engineering/kernel-debugging-infra.md`. (Note: `triage-bugcheck` is amd64-only.)
- **Manual boot media build:** `cd build-i386 && ninja livecd` (x86) produces `livecd.iso`.
- **i386 KTRAP_FRAME offsets** (BUGCHECK 0x1E P3 = trapframe ptr, NOT fault VA): Eip@+0x68,
  Esp@+0x74, Ebp@+0x60, Esi@+0x58, Edi@+0x54, Eax@+0x44, EFlags@+0x70. Rust release omits frame
  pointers, so ebp-chain unwinds fail — scan the stack from Esp for return addresses.

---

## 6. After i386 is green

1. **amd64** (`./vibeos-cli smoke --arch amd64`): finish the win32k flip if undefined fastcall symbols
   remain. amd64/arm64 NTAPI = cdecl (just `Name`); i386 NTAPI = stdcall (`_Name@N`). Mismatched
   `extern "C"` vs `extern "system"` link silently on amd64/arm64 but break i386 — use
   `tools/rustconv/rustconv/verify_abi.py`. Watch for the amd64 `xchg` 64-bit-vs-32-bit zero-extension
   hazard in CMPXCHG16B inline asm if you touch interlocked/SList code.
2. **arm64** (`./vibeos-cli smoke --arch arm64`): **rebuild user-mode cohab targets manually first**
   (the arm64 build path skips them) or you'll get a false green from stale media. arm64 uses idle
   detection, not `shell ready`.
3. All three must pass **independently** and on a **`--clean`** build (cargo incremental can mask
   regressions). Always check the saved screenshot `qemu/<arch>_smoke.png`.

---

## 7. One-line summary

i386 boots to the desktop then a win32k buffer overflow into a free session-heap block
(`0xbc951398`) corrupts the heap free list → BUGCHECK 0x1E. Root mechanism confirmed
(`NtUserDefSetText` overflow, TCG watchpoint), but the corruptor varies per boot → it's a cascade
whose *first* overflow is unpinned. Next move: `HEAP_TAIL_CHECKING` on the session heap to catch the
first-overflowed allocation. Then verify/fix amd64 and arm64. Use TCG for breakpoints, be patient with
slow boots, `touch` `.rs` files before rebuilding, and never revert/weaken to go green.
