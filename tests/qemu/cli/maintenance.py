"""Fleet retention interlock. Apply is disabled until an operator pins an owner.

Rollout requires draining older CLI processes before creating maintenance-owner.
The lock is cooperative, not protection against raw tools or older CLI copies.
"""

from __future__ import annotations

import contextlib
import fcntl
from collections.abc import Iterator
from pathlib import Path

from . import hostlock

# Default new commands to protected, including generated-artifact readers.
UNLOCKED = {"prune", "host-status", "disk-status", "land-status", "ps", "kill"}


class MaintenanceError(RuntimeError):
    pass


def check_owner(repo: Path) -> None:
    root = hostlock.HOSTLOCK_DIR
    if (root / "maintenance-disabled").exists():
        raise MaintenanceError("host maintenance-disabled kill switch is present")
    try:
        owner = (root / "maintenance-owner").read_text().strip()
    except OSError as exc:
        raise MaintenanceError(
            "prune disabled: drain old clients, then pin maintenance-owner"
        ) from exc
    if not owner or Path(owner) != repo.resolve():
        raise MaintenanceError(f"prune must run from pinned owner {owner!r}")


@contextlib.contextmanager
def activity(repo: Path, *, exclusive: bool = False) -> Iterator[None]:
    root = hostlock.HOSTLOCK_DIR
    root.mkdir(parents=True, exist_ok=True)
    with (root / "maintenance.lock").open("a+") as stream:
        mode = fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH
        try:
            fcntl.flock(stream, mode | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise MaintenanceError(
                "fleet activity/cleanup in progress; retry after it completes"
            ) from exc
        if exclusive:
            check_owner(repo)
        yield
