"""`vibeos-cli debug ...` and `check`.

The decisions are native (`tools/vibeos-test/src/debugcmd.rs`): which shell tool a debug
verb runs, whose session owns a debug VM, what `debug diff` compares, and what a `check`
invocation asks for. This facade runs the shell toolkit and the lints with that plan.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

from tools import qemu_infra_compat as native

from . import lints
from .artifacts import Run, Stage


def _native(op: str, **kwargs):
    return native.call("debugcmd_" + op, kwargs)


def _normalize(line: str) -> str:
    return _native("normalize", line=line)


def debug_diff(args, repo: Path) -> int:
    """Normalized serial diff between two run-ids — surfaces arch-unique lines
    (smss-csrss #4: arch-only-bug differential)."""

    def serial_of(run_id: str) -> str:
        p = repo / "qemu" / "runs" / run_id / "serial.log"
        if not p.is_file():
            print(f"no serial.log for run {run_id} at {p}")
            return ""
        return p.read_text(errors="replace")

    only_a, only_b = _native("diff", a=serial_of(args.run_a), b=serial_of(args.run_b))
    print(f"=== only in {args.run_a} ({len(only_a)}) ===")
    for line in only_a[:60]:
        print("  " + line[:160])
    print(f"=== only in {args.run_b} ({len(only_b)}) ===")
    for line in only_b[:60]:
        print("  " + line[:160])
    return 0


def _session_owner(start: int) -> int:
    """The first non-shell ancestor of `start`: the session that outlives the CLI."""
    return _native("session_owner", proc="/proc", start=start)


def debug(args, repo: Path) -> int:
    verb = args.verb
    if verb == "diff":
        return debug_diff(args, repo)
    tool = _native("verb_tool", verb=verb)
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
    request = {
        "update_baseline": getattr(args, "update_baseline", None),
        "only": getattr(args, "only", None),
        "scope": getattr(args, "scope", None),
        "strict": bool(getattr(args, "strict", False)),
        "arch": getattr(args, "arch", None),
    }
    known = (
        sorted(lints.lint_tool_names(repo)) if _native("check_needs_inventory", **request) else []
    )
    plan = _native("check_plan", known=known, **request)
    if plan.get("rc") is not None:
        for line in plan["lines"]:
            print(line)
        return plan["rc"]
    mode = plan["mode"]
    if mode == "update_baseline":
        only = None if plan["only"] is None else set(plan["only"])
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
    run = Run(plan["run"], plan["arch"])
    run.extra.update(plan.get("extra") or {})
    if plan.get("stage"):
        st = plan["stage"]
        run.add(Stage(st["name"], status=st["status"], detail=st["detail"]))
    if mode == "scope":
        scope = plan["scope"]
        if scope in ("source-early", "source-deferred"):
            expected = [name for name, _, _ in lints.source_partition_specs(repo, scope)]
            lints.run_source_lints(run, repo, strict=True, only=set(expected))
            # Not the reconfigure stages: run_source_lints regenerates a stale build.ninja first
            # (a merge that touched cmake input), and counting those as checks failed a gate
            # whose every required check had run (20261108-020815-xRELnn).
            actual = [
                stage.name
                for stage in run.stages
                if stage.name != "partial" and not stage.name.startswith("reconfigure:")
            ]
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
    elif mode == "partial":
        lints.run_source_lints(
            run, repo, strict=plan["strict"], only=set(plan["only"]), new_fail=True
        )
    else:
        # NEW findings fail with or without --strict: they are what the gate refuses, and a
        # plain check that merely warned about them was how lanes submitted work the
        # preflight then bounced (18 of 56 plain checks, 2026-10-14..16, ended WARN with a
        # NEW finding and exit 0). --strict still adds the coverage rule (a missing tool
        # fails) and the per-arch guards.
        lints.run_source_lints(run, repo, strict=plan["strict"], new_fail=True)
        if plan["strict"]:
            _strict_arch_checks(run, repo, getattr(args, "arch", None))
    return run.finalize(repo)
