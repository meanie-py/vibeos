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

THE TRAIN (2026-09-26): gates are serial, so one slow gate used to stall every lane behind
it. At VIBEOS_QUEUE_DEPTH=2 (the default) the conductor gates the NEXT batch on top of the
current one while the current one is still gating, in a second integration worktree:

    car A: master + A          ── gate ──▶ push A
    car B: master + A + B      ── gate ──────────▶ push B   (only if A landed WHOLE)

B's verdict is about exactly the tree master will be once A lands, so it is trusted only
then. If A goes red, B's gate is joined and thrown away and B stays queued, unjudged: it
is neither landed nor blamed for a tree that never existed (see `_pipelined`). Depth 1 is
the serial train. `land --withdraw` takes a lane's own submissions out of line.

ORPHAN WINDOW (the one honest wart): a submission arriving between the conductor's final
empty-queue scan and its release of the lock has no conductor. It waits until the next
submission elects one. Microseconds wide, self-healing, and `land-status` shows it as
pending with nobody landing — so it is visible rather than mysterious. `land --drain` forces
a drain if you ever see one sitting.
"""

from __future__ import annotations

import contextlib
import fcntl
import json
import os
import re
import shutil
import subprocess
import time
from collections.abc import Mapping
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path
from types import MappingProxyType
from typing import NamedTuple

from . import cachecfg, hostlock

# Submissions per gate. A green batch is one gate for N landings; a RED batch costs a
# recursive split, so the cap bounds the worst case (a 4-batch with one bad apple is
# 1 + 2 gates, still fewer than 4 serial ones). Raise when the fleet is mostly-green.
BATCH_CAP = max(1, int(os.environ.get("VIBEOS_QUEUE_BATCH", "4")))
# Cars gated AT ONCE. 1 is the serial train; 2 gates the next batch on top of the one
# ahead while that one is still gating (see `drain`). Each car beyond the first has its own
# integration worktree and costs a second concurrent gate's worth of host.
PIPELINE_DEPTH = max(1, int(os.environ.get("VIBEOS_QUEUE_DEPTH", "2")))
INTEGRATION_BRANCH = "integration"
INTEGRATION_DIRNAME = "vibeos-integration"


def integration_names(slot: int) -> tuple[str, str]:
    """(worktree dirname, branch) for pipeline slot `slot`. Slot 0 keeps the historical
    names, so the tree every conductor has been warming stays the first car's."""
    if slot == 0:
        return INTEGRATION_DIRNAME, INTEGRATION_BRANCH
    return f"{INTEGRATION_DIRNAME}-{slot + 1}", f"{INTEGRATION_BRANCH}-{slot + 1}"


def is_integration_dirname(name: str) -> bool:
    return name == INTEGRATION_DIRNAME or (
        name.startswith(INTEGRATION_DIRNAME + "-")
        and name[len(INTEGRATION_DIRNAME) + 1 :].isdigit()
    )


class QueueError(RuntimeError):
    """A queue operation could not be completed. Always carries an operator-facing reason —
    never a silent no-op, because a swallowed submission is a lost landing."""


@dataclass
class Submission:
    # "<ns>-<lane>" → lexical sort IS arrival order. Allocated by _next_seq(), which
    # bumps past the queue's high-water mark: the wall clock alone is not monotonic.
    seq: str
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


def _attempts_dir(repo: Path) -> Path:
    return queue_dir(repo) / "attempts"


# ---------------------------------------------------------------------------------------
# STOP THE LINE. A hold is one JSON file in the shared queue dir: while it exists no
# conductor starts a batch, submissions still QUEUE (nobody loses their place, nobody
# retry-loops), and only the holder may `land --direct`.
#
# WHY (2026-09-25, a whole night): mainline was red and the queue kept going. Every new
# submission elected a conductor, every conductor gated a batch (3 cold builds + boots),
# split it, gated the base alone, halted on BaseRed -- and the next submission started the
# same cycle, while the load it generated turned a 10 ms test timing assumption into a
# second, unrelated red. The fix lane's own gate queued behind that churn for hours. The
# only way to stop it was killing conductors by hand and messaging every lane to stop
# submitting; `wsl --shutdown` is what finally did it.
#
# Two kinds:
#   * MANUAL (`land --hold REASON`): an agent stops the line to land a fix. Lasts until
#     `land --release`; `--preempt` also stops a conductor that is mid-batch.
#   * AUTO: the train halted on BaseRed. Pinned to the red master sha, and cleared by itself
#     the moment origin/master moves, i.e. when a fix lands. While it stands ANY lane may
#     `land --direct` (the fix can come from anywhere, and the direct gate proves it first).
# ---------------------------------------------------------------------------------------

HOLD_FILE = "HOLD.json"


def hold_path(repo: Path) -> Path:
    return queue_dir(repo) / HOLD_FILE


def read_hold(repo: Path) -> dict | None:
    """The raw hold record, or None. An unreadable file IS a hold -- failing open here is
    how the line restarts under someone who stopped it."""
    p = hold_path(repo)
    if not p.exists():
        return None
    try:
        rec = json.loads(p.read_text() or "{}")
    except (OSError, ValueError):
        why = f"unreadable hold file {p} -- inspect it, then `land --release`"
        return {"lane": "?", "reason": why}
    return rec if isinstance(rec, dict) else {"lane": "?", "reason": f"malformed {p}"}


def set_hold(repo: Path, *, lane: str, reason: str, auto: bool = False, base: str = "") -> dict:
    rec = {
        "lane": lane,
        "reason": reason,
        "since": time.time(),
        "auto": auto,
        "base": base,
        "pid": os.getpid(),
    }
    p = hold_path(repo)
    p.parent.mkdir(parents=True, exist_ok=True)
    tmp = p.with_suffix(f".tmp{os.getpid()}")
    tmp.write_text(json.dumps(rec, indent=2))
    os.replace(tmp, p)  # atomic: a reader sees the old hold or the new one, never half
    return rec


def clear_hold(repo: Path) -> dict | None:
    rec = read_hold(repo)
    hold_path(repo).unlink(missing_ok=True)
    return rec


def effective_hold(repo: Path) -> dict | None:
    """The hold in force, or None. An AUTO hold that is pinned to a master sha lapses as soon
    as origin/master is somewhere else: the red base it described is gone. Reads the LOCAL
    origin/master ref, so callers that care fetch first (drain does)."""
    rec = read_hold(repo)
    if rec is None:
        return None
    if rec.get("auto") and rec.get("base"):
        now = git(repo, "rev-parse", "origin/master", check=False)
        if now and now != rec["base"]:
            clear_hold(repo)
            return None
    return rec


def describe_hold(rec: dict) -> str:
    age = max(0, int(time.time() - float(rec.get("since") or time.time())))
    kind = "AUTO (red base)" if rec.get("auto") else f"by {rec.get('lane', '?')}"
    return f"LINE HELD {kind}, {age // 60}m{age % 60:02d}s ago: {rec.get('reason', '')}"


def may_land_direct(rec: dict | None, lane: str) -> bool:
    return rec is None or bool(rec.get("auto")) or rec.get("lane") == lane


class LineHeld(QueueError):
    """The line is held; the conductor stops before starting a batch. Submissions stay
    QUEUED -- this is not a verdict about anyone."""


def begin_attempt(repo: Path, lane: str, tip: str) -> Path:
    """Record that a submit is IN PROGRESS, so a killed one leaves evidence.

    `land` runs a multi-minute preflight before it enqueues anything. A kill in that window
    (the host filling its disk does it, and the harness reports it as low memory) leaves no
    queue entry and no error, and `land-status` omitting a lane is indistinguishable from a
    conductor that has not reached it — so the agent waits on a position it never had. Ask
    nobody to remember that: leave a marker and let the tool say it.

    Removed by `end_attempt` on every ordinary exit INCLUDING an abort, via `finally`. A
    SIGKILL is precisely the case that skips `finally`, which is what makes a surviving
    marker mean "killed" rather than "failed".
    """
    d = _attempts_dir(repo)
    d.mkdir(parents=True, exist_ok=True)
    path = d / f"{lane.replace('/', '_')}.json"
    path.write_text(
        json.dumps({"lane": lane, "tip": tip, "pid": os.getpid(), "started_at": time.time()}),
        encoding="utf-8",
    )
    return path


def end_attempt(path: Path | None) -> None:
    if path is not None:
        path.unlink(missing_ok=True)


def _proc_started_at(pid: int) -> float | None:
    """Wall-clock start time of `pid` from /proc, or None if it cannot be read.

    Field 22 of /proc/<pid>/stat is the process start in clock ticks since boot; /proc/stat's
    `btime` is boot in epoch seconds. The comm field can contain spaces and parentheses, so
    split after the LAST ')' rather than tokenizing the whole line.
    """
    try:
        raw = Path(f"/proc/{pid}/stat").read_text()
        fields = raw[raw.rindex(")") + 2 :].split()
        ticks = float(fields[19])  # field 22 overall, 20th after the comm
        hz = os.sysconf("SC_CLK_TCK")
        for line in Path("/proc/stat").read_text().splitlines():
            if line.startswith("btime "):
                return float(line.split()[1]) + ticks / hz
    except (OSError, ValueError, IndexError):
        return None
    return None


def _pid_alive(pid: int, *, since: float | None = None) -> bool:
    """Is `pid` running, and is it plausibly still OUR process?

    `since` is the marker's own `started_at`. A pid that is alive but STARTED AFTER the marker
    was written is a recycled pid belonging to something else -- on a host churning through
    thousands of build processes that is not exotic -- and treating it as alive would suppress
    the marker permanently. Both this and the malformed-pid guard in `abandoned_attempts` fail
    toward REPORTING, because silence is the bug this whole mechanism exists to remove.
    """
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        pass  # exists, owned by someone else
    if since is not None:
        started = _proc_started_at(pid)
        # 2s of slack: the marker is written moments after the process starts, and the two
        # clocks (wall clock vs boot + ticks) do not agree to better than that.
        if started is not None and started > since + 2.0:
            return False
    return True


def _as_float(v: object) -> float | None:
    try:
        return float(v)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return None


def abandoned_attempts(repo: Path, *, lane: str | None = None) -> list[dict]:
    """Submits that started, never enqueued, and whose process is gone.

    A live pid is an in-flight submit, not an abandoned one, so it is excluded — this must
    not cry wolf at the agent watching its own `land` run.
    """
    out: list[dict] = []
    queued = {s.lane for s in pending(repo)}
    d = _attempts_dir(repo)
    if not d.is_dir():
        return out
    for f in sorted(d.glob("*.json")):
        try:
            rec = json.loads(f.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if lane is not None and rec.get("lane") != lane:
            continue
        if rec.get("lane") in queued:
            continue
        pid = rec.get("pid")
        if not isinstance(pid, int) or pid <= 0:
            # An unusable pid must REPORT, not suppress. `os.kill(-1, 0)` succeeds -- it is
            # the BROADCAST form -- so a malformed record (a marker truncated by the disk
            # filling, which is exactly the scenario that produces one) would otherwise read
            # as "alive" forever and stay silent. A marker existing at all means a submit
            # started. Never let a pid from a file reach os.kill unvalidated: signal 0 is
            # harmless, but a broadcast kill in fleet tooling is one edit away.
            out.append(rec)
            continue
        if _pid_alive(pid, since=_as_float(rec.get("started_at"))):
            continue
        out.append(rec)
    return out


def lane_id(repo: Path) -> str:
    branch = git(repo, "rev-parse", "--abbrev-ref", "HEAD")
    return f"{branch}@{repo.resolve().name}"


def _next_seq(repo: Path, lane_tag: str) -> str:
    """Allocate a seq that sorts strictly AFTER every submission already queued.

    seq is "<ns>-<lane>" and the whole FIFO guarantee rests on lexical order being
    arrival order. The numeric half is `time.time_ns()`, i.e. CLOCK_REALTIME — and
    CLOCK_REALTIME is not monotonic. It STEPS.

    Not hypothetical: reproduced on this fleet at ~1.3% under parallel load. A lane
    submitted 2.59 SECONDS "before" one that was already on disk —

        lane-a  01785085499633383341   submitted first, LARGER
        lane-b  01785085497039700594   submitted second, SMALLER by 2,593,682,747ns

    — because WSL2 resynced the guest clock to the Windows host between the two
    calls. The later arrival then sorts first and silently jumps the queue. It also
    surfaced as a ~1-in-80 failure of test_queue_is_fifo_by_arrival, which is how it
    was found; treating that as "a flaky test" and quieting it would have left a live
    fairness bug in the thing that decides whose work lands.

    Fix is a Lamport-style bump: keep wall-clock ns (it is meaningful and readable)
    but never hand out a value that would sort at or before something already queued.
    The 20-digit zero-pad makes the numeric prefix dominate the comparison, so
    max+1 beats every existing entry regardless of lane name.

    Deliberately scanned over PENDING only, not results. Ordering only has to hold
    among submissions competing for a conductor, and pending-only SELF-HEALS: if the
    clock ever jumps far forward, the queue drains and the next submission is back on
    real time. Including results would pin every future seq to the bad value until
    the history was pruned.
    """
    now = time.time_ns()
    floor = 0
    try:
        for p in _pending_dir(repo).iterdir():
            if p.suffix != ".json":
                continue
            head = p.name.split("-", 1)[0]
            if head.isdigit():
                floor = max(floor, int(head))
    except OSError:
        pass  # no queue dir yet: nothing to sort after
    return f"{max(now, floor + 1):020d}-{lane_tag}"


# ── submit ──────────────────────────────────────────────────────────────────


def submit(repo: Path, *, tip: str | None = None, lane: str | None = None) -> Submission:
    """Enqueue a lane tip. Writes the ref FIRST (so the objects are pinned before anything
    can observe the record) and only then the record — a conductor must never see a submission
    whose tip could be GC'd.

    Idempotent per (lane, tip), and SUPERSEDES this lane's older pending tips that the new one
    already contains. Both were observed in the first live batch: submitting the same canary tip
    twice put it in the batch twice, and a lane's earlier tip sat alongside its own descendant.
    Neither is unsafe (a descendant's merge subsumes its ancestor's), but they are noise, and a
    superseded ancestor can be REJECTED by a split while its descendant lands — a verdict about
    a commit the lane has already moved past. Deduping here rather than only in the CLI, because
    the CLI is not the only caller."""
    tip = tip or git(repo, "rev-parse", "HEAD")
    lane = lane or lane_id(repo)
    existing = [s for s in pending(repo) if s.lane == lane]
    for prev in existing:
        if prev.tip == tip:
            return prev  # already queued, byte-identical — a no-op, not a second submission
    for prev in existing:
        # `prev.tip` is contained in the new tip ⇒ landing the new one lands it too.
        if git_ok(repo, "merge-base", "--is-ancestor", prev.tip, tip):
            resolve(
                repo,
                prev,
                outcome="superseded",
                reason=f"this lane submitted {tip[:12]}, which already contains it",
            )
    seq = _next_seq(repo, lane.replace("/", "_").replace(" ", "_"))
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
        "outcome": outcome,  # landed | rejected | superseded | withdrawn | evicted
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


# ── in flight, and stepping out of line ────────────────────────────────────
#
# WHY (2026-09-26): upstream-sync submitted a change that forced a full kernel recompile —
# hours per arch — and then learned it would never gate inside the build timeout. It had no
# way to step aside: `--hold` stops EVERYONE, and nothing removed one lane's own submission.
# So it sat at the head of the queue until the gate timed out, ~80 minutes, with nine lanes
# behind it. `withdraw` is that missing exit.
#
# A submission a conductor has already TAKEN into a batch cannot be withdrawn: its merge is
# in the tree being gated, and pulling it out would push a tree nobody gated. The conductor
# therefore publishes what it has taken (INFLIGHT.json), and both "take a batch" and
# "withdraw" run under one flock, so a withdrawal either happens before the take or is
# refused after it. `land --hold --preempt` remains the way to stop a batch mid-gate.

STATE_LOCK = "state.lock"
INFLIGHT_FILE = "INFLIGHT.json"


@contextlib.contextmanager
def state_lock(repo: Path):
    """Short critical sections over the pending set. Never held across a gate."""
    d = queue_dir(repo)
    d.mkdir(parents=True, exist_ok=True)
    with open(d / STATE_LOCK, "a+") as fh:
        fcntl.flock(fh, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(fh, fcntl.LOCK_UN)


def read_inflight(repo: Path) -> dict[str, dict]:
    """seq -> {pid, started_at} for every submission a LIVE conductor has taken. A record left
    by a dead conductor is ignored: its submissions are still pending and will be redone."""
    try:
        rec = json.loads((queue_dir(repo) / INFLIGHT_FILE).read_text())
    except (OSError, ValueError):
        return {}
    if not isinstance(rec, dict):
        return {}
    live = {}
    for seq, who in rec.items():
        try:
            pid, since = int(who["pid"]), float(who["started_at"])
        except (KeyError, TypeError, ValueError):
            continue
        if _pid_alive(pid, since=since):
            live[seq] = who
    return live


def _write_inflight(repo: Path, rec: dict[str, dict]) -> None:
    path = queue_dir(repo) / INFLIGHT_FILE
    if not rec:
        path.unlink(missing_ok=True)
        return
    tmp = path.with_suffix(".tmp")
    tmp.write_text(json.dumps(rec, indent=1))
    tmp.replace(path)


def take(repo: Path, subs: list[Submission]) -> list[Submission]:
    """Mark `subs` in flight, dropping any withdrawn since the caller read the queue.
    Returns what was actually taken."""
    with state_lock(repo):
        still = [s for s in subs if s.path.is_file()]
        rec = read_inflight(repo)
        now = time.time()
        for s in still:
            rec[s.seq] = {"pid": os.getpid(), "started_at": now, "lane": s.lane}
        _write_inflight(repo, rec)
    return still


def release_inflight(repo: Path, subs: list[Submission]) -> None:
    with state_lock(repo):
        rec = read_inflight(repo)
        for s in subs:
            rec.pop(s.seq, None)
        _write_inflight(repo, rec)


def withdraw(repo: Path, *, lane: str | None = None) -> tuple[list[Submission], list[Submission]]:
    """Pull this lane's QUEUED submissions out of line. Returns (withdrawn, in_flight): the
    in-flight ones are being gated right now and are left alone."""
    lane = lane or lane_id(repo)
    withdrawn: list[Submission] = []
    busy: list[Submission] = []
    with state_lock(repo):
        inflight = read_inflight(repo)
        for sub in pending(repo):
            if sub.lane != lane:
                continue
            if sub.seq in inflight:
                busy.append(sub)
                continue
            resolve(
                repo, sub, outcome="withdrawn", reason="the lane withdrew it (`land --withdraw`)"
            )
            withdrawn.append(sub)
    return withdrawn, busy


def evict(
    repo: Path, *, reason: str, keep_lane: str | None = None
) -> tuple[list[Submission], list[Submission]]:
    """Kick every OTHER lane's queued submissions back to its owner (verdict `evicted`, with
    `reason`). For clearing the line: the caller must already hold it (see land.py), so no
    conductor takes a batch mid-eviction. In-flight submissions are left alone, exactly as
    for `withdraw`. Returns (evicted, in_flight)."""
    keep_lane = keep_lane or lane_id(repo)
    evicted: list[Submission] = []
    busy: list[Submission] = []
    with state_lock(repo):
        inflight = read_inflight(repo)
        for sub in pending(repo):
            if sub.lane == keep_lane:
                continue
            if sub.seq in inflight:
                busy.append(sub)
                continue
            resolve(repo, sub, outcome="evicted", reason=reason)
            evicted.append(sub)
    return evicted, busy


# ── the integration worktree ────────────────────────────────────────────────


def integration_path(repo: Path, slot: int = 0) -> Path:
    return cachecfg.vibeos_root(repo) / integration_names(slot)[0]


def ensure_integration_worktree(repo: Path, slot: int = 0) -> Path:
    """The dedicated tree the train builds in. Created on first use.

    Dedicated, rather than gating in the conductor's own worktree, for two reasons: its
    ccache/sccache and build dirs stay HOT across every batch (so a batch gates as fast as
    the machine allows, and it is always the same tree, so no cache thrash from whichever
    lane happens to be conductor), and no lane's checkout is ever disturbed by a train that
    dies mid-batch. Build dirs are position-independent here (everything builds at the
    canonical /tmp/vibeos mount), so this tree is exactly as buildable as any other."""
    path = integration_path(repo, slot)
    if (path / ".git").exists():
        return path
    path.parent.mkdir(parents=True, exist_ok=True)
    git(repo, "fetch", "origin")
    # -B: create or reset the scratch branch. `integration` is never mainline, so resetting
    # it is not a rewind of anything shared (the reference-transaction guard covers master).
    git(repo, "worktree", "add", "-B", integration_names(slot)[1], str(path), "origin/master")
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


def build_batch(
    repo: Path, wt: Path, subs: list[Submission], log, *, onto: str = "origin/master"
) -> BatchResult:
    """Reset the integration tree onto `onto` (current origin/master, or -- in a pipelined
    drain -- the merged tip of the car ahead) and merge `subs` in order. Conflicting
    submissions are collected (to be rejected) and skipped, not resolved."""
    git(wt, "fetch", "origin")
    git(wt, "reset", "--hard", onto)
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


def dirty_paths(wt: Path) -> list[str]:
    """Tracked-file dirt in the integration tree, by the SAME criterion the gate's stamp step
    uses (`--untracked-files=no`). Reading it any other way would let the two disagree about
    whether a tree is stampable, which is the whole bug below."""
    out = git(wt, "status", "--porcelain", "--untracked-files=no", check=False)
    paths = []
    for raw in out.splitlines():
        # Split on the status field, never slice by column: `git()` strips its whole stdout,
        # so the FIRST line arrives without porcelain's leading space and a fixed `[3:]`
        # silently eats a character of the path (`tests/…` -> `ests/…`).
        parts = raw.strip().split(maxsplit=1)
        if parts:
            paths.append(parts[1] if len(parts) > 1 else parts[0])
    return paths


def _workspace_roots(wt: Path) -> list[Path]:
    """Directories holding a tracked `Cargo.lock` — derived, not hardcoded, so a second
    workspace cannot quietly escape the preflight."""
    locks = git(wt, "ls-files", "--", "*Cargo.lock", check=False)
    return [wt / Path(p).parent for p in locks.splitlines() if p.strip()]


def settle_cargo_locks(wt: Path, log) -> list[str]:
    """Materialize any lockfile the BUILD would have rewritten, in ~0.2s, before the gate.

    WHY (2026-08-04, three lanes wedged for an hour each): a lane parked a crate out of
    `workspace.members` but left its `[[package]]` entry in Cargo.lock. Cargo prunes that
    orphan on the next resolve, so the GATE rewrote the lock — and `_write_gate_stamp` refuses
    a dirty tree, because a stamp must hash content some commit actually hashes to. Green
    gate, no stamp, `pre-push` rejects, conductor dies, next conductor repeats the whole
    30-minute cold gate. Re-gating can never fix it, so it loops forever.

    Doing this BEFORE the gate is what makes it safe, and it is strictly better than settling
    afterwards: the tree that gets gated is then byte-identical to the tree that gets pushed,
    so there is no window in which the stamp certifies something other than what booted.

    `--locked` is NOT the probe — measured 2026-08-05: `cargo metadata --locked` exits 0 on
    exactly this input, because `--locked` objects to a lock that needs ADDING to, not one
    that needs PRUNING. The only honest probe is to let cargo resolve and see if the file
    moved. Returns the lock paths that changed (relative to `wt`)."""
    changed: list[str] = []
    for root in _workspace_roots(wt):
        cp = subprocess.run(
            ["cargo", "metadata", "--offline", "--format-version", "1"],
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=300,
        )
        if cp.returncode != 0:
            # Not fatal: the post-gate stamp backstop still catches the consequence. Say so
            # rather than letting a silently-skipped preflight read as "checked, and clean".
            log(f"queue: lock preflight could not resolve {root.name} — {cp.stderr.strip()[:200]}")
            continue
        rel = str((root / "Cargo.lock").relative_to(wt))
        if git(wt, "status", "--porcelain", "--untracked-files=no", "--", rel, check=False):
            changed.append(rel)
    return changed


_NO_STAGES: Mapping[str, str] = MappingProxyType({})


class GateVerdict(NamedTuple):
    """One gate run's judgement.

    `stages` (name -> "ok"/"fail"/"warn") is the part that matters beyond the headline: a
    single `classification` collapses a whole gate to one word, and that word is the loudest
    failure, not the only one. Keeping the per-stage map is what lets a red-base drain tell
    "framed" apart from "guilty of something else" — see `BaseCheck.own_faults`."""

    ok: bool
    classification: str
    artifact: str
    salient: str
    stages: Mapping[str, str] = _NO_STAGES


# Stages whose verdict a busy host cannot change. Builds, lints, the Python tooling gate and
# difftest are deterministic functions of the tree: they read source and either find a thing or
# do not. Smoke stages are deliberately NOT here — they are timing-sensitive, and mistaking
# contention for a regression is the whole reason the base check exists (field manual §9.84).
DETERMINISTIC_STAGES = ("build:", "lint:", "tooling", "difftest")


def _is_deterministic(stage: str) -> bool:
    return stage.startswith(DETERMINISTIC_STAGES)


# ── A RED STAGE THAT NOTHING IN THE BATCH COULD HAVE CAUSED ────────────────────────────
#
# 2026-09-13. `warnings-burndown` submitted 42 files of ntoskrnl/cdfs/fastfat/disk
# CONSTANTS and was rejected DIFFTEST_FAIL. The failing sub-stage was
# `msvcrt-difftest --test perf`, reporting towlower 10.8x slower than the C. The batch
# contained no msvcrt file at all, and the base check had already passed, so the lane was
# convicted of a regression it could not physically have introduced -- and every lane
# behind it re-paid a full three-arch gate to rediscover the same thing.
#
# This is the headline rule from the land-verification work, applied to the adjudicator:
# DOES THE THING YOU CONSULTED HAVE AUTHORITY OVER THE QUESTION YOU ARE ASKING. "The gate
# went red" has no authority over "is this lane at fault" when the red is in a harness the
# batch cannot reach.
#
# NARROW BY CONSTRUCTION, three ways, because this sits on the merge train for every lane:
#
#   1. It is NOT a widening of INCONCLUSIVE. That set's membership is untouched; this is a
#      separate path with its own classification, and its precondition is a file-set test
#      rather than a classification name. Conformance's standing constraint is that
#      INCONCLUSIVE must not grow loose, and a named test with an explicit precondition is
#      the opposite of that.
#   2. UNKNOWN MEANS GUILTY. Every derivation below returns None when it cannot answer, and
#      None declines the excuse. A harness we cannot map, a salient we cannot parse, a git
#      call that fails -- all fall through to the existing reject path. The only direction
#      this can err is toward the status quo.
#   3. The closure is OVER-approximated (every transitive path-dep, plus whole C directories)
#      so that "disjoint" is a strong claim. Under-approximating is the dangerous direction:
#      it would excuse a lane that really did break something.
#
# The loop-breaker is deliberate and explicit (see `_unrelated_excuses`): a submission may be
# excused this way twice, and the third time it is judged on the evidence. Without a bound,
# a genuinely broken batch whose breakage happens to live outside its own diff -- which is
# exactly what a bad *generated* artifact looks like -- would sit queued forever.

_MAX_UNRELATED_EXCUSES = 2

# `✗ msvcrt-difftest(exit 1)` inside the difftest stage's salient line.
_FAILED_SUBSTAGE = re.compile(r"[✗x]\s*([A-Za-z0-9_.:-]+)\(exit\b")
_CARGO_PATH_DEP = re.compile(r'path\s*=\s*"([^"]+)"')
# A repo-relative directory named as a string literal in a harness build.rs.
_BUILD_RS_DIR = re.compile(
    r'"((?:dll|drivers|ntoskrnl|hal|base|win32ss|boot|sdk)/[A-Za-z0-9_/.-]+)"'
)


def _failed_substages(salient: str) -> list[str] | None:
    """The sub-stages named as failing inside a stage's salient line.

    None when the line carries no parseable failure -- we then know only that the stage is
    red, which is not enough to say what it is red ABOUT."""
    names = _FAILED_SUBSTAGE.findall(salient or "")
    return names or None


def _harness_closure(repo: Path, harness: str) -> set[str] | None:
    """Repo-relative path prefixes a difftest harness's verdict can depend on.

    None when the harness is not a crate we can locate -- `drift:consts` and friends are
    sub-stages of the difftest driver itself, not crates, and we must not pretend to know
    their reach."""
    root = repo / "sdk/lib/rust" / harness
    if not (root / "Cargo.toml").is_file():
        return None
    seen: set[str] = set()
    stack = [root]
    while stack:
        crate = stack.pop()
        rel = crate.relative_to(repo).as_posix()
        if rel in seen:
            continue
        seen.add(rel)
        toml = crate / "Cargo.toml"
        if not toml.is_file():
            continue
        try:
            text = toml.read_text(errors="replace")
        except OSError:
            return None  # cannot enumerate the closure -> cannot claim disjoint
        for dep in _CARGO_PATH_DEP.findall(text):
            target = (crate / dep).resolve()
            try:
                target.relative_to(repo)
            except ValueError:
                continue  # outside the repo; nothing in a batch can touch it
            if (target / "Cargo.toml").is_file():
                stack.append(target)
        build_rs = crate / "build.rs"
        if build_rs.is_file():
            try:
                seen.update(_BUILD_RS_DIR.findall(build_rs.read_text(errors="replace")))
            except OSError:
                return None
    return seen


def _batch_paths(wt: Path, tip: str) -> set[str] | None:
    """Every path the batch changes relative to origin/master. None if git cannot say."""
    out = git(wt, "diff", "--name-only", "origin/master", tip, check=False)
    if not out:
        return None
    return {line.strip() for line in out.splitlines() if line.strip()}


def _touches(paths: set[str], closure: set[str]) -> bool:
    """Does any changed path fall inside any member of the closure?

    Prefix match on a path BOUNDARY -- `sdk/lib/rust/msvcrt-sys` must not swallow
    `sdk/lib/rust/msvcrt-sys-extra`, and a bare `str.startswith` would."""
    return any(p == c or p.startswith(c.rstrip("/") + "/") for p in paths for c in closure)


def unrelated_red(repo: Path, wt: Path, tip: str, verdict: GateVerdict) -> str:
    """'' unless the gate's red is provably outside everything this batch changed.

    Returns the human-readable reason when, and only when, every failing stage is one we
    can map to a source closure AND that closure is disjoint from the batch's diff."""
    failing = [n for n, s in verdict.stages.items() if s == "fail"]
    if not failing or failing != ["difftest"]:
        # Scoped to difftest on purpose. It is the only stage whose sub-results name a
        # crate, and therefore the only one whose reach is derivable today. A build or lint
        # red is global by nature and must keep rejecting.
        return ""
    subs = _failed_substages(verdict.salient)
    if not subs:
        return ""
    closure: set[str] = set()
    for harness in subs:
        part = _harness_closure(repo, harness)
        if part is None:
            return ""  # unknown reach -> guilty
        closure |= part
    paths = _batch_paths(wt, tip)
    if paths is None or _touches(paths, closure):
        return ""
    return (
        f"the only red is difftest/{', '.join(subs)}, whose source closure "
        f"({len(closure)} path(s)) contains NONE of the {len(paths)} file(s) this batch "
        f"changes. A harness the batch cannot reach has no authority over whether the "
        f"batch is at fault"
    )


def _excuse_path(repo: Path, sub: Submission) -> Path:
    return queue_dir(repo) / "unrelated" / f"{sub.seq}.json"


def _unrelated_excuses(repo: Path, sub: Submission, *, bump: bool = False) -> int:
    """How many times this submission has been excused for an unrelated red.

    Persisted per SUBMISSION (seq), not per batch: batches re-form every drain, so a
    per-batch counter would reset itself and never bound anything."""
    path = _excuse_path(repo, sub)
    try:
        count = int(json.loads(path.read_text()).get("count", 0))
    except (OSError, ValueError, AttributeError):
        count = 0
    if bump:
        count += 1
        try:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps({"count": count, "lane": sub.lane, "tip": sub.tip}))
        except OSError:
            # Cannot persist -> cannot bound the loop -> do not grant the excuse. Failing
            # closed here is what keeps the bound real rather than advisory.
            return _MAX_UNRELATED_EXCUSES + 1
    return count
def inert_only_batch(wt: Path) -> list[str]:
    """Paths this batch would add to master that something might actually RUN ([] = none).

    A batch whose entire delta is documentation cannot break a build, a boot or a scanner,
    so gating it spends 3 builds and 4 boots — roughly half an hour of the whole host — to
    re-prove that Markdown compiles. The pre-push hook has granted the matching exemption
    since it was written, and both now read the SAME predicate (`tools/docs_only.py`), so
    skipping the gate here cannot produce a tree the hook then refuses: the two agree by
    construction rather than by a comment asking them to.

    MEASURED ON THE MERGED TREE, NOT ON ANY LANE. `origin/master..HEAD` in the integration
    worktree is exactly what the push will add, after every lane in the batch is merged and
    after `settle_cargo_locks` has had its say. A per-lane test would be wrong in the unsafe
    direction twice over: three docs lanes plus one code lane is a code batch, and a lockfile
    the queue settles into the tip appears in no lane's own delta.

    DELIBERATELY NOT RESTRICTED TO A SINGLE SUBMISSION. "Solo docs-only" is the common case,
    but the property that makes the skip sound is the batch's CONTENT, not its size — four
    docs lanes are as inert as one, and one code lane poisons a batch of forty. Keying on
    content rather than count removes the special case instead of adding one.

    Returns the offending paths so the caller can SAY why it gated, and fails closed: any
    error, or an empty diff, yields a non-empty reason and the batch gates normally.
    """
    try:
        diff = git(wt, "diff", "--name-only", "origin/master", "HEAD")
    except Exception as exc:
        return [f"(could not diff against origin/master: {exc})"]
    paths = [line for line in diff.splitlines() if line.strip()]
    if not paths:
        return ["(empty delta against origin/master)"]
    # Load the predicate BY PATH, from THIS worktree. A `sys.path` insert plus
    # `from docs_only import ...` would be served from `sys.modules` if anything in the
    # process had already imported that name -- and land.py has, from the LANE's tools/.
    # The conductor would then judge the integration tree with a sibling worktree's copy
    # of the rule, which is the read half of the fleet-isolation law (AGENTS.md worktree
    # discipline 8) and is silent when the two copies differ.
    import importlib.util

    module_path = wt / "tools" / "docs_only.py"
    try:
        spec = importlib.util.spec_from_file_location("_queue_docs_only", module_path)
        if spec is None or spec.loader is None:
            raise ImportError(f"no loader for {module_path}")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        is_inert = module.is_inert
    except Exception as exc:
        return [f"(docs_only predicate unavailable: {exc})"]
    return [p for p in paths if not is_inert(p)]


def gate_batch(wt: Path, log, *, retries: int = 2) -> GateVerdict:
    """Run the FULL gate in the integration tree. This is the only place a batch is judged —
    no batch is ever presumed green.

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
    # COLD + collect-everything, by default, for the batch gate (2026-07-25).
    #
    # Warm builds hid THREE independent breaks on mainline simultaneously — two E0659
    # ambiguities and an undefined `___debugbreak` — because a crate that is not recompiled
    # cannot fail, and a `.sys` that is not relinked cannot fail to link. Every lane's gate was
    # green; mainline did not build from scratch on any arch.
    #
    # This is affordable precisely because the caches are content-addressed and live OUTSIDE the
    # target dirs `--clean` nukes: a cold replay measured 100%/100% ccache/sccache hits, so the
    # cost is cargo bookkeeping + linking, not recompilation. And a compile ERROR is never
    # cached — the crate really must compile — which is exactly why cold catches this class.
    # Amortized over a batch of N landings it is cheaper than N warm gates.
    #
    # `--keep-going` (ninja -k0) is the other half: the three breaks were found ONE PER BUILD
    # because ninja halts at the first failing edge. A verdict build wants the whole failure set
    # in one pass. VIBEOS_QUEUE_CLEAN=0 opts out if a batch ever needs to be fast over sure.
    if os.environ.get("VIBEOS_QUEUE_CLEAN", "1").lower() not in ("0", "false", "no"):
        cmd += ["--clean", "--keep-going"]
    cp = subprocess.run(
        cmd,
        cwd=str(wt),
        capture_output=True,
        text=True,
        env=dict(os.environ, VIBEOS_ARTIFACT_ROOT=str(root), VIBEOS_JOB_CLASS="integration"),
    )
    fresh = sorted(root.glob("*-gate-*"))
    artifact = str(fresh[-1]) if fresh else ""
    classification, salient = "", ""
    stages: dict[str, str] = {}
    if artifact:
        try:
            s = json.loads((Path(artifact) / "summary.json").read_text())
            classification = s.get("classification", "")
            for st in s.get("stages", []):
                name, status = st.get("name"), st.get("status")
                if name:
                    stages[name] = status or ""
                if status == "fail" and not salient:
                    salient = f"{name}: {st.get('detail', '')}"
        except (OSError, ValueError):
            pass
    ok = cp.returncode == 0
    if any(line.startswith("DISK_ADMISSION_FAIL:") for line in (cp.stdout or "").splitlines()):
        classification = "DISK_ADMISSION_FAIL"
        salient = cp.stdout.strip()
    why = "" if ok else f" ({classification or cp.returncode})"
    log(f"queue: gate {'GREEN' if ok else 'RED'}{why}")
    return GateVerdict(ok, classification, artifact, salient, MappingProxyType(stages))


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
# BUILD_INCOMPLETE joined the set on 2026-07-25, and it is worth being precise about why,
# because it looks conclusive. It means ninja exited 0 but a cohab PE is older than the
# staticlib it links (`artifact-check`). Observed 5 times that day on arm64 — and the STALE
# SET WAS DIFFERENT EVERY TIME (kernel32; version; shell32+usbport; advapi32+kernel32+
# ws2_32+mouhid). A real missing dependency edge is not random, so this is a race or an
# oracle false-positive, i.e. it says nothing about the submission. Until it is root-caused
# it must not reject a lane. Note these were previously reaching the queue as "" — the gate
# was dropping the child's classification entirely (see gate._child_classification), so this
# entry changes the REASON the train halts, not whether it halts.
INCONCLUSIVE = {
    "",
    "LAUNCH_FAIL",
    "HOST_STARVED",
    "NET_HANG",
    "BUILD_INCOMPLETE",
    "DISK_ADMISSION_FAIL",
    "BUILD_POLICY_REFUSED",
    "SCANNER_ERROR",
    # RUN_INTERRUPTED: the child was killed before it could finalize -- the host's
    # memory reaper, a broad-kill, the machine going away. It is the marker
    # `artifacts.write_interrupted_marker` seeds at run-dir creation, and it reaches
    # here only because the child never overwrote it. Purest environment: the lane's
    # code was never graded at all. Note this NARROWS nothing -- these runs previously
    # arrived as "" (no summary to read), which is already in this set. The entry
    # exists so that gaining a positive marker does not silently move them OUT of it
    # and start rejecting lanes for being reaped.
    "RUN_INTERRUPTED",
}


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
        self.stages: Mapping[str, str] = _NO_STAGES
        self.landed: list[str] = []  # lanes landed EARLIER in this same drain

    def is_green(self, repo: Path, wt: Path, log, *, retries: int) -> bool:
        if self.verdict is not None:
            return self.verdict
        log("queue: a lone submission failed — gating the BASE alone to see who is at fault")
        git(wt, "fetch", "origin")
        git(wt, "reset", "--hard", "origin/master")
        git(wt, "clean", "-fdq")
        v = gate_batch(wt, log, retries=retries)
        self.verdict, self.classification = v.ok, v.classification
        self.artifact, self.salient, self.stages = v.artifact, v.salient, v.stages
        log(
            "queue: BASE is GREEN — the submission really is the culprit"
            if v.ok
            else f"queue: BASE ITSELF IS RED ({v.classification}) — mainline is broken"
        )
        return v.ok

    def own_faults(self, stages: Mapping[str, str]) -> list[str]:
        """The DETERMINISTIC stages this submission fails that the base RAN AND PASSED.

        WHY (2026-07-31, a train wedged for 19 hours): "the base is red" was treated as a
        blanket alibi, but a red base only excuses the failures it can actually explain. The
        batch here failed `lint:{i386,amd64,arm64}` while the base-alone control failed only
        `smoke:net` — a contention flake (§9.84) that says exactly nothing about whether a
        lane added new lint findings. So three real `verify_signatures` regressions sat
        unattributed while every other lane queued behind them, and the evidence to attribute
        them was already on disk in both summaries.

        Two guards keep this from re-inventing the bug it replaces. A stage the base did NOT
        RUN proves nothing — an early-halting base gate must not convict a lane by omission —
        so only `"ok"` counts, never a missing key. And only deterministic stages are eligible:
        blaming a lane for a smoke stage that merely happened to pass on the base at a
        different moment is precisely the false conviction the base check exists to prevent."""
        return sorted(
            name
            for name, status in stages.items()
            if status == "fail" and _is_deterministic(name) and self.stages.get(name) == "ok"
        )


def land_subset(
    repo: Path,
    wt: Path,
    subs: list[Submission],
    log,
    *,
    retries: int = 2,
    base: BaseCheck | None = None,
    depth: int = 0,
) -> None:
    """Gate `subs` as one batch; on red, split and recurse.

    Landing a green half IMMEDIATELY (rather than after the whole bisect) is deliberate: the
    innocent half is already proven green against current master, and holding it hostage
    while its neighbour's culprit is isolated is exactly the "everyone waits on one lane"
    failure this queue exists to remove."""
    if not subs:
        return
    base = base or BaseCheck()  # one per drain; shared down the whole split tree
    prepared = prepare_car(repo, wt, subs, log)
    if prepared is None:
        return
    res, subs = prepared
    judge_car(
        repo, wt, subs, res, gate_car(wt, log, retries=retries), log, retries=retries, base=base
    )


def prepare_car(
    repo: Path,
    wt: Path,
    subs: list[Submission],
    log,
    *,
    onto: str = "origin/master",
    defer_conflicts: bool = False,
) -> tuple[BatchResult, list[Submission]] | None:
    """Merge `subs` onto `onto` in `wt` and make the tree gateable. Returns None when nothing
    is left to gate; raises Inconclusive on tree dirt.

    Conflicts are rejected here, unless `defer_conflicts`: a car merged onto the UNLANDED tip
    of the car ahead may conflict only with that car's code, and if that car then fails to
    land the conflict never existed. The caller rejects them once the car ahead is on master
    (see `_pipelined`), and otherwise leaves them queued."""
    res = build_batch(repo, wt, subs, log, onto=onto)
    for sub in [] if defer_conflicts else res.conflicted:
        resolve(
            repo,
            sub,
            outcome="rejected",
            reason="merge conflict with master — resolve it in your lane and resubmit",
            classification="CONFLICT",
        )
    subs = res.merged
    if not subs:
        return None
    # PREFLIGHT, ~0.2s. A tree the gate will dirty can never be stamped, and the failure would
    # otherwise surface half an hour later as an opaque pre-push rejection with nothing to
    # point at. Cheap enough to run every drain; the whole point is that it costs a heartbeat
    # against a 30-minute cold gate.
    stray = dirty_paths(wt)
    if stray:
        raise Inconclusive(
            "the integration tree was DIRTY before the gate — this is merge debris in the "
            f"conductor's worktree, not any lane's code: {', '.join(stray[:10])}. "
            f"{len(subs)} submission(s) stay QUEUED; nothing was rejected."
        )
    for lock in settle_cargo_locks(wt, log):
        git(wt, "add", "--", lock)
        git(wt, "commit", "--no-verify", "-m", f"queue: settle {lock} (the build rewrites it)")
        res.tip = git(wt, "rev-parse", "HEAD")
        log(f"queue: settled {lock} into the batch tip — the gate would have rewritten it")
    stray = dirty_paths(wt)
    if stray:
        raise Inconclusive(
            "the integration tree is dirty and it is not a lockfile the queue may settle: "
            f"{', '.join(stray[:10])}. The gate would run green and mint no push stamp, so "
            f"the push would be refused. {len(subs)} submission(s) stay QUEUED."
        )
    return res, subs


def gate_car(wt: Path, log, *, retries: int = 2) -> GateVerdict:
    """Gate the prepared tree in `wt` -- or, when every path it adds is inert, don't."""
    if inert_only_batch(wt):
        return gate_batch(wt, log, retries=retries)
    # Every path this batch adds is inert. See `inert_only_batch`.
    log(
        "queue: batch is DOCUMENTATION ONLY — skipping the gate (3 builds + 4 boots) and "
        "pushing; the pre-push hook grants the same exemption from the same predicate"
    )
    return GateVerdict(ok=True, classification="DOCS_ONLY", artifact="", salient="")


def judge_car(
    repo: Path,
    wt: Path,
    subs: list[Submission],
    res: BatchResult,
    verdict: GateVerdict,
    log,
    *,
    retries: int = 2,
    base: BaseCheck,
) -> None:
    """Act on a car's verdict: push a green one, requeue an inconclusive one, and split a
    red one until the culprit is alone."""
    ok, classification = verdict.ok, verdict.classification
    artifact, salient = verdict.artifact, verdict.salient
    if ok:
        # We hold the land lock for the whole drain, and every landing path takes it, so
        # master cannot have moved under us — assert it rather than assume it.
        if not git_ok(wt, "merge-base", "--is-ancestor", "origin/master", "HEAD"):
            raise QueueError(
                "master moved during a batch gate while the land lock was held — something "
                "pushed outside the queue; re-run the drain"
            )
        # BACKSTOP. The stamp is the push credential; a green gate that minted none means the
        # hook is about to refuse this update, and the preflight above evidently missed the
        # cause. Say that HERE, where the evidence is, instead of letting it surface as a bare
        # git rejection the conductor dies on. Only on evidence: an unparsed summary (empty
        # `stages`) proves nothing, so it falls through to the push as before.
        stamp = verdict.stages.get("stamp", "")
        # A DOCS_ONLY batch never ran a gate, so of course it minted no stamp — the hook
        # will accept it on the docs exemption instead. Exempt it explicitly rather than
        # leaning on `stages` happening to be empty.
        if classification != "DOCS_ONLY" and verdict.stages and stamp != "ok":
            raise Inconclusive(
                f"the gate was GREEN but minted no push stamp (stamp: {stamp or 'absent'}) — "
                "the pre-push hook refuses a master update whose tree has no fresh stamp, so "
                "pushing would fail with nothing to point at. Dirt: "
                f"{', '.join(dirty_paths(wt)) or '(none — see the gate artifact)'}. "
                f"{len(subs)} submission(s) stay QUEUED. Artifact: {artifact or '(none)'}"
            )
        try:
            git(wt, "push", "origin", f"{res.tip}:refs/heads/master")
        except QueueError as e:
            # Keep the train's failure mode uniform: a push that cannot happen leaves every
            # submission QUEUED and halts, rather than escaping as an unhandled error that
            # reads like a crash and resolves nobody.
            raise Inconclusive(
                f"the batch gated GREEN but the push was refused: {e}. "
                f"{len(subs)} submission(s) stay QUEUED. Artifact: {artifact or '(none)'}"
            ) from e
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
    # WHERE the excuse may be spent. Granting it for an intermediate sub-batch would abort
    # the split -- the two halves run in sequence with no handler, so raising inside the first
    # means the second is never gated and a guilty sibling stays queued behind an innocent
    # one. So it applies only where a CONVICTION would otherwise happen: the whole batch
    # (depth 0, nothing to isolate -- no member can be responsible, so splitting cannot
    # attribute it either) or a lone submission (the split is already finished). An
    # intermediate sub-batch splits further instead, which costs a gate but keeps the train
    # moving and still protects the lone lane at the bottom.
    decisive = depth == 0 or len(subs) == 1
    unrelated = unrelated_red(repo, wt, res.tip, verdict) if decisive else ""
    if unrelated:
        # Spend an excuse per submission, and only grant it while EVERY submission in the
        # batch still has one. A batch is excused as a unit, so the bound has to hold for the
        # unit: if one member has exhausted its excuses, the batch is judged on the evidence
        # and the split will isolate it as usual.
        spent = [s for s in subs if _unrelated_excuses(repo, s) >= _MAX_UNRELATED_EXCUSES]
        if spent:
            log(
                f"queue: unrelated-red excuse EXHAUSTED for {', '.join(s.lane for s in spent)}"
                f" — judging on the evidence"
            )
        else:
            for s in subs:
                _unrelated_excuses(repo, s, bump=True)
            raise Inconclusive(
                f"the gate is red but not about this batch: {unrelated}. "
                f"{len(subs)} submission(s) stay QUEUED; nothing was rejected. This excuse is "
                f"bounded — after {_MAX_UNRELATED_EXCUSES} the batch is judged on the evidence "
                f"regardless. Artifact: {artifact or '(none)'}"
            )
    if len(subs) == 1:
        sub = subs[0]
        # GUILTY OR FRAMED? A lone red submission is only the culprit if the BASE is green.
        # Skipping this check is how the first real batch rejected a docs-only change for a
        # build failure that was already on mainline.
        if not base.is_green(repo, wt, log, retries=retries):
            # A red base is an alibi only for the failures it explains. If this submission
            # also fails a DETERMINISTIC stage that the base itself passed, that part is its
            # own, and saying so is what stops a doomed lane from blocking the queue forever
            # while its real defect goes unreported.
            own = base.own_faults(verdict.stages)
            if own:
                log(
                    f"queue: REJECTED {sub.lane} {sub.tip[:12]} — "
                    f"{', '.join(own)} (the base PASSES these)"
                )
                resolve(
                    repo,
                    sub,
                    outcome="rejected",
                    reason=(
                        f"{', '.join(own)} fail here and PASS on origin/master alone, so they "
                        f"are this lane's: {salient or 'see the artifact'}. (The base is "
                        f"separately red — {base.classification or 'red'} — which is not "
                        f"yours and is why nothing landed this drain.) "
                        f"Artifact: {artifact or '(none)'}"
                    ),
                    classification=classification or "GATE_RED",
                    artifact=artifact,
                )
            raise BaseRed(
                f"origin/master itself fails the gate ({base.classification or 'red'}) — "
                f"{base.salient or 'see the artifact'}. "
                + (
                    f"{sub.lane} was rejected for {', '.join(own)}, which the base passes; "
                    "no other submission was judged. "
                    if own
                    else "No submission was rejected (they are not at fault) "
                )
                + f"The train is halted: nothing can land on a red base. "
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
    land_subset(repo, wt, subs[:mid], log, retries=retries, base=base, depth=depth + 1)
    land_subset(repo, wt, subs[mid:], log, retries=retries, base=base, depth=depth + 1)


def _landed_exactly(repo: Path, tip: str) -> bool:
    """Is origin/master now exactly `tip` -- i.e. did the car ahead land whole, as merged?"""
    git(repo, "fetch", "origin")
    return git(repo, "rev-parse", "origin/master", check=False) == tip


def _pipelined(
    repo: Path,
    lead: list[Submission],
    follower: list[Submission],
    log,
    *,
    retries: int,
) -> None:
    """Gate `follower` on top of `lead` while `lead` gates. CALLER MUST HOLD the land lock,
    and releases both cars' in-flight marks.

    The follower's tree is master + lead + follower, so its verdict is about exactly what
    master will be once the lead lands -- and only then. So the follower is judged only if
    the lead landed WHOLE (origin/master == the lead's merged tip). Any other outcome -- the
    lead red and split, requeued as inconclusive, or halting the train -- makes the
    follower's gate a verdict about a tree that will never exist: it is discarded, and the
    follower stays QUEUED for the next car, unjudged and unblamed.

    Both gates are always JOINED before this returns or raises: a gate outliving the land
    lock would be running in a worktree the next conductor is about to reset."""
    wt_lead = ensure_integration_worktree(repo, 0)
    prepared = prepare_car(repo, wt_lead, lead, log)
    if prepared is None:
        return
    res_a, lead_subs = prepared
    with ThreadPoolExecutor(max_workers=2) as pool:
        gate_a = pool.submit(gate_car, wt_lead, log, retries=retries)
        gate_b = None
        res_b: BatchResult | None = None
        follower_subs: list[Submission] = []
        try:
            wt_follow = ensure_integration_worktree(repo, 1)
            prepared_b = prepare_car(
                repo, wt_follow, follower, log, onto=res_a.tip, defer_conflicts=True
            )
        except QueueError as e:
            # A follower that cannot even be prepared costs nothing but its speed-up: the
            # lead is unaffected, and the follower gates normally as a later car.
            log(f"queue: not pipelining the next car ({e}) — it stays queued")
            prepared_b = None
        if prepared_b is not None:
            res_b, follower_subs = prepared_b
            if follower_subs:
                log(
                    f"queue: PIPELINED — gating the next {len(follower_subs)} on top of this "
                    f"car ({', '.join(s.lane for s in follower_subs)})"
                )
                gate_b = pool.submit(gate_car, wt_follow, log, retries=retries)
        try:
            judge_car(
                repo,
                wt_lead,
                lead_subs,
                res_a,
                gate_a.result(),
                log,
                retries=retries,
                base=BaseCheck(),
            )
        finally:
            verdict_b = gate_b.result() if gate_b is not None else None
        if res_b is None:
            return
        if not _landed_exactly(repo, res_a.tip):
            log(
                "queue: the car ahead did not land whole — discarding the pipelined verdict; "
                f"{len(follower)} submission(s) stay QUEUED to be gated on the new master"
            )
            return
        for sub in res_b.conflicted:
            resolve(
                repo,
                sub,
                outcome="rejected",
                reason="merge conflict with master — resolve it in your lane and resubmit",
                classification="CONFLICT",
            )
        if verdict_b is not None and not verdict_b.ok and verdict_b.classification in INCONCLUSIVE:
            # Two gates at once is more host contention, and a starved SECOND gate says
            # nothing about its lanes. It must not halt a train whose lead just landed:
            # the follower simply goes again as the next lead, gated alone.
            why = verdict_b.classification or "no classification"
            log(
                f"queue: pipelined gate was inconclusive ({why}) — "
                f"{len(follower_subs)} submission(s) stay QUEUED"
            )
            return
        if follower_subs and verdict_b is not None:
            judge_car(
                repo,
                ensure_integration_worktree(repo, 1),
                follower_subs,
                res_b,
                verdict_b,
                log,
                retries=retries,
                base=BaseCheck(),
            )


def already_landed(repo: Path, sub: Submission) -> bool:
    """A tip already contained in origin/master needs no gate — it landed in someone else's
    batch (or the lane pushed it directly). Resolve it as landed rather than re-gating."""
    return git_ok(repo, "merge-base", "--is-ancestor", sub.tip, "origin/master")


def drain(
    repo: Path, log, *, retries: int = 2, cap: int = BATCH_CAP, depth: int = PIPELINE_DEPTH
) -> int:
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
        # Checked per BATCH, not per drain: a hold placed while this conductor is gating
        # takes effect at the next batch boundary (`land --hold --preempt` for sooner).
        held = effective_hold(repo)
        if held:
            log(f"queue: {describe_hold(held)} -- {len(subs)} submission(s) stay QUEUED")
            raise LineHeld(describe_hold(held))
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
        batch = take(repo, fresh[:cap])
        if not batch:
            continue  # everything we looked at was withdrawn in the meantime
        log(
            f"queue: batch of {len(batch)} — "
            + ", ".join(f"{s.lane}@{s.tip[:8]}" for s in batch)
            + (f" ({len(fresh) - len(batch)} more queued)" if len(fresh) > len(batch) else "")
        )
        taken = {s.seq for s in batch}
        follower = (
            take(repo, [s for s in fresh[cap:] if s.seq not in taken][:cap]) if depth > 1 else []
        )
        try:
            if follower:
                _pipelined(repo, batch, follower, log, retries=retries)
            else:
                land_subset(repo, wt, batch, log, retries=retries)
        except NoVerdict as e:
            # Everything still queued STAYS queued — it was never judged. Halting is the only
            # honest move: on a red base (or a starved host) every batch would fail for a
            # reason that has nothing to do with the lanes in it.
            log(f"queue: TRAIN HALTED — {e}")
            if isinstance(e, BaseRed):
                # Stop the line until master moves. Without this, every later submission
                # elects a conductor that re-gates the same red base (the 2026-09-25 night).
                red = git(repo, "rev-parse", "origin/master", check=False)
                if red and read_hold(repo) is None:
                    set_hold(repo, lane="(train)", reason=str(e)[:600], auto=True, base=red)
                    log(
                        f"queue: line AUTO-HELD on red master {red[:12]} -- clears itself when "
                        "master moves; land the fix with `vibeos-cli land --direct`"
                    )
            raise
        finally:
            release_inflight(repo, batch + follower)
        batches += 1


def try_become_conductor(repo: Path, log, *, retries: int = 2, timeout: float = 0.0) -> bool:
    """Take the land lock and drain. Returns False if the lock could not be had in `timeout`.

    The election is the lock: no daemon, no leader file, no heartbeat.

    `timeout` is the whole subtlety. It was 0.0 unconditionally, so an agent that lost the
    election left immediately — correct when the agent's own turn was on the line, but it left
    an ORPHAN WINDOW: a submission whose only would-be conductor lost the election has nobody
    to drain it, and it sits until the next submission elects someone. Observed within minutes
    of shipping (a pre-queue lane held the lock via the direct path, and the submission waited).

    Now that conducting is DETACHED, the spawned child has no reason to give up: nothing is
    blocked by it waiting, so it queues on the lock's FIFO and drains when its turn comes. A
    conductor that wakes to an empty queue exits in ~1s, so several waiters are cheap rather
    than wasteful. Callers that genuinely cannot block (the inline-from-submit convenience path)
    keep passing 0.0."""
    try:
        with hostlock.land_lock(f"conductor:{lane_id(repo)}", timeout=timeout, repo=str(repo)):
            drain(repo, log, retries=retries)
        return True
    except hostlock.LandLockBusy:
        return False
