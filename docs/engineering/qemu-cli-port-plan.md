# Plan: port the `qemu/` shell toolkit to a Python `q` CLI

**Status:** In progress (captured 2026-05-30). Decided scope: *full* port to a
Python CLI, executed incrementally, with shell scripts kept as deprecated shims
until each command is proven on a real debug session.

**Done so far:** `qemu/ps` (was `ps.sh`) ported to Python — the lowest-risk
piece (read-only observability, no lifecycle), done first because the shell
version hit two shell-only bugs in one session: multibyte `printf` misalignment
and `$(( ))` choking on the float `deadline` the harness writes. Standalone
today; folds into `q ps` when the dispatcher lands.

## Motivation

The `qemu/` directory is ~17 shell scripts (the agent-driven kernel-debugging
toolkit) and the smoke harness under `tests/qemu/` is already substantial
Python (`vm.py`, `conftest.py`, `serial_client.py`, `qmp_client.py`,
`debugcon.py`). The two halves **independently reimplement the same resource
layer**, and keeping them byte-compatible is a standing maintenance tax.

Concrete evidence (from the 2026-05-30 resource-management work):

| Concern | Shell impl | Python impl |
|---|---|---|
| owner-pid ancestor walk | `lib/common.sh:owner_pid_for` | (pytest is implicitly the owner) |
| concurrency slot pool | `lib/common.sh:acquire_qemu_slot` | `vm.py:_acquire_slot` |
| orphan reaper | `reap.sh` | `vm.py:reap_orphan_qemus` |
| 30-min lifetime cap | `start.sh` detached watchdog | `vm.py` `threading.Timer` |
| lockfile JSON schema | `jq` in `start.sh` | `json.dumps` in `vm.py` |

Both sides share `/tmp/vibeos-qemu-locks/` and must agree on the lockfile
schema, slot-dir layout, and deadline semantics or the cross-worktree reaper
silently breaks. They were hand-matched. That drift risk is the primary driver.

Secondary pains:
- `ps.sh` forks `jq` ~7× per lockfile (≈70 forks for 10 VMs) where Python reads
  each file once into a dict.
- Shell footguns hit during this session alone: `set -m` for process groups,
  the fd-200 `flock` inheritance subtlety, multibyte-`printf` column misalignment.
- GDB orchestration (`inspect.sh`, `run-until.sh`) builds command files via
  heredocs and parses output with `sed`/`grep` — structured parsing in Python
  is materially clearer and unit-testable.

## The key design constraint — two lifecycle models

`QemuVM` (smoke) and `start.sh` (toolkit) are **not** the same lifecycle and
cannot share one orchestration object:

- **Smoke `vm.py`**: pytest is long-lived and *holds* the VM in-process — QMP
  socket, the lifetime serial-drainer thread, and the slot fd all live on the
  `QemuVM` instance. The VM dies with pytest.
- **Toolkit `start.sh`**: launches a **detached daemon** and exits immediately;
  the VM outlives the launcher (that's why the slot is held via an fd the QEMU
  *inherits*, and why the lifetime cap needs a detached watchdog rather than an
  in-process timer).

So `q start` (which must return control to the user's shell while the VM keeps
running) is closer to `start.sh` than to `QemuVM.start()`. **The genuinely
shareable thing is the data/resource layer, not the lifecycle orchestration.**
Unifying the lifecycle is the trap to avoid.

This also means some shell idioms are *better left as the model even in Python*:
`setsid` / `disown` / `kill -- -<pgid>` / `flock` fd-inheritance are OS
primitives. The Python `start` will reproduce the detach (double-fork +
`os.setsid`, or `subprocess` + `pass_fds` for the slot fd), but the semantics
are the shell ones.

## Target architecture

```
tests/qemu/                 # promote to an importable package used by BOTH pytest and the CLI
  vm.py                     # QemuVM — in-process model (smoke). unchanged role.
  locks.py        (new)     # SINGLE source of truth: manifest+lockfile schema,
                            #   slot pool acquire/release, reaper, owner-pid, deadline.
  daemon.py       (new)     # detached-launch model (what `q start` uses): fork/setsid,
                            #   slot-fd inheritance, manifest write, watchdog arm.
  gdb.py          (new)     # inspect/run-until: command-file build + output parse + reloc.
  symbols.py      (new)     # build-index / addr2sym / sym2addr (objdump/nm wrappers).
  cli.py          (new)     # argparse dispatcher: q <subcommand>
  serial_client.py qmp_client.py debugcon.py   # as today; reused by both models.
qemu/q             (new)    # one-line wrapper: exec "$VENV/bin/python" -m tests.qemu.cli "$@"
qemu/*.sh                   # become thin shims → `q <cmd>`; deleted per-command once proven.
```

`q` subcommand surface (mirrors today's scripts):

```
q start <arch> [--gdb|--gdb-wait]      q ps [--json]
q stop <run-id>                        q reap [--dry-run]
q status <run-id> [--json]             q wait-for <run-id> --pattern … --timeout …
q tail <run-id> [--lines N]            q inspect <run-id> --command … [--json]
q run-until <run-id> --break … --then …
q build-index <arch>   q addr2sym …   q sym2addr …   q sample-pc <run-id>
```

(`qemu/sym.sh` — the uefildr/ARM64-bootloader symbolicator — is unrelated to the
ntoskrnl pair and can stay shell or become `q uefi-sym`; low priority.)

## Migration phases (risk-ordered; never break the live debug path)

**Cardinal rule:** don't break the debugger to make it prettier. Each shell
script stays as a shim and is deleted only after its `q` equivalent has survived
a *real* bugcheck/hang session, not just a unit test.

- **Phase 0 — shared layer (`locks.py`).** Extract the duplicated resource logic
  into one module. `vm.py` imports it; the bash side calls it via
  `python -m tests.qemu.cli {acquire-slot,reap,owner-pid}` shims. Kills the
  drift risk immediately; zero lifecycle change. *Lowest risk, do first.*
- **Phase 1 — observability/lifecycle CLI.** `q ps`, `q status`, `q reap`,
  `q stop`, `q tail`, `q wait-for`, then `q start` (via `daemon.py`). These are
  the `jq`-heavy ones. Shell scripts → shims.
- **Phase 2 — GDB commands.** `q inspect`, `q run-until` (`gdb.py`). Biggest
  quality jump; validate against a known bugcheck (e.g. the amd64 KiUnwaitThread
  flake recipe) before deleting `inspect.sh`/`run-until.sh`.
- **Phase 3 — offline symbol tools.** `q build-index/addr2sym/sym2addr`
  (`symbols.py`). Lowest payoff (thin objdump/nm wrappers); migrate last or leave.

## Back-compat & invocation

- `qemu/q` wrapper resolves the repo venv so callers don't need it activated
  (we already require `.venv` for pytest, so no new dependency).
- During transition each `foo.sh` becomes: `exec "$(dirname "$0")/q" foo "$@"`,
  so existing muscle memory and the CLAUDE.md/AGENTS.md recipes keep working.
- The standard recipe in AGENTS.md keeps working unchanged through Phase 1
  (it uses start/wait-for/run-until/stop).

## Testing strategy

- **Unit tests** (pytest, no QEMU): `locks.py` slot pool (cross-process via
  subprocess), reaper decision matrix, owner-pid walk, manifest round-trip,
  symbol lookup, GDB output parsing on captured fixtures.
- **Smoke gate**: every phase must keep i386/amd64/arm64 smoke green (the
  harness imports `locks.py`, so Phase 0 alone touches the boot path).
- **Real-session gate** for Phase 2: the `q inspect`/`q run-until` path must
  reproduce a real bugcheck triage before the shell versions are removed.

## Risks / non-goals

- **Risk:** regressing the one tool you reach for mid-fire. *Mitigation:* shims +
  real-session gate; delete shell only after proof.
- **Risk:** the detach model in Python (double-fork/setsid, slot-fd `pass_fds`)
  is fiddlier than shell. *Mitigation:* `daemon.py` is small and unit-testable
  with a sleep stand-in (as the flock inheritance was validated this session).
- **Non-goal:** unifying the two lifecycle models into one object. They are
  legitimately different; only the data layer is shared.
- **Non-goal:** porting `run.sh` (interactive dev launcher) or `mkramdisk.sh`
  (boot-media) — different audience, fine as shell.

## Acceptance criteria

1. One implementation of the resource layer (no shell/Python duplication).
2. `q` covers every toolkit command; `qemu/*.sh` are shims or removed.
3. All 3 arches' smoke stays green throughout.
4. The GDB path is proven on a real bugcheck before shell removal.
5. Docs updated: `qemu/README.md`, `kernel-debugging-infra.md`, AGENTS.md.
