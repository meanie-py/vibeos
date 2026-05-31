"""Source + binary lint runners with baseline-diff (plan §20.3).

Gate = zero NEW findings vs a checked-in baseline (tools/lint-baseline/<tool>.txt),
never absolute zero — the tree carries known pre-existing findings. Advisory
(warn) in `build`'s auto-check; blocking-on-NEW only under `check --strict`.
"""
from __future__ import annotations

import shutil
import sys
from pathlib import Path

from .artifacts import Run, Stage

BASELINE_DIR = "tools/lint-baseline"


def _shellcheck_bin() -> str | None:
    b = shutil.which("shellcheck")
    if b:
        return b
    # venv bin dir (shellcheck-py installs here; not necessarily on PATH when
    # invoked via the wrapper's explicit interpreter)
    cand = Path(sys.executable).parent / "shellcheck"
    if cand.exists():
        return str(cand)
    try:
        import shellcheck_py  # type: ignore
        cand = Path(shellcheck_py.__file__).parent / "resources" / "shellcheck"
        if cand.exists():
            return str(cand)
    except Exception:
        pass
    return None


def _baseline(repo: Path, tool: str) -> set[str]:
    p = repo / BASELINE_DIR / f"{tool}.txt"
    try:
        return {ln.rstrip() for ln in p.read_text().splitlines() if ln.strip()}
    except OSError:
        return set()


def _finding_lines(out: str) -> set[str]:
    return {ln.rstrip() for ln in out.splitlines() if ln.strip()}


def _lint_stage(run: Run, repo: Path, name: str, cmd: list[str], *,
                cwd: Path | None = None, strict: bool,
                findings_from: str = "stdout") -> Stage:
    """Run one lint; classify NEW-vs-baseline. Advisory unless strict."""
    if cmd[0] == "MISSING":
        return run.add(Stage(name, status="skip", detail="tool unavailable"))
    cp = run.run(cmd, cwd=cwd or repo, stage=f"lint:{name}")
    text = cp.stdout if findings_from == "stdout" else (cp.stdout + "\n" + cp.stderr)
    base = _baseline(repo, name)
    cur = _finding_lines(text)
    new = cur - base
    # The gate is NEW findings vs baseline — NOT the exit code. Tools like
    # shellcheck/verify_* exit nonzero whenever ANY finding exists, but those
    # are baselined/accepted; only new ones matter.
    if new:
        st = "fail" if strict else "warn"
        return run.add(Stage(name, exit=cp.returncode, status=st,
                             detail=f"{len(new)} new", salient=sorted(new)[:3]))
    if cur:
        # findings exist but all are in the baseline → accepted
        return run.add(Stage(name, exit=cp.returncode, status="ok", detail="baselined"))
    if cp.returncode != 0:
        # no parseable findings yet nonzero exit → the tool itself failed to run
        return run.add(Stage(name, exit=cp.returncode,
                             status=("fail" if strict else "warn"),
                             detail=f"exit {cp.returncode}",
                             salient=[ln for ln in text.splitlines()[:3]]))
    return run.add(Stage(name, exit=0, status="ok"))


def source_lint_specs(repo: Path) -> list[tuple]:
    """(name, cmd, findings_from) for every source lint. Single source of truth
    shared by run_source_lints and write_baselines."""
    py = sys.executable
    sc = _shellcheck_bin() or "MISSING"
    sh_files = sorted(str(p) for p in (repo / "qemu").glob("*.sh"))
    sh_files += sorted(str(p) for p in (repo / "qemu" / "lib").glob("*.sh"))
    rc = repo / "tools" / "rustconv" / "rustconv"
    lk = repo / "tools" / "lint_kernel.py"
    specs = [
        ("shellcheck", [sc, "-S", "warning", "-f", "gcc", *sh_files]
            if sc != "MISSING" and sh_files else ["MISSING"], "stdout"),
        ("py-syntax", [py, "-m", "compileall", "-q", "tests/qemu", "tools"], "both"),
    ]
    for tool, rel in (("verify_abi", "verify_abi.py"),
                      ("verify_ffi", "verify_ffi.py"),
                      ("verify_constants", "verify_constants.py")):
        f = rc / rel
        specs.append((tool, [py, str(f)] if f.is_file() else ["MISSING"], "both"))
    specs.append(("lint_kernel", [py, str(lk)] if lk.is_file() else ["MISSING"], "stdout"))
    return specs


def run_source_lints(run: Run, repo: Path, *, strict: bool) -> None:
    for name, cmd, src in source_lint_specs(repo):
        _lint_stage(run, repo, name, cmd, strict=strict, findings_from=src)


def write_baselines(repo: Path) -> dict:
    """Capture current findings into tools/lint-baseline/<name>.txt so the gate
    becomes 'zero NEW vs baseline' (plan §20.3). Returns {name: count}."""
    import subprocess
    out = {}
    bdir = repo / BASELINE_DIR
    bdir.mkdir(parents=True, exist_ok=True)
    for name, cmd, src in source_lint_specs(repo):
        if cmd[0] == "MISSING":
            continue
        try:
            cp = subprocess.run(cmd, cwd=repo, capture_output=True, text=True, timeout=600)
        except (OSError, subprocess.SubprocessError):
            continue
        text = cp.stdout if src == "stdout" else (cp.stdout + "\n" + cp.stderr)
        findings = sorted(_finding_lines(text))
        (bdir / f"{name}.txt").write_text("\n".join(findings) + ("\n" if findings else ""))
        out[name] = len(findings)
    return out


def run_binary_lints(run: Run, repo: Path, arch: str, *, strict: bool) -> None:
    """Need a built binary (run after `build`). scan_ud2 + sacred primitives."""
    py = sys.executable
    build_dir = repo / f"build-{arch}"
    ntos = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntos.is_file():
        run.add(Stage("scan_ud2", status="skip", detail="no binary"))
        run.add(Stage("sacred_primitives", status="skip", detail="no binary"))
        return
    ud2 = repo / "tools" / "scan_ud2.py"
    _lint_stage(run, repo, "scan_ud2",
                [py, str(ud2), "--check", str(build_dir)] if ud2.is_file() else ["MISSING"],
                strict=strict, findings_from="both")
    sp = repo / "tools" / "verify_sacred_primitives.sh"
    _lint_stage(run, repo, "sacred_primitives",
                ["bash", str(sp), str(build_dir)] if sp.is_file() else ["MISSING"],
                strict=strict, findings_from="both")
