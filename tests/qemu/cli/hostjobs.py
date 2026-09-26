"""Provision the worktree-local Rust execution broker and its build environment."""

from __future__ import annotations

import fcntl
import hashlib
import json
import os
import subprocess
from pathlib import Path

from tools.source_walk import source_files

from . import buildpolicy, canon, hostlock


def binary(repo: Path) -> Path:
    return repo / ".cache/host-jobs/host-jobs"


def ensure(repo: Path) -> Path:
    """Compile a pinned, dependency-free host tool once per source/compiler revision."""
    target = binary(repo)
    target.parent.mkdir(parents=True, exist_ok=True)
    source = repo / "tools/host-jobs"
    files = [*sorted(source_files(source / "src")), repo / "rust-toolchain.toml"]
    digest = hashlib.sha256(b"".join(p.read_bytes() for p in files)).hexdigest()
    stamp = target.with_suffix(".sha256")
    with target.with_suffix(".lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if target.is_file() and stamp.exists() and stamp.read_text() == digest:
            return target
        temporary = target.with_suffix(f".{os.getpid()}.tmp")
        try:
            subprocess.run(
                [
                    "rustc",
                    "--edition=2021",
                    "-O",
                    str(source / "src/main.rs"),
                    "-o",
                    str(temporary),
                ],
                cwd=repo,
                check=True,
                env={k: v for k, v in os.environ.items() if k != "RUSTUP_TOOLCHAIN"},
                capture_output=True,
                text=True,
            )
            temporary.replace(target)
            stamp.write_text(digest)
        except subprocess.CalledProcessError as error:
            raise RuntimeError(f"host-jobs bootstrap failed: {error.stderr}") from error
        finally:
            temporary.unlink(missing_ok=True)
    return target


def environment(
    repo: Path, env: dict | None, *, canonical: bool, run_dir: Path | None = None
) -> dict:
    result = dict(os.environ if env is None else env)
    runner = (Path(canon.canon_path()) if canonical else repo) / binary(repo).relative_to(repo)
    inner = result.get("RUSTC_WRAPPER", result.get("CARGO_BUILD_RUSTC_WRAPPER", ""))
    if inner and inner != str(runner):
        result["VIBEOS_INNER_RUSTC_WRAPPER"] = inner
    else:
        result.pop("VIBEOS_INNER_RUSTC_WRAPPER", None)
    result.update(
        VIBEOS_HOST_JOBS=str(runner),
        RUSTC_WRAPPER=str(runner),
        CARGO_BUILD_RUSTC_WRAPPER=str(runner),
        CARGO_BUILD_JOBS="1",
        VIBEOS_JOB_REPO=str(repo),
        VIBEOS_JOB_OWNER_PID=str(os.getpid()),
        VIBEOS_JOB_RUN=str(run_dir or ""),
        VIBEOS_JOB_CLASS=hostlock.job_class(),
    )
    policy = buildpolicy.read(hostlock.POLICY_DIR)
    if policy:
        result["VIBEOS_HOST_JOB_SLOTS"] = str(policy["max_builds"] * policy["ninja_jobs"])
        result["VIBEOS_HOST_CPU_LEASES"] = str(policy["leases"])
    return result


def status(repo: Path) -> dict:
    runner = binary(repo)
    if not runner.exists():
        return {"available": False}
    env = dict(os.environ)
    policy = buildpolicy.read(hostlock.POLICY_DIR)
    if policy:
        env["VIBEOS_HOST_JOB_SLOTS"] = str(policy["max_builds"] * policy["ninja_jobs"])
        env["VIBEOS_HOST_CPU_LEASES"] = str(policy["leases"])
    cp = subprocess.run(
        [str(runner), "--status"], check=True, capture_output=True, text=True, env=env
    )
    return json.loads(cp.stdout)


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
        "interactive priority with 120s aging"
    ]
    if state["legacy_adjusted_cpu_slots"] != state["cpu_slots"]:
        lines.append(
            f"  old-lane reservations leave {state['legacy_adjusted_cpu_slots']} job slots"
        )
    for row in rows[:12]:
        lines.append(
            f"  {row['state']} {row['kind']} pid={row['pid']} "
            f"owner={row['owner_pid']} {row['job_class']} {row['repo'] or row['cwd']}"
        )
    if len(rows) > 12:
        lines.append(f"  +{len(rows) - 12} jobs; --json lists all owners and commands")
    return "\n".join(lines)
