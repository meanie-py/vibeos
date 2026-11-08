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
import shutil
import time
from dataclasses import dataclass
from pathlib import Path

from tools import build_infra_compat as native

from . import canon, hostlock, maintenance, trim

# The decisions are native: tools/vibeos-build/src/prune.rs (which trees are live or building,
# the protection gates, sizes, the retention budget). This module keeps the maintenance lock,
# the printout and the delete loop, which rechecks liveness before every directory.


@dataclass
class Candidate:
    worktree: Path
    dirs: list[tuple[Path, int]]  # (path, size_bytes) that exist
    eligible: bool
    skip_reason: str | None

    @property
    def total(self) -> int:
        return sum(sz for _, sz in self.dirs)


def _call(op: str, **kwargs):
    try:
        return native.call("prune_" + op, kwargs)
    except native.NativeError as exc:
        if exc.kind == "maintenance":
            raise maintenance.MaintenanceError(str(exc)) from exc
        raise


def _num(value):
    """JSON cannot spell inf/nan, and refusing them is the native validator's job."""
    return None if value is None else repr(float(value))


def git_worktrees(repo: Path) -> list[Path]:
    """All worktree roots from `git worktree list --porcelain`."""
    return [Path(p) for p in _call("git_worktrees", repo=repo)]


def live_qemu_worktrees(worktrees: list[Path]) -> set[Path]:
    """Worktrees with a live QEMU, via the global lock files (the `ps` source of truth)."""
    return {Path(p) for p in _call("live_qemu", worktrees=worktrees)}


def active_build_worktrees(worktrees: list[Path]) -> set[Path]:
    """Worktrees with a running build process (cwd, or canonical bind source, under it)."""
    return {
        Path(p) for p in _call("active_builds", worktrees=worktrees, canonical=canon.canon_path())
    }


def _fmt(n: int) -> str:
    g = n / 1024**3
    return f"{g:6.1f} G" if g >= 0.1 else f"{n / 1024**2:6.0f} M"


def prune(args, repo: Path) -> int:
    if getattr(args, "trim", False):
        return _trim(args, repo)
    try:
        if not getattr(args, "apply", False):
            return _prune(args, repo)
        # Every running vibeos-cli command holds the maintenance lock SHARED, so an exclusive
        # prune fails whenever anything at all is building or gating. --wait-minutes polls
        # for the quiet moment rather than leaving the caller to hand-roll a retry loop.
        deadline = time.monotonic() + 60 * float(getattr(args, "wait_minutes", 0) or 0)
        while True:
            try:
                with maintenance.activity(repo, exclusive=True):
                    return _prune(args, repo)
            except maintenance.MaintenanceError as exc:
                if "in progress" not in str(exc) or time.monotonic() >= deadline:
                    raise
            time.sleep(30)
    except maintenance.MaintenanceError as exc:
        print(f"PRUNE_REFUSED: {exc}")
        return 2


def _trim(args, repo: Path) -> int:
    """`--trim`: superseded cargo units + idle ad-hoc target trees, every worktree. Needs no
    quiet fleet and no pinned owner, unlike a full prune: nothing it removes is reachable by
    any tree's current build (see trim.py), so it costs no lane a warm build. The host kill
    switch still stops it."""
    worktrees = git_worktrees(repo)
    if not worktrees:
        print("prune: no git worktrees found")
        return 1
    idle = float(getattr(args, "idle_hours", 0) or 24)
    if not math.isfinite(idle) or idle < 1:
        print("PRUNE_REFUSED: --trim needs --idle-hours >= 1")
        return 2
    apply = getattr(args, "apply", False)
    header = "TRIMMING" if apply else "DRY-RUN (nothing deleted; pass --apply)"
    print(f"vibeos-cli prune --trim — {header}  (untouched >= {idle:g}h)\n")
    plans = [(wt, trim.plan_worktree(wt, idle_hours=idle)) for wt in worktrees]
    plans.sort(key=lambda x: x[1].total, reverse=True)
    for wt, plan in plans:
        if plan.paths:
            print(
                f"  {_fmt(plan.total)}  {wt.name}  "
                f"({plan.superseded_units} superseded units, {plan.adhoc_dirs} ad-hoc dirs, "
                f"{plan.dead_targets} dead custom targets, {plan.retired_dirs} retired layouts)"
            )
    staging = trim.plan_port_staging(idle_hours=idle)
    if staging.paths:
        print(
            f"  {_fmt(staging.total)}  {trim.PORT_STATE_ROOT}  "
            f"({staging.adhoc_dirs} idle port cargo targets)"
        )
        plans.append((trim.PORT_STATE_ROOT, staging))
    total = sum(p.total for _, p in plans)
    print(f"\n  trimmable: {_fmt(total)}")
    if not apply:
        print("\n  → re-run with --apply to delete.")
        return 0
    if (hostlock.HOSTLOCK_DIR / "maintenance-disabled").exists():
        print("PRUNE_REFUSED: host maintenance-disabled kill switch is present")
        return 2
    freed = failures = 0
    for wt, plan in plans:
        f, n = trim.apply(wt, plan)
        freed += f
        failures += n
    print(f"  freed {_fmt(freed)}.")
    return 1 if failures else 0


def _prune(args, repo: Path) -> int:
    worktrees = git_worktrees(repo)
    if not worktrees:
        print("prune: no git worktrees found")
        return 1
    only = getattr(args, "only", None)
    if only:
        names = {wt.name: wt for wt in worktrees}
        unknown = sorted(set(only) - set(names))
        if unknown:
            raise maintenance.MaintenanceError(
                f"--only names no worktree: {', '.join(unknown)} (have: {', '.join(sorted(names))})"
            )
        worktrees = [names[n] for n in only]
    if getattr(args, "apply", False):
        control = maintenance.hostlock.HOSTLOCK_DIR.resolve()
        if any(control.is_relative_to(wt.resolve()) for wt in worktrees):
            raise maintenance.MaintenanceError(
                "host maintenance controls must be outside all worktrees"
            )
    min_free = getattr(args, "min_free_gib", None)
    try:
        plan = _call(
            "plan",
            repo=repo,
            worktrees=worktrees,
            live=live_qemu_worktrees(worktrees),
            active=active_build_worktrees(worktrees),
            now=_num(time.time()),
            allow_self=bool(getattr(args, "self", False)),
            include_integration=bool(getattr(args, "include_integration", False)),
            idle_hours=_num(getattr(args, "idle_hours", 0) or 0),
            retain_gib=_num(getattr(args, "retain_gib", None)),
            min_free_gib=_num(min_free),
            free_bytes=None if min_free is None else shutil.disk_usage(repo).free,
        )
    except native.NativeError as exc:
        if exc.kind != "budget":
            raise
        print(f"PRUNE_REFUSED: {exc}")
        return 2
    budget = plan["budget"]
    candidates = [
        Candidate(
            Path(c["worktree"]),
            [(Path(d), int(sz)) for d, sz in c["dirs"]],
            c["eligible"],
            c["skip_reason"],
        )
        for c in plan["candidates"]
    ]
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
