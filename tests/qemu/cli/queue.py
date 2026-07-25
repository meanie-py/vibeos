"""The merge queue — a local Bors/Zuul-shaped train, with no PRs, no reviews, no daemon.

WHY, in one line: with the land lock (hostlock.land_lock) landings no longer RACE, but they
still SERIALIZE — throughput is 1/gate, and every waiting lane is a blocked agent. The queue
fixes both: N lanes ready become ONE gate, and nobody waits.

    lane A ── submit ─┐
    lane B ── submit ─┤                                  ┌─ A landed
    lane C ── submit ─┴─▶ [conductor] merge onto master ─▶ gate ×1 ─┼─ B landed
                                  (FIFO, conflicts bounced)      └─ C landed
    (all three agents went back to work ~2s after submitting)

THE FIVE THINGS THAT MAKE IT SAFE
  1. **The pushed tree is always the gated tree.** The conductor merges onto current
     origin/master in a dedicated integration worktree, gates THAT, and fast-forwards master
     to exactly the commit it gated. Nothing is ever "presumed" green from its parts — a
     batch of two individually-green lanes can still be red together, and that is precisely
     the class this catches BEFORE master (today it lands unnoticed).
  2. **Conflicts are bounced, never resolved.** Resolving someone's conflict for them is
     guessing at intent. The submission is rejected with the conflicting paths; the rest of
     the batch proceeds.
  3. **A red batch does not punish the innocent.** Recursive split: gate each half, land any
     green half IMMEDIATELY (so good work isn't held hostage by a neighbour), and recurse
     until the culprit is a single submission — which is then rejected with its
     classification and artifact dir, i.e. a triage pointer, not "try again".
  4. **No daemon.** The conductor is whichever agent submitted and found the land lock free.
     Exactly one agent gates at a time and it does the whole fleet's work. Nothing to
     supervise, nothing to restart, no process that can be "down".
  5. **Crash-safety is the kernel's job, as everywhere else here.** A submission is a REF
     plus a JSON record on disk; the conductor's claim is an flock that dies with it. Kill a
     conductor mid-batch and its submissions are still queued — the next conductor redoes them.
     A submission leaves the queue only when it has landed or been rejected, so the failure
     mode is repeated work, never a silent drop.

ORPHAN WINDOW (the one honest wart): a submission arriving between the conductor's final
empty-queue scan and its release of the lock has no conductor. It waits until the next
submission elects one. Microseconds wide, self-healing, and `land-status` shows it as
pending with nobody landing — so it is visible rather than mysterious. `land --drain` forces
a drain if you ever see one sitting.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path

from . import cachecfg, hostlock

# Submissions per gate. A green batch is one gate for N landings; a RED batch costs a
# recursive split, so the cap bounds the worst case (a 4-batch with one bad apple is
# 1 + 2 gates, still fewer than 4 serial ones). Raise when the fleet is mostly-green.
BATCH_CAP = max(1, int(os.environ.get("VIBEOS_QUEUE_BATCH", "4")))
INTEGRATION_BRANCH = "integration"
INTEGRATION_DIRNAME = "vibeos-integration"


class QueueError(RuntimeError):
    """A queue operation could not be completed. Always carries an operator-facing reason —
    never a silent no-op, because a swallowed submission is a lost landing."""


@dataclass
class Submission:
    seq: str  # "<time_ns>-<lane>" → lexical sort IS arrival order
    lane: str  # "<branch>@<worktree>" — a lane is a branch in a worktree
    tip: str  # the commit to land
    worktree: str
    submitted_at: int
    path: Path = field(compare=False, default=Path())

    @property
    def ref(self) -> str:
        """Queue ref. Keeps the tip's objects reachable even if the lane branch is deleted,
        rewound, or its worktree pruned while the submission is queued."""
        return f"refs/vibeos/queue/{self.seq}"

    def to_json(self) -> dict:
        return {
            "seq": self.seq,
            "lane": self.lane,
            "tip": self.tip,
            "worktree": self.worktree,
            "submitted_at": self.submitted_at,
        }

    @staticmethod
    def from_json(d: dict, path: Path) -> Submission:
        return Submission(
            seq=d["seq"],
            lane=d.get("lane", "?"),
            tip=d["tip"],
            worktree=d.get("worktree", "?"),
            submitted_at=int(d.get("submitted_at", 0)),
            path=path,
        )


def git(repo: Path, *args: str, check: bool = True, timeout: float = 900) -> str:
    cp = subprocess.run(
        ["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=timeout
    )
    if check and cp.returncode != 0:
        raise QueueError(f"git {' '.join(args)} failed ({cp.returncode}): {cp.stderr.strip()}")
    return cp.stdout.strip()


def git_ok(repo: Path, *args: str) -> bool:
    return (
        subprocess.run(
            ["git", *args], cwd=str(repo), capture_output=True, text=True, timeout=900
        ).returncode
        == 0
    )


def _common_dir(repo: Path) -> Path:
    common = Path(git(repo, "rev-parse", "--git-common-dir"))
    return common if common.is_absolute() else (repo / common).resolve()


def queue_dir(repo: Path) -> Path:
    """Queue state lives in the SHARED git dir, so every worktree sees one queue."""
    return _common_dir(repo) / "vibeos-queue"


def _pending_dir(repo: Path) -> Path:
    return queue_dir(repo) / "pending"


def _results_dir(repo: Path) -> Path:
    return queue_dir(repo) / "results"


def lane_id(repo: Path) -> str:
    branch = git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    return f"{branch}@{repo.resolve().name}"


# ── submit ──────────────────────────────────────────────────────────────────


def submit(repo: Path, *, tip: str | None = None, lane: str | None = None) -> Submission:
    """Enqueue a lane tip. Writes the ref FIRST (so the objects are pinned before anything
    can observe the record) and only then the record — a conductor must never see a submission
    whose tip could be GC'd."""
    tip = tip or git(repo, "rev-parse", "HEAD")
    lane = lane or lane_id(repo)
    seq = f"{time.time_ns():020d}-{lane.replace('/', '_').replace(' ', '_')}"
    sub = Submission(
        seq=seq,
        lane=lane,
        tip=tip,
        worktree=str(repo.resolve()),
        submitted_at=int(time.time()),
    )
    git(repo, "update-ref", sub.ref, tip)
    d = _pending_dir(repo)
    d.mkdir(parents=True, exist_ok=True)
    tmp = d / f".{seq}.tmp"
    tmp.write_text(json.dumps(sub.to_json(), indent=1))
    tmp.replace(d / f"{seq}.json")  # atomic: visible ⇒ complete
    sub.path = d / f"{seq}.json"
    return sub


def pending(repo: Path) -> list[Submission]:
    """Queued submissions in ARRIVAL order (seq is time-prefixed, so lexical == FIFO)."""
    d = _pending_dir(repo)
    out: list[Submission] = []
    try:
        names = sorted(p for p in d.iterdir() if p.suffix == ".json")
    except OSError:
        return out
    for p in names:
        try:
            out.append(Submission.from_json(json.loads(p.read_text()), p))
        except (OSError, ValueError, KeyError):
            continue  # unreadable record: leave it for a human, don't crash the train
    return out


def resolve(
    repo: Path,
    sub: Submission,
    *,
    outcome: str,
    reason: str = "",
    classification: str = "",
    artifact: str = "",
    landed_as: str = "",
    batch: list[str] | None = None,
) -> None:
    """Record a verdict and remove the submission from the queue. Every exit path from the
    train goes through here: a submission that leaves the queue WITHOUT a result would be a
    silent drop, which is the one outcome an agent cannot recover from."""
    rd = _results_dir(repo)
    rd.mkdir(parents=True, exist_ok=True)
    rec = {
        **sub.to_json(),
        "outcome": outcome,  # landed | rejected
        "reason": reason,
        "classification": classification,
        "artifact": artifact,
        "landed_as": landed_as,
        "batch": batch or [],
        "resolved_at": int(time.time()),
    }
    tmp = rd / f".{sub.seq}.tmp"
    tmp.write_text(json.dumps(rec, indent=1))
    tmp.replace(rd / f"{sub.seq}.json")
    # Drop the pending record before the ref: a record-less ref is inert garbage the next
    # prune collects, whereas a ref-less record would point at a tip that can be GC'd.
    try:
        sub.path.unlink()
    except OSError:
        pass
    git(repo, "update-ref", "-d", sub.ref, check=False)


def results(repo: Path, *, lane: str | None = None, limit: int = 20) -> list[dict]:
    d = _results_dir(repo)
    try:
        names = sorted((p for p in d.iterdir() if p.suffix == ".json"), reverse=True)
    except OSError:
        return []
    out = []
    for p in names:
        try:
            rec = json.loads(p.read_text())
        except (OSError, ValueError):
            continue
        if lane and rec.get("lane") != lane:
            continue
        out.append(rec)
        if len(out) >= limit:
            break
    return out


def prune_results(repo: Path, *, max_age_s: int = 7 * 24 * 3600) -> None:
    """Results are a mailbox, not a ledger (artifacts.py's ledger is the durable record).
    A week is long enough for an agent to come back and read its rejection."""
    now = time.time()
    try:
        for p in _results_dir(repo).iterdir():
            if p.suffix == ".json" and now - p.stat().st_mtime > max_age_s:
                p.unlink()
    except OSError:
        pass


# ── the integration worktree ────────────────────────────────────────────────


def integration_path(repo: Path) -> Path:
    return cachecfg.vibeos_root(repo) / INTEGRATION_DIRNAME


def ensure_integration_worktree(repo: Path) -> Path:
    """The dedicated tree the train builds in. Created on first use.

    Dedicated, rather than gating in the conductor's own worktree, for two reasons: its
    ccache/sccache and build dirs stay HOT across every batch (so a batch gates as fast as
    the machine allows, and it is always the same tree, so no cache thrash from whichever
    lane happens to be conductor), and no lane's checkout is ever disturbed by a train that
    dies mid-batch. Build dirs are position-independent here (everything builds at the
    canonical /tmp/vibeos mount), so this tree is exactly as buildable as any other."""
    path = integration_path(repo)
    if (path / ".git").exists():
        return path
    path.parent.mkdir(parents=True, exist_ok=True)
    git(repo, "fetch", "origin")
    # -B: create or reset the scratch branch. `integration` is never mainline, so resetting
    # it is not a rewind of anything shared (the reference-transaction guard covers master).
    git(repo, "worktree", "add", "-B", INTEGRATION_BRANCH, str(path), "origin/master")
    _seed_gitignored_boot_assets(repo, path)
    return path


def _seed_gitignored_boot_assets(repo: Path, dest: Path) -> None:
    """arm64 boots need `qemu/QEMU_EFI.fd` + `efivars.fd`, which are gitignored — a fresh
    worktree has neither, and without them the arm64 lane of every batch fails as an
    environment error rather than a code verdict. boot.img/ramdisk.img are regenerated by
    the build's mkramdisk step, so only the firmware needs seeding."""
    for name in ("QEMU_EFI.fd", "efivars.fd"):
        src = repo / "qemu" / name
        dst = dest / "qemu" / name
        if src.is_file() and not dst.is_file():
            dst.parent.mkdir(parents=True, exist_ok=True)
            try:
                shutil.copy2(src, dst)
            except OSError:
                pass


# ── the train ───────────────────────────────────────────────────────────────


@dataclass
class BatchResult:
    ok: bool
    tip: str = ""
    classification: str = ""
    artifact: str = ""
    salient: str = ""
    conflicted: list[Submission] = field(default_factory=list)
    merged: list[Submission] = field(default_factory=list)


def build_batch(repo: Path, wt: Path, subs: list[Submission], log) -> BatchResult:
    """Reset the integration tree onto current origin/master and merge `subs` in order.
    Conflicting submissions are collected (to be rejected) and skipped, not resolved."""
    git(wt, "fetch", "origin")
    git(wt, "reset", "--hard", "origin/master")
    git(wt, "clean", "-fdq")  # a previous batch's mid-merge debris must not leak into this one
    res = BatchResult(ok=False)
    for sub in subs:
        # The ref, not the branch name: the lane may have moved on since it submitted, and
        # the queue's promise is to land the tip that WAS submitted.
        cp = subprocess.run(
            [
                "git",
                "merge",
                "--no-ff",
                "--no-edit",
                "-m",
                f"queue: {sub.lane} {sub.tip[:12]}",
                sub.tip,
            ],
            cwd=str(wt),
            capture_output=True,
            text=True,
            timeout=900,
        )
        if cp.returncode != 0:
            paths = git(wt, "diff", "--name-only", "--diff-filter=U", check=False)
            subprocess.run(["git", "merge", "--abort"], cwd=str(wt), capture_output=True)
            sub_paths = paths.replace("\n", ", ")[:400] or cp.stderr.strip()[:400]
            log(f"queue: CONFLICT {sub.lane} {sub.tip[:12]} — {sub_paths}")
            res.conflicted.append(sub)
            continue
        res.merged.append(sub)
    res.tip = git(wt, "rev-parse", "HEAD")
    return res


def gate_batch(wt: Path, log, *, retries: int = 2) -> tuple[bool, str, str, str]:
    """Run the FULL gate in the integration tree. Returns (ok, classification, artifact,
    salient). This is the only place a batch is judged — no batch is ever presumed green.

    The gate gets its OWN artifact root. It used to inherit the shared /tmp/vibeos-cli and
    the batch's artifact was then found by diffing that directory before/after — which
    silently picked up a CONCURRENT lane's gate (2026-07-25: the first real batch reported
    vibeos-fex's green gate as its own). Artifact provenance has to be by construction, not
    by timing."""
    root = (
        Path(os.environ.get("VIBEOS_ARTIFACT_ROOT", "/tmp/vibeos-cli"))
        / f"queue-batch-{time.time_ns():020d}"
    )
    root.mkdir(parents=True, exist_ok=True)
    cmd = [str(wt / "vibeos-cli"), "gate", "--retries", str(retries)]
    cp = subprocess.run(
        cmd,
        cwd=str(wt),
        capture_output=True,
        text=True,
        env=dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root)),
    )
    fresh = sorted(root.glob("*-gate-*"))
    artifact = str(fresh[-1]) if fresh else ""
    classification, salient = "", ""
    if artifact:
        try:
            s = json.loads((Path(artifact) / "summary.json").read_text())
            classification = s.get("classification", "")
            for st in s.get("stages", []):
                if st.get("status") == "fail":
                    salient = f"{st['name']}: {st.get('detail', '')}"
                    break
        except (OSError, ValueError):
            pass
    ok = cp.returncode == 0
    why = "" if ok else f" ({classification or cp.returncode})"
    log(f"queue: gate {'GREEN' if ok else 'RED'}{why}")
    return ok, classification, artifact, salient


class NoVerdict(QueueError):
    """The train could not render a judgement about the submissions. They stay QUEUED and the
    train halts. REJECT ONLY ON EVIDENCE is the whole rule; everything else is a requeue."""


class BaseRed(NoVerdict):
    """The BASE (origin/master with nothing merged) does not pass the gate. Nothing can land
    on it, and no submission may be blamed for it."""


class Inconclusive(NoVerdict):
    """The gate failed for a reason that is not evidence about the code: the environment."""


# A red gate with one of these classifications says nothing about the submission — the host
# was starved, QEMU never launched, the suite hung on infrastructure. Rejecting a lane for one
# is the same error as rejecting it for a broken base, and it is worse than useless: the lane
# goes hunting a bug that does not exist. An EMPTY classification is included deliberately —
# it means the gate died without writing a verdict, which is the least conclusive outcome there
# is. Conclusive reds (BUILD_FAIL / LINT_FAIL / BUGCHECK / RUST_PANIC / FAIL_MARKER / ...) are
# evidence and DO reject.
INCONCLUSIVE = {"", "LAUNCH_FAIL", "HOST_STARVED", "NET_HANG"}


class BaseCheck:
    """Per-drain context: the lazily-evaluated 'is origin/master itself green?' verdict,
    memoized, plus the lanes this drain has already landed (so a later rejection can say what
    it was actually gated against).

    WHY (2026-07-25, the first real batch): mainline was red — `fltmgr-rust` failed to
    compile for i386 — and the train dutifully split the batch, reached one submission per
    half, and REJECTED BOTH. One of them was a docs-only change, which cannot break a build.
    A merge queue that blames the base's breakage on whoever happened to be in the batch is
    worse than no queue: it teaches lanes to distrust their own verdicts.

    Lazy on purpose: a green batch never pays for this. It is evaluated only when a lone
    submission comes back red, which is exactly when 'guilty or framed?' has to be answered.
    """

    def __init__(self) -> None:
        self.verdict: bool | None = None
        self.artifact = ""
        self.classification = ""
        self.salient = ""
        self.landed: list[str] = []  # lanes landed EARLIER in this same drain

    def is_green(self, repo: Path, wt: Path, log, *, retries: int) -> bool:
        if self.verdict is not None:
            return self.verdict
        log("queue: a lone submission failed — gating the BASE alone to see who is at fault")
        git(wt, "fetch", "origin")
        git(wt, "reset", "--hard", "origin/master")
        git(wt, "clean", "-fdq")
        ok, cls, art, sal = gate_batch(wt, log, retries=retries)
        self.verdict, self.classification, self.artifact, self.salient = ok, cls, art, sal
        log(
            "queue: BASE is GREEN — the submission really is the culprit"
            if ok
            else f"queue: BASE ITSELF IS RED ({cls}) — mainline is broken; no lane is at fault"
        )
        return ok


def land_subset(
    repo: Path,
    wt: Path,
    subs: list[Submission],
    log,
    *,
    retries: int = 2,
    base: BaseCheck | None = None,
) -> None:
    """Gate `subs` as one batch; on red, split and recurse.

    Landing a green half IMMEDIATELY (rather than after the whole bisect) is deliberate: the
    innocent half is already proven green against current master, and holding it hostage
    while its neighbour's culprit is isolated is exactly the "everyone waits on one lane"
    failure this queue exists to remove."""
    if not subs:
        return
    base = base or BaseCheck()  # one per drain; shared down the whole split tree
    res = build_batch(repo, wt, subs, log)
    for sub in res.conflicted:
        resolve(
            repo,
            sub,
            outcome="rejected",
            reason="merge conflict with master — resolve it in your lane and resubmit",
            classification="CONFLICT",
        )
    subs = res.merged
    if not subs:
        return
    ok, classification, artifact, salient = gate_batch(wt, log, retries=retries)
    if ok:
        # We hold the land lock for the whole drain, and every landing path takes it, so
        # master cannot have moved under us — assert it rather than assume it.
        if not git_ok(wt, "merge-base", "--is-ancestor", "origin/master", "HEAD"):
            raise QueueError(
                "master moved during a batch gate while the land lock was held — something "
                "pushed outside the queue; re-run the drain"
            )
        git(wt, "push", "origin", f"{res.tip}:refs/heads/master")
        log(f"queue: LANDED {len(subs)} submission(s) as {res.tip[:12]}")
        base.landed.extend(s.lane for s in subs)
        for sub in subs:
            resolve(
                repo,
                sub,
                outcome="landed",
                landed_as=res.tip,
                artifact=artifact,
                batch=[s.lane for s in subs],
            )
        return
    if classification in INCONCLUSIVE:
        # Not evidence. Splitting would just burn gates re-discovering that the host is busy.
        raise Inconclusive(
            f"the gate could not render a verdict ({classification or 'no classification'}) — "
            f"{salient or 'environment failure'}. {len(subs)} submission(s) stay QUEUED; "
            f"nothing was rejected. Artifact: {artifact or '(none)'}"
        )
    if len(subs) == 1:
        sub = subs[0]
        # GUILTY OR FRAMED? A lone red submission is only the culprit if the BASE is green.
        # Skipping this check is how the first real batch rejected a docs-only change for a
        # build failure that was already on mainline.
        if not base.is_green(repo, wt, log, retries=retries):
            raise BaseRed(
                f"origin/master itself fails the gate ({base.classification or 'red'}) — "
                f"{base.salient or 'see the artifact'}. No submission was rejected (they are "
                f"not at fault) and the train is halted: nothing can land on a red base. "
                f"Artifact: {base.artifact or '(none)'}"
            )
        log(f"queue: REJECTED {sub.lane} {sub.tip[:12]} — {classification or 'gate red'}")
        # Say what it was gated AGAINST. "red on this submission alone" is misleading when an
        # earlier half of the same batch has already landed: the lane may be green against the
        # master it branched from and red only against the one that now includes its neighbour.
        # Without this the lane hunts a phantom bug in its own diff.
        against = git(wt, "rev-parse", "origin/master", check=False)[:12] or "master"
        note = f" (gated against master {against}"
        note += (
            f", which now includes this drain's earlier landing(s): {', '.join(base.landed)})"
            if base.landed
            else ")"
        )
        resolve(
            repo,
            sub,
            outcome="rejected",
            reason=(salient or "the full gate was red on this submission alone") + note,
            classification=classification or "GATE_RED",
            artifact=artifact,
        )
        return
    mid = len(subs) // 2
    log(f"queue: red batch of {len(subs)} — splitting {mid}/{len(subs) - mid} to find the culprit")
    land_subset(repo, wt, subs[:mid], log, retries=retries, base=base)
    land_subset(repo, wt, subs[mid:], log, retries=retries, base=base)


def already_landed(repo: Path, sub: Submission) -> bool:
    """A tip already contained in origin/master needs no gate — it landed in someone else's
    batch (or the lane pushed it directly). Resolve it as landed rather than re-gating."""
    return git_ok(repo, "merge-base", "--is-ancestor", sub.tip, "origin/master")


def drain(repo: Path, log, *, retries: int = 2, cap: int = BATCH_CAP) -> int:
    """Process the queue until it is empty. CALLER MUST HOLD hostlock.land_lock.

    Loops rather than doing one batch: submissions that arrive while a batch gates are the
    normal case (that's where batching comes from), and draining them in the same conductor
    saves an election.
    """
    wt = ensure_integration_worktree(repo)
    prune_results(repo)
    batches = 0
    while True:
        subs = pending(repo)
        if not subs:
            return batches
        git(repo, "fetch", "origin")
        # Free landings first: anything already in master leaves the queue without a gate.
        fresh = []
        for sub in subs:
            if already_landed(repo, sub):
                log(f"queue: {sub.lane} {sub.tip[:12]} already in master — no gate needed")
                resolve(repo, sub, outcome="landed", landed_as=sub.tip, reason="already in master")
            else:
                fresh.append(sub)
        if not fresh:
            continue
        batch = fresh[:cap]
        log(
            f"queue: batch of {len(batch)} — "
            + ", ".join(f"{s.lane}@{s.tip[:8]}" for s in batch)
            + (f" ({len(fresh) - len(batch)} more queued)" if len(fresh) > len(batch) else "")
        )
        try:
            land_subset(repo, wt, batch, log, retries=retries)
        except NoVerdict as e:
            # Everything still queued STAYS queued — it was never judged. Halting is the only
            # honest move: on a red base (or a starved host) every batch would fail for a
            # reason that has nothing to do with the lanes in it.
            log(f"queue: TRAIN HALTED — {e}")
            raise
        batches += 1


def try_become_conductor(repo: Path, log, *, retries: int = 2) -> bool:
    """Take the land lock if it is FREE and drain; otherwise return False immediately.

    The election is the lock: no daemon, no leader file, no heartbeat. `timeout=0` is what
    makes submitting non-blocking — an agent either becomes the conductor (and does the
    fleet's work) or leaves, knowing a conductor already exists."""
    try:
        with hostlock.land_lock(f"conductor:{lane_id(repo)}", timeout=0.0, repo=str(repo)):
            drain(repo, log, retries=retries)
        return True
    except hostlock.LandLockBusy:
        return False
