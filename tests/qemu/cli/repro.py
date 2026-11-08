"""`vibeos-cli repro` — reproducibility / determinism measurement harness (rung 0).

The determinism lane's gauge (docs/engineering/deterministic-builds-plan.md). Answers two
questions without any external dependency:

  1. **Are two builds of the same commit byte-identical?** (`compare A B`) — and if not,
     *which PE sections differ* and *does the PE TimeDateStamp diverge*, so a leak is
     localized instead of just "the bytes differ".
  2. **What determinism-relevant metadata does an artifact carry today?** (`report F...`) —
     the COFF TimeDateStamp (the measured live leak on all 3 arches) + per-section digests.

The PE parser, the comparison and the report text are native
(tools/vibeos-build/src/repro.rs, unit-tested there). This facade keeps `verify`, whose two
cold builds stream to the terminal, and the optional `diffoscope` deep localizer
(`--diffoscope`), whose absence degrades gracefully.

This is the foundation for the ladder's rung 5 (a build-twice byte-identical gauge wired into
the harness): `diff-dirs` compares the staged artifact set across two build trees, which is
exactly the build-twice-in-two-paths verification methodology the plan calls for.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from tools import build_infra_compat as native


def _call(op: str, **kwargs):
    return native.call("repro_" + op, kwargs)


def _print(result: dict) -> None:
    for line in result["lines"]:
        print(line)


def _run_diffoscope(a: Path, b: Path) -> None:
    tool = shutil.which("diffoscope")
    if not tool:
        print("  (diffoscope not installed — `pip install diffoscope` for byte-level localization)")
        return
    print(f"\n  diffoscope {a} {b}:")
    try:
        cp = subprocess.run(
            [tool, "--max-report-size", "40000", str(a), str(b)],
            capture_output=True,
            text=True,
            timeout=300,
        )
        print("\n".join(("    " + ln) for ln in cp.stdout.splitlines()[:200]))
    except (OSError, subprocess.SubprocessError) as e:
        print(f"    diffoscope failed: {e}")


def _cmd_compare(args) -> int:
    a, b = Path(args.file_a), Path(args.file_b)
    rep = _call("compare", a=a, b=b, name=a.name)
    _print(rep)
    if rep["ok"] and not rep["identical"] and getattr(args, "diffoscope", False):
        _run_diffoscope(a, b)
    return 0 if rep["identical"] else 1


def _cmd_report(args) -> int:
    rep = _call("report", files=[Path(f) for f in args.files])
    _print(rep)
    return rep["rc"]


def _cmd_diff_dirs(args) -> int:
    rep = _call(
        "diff_dirs", dir_a=Path(args.dir_a), dir_b=Path(args.dir_b), artifacts=args.artifacts
    )
    _print(rep)
    if getattr(args, "diffoscope", False):
        for a, b in rep["differing"]:
            _run_diffoscope(Path(a), Path(b))
    return rep["rc"]


def _cli_build_clean(repo: Path, base_arch: str, label: str) -> bool:
    """Run a cold `vibeos-cli build --arch <base> --clean` as a subprocess (reuses the exact
    build path: target nuke, cargo relink, lints, stamp). Returns True on success."""
    print(f"\n[repro verify] {label}: build --arch {base_arch} --clean ...")
    cp = subprocess.run(
        [sys.executable, "-m", "tests.qemu.cli", "build", "--arch", base_arch, "--clean"],
        cwd=str(repo),
    )
    return cp.returncode == 0


def _cmd_verify(args, repo: Path) -> int:
    """Rung 5: build the arch cold TWICE and assert the artifacts are byte-identical.

    The reproducibility gauge — if two cold builds of the same commit at the same path differ,
    something nondeterministic leaked (the rungs 1/2 fixes are exactly what make this pass).
    Between the builds we DELETE the artifacts: `vibeos-cli build --clean` nukes the rust
    target dirs + `lib*_rust.a` but NOT the copied-out cargo module exes (deterministic-builds-
    plan.md §stale-cargo / task #41), so without removing them ninja would see the stale copy
    as current and skip the relink — a false "identical". Removing them forces a genuine relink.

    NOTE: cross-WORKTREE byte-identity (the dedup payoff) comes from the canonical build
    path (cli.canon — every worktree builds at /tmp/vibeos), making it equivalent to this
    same-path build-twice, which proves the timestamp/date/path determinism holds."""
    from . import archcfg

    base = archcfg.base_arch(args.arch)
    build_dir = repo / f"build-{base}"
    artifacts = args.artifacts or list(_call("default_artifacts"))

    if not _cli_build_clean(repo, base, "build #1 (cold)"):
        print("[repro verify] build #1 FAILED")
        return 1

    snap = Path(tempfile.mkdtemp(prefix="repro-verify-"))
    snapped: list[tuple[str, Path]] = []
    for rel in artifacts:
        src = build_dir / rel
        if src.is_file():
            dst = snap / rel.replace("/", "__")
            shutil.copy2(src, dst)
            snapped.append((rel, dst))
    if not snapped:
        print(f"[repro verify] no artifacts found under {build_dir} after build #1")
        shutil.rmtree(snap, ignore_errors=True)
        return 1

    # Force a genuine relink (defeat the §41 stale-output trap) by removing the copies.
    for rel, _ in snapped:
        (build_dir / rel).unlink(missing_ok=True)

    if not _cli_build_clean(repo, base, "build #2 (cold, forced relink)"):
        print("[repro verify] build #2 FAILED")
        shutil.rmtree(snap, ignore_errors=True)
        return 1

    print(f"\n[repro verify] byte-identity {base}: build #1 vs build #2")
    any_diff = False
    for rel, snap_path in snapped:
        b = build_dir / rel
        if not b.is_file():
            print(f"  ! {rel}: missing after build #2")
            any_diff = True
            continue
        rep = _call("compare", a=snap_path, b=b, name=rel)
        _print(rep)
        if not rep["identical"]:
            any_diff = True
            if getattr(args, "diffoscope", False):
                _run_diffoscope(snap_path, b)
    shutil.rmtree(snap, ignore_errors=True)
    print()
    print(
        "  → NOT reproducible (cold build-twice differs)"
        if any_diff
        else "  → cold build-twice BYTE-IDENTICAL ✓"
    )
    return 1 if any_diff else 0


def repro(args, repo: Path) -> int:
    """Dispatch the `repro` subverb."""
    cmd = getattr(args, "repro_cmd", None)
    if cmd == "compare":
        return _cmd_compare(args)
    if cmd == "report":
        return _cmd_report(args)
    if cmd == "diff-dirs":
        return _cmd_diff_dirs(args)
    if cmd == "verify":
        return _cmd_verify(args, repo)
    print("repro: specify a subcommand (compare | report | diff-dirs | verify)")
    return 2
