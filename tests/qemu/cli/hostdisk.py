"""Host free-space preflight + ENOSPC classification.

Why this exists: on 2026-09-06 the root filesystem hit 100% and cost four lanes a
day, because at 100% the failure does not present as a disk problem. It surfaced
as a gate RED with **no failing test**, an ENOSPC raised deep in `artifacts.py`
writing `summary.json`, "system is running low on memory" in two harnesses, and a
conductor dying mid-drain. Every symptom named a plausible wrong cause.

Worse, the tooling you would reach for to diagnose it is the tooling that stops
working: at 0 bytes free the harness cannot run ANY command, not even `true`,
because it creates each command's stdout file first.

So the rule "check `df -h /` before diagnosing anything else" is real, but a rule
with no command behind it is a rule we break — that is the AGENTS.md argument
about itself. This module is the command. The whole value is in printing the
NUMBER: "DISK_FULL: 192K free on / (need 5.0G)" ends the investigation in one
line, where four agents reading a RED with no failing test did not.

Field manual: "A gate can be RED for reasons that are not in the tree at all".
"""

from __future__ import annotations

import os
import shutil
from pathlib import Path

# Headroom below which we refuse to start rather than fail somewhere unhelpful.
# A full gate writes three ISOs plus per-arch artifact dirs; 5G is well under
# what a green run consumes and well over the "cannot write summary.json" cliff.
MIN_FREE_BYTES = 5 * 1024**3

# Kept separate and much smaller: this one is only asking "can the harness still
# record a verdict at all", which is the failure that makes everything else
# illegible.
CRITICAL_FREE_BYTES = 256 * 1024**2


def human(n: int) -> str:
    """Bytes -> short human string. Exact enough to act on, short enough to log."""
    # Separate float accumulator: dividing the int parameter in place retypes it
    # (pyright reportAssignmentType) and loses the caller's contract.
    v = float(n)
    for unit in ("B", "K", "M", "G", "T"):
        if abs(v) < 1024 or unit == "T":
            return f"{v:.0f}{unit}" if unit == "B" else f"{v:.1f}{unit}"
        v /= 1024.0
    return f"{v:.1f}T"


def free_bytes(path: Path) -> int | None:
    """Free bytes on the filesystem holding `path`, or None if unknowable.

    Walks up to the nearest existing ancestor: a build dir that does not exist
    yet still sits on a filesystem whose free space we can and should report.
    """
    p = Path(path).resolve()
    while not p.exists() and p != p.parent:
        p = p.parent
    try:
        return shutil.disk_usage(p).free
    except OSError:
        return None


def probe(paths, threshold: int | None = None) -> dict:
    """Check every distinct filesystem behind `paths`.

    Returns a dict with `ok`, `critical`, and a `lines` list already carrying the
    numbers — callers should print these verbatim rather than re-summarising,
    because the number is the entire point.

    `threshold` resolves at CALL time. It is deliberately not a default argument:
    `threshold: int = MIN_FREE_BYTES` binds the constant once at import, so the
    module value could never be overridden and the failure path would go untested.
    """
    threshold = MIN_FREE_BYTES if threshold is None else threshold
    seen: dict[int, tuple[Path, int]] = {}
    for raw in paths:
        if raw is None:
            continue
        p = Path(raw)
        n = free_bytes(p)
        if n is None:
            continue
        try:
            key = os.stat(_nearest_existing(p)).st_dev
        except OSError:
            continue
        # One report per filesystem, attributed to the first path that named it.
        seen.setdefault(key, (p, n))

    lines: list[str] = []
    ok = True
    critical = False
    for p, n in sorted(seen.values(), key=lambda t: t[1]):
        if n < threshold:
            ok = False
            lines.append(f"DISK_FULL: {human(n)} free on {p} (need {human(threshold)})")
            if n < CRITICAL_FREE_BYTES:
                critical = True
                lines.append(
                    f"  CRITICAL: below {human(CRITICAL_FREE_BYTES)} — the harness may be "
                    f"unable to write summary.json or even run `true`, so any verdict "
                    f"from this host is unreliable rather than merely red."
                )
    return {
        "ok": ok,
        "critical": critical,
        "lines": lines,
        "free": {str(p): n for p, n in seen.values()},
    }


def _nearest_existing(p: Path) -> Path:
    p = p.resolve()
    while not p.exists() and p != p.parent:
        p = p.parent
    return p


def gate_paths(repo: Path, arches) -> list[Path]:
    """The filesystems a gate actually writes to: the tree, the build dirs, /tmp.

    /tmp matters independently — artifact dirs and every command's stdout file
    live under it, and it is where the 2026-09-06 ENOSPC was actually raised.
    """
    out = [repo, Path("/tmp")]
    out += [repo / f"build-{a}" for a in arches]
    return out


# ENOSPC does not always arrive as a clean OSError: it comes back as compiler,
# linker or python text on stderr. Matching the message is what lets a build
# failure be classified rather than filed under a wrong cause.
_ENOSPC_MARKERS = (
    "no space left on device",
    "errno 28",
    "enospc",
    "disk quota exceeded",
    "write error: no space",
)


def looks_like_enospc(blob: str) -> bool:
    low = blob.lower()
    return any(m in low for m in _ENOSPC_MARKERS)


# This worktree, never a sibling: <repo>/tests/qemu/cli/hostdisk.py -> parents[3].
# Deriving it from __file__ rather than a caller-supplied path is what keeps the
# fleet-isolation law (AGENTS.md worktree discipline §7-8) true by construction.
REPO_ROOT = Path(__file__).resolve().parents[3]


def enospc_detail(repo: Path | None = None) -> list[str]:
    repo = REPO_ROOT if repo is None else repo
    """Salient lines for an ENOSPC classification — always carrying the number."""
    rep = probe(gate_paths(repo, ()), threshold=MIN_FREE_BYTES)
    if rep["lines"]:
        return rep["lines"][:3]
    # Space was reclaimed between the failure and this probe: say so, rather than
    # printing a reassuring "plenty free" that contradicts the error just seen.
    return [
        "ENOSPC reported by the toolchain but the filesystem now has headroom — "
        "space was freed after the failure, or a different mount filled. "
        f"now: {', '.join(f'{k}={human(v)}' for k, v in rep['free'].items())}"
    ]


def admission_lines(repo: Path, arches=()) -> list[str]:
    """The quantity that actually gates a run, which `df` cannot show.

    FULLNESS and ADMISSION HEADROOM are different numbers, and only one of them is
    visible to the check every agent thinks to run. On 2026-09-06 six lanes stalled
    with the host at 88% and 117G free — `df` fine, this module's own 5G threshold
    fine — because a 3-arch gate reserves 3 x 25G of build headroom above a 20G
    floor, and the unreserved remainder had gone under. Every conduct failed
    DISK_ADMISSION_FAIL on its third arch, so the queue could not self-heal, and
    nothing about the failure pointed at reservations. **"df looks fine" is a
    documented false negative for that class**, which is why this prints on the
    GREEN path too: the whole point is the run that looks healthy.

    Reads `diskspace`, which owns the accounting; this only surfaces it. A failure
    to read is REPORTED rather than swallowed — a missing number must not look like
    a healthy one — but never blocks the free-space verdict, which stands alone.
    """
    try:
        from . import diskspace

        st = diskspace.status(repo)
    except Exception as exc:  # reported below, never silently dropped
        return [f"admission headroom unavailable ({type(exc).__name__}: {exc})"]
    policy = st.get("policy") or {}
    requests = policy.get("requests_gib") or {}
    floor = policy.get("min_free_gib", 0)
    n = len(list(arches)) or 1
    wants = requests.get("build", 0) * n + requests.get("check-tooling", 0)
    lines = []
    for fs in st.get("filesystems") or []:
        head = fs.get("unreserved_above_floor_bytes", 0)
        mark = "" if head >= wants * 2**30 else "  << SHORT"
        lines.append(
            f"admission: {human(head)} unreserved above the {floor}G floor on "
            f"{fs.get('path')} ({n}-arch run requests ~{wants}G){mark}"
        )
    return lines


def preflight(run, repo: Path, arches) -> bool:
    """Gate preflight. Returns False (and records the stage) when space is short.

    Deliberately a hard stop rather than a warning: a gate run on a full host
    produces a verdict nobody can trust — including, at the extreme, no verdict
    at all, because writing summary.json is itself a write. Refusing up front
    with the number costs seconds; the alternative cost four lanes a day.
    """
    from .artifacts import Stage

    rep = probe(gate_paths(repo, arches), threshold=MIN_FREE_BYTES)
    admission = admission_lines(repo, arches)
    for ln in admission:
        run.log(ln)
    if rep["ok"]:
        smallest = min(rep["free"].values(), default=0)
        detail = f"{human(smallest)} free"
        # Carry the SHORT marker into the stage line: a run that passes free-space
        # but is out of reservation headroom is exactly the case that read healthy.
        if any("<< SHORT" in ln for ln in admission):
            detail += "; admission headroom SHORT"
        run.add(Stage("host-disk", status="ok", detail=detail))
        return True
    for ln in rep["lines"]:
        run.log(ln)
    run.classification = "DISK_FULL"
    run.add(Stage("host-disk", status="fail", detail=rep["lines"][0]))
    run.log(
        "fix: reclaim space, then re-run. Note sccache evicts only entries in ITS OWN "
        "index, so a directory grown by an earlier server generation is invisible to it "
        "and is never reclaimed at any cap -- check the dir size against the cap the "
        "running server reports."
    )
    return False
