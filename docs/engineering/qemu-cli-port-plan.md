# Architecture & Implementation Specification: `vibeos-cli`

**Agentic Build, Smoke, Lifecycle, and Diagnostic Operations Control Plane**

**Status:** Plan, audited against the current tree on May 30, 2026

**Date:** May 30, 2026

**Intended to supersede:** Legacy `qemu/*.sh` suite, `pytest` orchestration wrappers, and `port ps.sh to Python` design iterations. This remains an implementation plan; today only `qemu/ps` is already a Python tool.

---

## 1. Executive Summary

`vibeos-cli` is the planned centralized production operations control plane for VibeOS development. It becomes the single Python entry point for the entire development loop: building the operating system, linting, smoke booting, managing virtual machine lifecycles, executing failure triage, symbolicating memory, capturing screenshots, and providing low-level debugger access.

The primary users of this system are autonomous swarms of coding and debugging agents. While human engineers must be able to read and interpret the output easily, the CLI's default operational contract is optimized for automated callers that require deterministic commands, token-frugal bounded `stdout` summaries, durable artifacts, aggressive preflight validations, and automated next-best debugging actions.

---

## 2. Why This Exists

The legacy smoke testing harness (`tests/qemu/*.py`) and the shell toolkit (`qemu/*.sh`) independently duplicate operational logic. This fragmented approach leads to several core infrastructure failures:

* **Duplicated Resource Logic:** Shell scripts and Python independently implement or omit resource slot pools, process reapers, owner-PID traversals, execution deadline caps, and live log tailing.
* **Shell Fragility:** Critical bugs surface due to shell limitations, such as bash deadline arithmetic `(( deadline - $(date +%s) ))` crashing when evaluated against float parameters passed by the test runner, or multibyte `printf %-Ns` character formatting misaligning status columns.
* **Subprocess Overhead:** Scripts like `reap.sh` rely on heavy `jq` forking (up to eight forks per loop iteration) to process simple structured text data.
* **False-Green Smoke Results:** Inconsistencies between compilation steps and execution layers allow stale build directories and out-of-date binaries to run during smoke tests, reporting false successes.
* **Token Context Waste:** Standard test frameworks dump thousands of lines of raw log data directly to `stdout`, rapidly consuming the limited context windows of automated development agents.

`vibeos-cli` addresses these issues by treating operational tooling as critical QA/SRE infrastructure. The implementation must enforce production-quality engineering standards, modular isolation, and automated validation so environment failures do not break the agentic development loop.

---

## 3. Current-State Audit

This section records what the tree actually contains today, so the implementation plan below does not accidentally rely on false assumptions about shipped behavior.

* **No unified CLI exists yet.** There is no `tests/qemu/cli.py`, no `python3 -m tests.qemu.cli` entry point, and no `vibeos-cli` wrapper in the current tree. The only completed CLI-like replacement is `qemu/ps`, a standalone Python script with `qemu/ps` and `qemu/ps --json`.
* **The planned shared modules do not exist yet.** `tests/qemu/slots.py`, `artifacts.py`, `gdb.py`, `monitor.py`, `runner.py`, and `daemon.py` are not present. Current pytest smoke orchestration still lives mainly in `tests/qemu/conftest.py`, `tests/qemu/vm.py`, `tests/qemu/serial_client.py`, `tests/qemu/qmp_client.py`, and `tests/qemu/debugcon.py`.
* **Resource locking is real, but not factored as described below.** The current shared lock root is `/tmp/vibeos-qemu-locks`, with slot files under `/tmp/vibeos-qemu-locks/slots`. Both `tests/qemu/vm.py` and `qemu/lib/common.sh` use the same slot pool and default cap (`VIBEOS_QEMU_MAX_CONCURRENT`, default 10). This is not `/var/lock/vibeos.slots`.
* **Debug tooling remains shell-first.** `qemu/start.sh`, `wait-for.sh`, `run-until.sh`, `inspect.sh`, `status.sh`, `tail.sh`, `stop.sh`, `addr2sym.sh`, and `sym2addr.sh` are active shell tools. They already provide run IDs, manifests under `qemu/runs/<run-id>/`, GDB helpers, and a 30-minute lifetime cap.
* **Smoke still uses pytest directly.** `tests/qemu/conftest.py` boots one VM per pytest session, waits for the configured boot marker, saves a `ready` snapshot, and restores that snapshot before each test. There is no `--repeat N` cold-boot loop in the harness.
* **Current smoke defaults conflict with the target CLI principle.** Pytest `--arch` defaults to `arm64`. The no-default-arch rule below is a desired `vibeos-cli` property, not current pytest behavior.
* **Current x86/arm64 boot media behavior is mixed.** i386 and amd64 use `build-<arch>/livecd.iso`; arm64 uses `qemu/boot.img` plus `qemu/ramdisk.img`. The current pytest clean path runs `ninja livecd` for i386/amd64 and `ninja ntoskrnl` plus best-effort arm64 input/video targets and `qemu/mkramdisk.sh --refresh` for arm64.
* **The stale Rust cache problem is real and currently under-covered.** Current `_ensure_clean_rust_build()` removes only `build-<arch>/ntoskrnl/rust-kernel-target` and `build-<arch>/ntoskrnl/libkernel_rust.a`. This worktree also has Rust targets and archives for freeldr, rtl, HAL, arm64input, and other consumers, so the proposed dynamic discovery must be load-bearing.
* **No build provenance stamp exists yet.** No `.vibeos-build-stamp.json`, `summary.json`, or `commands.jsonl` artifacts were found in the current checkout. The stamp and artifact contract below are proposed.
* **Current triage is partial.** Pytest captures screenshots on failure, decodes debugcon when available, checks serial for BUGCHECK/STOP in `test_boot_and_serial`, and does limited objdump symbolication. The richer auto-triage pipeline below is a target design.

---

## 4. Design Principles

1. **One Front Door:** A single binary surface (`vibeos-cli {build,check,smoke,debug,ps}`) replaces the scattered shell toolkit and boilerplate test invocations. Shell scripts are retired to compatibility shims and removed only after their Python replacements are proven in real environments.
2. **Agent-First Interface:** Programmatic outputs remain deterministic and stable for automated parsing. Terminal summaries are strictly bounded (~12 lines) to keep agent context consumption minimal.
3. **Production-Quality Infrastructure:** All core modules must adhere to strict type checking, comprehensive linting guidelines, modular separation, and dedicated unit testing.
4. **No Default Architecture:** The target architecture parameter (`--arch {i386|amd64|arm64}`) must be declared explicitly on every `vibeos-cli` execution to prevent silent fallbacks to mismatched platforms. This intentionally changes current pytest behavior, where `--arch` defaults to `arm64`.
5. **Auto-Chaining with Hard-Failing Prerequisites:** Downstream operations automatically trigger upstream validations (e.g., `build` executes code checks before compilation). Conversely, a failure in a prerequisite boundary (e.g., a stale build detected during `smoke`) halts execution immediately.
6. **Explicit Advisory vs. Blocking Boundaries:** Verification lints are clearly categorized. Translation and structural lints are advisory by default during rapid local development loops but hard-fail during full validation checks.
7. **Aggressive Automatic Diagnosis:** When an execution stage fails, the control plane automatically runs low-cost diagnostic routines (such as capturing screenshots, extracting panic tokens, or running stack traces) before exiting.
8. **Token-Frugal Output, Complete Evidence:** Detailed logs and subprocess traces are segregated into structured, timestamped artifact directories. Only compact, high-signal summaries are emitted to `stdout`.
9. **Whole-Tree Linting:** Static validation operates across the entire source tree rather than diff-scoped boundaries, ensuring that breaking changes in shared, cross-architectural components are caught immediately.
10. **Delegate, Don't Reinvent:** The tool leverages established systems for source inventories and build graphs (`git` and `Ninja`), serving as an orchestrator rather than a standalone file tracker.
11. **Global Resource Discipline:** Every path that initializes a virtual machine must utilize the identical `flock`-backed lock pool and background process reaper to guarantee that no rogue hypervisor instances survive.
12. **Preserve the Rust Translation Signal:** Operational policy must never obscure compilation failures or missing symbol indices behind unhandled shell exceptions.

---

## 5. Subcommand API Surface

```bash
vibeos-cli build --arch <i386|amd64|arm64> [--clean] [--no-check]
vibeos-cli check --arch <i386|amd64|arm64> [--strict]

vibeos-cli smoke --arch <i386|amd64|arm64> \
  [--allow-stale] [--no-gdb] [--attach-on-fail] [--screenshot-on-pass] \
  [--repeat N] [--until <marker>] [--fail-on <regex>]

vibeos-cli boot  --arch <i386|amd64|arm64> --until <marker> \
  [--fail-on <regex>] [--json]          # raw fast boot; reuses smoke's EXACT arch-config

vibeos-cli debug start --arch <i386|amd64|arm64> [--gdb-wait] [--allow-stale]

vibeos-cli debug inspect <run-id> [--json]
vibeos-cli debug run-until <run-id> <condition> [--json]
vibeos-cli debug bt <run-id> [--json]
vibeos-cli debug triage-bugcheck <run-id> [--json]
vibeos-cli debug screenshot <run-id> [--format png|ppm] [--out <path>]
vibeos-cli debug profile <run-id> --seconds <N> [--hz <Hz>]
vibeos-cli debug gdb <run-id> -- <raw-gdb-arguments>
vibeos-cli debug wait-for <run-id> <condition> [--json]
vibeos-cli debug tail <run-id> [--lines <N>]
vibeos-cli debug status <run-id> [--json]
vibeos-cli debug stop <run-id>
vibeos-cli debug diff <run-id-a> <run-id-b> [--json]   # normalized serial/debugcon diff

vibeos-cli ps [--json]

```

### Core CLI Roles

* **`build`:** Coordinates the daily compilation loop. It delegates dependency tracking to Ninja, executes binary-level checks, updates the symbol index, and writes the cryptographic provenance stamp upon success.
* **`check`:** Performs static code validation, repository syntax checking, and architecture policy enforcement without triggering a full compilation or updating the provenance stamp.
* **`smoke`:** Validates build freshness, allocates an isolated resource slot, boots the operating system target image, monitors runtime milestones, and yields a clear pass/fail verdict. Invoking `smoke --attach-on-fail` preserves the active VM and its artifacts on failure, returning a unique `run-id` for interactive debugging.
* **`debug start`:** Spawns long-lived, detached target instances intended for investigative steps, manual exploitation, or early-stage kernel initialization analysis.
* **Run-ID Scoped Debug Verbs:** A collection of post-boot inspection commands that attach to a running virtual machine instance using its unique `run-id` to perform non-destructive diagnostics, state inspection, and live control.
* **`ps`:** Dynamically queries the global resource pool, listing active virtual machine processes, assigned slot allocations, owner PIDs, and associated communication ports. The current shipped form is `qemu/ps [--json]`, with no architecture filter.

---

## 6. Output and Artifact Contract

The CLI must strictly separate high-level terminal summaries from detailed log data. The artifact layout in this section is proposed; the current tree does not yet emit these files.

### Standard Output (`stdout`) Layout

On successful execution, the tool prints a single compressed status header:

```text
build i386  —  cold · ninja OK · 5 lint ok · 1 warn   (2m41s)

artifacts: /tmp/vibeos-cli/20260530-141200-build-i386/
full log:  /tmp/vibeos-cli/20260530-141200-build-i386/full.log

```

If an execution stage fails, the tool displays the failure classification along with the critical high-signal lines extracted from the underlying log:

```text
smoke i386  —  FAIL: livelock   (1m48s)

> [00:01:04.12] KERNEL: BugCheck: 0x000000D1 (DRIVER_IRQL_NOT_LESS_OR_EQUAL)
> [00:01:04.15] KERNEL: Failing address: 0x00000020, PC: 0x8010C340 (hal.dll!HalpQuerySystemTime+0x14)

artifacts:   /tmp/vibeos-cli/20260530-142000-smoke-i386/
full log:    /tmp/vibeos-cli/20260530-142000-smoke-i386/full.log  (4,201 lines)
serial:      /tmp/vibeos-cli/20260530-142000-smoke-i386/serial.log
screenshot:  /tmp/vibeos-cli/20260530-142000-smoke-i386/screen.ppm

```

### Durable Artifact Directory Layout

Every command execution generates a unique workspace directory under `${VIBEOS_ARTIFACT_ROOT:-/tmp/vibeos-cli}/<ts>-<command>-<arch>/`. The directory includes the following diagnostic files:

* `summary.json`: High-level execution metrics, stage outcomes, and artifact metadata.
* `full.log`: The raw output from every executed stage, organized under structured headers.
* `commands.jsonl`: A chronological record containing the exact arguments, directory contexts, environment keys, and exit statuses of all spawned subprocesses.
* `serial.log` & `debugcon.bin`: Raw textual and binary output streams extracted from the hypervisor ports.
* `qmp.log`: Chronological JSON commands and responses exchanged with the QEMU Monitor Protocol interface.
* `screen.ppm`: Graphic frame buffer snapshots captured during runtime failures or requested on demand.

#### Illustrative `summary.json` Structure

```json
{
  "run_id": "20260530-142000-smoke-i386",
  "command": "smoke",
  "target_architecture": "i386",
  "exit_status": "FAIL",
  "failure_classification": "LIVELOCK_DETECTED",
  "duration_ms": 108000,
  "git_commit_sha": "a1b2c3d4e5...",
  "failing_component": "hal.dll",
  "highest_signal_frame": "hal.dll!HalpQuerySystemTime+0x14",
  "artifacts": {
    "full_log": "/tmp/vibeos-cli/20260530-142000-smoke-i386/full.log",
    "serial_log": "/tmp/vibeos-cli/20260530-142000-smoke-i386/serial.log",
    "screenshot": "/tmp/vibeos-cli/20260530-142000-smoke-i386/screen.ppm"
  }
}

```

The system returns a non-zero exit code on all hard failures and a zero exit code when operations conclude successfully or emit warnings only.

---

## 7. Staleness and Build Provenance Model

The control plane assumes that a build environment can report inaccurate status indicators across four independent boundaries:

| Layer Boundary | Detection Vector | Blind To |
| --- | --- | --- |
| **Source -> Ninja DAG** | `git log` / `git diff-index` tracking plus `ninja -n` after CMake configure | Untracked modifications outside git unless explicitly included in the fingerprint |
| **Rust Source -> `.a`** | Cryptographic `rust_src_hash` checking | Incremental compilation masking bugs |
| **Cargo Incremental Masking** | Stamp file provenance metadata tracking | Internal compiler cache corruptions |
| **Built Binary $\rightarrow$ Boot Media** | Target `mtime` checking and file hashing | External image packaging script crashes |

Instead of managing an independent filesystem inventory, the tool uses Ninja to evaluate build graphs and Git to track source files. The provenance stamp maps the gaps between these layers.

### The Provenance Token Schema

The build stamp is written atomically to `build-<arch>/.vibeos-build-stamp.json` only after a compilation and linking cycle completes successfully:

```json
{
  "arch": "i386",
  "ts": 1782741600,
  "git_fingerprint": "a1b2c3d4e5...",
  "rust_src_hash": "f6g7h8i9j0...",
  "boot_media_sha": "z1x2c3v4b5...",
  "flags": { "clean": false },
  "provenance": { "cold": true, "ninja_rebuilt_count": 42 }
}

```

### Multi-Tiered Staleness Preflight Flow

Before booting a target image, the execution engine evaluates the environment using a multi-tiered check:

1. **Cryptographic Fingerprinting:** Recompute the current Git fingerprint. If it matches the value stored in the `git_fingerprint` stamp field, the environment is marked **FRESH** and the check terminates.
2. **Per-Layer Verification:** If the fingerprints do not match, the tool runs deeper validations:
* **CMake Context Integrity:** Compares the active worktree path against the internal CMake cache configuration. A mismatch flags the build as **STALE**.
* **Rust Compilation Validation:** Computes the cryptographic hash of all Rust source code folders. If it deviates from `rust_src_hash`, the build is marked **STALE**. If the stamp indicates the last compilation was warm, the tool emits a warning about potential incremental masking. This check must cover every Rust consumer target and copied archive, not only `ntoskrnl`.
* **Binary Modification Analysis (arm64 Re-stage Isolation):** For `arm64` targets, the tool evaluates the modification times (`mtime`) of compiled core binaries against the target `boot.img` asset. If a compiled binary is newer than the packaged image, the target is marked **STALE** (indicating a missed staging step).


3. **Media Availability:** If the boot target asset is missing or its hash does not match `boot_media_sha`, the state maps to **STALE/ABSENT**.
4. **Clean Diff Resolution:** If the Git status contains modifications that do not affect compiled outputs (e.g., changes to documentation), the environment resolves to **NOT STALE (CLEAN-DIFF)** and execution proceeds safely.

### Context Rationale: Why Git Fingerprints Replace Global `mtime` Validation

Git commands like checkout switches or multi-worktree initializations reset file modification times across the filesystem. This makes `mtime` comparisons unreliable for verifying source code freshness across multiple worktrees. Git content fingerprinting avoids this issue entirely. The tool restricts `mtime` checking to build outputs within a single worktree, such as comparing compiled binaries against a local `boot.img`.

### Dynamic Target Discovery & Linkage

The CLI must discover Rust target workspaces and generated static libraries dynamically using directory patterns rather than relying on hardcoded paths. Current evidence shows at least `build-*/ntoskrnl/rust-kernel-target`, `build-*/boot/freeldr/freeldr/rust-freeldr-target`, `build-*/sdk/lib/rtl/rust-target`, `build-*/hal/**/rust-hal-target`, `build-i386/hal/halx86/rust-hal-mini-target`, and `build-arm64/drivers/input/arm64input/rust-arm64input-target`. The clean/staleness implementation must also account for copied outputs such as `libkernel_rust.a`, `libfreeldr_rust.a`, `librtl_rust.a`, `libhal_rust.a`, and `libarm64input_rust.a`.

Following every successful kernel-affecting build, the tool should run the existing `qemu/build-index.sh <arch>` routine so helper tools can resolve addresses during post-failure triage.

---

## 8. Per-Architecture Boot Media and Packaging

Build steps and image assembly follow separate pipelines across different target architectures:

* **`i386` / `amd64` Foundations:** The compilation pipeline targets a single unified goal via `ninja livecd`, producing a bootable ISO image at `build-<arch>/livecd.iso` that mounts using standard `-cdrom` arguments. Dependency tracking is fully managed by the Ninja build graph; compiling underlying components automatically updates the target ISO. However, manually building individual components without invoking the `livecd` target can leave the boot image stale, a condition explicitly monitored by the preflight engine.
* **`arm64` Foundations:** This target does not generate an ISO image. It boots using standalone image files (`qemu/boot.img` and `qemu/ramdisk.img`) located inside the core infrastructure folder. The build process follows a two-step pipeline:
1. Compile separate kernel binaries, HAL targets, and driver packages via Ninja.
2. Invoke the CLI refresh path to execute staging tools, copy updated binaries into the ramdisk archive using `mkramdisk.sh`, and rebuild the final `boot.img`. Current pytest uses `qemu/mkramdisk.sh --refresh` after the arm64 build step.


This multi-step pipeline introduces the **arm64 re-stage trap**, where a driver updates successfully on disk but is not packaged into the ramdisk image, causing smoke tests to silently execute old code. The CLI prevents this by explicitly checking compiled binary modification times and content hashes against the output images.

---

## 9. Build, Check, and Smoke Semantics

* **`vibeos-cli build`:** Runs incremental builds by default. Passing the `--clean` flag removes discovered Rust target outputs and stale copied Rust archives before initiating compilation. Removing whole CMake build directories should be an explicit stronger mode, not the default meaning of `--clean`, because these directories are large and often worktree-local.
* **`vibeos-cli check`:** Executes static analysis tools (`shellcheck`, Python syntax validation, `verify_abi`, `lint_kernel`, trace-catalog checks, and PE-decoration checks where applicable) across the source tree. It does not compile binaries or modify the provenance stamp. Lints should compare against baselines where the tree has known pre-existing findings, so the actionable gate is "no new findings."
* **`vibeos-cli smoke`:** Executes the preflight validation engine. If it encounters a stale or missing image file, execution halts immediately unless bypassed via `--allow-stale`. Upon validation, it locks a resource slot, initializes the hypervisor, evaluates boot milestones, and generates an execution verdict. If the test fails, it triggers the automated triage pipeline.

---

## 10. Execution Lifecycle Models

The system architecture isolates virtual machine operations into two separate execution lifecycles. These paths share a resource allocation layer but use independent runtime management models:

* **In-Process Lifecycle Runner (`QemuVM`):** Currently used by the pytest harness and intended to back future `smoke` commands. The parent execution thread retains direct, blocking control over the QEMU process, serial data channels, and the assigned lock descriptor. This lifecycle is optimized for short-lived, automated testing where immediate resource reclamation is required upon completion.
* **Detached Shell-Launched Debug Runs:** Currently implemented by `qemu/start.sh` and companion scripts. A future `debug start` command can wrap or port this model. The assigned `flock` slot descriptor is passed directly to the QEMU process, and an independent background watchdog process ensures that the virtual machine is cleanly reaped if communication drops or lifetime limits are exceeded.

---

## 11. Shared Architecture

The target package architecture should enforce a strict separation between command-line argument interfaces and shared business logic. These modules are proposed, not present today:

```text
tests/qemu/                 <- Importable package utilized by pytest and CLI
├── __init__.py
├── slots.py                <- Shared flock allocation, port ranges, cleanup reaper
├── artifacts.py            <- Shared artifact layout, metadata writer, raw-log splitters
├── gdb.py                  <- Shared execution engine, command generator, symbol matcher
├── monitor.py              <- Shared hang monitor, CPU analyzer, milestone watchdog
├── runner.py               <- In-process lifecycle runner used by smoke and pytest
└── daemon.py               <- Detached background VM manager used by debug start

```

The future system entry point should be exposed via `python3 -m tests.qemu.cli` and a `vibeos-cli` wrapper. Legacy utilities like `qemu/ps` should be absorbed into the `vibeos-cli ps` command surface after parity exists. Remaining shell scripts should become thin wrappers only after their Python equivalents pass real-world smoke/debug verification.

---

## 12. Global Resource Discipline

To prevent concurrent agent processes or multi-worktree testing tasks from resource starvation or port collisions, all virtual machine operations route through a centralized lock manager.

The allocation engine establishes a predictable coordination pool by utilizing exclusive advisory file locks via `flock` targeting a designated path:

```text
/tmp/vibeos-qemu-locks/slots

```

The slot ID currently controls concurrency only. Future implementations may also derive communication ports from the slot ID:

$$\text{GDB Control Port} = \text{Base GDB Port} + \text{Slot ID}$$

$$\text{QMP Control Port} = \text{Base QMP Port} + \text{Slot ID}$$

The current reaper scans `qemu/runs/*/manifest.json` and `/tmp/vibeos-qemu-locks/*.json`. It kills QEMU processes whose owner/launcher is dead, or whose lifetime cap has expired, while leaving live-owner VMs alone. It does not kill every unmanaged QEMU merely because it lacks a slot descriptor; that broader policy would be too risky across worktrees and manual debug sessions.

---

## 13. Continuous Hang Monitoring

The continuous monitoring engine fills a current gap. Current pytest has passive serial reads, milestone assertions, debugcon reports, and a `process_cpu_pct()` helper, but not the full live classifier described here:

| Hang State Classification | Host CPU Utilization | Serial Communication Line Status | Primary Detection Vector |
| --- | --- | --- | --- |
| **`stage stall`** | Variable | Communication halts completely. | Expected milestone markers (defined in `required_milestones`) fail to emit within specified timeouts. |
| **`deadlock`** | Drops to ~0% | Communication halts completely. | The guest OS logs an idle status while milestone progress remains entirely frozen. |
| **`livelock / spin`** | Climbs to ~100% | Communication halts completely. | High continuous host CPU utilization combined with a total lack of serial output progression. |
| **`looping output`** | Elevated | Emits rapid, repetitive characters. | Sliding-window sequence analysis detects repeating character patterns or infinite error print cycles. |

### Monitoring Escalation Flow

To eliminate performance overhead during healthy boot cycles, the system follows a tiered observation model. It relies on passive serial logging and host metrics during normal operation, only escalating to intrusive checks when a hang condition is flagged:

```
[ Passive Performance Profiling ]
  - Verify milestone tracking metrics
  - Trace sliding-window text patterns
  - Track active host CPU allocation
                |
                +---> Threshold Exceeded (Suspected Stall)
                                |
                                v
                [ Intrusive Diagnostic Escalation ]
                - Pause target execution via QMP interface
                - Attach GDB stub and sample execution registers
                - Resolve active PC pointers to symbol definitions
                - Generate thread stack traces for idle contexts

```

---

## 14. Auto-Triage Policy

When an execution failure or hang condition occurs, the CLI executes an automated diagnostic pipeline to gather evidence before resource cleanup:

1. **Secure Execution Logs:** Capture and isolate active system traces, including raw serial records, unbuffered debug logs, QMP configuration metrics, and the current frame buffer snapshot.
2. **Classify Failure Mode:** Evaluate log signatures to map the failure to a specific category (e.g., build mismatch, launch failure, timeout, or guest kernel panic).
3. **Symbolicate Memory Addresses:** Parse log outputs to extract raw pointer addresses, running them through the local index maps to resolve functions (e.g., `hal.dll!HalpQuerySystemTime+0x14`) without requiring an active debugger connection.
4. **Execute In-Process GDB Queries:** If the target environment is responsive and the GDB stub is accessible, attach the debugger to extract system state:
* **Kernel Panics:** Decode active trap frames, register records, and the faulting thread context.
* **System Hangs:** Sample program counters across short intervals to identify spinning loops, and walk internal thread structures to locate deadlocked resources.
* **Execution Stalls:** Match program counter samples against milestone records to pinpoint the block.


5. **Fallback Diagnostics:** If the virtual machine is responsive but the GDB connection is unavailable, use QMP commands to dump core registers, capture active status codes, and save frame buffer images.
6. **Emit Diagnostic Summary:** Generate a compact summary block on `stdout` containing the failure classification, probable faulting component, and high-signal stack traces, while writing detailed data logs to the artifact workspace.

### Safe Degradation Behavior

If the triage subsystem encounters errors during execution, it degrades gracefully to protect existing evidence:

* If GDB commands timeout, the engine skips active memory inspection and continues processing text logs.
* If a QMP frame buffer capture fails, the tool logs the interface exception and proceeds with log packaging.
* If symbol files are missing or out of date, the system notes the mismatch and prints raw program counter addresses rather than halting the triage pipeline.

---

## 15. Debug Integration

The `smoke` runner and the `debug` management tool operate as sibling interfaces that share a common diagnostic backend rather than forming a parent-child relationship:

```
+-------------------------------------------------------------+
|                     Shared Inspection Engine                |
|                    (gdb.py / Port-Keyed Communication)      |
+-------------------------------------------------------------+
                               ^
                               |
               +---------------+---------------+
               |                               |
               v                               v
  +--------------------------+    +--------------------------+
  |    smoke Execution       |    |    debug Execution       |
  |  - Automated Verdicts    |    |  - Long-Lived Instances  |
  |  - Auto-Triage on Failure|    |  - Scripted Control      |
  +--------------------------+    +--------------------------+

```

When a smoke test fails, invoking `--attach-on-fail` bridges execution to the debug tools by preserving the active virtual machine instance and returning its unique `run-id`. Debug commands can then attach directly to the open communication ports using that ID:

```bash
vibeos-cli debug inspect 20260530-142000-smoke-i386

```

If long-lived background testing paths (`debug start`) prove unnecessary over time, the system can later collapse into purely run-scoped verbs. However, the background daemon model is preserved for now to support early-stage kernel initialization analysis and arm64 step-from-zero debugging.

---

## 16. Engineering Quality Gates

The control plane maintains strict engineering quality gates across both local and integration testing environments:

* **Type Signature Enforcement:** New shared CLI modules should pass strict type-checking validation. The current `tests/qemu/` tree is typed in places but is not yet backed by an enforced strict-mode type gate.
* **Format & Style Conformity:** Python source code must comply with repository linting guidelines and PEP 8 structural map layouts.
* **Comprehensive Unit Testing:** Code updates require unit tests covering key behaviors, including cryptographic fingerprint calculations, atomic stamp updates, sliding-window text loop detection, and triage degradation logic.
* **Subprocess Mock Validation:** The tool includes integration tests that mock external subprocess calls (`ninja`, `cmake`, `git`), allowing option routing, path formatting, and environment mapping to be verified in environments that lack hypervisor access.
* **Hypervisor Integration Testing:** Major updates require at least one successful end-to-end boot block per architecture before corresponding legacy shell tools can be deprecated.
* **Resource Contention Testing:** The allocation engine undergoes automated stress testing to verify the stability of file locking, slot reclamation, and orphan process cleanup under heavy concurrent loads.
* **Artifact Completeness Validation:** Test suites ensure that summary logs accurately link to existing workspace assets and that raw subprocess dumps are cleanly organized within log structures.

---

## 17. Migration Plan

Migration follows a strict infrastructure safety rule: **Do not break the debugger to make it prettier.** Legacy shell scripts are converted into thin compatibility wrappers first and are removed only after their Python replacements are proven in production.

* **Phase 0 (Partially Complete):** `qemu/ps` is implemented in Python, and the current pytest and shell debug paths share `/tmp/vibeos-qemu-locks/slots` plus compatible lockfiles. The proposed `slots.py` / `artifacts.py` factoring is not complete.
* **Phase 1 (P0 Priority) - Core Build & Staleness Engines:** Implement `build` and `check` command surfaces, integrate dynamic path discovery, and enforce atomic provenance stamp checks.
* **Phase 2 (P1 Priority) - Media Alignment & Target Packaging:** Integrate architecture-specific packaging loops and implement `arm64` image staging verification to resolve the re-stage trap.
* **Phase 3 (P2 Priority) - Smoke Engine Integration:** Migrate virtual machine orchestration from internal `pytest` logic into the unified Python runner framework, updating `pytest` to read structured execution verdicts.
* **Phase 4 (P3 Priority) - Continuous Monitoring Deployment:** Activate the multi-channel hang monitor to replace the legacy post-execution CPU utilization check.
* **Phase 5 (P4 Priority) - Debug Surface Parity:** Finalize background daemon management, implement run-ID tracking verbs, and deprecate the remaining legacy shell scripts.

---

## 18. Acceptance Criteria

1. **Type Safety:** The entire `tests/qemu/` code tree passes type checking in strict mode without errors or omissions.
2. **Unified Surface:** The single `vibeos-cli` interface successfully coordinates all build, check, smoke, debug, and ps tasks, with legacy `qemu/*.sh` tools either wrapped or removed.
3. **Cross-Architecture Stability:** Automated test executions pass successfully across all three target configurations (`i386`, `amd64`, `arm64`).
4. **Staleness Prevention:** The preflight validation engine successfully blocks execution when out-of-date media is detected—including unbuilt ISO layouts and missing arm64 driver staging actions—before hypervisor initialization.
5. **Automated Triage Performance:** On test failures, the diagnostic system automatically parses raw log streams, identifies fault contexts, and maps memory locations to valid function symbols without manual intervention.
6. **Robust Interactive Debugging:** The debug subcommand surface supports frame-buffer captures, raw GDB pass-through options, structured trace evaluations, and active process status metrics using individual run-ID contexts.
7. **Token Efficiency:** All commands output concise terminal summaries to preserve agent context windows while writing complete execution artifacts to a standardized directory tree.
8. **Resource Pool Reliability:** Every path that creates a virtual machine complies with advisory lock pool allocations, orphan process cleanup rules, and execution timeout parameters.
9. **Infrastructure Test Validation:** Code signature linter checks, unit tests, mock-based integration workflows, and real hypervisor executions pass completely before code integration.
10. **Documentation Integration:** Technical reference manuals (`qemu/README.md`, `kernel-debugging-infra.md`, `AGENTS.md`) are fully updated to match the unified command plane.

---

## 19. Feedback Summary

Ranked themes from the worktree feedback sections below. The detailed feedback is intentionally left as field notes; this section consolidates the implementation priorities.

1. **Stale-green prevention is the top priority.** Every worktree reports false confidence from stale boot media, stale copied `.a` files, or Cargo incremental state. The staleness model must cover all Rust target dirs, all copied `lib*_rust.a` outputs, per-module boot-media contents, CMake configure status, and warm-vs-cold provenance.
2. **Build -> media -> boot must become one trustworthy chain.** The CLI must either rebuild/re-stage the relevant boot media or refuse to run. This applies to arm64 ramdisk staging, x86 `livecd.iso` refreshes, per-DLL grafting, and component-only rebuilds that do not update boot media.
3. **Repeated independent cold boots are required for race-sensitive verdicts.** A single pytest session boot plus snapshot restores cannot characterize intermittent `0xF4` / user-mode-spine races. Add `smoke --repeat N` / `--runs N` with per-run classifications and pass-rate summaries.
4. **Structured artifacts are a core contract, not decoration.** Agents need a durable `summary.json`, exact command logs, serial/debugcon artifacts, screenshots, and stable exit codes. Human-readable stdout should summarize, not be the only source of truth.
5. **Keep the shell debug toolkit during the transition.** Feedback consistently favors wrapping `qemu/start.sh`, `wait-for.sh`, `run-until.sh`, `inspect.sh`, and related symbol tools first. Port or retire shell tools only after the Python path proves parity on real failures.
6. **Build and link failures need first-class triage.** The plan must classify `CMAKE_CONFIGURE_FAIL`, `CARGO_CHECK_FAIL`, `LINK_FAIL`, and boot failures separately. i386 PE decoration (`_Foo`, `_Foo@N`, `@Foo@N`) should be decoded into likely ABI/root-cause hints.
7. **Runtime triage must cover user mode, not only kernel bugchecks.** Worktrees need `RUST_PANIC`, user-mode image symbolication (`ntdll.dll`, `csrss.exe`, `winsrv.dll`, win32k paths), fail-fast regexes, known-flake tagging, and milestone coverage beyond kernel init.
8. **Shared architecture config must be the only source of QEMU truth.** Raw boot, smoke, debug, and matrix paths must reuse identical per-arch machine type, KVM/TCG flags, media, timeout, serial, QMP, and debugcon configuration to avoid q35-vs-pc style misdiagnosis.
9. **Multi-arch and differential workflows should be built in.** Common workflows are all-arch matrices, same-arch repeats, arch-to-arch serial/debugcon diffs, and `--ref <git-ref>` confound checks for CGU-shift or pre-existing base-branch failures.
10. **Worktree ergonomics matter.** The CLI should locate the repo root from cwd, detect CMake caches configured for another worktree, avoid assuming a local `.venv`, and avoid forcing harness updates through kernel rebuild churn where possible.

---

## 20. Refinement Pass — Skeptical Review, Gap Closure, Rollout Priority

*(2026-05-30, after the five worktree feedback sections. §19 lists the feedback;
this section reconciles it with the body, pushes back where an ask is risky or
over-scoped, and sequences the work so a trustworthy green smoke ships before the
long tail. These deltas augment the referenced sections.)*

### 20.1 Where the body still lags the feedback (concrete deltas)

| Theme | Worktrees | Body status | Concrete delta | Pri |
| --- | --- | --- | --- | --- |
| **All-rust-target staleness** | all 5 | §6 has dynamic discovery but one coarse `rust_src_hash` | **Per-target** hash of *every* discovered `rust-*target` (incl. `build-*/dll/**`, win32k, csrss, freeldr, rtl, hal) **and every** copied `lib*_rust.a`; `--clean` enumerates + **prints the count**; treat a crate dirty by **content hash of its src tree** (independent of cmake's hand-kept `DEPENDS`); a stale sibling `.a` is *flagged*, not trusted | **P0** |
| **Per-contributing-binary media provenance** | ntdll #1, rtl #2 | §6 hashes whole `boot_media_sha` only | Stamp + verify the content hash of each *binary extracted from the media* (`ntoskrnl.exe`, `hal.dll`, freeldr, the cohab DLL); warn *"media contains a warm-cached `freeldr_pe.exe`; cold hash differs."* A whole-ISO hash passes even when a graft silently no-op'd | **P0** |
| **Independent repeat boots** | smss #2, ntdll #7, win32k #6 | absent from §9 | `smoke --repeat N` = **N independent boots** (fresh QEMU each, **not** savevm/loadvm restores), flake rate in summary (`i386: 2/3 — FLAKY`) | **P0** |
| **Fail-fast marker** | ntdll #6 | absent | `--fail-on <regex>` aborts at the first hard-failure marker (`RUST PANIC HIT`, `BugCheck`) instead of burning the full timeout (5 min → ~15 s) | **P0** |
| **Build/link failure taxonomy** | win32k #3/#4, freeldr #4 | §14 is runtime/bugcheck-only | Classify `CMAKE_CONFIGURE_FAIL` / `CARGO_CHECK_FAIL` / `LINK_FAIL` distinctly; surface undefined symbol(s) in the 12-line summary; **i386 PE-decoration-aware** hint (`_Foo`/`_Foo@N`/`@Foo@N` → cdecl/stdcall/fastcall or missing `#[no_mangle]`) | **P0** |
| **Raw boot shares arch-config** | smss #3 | no raw verb (now added to §5) | `boot --raw` reuses smoke's *exact* machine/KVM/vga/media/debugcon config, writes artifacts, auto-decodes the debugcon catalog (kills the q35-vs-pc misdiagnosis class) | **P1** |
| **User-mode + RUST_PANIC triage** | ntdll #5, smss | §14 kernel-centric | `RUST_PANIC` class (grep banner → `file:line` into `summary.json`); symbolicate against user-mode image bases (`ntdll.dll`, `csrss.exe`, `winsrv.dll`), not just the kernel | **P1** |
| **Known-flake catalog** | ntdll #8, smss #5, win32k #6 | absent | Match signature (marker + bugcheck + frame) → `KNOWN_FLAKE` vs `NEW_REGRESSION`; seed with the shared `[EXPLORER] _tWinMain` `0xF4` race all three hit | **P1** |
| **Lint baseline/diff** | win32k #5 | §9 says it, §7/§8 contradict | Reconciled in 20.3: gate = **0 NEW vs checked-in baseline**, not absolute zero | **P1** |
| **User-mode milestone localization** | win32k #8 | partial (`vm.py` already carries SMSS/CSRSS/WINLOGON/EXPLORER markers) | Hang monitor + debugcon pin a pre-`shell ready` hang to a *stage* (win32k-init vs explorer), not a binary pass/fail | **P1** |
| **Worktree ergonomics** | rtl #3, smss #6, ntdll | partial (§6 CMake-context *detect*) | Locate repo root from cwd; don't assume a local `.venv`; on "CMake cache configured for another worktree," **offer to re-point/re-create** incl. `.build-tools` + `.venv` symlinks | **P1** |
| **`debug diff`** | smss #4 | now in §5 | Normalize hex/PIDs, surface arch-unique serial/debugcon lines between two runs | **P2** |
| **Decoupled media graft** | ntdll #2 | absent | Graft an already-built module into existing media without re-driving a broken sibling target — see tension 20.2.a | **P2** |
| **`--ref` confound build** | smss #5, win32k #2 | absent | Boot a base git-ref to separate "my bug" from a cgu-shift — scoped down in 20.2.b | **P2** |
| **`--verify-stamp <sha>`** | rtl #4 | partial (stamp exists) | Fail if artifacts don't match the commit you think you built (defends against stale/phantom stdout) | **P2** |
| **Workspace-membership → force-cold** | win32k #2 | absent | A `Cargo.toml` / `extern crate` diff repartitions cgu=256 and can regress an *untouched* arch → treat as a force-cold-rebuild trigger; run the ud2 scan on all 3 binaries vs a checked-in allowlist | **P1** |

### 20.2 Skeptical pushback / open tensions

a. **Decoupled media graft vs principle #10 ("delegate to ninja").** ntdll #2's hand-`mkisofs`+`isohybrid` is real but fragile (its own author says so). Prefer making the `livecd` graph robust to a broken sibling (or building just the needed subtarget) over a bespoke graft path. **Detection (per-binary hash, 20.1) is cheap and already catches the false-green; the graft *fix* is the expensive, fragile part — ship detection first, graft behind a flag and maybe never.**

b. **`--ref` is the heaviest ask.** A from-scratch second-ref build needs a second build dir and ~full build time (minutes–tens). The existing worktree workflow already gives base-branch comparison. **Scope `--ref` to "boot a *previously stamped* artifact set," not checkout+build-from-scratch**; full confound builds stay a manual worktree dance. It must not block P1.

c. **Scope creep is the real risk for *this* plan.** The feedback is excellent and consistent, which makes it tempting to absorb all of it into the core — and then the trust-restoring spine never ships. The cardinal failure here is "comprehensive doc, nothing built." P1 (20.4) is deliberately narrow.

d. **Per-module media extraction has a cost** (extract+hash on every smoke). Key it off the cheap whole-media `sha` first — only re-extract per-binary when the media hash changed.

e. **The body has prose bloat** ("centralized production operations control plane," "cryptographic provenance token"). Not blocking, but the *body* (§5–§18) should remain the single implementable source of truth; the 400 lines of field notes are appendix, and an implementer shouldn't have to read them to build P1.

### 20.3 Reconciled lint-gate policy (supersedes the §7/§8/§9 ambiguity)

- Gate = **zero NEW findings vs a checked-in baseline** (`tools/lint-baseline/`), never absolute zero — the tree carries known pre-existing findings (e.g. `verify_abi`'s 65 dead-cfg mismatches) that would otherwise bury the one new one and train agents to ignore a perpetually-red lint (win32k #5).
- `build`'s auto-`check`: **advisory** (warn, never blocks the build).
- `check` (default): reports NEW vs baseline; nonzero exit only on NEW.
- `check --strict`: blocks on NEW (the pre-commit / CI gate).
- Binary lints (`scan_ud2`, sacred-primitives) run against **cold** artifacts — a warm `.a` passes ud2 while the cold link would fail (rtl-priv).

### 20.4 Re-prioritized rollout (trust first, tail later)

Re-orders §17 around the one thing every worktree ranked #1: *can I trust a green smoke?*

- **P1 — Trust core (ship before anything else).** Complete staleness — all-rust-target + all-`lib*_rust.a` **per-target** hashing, content-hash-over-`DEPENDS`, per-contributing-binary media provenance, CMake-context check, configure-fail-loud, workspace-membership force-cold. Plus the cheap velocity verbs that need no new infra: `smoke --repeat N`, `--fail-on`, `--until`+JSON, and `LINK_FAIL`/`CONFIGURE_FAIL` classification with i386 decoration hints. Repo-root-from-cwd. This is the daily driver and it directly kills the false-green class.
- **P2 — Triage depth.** `gdb.py` engine + auto-triage, `RUST_PANIC` + user-mode symbolication, known-flake catalog, user-mode milestone localization, the `debug` group.
- **P3 — Hang monitor** (continuous classifier).
- **P4 — Heavy / optional, behind flags.** `boot --raw` config-share (or fold into `smoke --until`), `debug diff`, `--verify-stamp`, decoupled media graft (fragile fallback), `--ref` (scoped to stamped-artifact compare). Build only if P1–P3 prove insufficient.

This pulls staleness completeness and the cheap verbs together into P1 — matching rtl-priv's "prioritize staleness over monitoring" and the universal #1 ranking — and demotes the genuinely heavy/risky items so they can't stall the spine.

---

## Appendix: Implementation Insights & Field Notes

### 1. The arm64 Re-stage Trap is Real

On x86 platforms, executing `ninja livecd` automatically evaluates the dependency graph and re-rolls the target ISO image. On arm64, however, the loop requires compiling binaries with Ninja and then manually running `qemu/mkramdisk.sh` to stage outputs inside the boot images.

If this staging step is skipped, smoke tests can silently boot old binaries and report a false success. The multi-tiered preflight model addresses this by explicitly checking compiled binary modification times against the output images. A similar issue occurs on x86 when individual driver binaries are compiled but the parent ISO image is not re-rolled.

### 2. Live Tailing via Polling Logs is Awful

The legacy workflow often relies on manual terminal commands to poll log files using grep loops (e.g., `until grep -q "passed\|failed" /tmp/smoke_x.log; do sleep 10; done`). This approach is fragile because testing frameworks buffer text streams, log outputs can be truncated mid-run, and deadlocked QEMU instances can cause polling loops to run indefinitely. Replacing this with an active Python monitor that exposes structured JSON status metrics provides a predictable API for automated tools.

### 3. Async/Background Multi-Arch Checks are Too Hard Today

Because an amd64 smoke test can take several minutes to conclude, engineers and agents frequently background the process manually. This makes tracking execution status awkward and prone to race conditions when reading logs. Implementing a structured status verb like `vibeos-cli debug status --json` allows status reporting to be queried reliably via run IDs, removing the need for ad-hoc terminal loops.

### 4. CMake Configure Failure is Invisible in the Old Smoke Result

If a CMake modification encounters an evaluation error, the configuration step fails. However, because the existing build folder still contains compiled binaries from previous runs, downstream Ninja commands can report a false success and execute out-of-date assets. `vibeos-cli build` addresses this by forcing hard failures on all configuration errors, blocking subsequent smoke tasks from executing stale media.

### 5. Architecture Command Template Bloat

Different targets require specific operational flags and media conventions: i386/amd64 use ISO boot media with x86 QEMU machine settings, while arm64 uses UEFI firmware, pflash variables, virtio boot media, and ramdisk staging. The control plane hides these low-level configuration complexities behind the uniform `--arch` flag, managing timeouts, storage formats, firmware paths, and hypervisor arguments internally.

### 6. Parallel Verification

A future implementation of a multi-target smoke execution command (e.g., `vibeos-cli smoke --all --json`) will allow a single call to return a comprehensive per-architecture success matrix. This expansion will replace sequential test runs and significantly accelerate integration pipelines.

### 7. Answers to Open Questions

* **Wrap or Replace `qemu/*.sh`?** Wrap first. The existing debugging toolkit is highly effective for active kernel development. Retaining these scripts as thin wrappers ensures they remain available during the infrastructure transition, with individual scripts removed only after their Python equivalents pass real-world verification.
* **Whole-Tree vs. Diff-Scoped Linting?** Whole-tree. Diff-scoped analysis fails to catch breaking changes that impact shared, cross-architectural component files.
* **Where Do Translation Lints Belong?** Keep translation lints inside the `check` subcommand surface as blocking quality gates. During local `build` iterations, these checks are advisory by default to keep local development loops fast, unless strict mode is requested.
* **Inline ASM Assertions:** The static checking suite includes a custom validation step that flags errors if inline-assembly markers or sacred primitive layouts deviate from expected architectural patterns, catching bugs before the compilation phase.

---

<!-- freeldr-worktree-painpoints -->
## Pain points from worktree `vibeos-wt-freeldr`

Context: kill-BIOS + pre-Vista C prune (BIOS prune stages 1–5, driver prune E1–E5).
Ran the full 3-arch build+smoke loop ~6× in one session, heavy `--skip-clean-build`
iteration. Recurring frictions and what I'd want the CLI to solve:

### 1. The build → media → smoke chain is 3 disjoint manual steps (arm64 footgun)
x86 loop is `ninja livecd` → pytest. arm64 is `ninja` → **`bash qemu/mkramdisk.sh`** →
pytest, and if you forget the mkramdisk step the smoke silently boots the OLD binaries
and reports green. Burned a cycle this session trusting a stale arm64 "pass". The x86
`livecd.iso` has the same trap: `ninja hal`/`ntoskrnl` rebuilds the .dll but not the iso.
**Want:** `smoke --arch arm64` should depend-check build artifacts and re-roll the
ramdisk/iso itself, or refuse + warn when the media is older than the build output. A
`--build` flag that does `ninja <target> && <roll media> && boot` as one verb would kill
this whole class of stale-artifact false-greens.

### 2. No single "boot to marker, tell me what happened" verb
I kept hand-writing `until grep -q "passed\|failed" /tmp/smoke.log; do sleep 10; done`
and it's fragile (pytest buffers, log is one physical mid-run line, polls return partial
output). `boot --until "shell ready" --json` is exactly right — please make the JSON
`{reached, marker, seconds, last_serial_tail, debugcon_triage}` so there's nothing to scrape.

### 3. Backgrounding long smokes fights the harness sleep-blocker
amd64 smoke is ~5 min; backgrounding it then waiting is awkward — chained `sleep` is
blocked by the harness and polling per-arch logs races (I read a log before it existed).
A first-class blocking `--wait` mode with a clean exit code, or a `status --json` verb
reporting `{running, done, result}` per arch, would remove all my ad-hoc `until` loops.

### 4. cmake-configure failure is invisible in the smoke result (bit me hard)
A CMakeLists edit silently failed and a follow-up `git rm` left dangling
`add_subdirectory` refs — **all 3 cmake configures errored**. But under
`--skip-clean-build` the smoke booted the *stale* ISO and printed "8 passed". Nothing
shipped (caught it from buffered output) but the harness actively hid a broken build
graph behind a green smoke. **Want:** `smoke`/`boot` must (a) run/verify cmake configure
and FAIL LOUD if it errored, and (b) assert boot media is newer than build inputs.
Stale-binary green is strictly worse than red — it defeats "smoke is the single source of truth".

### 5. Per-arch invocation is copy-paste with subtle differences
i386-uefi wants `--timeout 400`, amd64-uefi `--timeout 500` (cold-KVM is slower), arm64
is ramdisk-not-iso and uses idle-detection not `shell ready`. Three near-identical
templates I occasionally cross. One CLI with shared arch-config (timeouts, media type,
OVMF paths, KVM flags, marker logic all internal) so I just say `--arch <x>` — big win,
same consolidation `vm.py` already wants.

### 6. (Nice-to-have) a batch/matrix verb
~90% of my runs are "all 3 arches, did anything regress?". `smoke --arch all --json`
returning a per-arch pass/fail matrix in one call replaces the 3 sequential invocations
+ manual tallying I did at every prune stage.

### Answers to the open questions (heavy-consumer view)
- **Wrap or replace `qemu/*.sh`?** Wrap first. The `start.sh`/`run-until.sh`/`inspect.sh`
  kernel-debug toolkit is genuinely good; layer `smoke`/`boot` on top so triage (`bt`,
  `triage-bugcheck`) is reachable from the same CLI. Replace the smoke-side `vm.py`/`conftest`
  bespoke bits, keep the debug toolkit.
- **Python or Rust?** Python — that's where the harness + trace-catalog logic lives; the
  value is consolidation, not a rewrite.
- **Trace-catalog sync:** add `trace-catalog --check` that fails when inline-asm markers in
  kernel source drift from the JSON (mirror the `verify_sizes.py --check` pattern).

---

<!-- rtl-priv-worktree-painpoints -->
## Pain points from worktree `vibeos-wt-rtl-priv`

Context: rtl-strings C→Rust cohabitation (core_rs + cabi NTAPI shims), feature-gated
alloc. Session = merging crp freeldr fixes + chasing an i386 cold-build 0x6B regression.
Heavy warm/clean rebuild churn across a worktree with a repointed build dir.

### 1. Stale cargo/.a cache produced false-green AND hid a real compile bug
The headline failure of my session. A latent E0308 (`InitializeObjectAttributes` signature
mismatch) was sitting in BOTH branches, invisible because the incremental cache never
recompiled the affected CGU. It only surfaced on a true clean build. Conversely, the smoke
harness nukes *one* rust target (`rust-kernel-target`) but there are ~5 `rust-*-target`
dirs (ntoskrnl, freeldr, rtl, hal, csrss) — so "clean build, green smoke" is a lie unless
*all* of them are nuked. I burned a cycle trusting a warm-built freeldr binary.
**Want:** the provenance `rust_src_hash` (§6) must hash + gate **every** discovered
`build-*/**/rust-target` dir, and `build --clean` must enumerate them dynamically (§6
"Dynamic Target Discovery" — please make this load-bearing, not best-effort). A
`--cold-verify` that asserts a from-scratch rust link before declaring green would have
caught both bugs.

### 2. The freeldr warm-green/cold-red trap, concretely
crp's i386 smoke is green on a **warm** freeldr_pe.exe (425,984 bytes). The same source
clean-builds to 437,248 bytes (+11 KB) and bugchecks 0x6B at PsLocateSystemDll. The
harness can't see this: it doesn't rebuild freeldr from cold and doesn't fingerprint the
loader binary into the boot media stamp. **Want:** §6's `boot_media_sha` should also stamp
the *contributing binaries* (freeldr_pe.exe, ntoskrnl.exe, hal.dll) by content hash, and
preflight should warn "boot media contains a warm-cached freeldr; cold hash differs" — the
single highest-value staleness check for this codebase.

### 3. Worktree build-dir repoint is manual and silently half-breaks the toolchain
`build-*` is gitignored (lives only in main), so a worktree has no build dir. The recipe is
copy-warm-dir + `sed` the absolute paths in CMakeCache.txt + clear nested host-tools +
`cmake .`. I hit `llvm-strip: not found` because the repoint rewrote the toolchain path to
the worktree, which lacked `.build-tools` — fixed by hand-symlinking it. **Want:** a
`vibeos-cli build` that detects "build dir belongs to a different worktree path" (it already
plans CMake-cache-vs-worktree comparison in §6) and **offers to re-point or re-create it
correctly**, including the `.build-tools`/`.venv` symlinks, instead of failing 20 minutes
into a build with a missing-tool error.

### 4. Output-stream corruption makes "it said success" worthless
The single most expensive failure mode this session: scrambled / stale / fabricated stdout
led me to (a) make an edit that silently no-op'd against hallucinated text, (b) report a
smoke pass that never ran, (c) commit a half-fix that wouldn't compile. I now route
anything load-bearing through write-to-/tmp-then-Read-tool, base64, and single-tool-call
discipline, and trust only git/file ground truth. **Want:** this is exactly why §5's
"durable artifact + summary.json + non-zero exit code" contract matters — a verdict I can
re-read from a file with an exit code beats any prose. Please make the smoke verdict a
`summary.json` field an agent re-reads, never a line it has to scrape from a stream that
might be stale. A `--verify-stamp <expected-sha>` that fails if the artifacts don't match
the commit I think I built would let me detect a stale/phantom run mechanically.

### 5. 0x6B told me "broken" but not "where" — manual symbolication every time
The bugcheck localized to PsLocateSystemDll / `\ArcName\<boot>` not found
(0xc0000034/0xc000003a) only after I hand-symbolicated. §13 auto-triage with
`highest_signal_frame` in summary.json is exactly right; for boot-path failures please also
surface the **last debugcon trace marker** + decoded catalog stage, since the freeldr→kernel
ARC handoff dies before serial is fully up and the byte-marker is the only signal.

### 6. Merging crp repeatedly just to get harness fixes is churn
I merged continue-rust-port twice this session purely to pull smoke-harness improvements
(the reaper). When the harness lives in the same tree as the code under test, every harness
fix is a merge + potential rebuild. Not a CLI feature request per se, but: keeping
`tests/qemu/` import-stable (§10) so harness updates don't force a kernel rebuild would cut
this.

### Answers to the open questions (rtl/cohab-consumer view)
- **Wrap or replace?** Wrap — but the *staleness engine* (§6) is the part I'd want first and
  strongest. For this codebase, cache-masking is a more frequent false-green source than
  orphan QEMU or column misalignment. Prioritize Phase 1 staleness over Phase 4 monitoring.
- **Where do translation lints belong?** `check`, blocking — but `verify_abi`/`scan_ud2`
  must run against the **cold** artifacts, not warm ones, or they inherit the same masking
  (a stale .a passes ud2 scan while the cold link would fail).
- **One thing to add:** `build --clean` must nuke **all** `rust-*-target` dirs it discovers,
  and print the count. Silent partial-clean is how a "clean build" stayed dirty all session.

---

<!-- smss-csrss-worktree-painpoints -->
## Pain points from worktree `vibeos-wt-smss-csrss`

Context: user-mode subsystem-spine translation (smss/csrss/csrsrv/basesrv/winsrv
C→Rust), cohabitation flips, and heavy intermittent-race + arch-only bugcheck
debugging (0xF4 thread-hash race, arm64 0xc0000005 SD-length, amd64 0x6B
cdrom/rtl-strings). Ran the 3-arch build+smoke loop many times plus dozens of
hand-rolled raw QEMU boots.

### 1. Stale .a across the MULTI-staticlib user-mode spine (conftest nukes only the kernel one)
+1 to freeldr's stale-.a pain, but worse for user-mode: there isn't one rust
staticlib, there are 5+ per arch (libkernel_rust.a, libsmss_csrss_rust.a,
libwinsrv_rust.a, rtl, hal…). A `constants.rs` edit in ntdll-sys/ntkrnl-sys
doesn't retrigger the csrsrv/winsrv cargo build (cmake DEPENDS doesn't track it),
and the smoke conftest only nukes `rust-kernel-target/` — so a real source change
runs a stale `libsmss_csrss_rust.a` and reports green. I manually `rm -f
build-<arch>/subsystems/csr/csrsrv/libsmss_csrss_rust.a` + `find … -name
'*smss_csrss_rust*' -delete` every iteration. **Want:** §6 `rust_src_hash` +
"dynamic target discovery" MUST enumerate ALL `build-*/**/rust-*target` dirs and
ALL `lib*_rust.a` outputs, not just the kernel's. If any rust source in the
workspace changed, every dependent staticlib is stale — stamp them all, nuke them
all under `--clean`. (Memory: conftest only nukes 1 of 5 rust-*-target.)

### 2. n=1 boot hides intermittent races — I need `--runs N` independent COLD boots
My nastiest class this session was timing-dependent (0xF4 thread-hash race, the
`_tWinMain` suspended-thread flake): **byte-identical binaries boot green one run
and bugcheck the next.** The harness boots ONCE per session then snapshot-restores
per test — so it reports n=1, and a green run proves almost nothing for a
Heisenbug. I hand-looped raw QEMU ≥3× to trust any result. **Want:** `smoke
--runs N` doing N *independent cold* boots (not savevm/loadvm restores), reporting
`{green: k/N, classifications: […]}`. For intermittent bugs the flake RATE is the
signal; one boot is a coin flip presented as a verdict.

### 3. Raw-boot machine config MUST match the harness exactly (q35-vs-pc cost me a misdiagnosis)
For fast iteration I bypass pytest and hand-roll `qemu-system-x86_64 -M pc -accel
kvm … -serial file: -debugcon file:`. This session I accidentally used `-M q35`
and got a totally misleading trace (q35's AHCI controller uniata can't bind → "no
storage" red herring) before realizing amd64 smoke uses `-M pc`. A whole
diagnostic branch wasted on a config divergence. **Want:** `boot --raw --arch <x>
--until <marker>` that reuses the EXACT internal arch-config as `smoke` (machine
type, KVM flags, vga, timeouts, media), writes serial+debugcon to artifacts,
auto-decodes the debugcon trace catalog, and greps the marker. The "arch-config is
internal" principle (§3.4 / Field-Note 5) would have structurally prevented this —
make the *fast raw path* go through the same config, not just the pytest path.

### 4. Multi-arch DIFFERENTIAL diffing is the core technique for arch-only bugs — all manual
Every amd64-only / arm64-only bug I closed (0x6B, SD-length 0xc0000005) was solved
by capturing two arches' traces and diffing them — e.g. for 0x6B I normalized
WDFTrace lines (`sed 's/hex/X/'`) and `sort|uniq -c`'d each arch, then eyeballed
which lines were amd64-unique. That differential *is* the diagnosis, and I rebuilt
it by hand each time. **Want:** beyond freeldr's `--arch all` matrix, a `debug diff
<run-a> <run-b>` that normalizes (mask hex/PIDs) and diffs serial+debugcon between
two runs, surfacing arch-unique lines. "What does the broken arch print that the
green arch doesn't" should be one command.

### 5. CGU-shift confound: a red smoke after adding a workspace member may not be MY bug
Adding a crate repartitions the cgu=256 firewall and can surface a latent
`ud2`/captures(none) on live boot that has nothing to do with my change (the wave-2
"false-RED" confound). So a smoke regression needs a confound check against
base-branch HEAD before I trust it's mine. **Want:** a cheap `smoke --ref <git-ref>`
that builds/boots the committed base ref into a scratch artifact dir, so "did my
change cause this, or is it a pre-existing cgu-shift?" is one call instead of a
manual worktree dance.

### 6. venv-in-worktree friction
The worktree has no `.venv`; I run the *main repo's* `…/vibeos/.venv/bin/python`
from the worktree cwd and rely on conftest resolving worktree paths via `__file__`.
Works, but it's a sharp edge (easy to run the wrong tree's harness). A `vibeos-cli`
on PATH that locates the repo root from cwd makes worktree invocation unambiguous.

### Answers to the open questions (user-mode-spine + race-debugging view)
- **Wrap or replace `qemu/*.sh`?** Wrap — strong +1. The start/run-until/inspect
  toolkit is what I reach for on every bugcheck; `triage-bugcheck` + auto-symbol
  already replaced my DbgPrint-rebuild loop. Build smoke/boot on top, keep the verbs.
- **Staleness model:** the single highest-value thing the CLI can do for me is kill
  stale-green: multi-staticlib stamping (#1) + cmake-configure-fail-loud (freeldr
  #4, hit it too) + media-newer-than-build assertion. Stale-green on the user-mode
  spine is invisible and I lost cycles to it.
- **One serial gotcha to encode:** DbgPrint is a no-op/AV inside winsrv (libcntpr's
  local DbgPrint) but works in csrsrv (resolves ntdll's). If the CLI ever suggests
  "add a DbgPrint here," it should know which modules can actually emit — else the
  marker silently never appears.

---

<!-- ntdll-worktree-painpoints -->
## Pain points from worktree `vibeos-wt-ntdll`

Context: per-leaf ntdll.dll C→Rust cohabitation (rtl/version, etw/trace, dbg/dbgui,
ldr/verifier, …). Each leaf is a translate → cmake cohab-flip → cargo `.a` → roll ISO →
boot ×3 cycle. Heavy `start.sh`/`wait-for.sh`/`stop.sh` use.

### 1. "Is MY freshly-built DLL actually the one in the boot media?" — per-module, not whole-media, provenance
The #1 cohab false-green: I rebuild `ntdll.dll`, but did it actually graft into
`livecd.iso`, or did the ISO keep a stale copy? The §6 provenance model hashes
`boot_media_sha` *as a whole*, which doesn't answer "is the module I just touched the one
that'll boot." **Want:** preflight that extracts the specific module(s) I changed *out of
the boot media* and hashes them against the freshly-built `build-<arch>/.../ntdll.dll`. A
whole-ISO hash passes even when the graft step silently no-op'd my DLL.

### 2. `ninja livecd` is breakable by a *concurrent* worktree — need decoupled media re-roll
Multiple worktrees share the base branch; when freeldr is mid-port, `ninja livecd` fails
on freeldr and I can't roll an ISO *even though my `ntdll.dll` built perfectly*. I fall
back to hand-running the raw `mkisofs`+`isohybrid` incantation to graft just the fresh DLL.
**Want:** `smoke`/`build --refresh` media-roll that can graft an already-built module into
existing boot media *without* re-driving a broken sibling target — i.e. "the DLL is fresh,
re-stage it into the ISO" as its own step, decoupled from the full `livecd` graph.

### 3. Per-DLL cargo target dirs must be in the dynamic-discovery / nuke set
`ntdll-rust` (and `ntdll_vista`) build to their *own* `rust-target` dirs under
`build-<arch>/dll/ntdll/`, distinct from `ntoskrnl-rust`'s. The smoke conftest nukes the
ntoskrnl rust target but NOT these per-DLL ones, so cargo's incremental cache masks
regressions. §6's "Dynamic Target Discovery" (`build-*/rust-target` glob) is exactly right
— **please make the glob explicitly catch `build-*/dll/**/rust-target` and every
per-consumer target dir**, and have `rust_src_hash` cover all of them, or the masking
persists for cohab DLLs. (Same root cause as the rtl-priv worktree's items 1–2 and 4 —
the per-consumer target-dir multiplication is a shared, cross-track hazard.)

### 4. The cohab build isn't a ninja node — hash-based dirty detection beats the DEPENDS list
The Rust `.a` is wired via `add_custom_command(DEPENDS <every .rs by hand>)`. Add a new
`.rs` and forget to append it to that DEPENDS list → cargo silently doesn't rebuild →
stale `.a` links clean. **Want:** `build` treats the rust crate as dirty by *content hash
of the crate's src tree*, independent of whether cmake's hand-maintained DEPENDS list is
complete. This kills a footgun the Ninja graph structurally can't see.

### 5. User-mode symbolication + Rust-panic banner as a first-class failure class
§13 triage is kernel-centric (trap frames, bugcheck). My failures are often a *user-mode*
DLL panicking: `ntdll-rust`'s `#[panic_handler]` prints `!!! [NTDLL] RUST PANIC HIT !!!`
then `PANIC LOC: file:line`. **Want:** (a) a `RUST_PANIC` failure classification that greps
serial for that banner and surfaces the `file:line` directly into `summary.json`; (b)
symbolicator that resolves addresses against user-mode image bases (`ntdll.dll`,
`csrss.exe`, `winsrv.dll`), not just the kernel. The whole user-mode-spine cohort
(smss/csrss/ntdll/win32k) needs this.

### 6. Fail-fast on a panic regex instead of burning the full timeout
When ntdll panics at second ~12, I still wait the full 300s `shell ready` timeout before
the harness gives up. **Want:** `--fail-on "RUST PANIC HIT|BugCheck|Unhandled exception"`
alongside `--until "shell ready"`, so a hard-failure marker aborts the boot immediately
with the captured context. Turns a 5-min dead wait into a 15-sec verdict.

### 7. Same-arch `--repeat N` for flake detection (distinct from `--arch all`)
There's a known intermittent post-`shell ready` explorer `0xF4` race (win32k/smss domain)
that makes byte-identical binaries green one boot and `0xF4` the next — so my discipline is
"never trust n=1, boot ≥3×." I hand-loop start/wait/stop three times and eyeball. **Want:**
`smoke --arch i386 --repeat 3` returning `{3/3 reached}` or `{2/3, flake at marker X on run
2}`. This is *orthogonal* to the freeldr worktree's `--arch all` matrix (§Appendix 6) —
mine is same-arch repetition for flake-rate.

### 8. Classify red against a known-flake catalog so "red" ≠ automatically "my change"
When smoke goes red I waste cycles deciding: my ntdll flip? the known `0xF4` race (someone
else's domain)? or the shared base branch shifted under me? **Want:** triage matches the
failure signature (marker + bugcheck code + faulting frame) against a small known-flake
catalog and tags `KNOWN_FLAKE` vs `NEW_REGRESSION`. A `KNOWN_FLAKE` verdict with "re-run to
confirm" short-circuits the "is this even mine?" investigation that the n=1 race forces today.

### Answers to the open questions (per-DLL-cohab consumer view)
- **Wrap or replace `qemu/*.sh`?** Wrap. The `start.sh`/`wait-for.sh`/`stop.sh` run-id model
  is what I lean on all day; layer `smoke`/`boot` over it. The thing I'd *add* under the same
  CLI is media-graft + per-module provenance (items 1–2), which has no shell equivalent today.
- **Trace-catalog / static checks:** extend `check` to run `verify_abi` and the `@N`-decoration
  check on the *cohab `.a`* (nm the consumer DLL for the symbol + correct stdcall decoration)
  — calling-convention mismatch is the #1 i386 cohab crash cause and it's a pure static check
  that'd catch the bug before any boot.

---

<!-- win32k-worktree-painpoints -->
## Pain points from worktree `vibeos-wt-win32k`

Context: bulk win32k C→Rust cohabitation flips (gdi/ntgdi geometry+math waves, eng/dib
clusters, USER spine). My unit of work is a 3×3 matrix — {cargo check, link, smoke} ×
{i386, amd64, arm64} — run dozens of times per session. The dominant bugs are i386-only
(PE decoration / ABI) and they surface at LINK or at BOOT, never at cargo check. Frictions:

### 1. The harness only nukes ONE of ~5 rust target dirs — #1 false-green source
`conftest.py` nukes `ntoskrnl/rust-kernel-target` but NOT `win32k-rust`'s target (nor
freeldr's, csrss's, rtl's). A cohab flip links clean against a STALE `libwin32k_rust.a`
and smokes green even when the new Rust is broken. `--skip-clean-build` widens the hole.
"Cold green" is bogus unless EVERY rust target is nuked. The plan's §6 "Dynamic Target
Discovery" (`build-*/rust-target` glob) is exactly the fix — please make `--clean` and the
`rust_src_hash` staleness check cover ALL discovered cargo target dirs, not just ntoskrnl's,
and have the stamp record a per-target hash so a stale sibling .a is flagged, not trusted.
(Strong +1 to the rtl-priv / smss-csrss / ntdll worktrees above — this is the single most
cross-track hazard in the whole repo.)

### 2. cgu=256 repartition means a NEW workspace member can regress an UNTOUCHED arch
Adding a crate repartitions codegen units across the whole workspace; that can surface a
captures(none)→ud2 in a file I never edited, visible only on a live boot. So "I only added
one module" is never safe — the whole 3-arch smoke must re-run cold. I want the staleness
engine to treat "workspace membership changed" (Cargo.toml/lib.rs `extern crate` diff) as a
force-cold-rebuild trigger, and the post-build ud2 scan (§13) to run on ALL three binaries
every time, with a diff vs a checked-in ud2 allowlist so KNOWN_CGU_UD2 entries don't cry wolf.
(The smss-csrss worktree's `smoke --ref <git-ref>` confound-check is the right companion to
this — distinguishing "my bug" from "pre-existing cgu-shift" needs both.)

### 3. Link is a first-class failure axis the plan folds into "build" — split it out
For cohab work the canonical failure is: **cargo check GREEN ×3, LINK FAIL on i386 only**
(missing `#[no_mangle]`, wrong `extern`, or a dangling-cfg orphan fn that silently never
compiles so only the external C reference is undefined). Today that undefined-symbol line is
buried in thousands of lines of `ninja livecd` output and I scroll hunting for it. Please make
`build`/`smoke` classify `LINK_FAIL` distinctly from cargo-check errors and from boot failures,
and surface the undefined symbol(s) in the 12-line summary, e.g.
`LINK FAIL i386: undefined _XFORMOBJ_iSetXform, @IntSetSysColors@12`.

### 4. i386 link triage must be PE-decoration-aware
`undefined: _Foo` vs `_Foo@N` vs `@Foo@N` tells me *which ABI bug* it is (cdecl / stdcall /
fastcall mismatch or a forgotten `#[no_mangle]`). I decode this by hand every time. A triage
class that recognizes the three i386 decoration schemes and suggests the cause —
"stdcall symbol `IntSetSysColors@12` undefined → likely missing #[no_mangle] or wrong extern
on i386" — would collapse my most common debug loop. This is a build-time triage the plan's
§13 (runtime bugcheck) doesn't cover. (Dovetails with the ntdll worktree's static `@N`-decoration
nm-check ask — same root cause, one at build, one at check.)

### 5. Lints need a BASELINE/diff, not absolute counts
`verify_abi.py` reports 65 pre-existing ntoskrnl mismatches in dead-cfg paths — pure noise that
buries the ONE new mismatch I just introduced. Same risk for `lint_kernel`/ud2. The plan makes
these blocking in `check` (§8) — please make them diff against a checked-in baseline so the gate
is "0 NEW findings vs baseline," not "0 findings." Otherwise I'll learn to ignore a perpetually-
red lint, which is worse than no lint.

### 6. n=1 smoke is a lie on these branches — formalize repeat-and-report
The win32k/csrss spine has a known intermittent i386 `0xF4` at `[EXPLORER] _tWinMain`
(suspended-thread race) — byte-identical binaries go green one run, red the next. I've trained
myself to never trust a single boot, and I re-run 3× by hand to tell a real regression from the
race. Snapshot-restore itself adds input/mouse flake. Please add `smoke --repeat N`
(or `--until-stable`) that boots K times and reports a flake rate in the summary
(`i386: 2/3 pass — FLAKY`), so "did I regress, or is it the race?" is one command, not three.
(Same ask as smss-csrss #2 and ntdll #7 — note all three of us hit the *same* `_tWinMain` race;
worth a known-flake catalog entry per ntdll #8.)

### 7. The 3×3 matrix is my real verb — and it's i386-link/i386-smoke that matter
Echoing the freeldr `--arch all` ask, but for me the matrix cells aren't equal: amd64/arm64
almost always pass when cargo check passes; the action is in i386-link and i386-smoke. A
`smoke --arch all --json` matrix that calls out the i386 link + boot cells (and runs cargo-check
→ link → boot as one chain per arch) would replace ~90% of my manual invocations.

### 8. Extend the debugcon milestone set past kernel init into the win32k/user spine
debugcon triage (§12/§13) localizes kernel-init hangs beautifully, but my hangs are usually
*after* kernel init — between SMSS → csrss → win32k InitVideo → winlogon → explorer. To the
current summary a hang at win32k-init and a hang at explorer both just look like "didn't reach
shell ready." Please let the milestone catalog carry user-mode-spine markers so a pre-`shell
ready` hang is pinned to a stage, not a binary pass/fail.

### Cross-cutting: multi-worktree CMake-context check is essential (don't drop it)
Per §6 "CMake Context Integrity," the staleness engine compares worktree path vs the cmake
cache. KEEP THIS — with concurrent worktrees (`vibeos-wt-win32k`, `vibeos-wt-freeldr`, main) a
`build-i386` configured for a different worktree silently builds the wrong tree. This check, plus
the all-rust-targets hashing in #1, is what makes smoke trustworthy across parallel agents.

### Answers to the open questions (cohab-flip-consumer view)
- **Wrap or replace `qemu/*.sh`?** Wrap. The kernel-debug toolkit (start/inspect/run-until/
  triage-bugcheck) is what I reach for when a flip bugchecks mid-init; keep it, layer smoke on top.
- **Whole-tree lint?** Strongly yes — a win32k ABI change breaks i386 link in a file the diff
  never touched. Diff-scoped linting would miss exactly my failure mode.
