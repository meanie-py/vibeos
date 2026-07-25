"""`vibeos-cli land` — submit to the merge queue (default), or land one lane directly.

TWO PATHS, and the default is the queue:

  * **`land`** → `queue.py`: publish this lane's tip to the merge queue and, if no other
    agent is already the conductor, become it and drain the queue. Returns as soon as the work
    is either done (we trained) or in someone else's hands. N ready lanes cost ONE gate and
    nobody blocks. Read queue.py's header for the train's design.
  * **`land --direct`** → the serialized single-lane path documented below: merge
    origin/master into THIS tree, gate it, push it, all under the same fleet-wide land lock.
    Right when you are the only active agent, or when the queue itself needs bypassing.

Both hold `hostlock.land_lock`, so they can never race each other.

THE PROBLEM THE DIRECT PATH REMOVED (2026-07-25). Before it, landing was: merge
origin/master → commit → `vibeos-cli gate` → `git push`. The gate mints a push credential
keyed to an exact TREE HASH, so everything between "gate green" and "push accepted" is a
RACE:

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

The direct path fixed the RACE but not the SERIALIZATION: throughput is one landing per
gate, and every queued lane is a blocked agent. That is what the merge queue adds on top.
"""

from __future__ import annotations

import json
import subprocess
import time
from pathlib import Path

from . import hostlock, queue
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


def _wait_for_verdict(repo: Path, seq: str, timeout: float, run: Run) -> dict | None:
    """Poll the queue's result mailbox for OUR submission. Only used by `--wait`; the whole
    point of the queue is that you don't have to."""
    deadline = time.monotonic() + timeout
    while True:
        for rec in queue.results(repo, limit=200):
            if rec.get("seq") == seq:
                return rec
        if time.monotonic() >= deadline:
            return None
        time.sleep(5)


def _report_prior_results(repo: Path, lane: str, run: Run) -> None:
    """Surface this lane's most recent verdict on submit. A rejection an agent never reads is
    the same as a silent drop, and the agent that most needs to see it is the one that just
    came back to submit again."""
    # Skip supersessions: "your older tip was replaced by your newer one" is bookkeeping, not
    # news, and surfacing it as the lane's headline verdict buries a real prior rejection.
    prior = [r for r in queue.results(repo, lane=lane, limit=8) if r.get("outcome") != "superseded"]
    if not prior:
        return
    rec = prior[0]
    if rec.get("outcome") == "rejected":
        run.add(
            Stage(
                "prior",
                status="warn",
                detail=f"previous submission REJECTED ({rec.get('classification') or '?'})",
                salient=[
                    rec.get("reason", ""),
                    f"artifact: {rec.get('artifact') or '(none)'}",
                    f"tip: {rec.get('tip', '')[:12]}",
                ],
            )
        )
    else:
        run.add(
            Stage(
                "prior",
                status="ok",
                detail=f"previous submission landed as {rec.get('landed_as', '')[:12]}",
            )
        )


def _land_queued(args, repo: Path) -> int:
    """The default landing path: submit to the merge queue, and become the conductor if no
    other agent already is. Returns as soon as the work is either done (we trained) or in
    somebody else's hands (they are training) — never blocks holding a worktree hostage."""
    run = Run("land", "queue")
    try:
        _branch, head = _preflight(repo)
        lane = queue.lane_id(repo)
        _report_prior_results(repo, lane, run)
        queue.git(repo, "fetch", "origin")
        if queue.git_ok(repo, "merge-base", "--is-ancestor", head, "origin/master"):
            run.add(
                Stage("submit", status="skip", detail="HEAD already in master — nothing to land")
            )
            return run.finalize(repo)
        dup = next(
            (s for s in queue.pending(repo) if s.lane == lane and s.tip == head),
            None,
        )
        if dup:
            sub = dup
            run.add(
                Stage(
                    "submit",
                    status="ok",
                    detail=f"already queued as {sub.seq[-24:]} (not re-submitted)",
                )
            )
        else:
            sub = queue.submit(repo, tip=head, lane=lane)
            depth = len(queue.pending(repo))
            run.add(
                Stage(
                    "submit",
                    status="ok",
                    detail=f"{lane} @ {head[:12]} queued (position {depth})",
                )
            )
        _conduct(repo, run, args)
        rec = next((r for r in queue.results(repo, limit=200) if r.get("seq") == sub.seq), None)
        if rec is None and getattr(args, "wait", False):
            rec = _wait_for_verdict(repo, sub.seq, float(getattr(args, "wait_timeout", 3600)), run)
        if rec is None:
            run.add(
                Stage(
                    "verdict",
                    status="ok",
                    detail="still queued — `vibeos-cli land-status --mine` for the verdict",
                )
            )
        elif rec.get("outcome") == "landed":
            run.add(
                Stage("verdict", status="ok", detail=f"LANDED as {rec.get('landed_as', '')[:12]}")
            )
        else:
            run.classification = rec.get("classification") or "REJECTED"
            run.add(
                Stage(
                    "verdict",
                    status="fail",
                    detail=f"REJECTED ({run.classification})",
                    salient=[rec.get("reason", ""), f"artifact: {rec.get('artifact') or '(none)'}"],
                )
            )
        run.extra["lane"] = lane
        run.extra["seq"] = sub.seq
        return run.finalize(repo)
    except queue.NoVerdict as e:
        # Our submission is still QUEUED and unjudged — say so, or the agent will assume its
        # work was rejected and start "fixing" a bug that is on mainline.
        run.classification = "BASE_RED"
        run.add(
            Stage(
                "train",
                status="fail",
                detail="mainline itself is red — train halted, your submission is STILL QUEUED",
                salient=[str(e)],
            )
        )
        return run.finalize(repo)
    except (queue.QueueError, LandAbort) as e:
        run.classification = "LAND_ABORT"
        run.add(Stage("land", status="fail", detail=str(e).splitlines()[0], salient=[str(e)]))
        return run.finalize(repo)


def _conductor_log(run: Run):
    """Conductor progress goes to BOTH the run log and stdout. The conductor is doing the whole
    fleet's work — 'batch of 3', 'gate RED', 'splitting 2/1' is exactly what a watching agent
    needs, and it used to be buried in full.log until the drain exited."""

    def _log(msg: str) -> None:
        run.log(msg, header="train")
        print(msg, flush=True)

    return _log


def _spawn_detached(argv: list[str], cwd: Path, log_path: Path) -> None:
    """Start `argv` in its OWN SESSION with output to `log_path`. Its own function so it is a
    seam a test can replace — patching `subprocess.Popen` instead would also break
    `subprocess.run`, which uses Popen as a context manager."""
    with open(log_path, "w") as fh:
        subprocess.Popen(
            argv,
            cwd=str(cwd),
            stdout=fh,
            stderr=subprocess.STDOUT,
            start_new_session=True,  # outlives this agent's turn
        )


def _conduct(repo: Path, run: Run, args) -> tuple[bool, str]:
    """Drain the queue. DETACHED by default; returns (conducted_inline, note).

    Why detached: the whole promise of the queue is that no agent waits on a gate, and the
    ELECTED agent was the last one still waiting — its turn blocked for the length of a batch
    even though its worktree was free. The tell was that every drain during development went
    through a hand-rolled `nohup setsid drain.sh` wrapper; working around your own tool by
    hand is a bug report against the tool (AGENTS.md). So `land` spawns the conductor in its
    own session and returns in ~2s. `--foreground` keeps it inline when you want to watch.

    A detached conductor that loses the election just exits — the lock is the election, so
    spawning one speculatively costs a second of a subprocess and never double-conducts."""
    if getattr(args, "foreground", False):
        # An explicit conduct request (`--drain`, which is also how the detached child runs)
        # WAITS for the lock: closing the orphan window is the whole point, and nothing is
        # blocked by it waiting. The inline-from-submit path keeps 0.0 — there, the agent's own
        # turn is on the line, so losing the election must return immediately.
        wait = hostlock.LAND_LOCK_TIMEOUT if getattr(args, "drain", False) else 0.0
        try:
            trained = queue.try_become_conductor(
                repo,
                _conductor_log(run),
                retries=int(getattr(args, "retries", 2)),
                timeout=wait,
            )
        except queue.NoVerdict as e:
            run.classification = "BASE_RED"
            run.add(
                Stage(
                    "conduct",
                    status="fail",
                    detail="mainline itself is red — train halted, submissions STILL QUEUED",
                    salient=[str(e)],
                )
            )
            return True, "base-red"
        run.add(
            Stage(
                "conduct",
                status="ok" if trained else "skip",
                detail="drained the queue" if trained else "another agent is conducting",
            )
        )
        return True, "inline"
    log_path = run.dir / "conduct.log"
    try:
        _spawn_detached(
            [str(repo / "vibeos-cli"), "land", "--drain", "--foreground"], repo, log_path
        )
    except OSError as e:
        run.add(Stage("conduct", status="warn", detail=f"could not spawn a conductor: {e}"))
        return False, ""
    run.add(
        Stage(
            "conduct",
            status="ok",
            detail="conductor spawned (detached) — this agent is free",
            salient=[f"progress: {log_path}", "verdict: vibeos-cli land-status --mine"],
        )
    )
    return False, str(log_path)


def land(args, repo: Path) -> int:
    """`vibeos-cli land`. Queue path by default (see _land_queued); `--direct` keeps the
    serialized single-lane path below, which is right when you are the only active agent or
    when the queue itself needs bypassing."""
    if getattr(args, "drain", False):
        run = Run("land", "drain")
        _conduct(repo, run, args)
        return run.finalize(repo)
    if not getattr(args, "direct", False):
        return _land_queued(args, repo)
    return _land_direct(args, repo)


def _land_direct(args, repo: Path) -> int:
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
    """`vibeos-cli land-status` — the merge queue at a glance: who is training, what is
    queued behind them, and recent verdicts (a rejected lane's triage pointer lives here)."""
    holder = hostlock.land_holder()
    subs = queue.pending(repo)
    mine = queue.lane_id(repo) if getattr(args, "mine", False) else None
    recent = queue.results(repo, lane=mine, limit=int(getattr(args, "limit", 8)))
    if getattr(args, "json", False):
        print(
            json.dumps(
                {
                    "conductor": holder,
                    "pending": [s.to_json() for s in subs],
                    "lock_queue": hostlock.land_queue(),
                    "results": recent,
                },
                indent=2,
            )
        )
        return 0
    if holder:
        print(
            f"CONDUCTOR: pid={holder.get('pid')} {holder.get('kind', '?')} "
            f"in {holder.get('cwd', '?')}"
        )
    else:
        print("CONDUCTOR: none" + (" — queue is NOT being drained" if subs else " (queue empty)"))
    now = time.time()
    for i, s in enumerate(subs):
        waited = max(0, int(now - s.submitted_at))
        print(f"  queued #{i + 1}: {s.lane} @ {s.tip[:12]}  ({waited // 60}m{waited % 60:02d}s)")
    if not subs:
        print("  queue: empty")
    for rec in recent:
        outcome = rec.get("outcome")
        tag = {"landed": "landed ", "superseded": "superseded"}.get(outcome, "REJECTED")
        extra = (
            f" as {rec.get('landed_as', '')[:12]}"
            if rec.get("outcome") == "landed"
            else f" ({rec.get('classification') or '?'}) {rec.get('reason', '')[:70]}"
        )
        print(f"  {tag}: {rec.get('lane')} @ {rec.get('tip', '')[:12]}{extra}")
    return 0


__all__ = ["STAMP_MAX_AGE_S", "LandAbort", "cmd_land_status", "fresh_full_stamp", "land"]
