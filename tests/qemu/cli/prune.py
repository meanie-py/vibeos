"""`vibeos-cli prune` — reclaim disk by GC'ing idle worktrees' build + Rust target dirs.

Per the build-cache plan's disk-first reframing (docs/engineering/build-cache-and-determinism-
plan.md): build trees are *ephemeral scratch* — the shared cache (ccache/sccache) rehydrates a
pruned tree fast, so an idle worktree's 26-42 G build+target footprint can be reclaimed and
regenerated on demand. Measured ~150 G across 16 worktrees, most of it idle.

SAFETY (manual + gated, by user policy 2026-06-20):
- **dry-run by default** — prints what WOULD be freed; deletes nothing without `--apply`.
- never a worktree with a **live QEMU** (reads the same /tmp/vibeos-qemu-locks/*.json that
  `qemu/ps` uses), so a running smoke/boot is never yanked out from under itself;
- never a worktree with a **running ninja/cargo/rustc/clang/lld** (scans /proc cwd);
- never the **current** worktree unless `--self`;
- only ever deletes the build-`<arch>`/ and Rust target dirs — never source, never the
  worktree itself.
"""

from __future__ import annotations

import math
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from . import canon, fsutil, maintenance, queue

# Same global lock dir vm.py / qemu/start.sh / qemu/ps use.
LOCK_DIR = Path("/tmp/vibeos-qemu-locks")

# Per-worktree dirs we treat as disposable build scratch (relative to a worktree root).
PRUNE_RELDIRS = (
    "build-i386",
    "build-amd64",
    "build-arm64",
    "sdk/lib/rust/target",
    "ntoskrnl/rust-kernel-target",
)

# Process names that mean "a build is in flight" — never prune their worktree.
_BUILD_PROCS = {
    "ninja",
    "cargo",
    "rustc",
    "cc1",
    "cc1plus",
    "clang",
    "clang++",
    "lld",
    "ld.lld",
    "cmake",
    "make",
    "mcopy",
    "mkramdisk.sh",
}


@dataclass
class Candidate:
    worktree: Path
    dirs: list[tuple[Path, int]]  # (path, size_bytes) that exist
    eligible: bool
    skip_reason: str | None

    @property
    def total(self) -> int:
        return sum(sz for _, sz in self.dirs)


def _pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _is_qemu(pid: int) -> bool:
    try:
        return "qemu" in (Path(f"/proc/{pid}/comm").read_text())
    except OSError:
        return False


def git_worktrees(repo: Path) -> list[Path]:
    """All worktree roots from `git worktree list --porcelain`."""
    try:
        out = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            cwd=repo,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return []
    return [
        Path(line[len("worktree ") :]) for line in out.splitlines() if line.startswith("worktree ")
    ]


def _attribute(path_str: str, worktrees: list[Path]) -> Path | None:
    """Which worktree (if any) contains this absolute path. Longest match wins so a
    nested worktree isn't mis-attributed to its parent."""
    try:
        p = Path(path_str).resolve()
    except (OSError, ValueError):
        return None
    best: Path | None = None
    for wt in worktrees:
        try:
            wtr = wt.resolve()
        except OSError:
            continue
        if (p == wtr or wtr in p.parents) and (best is None or len(str(wtr)) > len(str(best))):
            best = wt
    return best


def live_qemu_worktrees(worktrees: list[Path]) -> set[Path]:
    """Worktrees with a live QEMU, via the global lock files (the `ps` source of truth)."""
    import json

    live: set[Path] = set()
    if not LOCK_DIR.is_dir():
        return live
    for lf in LOCK_DIR.glob("*.json"):
        try:
            info = json.loads(lf.read_text())
        except (OSError, ValueError):
            continue
        qpid = info.get("qemu_pid")
        if not (qpid and _pid_alive(qpid) and _is_qemu(qpid)):
            continue
        # Any path-bearing field attributes the run to its worktree.
        for key in ("live_log", "work_dir", "manifest_path"):
            val = info.get(key)
            if val and (wt := _attribute(str(val), worktrees)):
                live.add(wt.resolve())
                break
    return live


def active_build_worktrees(worktrees: list[Path]) -> set[Path]:
    """Worktrees with a running build process (cwd under the worktree)."""
    active: set[Path] = set()
    proc = Path("/proc")
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        try:
            comm = (entry / "comm").read_text().strip()
        except OSError:
            continue
        if comm not in _BUILD_PROCS:
            continue
        try:
            cwd = os.readlink(entry / "cwd")
        except OSError:
            # An unattributable live compiler is not evidence of an idle tree.
            if entry.exists():
                active.update(wt.resolve() for wt in worktrees)
            continue
        # Bind-mount builds report /tmp/vibeos, not their physical worktree.
        # Protect the fleet conservatively rather than misclassifying an orphan
        # compiler as idle after its CLI parent (and shared lease) has died.
        canonical = Path(canon.canon_path())
        if Path(cwd) == canonical or canonical in Path(cwd).parents:
            active.update(wt.resolve() for wt in worktrees)
            continue
        if wt := _attribute(cwd, worktrees):
            active.add(wt.resolve())
    return active


def classify(
    worktree: Path,
    *,
    current: Path,
    live_qemu: set[Path],
    active_build: set[Path],
    allow_self: bool,
    min_age_hours: float,
    now: float,
    newest_mtime: float | None,
    include_integration: bool = False,
) -> str | None:
    """Pure eligibility decision → skip reason string, or None if prunable. All
    state (live sets, `now`, the tree's `newest_mtime`) is injected so this is
    deterministic + unit-testable. Gate order = most-protective first."""
    wtr = worktree.resolve()
    if wtr == current.resolve() and not allow_self:
        return "current worktree (use --self)"
    # Preserve conductor scratch for ordinary manual pruning. Scheduled retention
    # explicitly includes it, subject to the same idle window and activity lock.
    if queue.is_integration_dirname(wtr.name) and not allow_self and not include_integration:
        return "merge-queue integration tree (its warm caches gate every batch)"
    if wtr in live_qemu:
        return "live QEMU running"
    if wtr in active_build:
        return "build in flight"
    if (
        min_age_hours > 0
        and newest_mtime is not None
        and (now - newest_mtime) < min_age_hours * 3600
    ):
        return f"touched < {min_age_hours:g}h ago"
    return None


def _dir_size(path: Path) -> int:
    try:
        return fsutil.disk_usage_bytes(path)
    except OSError as exc:
        raise maintenance.MaintenanceError(f"cannot measure {path}: {exc}") from exc


def _newest_mtime(worktree: Path) -> float | None:
    """Newest descendant mtime: rewriting an object need not touch its root dir."""
    best: float | None = None
    for rel in PRUNE_RELDIRS:
        d = worktree / rel
        if d.exists():
            m = d.stat().st_mtime
            best = m if best is None else max(best, m)
            for parent, dirs, files in os.walk(d, followlinks=False):
                for name in dirs + files:
                    try:
                        m = (Path(parent) / name).lstat().st_mtime
                    except FileNotFoundError:
                        continue
                    best = max(best, m)
    return best


def _prunable_dirs(worktree: Path) -> list[tuple[Path, int]]:
    return [
        (worktree / rel, _dir_size(worktree / rel))
        for rel in PRUNE_RELDIRS
        if (worktree / rel).exists()
    ]


def _fmt(n: int) -> str:
    g = n / 1024**3
    return f"{g:6.1f} G" if g >= 0.1 else f"{n / 1024**2:6.0f} M"


def prune(args, repo: Path) -> int:
    try:
        if not getattr(args, "apply", False):
            return _prune(args, repo)
        with maintenance.activity(repo, exclusive=True):
            return _prune(args, repo)
    except maintenance.MaintenanceError as exc:
        print(f"PRUNE_REFUSED: {exc}")
        return 2


def select_budget(candidates: list[Candidate], budget: float | None) -> list[Candidate]:
    """Keep protected outputs even if they alone exceed the requested budget."""
    if budget is None:
        return candidates
    if not math.isfinite(budget) or budget < 0:
        raise ValueError("--retain-gib must be finite and nonnegative")
    remaining = sum(c.total for c in candidates)
    result = []
    for c in sorted(candidates, key=lambda c: c.total, reverse=True):
        if c.eligible and remaining <= budget * 1024**3:
            c = Candidate(c.worktree, c.dirs, False, "within retention budget")
        elif c.eligible:
            remaining -= c.total
        result.append(c)
    return result


def _prune(args, repo: Path) -> int:
    import time

    worktrees = git_worktrees(repo)
    if not worktrees:
        print("prune: no git worktrees found")
        return 1
    if getattr(args, "apply", False):
        control = maintenance.hostlock.HOSTLOCK_DIR.resolve()
        if any(control.is_relative_to(wt.resolve()) for wt in worktrees):
            raise maintenance.MaintenanceError(
                "host maintenance controls must be outside all worktrees"
            )
    live = live_qemu_worktrees(worktrees)
    active = active_build_worktrees(worktrees)
    now = time.time()
    min_age = float(getattr(args, "idle_hours", 0) or 0)
    if (
        getattr(args, "retain_gib", None) is not None
        or getattr(args, "min_free_gib", None) is not None
    ) and min_age == 0:
        min_age = 24  # bounded fleet retention must not reclaim newly completed work
    if not math.isfinite(min_age) or min_age < 0:
        raise maintenance.MaintenanceError("--idle-hours must be finite and nonnegative")

    candidates: list[Candidate] = []
    for wt in worktrees:
        dirs = _prunable_dirs(wt)
        if not dirs:
            continue  # nothing to reclaim
        reason = classify(
            wt,
            current=repo,
            live_qemu=live,
            active_build=active,
            allow_self=getattr(args, "self", False),
            include_integration=getattr(args, "include_integration", False),
            min_age_hours=min_age,
            now=now,
            newest_mtime=_newest_mtime(wt) if min_age > 0 else None,
        )
        candidates.append(Candidate(wt, dirs, eligible=reason is None, skip_reason=reason))

    budget = getattr(args, "retain_gib", None)
    min_free = getattr(args, "min_free_gib", None)
    if min_free is not None:
        if not math.isfinite(min_free) or min_free < 0:
            raise maintenance.MaintenanceError("--min-free-gib must be finite and nonnegative")
        device = repo.stat().st_dev
        if any(d.stat().st_dev != device for c in candidates for d, _ in c.dirs):
            raise maintenance.MaintenanceError(
                "headroom retention requires outputs on one filesystem"
            )
        shortfall = max(0, min_free * 1024**3 - shutil.disk_usage(repo).free)
        headroom_budget = max(0, sum(c.total for c in candidates) - shortfall) / 1024**3
        budget = min(budget, headroom_budget) if budget is not None else headroom_budget

    try:
        candidates = select_budget(candidates, budget)
    except ValueError as exc:
        print(f"PRUNE_REFUSED: {exc}")
        return 2
    candidates.sort(key=lambda c: c.total, reverse=True)
    reclaim = sum(c.total for c in candidates if c.eligible)
    held = sum(c.total for c in candidates if not c.eligible)

    apply = getattr(args, "apply", False)
    header = "PRUNING" if apply else "DRY-RUN (nothing deleted; pass --apply)"
    print(f"vibeos-cli prune — {header}\n")
    for c in candidates:
        mark = "PRUNE " if c.eligible else "keep  "
        note = "" if c.eligible else f"  ({c.skip_reason})"
        print(f"  {mark} {_fmt(c.total)}  {c.worktree.name}{note}")
    print()
    print(f"  reclaimable: {_fmt(reclaim)}   held (in use/recent): {_fmt(held)}")
    if budget is not None and held > budget * 1024**3:
        print("  budget cannot be met: protected outputs exceed it; protection takes precedence")

    if not apply:
        print("\n  → re-run with --apply to delete the PRUNE rows.")
        return 0

    freed = 0
    failures = 0
    for c in candidates:
        if not c.eligible:
            continue
        for d, sz in c.dirs:
            maintenance.check_owner(repo)  # kill switch rechecked for every directory
            # Never follow an ancestor symlink outside the selected worktree.
            if d.resolve() != c.worktree.resolve() / d.relative_to(c.worktree):
                raise maintenance.MaintenanceError(f"symlinked output path: {d}")
            # Also catch raw/legacy activity visible at deletion time. Those clients
            # are NOT race-safe; drain them before enabling the owner stamp.
            if live_qemu_worktrees(worktrees) or active_build_worktrees(worktrees):
                raise maintenance.MaintenanceError(
                    "live QEMU/build detected; nothing further removed"
                )
            try:
                shutil.rmtree(d)
                freed += sz
                print(f"  removed {d}")
            except OSError as e:
                failures += 1
                print(f"  FAILED {d}: {e}")
    print(f"\n  freed {_fmt(freed)}.")
    return 1 if failures else 0
