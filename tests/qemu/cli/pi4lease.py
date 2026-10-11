"""Who may drive the Pi 4 bench right now.

There is one physical board. A lane that power-cycles it, restages its payload or
types into it while another lane is mid-boot turns both verdicts into noise, so
the bench has a lease: a JSON record of the owning worktree, a reason and an
expiry, written under a flock in the host lock directory (ephemeral like every
other lock there - a reboot frees the bench).

Stdlib only and runnable as a script, because the bench shell scripts call it
before every action that changes the board's state:

    python3 tests/qemu/cli/pi4lease.py check   # exit 75 if another lane holds it
    python3 tests/qemu/cli/pi4lease.py touch   # check, then take or renew
    python3 tests/qemu/cli/pi4lease.py show

`vibeos-cli pi4 lease` is the front door for people; this is the mechanism.
Ownership is the worktree a command runs from (VIBEOS_PI4_OWNER overrides), so
every agent in one lane shares the lease and other lanes are refused.
"""

from __future__ import annotations

import contextlib
import fcntl
import json
import os
import subprocess
import sys
import time
from collections.abc import Iterator
from pathlib import Path

DEFAULT_MINUTES = 30
REFUSED = 75  # EX_TEMPFAIL; pico.sh already uses 3 for "unreachable"


BOARDS = ("pi4", "pi5", "optiplex")


def board() -> str:
    """Which bench: VIBEOS_BENCH_BOARD (tools/pi5/env.sh sets pi5), default pi4.
    Each board has its own lease; holding one says nothing about the other."""
    name = os.environ.get("VIBEOS_BENCH_BOARD", "pi4")
    if name not in BOARDS:
        raise ValueError(f"VIBEOS_BENCH_BOARD={name!r}: choose from {', '.join(BOARDS)}")
    return name


def lock_dir() -> Path:
    return Path(os.environ.get("VIBEOS_HOSTLOCK_DIR", "/tmp/vibeos-qemu-locks/host"))


# Benches that share hardware share a lease. The OptiPlex took over the Pi 4's
# plug and Pico (picousb5) on 2026-10-09, and tools/pi4's defaults still point
# at both, so a lane running tools/pi4 power-cycles the OptiPlex: it must
# contend for the OptiPlex's lease, not a separate pi4 one nothing else checks.
LEASE_OF = {"pi4": "optiplex"}


def lease_name() -> str:
    return LEASE_OF.get(board(), board())


def lease_file() -> Path:
    return lock_dir() / f"{lease_name()}-lease.json"


def owner_of(cwd: Path | None = None) -> str:
    """The lane identity: VIBEOS_PI4_OWNER, else the worktree top level."""
    forced = os.environ.get("VIBEOS_PI4_OWNER")
    if forced:
        return forced
    try:
        top = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=10,
        ).stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        top = ""
    return top or str((cwd or Path.cwd()).resolve())


@contextlib.contextmanager
def _locked() -> Iterator[None]:
    lock_dir().mkdir(parents=True, exist_ok=True)
    fd = os.open(str(lock_dir() / f"{lease_name()}-lease.lock"), os.O_RDWR | os.O_CREAT, 0o666)
    try:
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        os.close(fd)


def current(now: float | None = None) -> dict:
    """The live lease, or {} when nobody holds one (absent, unreadable or expired)."""
    try:
        lease = json.loads(lease_file().read_text())
    except (OSError, ValueError):
        return {}
    if not isinstance(lease, dict) or lease.get("expires", 0) <= (now or time.time()):
        return {}
    return lease


def describe(lease: dict, now: float | None = None) -> str:
    if not lease:
        return "free"
    left = max(0, int(lease["expires"] - (now or time.time())))
    why = f" - {lease['why']}" if lease.get("why") else ""
    return f"held by {lease['owner']} for {left // 60}m{left % 60:02d}s more{why}"


def take(owner: str, minutes: float = DEFAULT_MINUTES, why: str = "") -> tuple[bool, dict]:
    """Take or renew. Returns (granted, lease); on refusal the lease is the holder's.
    A renewal never shortens a longer lease the owner already holds."""
    with _locked():
        now = time.time()
        held = current(now)
        if held and held.get("owner") != owner:
            return False, held
        expires = max(now + minutes * 60, held.get("expires", 0) if held else 0)
        lease = {
            "owner": owner,
            "why": why or (held.get("why", "") if held else ""),
            "since": held.get("since", now) if held else now,
            "expires": expires,
            "pid": os.getppid(),
        }
        tmp = lease_file().with_suffix(".tmp")
        tmp.write_text(json.dumps(lease))
        tmp.replace(lease_file())
        return True, lease


def drop(owner: str, force: bool = False) -> tuple[bool, dict]:
    """Release. Refuses another owner's lease unless forced (returns the holder)."""
    with _locked():
        held = current()
        if held and held.get("owner") != owner and not force:
            return False, held
        lease_file().unlink(missing_ok=True)
        return True, held


def check(owner: str) -> tuple[bool, dict]:
    held = current()
    return (not held or held.get("owner") == owner), held


def main(argv: list[str]) -> int:
    verb = argv[1] if len(argv) > 1 else "show"
    owner = owner_of()
    if verb == "show":
        print(describe(current()))
        return 0
    if verb in ("check", "touch"):
        ok, held = check(owner)
        if ok and verb == "touch":
            minutes = float(os.environ.get("VIBEOS_PI4_LEASE_MINUTES", DEFAULT_MINUTES))
            ok, held = take(owner, minutes)
        if not ok:
            print(
                f"{board()} bench is {describe(held)}.\n"
                "  Wait for it, or coordinate with that lane; "
                f"`vibeos-cli {board()} status` shows it.",
                file=sys.stderr,
            )
            return REFUSED
        return 0
    print("usage: pi4lease.py show|check|touch", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
