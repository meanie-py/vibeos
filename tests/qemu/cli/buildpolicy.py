"""Host-wide build capacity, changed only at a drained fleet boundary."""

from __future__ import annotations

import fcntl
import json
import os
import subprocess
from contextlib import ExitStack, contextmanager
from functools import wraps
from pathlib import Path

VERSION = 1
_guards: dict = {}
TRIAL = {"version": VERSION, "leases": 6, "max_builds": 3, "ninja_jobs": 6, "cargo_jobs": 1}


class PolicyError(RuntimeError):
    """Infrastructure refusal, never a code regression."""


def read(root: Path) -> dict | None:
    path = root / "build-policy.json"
    if not path.exists() and not path.is_symlink():
        return None
    try:
        value = json.loads(path.read_text())
    except (OSError, ValueError) as exc:
        raise PolicyError(f"unreadable host build policy: {path}: {exc}") from exc
    if (
        not isinstance(value, dict)
        or set(value) != set(TRIAL)
        or any(type(v) is not int or v <= 0 for v in value.values())
        or value["version"] != VERSION
        or value["leases"] % value["max_builds"]
        or value["cargo_jobs"] != 1
    ):
        raise PolicyError(f"invalid host build policy: {path}")
    return value


@contextmanager
def session(root: Path):
    root.mkdir(parents=True, exist_ok=True)
    with (root / "build-policy.lock").open("a+") as lock:
        fcntl.flock(lock, fcntl.LOCK_SH)
        yield read(root)


def guard(fn):
    """Pin capacity from ticket creation until the final permit is released."""

    @wraps(fn)
    def acquire(*args, **kwargs):
        from . import hostlock

        manager = session(hostlock.HOSTLOCK_DIR)
        policy = manager.__enter__()
        try:
            if policy:
                if "leases" in kwargs and kwargs["leases"] != policy["leases"]:
                    raise PolicyError("per-client lease override conflicts with host policy")
                kwargs["leases"] = policy["leases"]
                kind = args[0] if args else kwargs.get("kind", "")
                if fn.__name__ == "acquire_cpu_share" and kind.startswith("build:"):
                    kwargs["n"] = policy["leases"] // policy["max_builds"]
            result = fn(*args, **kwargs)
            if result is None:
                if policy:
                    raise PolicyError("CPU admission timed out; host policy forbids unlocked work")
                return None
            fds = result if isinstance(result, list) else [result]
            if fds:
                _guards[fds[-1]] = manager
                manager = None
            return result
        finally:
            if manager is not None:
                manager.__exit__(None, None, None)

    return acquire


def take_guard(fd: int):
    return _guards.pop(fd, None)


def require_admission(root: Path) -> None:
    if (root / "build-policy.paused").exists():
        raise PolicyError(
            "CPU admission paused for fleet drain; use host-policy --resume after cutover"
        )


def blockers(repo: Path) -> list[str]:
    """Inspect source capability and old processes; never modify sibling trees."""
    cp = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    )
    reasons = []
    for line in cp.stdout.splitlines():
        if not line.startswith("worktree "):
            continue
        tree = Path(line.removeprefix("worktree "))
        source = tree / "tests/qemu/cli/hostlock.py"
        if not source.is_file() or "HOST_POLICY_VERSION = 1" not in source.read_text():
            reasons.append(f"update policy-unaware worktree: {tree}")
    for p in Path("/proc").iterdir():
        if not p.name.isdigit() or int(p.name) == os.getpid():
            continue
        try:
            cmd = (p / "cmdline").read_bytes().split(b"\0")
            comm = (p / "comm").read_text().strip()
        except FileNotFoundError:
            continue
        if b"tests.qemu.cli" in cmd or comm in {
            "ninja",
            "cargo",
            "rustc",
            "clang",
            "clang++",
            "ld.lld",
            "cmake",
            "make",
        }:
            reasons.append(f"drain artifact process: pid={p.name} ({comm})")
    return reasons


def activate(repo: Path, root: Path, *, rollback: bool = False) -> None:
    root.mkdir(parents=True, exist_ok=True)
    with (root / "build-policy.lock").open("a+") as lock, ExitStack() as slots:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            for path in root.glob("cpuheavy-*"):
                held = slots.enter_context(path.open("a+"))
                fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise RuntimeError("policy cutover refused: active or queued CPU clients") from exc
        reasons = blockers(repo)
        if reasons:
            raise RuntimeError("policy cutover refused:\n" + "\n".join(reasons))
        value = dict(TRIAL)
        if rollback:
            value.update(leases=4, max_builds=2)
        if value["max_builds"] * value["ninja_jobs"] > (os.cpu_count() or 1):
            raise RuntimeError("trial job budget exceeds visible CPU count")
        path = root / "build-policy.json"
        temp = root / f"build-policy.{os.getpid()}.tmp"
        try:
            temp.write_text(json.dumps(value, indent=2) + "\n")
            temp.replace(path)
        finally:
            temp.unlink(missing_ok=True)


def command(args, repo: Path) -> int:
    from . import hostlock

    root = hostlock.HOSTLOCK_DIR
    if args.pause or args.resume:
        root.mkdir(parents=True, exist_ok=True)
        with (root / "build-policy-control.lock").open("a+") as control:
            fcntl.flock(control, fcntl.LOCK_EX)
            if args.pause:
                (root / "build-policy.paused").touch()
            else:
                (root / "build-policy.paused").unlink(missing_ok=True)
    if args.apply or args.rollback:
        try:
            activate(repo, root, rollback=args.rollback)
        except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
            print(f"BUILD_POLICY_REFUSED: {exc}")
            return 1
    print(
        json.dumps(
            {
                "active": read(root),
                "trial": TRIAL,
                "paused": (root / "build-policy.paused").exists(),
                "blockers": blockers(repo),
            },
            indent=2,
        )
    )
    return 0
