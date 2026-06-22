"""`vibeos-cli debug ...` and `check`.

Per the "wrap first" decision (plan §17), the debug verbs wrap the proven shell
toolkit (start.sh / inspect.sh / run-until.sh / wait-for.sh / tail.sh /
status.sh / stop.sh) — porting to a Python gdb.py engine is P2. `debug diff` is
new. `check` runs the source lints with the §20.3 baseline-diff gate.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from . import lints
from .artifacts import Run

# debug verb -> shell tool that implements it (run-id passthrough)
_SHELL_VERB = {
    "start": "start.sh",
    "inspect": "inspect.sh",
    "run-until": "run-until.sh",
    "bt": "inspect.sh",  # bt/triage are inspect.sh --command forms
    "triage-bugcheck": "inspect.sh",
    "wait-for": "wait-for.sh",
    "tail": "tail.sh",
    "status": "status.sh",
    "stop": "stop.sh",
    "screenshot": "inspect.sh",
    "profile": "sample-pc.sh",
    "gdb": "inspect.sh",
}

_HEX = re.compile(r"0x[0-9a-fA-F]+")
_PID = re.compile(r"\b(pid|tid)[=: ]+\d+", re.I)


def _normalize(line: str) -> str:
    line = _HEX.sub("0xX", line)
    line = _PID.sub(lambda m: m.group(0).split()[0] + "=N", line)
    return line.rstrip()


def debug_diff(args, repo: Path) -> int:
    """Normalized serial diff between two run-ids — surfaces arch-unique lines
    (smss-csrss #4: arch-only-bug differential)."""

    def serial_of(run_id: str) -> list[str]:
        p = repo / "qemu" / "runs" / run_id / "serial.log"
        if not p.is_file():
            print(f"no serial.log for run {run_id} at {p}")
            return []
        raw = p.read_text(errors="replace")
        return [_normalize(line) for line in raw.splitlines() if line.strip()]

    a = set(serial_of(args.run_a))
    b = set(serial_of(args.run_b))
    only_a = sorted(a - b)
    only_b = sorted(b - a)
    print(f"=== only in {args.run_a} ({len(only_a)}) ===")
    for line in only_a[:60]:
        print("  " + line[:160])
    print(f"=== only in {args.run_b} ({len(only_b)}) ===")
    for line in only_b[:60]:
        print("  " + line[:160])
    return 0


def debug(args, repo: Path) -> int:
    verb = args.verb
    if verb == "diff":
        return debug_diff(args, repo)
    tool = _SHELL_VERB.get(verb)
    if not tool:
        print(f"unknown debug verb: {verb}")
        return 2
    script = repo / "qemu" / tool
    if not script.is_file():
        print(f"shell tool not found: {script}")
        return 127
    # Pass through remaining args verbatim; inherit stdio (interactive-ish).
    cmd = ["bash", str(script), *getattr(args, "rest", [])]
    return subprocess.run(cmd, cwd=repo).returncode


def check(args, repo: Path) -> int:
    if getattr(args, "update_baseline", False):
        counts = lints.write_baselines(repo)
        for name, n in counts.items():
            print(f"  baseline {name}: {n} findings")
        print(
            f"wrote {len(counts)} baselines to tools/lint-baseline/ "
            "(review before committing — captures CURRENT known findings as accepted)"
        )
        return 0
    run = Run("check", args.arch if getattr(args, "arch", None) else "all")
    strict = bool(getattr(args, "strict", False))
    lints.run_source_lints(run, repo, strict=strict)
    if strict:
        # Guard the cargo-driven link PoCs (ntoskrnl/ntdll/ntdll_vista) against
        # silent bit-rot — the pre-commit/merge gate is where to catch the
        # off-default-path replacement links drifting. Self-skips when their
        # prerequisites (target json / build-<arch> / nightly) are absent.
        lints.run_cargo_link_guard(run, repo, getattr(args, "arch", None))
        # Guard the cmake-off committed fallbacks (ks*.inc/cflags/.res/widl .rs)
        # against silent staleness vs cmake regeneration. Self-skips without a
        # configured build-<arch>/.
        lints.run_committed_drift_guard(run, repo, getattr(args, "arch", None))
        if getattr(args, "arch", None):
            lints.run_binary_lints(run, repo, args.arch, strict=True)
    return run.finalize(repo)
