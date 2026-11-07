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

from pathlib import Path

from tools import build_infra_compat as native

# The probe, the ENOSPC text and the admission lines are native
# (tools/vibeos-build/src/diskspace.rs); this keeps the gate-stage glue.

# Headroom below which we refuse to start rather than fail somewhere unhelpful.
MIN_FREE_BYTES = 5 * 1024**3
# "Can the harness still record a verdict at all" (mirrors diskspace.rs).
CRITICAL_FREE_BYTES = 256 * 1024**2


def human(n: int) -> str:
    """Bytes -> short human string. Exact enough to act on, short enough to log."""
    return native.call("disk_human", {"n": n})


def free_bytes(path: Path) -> int | None:
    """Free bytes on the filesystem holding `path` (nearest existing ancestor), or None."""
    return native.call("disk_free_bytes", {"path": path})


def probe(paths, threshold: int | None = None) -> dict:
    """Every distinct filesystem behind `paths`: `ok`, `critical`, and `lines` already carrying
    the numbers -- print them verbatim, because the number is the entire point. `threshold`
    resolves at CALL time so the failure path stays testable."""
    return native.call(
        "disk_probe",
        {
            "paths": [p for p in paths if p is not None],
            "threshold": MIN_FREE_BYTES if threshold is None else threshold,
        },
    )


def gate_paths(repo: Path, arches) -> list[Path]:
    """The filesystems a gate actually writes to: the tree, the build dirs, /tmp."""
    return [Path(p) for p in native.call("disk_gate_paths", {"repo": repo, "arches": list(arches)})]


def looks_like_enospc(blob: str) -> bool:
    return native.call("cli_build_check_enospc", {"text": blob})


# This worktree, never a sibling: <repo>/tests/qemu/cli/hostdisk.py -> parents[3].
# Deriving it from __file__ rather than a caller-supplied path is what keeps the
# fleet-isolation law (AGENTS.md worktree discipline §7-8) true by construction.
REPO_ROOT = Path(__file__).resolve().parents[3]


def enospc_detail(repo: Path | None = None) -> list[str]:
    """Salient lines for an ENOSPC classification -- always carrying the number."""
    return native.call("disk_enospc_detail", {"repo": REPO_ROOT if repo is None else repo})


def admission_lines(repo: Path, arches=()) -> list[str]:
    """The admission headroom that actually gates a run, which `df` cannot show -- printed on
    the GREEN path too. A failure to read it is reported, never swallowed, and never blocks
    the free-space verdict."""
    from . import diskspace

    request: dict = {"arch_count": len(list(arches))}
    try:
        request["status"] = diskspace.status(repo)
    except Exception as exc:  # reported in the lines, never silently dropped
        request["status_error"] = f"{type(exc).__name__}: {exc}"
    return native.call("disk_admission_lines", request)


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
