"""Fleet retention interlock. Apply is disabled until an operator pins an owner.

Rollout requires draining older CLI processes before creating maintenance-owner.
The lock is cooperative, not protection against raw tools or older CLI copies.

The flock lives here because it must be held for the life of the command; the owner and
kill-switch decision is native (`tools/vibeos-build/src/retention.rs`).
"""

from __future__ import annotations

import contextlib
import fcntl
from collections.abc import Iterator
from pathlib import Path

from tools import build_infra_compat as native

from . import hostlock

# The interlock protocol this file implements: shared flock on maintenance.lock for every
# command, exclusive for cleanup, plus the owner pin and the kill switch. Scheduled
# retention accepts a lane only if its copy of this file declares the same protocol (or is
# byte-identical), so editing this file no longer blocks retention until every lane merges.
# Bump it ONLY when the protocol itself changes.
INTERLOCK_PROTOCOL = 1

# Default new commands to protected, including generated-artifact readers.
UNLOCKED = {"prune", "host-status", "disk-status", "land-status", "ps", "kill"}


class MaintenanceError(RuntimeError):
    pass


def check_owner(repo: Path) -> None:
    refusal = native.call(
        "retention_owner_refusal",
        {"dir": str(hostlock.HOSTLOCK_DIR), "repo": str(repo.resolve())},
    )
    if refusal:
        raise MaintenanceError(refusal)


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
