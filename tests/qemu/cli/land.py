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

import argparse
import json
import os
import signal
import subprocess
import time
from pathlib import Path

from . import hostlock, queue, warmth
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


def _docs_only_vs_master(repo: Path) -> bool:
    """Does this lane change NOTHING but docs, measured against the merge base with master?

    The pre-push hook has carried this exemption since it was written -- a docs-only delta
    needs no fresh gate stamp -- but the land preflights did not, so a commit touching one
    `.md` paid a full `check-tooling` plus both strict source-lint partitions. A docs-only
    change cannot fail a Python linter or a source scanner, because it contains no Python
    and no source.

    It bit the change that recorded the orphan inventory: refused at
    `test_verify_ucrtbase_cohab.py` over a stale `build-i386` in the author's worktree,
    for a delta of one Markdown file.

    Skipping here is safe in the direction that matters. The CONDUCTOR still runs every
    check on the merged batch, so this cannot put unlinted code on master -- it only
    declines to lint a lane that has none. The predicate is `tools/docs_only.py`, shared
    with the pre-push hook: two spellings of "docs-only" that can disagree is worse than
    neither, so there is exactly one.

    Uses the three-dot merge base, so a lane that is merely BEHIND master is still judged
    on what IT changed rather than on what landed while it worked. Fails CLOSED -- any git
    error returns False and the lane lints normally.
    """
    try:
        diff = _git(repo, "diff", "--name-only", "origin/master...HEAD")
    except Exception:
        return False
    module = _docs_only_module(repo)
    return bool(module and module.docs_only(diff.splitlines()))


def _docs_only_module(repo: Path):
    """`tools/docs_only.py` from THIS worktree, or None.

    By PATH, not by name: a `sys.path` insert leaves `docs_only` in `sys.modules`, and a
    process that later conducts a drain would then judge the INTEGRATION tree with this
    lane's copy of the rule. Same reason queue.py loads it this way."""
    import importlib.util

    try:
        spec = importlib.util.spec_from_file_location(
            "_land_docs_only", repo / "tools" / "docs_only.py"
        )
        if spec is None or spec.loader is None:
            return None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
    except Exception:
        return None
    return module


GATE_ARCHES = ("i386", "amd64", "arm64")


def _warm_preflight(repo: Path, run: Run, args) -> None:
    """Refuse a lane whose code no build has seen, so the gate would compile it cold.

    See warmth.py for the incident. Cheap (a few git calls). Skipped for a docs-only lane,
    which the conductor does not gate at all, and overridable with `--cold` for the rare
    lane that means to land cold anyway -- report every use, since that lane's gate can
    hold the whole queue for hours. Fails OPEN when the receipts cannot be read: this is a
    courtesy to the fleet, not a correctness check, and the gate itself still judges."""
    if getattr(args, "cold", False):
        run.add(
            Stage("warm", status="warn", detail="--cold: gate may compile this lane from scratch")
        )
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("warm", status="skip", detail="docs-only delta"))
        return
    module = _docs_only_module(repo)
    if module is None:
        run.add(Stage("warm", status="warn", detail="docs_only predicate unavailable; not checked"))
        return
    try:
        tree = _git(repo, "rev-parse", "HEAD^{tree}")
        master = _git(repo, "rev-parse", "origin/master^{tree}")
        cold = warmth.missing(repo, tree, GATE_ARCHES, module.is_inert, master=master)
    except Exception as exc:
        run.add(Stage("warm", status="warn", detail=f"could not check receipts: {exc}"))
        return
    if not cold:
        run.add(Stage("warm", status="ok", detail="built on all three arches"))
        return
    raise LandAbort(
        f"this tree has not been BUILT on {', '.join(cold)} — the merge queue's gate builds "
        "cold and leans on the shared compiler caches, so code no build has seen compiles "
        "from scratch there and can hold the whole queue for hours (upstream-sync, "
        "2026-09-26). Build first, which warms the cache for the gate:\n    "
        + "\n    ".join(f"./vibeos-cli build --arch {a}" for a in cold)
        + "\n  then `./vibeos-cli land` again. Commit exactly what you built: the receipt is "
        "keyed on the tree's content (docs-only differences are fine).\n"
        "  override (report every use): ./vibeos-cli land --cold"
    )


def _lint_preflight(repo: Path, run: Run, args) -> None:
    """Run `check-tooling` HERE, before this lane consumes a batched gate.

    The conductor already runs this exact command, so this can never block anything the
    gate would have let through — it is the same verdict, moved from the most expensive
    place in the system to the cheapest. That matters because a batched gate is 3 builds
    + 4 boots of fleet load, and a red one bounces the whole train: the lint that
    rejected this lane on 2026-07-26 was a single line 101 characters long.

    Three properties make the move sound:
      * `check-tooling` is arch-independent, host-only and deterministic (gate.py's
        `_tooling_once` says so itself, and starts it before the first build);
      * it needs no build-<arch>, so no fleet-isolation question arises;
      * failing here is strictly earlier than failing there — never *more* strict.

    It is an approximation in one direction only: we lint the lane tip, while the
    conductor lints the tip MERGED with master. So this can miss a failure the merge
    introduces. That is fine — it is an early filter, not a replacement for the gate.

    FAIL OPEN if the check itself cannot run (missing venv, no ruff, timeout). A broken
    preflight must never be able to wedge the fleet's ability to land; only a check that
    ran and returned nonzero blocks.
    """
    if getattr(args, "skip_lint", False):
        run.add(Stage("lint", status="skip", detail="--skip-lint: check-tooling not run"))
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("lint", status="skip", detail="docs-only delta: nothing for it to lint"))
        return
    t0 = time.monotonic()
    try:
        cp = subprocess.run(
            [str(repo / "vibeos-cli"), "check-tooling"],
            cwd=str(repo),
            capture_output=True,
            text=True,
            timeout=1800,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        run.add(Stage("lint", status="warn", detail=f"check-tooling could not run: {exc}"))
        return
    dt = time.monotonic() - t0
    if cp.returncode and any(
        line.startswith("DISK_ADMISSION_FAIL:") for line in (cp.stdout or "").splitlines()
    ):
        run.classification = "DISK_ADMISSION_FAIL"
        raise LandAbort(cp.stdout.strip())
    if cp.returncode == 0:
        run.add(Stage("lint", status="ok", detail=f"check-tooling clean ({dt:.0f}s)"))
        return
    # PERSIST what we captured before aborting. check-tooling prints the failing
    # test's own output after the summary, and this is the only copy of it: the
    # child writes no artifact dir, so discarding stdout here leaves a lane with a
    # ✗ line and nothing to debug. That is not hypothetical — a preflight abort on
    # 2026-09-06 named only `pytest:.` for a failure that did not reproduce, and
    # the evidence was gone.
    log = run.dir / "preflight-check-tooling.log"
    try:
        log.write_text((cp.stdout or "") + (cp.stderr or ""), encoding="utf-8", errors="replace")
    except OSError:
        pass
    salient = [ln.strip() for ln in (cp.stdout or "").splitlines() if "✗" in ln]
    raise LandAbort(
        "check-tooling FAILED — fix it before queueing, or the conductor bounces the whole "
        "batch (3 builds + 4 boots of fleet load) for it:\n    "
        + ("\n    ".join(salient) if salient else f"exit {cp.returncode}")
        + f"\n  captured output: {log}"
        + "\n  run `./vibeos-cli check-tooling` for the detail."
        + "\n  override (report every use): ./vibeos-cli land --skip-lint"
    )


def _source_lint_preflight(repo: Path, run: Run, args) -> None:
    """Run the conductor's OWN source-lint partitions here, before the batched gate.

    `_lint_preflight` above is named "lint" but only ever ran `check-tooling`. The gate
    runs that AND `check --strict --scope source-early|source-deferred`, and those two
    are the other cheap half — pure source, no build, no boot. On 2026-09-19 they
    rejected the same lane twice (`scan_unpruned_walk`, a one-line fix each time) after
    the full queue wait, while `./vibeos-cli check` stayed green the whole time, because
    plain `check` does not run those partitions in STRICT mode. A gate downstream of the
    cost is not a gate.

    Uses gate's `_lint_once` rather than re-deriving the command, so this cannot drift
    from what the conductor actually runs.

    NEVER MORE STRICT THAN THE GATE, which is the property that makes moving it sound:
    a partition that could not RUN here must not block. `source-deferred` contains
    `verify_signatures` and `gen_ctype_tables`, which hard-fail without this worktree's
    `build-<arch>` — a fresh lane legitimately has none. `_lint_once` already separates
    those: a crash, or a nonzero exit with no parseable finding lines, classifies as
    SCANNER_ERROR, and only a real NEW-findings verdict blocks. Everything else warns,
    matching the fail-open rule the check-tooling half already follows.
    """
    if getattr(args, "skip_lint", False):
        return
    if _docs_only_vs_master(repo):
        for part in ("source-early", "source-deferred"):
            run.add(Stage(f"lint:{part}", status="skip", detail="docs-only delta"))
        return
    from .gate import _lint_once

    for part in ("source-early", "source-deferred"):
        try:
            result = _lint_once(repo, part, run.dir / f"preflight-lint-{part}")
        except (OSError, subprocess.SubprocessError) as exc:
            run.add(Stage(f"lint:{part}", status="warn", detail=f"could not run: {exc}"))
            continue
        if result["ok"]:
            run.add(Stage(f"lint:{part}", status="ok", detail=f"clean ({result['seconds']:.0f}s)"))
            continue
        if result["classification"] == "SCANNER_ERROR":
            # Infrastructure, not a finding — the conductor has a built tree and we may
            # not. Say so and let the gate be the oracle.
            #
            # BUT ONLY FOR THE SCANNER THAT CRASHED. A partition is one classification
            # for many scanners, and SCANNER_ERROR from one of them used to wave through
            # a real NEW finding from another: on 2026-09-26 msvcrt-homebrew's preflight
            # reported `scan_const_contradictions: 4 new` right beside a marshalgen crash
            # (a stale build dir), submitted three times on "could not complete", and the
            # conductor's gate caught the finding ~80 minutes later.
            real = _real_findings(run.dir / f"preflight-lint-{part}")
            if real:
                raise LandAbort(
                    f"check --strict --scope {part} found NEW findings (another scanner in "
                    "the partition also crashed, which does not excuse these):\n    "
                    + "\n    ".join(real[:8])
                    + f"\n  artifact: {run.dir / f'preflight-lint-{part}'}"
                    + f"\n  reproduce: ./vibeos-cli check --strict --scope {part}"
                    + "\n  override (report every use): ./vibeos-cli land --skip-lint"
                )
            run.add(
                Stage(
                    f"lint:{part}",
                    status="warn",
                    detail=f"could not complete here ({result['salient'] or 'scanner error'})",
                )
            )
            continue
        raise LandAbort(
            f"check --strict --scope {part} FAILED — fix it before queueing, or the "
            "conductor bounces the whole batch (3 builds + 4 boots of fleet load) for a "
            "source lint:\n    "
            + (result["salient"] or f"exit {result['exit']}")
            + f"\n  artifact: {run.dir / f'preflight-lint-{part}'}"
            + f"\n  reproduce: ./vibeos-cli check --strict --scope {part}"
            + "\n  (plain `./vibeos-cli check` does NOT run this partition in strict mode,"
            " so a green `check` is not evidence here.)"
            + "\n  override (report every use): ./vibeos-cli land --skip-lint"
        )


def _real_findings(root: Path) -> list[str]:
    """Scanners in a partition run under `root` that FAILED WITH FINDINGS, as opposed to
    crashing. A crash reports `SCANNER_ERROR: ...` in its stage detail; anything else
    that failed is a verdict. Unreadable summaries yield [] (fail open, as before)."""
    out: list[str] = []
    for summary in sorted(root.glob("**/summary.json")):
        try:
            stages = json.loads(summary.read_text()).get("stages", [])
        except (OSError, ValueError):
            continue
        for st in stages:
            detail = str(st.get("detail", ""))
            if st.get("status") != "fail" or detail.startswith("SCANNER_ERROR"):
                continue
            first = (st.get("salient") or [""])[0]
            out.append(f"{st.get('name')}: {detail}" + (f" — {first}" if first else ""))
    return out


def _difftest_preflight(repo: Path, run: Run, args) -> None:
    """Run the gate's host differential harnesses HERE, before this lane is batched.

    The other cheap, build-free, boot-free third of the gate. Without it, on 2026-09-25 a
    lane that deleted a Microsoft source file its own difftest still linked against went
    through lint preflight green, then turned SEVENTEEN of eighteen batched gates red over
    one evening -- splitting every batch it rode in, and (through a stale cargo cache in
    the integration tree) poisoning batches it was not even in. ~30s here would have
    stopped it at the door.

    Uses gate's `_difftest_once`, so this cannot drift from what the conductor runs. Same
    rules as the two lint preflights: never stricter than the gate (it is the same
    command), approximate only in that it tests the lane tip rather than tip+master, and
    FAIL OPEN when the harness cannot run at all -- only a harness that ran and reported a
    divergence blocks.
    """
    if getattr(args, "skip_lint", False):
        run.add(Stage("difftest", status="skip", detail="--skip-lint: difftest not run"))
        return
    if _docs_only_vs_master(repo):
        run.add(Stage("difftest", status="skip", detail="docs-only delta"))
        return
    from .gate import _difftest_once

    root = run.dir / "preflight-difftest"
    try:
        result = _difftest_once(repo, root)
    except (OSError, subprocess.SubprocessError) as exc:
        run.add(Stage("difftest", status="warn", detail=f"could not run: {exc}"))
        return
    if result["ok"]:
        run.add(Stage("difftest", status="ok", detail=f"clean ({result['seconds']:.0f}s)"))
        return
    raise LandAbort(
        "difftest FAILED — fix it before queueing, or the conductor bounces the whole batch "
        "(3 builds + 4 boots of fleet load) for it:\n    "
        + (result["salient"] or f"exit {result['exit']}")
        + f"\n  console: {root / 'difftest-console.log'}"
        + "\n  reproduce: ./vibeos-cli difftest"
        + "\n  override (report every use): ./vibeos-cli land --skip-lint"
    )


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
    cp = run.run(cmd, cwd=repo, stage="gate", env=dict(os.environ, VIBEOS_JOB_CLASS="integration"))
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
    attempt = None
    try:
        _branch, head = _preflight(repo)
        lane = queue.lane_id(repo)
        # Marker first: everything from here to the submit is the window where a kill leaves
        # the lane silently unregistered. See queue.begin_attempt.
        attempt = queue.begin_attempt(repo, lane, head)
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
        if getattr(args, "dry_run", False):
            # `--dry-run` used to be honoured only on the `--direct` path, so the DEFAULT
            # path — the one every agent reaches for — submitted for real while reporting
            # itself as a plan. A dry run that performs the action is worse than no dry run
            # at all: it teaches you the flag is safe right before it isn't.
            depth = len(queue.pending(repo))
            run.add(
                Stage(
                    "submit",
                    status="skip",
                    detail=(
                        f"dry run: {lane} @ {head[:12]} is already queued as "
                        f"{dup.seq[-24:]} — would NOT re-submit"
                        if dup
                        else f"dry run: would queue {lane} @ {head[:12]} at position {depth + 1}"
                    ),
                )
            )
            _report_queue(run)
            return run.finalize(repo)
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
            # Only on a REAL submission: if this tip is already queued we are adding no new
            # risk to the batch, and a --dry-run has already returned above.
            _warm_preflight(repo, run, args)
            _lint_preflight(repo, run, args)
            _source_lint_preflight(repo, run, args)
            _difftest_preflight(repo, run, args)
            sub = queue.submit(repo, tip=head, lane=lane)
            # VERIFY the submission persisted, by RE-READING the queue from disk rather
            # than trusting the return value. `queue.submit` returning a Submission means
            # its writes did not raise; it is not the same claim as "a later reader will
            # find this record". The two diverge on a full disk, a partial write, a
            # concurrent resolve -- and every one of those ends with an agent reporting a
            # landing on the strength of an `OK` line. This is Ask 1 of
            # land-verification-asks.md, and it closes shape (a) ONLY: a process killed
            # before reaching here still cannot report, which is why the
            # RUN_INTERRUPTED marker (artifacts.write_interrupted_marker) has to exist
            # separately. Do not read this stage as closing both.
            landed = queue.pending(repo)
            if not any(s.seq == sub.seq and s.tip == head for s in landed):
                raise LandAbort(
                    f"submit did NOT persist: {lane} @ {head[:12]} (seq {sub.seq[-24:]}) is "
                    f"not in the queue when re-read, though queue.submit returned it. "
                    f"{len(landed)} other submission(s) are queued, so the queue itself is "
                    f"readable. You are NOT in the queue -- do not report this as landed. "
                    f"Re-run `./vibeos-cli land`, and check `df -h /` first."
                )
            run.add(
                Stage(
                    "submit",
                    status="ok",
                    detail=f"{lane} @ {head[:12]} queued (position {len(landed)}, verified)",
                )
            )
        held = queue.effective_hold(repo)
        if held:
            # Queued, not refused: a refusal would have every lane retry-looping against the
            # hold, and a queued submission costs nothing while no conductor is running.
            run.add(
                Stage(
                    "conduct",
                    status="warn",
                    detail="line is HELD — queued, no conductor started",
                    salient=[queue.describe_hold(held), "drains on `vibeos-cli land --release`"],
                )
            )
        else:
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
        #
        # BaseRed is only ONE of the ways a train renders no verdict, and reporting them all
        # as "mainline is red" sends the lane hunting a mainline break that does not exist —
        # the same misattribution BaseCheck exists to prevent, committed by the reporter
        # instead of the judge. An Inconclusive carries its own reason (dirty integration
        # tree, starved host, gate that never wrote a verdict); pass it through as itself.
        base_red = isinstance(e, queue.BaseRed)
        run.classification = "BASE_RED" if base_red else "NO_VERDICT"
        run.add(
            Stage(
                "train",
                status="fail",
                detail=(
                    "mainline itself is red — train halted, your submission is STILL QUEUED"
                    if base_red
                    else "the train rendered NO VERDICT — your submission is STILL QUEUED"
                ),
                salient=[str(e)],
            )
        )
        return run.finalize(repo)
    except (queue.QueueError, LandAbort) as e:
        run.classification = run.classification or "LAND_ABORT"
        run.add(Stage("land", status="fail", detail=str(e).splitlines()[0], salient=[str(e)]))
        return run.finalize(repo)
    finally:
        # Every ordinary exit clears the marker, an abort included: an abort is LOUD, and the
        # marker exists only for the silent death. A SIGKILL skips this, which is the whole
        # mechanism -- a surviving marker means "killed", not "failed".
        queue.end_attempt(attempt)


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
            env=dict(os.environ, VIBEOS_JOB_LAUNCHER_PID=str(os.getpid())),
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
        except queue.LineHeld as e:
            run.add(
                Stage(
                    "conduct",
                    status="warn",
                    detail="line is HELD — nothing drained",
                    salient=[str(e)],
                )
            )
            return True, "held"
        except queue.NoVerdict as e:
            # Same distinction as the submit path above: "no verdict" is not "mainline is red".
            base_red = isinstance(e, queue.BaseRed)
            run.classification = "BASE_RED" if base_red else "NO_VERDICT"
            run.add(
                Stage(
                    "conduct",
                    status="fail",
                    detail=(
                        "mainline itself is red — train halted, submissions STILL QUEUED"
                        if base_red
                        else "the train rendered NO VERDICT — submissions STILL QUEUED"
                    ),
                    salient=[str(e)],
                )
            )
            return True, "base-red" if base_red else "no-verdict"
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
    if getattr(args, "release", False):
        return _release(args, repo)
    if getattr(args, "withdraw", False):
        return _withdraw(args, repo)
    if getattr(args, "evict", None) is not None:
        return _evict(args, repo)
    if getattr(args, "hold", None) is not None:
        return _hold(args, repo)
    if getattr(args, "drain", False):
        run = Run("land", "drain")
        _conduct(repo, run, args)
        return run.finalize(repo)
    if not getattr(args, "direct", False):
        return _land_queued(args, repo)
    return _land_direct(args, repo)


def _land_direct(args, repo: Path) -> int:
    run = Run("land", "master")
    pushed = False
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

        # BEFORE the land lock, not inside it: the direct path's gate is this lane's own,
        # but a lint failure still burns a whole gate, and there is no reason to hold a
        # fleet-wide lock for the ~40s this takes.
        _warm_preflight(repo, run, args)
        _lint_preflight(repo, run, args)
        _source_lint_preflight(repo, run, args)
        _difftest_preflight(repo, run, args)

        _check_hold_for_direct(repo)
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
            # And the hold again: someone may have stopped the line while we queued.
            _check_hold_for_direct(repo)
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
                pushed = True
        if pushed:
            _after_direct_landing(repo, run, args)
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


def _explain_tip(repo: Path, tip: str) -> int:
    """`land-status --tip <sha>` — where did THIS commit get to?

    Ask 1's other half. `land` is honest in all four failure shapes (see
    land-verification-asks.md), so the question an agent actually has -- "is the work I
    just submitted lost?" -- was answerable only by remembering a two-test ancestry
    recipe and knowing that supersessions render in a third list. A rule with no command
    behind it is a rule that gets skipped, and this one was skipped into a wrong report
    to a user, which is the only failure here that cannot be fixed after the fact.

    Answers about a SPECIFIC commit, never "did this lane land anything recently" -- that
    is the question both failure modes answer yes to. And it does not depend on a record
    being inside `results()`'s recent window: on a busy morning a lane's own superseded
    row scrolls off behind other lanes' landings, which is exactly how "my tip vanished"
    was read as work-loss when nothing had been lost.
    """
    try:
        tip = queue.git(repo, "rev-parse", tip)
    except subprocess.SubprocessError:
        print(f"  UNKNOWN COMMIT: {tip} does not resolve in this worktree.")
        return 2
    short = tip[:12]
    queue.git(repo, "fetch", "origin")
    if queue.git_ok(repo, "merge-base", "--is-ancestor", tip, "origin/master"):
        print(f"  LANDED: {short} is an ancestor of origin/master. The work is on mainline.")
        return 0
    pend = queue.pending(repo)
    for s in pend:
        if s.tip == tip:
            waited = max(0, int(time.time() - s.submitted_at))
            print(
                f"  QUEUED: {short} is pending as {s.lane} "
                f"({waited // 60}m{waited % 60:02d}s). Not lost, not landed yet."
            )
            return 0
    # A RECORDED outcome outranks the containment inference below, because a superseded tip
    # is ALSO contained in the descendant that superseded it -- both statements are true and
    # only one of them explains why the tip is not listed on its own. Carrier named anyway.
    carrier = next(
        (s for s in pend if queue.git_ok(repo, "merge-base", "--is-ancestor", tip, s.tip)),
        None,
    )
    for rec in queue.results(repo, limit=500):
        if rec.get("tip") != tip:
            continue
        outcome = rec.get("outcome") or "?"
        if outcome == "superseded":
            print(f"  SUPERSEDED: {short} — {rec.get('reason', '')}")
            print("    Nothing is wrong: your newer tip contains this one. Follow that tip.")
            if carrier:
                print(f"    Carried by {carrier.lane} @ {carrier.tip[:12]}, still queued.")
        elif outcome == "landed":
            # Reachable when origin/master is behind what the queue recorded.
            print(
                f"  LANDED as {str(rec.get('landed_as', ''))[:12]}, but {short} is not yet "
                f"an ancestor of THIS worktree's origin/master — run `git fetch`."
            )
        else:
            print(
                f"  {outcome.upper()}: {short} ({rec.get('classification') or '?'}) "
                f"{str(rec.get('reason', ''))[:120]}"
            )
            print(f"    artifact: {rec.get('artifact') or '(none)'}")
        return 0
    if carrier:
        print(
            f"  TRAVELLING: {short} is not queued itself, but is contained in "
            f"{carrier.lane} @ {carrier.tip[:12]}, which IS queued. Landing that lands this."
        )
        return 0
    for rec in queue.abandoned_attempts(repo):
        if rec.get("tip") == tip:
            print(
                f"  NEVER QUEUED: a `land` for {short} died before enqueueing anything "
                f"(pid {rec.get('pid')} is gone). Re-run `./vibeos-cli land`."
            )
            return 1
    print(
        f"  LOST: {short} is not on origin/master, not queued, not contained in anything "
        f"queued, and has no verdict or abandoned attempt on record. It was never "
        f"submitted, or its submission did not persist. Re-run `./vibeos-cli land`."
    )
    return 1


def _check_hold_for_direct(repo: Path) -> None:
    """`--direct` is how a held line gets its fix in, so it is the one path a hold does not
    simply close -- but only for the lane that holds it (or anyone, under an AUTO hold on a
    red base). Everyone else waiting on the hold is the point of it."""
    rec = queue.effective_hold(repo)
    if not queue.may_land_direct(rec, queue.lane_id(repo)):
        assert rec is not None
        raise LandAbort(
            f"{queue.describe_hold(rec)}. Only {rec.get('lane')} may land while it stands. "
            "Commit and wait -- `vibeos-cli land-status` shows the hold; "
            "`vibeos-cli land` still QUEUES your lane and it drains on release."
        )


def _after_direct_landing(repo: Path, run: Run, args) -> None:
    """A direct landing leaves any queued lanes with no conductor (2026-09-26: five
    submissions sat under `CONDUCTOR: none` after the fix landed). Wake the train -- unless
    the line is still held, in which case say so, because a forgotten hold is a silent stop."""
    queue.git(repo, "fetch", "origin")
    rec = queue.effective_hold(repo)  # an AUTO hold lapses here: master just moved
    if rec is not None:
        run.add(
            Stage(
                "hold",
                status="warn",
                detail="the line is STILL HELD — `vibeos-cli land --release` when you are done",
                salient=[queue.describe_hold(rec)],
            )
        )
        return
    if queue.pending(repo) and not getattr(args, "no_push", False):
        _conduct(repo, run, argparse.Namespace(foreground=False))


def _preempt_conductor(run: Run) -> None:
    """Stop a conductor that is mid-batch. Its submissions stay queued -- a killed conductor
    is the crash case the queue was built for (queue.py: 'Crash-safety is the kernel's job'),
    and the next drain resets the integration tree hard. Only a DETACHED conductor is
    stopped, as its whole session: that is the gate's builds and boots with it, and it can
    never be an ancestor of ours. A `--direct` lander is someone's own landing -- that one
    is named, never killed."""
    h = hostlock.land_holder()
    kind = str(h.get("kind", ""))
    pid = int(h.get("pid") or 0)
    if not h or not pid:
        run.add(Stage("preempt", status="skip", detail="no conductor running"))
        return
    if not kind.startswith("conductor:"):
        run.add(
            Stage(
                "preempt",
                status="warn",
                detail=f"the land lock is held by a direct landing ({kind}, pid {pid}) — not "
                "killed; the hold takes effect when it finishes",
            )
        )
        return
    try:
        sid, pgid = os.getsid(pid), os.getpgid(pid)
    except OSError:
        run.add(Stage("preempt", status="skip", detail=f"conductor pid {pid} already gone"))
        return
    if sid != pgid or pgid in (os.getpgid(0), os.getsid(0)):
        run.add(
            Stage(
                "preempt",
                status="warn",
                detail=f"conductor pid {pid} is not a detached session (it runs inside "
                "someone's shell) — not killed; it stops at its next batch boundary",
            )
        )
        return
    os.killpg(pgid, signal.SIGTERM)
    run.add(
        Stage(
            "preempt",
            status="ok",
            detail=f"stopped conductor pid {pid} (session {pgid}); its batch stays QUEUED",
            salient=["stray QEMUs, if any: `vibeos-cli kill --stale`"],
        )
    )


def _hold(args, repo: Path) -> int:
    run = Run("land", "hold")
    lane = queue.lane_id(repo)
    reason = str(args.hold or "").strip()
    if not reason:
        run.add(Stage("hold", status="fail", detail='say WHY: `land --hold "master red: ..."`'))
        run.classification = "LAND_ABORT"
        return run.finalize(repo)
    cur = queue.read_hold(repo)
    if cur and not cur.get("auto") and cur.get("lane") != lane:
        run.classification = "LAND_ABORT"
        run.add(
            Stage(
                "hold",
                status="fail",
                detail="the line is already held by someone else — not taken over",
                salient=[queue.describe_hold(cur)],
            )
        )
        return run.finalize(repo)
    rec = queue.set_hold(repo, lane=lane, reason=reason)
    run.add(Stage("hold", status="ok", detail=queue.describe_hold(rec)))
    if getattr(args, "preempt", False):
        _preempt_conductor(run)
    else:
        h = hostlock.land_holder()
        if h:
            run.add(
                Stage(
                    "hold",
                    status="warn",
                    detail=f"a landing is in flight ({h.get('kind')}, pid {h.get('pid')}) — it "
                    "finishes its current batch first; `--preempt` stops it now",
                )
            )
    return run.finalize(repo)


def _release(args, repo: Path) -> int:
    run = Run("land", "release")
    rec = queue.clear_hold(repo)
    if rec is None:
        run.add(Stage("release", status="skip", detail="the line was not held"))
        return run.finalize(repo)
    run.add(Stage("release", status="ok", detail=f"released: {queue.describe_hold(rec)}"))
    if queue.pending(repo):
        _conduct(repo, run, argparse.Namespace(foreground=False))
    return run.finalize(repo)


def _withdraw(args, repo: Path) -> int:
    """Step this lane out of line without stopping anyone else (`--hold` stops everyone)."""
    run = Run("land", "withdraw")
    withdrawn, busy = queue.withdraw(repo)
    if withdrawn:
        run.add(
            Stage(
                "withdraw",
                status="ok",
                detail=f"withdrew {len(withdrawn)} submission(s); resubmit with `vibeos-cli land`",
                salient=[f"{s.tip[:12]} (queued {s.seq[:10]})" for s in withdrawn],
            )
        )
    if busy:
        run.add(
            Stage(
                "in-flight",
                status="warn",
                detail=(
                    f"{len(busy)} submission(s) are being GATED right now and were left alone: "
                    "they will land or be rejected. `vibeos-cli land --hold REASON --preempt` "
                    "stops the batch (and holds the whole line)"
                ),
                salient=[s.tip[:12] for s in busy],
            )
        )
    if not withdrawn and not busy:
        run.add(Stage("withdraw", status="skip", detail="this lane has nothing queued"))
    return run.finalize(repo)


def _evict(args, repo: Path) -> int:
    """Kick every other lane's queued work back to it. Only under THIS lane's own hold:
    eviction is for clearing the line, and a line that is not held can start a batch
    halfway through it."""
    run = Run("land", "evict")
    reason = str(args.evict or "").strip()
    if not reason:
        run.add(Stage("evict", status="fail", detail='say WHY: `land --evict "..."`'))
        return run.finalize(repo)
    hold = queue.effective_hold(repo)
    lane = queue.lane_id(repo)
    if not hold or hold.get("auto") or hold.get("lane") != lane:
        run.add(
            Stage(
                "evict",
                status="fail",
                detail="hold the line first: `vibeos-cli land --hold REASON --preempt`, then "
                "--evict, then land your fix, then --release",
            )
        )
        return run.finalize(repo)
    evicted, busy = queue.evict(repo, reason=reason, keep_lane=lane)
    run.add(
        Stage(
            "evict",
            status="ok" if evicted else "skip",
            detail=f"evicted {len(evicted)} submission(s)" if evicted else "nothing to evict",
            salient=[f"{s.lane} @ {s.tip[:12]}" for s in evicted],
        )
    )
    if busy:
        run.add(
            Stage(
                "in-flight",
                status="warn",
                detail=f"{len(busy)} submission(s) are being GATED by a live conductor and were "
                "left alone — `land --hold ... --preempt` stops it",
                salient=[f"{s.lane} @ {s.tip[:12]}" for s in busy],
            )
        )
    return run.finalize(repo)


def cmd_land_status(args, repo: Path) -> int:
    """`vibeos-cli land-status` — the merge queue at a glance: who is training, what is
    queued behind them, and recent verdicts (a rejected lane's triage pointer lives here).

    With `--tip <sha>`, answers about ONE commit instead (see `_explain_tip`)."""
    tip = getattr(args, "tip", None)
    if tip:
        return _explain_tip(repo, tip)
    holder = hostlock.land_holder()
    subs = queue.pending(repo)
    hold = queue.effective_hold(repo)
    mine = queue.lane_id(repo) if getattr(args, "mine", False) else None
    recent = queue.results(repo, lane=mine, limit=int(getattr(args, "limit", 8)))
    inflight = queue.read_inflight(repo)
    if getattr(args, "json", False):
        print(
            json.dumps(
                {
                    "conductor": holder,
                    "pending": [s.to_json() for s in subs],
                    "inflight": inflight,
                    "lock_queue": hostlock.land_queue(),
                    "hold": hold,
                    "results": recent,
                },
                indent=2,
            )
        )
        return 0
    if hold:
        print(queue.describe_hold(hold))
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
        gating = "  [GATING]" if s.seq in inflight else ""
        print(
            f"  queued #{i + 1}: {s.lane} @ {s.tip[:12]}  ({waited // 60}m{waited % 60:02d}s)"
            + gating
        )
    if not subs:
        print("  queue: empty")
    # A killed submit is INVISIBLE here otherwise, and "not listed" reads as "the conductor
    # has not reached me" -- so the agent waits on a position it never had. Say it instead.
    for rec in queue.abandoned_attempts(repo, lane=mine):
        ago = max(0, int(now - float(rec.get("started_at", now))))
        print(
            f"  NOT QUEUED: {rec.get('lane')} @ {str(rec.get('tip', ''))[:12]} — a `land` "
            f"started {ago // 60}m{ago % 60:02d}s ago DIED before it enqueued anything "
            f"(pid {rec.get('pid')} is gone). You are not in the queue; re-run "
            f"`./vibeos-cli land`. If the host was full, check `df -h /` first."
        )
    for rec in recent:
        # `or ""` keeps the key a str: a record with no outcome yet reads as
        # REJECTED either way (a None key simply misses), but dict.get is typed
        # for str keys and pyright rejects the Unknown | None that JSON gives back.
        outcome = rec.get("outcome") or ""
        tag = {
            "landed": "landed ",
            "superseded": "superseded",
            "withdrawn": "withdrawn",
            "evicted": "EVICTED",
        }.get(outcome, "REJECTED")
        extra = (
            f" as {rec.get('landed_as', '')[:12]}"
            if rec.get("outcome") == "landed"
            else f" ({rec.get('classification') or '?'}) {rec.get('reason', '')[:70]}"
        )
        print(f"  {tag}: {rec.get('lane')} @ {rec.get('tip', '')[:12]}{extra}")
    return 0


__all__ = ["STAMP_MAX_AGE_S", "LandAbort", "cmd_land_status", "fresh_full_stamp", "land"]
