"""Compatibility bindings for native, flock-protected physical bench leases."""

from __future__ import annotations

import os
import sys
from pathlib import Path

# Shell device authorities also invoke this adapter directly.
REPO = Path(__file__).resolve().parents[3]
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools import bench_infra_compat as native  # noqa: E402

DEFAULT_MINUTES = 30
REFUSED = 75
BOARDS = ("pi4", "pi5", "optiplex")
LEASE_OF = {"pi4": "optiplex"}


def _call(op, **kwargs):
    return native.call("bench_lease_" + op, {"env": dict(os.environ), **kwargs}, Path.cwd())


def board():
    return _call("board")


def lock_dir():
    return Path(_call("lock_dir"))


def lease_name():
    return _call("lease_name")


def lease_file():
    return Path(_call("lease_file"))


def owner_of(cwd=None):
    return _call("owner_of", cwd=str(cwd or Path.cwd()))


def current(now=None):
    return _call("current", now=now)


def describe(lease, now=None):
    return _call("describe", lease=lease, now=now)


def take(owner, minutes=DEFAULT_MINUTES, why=""):
    return tuple(_call("take", owner=owner, minutes=minutes, why=why, pid=os.getppid()))


def drop(owner, force=False):
    return tuple(_call("drop", owner=owner, force=force))


def check(owner):
    return tuple(_call("check", owner=owner))


def main(argv):
    result = _call("main", verb=argv[1] if len(argv) > 1 else "show")
    sys.stdout.write(result["stdout"])
    sys.stderr.write(result["stderr"])
    return result["code"]


if __name__ == "__main__":
    sys.exit(main(sys.argv))
