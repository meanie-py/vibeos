"""`vibeos-cli debug ...` and `check`.

Per the "wrap first" decision (plan §17), the debug verbs wrap the proven shell
toolkit (start.sh / inspect.sh / run-until.sh / wait-for.sh / tail.sh /
status.sh / stop.sh) — porting to a Python gdb.py engine is P2. `debug diff` is
new. `check` runs the source lints with the §20.3 baseline-diff gate.
"""

from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

from . import lints
from .artifacts import Run, Stage

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


_SHELLS = {s for b in ("bash", "sh", "dash", "zsh", "fish", "ksh") for s in (b, "-" + b)}


def _session_owner(start: int) -> int:
    """The first non-shell ancestor of `start` -- qemu/lib/common.sh:owner_pid_for, seeded
    ABOVE this process.

    start.sh walks up from its own parent and stops at the first non-shell, which under
    `vibeos-cli debug start` is this CLI: a process that exits as soon as start.sh returns.
    The VM was then recorded as owned by a dead pid, and the next smoke from ANY lane reaped
    it as an orphan (two arm64 debug VMs SIGTERMed mid-gdb by a sibling's test_payload,
    2026-10-03, the gdb session surfacing it as "Remote communication error"). The session
    that outlives us is whatever ran us, past the shells in between."""
    p = start
    for _ in range(12):
        if p <= 1:
            break
        try:
            comm = Path(f"/proc/{p}/comm").read_text().strip()
            stat = Path(f"/proc/{p}/stat").read_text()
        except OSError:
            break
        if comm not in _SHELLS:
            return p
        p = int(stat.rsplit(") ", 1)[1].split()[1])
    return start


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
    env = None
    if verb == "start" and not os.environ.get("VIBEOS_QEMU_OWNER_PID"):
        env = {**os.environ, "VIBEOS_QEMU_OWNER_PID": str(_session_owner(os.getppid()))}
    return subprocess.run(cmd, cwd=repo, env=env).returncode


def _strict_arch_checks(run: Run, repo: Path, arch: str | None) -> None:
    """One guard list shared by full checks and the gate's arch partition."""
    lints.run_committed_drift_guard(run, repo, arch)
    if arch:
        lints.run_binary_lints(run, repo, arch, strict=True)


def check(args, repo: Path) -> int:
    ub = getattr(args, "update_baseline", None)
    selection = getattr(args, "only", None)
    scope = getattr(args, "scope", None)
    if scope is not None:
        if (
            scope not in ("source", "source-early", "source-deferred", "arch")
            or ub is not None
            or selection is not None
        ):
            print("invalid --scope or combination with --only/--update-baseline")
            return 2
        if not getattr(args, "strict", False) or (
            scope == "arch" and not getattr(args, "arch", None)
        ):
            print("--scope requires --strict; --scope arch also requires --arch")
            return 2
        run = Run(f"check-{scope}", getattr(args, "arch", None) or "all")
        run.extra.update(coverage="partial", scope=scope)
        run.add(Stage("partial", status="warn", detail=f"{scope} partition only; not a full gate"))
        if scope in ("source-early", "source-deferred"):
            expected = [name for name, _, _ in lints.source_partition_specs(repo, scope)]
            lints.run_source_lints(run, repo, strict=True, only=set(expected))
            actual = [stage.name for stage in run.stages if stage.name != "partial"]
            run.extra.update(
                checked_source_checks=actual,
                unchecked_source_checks=sorted(lints.lint_tool_names(repo) - set(expected)),
                unchecked_categories=[
                    "architecture guards",
                    "builds",
                    "smokes",
                    "tooling",
                    "difftests",
                ],
            )
            if actual != expected:
                run.add(
                    Stage(
                        "coverage-inventory",
                        status="fail",
                        detail="executed checks differ from required partition",
                    )
                )
        elif scope == "source":
            lints.run_source_lints(run, repo, strict=True)
        else:
            _strict_arch_checks(run, repo, args.arch)
        return run.finalize(repo)
    if selection is not None:
        if ub is not None:
            print("--only cannot be combined with --update-baseline")
            return 2
        only = {name.strip() for name in selection.split(",") if name.strip()}
        known = lints.lint_tool_names(repo)
        if not only or only - known:
            print(f"empty or unknown scanner selection: {', '.join(sorted(only - known))}")
            print(f"known: {', '.join(sorted(known))}")
            return 2
        # A separate command identity prevents a partial result being mistaken
        # for a complete check in artifact discovery or fleet health history.
        run = Run("check-partial", getattr(args, "arch", None) or "all")
        run.extra["coverage"] = "partial"
        run.extra["selected_scanners"] = sorted(only)
        run.extra["omitted_scanners"] = sorted(known - only)
        run.add(
            Stage("partial", status="warn", detail="selected source scanners only; not a full gate")
        )
        lints.run_source_lints(run, repo, strict=bool(getattr(args, "strict", False)), only=only)
        return run.finalize(repo)
    if ub is not None:
        # --update-baseline           -> "__ALL__" (blanket; the footgun, see lints)
        # --update-baseline lint_kernel[,tool2,...] -> just those tools (T2 1b)
        only = None if ub == "__ALL__" else {t.strip() for t in ub.split(",") if t.strip()}
        if only is not None:
            known = lints.lint_tool_names(repo)
            unknown = only - known
            if unknown:
                print(f"unknown lint tool(s): {', '.join(sorted(unknown))}")
                print(f"known: {', '.join(sorted(known))}")
                return 2
        counts = lints.write_baselines(repo, only=only)
        for name, d in counts.items():
            delta = (
                f"+{d['added']}/-{d['removed']}" if (d["added"] or d["removed"]) else "unchanged"
            )
            print(f"  baseline {name}: {d['count']} findings ({delta})")
        scope = "ALL tools" if only is None else ", ".join(sorted(only))
        print(
            f"wrote {len(counts)} baseline(s) [{scope}] to tools/lint-baseline/ — review the "
            "+added lines before committing (each is a NEW finding now accepted as debt)"
        )
        return 0
    run = Run("check", args.arch if getattr(args, "arch", None) else "all")
    strict = bool(getattr(args, "strict", False))
    lints.run_source_lints(run, repo, strict=strict)
    if strict:
        _strict_arch_checks(run, repo, getattr(args, "arch", None))
    return run.finalize(repo)
