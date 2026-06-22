"""Source + binary lint runners with baseline-diff (plan §20.3).

Gate = zero NEW findings vs a checked-in baseline (tools/lint-baseline/<tool>.txt),
never absolute zero — the tree carries known pre-existing findings. Advisory
(warn) in `build`'s auto-check; blocking-on-NEW only under `check --strict`.
"""

from __future__ import annotations

import shutil
import subprocess
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

        if shellcheck_py.__file__:
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


def _rel(text: str, repo: Path) -> str:
    """Strip the worktree's absolute repo prefix from tool output so findings
    are repo-relative — otherwise a baseline captured in one worktree never
    matches another (the abs path differs: vibeos vs vibeos-smoke-harness),
    making every finding spuriously 'new'. Makes baselines cross-worktree."""
    return text.replace(str(repo) + "/", "").replace(str(repo), "")


def _lint_stage(
    run: Run,
    repo: Path,
    name: str,
    cmd: list[str],
    *,
    cwd: Path | None = None,
    strict: bool,
    findings_from: str = "stdout",
    by_exit_code: bool = False,
) -> Stage:
    """Run one lint; classify NEW-vs-baseline. Advisory unless strict.

    by_exit_code: judge purely on the tool's --check exit status instead of
    line-baseline-diff. Required for BINARY lints (scan_ud2, sacred_primitives):
    their output is ARCH-SPECIFIC (i386 ud2 lines ≠ arm64 branch-to-self spins ≠
    amd64), so a single arch-agnostic baseline file can never match across arches
    — every line reads as "new". These tools already encode the correct per-arch
    pass/fail in their exit code (their own EXPECTED-site classification), so the
    exit code is the right signal."""
    if cmd[0] == "MISSING":
        return run.add(Stage(name, status="skip", detail="tool unavailable"))
    cp = run.run(cmd, cwd=cwd or repo, stage=f"lint:{name}")
    if by_exit_code:
        text = _rel((cp.stdout or "") + "\n" + (cp.stderr or ""), repo)
        if cp.returncode == 0:
            return run.add(Stage(name, exit=0, status="ok"))
        st = "fail" if strict else "warn"
        salient = [ln for ln in text.splitlines() if "*** BUG" in ln or "FAIL" in ln][:3]
        return run.add(
            Stage(name, exit=cp.returncode, status=st, detail=f"exit {cp.returncode}",
                  salient=salient or text.splitlines()[:3])
        )
    text = cp.stdout if findings_from == "stdout" else (cp.stdout + "\n" + cp.stderr)
    text = _rel(text, repo)
    base = _baseline(repo, name)
    cur = _finding_lines(text)
    new = cur - base
    # The gate is NEW findings vs baseline — NOT the exit code. Tools like
    # shellcheck/verify_* exit nonzero whenever ANY finding exists, but those
    # are baselined/accepted; only new ones matter.
    if new:
        st = "fail" if strict else "warn"
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=st,
                detail=f"{len(new)} new",
                salient=sorted(new)[:3],
            )
        )
    if cur:
        # findings exist but all are in the baseline → accepted
        return run.add(Stage(name, exit=cp.returncode, status="ok", detail="baselined"))
    if cp.returncode != 0:
        # no parseable findings yet nonzero exit → the tool itself failed to run
        return run.add(
            Stage(
                name,
                exit=cp.returncode,
                status=("fail" if strict else "warn"),
                detail=f"exit {cp.returncode}",
                salient=[ln for ln in text.splitlines()[:3]],
            )
        )
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
        (
            "shellcheck",
            [sc, "-S", "warning", "-f", "gcc", *sh_files]
            if sc != "MISSING" and sh_files
            else ["MISSING"],
            "stdout",
        ),
        ("py-syntax", [py, "-m", "compileall", "-q", "tests/qemu", "tools"], "both"),
    ]
    for tool, rel in (
        ("verify_abi", "verify_abi.py"),
        ("verify_ffi", "verify_ffi.py"),
        ("verify_constants", "verify_constants.py"),
    ):
        f = rc / rel
        specs.append((tool, [py, str(f)] if f.is_file() else ["MISSING"], "both"))
    specs.append(("lint_kernel", [py, str(lk)] if lk.is_file() else ["MISSING"], "stdout"))
    # C↔Rust differential oracles in tools/ (offsets are a compile-time gate, not
    # here). tables + match_consts are clean today (empty baseline = strict);
    # signatures carries known dual-cfg-shim FPs, so the baseline gates NEW only.
    for tool, rel in (
        ("verify_tables", "verify_tables.py"),
        ("verify_match_consts", "verify_match_consts.py"),
        ("verify_signatures", "verify_signatures.py"),
    ):
        t = repo / "tools" / rel
        specs.append((tool, [py, str(t)] if t.is_file() else ["MISSING"], "stdout"))
    # Inlined flag/constant-literal lint (AST/tree-sitter). verify_constants only
    # checks named `const` defs; this catches bare `0x2 /* TOP_LEVEL */` literals.
    # Carries residual FP classes (mask unions, informational args) → baseline gates NEW.
    sif = repo / "tools" / "scan_inline_flag_literals.py"
    specs.append(
        (
            "scan_inline_flag_literals",
            [py, str(sif), "--lint"] if sif.is_file() else ["MISSING"],
            "stdout",
        )
    )
    return specs


def run_source_lints(run: Run, repo: Path, *, strict: bool) -> None:
    for name, cmd, src in source_lint_specs(repo):
        _lint_stage(run, repo, name, cmd, strict=strict, findings_from=src)


def write_baselines(repo: Path) -> dict:
    """Capture current findings into tools/lint-baseline/<name>.txt so the gate
    becomes 'zero NEW vs baseline' (plan §20.3). Returns {name: count}."""
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
        text = _rel(cp.stdout if src == "stdout" else (cp.stdout + "\n" + cp.stderr), repo)
        findings = sorted(_finding_lines(text))
        (bdir / f"{name}.txt").write_text("\n".join(findings) + ("\n" if findings else ""))
        out[name] = len(findings)
    return out


# --- cargo-driven link PoCs (off-default-path → bit-rot risk) ---------------
# Each: (label, crate dir under sdk/lib/rust, cargo --bin, supported arches,
#        plausible-output floor in bytes). The cargo-driven links replace the
#        cmake link long-term; they live OFF the default build path, so nothing
#        else notices when their machinery (header-free C shims, pre-expanded
#        .s, self-hosted spec2def, the static-lib list, the reactos-link engine
#        itself, or the checked-in .res) drifts out of sync with the tree.
_CARGO_LINK_POCS = [
    ("ntoskrnl", "ntoskrnl-cargo", "ntoskrnl", ("i386", "amd64", "arm64"), 1_000_000),
    ("ntdll", "ntdll-cargo", "ntdll", ("i386", "amd64", "arm64"), 1_000_000),
    ("ntdll_vista", "ntdll_vista-cargo", "ntdll_vista", ("i386", "amd64", "arm64"), 200_000),
]
_ARCH_TARGET = {"i386": "i686-kernel", "amd64": "x86_64-kernel", "arm64": "aarch64-kernel"}


def _one_cargo_link(
    run: Run, repo: Path, rust_ws: Path, label: str, crate: str, binname: str, arch: str, floor: int
) -> None:
    """Re-run one cargo-driven link and assert it still produces a valid PE,
    WITHOUT booting. Skips cleanly when prerequisites are absent."""
    stage = f"cargo_link:{label}:{arch}"
    target = _ARCH_TARGET[arch]
    if not (rust_ws / crate / f"{target}.json").is_file():
        run.add(Stage(stage, status="skip", detail="PoC absent"))
        return
    # The engine reuses cmake-built sibling .a (librtl is universal); a missing
    # build-<arch> means the inputs the link consumes were never produced.
    if not (repo / f"build-{arch}" / "sdk" / "lib" / "rtl" / "librtl.a").is_file():
        run.add(Stage(stage, status="skip", detail=f"build-{arch} not built"))
        return
    if shutil.which("rustup") is None:
        run.add(Stage(stage, status="skip", detail="no rustup (need nightly+rust-src)"))
        return

    cp = run.run(
        [
            "cargo",
            "+nightly",
            "build",
            "--release",
            "--target",
            f"{crate}/{target}.json",
            "-Z",
            "build-std=core,alloc,compiler_builtins",
            "-Z",
            "build-std-features=compiler-builtins-mem",
            "-p",
            crate,
            "--bin",
            binname,
        ],
        cwd=rust_ws,
        stage=stage,
        timeout=900,
    )
    out = rust_ws / "target" / target / "release" / f"{binname}.exe"
    if cp.returncode != 0:
        tail = [ln for ln in (cp.stderr or "").splitlines() if ln.strip()][-3:]
        run.add(
            Stage(
                stage,
                exit=cp.returncode,
                status="fail",
                detail="link FAILED — cmake→cargo PoC bit-rotted",
                salient=tail,
            )
        )
        return
    if not out.is_file():
        run.add(Stage(stage, status="fail", detail="reported success but no output PE"))
        return
    size = out.stat().st_size
    if size < floor:
        run.add(
            Stage(stage, status="fail", detail=f"{binname}.exe implausibly small ({size} bytes)")
        )
        return
    with open(out, "rb") as f:
        if f.read(2) != b"MZ":
            run.add(Stage(stage, status="fail", detail="output not a PE (no MZ header)"))
            return
    run.add(Stage(stage, exit=0, status="ok", detail=f"{binname}.exe {size // 1024}K"))


def run_cargo_link_guard(run: Run, repo: Path, arch: str | None = None) -> None:
    """Guard every cargo-driven link PoC against silent bit-rot.

    Re-runs the cargo-driven links (`ntoskrnl-cargo`, `ntdll-cargo`,
    `ntdll_vista-cargo` via the shared `reactos-link` engine) and asserts each
    still produces a valid PE. These are the long-term replacement for the
    cmake-driven links; because they live OFF the default build path, the
    pre-commit/merge gate (`check --strict`) is exactly where to catch them
    rotting. Defaults to i386 (canonical, cheapest, the only arch all three
    PoCs share); pass an arch to guard ntdll/ntdll_vista on amd64/arm64.

    This is the LINK half of the gate (PE-valid, fast, no boot). Its runtime
    complement is `vibeos-cli cargo-swap` (tests/qemu/cli/cargoswap.py): that
    boot-swaps the cargo-built ntdll DLLs into the livecd and boots them via
    `vibeos-cli smoke`, catching a link that relinks clean but boots broken.
    Kept separate because cargo-swap boots QEMU (minutes) — too slow for the
    hot `check` loop; run it before trusting a cargo-drive handoff.

    Each link self-skips when its prerequisites (the crate's target json, a
    built `build-<arch>`, or nightly+rust-src) are absent, so this never blocks
    a tree without the PoCs. Gated to `--strict` (not every `build`) because
    each relink re-compiles a Rust staticlib into its own target dir (~20s) and
    would tax the hot loop."""
    rust_ws = repo / "sdk" / "lib" / "rust"
    arch = arch or "i386"
    for label, crate, binname, arches, floor in _CARGO_LINK_POCS:
        if arch in arches:
            _one_cargo_link(run, repo, rust_ws, label, crate, binname, arch, floor)


def run_committed_drift_guard(run: Run, repo: Path, arch: str | None = None) -> None:
    """Guard the cmake-off committed fallbacks (ks*.inc / cflags / .res / widl
    `.rs`) against silent staleness.

    Those artifacts are committed so a cargo build needs no cmake, but they are
    NOT regenerated on a normal build — so an upstream change (a `.template.h`,
    `.rc`, `.spec`/`.idl`, or cmake flags) silently staleens the committed copy
    while the configured cmake build stays green (it uses the fresh copy). Only a
    cmake-off build would then diverge. `tools/check_committed_drift.py`
    regenerates each from the configured `build-<arch>/` and byte-compares; this
    stage surfaces a DRIFT as a gate failure. Self-skips per artifact when the
    build dir / inputs are absent (so it never blocks a tree without builds)."""
    checker = repo / "tools" / "check_committed_drift.py"
    if not checker.is_file():
        run.add(Stage("committed_drift", status="skip", detail="checker absent"))
        return
    cmd = [sys.executable, str(checker)]
    if arch:
        cmd += ["--arch", arch]
    cp = subprocess.run(cmd, cwd=repo, capture_output=True, text=True)
    tail = (cp.stdout or "").strip().splitlines()
    summary = next((ln.strip() for ln in reversed(tail) if " OK," in ln), "")
    if cp.returncode == 0:
        if "0 OK," in summary and "skipped" in summary:
            run.add(Stage("committed_drift", status="skip", detail="no build dir"))
        else:
            run.add(Stage("committed_drift", exit=0, status="ok", detail=summary or "no drift"))
        return
    run.add(Stage("committed_drift", exit=cp.returncode, status="fail", detail=summary or "DRIFT"))


def run_binary_lints(run: Run, repo: Path, arch: str, *, strict: bool) -> None:
    """Need a built binary (run after `build`). scan_ud2 + sacred primitives."""
    py = sys.executable
    build_dir = repo / f"build-{arch}"
    ntos = build_dir / "ntoskrnl" / "ntoskrnl.exe"
    if not ntos.is_file():
        run.add(Stage("scan_ud2", status="skip", detail="no binary"))
        run.add(Stage("sacred_primitives", status="skip", detail="no binary"))
        return
    # Binary lints are judged by exit code (by_exit_code=True), not baseline-diff:
    # their output is arch-specific so a single baseline can't match across arches
    # (scan_ud2 emits ud2 on x86 vs branch-to-self spins on arm64). The tools'
    # --check exit already encodes the correct per-arch verdict.
    ud2 = repo / "tools" / "scan_ud2.py"
    _lint_stage(
        run,
        repo,
        "scan_ud2",
        [py, str(ud2), "--check", str(build_dir)] if ud2.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
    sp = repo / "tools" / "verify_sacred_primitives.sh"
    _lint_stage(
        run,
        repo,
        "sacred_primitives",
        ["bash", str(sp), str(build_dir)] if sp.is_file() else ["MISSING"],
        strict=strict,
        by_exit_code=True,
    )
