# `qemu/` — development & kernel-debugging scripts

Index of the scripts in this directory, grouped by what they're for. Most are
part of the **agent-driven kernel-debugging toolkit** (design doc:
`docs/engineering/kernel-debugging-infra.md`); the rest are boot-media setup and
one-off symbolicators. Each script's own header (`sed -n '2,15p' qemu/<x>.sh`)
has full usage.

> **Higher layer:** `./vibeos-cli` (repo root) is the unified front door for
> building, smoke verdicts, lint gating, and GDB debugging — it wraps these
> scripts and adds staleness gating + auto-triage + bounded summaries. Prefer it
> for the build→boot→verdict loop; drop to these scripts for ad-hoc/interactive
> single ops. Guide: `tests/qemu/cli/README.md`.

## Kernel-debugging toolkit (run-id based, batch-invocable)

Boot a kernel, watch it, inspect it, tear it down — one batch command in,
structured result out. Standard recipe:

```bash
RUN=$(qemu/start.sh amd64 --gdb)
qemu/wait-for.sh $RUN --pattern "ACPI Timer" --timeout 60
qemu/run-until.sh $RUN --break KeBugCheckEx --timeout 180 --then triage-bugcheck
qemu/stop.sh $RUN
```

| Script | Purpose |
|--------|---------|
| `start.sh` | Spawn a QEMU run (preflight + manifest + run-id). `--gdb` opens gdbstub. Acquires a concurrency slot; auto-reaps orphans first. |
| `stop.sh` | Stop a run, preserve artifacts. Idempotent. |
| `status.sh` | Read-only state of **one** run-id (process, serial size, gdb). |
| `ps` | **What's running right now across ALL worktrees** + slot usage (Python; first piece of the `q` CLI port). Start here when you're unsure what's live. |
| `wait-for.sh` | Poll a run's serial log for a regex; rich timeout response. |
| `tail.sh` | Sanitized read of a run's serial log. |
| `inspect.sh` | Single-shot GDB inspection (`bt`, `dump-thread`, `triage-bugcheck`, …). |
| `run-until.sh` | Breakpoint + continue + inspect + cleanup, all in one batch. |
| `reap.sh` | Cull orphan QEMUs (dead owner) and any VM past its 30-min cap. Auto-run by `start.sh` and the smoke harness; safe to run by hand. |
| `preflight.sh` | Environment checks (gdb-multiarch, DWARF richness, …). Auto-run inside `start.sh`. |
| `sample-pc.sh` | Crude sampling profiler over a running run (per-symbol / per-process histograms). |

## Offline symbol tools (no running kernel needed)

| Script | Purpose |
|--------|---------|
| `build-index.sh <arch>` | Build the cached symbol index `symbols-<arch>.tsv`. Run once per kernel build. |
| `addr2sym.sh <arch> [--reloc HEX] <addr>…` | Address → symbol+offset (ntoskrnl). |
| `sym2addr.sh <arch> <symbol>` | Symbol → address; handles i386 PE decoration. |
| `sym.sh <addr>…` | Symbolicate a **uefildr.efi** (ARM64 UEFI bootloader) crash address — separate from the ntoskrnl pair above. |

## Boot media & manual launch

| Script | Purpose |
|--------|---------|
| `run.sh` | Interactive dev launcher (serial/GUI/debug) for arm64/i386/amd64. The hands-on counterpart to the toolkit's `start.sh`. |
| `mkramdisk.sh` | Build/refresh the ARM64 `ramdisk.img` + `boot.img`. Used by the smoke harness's arm64 `--refresh`. |
| `mouse_click.py` | Host-side mouse injection (GUI input testing). |

## Resource & lifetime management

The toolkit and the smoke harness share `/tmp/vibeos-qemu-locks/` to keep QEMUs
from exhausting host KVM/RAM:

- **Concurrency cap** — `slots/slot-NN` flock pool limits concurrent QEMUs to
  `VIBEOS_QEMU_MAX_CONCURRENT` (default 10) across all worktrees; over the cap,
  launchers queue.
- **Per-arch sub-cap** — layered under the global cap: `slots/arch-<arch>-NN`
  serializes **same-arch** boots (`VIBEOS_QEMU_CAP_I386` / `_AMD64`, default 1;
  `_ARM64` default 0 = uncapped). Two i386 (or two amd64) KVM guests starve each
  other on `/dev/kvm` + the shared IDE path into false `HANG_TIMEOUT`s, so a
  second same-arch boot **queues automatically** instead of contending; an arm64
  (TCG) boot still runs in parallel with an x86 one. This is what makes it safe
  to smoke without hand-coordinating same-arch boots.
- **Lifetime cap** — no QEMU outlives `VIBEOS_QEMU_MAX_LIFETIME` (default 1800 s);
  queue-wait doesn't count against it.
- **Orphan reaping** — `reap.sh` culls QEMUs whose owning session died.

`qemu/ps` is the one-glance view of all of the above. See
`docs/engineering/kernel-debugging-infra.md` for the full design.

## Deprecated / superseded

| Script | Status |
|--------|--------|
| `populate-disk.sh` | **Superseded by `mkramdisk.sh`** (same job: stage freeldr.ini / ntoskrnl / hal / kdcom / hive onto the ARM64 boot image). Zero references in-tree; removal pending owner sign-off. |
