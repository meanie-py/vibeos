"""Provision the worktree-local Rust execution broker and its build environment."""

from __future__ import annotations

import os
from pathlib import Path

from tools import build_infra_compat as native

from . import buildpolicy, canon, hostlock


def binary(repo: Path) -> Path:
    return repo / ".cache/host-jobs/host-jobs"


def ensure(repo: Path) -> Path:
    try:
        return Path(native.call("build_policy_ensure_host_jobs", {"repo": repo}))
    except native.NativeError as error:
        raise RuntimeError(str(error)) from error


def environment(
    repo: Path, env: dict | None, *, canonical: bool, run_dir: Path | None = None
) -> dict:
    return native.call(
        "build_policy_host_env",
        {
            "repo": repo,
            "environment": dict(os.environ if env is None else env),
            "canonical": Path(canon.canon_path()) if canonical else None,
            "run_dir": run_dir,
            "owner": os.getpid(),
            "class": hostlock.job_class(),
            "policy": buildpolicy.read(hostlock.POLICY_DIR),
        },
    )


def status(repo: Path) -> dict:
    return native.call(
        "build_policy_host_status",
        {
            "repo": repo,
            "root": hostlock.POLICY_DIR,
            "environment": dict(os.environ),
        },
    )


def render_status(state: dict) -> str:
    if not state.get("cpu_slots"):
        return "host jobs: not provisioned in this worktree (run build)"
    rows = state["jobs"]
    active = sum(row.get("state") == "active" for row in rows)
    granted = sum(row.get("state") == "granted" for row in rows)
    lines = [
        f"host jobs: {active + granted}/{state['cpu_slots']} CPU slots "
        f"({active} running, {granted} granted), "
        f"{len(rows) - active - granted} waiting; link cap {state['link_slots']}; "
        "landing (integration) priority, half the slots reserved while a gate is present, "
        "every class ages in after 120s"
    ]
    if state["legacy_adjusted_cpu_slots"] != state["cpu_slots"]:
        lines.append(
            f"  old-lane reservations leave {state['legacy_adjusted_cpu_slots']} job slots"
        )
    if "memory_budget_kib" in state:
        lines.append(
            f"  memory: {state['memory_reserved_kib'] / 1048576:.1f}/"
            f"{state['memory_budget_kib'] / 1048576:.1f} GiB reserved; "
            f"{state['memory_available_kib'] / 1048576:.1f} GiB available, "
            f"{state['memory_headroom_kib'] / 1048576:.1f} GiB headroom"
        )
        lines.append(f"  job observations: {state['metrics_path']}")
    for row in rows[:12]:
        lines.append(
            f"  {row['state']} {row['kind']} pid={row['pid']} "
            f"owner={row['owner_pid']} {row['job_class']} {row['repo'] or row['cwd']}"
        )
    if len(rows) > 12:
        lines.append(f"  +{len(rows) - 12} jobs; --json lists all owners and commands")
    return "\n".join(lines)
