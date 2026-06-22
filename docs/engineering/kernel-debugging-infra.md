# Kernel Debugging Infrastructure — Design Doc

**Status:** Phase 0–4 implemented and validated. Phase 5+ helpers (recipes, `dump-process`, `walk-event-waiters`, etc.) are added as bugs ask for them.

**Acceptance results:**
- Test A (amd64 KiUnwaitThread BUGCHECK): caught on first attempt; `decode-bugcheck` reported `0x1E KMODE_EXCEPTION_NOT_HANDLED` with `Param1=STATUS_IN_PAGE_ERROR`, faulting PC `0xfffff8000047e8d3` resolved to `KiUnwaitThread+0x23` — matching the offset documented in `amd64-taskbar-regression.md`.
- Test B (arm64 hang triage): `wait-for.sh` timeout returned structured JSON with serial tail, alive=true, serial bytes; QEMU stayed reachable; `stop.sh` was clean.
- Test C (parallel runs): two arches concurrent, distinct gdb ports (12000/12001), independent stop.

**Audience:** Written for a Claude Code agent to drive autonomously. Not for a human at two terminals — though humans can use it too.

## Motivation

Across recent sessions we've hit the same kind of wall repeatedly:

| Wall | What happened | What we did |
|------|---------------|-------------|
| amd64 BUGCHECK 0x1E in `KiUnwaitThread` | corrupt `Thread->WaitBlockList` from stale wait list | flake_probe runs + manual disasm + temp `DbgPrint` instrumentation in Rust source + rebuild + repro |
| amd64 winsta hang after `[WIN32K] shell ready` | syscall doesn't return to winlogon user mode | inferred from missing log lines; *no concrete diagnosis* |
| arm64 0/7 entirely | boot path silently fails before user mode | partial-boot markers in vm.py, ramdisk archeology, log diffing |
| `lpc/connect.rs` parked | crash inside one Rust function during port connect | reverted; no further debug |
| `cmdata.c` complexity | preprocessor-heavy header, hard to chop | translation-philosophy plan + multi-layer chopping plan |
| Various translation regressions | new Rust file integrated, smoke fails | bisect across translations, reading disasm, occasional `objdump` walks |

The recurring pattern: **probe → infer from serial → disassemble → add `DbgPrint` → rebuild → repro**. ~2 minutes per iteration dominated by build time. A single bug typically takes dozens of iterations.

Compounding factors:
- The kernel is increasingly Rust. Rust generics, inlining, and monomorphization hide structure that disasm-only investigation can't recover.
- Three architectures with arch-specific quirks (i386 fastcall ABI, amd64 captures(none) miscompilation history, arm64 no debugcon).
- *No interactive way to inspect a running kernel*. KDB exists but the smoke harness routes around it.

The thesis: **a modest investment in interactive, agent-drivable kernel debugging will pay back across every remaining translation, every regression, every flake.**

## Goals

1. **Reduce time-to-root-cause** for kernel bugs from hours of disasm-and-DbgPrint dancing to minutes of "set a breakpoint, dump the structure."
2. **Symbol-aware inspection** of running or halted kernels — think `Thread->WaitBlockList` by name, not `0x88(%rbx)`.
3. **Drivable autonomously by a Claude Code agent.** Every operation is a single-shot tool call returning structured output. No interactive sessions to babysit.
4. **All 3 architectures** with the same workflow.
5. **Zero external runtime deps** beyond what's already on a Linux/WSL2 dev host (no Windows host, no Wine).
6. **Fast inner loop** — no rebuild required for inspection of a running kernel.
7. **Composable** — primitives that can be chained and scripted.

## Non-goals

1. Replicating WinDbg's full `!extension` command surface verbatim.
2. Source-level debugging of *user-mode* processes.
3. Replacing smoke tests as the CI signal of record.
4. Becoming a build-time dependency.
5. **Source-level stepping** (n/s/list with cursor tracking). Stepping is human UX. We do source-level *inspection* instead — see Design Principles.

## Design Principles for Agent-Driven Debugging

The most important architectural insight: **GDB is a conversation; an agent is not.** The agent's execution model is "run a bash command, get stdout/stderr, decide next step." Every primitive in this infrastructure must respect that contract. The other principles flow from this one.

### 1. Stateless batch invocations

Every operation is one bash command in, structured result out. No persistent GDB sessions, no terminals to babysit. Internal GDB calls use `-batch -ex` patterns. The agent never owns state between tool calls.

### 2. Run IDs and isolated artifacts

Each kernel run produces a unique run-id and per-run artifact directory:

```
qemu/runs/<run-id>/
  manifest.json       # arch, qemu pid, gdb port, qmp socket, start time,
                      # kernel image path, kernel mtime, git sha + dirty bool,
                      # kernel_runtime_base (cached after first inspect)
  serial.log          # all serial output, append-only, RAW BYTES
  gdb.log             # tee of all GDB sessions targeting this run
  qmp.sock            # QMP control socket
  qemu.launch.log     # qemu's stdout/stderr (catch launch failures)
  pid                 # qemu process pid
```

`serial.log` stays raw on disk — keeps the option to investigate weird output later. `tail.sh` and `wait-for.sh` sanitize on read (see Lifecycle table). The manifest captures both git SHA + dirty bit (from `git status --porcelain`) and the booted binary's mtime, because a clean SHA can still mask "you booted a stale `ntoskrnl.exe` from a prior checkout" — the binary mtime is the ground truth.

> **Implementation note:** the original design called for a serial Unix socket (`serial.sock`) so future interactive clients could attach. We tried QEMU's `-chardev socket,...,server=on,wait=off,logfile=...` but QEMU buffers socket output until a client connects and the logfile stays empty in the meantime, defeating the point. The shipped scripts use `-serial file:<serial.log>` (write-only, immediate flush). Interactive serial, if it ever proves useful, should be exposed via QMP `human-monitor-command` rather than reviving the socket+logfile combo.

> **kernel_runtime_base caching:** the runtime kernel base (where ntoskrnl.exe is loaded in kernel VA space — typically `0xfffff80000400000` on amd64) is discovered by `find-kbase.py` scanning memory for the `MZ` PE header. The first `inspect.sh`/`run-until.sh` call writes the value back into the manifest so subsequent calls skip the scan.

Multiple runs (different arches, parallel investigations) coexist without collision. Ports auto-allocated. Sockets per-run. Older runs preserved until explicitly cleaned. Run-ids of the form `<arch>-YYYYMMDDHHMMSS` (or similar) are stable and human-readable.

### 3. Idempotent lifecycle

`start.sh` cleans up zombie QEMU processes for the same arch before starting. `stop.sh` is safe to call multiple times. `status.sh` always returns truth without side effects. The agent never needs to remember to clean up after itself.

### 4. Embedded preflight with fix-it commands

`start.sh` runs preflight automatically and refuses to start if the platform isn't debug-ready. The agent never confuses "kernel bug" with "debugger plumbing broken." Preflight as a separate ritual would be forgotten exactly when it's needed.

Every preflight failure prints the **exact command to fix it**, not just a description. "DWARF is thin" alone is a round-trip — "DWARF is thin in C, run `cmake -DCMAKE_C_FLAGS_RELEASE='-O2 -g3' ...` from `build-<arch>/`" is a fix the agent can execute directly. Detection runs per-axis (C and Rust independently) so the suggested fix is targeted, not shotgun.

### 5. Rich timeouts

When a bounded operation times out, the response includes recovery context (last N serial lines, QEMU process state, anything else cheap to attach). A bare "timed out" response wastes the agent's next turn re-deriving what just happened.

### 6. Source-level inspection, not stepping

DWARF symbols turn `0x482ac0` into `KiUnwaitThread+0x20`, `info locals` returns named locals, `b PspCreateProcess` works by name. **What we don't build:** workflows around `next`/`step`/`list` that ask the agent to track a cursor across batch invocations. Those cost thousands of tokens for what `bt` + `dump-thread` provide in one call.

### 7. Output format follows structure, not policy

JSON when the result has structure (thread state, decoded bugcheck args, manifest data, parsed wait-block contents). Labeled flat text when it's a list (backtrace, registers, memory hex). Both are equally parseable; uniformity isn't worth the token cost.

### 8. Translation-boundary tagging

Every backtrace frame tagged based on DWARF source-file extension. Four tags, not two:

| Tag | Source extension |
|-----|------------------|
| `[Rust]` | `.rs` |
| `[C]` | `.c`, `.cpp` |
| `[ASM]` | `.S`, `.asm`, `.s` |
| `[???]` | no DWARF info / no source mapping |

`[???]` is the genuinely valuable tag — it flags exactly the frames where symbol resolution failed, which is usually the interesting one. Caveat: `global_asm!` blocks (e.g. `KiSListPop` in `slist/src/lib.rs`) tag as `[Rust]` since DWARF reports the `.rs` source. That's correct — the tag is provenance, not implementation language. Fixes the recurring "Rust function reads field that C shim writes" confusion (KeGdiFlushUserBatch, COMPUTE_HASH_CHAR, the wait-block setup chain). Cheap to implement, default-on.

## Architecture

A small set of orthogonal commands, all run-id-based, all batch-invocable. Each command does one thing.

### Lifecycle

| Command | Purpose |
|---------|---------|
| `qemu/start.sh <arch> [--gdb] [--gdb-wait]` | Spawn QEMU with embedded preflight; emits run-id and manifest. `--gdb` opens gdbstub, `--gdb-wait` additionally halts at first instruction (for arm64 step-from-start work). |
| `qemu/status.sh <run-id>` | Process state, current serial log size, gdb attach state. No side effects. |
| `qemu/stop.sh <run-id>` | Kills QEMU; preserves artifacts. Idempotent. |
| `qemu/wait-for.sh <run-id> --pattern <regex> --timeout <secs>` | Polls serial log for pattern. Sanitizes output (`tr -cd '\11\12\15\40-\176'`) before regex match and before returning to caller — KDB ANSI escapes and bugcheck binary garbage otherwise break grep mid-match. On timeout: returns last 10 serial lines + QEMU process state. |
| `qemu/tail.sh <run-id> [--lines N]` | Cheap read-only access to serial log. Sanitized on read; raw bytes preserved on disk. |
| `qemu/ps [--json]` | Global "what's running right now" snapshot across **all** worktrees/sessions (reads the shared `/tmp/vibeos-qemu-locks/` lockfiles, not the per-worktree manifests). Columns: arch, source (smoke/start), qemu pid, owner pid (`!` = owning session dead), age, time-to-deadline (`OVERDUE` if past the cap), slot, and where to follow its serial. Plus a `slots: N/10 in use` summary. One-shot, never streams. |
| `qemu/reap.sh [--dry-run] [--verbose]` | Cull orphan QEMUs — those whose owning session died, **or** any VM past its 30-min lifetime cap regardless of owner. Auto-invoked by `start.sh` and the smoke harness; safe to run manually anytime (idempotent, never touches a live-owner VM still within its cap). |

### Resource management — concurrency cap, lifetime cap, orphan reaping

Three independent guards keep QEMUs from exhausting the host's KVM slots / RAM, all coordinated through the shared lock dir `/tmp/vibeos-qemu-locks/`:

- **Global concurrency cap** (`QEMU_MAX_CONCURRENT`, default 10; env `VIBEOS_QEMU_MAX_CONCURRENT`). A pool of `flock`'d slot files (`slots/slot-NN`) caps how many QEMUs run at once *across every worktree*. A launcher that can't get a slot **queues** (up to `VIBEOS_QEMU_QUEUE_TIMEOUT`, default 30 min) instead of piling another VM onto a saturated host. The slot is acquired on an fd that the QEMU **inherits**, so it releases automatically when QEMU exits — even though `start.sh` exits immediately after launch (BSD flock OFD semantics). Queue time is excluded from the lifetime cap (the deadline is computed *after* the slot is acquired).
- **Hard lifetime cap** (`QEMU_MAX_LIFETIME_SECONDS`, default 1800; env `VIBEOS_QEMU_MAX_LIFETIME`). No QEMU outlives 30 min. Enforced three ways: the smoke harness arms an in-process `threading.Timer`; `start.sh` spawns a detached watchdog (it has no long-lived launcher to host a timer); and every lockfile/manifest records a `deadline` so any `reap.sh` pass culls a past-deadline VM even if its owner is still alive but wedged.
- **Orphan reaping** (`reap.sh`). An orphan = a QEMU whose owning session (the first non-shell ancestor — claude/tmux/sshd/terminal — resolved by `lib/common.sh:owner_pid_for`) has died. Reaping a live-owner VM within its cap is never done; it might be another agent's active run.

### Inspection (single-shot, no execution)

| Command | Purpose |
|---------|---------|
| `qemu/inspect.sh <run-id> --command <name> [args] [--json]` | Single batch GDB invocation. Built-in commands listed below. Always issues `delete breakpoints` on attach as defense-in-depth — eliminates "stale breakpoint from a previous run-until wedged the next inspect" bugs. |

Initial inspect commands (Phase 2):
- `bt` — backtrace with translation-boundary tags
- `bt-all-cpus` — same, across all vCPUs (we run 1 vCPU today, but the abstraction is right)
- `regs` — register dump
- `decode-bugcheck` — symbolicate bugcheck args (status code, fault VA, PC → function+offset)
- `dump-thread [addr]` — pretty-print `_KTHREAD` with named fields; defaults to current thread. Uses DWARF type info when available, falls back to a per-arch table of hardcoded offsets when ntoskrnl's bindgen DWO CUs are missing (the common case today). Means `dump-thread` works even on freshly-built kernels with sparse DWO output.
- `symbolicate-pc [addr]` — single address → symbol+offset+source-line

Phase 2.5 inspect commands (added 2026-05-02 from the amd64 KiUnwaitThread session):
- `dump-trapframe <addr>` — pretty-print KTRAP_FRAME at addr using a per-arch hardcoded layout table. Replaces 30+ lines of manual offset arithmetic on a `x/64gx` raw dump. Currently surfaces the registers and segment selectors you actually want for a kernel #PF/#GP/etc. — not every Dr*/Xmm*. Layout mirrors `sdk/include/ndk/<arch>/ketypes.h` and must be bumped if that struct changes.
- `pool-tag <addr>` — walk backwards from a kernel pointer to find the enclosing POOL_HEADER, decode tag (4-byte ASCII), block size, pool type. Tries multiple back-offsets (raw pool body / pool+OBJECT_HEADER / pool+OBJECT_HEADER+CreatorInfo) so it works whether the user pointed at a pool body or an object body.
- `identify-object <addr>` — best-effort "what is this kernel pointer?" Combines DISPATCHER_HEADER.Type @ +0x00 with the pool-tag scan, and for `ProcessObject` reads `ImageFileName` at the body-relative offset (amd64=0x260). One-shot answer to questions like "is `0xfffffa80015c01f0` a thread, a process, or an event?" — replaces the manual chain of byte-reads, ASCII decode, and bindgen-offset lookups.
- `triage-bugcheck` — headline command for the "kernel BUGCHECK in some flake" loop. Decodes KeBugCheckEx args, drills into the trap frame at Param3 (when stop code is `0x1E KMODE_EXCEPTION_NOT_HANDLED`), and runs `identify-object` on the pointer in `Rcx` (the asm operand at the fault site for KiReadyThread-style sites). One tool call, structured output, identifies the bogus pointer end-to-end.

Phase 5+ inspect commands (incremental as bugs ask for them):
- `dump-process`, `walk-event-waiters`, `find-pool`, `dump-irql`, `dump-cpu`

### Bounded execution

| Command | Purpose |
|---------|---------|
| `qemu/run-until.sh <run-id> --break <symbol> --timeout <secs> [--then <inspect-command>]...` | Single bounded primitive that wraps breakpoint-set + continue + wait + inspect + cleanup. Replaces the fragile "set breakpoint in one GDB session, hope it persists, attach later" pattern. |

The `--then` flag chains inspect commands to run after the breakpoint hits, so the agent gets the snapshot it needs in one tool call.

`run-until.sh` always issues `delete breakpoints` before exiting — on hit *and* on timeout. Combined with `inspect.sh` clearing on attach, this is belt-and-suspenders against stale breakpoint state leaking between batch invocations.

### KDB fallback

| Command | Purpose |
|---------|---------|
| `qemu/kdb-cmd.sh <run-id> --cmd <name>...` | Batch wrapper for the in-kernel debugger. KDB auto-activates on bugcheck without GDB needing to be attached, so it stays valuable as the always-available fallback. |

### Snapshots — fast-iteration restore

| Command | Purpose |
|---------|---------|
| `qemu/snapshot.sh save <run-id> [--name <n>]` | Capture a running VM's full state (RAM + devices) to `qemu/snapshots/<n>.mig` via QMP `stop` + `migrate "exec:cat > file"`. Stamps a `.meta` sidecar with the kernel sha for staleness detection; stops the source VM after. |
| `qemu/snapshot.sh restore <n> [--gdb] [--allow-stale]` | Launch a new run via `start.sh --incoming` and resume at the captured point — ~1-2s vs a ~50s arm64 cold boot. Returns a normal run-id, so `inspect.sh`/`run-until.sh`/`wait-for.sh` all work against it unchanged. |
| `qemu/snapshot.sh list` / `rm <n>` | List snapshots with staleness, or delete one. |

Restore skips the cold boot for every debug cycle against a fixed kernel. Two load-bearing rules:

- **Snapshot at a QUIESCENT point, not mid-churn.** Migration-to-file is faithful when the guest is settled (idle loop, well past `shell ready`), but capturing during high activity (e.g. exactly at `shell ready`, while explorer is spawning threads) perturbs **~20% of restores** into a spurious post-resume `BUGCHECK 0x50` that *does not occur on fresh boots* (verified arm64-virt, 2026-06: 0/5 fresh vs 2/9 mid-churn restores, fault clustered at a kernel-stack VA; 0/10 idle-snapshot restores). Wait for serial to go quiet before `save`. This is a property of live-migration consistency, not a guest bug — so the snapshot path is **not** a substitute for record/replay when the goal is to *re-roll* a mid-churn heisenbug (that's what `-icount rr=` is for).
- **Stale = refused (fail loud).** The snapshot embeds the OLD kernel in RAM; gdb symbolication uses the on-disk image. If the kernel was rebuilt since `save`, `restore` refuses (symbols would mislead) unless `--allow-stale`. Mirrors smoke's stale-media refusal. `.mig` files (~350M) live in the gitignored `qemu/snapshots/`.

#### Record/replay (`-icount rr=`) — attempted, BLOCKED on arm64 (2026-06)

The bit-exact deterministic-replay path (`-icount shift=N,rr=record,rrfile=…` +
`blkreplay` on the boot disk) is the textbook tool for re-rolling a mid-churn
heisenbug under gdb, where snapshot-restore can't (see the quiescent-point rule
above). The QEMU substrate is present (record launches and writes an rr log), but
**`-icount` hangs our arm64 early boot before the kernel ever loads, at every
shift tried:** `shift=auto` reaches freeldr's "Switching stack…" then hangs;
`shift=3`/`shift=7` (with `sleep=off`) stall even earlier, in UEFI firmware. A
recording that doesn't move resets at 300s via the EFI boot-services watchdog.
This is a firmware/loader-under-icount incompatibility (almost certainly a
hardware-timer poll loop that starves when time is coupled to instruction count),
not fixable from the tooling layer — and even if the early-boot hang were fixed,
reaching a *post-shell-ready* heisenbug under icount would be punishingly slow and
watchdog-limited. **Deferred, not abandoned:** revisit if/when someone makes the
arm64 boot path icount-tolerant; until then the snapshot tool above is the
determinism/fast-iteration mechanism for arm64.

### Offline tools (no kernel needed)

| Command | Purpose |
|---------|---------|
| `qemu/build-index.sh <arch>` | Build cached symbol index `qemu/symbols-<arch>.tsv` from `ntoskrnl.exe`. Run after each kernel build. |
| `qemu/addr2sym.sh <arch> [--reloc <hex>] <addr>...` | Address → symbol+offset. Lookup against cached index. `--reloc` subtracts a runtime base (e.g. `0xfffff80000000000`) so a PC from a serial-log bugcheck resolves correctly. |
| `qemu/sym2addr.sh <arch> <symbol> [--regex]` | Symbol → address. Handles i386 PE decoration (cdecl `_<sym>`, stdcall `_<sym>@<n>`, fastcall `@<sym>@<n>`) without the caller having to guess. |

> **Note on naming:** the implementation uses `addr2sym.sh` / `sym2addr.sh` (not `sym.sh`/`addr.sh` as initially named in this doc) because `qemu/sym.sh` already exists as a UEFI bootloader symbolicator and is worth preserving. The verb-named pair also reads better — there's no ambiguity about which direction each tool runs.

### Differential utilities (Phase 5+)

| Command | Purpose |
|---------|---------|
| `qemu/logdiff.sh <run-id-a> <run-id-b>` | Diff serial outputs, find divergence point — directly addresses "i386 works, arm64 doesn't, where do they fork?" |
| `qemu/memdiff.sh <run-id-a> <run-id-b> <addr> <len>` | Dump memory at addr from each, show byte diff. Catches translation-introduced layout/constant bugs. Field-mapping via DWARF deferred. |
| `tools/since.sh <good-sha> <bad-sha> --path <glob>` | One-line-per-touched-file diff between two commits in a path glob. Triage tool for "amd64 worked at X, broke at Y, what changed?" |

## Approaches Considered

| Approach | Verdict |
|----------|---------|
| **WinDbg over KD64.** Mature, but requires Windows host or Wine; we emit DWARF, not PDB, so its `!extension` commands don't recognize symbols without cv2pdb conversion (its own project). | Defer indefinitely. The DWARF/PDB gap eats most of the value. |
| **KDB (in-kernel).** Already there, auto-activates on bugcheck, knows kernel symbols at runtime. Limited command set; hard to break into hung kernel; arm64 not yet integrated. | Keep as fallback with batch wrapper (`kdb-cmd.sh`). |
| **GDB + QEMU gdbstub.** DWARF native; Linux-native, scriptable; works on all 3 arches; Ctrl-C interrupts hangs. Doesn't know Windows kernel structures out of the box but DWARF gives field names for free. | Foundation. The whole architecture above. |
| **GDB Python helpers.** Layer on top of GDB. Wrap common queries (`dump-thread`, `walk-event-waiters`) in 50-line Python scripts. | Tier 3 helpers, incremental. |
| **Bespoke Python debugger via QMP + gdbstub.** Maximum control, maximum cost. | Out of scope. |

## Implementation Phases

**Phase 0 — Foundation (~1 hr, no kernel work).**
- Run-directory layout and manifest schema (write down the JSON shape — including git SHA, dirty bit, `ntoskrnl.exe` mtime).
- Preflight tool with **fix-it commands**:
  - `gdb-multiarch` presence → "apt install gdb-multiarch"
  - DWARF richness, **per axis**: check C objects (`readelf --debug-dump=info build-<arch>/ntoskrnl/CMakeFiles/ntoskrnl.dir/*.c.obj | grep -c DW_TAG_variable`) and Rust objects (same on `rust-kernel-target/.../*.rlib` extracted with `ar x`) independently. Emit *targeted* fix-it: cmake `CMAKE_C_FLAGS_RELEASE` patch if C is thin, `Cargo.toml [profile.release] debug = true` (or `RUSTFLAGS="-C debuginfo=2"`) if Rust is thin.
  - QEMU gdbstub responsiveness (try a connect + `info registers`, expect <1s)
  - `KdDebuggerDataBlock` discoverability (symbol exists in DWARF)
- Port auto-allocation strategy (scan free, persist in manifest).

**Phase 1 — Lifecycle (~2 hr).**
- `start.sh` (with embedded preflight), `stop.sh`, `status.sh`, `wait-for.sh`, `tail.sh`.
- Idempotent: zombie cleanup before start, safe re-stop, status as pure read.
- Rich timeouts in `wait-for.sh`: returns serial tail + process state on timeout.
- **Acceptance:** start two arches concurrently without collision; recover from a killed QEMU.

**Phase 2 — Single-shot inspection (~2 hr).**
- `inspect.sh` as the core batch GDB primitive.
- Image-base discovery: read `KdDebuggerDataBlock.KernBase` at attach (fallback: PE header scan starting at conventional kernel VA).
- Wire Rust pretty-printers (rust-gdb scripts) so `_KTHREAD` and Rust enums display correctly. **Without this, every Tier 3 helper output looks like raw bitfield bytes.**
- First-class helpers: `bt` (with `[Rust]`/`[C]` tags), `decode-bugcheck`, `dump-thread`, `symbolicate-pc`.
- Symbol index: `build-index.sh`/`sym.sh`/`addr.sh` working against cached index.

**Phase 3 — Bounded execution (~1 hr).**
- `run-until.sh` — single command combining break + continue + wait + inspect + cleanup.

**Phase 4 — Acceptance (~30 min).**
- Drive the amd64 `KiUnwaitThread` BUGCHECK end-to-end as the acceptance test (assertions below).
- Deliverable: a transcript proving the infra works, plus a list of helpers/recipes that proved missing in practice.
- This is the spike that informs Phase 5+ priorities.

**Phase 5+ — Incremental (as bugs ask for them).**
- Cross-arch sym lookup (e.g., "what's the i386 equivalent of this amd64 symbol?")
- `memdiff.sh` (raw bytes first; DWARF field mapping later)
- `logdiff.sh`
- More kernel-aware helpers: `dump-process`, `walk-event-waiters`, `find-pool`, `dump-irql`, `dump-cpu`
- "What changed since X" triage tool: `tools/since.sh`
- Recipe cookbook (next section)
- Per-run-id `session.log` recording every command invocation, so the agent can review what it's already inspected without burning a tool call to re-run

## Recipes (Phase 5+ artifact)

The most valuable artifact for the agent is a recipe file mapping bug archetypes to command sequences. Drafting these here as a starting point — the real list will form after Phase 4 reveals what's missing.

```
# Kernel bugchecks at boot
start --gdb-wait
run-until KeBugCheckEx --timeout 90 \
  --then bt-all-cpus \
  --then decode-bugcheck

# Kernel hangs after marker M
start --gdb
wait-for M
wait-for "<expected-next>" --timeout 30
# on timeout, the wait-for response includes serial tail
inspect bt-all-cpus
inspect dump-cpu

# Translation produces wrong constant or layout
# (boot two kernels, one with C version active, one with Rust)
run baseline (C version), inspect memdiff <addr> <len>
run candidate (Rust version), inspect memdiff <addr> <len>
diff the two byte snapshots
```

These remove decision overhead. With recipes the agent executes a known plan; without them it reasons from first principles every time. Token savings dwarf the cost of writing them down.

## Acceptance Tests

The infra is "done enough to ship Phase 1–4" when these pass autonomously. Each test specifies *outputs* the tools must produce, not just that they run.

**Test A: amd64 KiUnwaitThread BUGCHECK.**
- `start.sh amd64 --gdb-wait` returns run-id within 5 seconds.
- `run-until.sh <run-id> --break KeBugCheckEx --timeout 120 --then bt-all-cpus --then decode-bugcheck` completes within 120 seconds.
- `decode-bugcheck` output contains stop code `0x0000001E` and identifies the fault VA.
- `bt-all-cpus` output contains a frame in `KiUnwaitThread`.
- `dump-thread <suspect-addr>` either validates the address as KTHREAD or reports invalid pool/object header.
- Tool exits nonzero only on infra failure, not on the target's bugcheck.

**Test B: arm64 boot hang triage.**
- `start.sh arm64` returns run-id.
- `wait-for.sh --pattern "<known-marker>" --timeout 60` succeeds.
- `wait-for.sh --pattern "<expected-next>" --timeout 30` times out and returns serial tail + process state.
- `inspect.sh --command bt-all-cpus` returns at least one symbolicated frame for the running CPU.
- QEMU remains stoppable after; no leaked processes.

**Test C: parallel runs.**
- `start.sh amd64 --gdb` and `start.sh i386 --gdb` succeed concurrently with distinct run-ids and ports.
- `inspect.sh` against each returns independent results.
- `stop.sh` of one doesn't affect the other.

## Open Questions

| # | Question | Decision |
|---|----------|----------|
| 1 | Image-base discovery method | Read `KdDebuggerDataBlock.KernBase` via gdbstub. Fallback: PE header scan from conventional kernel VA. |
| 2 | gdbstub + KVM compatibility | Default to KVM (works for breakpoints, memory reads, register dumps). Document TCG fallback if stepping misbehaves. |
| 3 | File layout | Everything in `qemu/` next to `run.sh`. Per-run artifacts under `qemu/runs/<run-id>/`. |
| 4 | `.gdbinit` strategy | Explicit `source` only. Don't surprise other GDB users. |
| 5 | Output truncation policy | Deferred to implementation. Default to top-N for backtraces, structs at depth 2; `--full` flag to expand. Exact N TBD during Phase 2. |
| 6 | GDB Python helper distribution | `qemu/gdb-helpers/`; auto-sourced by `inspect.sh`. |
| 7 | Rust pretty-printer wiring | Phase 2 task. Need to plumb rust-gdb scripts into our gdbstub workflow. |

## What This Replaces / Doesn't Replace

**Replaces:**
- Manual disasm walks for "where did this PC land?"
- Adding temporary `DbgPrint` calls + rebuild + reboot cycles for inspection.
- `objdump --disassemble-symbols` followed by mental offset arithmetic.
- Repeated `flake_probe.py` runs to repro then re-disasm.

**Doesn't replace:**
- Smoke tests as CI signal of record.
- Static analysis tools (`scan_ud2.py`, `lint_kernel.py`, `verify_constants.py`) — different layer.
- Translation pipeline (rustconv) — debugger helps debug translation output, not generate it.
- KDB itself — it stays useful as the always-available bugcheck fallback.

## Out of Scope

- Step-debugging user-mode processes (csrss, explorer, winlogon).
- Time-travel / record-and-replay.
- Custom debugger UI/TUI.
- Cross-host debugging.
- Strict-JSON-everywhere policy. Output format follows result structure (see Design Principle 7).

## ROI

Per-bug cost is currently 1–4 hours of agent attention. Phase 0–4 is ~6.5 hours of implementation. Even at the conservative end (each bug only halves in cost) payback comes after ~3 bugs. The remaining work — cmdata layers, lpc/connect, both arm64 and amd64 root-cause work, ongoing translation chunks — has more than 3 bugs left.
