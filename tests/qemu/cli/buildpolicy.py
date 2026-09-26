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

POLICY_NAME = "build-policy.json"
EXAMPLE_NAME = "build-policy.example.json"


def repo_root() -> Path:
    """This module is <root>/tests/qemu/cli/buildpolicy.py."""
    return Path(__file__).resolve().parents[3]


def main_worktree(repo: Path) -> Path:
    """The MAIN worktree, which every linked worktree resolves to identically.

    The policy is ONE host-wide number, so it cannot live per-worktree -- eight lanes
    with eight different capacities is not a cap. It also must not live under /tmp:
    that is where it lived when a reboot silently deleted it, leaving `_ninja_jobs()`
    on its uncapped fallback while every lane believed it was throttled. A linked
    worktree's `.git` is a FILE reading `gitdir: <main>/.git/worktrees/<name>`, so the
    shared root is recoverable with one small read and no subprocess.
    """
    dot = repo / ".git"
    if dot.is_dir():
        return repo
    try:
        line = dot.read_text().strip()
    except OSError:
        return repo
    if line.startswith("gitdir:"):
        gitdir = Path(line.removeprefix("gitdir:").strip())
        for parent in gitdir.parents:
            if parent.name == ".git":
                return parent.parent
    return repo


def durable_dir() -> Path:
    override = os.environ.get("VIBEOS_BUILD_POLICY_DIR")
    return Path(override) if override else main_worktree(repo_root())


def heal(root: Path | None = None) -> None:
    """Seed a missing policy from the tracked, deliberately conservative example.

    Self-healing is the DEVXP path (a fresh clone builds capped without anyone
    running a command) and the belt to durability's braces. It seeds the EXAMPLE,
    never `TRIAL`: an unattended box that lost its policy should come back slow,
    not fast. `host-policy --apply` is how you opt into the tuned capacity, and
    that choice then persists because the file no longer lives in /tmp.

    Deliberately silent when it can do nothing -- a missing example or an
    unwritable root leaves `read()` returning None, which is the pre-existing
    no-policy behaviour, not a new failure mode.
    """
    from . import hostlock

    target = (root if root is not None else hostlock.POLICY_DIR) / POLICY_NAME
    if target.exists():
        return
    example = repo_root() / EXAMPLE_NAME
    if not example.is_file():
        return
    try:
        seed = json.loads(example.read_text())
    except (OSError, ValueError):
        return
    if not isinstance(seed, dict) or seed.get("version") != VERSION:
        return
    text = json.dumps(seed, indent=2) + "\n"
    try:
        target.parent.mkdir(parents=True, exist_ok=True)
        temp = target.parent / f"{POLICY_NAME}.{os.getpid()}.tmp"
        temp.write_text(text)
        temp.replace(target)
    except OSError:
        return
    mirror(text)


def mirror(text: str) -> None:
    """Copy the active policy into the OLD /tmp location for lanes still on v1.

    Transition shim, and the only reason it is safe to move the file at all. A lane on
    older tooling reads HOSTLOCK_DIR and nothing else; if we simply stopped writing
    there it would find no policy and fall through to the UNCAPPED branch -- the exact
    failure this whole change exists to remove, inflicted on every lane that had not yet
    merged. Old clients only ever READ this path, so an extra writer cannot confuse them.
    Delete once `host-policy` reports no stale worktrees.
    """
    from . import hostlock

    if hostlock.POLICY_DIR == hostlock.HOSTLOCK_DIR:
        return
    try:
        hostlock.HOSTLOCK_DIR.mkdir(parents=True, exist_ok=True)
        temp = hostlock.HOSTLOCK_DIR / f"{POLICY_NAME}.{os.getpid()}.mirror"
        temp.write_text(text)
        temp.replace(hostlock.HOSTLOCK_DIR / POLICY_NAME)
    except OSError:
        return


class PolicyError(RuntimeError):
    """Infrastructure refusal, never a code regression."""


def read(root: Path) -> dict | None:
    path = root / POLICY_NAME
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

        manager = session(hostlock.POLICY_DIR)
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


def stale_worktrees(repo: Path) -> list[str]:
    """Worktrees whose tooling predates the durable policy. ADVISORY, not a blocker.

    It was a hard refusal when an old client had no policy support at all, so a mixed
    fleet meant some lanes were genuinely uncapped. That is no longer the shape: a v1
    lane reads the /tmp path, `mirror()` keeps writing it, and it is capped correctly.
    Refusing the cutover for them would leave the whole host on the conservative seed
    just because a PAUSED lane has not merged yet -- a real cost to buy nothing.
    Reported everywhere so the staleness stays visible rather than becoming permanent.
    """
    cp = subprocess.run(
        ["git", "worktree", "list", "--porcelain"],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    )
    stale = []
    for line in cp.stdout.splitlines():
        if not line.startswith("worktree "):
            continue
        tree = Path(line.removeprefix("worktree "))
        source = tree / "tests/qemu/cli/hostlock.py"
        if not source.is_file() or "HOST_POLICY_VERSION = 2" not in source.read_text():
            stale.append(str(tree))
    return stale


def blockers(repo: Path) -> list[str]:
    """HARD cutover blockers only: live work that a capacity change would race."""
    reasons: list[str] = []
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
        path = root / POLICY_NAME
        temp = root / f"build-policy.{os.getpid()}.tmp"
        try:
            text = json.dumps(value, indent=2) + "\n"
            temp.write_text(text)
            temp.replace(path)
            mirror(text)
        finally:
            temp.unlink(missing_ok=True)


def command(args, repo: Path) -> int:
    from . import hostlock

    root = hostlock.POLICY_DIR
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
                "stale_worktrees": stale_worktrees(repo),
            },
            indent=2,
        )
    )
    return 0
