"""`vibeos-cli land` — land the current lane on master as ONE serialized critical section.

THE PROBLEM IT REMOVES (2026-07-25). The land protocol was: merge origin/master → commit →
`vibeos-cli gate` → `git push`. The gate mints a push credential keyed to an exact TREE
HASH, so everything between "gate green" and "push accepted" is a RACE:

    lane A: ---- gate (6-20 min) ----> push ✓
    lane B: ---- gate (6-20 min) ----> push ✗ non-fast-forward
                                       → merge origin/master → tree changed
                                       → stamp void → RE-GATE from scratch → race again

B's gate wasn't merely wasted wall-clock: it was 3 builds + 4 boots of *fleet* load that
made every other lane's gate slower, which widens the very window that voided it. Past a
couple of active lanes that is a congestion collapse, and it is exactly what "agents are
constantly waiting on the gate and racing to complete" is. Nothing in the system arbitrated
landing — the hostlock broker arbitrates CPU, and the pre-push hook only *detects* the
stale tree after the gate has already been spent.

THE FIX. Landing is mutual exclusion. This command holds the fleet-wide FIFO land lock
(hostlock.land_lock) across the WHOLE critical section:

    [queue FIFO] → fetch → merge origin/master → gate → push HEAD:master → release

Master therefore cannot move under a gating lane. Consequences:
  * a green gate ALWAYS lands — no lane ever re-gates because someone else landed;
  * the fleet spends exactly ONE gate per landing instead of one per attempt;
  * waiting is FIFO, bounded and VISIBLE (`vibeos-cli host-status` prints the queue and
    the current lander) instead of a lottery the most patient lane keeps losing;
  * the gate runs on the FINAL tree — the one actually being pushed — which is strictly
    more honest than gating a tree that a concurrent landing is about to invalidate.

Two deliberate design points:

  * **Gate INSIDE the lock, don't pre-gate.** A speculative gate before queueing is the
    thing that used to get thrown away. If you already have a fresh green full-gate stamp
    for the exact tree being pushed (i.e. nothing moved), `land` skips the gate — so the
    "I already gated and nobody raced me" path costs seconds.
  * **Push `HEAD:master`, never move the local `master` ref.** Force-updating `master`
    from a worktree that doesn't have it checked out leaves that worktree's index+files at
    the old tree, which resurfaces as a giant phantom "staged revert" — 2026-07-25 the main
    checkout was found in exactly that state, showing 61 commits of landed work as a
    197K-line deletion waiting to be committed. Pushing the ref straight to origin cannot
    do that; every worktree picks it up via its own `git merge origin/master`.

The pre-push hook stays as the backstop: it still refuses a master update whose tree has no
fresh green stamp, so a hand-rolled `git push` is no less safe than before.
"""

from __future__ import annotations

import json
import subprocess
import time
from pathlib import Path

from . import hostlock
from .artifacts import Run, Stage

STAMP_MAX_AGE_S = 24 * 3600
FULL_ARCHES = {"i386", "amd64", "arm64"}


class LandAbort(RuntimeError):
    """A precondition failed or a step could not be completed. Carries the operator-facing
    reason; `land()` turns it into a fail Stage + nonzero exit (never a silent skip)."""


def _git(repo: Path, *args: str, check: bool = True) -> str:
    cp = subprocess.run(["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=600)
    if check and cp.returncode != 0:
        raise LandAbort(f"git {' '.join(args)} failed ({cp.returncode}): {cp.stderr.strip()}")
    return cp.stdout.strip()


def _git_ok(repo: Path, *args: str) -> bool:
    return (
        subprocess.run(
            ["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=600
        ).returncode
        == 0
    )


def _stamps_dir(repo: Path) -> Path:
    common = Path(_git(repo, "rev-parse", "--git-common-dir"))
    if not common.is_absolute():
        common = (repo / common).resolve()
    return common


def fresh_full_stamp(repo: Path, tree: str, *, now: float | None = None) -> dict | None:
    """The gate stamp validating exactly `tree`, if one is fresh (<24h) and full-arch.

    Same acceptance rule as the pre-push hook — deliberately duplicated rather than
    imported, because the hook is a standalone script that must keep working on a clone
    whose python can't import this package. Kept in sync by test_cli_units."""
    now = time.time() if now is None else now
    common = _stamps_dir(repo)
    candidates = []
    d = common / "vibeos-gate-stamps"
    if d.is_dir():
        candidates.extend(sorted(d.glob("*.json")))
    legacy = common / "vibeos-gate-stamp.json"
    if legacy.is_file():
        candidates.append(legacy)
    for p in candidates:
        try:
            s = json.loads(p.read_text())
        except (OSError, ValueError):
            continue
        if s.get("tree") != tree:
            continue
        if now - s.get("time", 0) > STAMP_MAX_AGE_S:
            continue
        if not FULL_ARCHES.issubset(set(s.get("arches", []))):
            continue
        return s
    return None


def _preflight(repo: Path) -> tuple[str, str]:
    """Refuse to start unless the tree can produce a stampable push. Returns (branch, head)."""
    branch = _git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    if branch == "HEAD":
        raise LandAbort("detached HEAD — check out the lane branch you want to land")
    dirty = _git(repo, "status", "--porcelain", "--untracked-files=no")
    if dirty:
        n = len(dirty.splitlines())
        raise LandAbort(
            f"{n} uncommitted change(s) — commit (or `git stash push -m ...`) first; the "
            "gate stamp must hash the exact content being pushed, so a dirty tree can "
            "never mint one"
        )
    return branch, _git(repo, "rev-parse", "HEAD")


def _merge_master(repo: Path, run: Run) -> str:
    """Merge origin/master into HEAD (no-op when already contained). Returns the new HEAD.
    A conflicting merge is aborted and reported — resolving it is the lane's call, not
    something to do while holding the fleet's land lock."""
    remote = _git(repo, "rev-parse", "origin/master")
    if _git_ok(repo, "merge-base", "--is-ancestor", remote, "HEAD"):
        run.add(Stage("merge", status="ok", detail=f"already contains origin/master {remote[:12]}"))
        return _git(repo, "rev-parse", "HEAD")
    cp = subprocess.run(
        ["git", "merge", "--no-edit", remote],
        cwd=str(repo),
        capture_output=True,
        text=True,
        timeout=900,
    )
    if cp.returncode != 0:
        conflicts = _git(repo, "diff", "--name-only", "--diff-filter=U", check=False)
        subprocess.run(["git", "merge", "--abort"], cwd=str(repo), capture_output=True, text=True)
        raise LandAbort(
            "merge of origin/master CONFLICTS (aborted, tree restored) — resolve it, "
            f"commit, then re-run land. Conflicts:\n    {conflicts or cp.stderr.strip()}"
        )
    head = _git(repo, "rev-parse", "HEAD")
    run.add(Stage("merge", status="ok", detail=f"merged origin/master {remote[:12]} → {head[:12]}"))
    return head


def _run_gate(repo: Path, run: Run, args) -> None:
    """Run the FULL gate as a subprocess and require exit 0. Full arch set only: a
    partial gate never mints the stamp, so it could not land anyway."""
    cmd = [str(repo / "vibeos-cli"), "gate", "--retries", str(getattr(args, "retries", 2))]
    if getattr(args, "clean", False):
        cmd.append("--clean")
    t0 = time.monotonic()
    cp = run.run(cmd, cwd=repo, stage="gate")
    dt = time.monotonic() - t0
    if cp.returncode != 0:
        run.add(
            Stage(
                "gate",
                exit=cp.returncode,
                seconds=dt,
                status="fail",
                detail="gate RED — nothing pushed",
                salient=[f"vibeos-cli gate exited {cp.returncode}; see its own artifact dir"],
            )
        )
        raise LandAbort("gate failed — fix the regression, then re-run land (nothing pushed)")
    run.add(Stage("gate", seconds=dt, status="ok", detail="all arches green"))


def _push(repo: Path, run: Run, head: str) -> None:
    """Fast-forward origin/master to HEAD. We hold the land lock, so this cannot lose a
    race; a non-fast-forward here would mean somebody bypassed the lock, and the loud
    failure is the correct outcome."""
    remote = _git(repo, "rev-parse", "origin/master")
    if not _git_ok(repo, "merge-base", "--is-ancestor", remote, head):
        raise LandAbort(
            f"origin/master ({remote[:12]}) is NOT an ancestor of HEAD ({head[:12]}) — "
            "somebody landed while we held the land lock (bypassed it?). Re-run land."
        )
    cp = run.run(["git", "push", "origin", f"{head}:refs/heads/master"], cwd=repo, stage="push")
    if cp.returncode != 0:
        run.add(
            Stage(
                "push",
                exit=cp.returncode,
                status="fail",
                detail="push rejected",
                salient=[(cp.stderr or cp.stdout or "").strip().splitlines()[-1:][0]]
                if (cp.stderr or cp.stdout)
                else [],
            )
        )
        raise LandAbort("push to origin/master rejected — see the push stage output")
    run.add(Stage("push", status="ok", detail=f"origin/master → {head[:12]}"))


def _report_queue(run: Run) -> None:
    holder = hostlock.land_holder()
    waiters = hostlock.land_queue()
    if holder:
        run.log(
            f"another lane is landing right now: pid={holder.get('pid')} "
            f"{holder.get('kind', '?')} in {holder.get('cwd', '?')}"
            + (f"; {len(waiters)} already queued ahead-or-with us" if waiters else ""),
            header="land-queue",
        )


def land(args, repo: Path) -> int:
    run = Run("land", "master")
    try:
        branch, head = _preflight(repo)
        _git(repo, "fetch", "origin")
        remote = _git(repo, "rev-parse", "origin/master")
        if remote == head:
            run.add(
                Stage("preflight", status="ok", detail="HEAD == origin/master: nothing to land")
            )
            return run.finalize(repo)
        behind = _git_ok(repo, "merge-base", "--is-ancestor", remote, head)
        stamp = fresh_full_stamp(repo, _git(repo, "rev-parse", "HEAD^{tree}"))
        run.add(
            Stage(
                "preflight",
                status="ok",
                detail=f"branch {branch} @ {head[:12]}; "
                f"origin/master {'contained' if behind else 'AHEAD — will merge'}; "
                f"stamp {'present' if stamp else 'needed'}",
            )
        )
        if getattr(args, "dry_run", False):
            run.log(
                "dry run: would "
                + ("merge origin/master, " if not behind else "")
                + ("skip the gate (fresh stamp), " if stamp and behind else "run the full gate, ")
                + "then push HEAD:master under the land lock",
                header="dry-run",
            )
            _report_queue(run)
            return run.finalize(repo)

        _report_queue(run)
        timeout = float(getattr(args, "queue_timeout", 0) or hostlock.LAND_LOCK_TIMEOUT)
        t_queue = time.monotonic()
        with hostlock.land_lock(f"land:{branch}", timeout=timeout, repo=str(repo)):
            queued = time.monotonic() - t_queue
            run.add(
                Stage("land-lock", seconds=queued, status="ok", detail="held (fleet-wide, FIFO)")
            )
            # Re-fetch INSIDE the lock: whoever we queued behind just landed, so the
            # origin/master we resolved during preflight is stale by construction.
            _git(repo, "fetch", "origin")
            head = _merge_master(repo, run)
            tree = _git(repo, "rev-parse", "HEAD^{tree}")
            stamp = fresh_full_stamp(repo, tree)
            if stamp:
                run.add(
                    Stage(
                        "gate",
                        status="skip",
                        detail=f"fresh green full-gate stamp already covers tree {tree[:12]}",
                    )
                )
            else:
                _run_gate(repo, run, args)
            if getattr(args, "no_push", False):
                run.add(Stage("push", status="skip", detail="--no-push: gated but not landed"))
            else:
                _push(repo, run, head)
        run.extra["branch"] = branch
        run.extra["head"] = head
        run.extra["queued_s"] = round(queued, 1)
        return run.finalize(repo)
    except (hostlock.LandLockBusy, LandAbort) as e:
        run.classification = (
            "LAND_LOCK_BUSY" if isinstance(e, hostlock.LandLockBusy) else "LAND_ABORT"
        )
        run.add(Stage("land", status="fail", detail=str(e).splitlines()[0], salient=[str(e)]))
        return run.finalize(repo)
    except subprocess.SubprocessError as e:
        run.classification = "LAND_ABORT"
        run.add(Stage("land", status="fail", detail=f"git subprocess error: {e}"))
        return run.finalize(repo)


def cmd_land_status(args, repo: Path) -> int:
    """`vibeos-cli land --status` equivalent, exposed for scripts: who holds the land lock
    and who is queued, as JSON or one line each."""
    holder = hostlock.land_holder()
    waiters = hostlock.land_queue()
    if getattr(args, "json", False):
        print(json.dumps({"holder": holder, "queue": waiters}, indent=2))
        return 0
    if holder:
        print(
            f"LANDING: pid={holder.get('pid')} {holder.get('kind', '?')} "
            f"in {holder.get('cwd', '?')}"
        )
    else:
        print("LANDING: free")
    for i, w in enumerate(waiters):
        print(f"  queued #{i + 1}: pid={w['pid']} waiting {w['waited_s']:.0f}s")
    return 0


__all__ = ["STAMP_MAX_AGE_S", "LandAbort", "cmd_land_status", "fresh_full_stamp", "land"]
