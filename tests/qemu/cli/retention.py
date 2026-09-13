"""Install/run bounded hourly retention from one pinned checkout.

Activation requires a quiet fleet whose checked-out CLIs implement the same
maintenance interlock. Units and policy live outside all worktrees; disabling
maintenance stops deletions without waiting for a source change to propagate.
"""

from __future__ import annotations

import fcntl
import json
import math
import subprocess
from pathlib import Path
from types import SimpleNamespace

from . import hostlock, maintenance, prune

UNIT = "vibeos-retention"


def policy_path() -> Path:
    return hostlock.HOSTLOCK_DIR / "retention.json"


def read_policy(repo: Path) -> dict:
    value = json.loads(policy_path().read_text())
    if not isinstance(value, dict) or set(value) != {
        "version",
        "owner",
        "retain_gib",
        "min_free_gib",
        "idle_hours",
    }:
        raise ValueError("invalid retention policy fields")
    if value["version"] != 1 or value["owner"] != str(repo.resolve()):
        raise ValueError("retention policy owner/version mismatch")
    for name in ("retain_gib", "min_free_gib", "idle_hours"):
        n = value[name]
        if isinstance(n, bool) or not isinstance(n, (int, float)) or not math.isfinite(n) or n < 0:
            raise ValueError(f"invalid retention {name}")
    if value["idle_hours"] < 1:
        raise ValueError("scheduled retention requires at least one idle hour")
    return value


def verify_fleet(repo: Path) -> None:
    worktrees = prune.git_worktrees(repo)
    if not worktrees:
        raise ValueError("cannot enumerate fleet for maintenance cutover")
    reference = (repo / "tests/qemu/cli/maintenance.py").read_bytes()
    incompatible = []
    for wt in worktrees:
        # Source compatibility is mandatory even for lanes with no current output:
        # they may start a build immediately after this exclusive lock releases.
        try:
            lock_code = (wt / "tests/qemu/cli/maintenance.py").read_bytes()
            main = (wt / "tests/qemu/cli/__main__.py").read_text()
        except OSError:
            incompatible.append(str(wt))
            continue
        if lock_code != reference or "with maintenance.activity(REPO):" not in main:
            incompatible.append(str(wt))
    if incompatible:
        raise ValueError("update maintenance admission before cutover: " + ", ".join(incompatible))
    if prune.live_qemu_worktrees(worktrees) or prune.active_build_worktrees(worktrees):
        raise ValueError("raw/legacy build or QEMU remains active; retry cutover when idle")
    if any(hostlock.HOSTLOCK_DIR.resolve().is_relative_to(wt.resolve()) for wt in worktrees):
        raise ValueError("maintenance controls must be outside every worktree")


def units(repo: Path) -> tuple[str, str]:
    path = str(repo.resolve())
    # systemd expands percent specifiers even inside double quotes.
    escaped = path.replace("\\", "\\\\").replace('"', '\\"').replace("%", "%%")
    service = (
        "[Unit]\nDescription=VibeOS idle generated-output retention\n"
        "[Service]\nType=oneshot\n"
        f'ExecStart="{escaped}/vibeos-cli" retention --run\n'
        "Nice=19\nIOSchedulingClass=idle\n"
        # Busy fleet/kill switch is an expected skipped maintenance opportunity.
        "SuccessExitStatus=2\nTimeoutStartSec=30min\n"
    )
    timer = (
        "[Unit]\nDescription=Hourly VibeOS retention opportunity\n"
        "[Timer]\nOnBootSec=10min\nOnUnitActiveSec=1h\nRandomizedDelaySec=5min\n"
        "[Install]\nWantedBy=timers.target\n"
    )
    return service, timer


def enable(args, repo: Path) -> None:
    root = hostlock.HOSTLOCK_DIR
    root.mkdir(parents=True, exist_ok=True)
    with (root / "maintenance.lock").open("a+") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise ValueError(
                "fleet activity in progress; retry activation after it completes"
            ) from exc
        configured = policy_path().exists()
        if configured:
            read_policy(repo)  # only the current owner can reconfigure/re-enable
        if (root / "maintenance-disabled").exists() and not configured:
            raise ValueError("maintenance-disabled kill switch is present")
        verify_fleet(repo)
        owner = root / "maintenance-owner"
        if owner.exists() and owner.read_text().strip() != str(repo.resolve()):
            raise ValueError("another checkout owns maintenance; refusing to replace it")
        value = {
            "version": 1,
            "owner": str(repo.resolve()),
            "retain_gib": args.retain_gib,
            "min_free_gib": args.min_free_gib,
            "idle_hours": args.idle_hours,
        }
        unit_root = Path.home() / ".config/systemd/user"
        unit_root.mkdir(parents=True, exist_ok=True)
        paths = [unit_root / f"{UNIT}.{suffix}" for suffix in ("service", "timer")]
        for path, expected in zip(paths, units(repo), strict=True):
            if path.exists() and (not configured or path.read_text() != expected):
                raise ValueError(f"retention unit is not owned by this configuration: {path}")
        controls = [policy_path(), owner, root / "maintenance-disabled", *paths]
        previous = {path: path.read_bytes() if path.exists() else None for path in controls}
        try:
            policy_path().write_text(json.dumps(value, indent=2) + "\n")
            read_policy(repo)
            owner.write_text(str(repo.resolve()) + "\n")
            (root / "maintenance-disabled").unlink(missing_ok=True)
            for path, text in zip(paths, units(repo), strict=True):
                path.write_text(text)
            subprocess.run(["systemctl", "--user", "daemon-reload"], check=True)
            subprocess.run(["systemctl", "--user", "enable", "--now", f"{UNIT}.timer"], check=True)
        except Exception:
            # The service also requires this policy, so removing it prevents any
            # new deletion even if systemd partially enabled the timer.
            for path, content in previous.items():
                if content is None:
                    path.unlink(missing_ok=True)
                else:
                    path.write_bytes(content)
            raise
    print(f"Retention enabled from {repo}; policy: {policy_path()}")


def command(args, repo: Path) -> int:
    try:
        if args.enable:
            enable(args, repo)
            return 0
        if args.disable:
            read_policy(repo)  # refuse a different lane's maintenance configuration
            # Global switch first: an already-running prune checks it between dirs.
            hostlock.HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
            (hostlock.HOSTLOCK_DIR / "maintenance-disabled").touch()
            subprocess.run(["systemctl", "--user", "disable", "--now", f"{UNIT}.timer"], check=True)
            print("Retention disabled; maintenance-disabled also stops manual prune.")
            return 0
        policy = read_policy(repo)
        if not args.run:
            print(json.dumps(policy, indent=2))
            return 0
        with maintenance.activity(repo, exclusive=True):
            verify_fleet(repo)  # a newly created legacy lane must stop scheduled deletion
            return prune._prune(
                SimpleNamespace(
                    apply=True,
                    self=True,
                    include_integration=True,
                    retain_gib=policy["retain_gib"],
                    min_free_gib=policy["min_free_gib"],
                    idle_hours=policy["idle_hours"],
                ),
                repo,
            )
    except (OSError, ValueError, subprocess.SubprocessError, maintenance.MaintenanceError) as exc:
        print(f"RETENTION_REFUSED: {exc}")
        return 2
