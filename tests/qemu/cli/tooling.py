"""`vibeos-cli check-tooling` — the lint/type/test gate for our own Python tools.

One front door over the whole vibeos-cli tool surface: the rustconv pipeline
(incl. semdiff), the cli package itself, and the standalone tools/*.py scripts.
Runs ruff (lint), ruff format --check, pyright (types),
and pytest — the same standard each package's pyproject configures.

This is distinct from `vibeos-cli check` (which lints the *kernel* C/Rust source
against the baseline). This one keeps the tooling that builds the kernel sharp.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

# venv that launched us (vibeos-cli picks the main checkout's .venv); its bin/
# holds ruff + pyright, and sys.executable is the matching interpreter.
_VBIN = Path(sys.executable).parent
_RUFF = str(_VBIN / "ruff")
_PYRIGHT = str(_VBIN / "pyright")

# Surface roots. rustconv carries its own [tool.ruff]/[tool.pyright]; the cli +
# tools/*.py use the repo-root config. ruff/pyright discover the nearest config
# per file, so passing the paths is enough.
_RUFF_ROOTS = ["tools/rustconv/rustconv", "tests/qemu/cli"]
_PYRIGHT_ROOTS = ["tools/rustconv/rustconv", "tests/qemu/cli"]
# (package_dir, pythonpath, pytest_target) — pytest needs each package importable.
_PYTEST_SUITES = [
    ("tools/rustconv", "tools/rustconv", "tests"),
    (".", ".", "tests/qemu/cli/test_cli_units.py"),
    (".", ".", "tools/wow64gen/test_wow64gen.py"),
    # An oracle needs an oracle. verify_c_signatures reports its own type-resolution gaps as
    # defects in the code when it is wrong, and reports "verified" over types it never
    # resolved when it is blind — neither shows up in review or in its own output.
    (".", ".", "tools/test_verify_c_signatures.py"),
    (".", ".", "tools/test_scan_wide_literals.py"),
    # Same reason, one layer out: verify_cohab_coverage gates the merge train but reads
    # OBJECT FILES, so it cannot join test_scanner_controls (which drives source-only
    # `check` lints from fixture dirs) and would otherwise have no oracle at all.
    (".", ".", "tools/test_verify_cohab_coverage.py"),
    (".", ".", "tools/test_scan_import_cc.py"),
    (".", ".", "tools/test_scan_octal_literals.py"),
    (".", ".", "tools/test_kernel_probe.py"),
    (".", ".", "tools/test_scan_orphan_modules.py"),
    (".", ".", "tools/test_build_invalidation.py"),
    (".", ".", "tools/test_bindgen_generation.py"),
    (".", ".", "tools/test_cargo_depgraph.py"),
    (".", ".", "tools/test_retarget_cargo_depfile.py"),
    (".", ".", "tools/test_rust_module_tree.py"),
    (".", ".", "tools/test_scan_self_recursive.py"),
    # Same shape as the two above: the matcher is pure functions over a string, and its
    # positive control is the real demangler bug — a sequencing error that shipped because
    # no value-based check can see it.
    (".", ".", "tools/test_scan_arg_side_effects.py"),
    # Generalizes the two above from "this oracle has an oracle" to "every gate
    # lint has one, or a written reason it doesn't". A scanner reporting 0 means
    # nothing until something proves the matcher still fires: scan_bugcheck_codes
    # printed a clean 0 for six weeks over a live KeBugCheckEx(0x35u32,...)
    # (§9.120). Also fails on any gate lint that is in neither the control table
    # nor the documented-exemption list, so coverage gaps stay loud.
    (".", ".", "tools/test_scanner_controls.py"),
]


def _run(
    label: str, cmd: list[str], repo: Path, *, cwd: Path | None = None, env_pp: str | None = None
):
    """Run one check; return (label, ok, tail). Output captured, tail shown on fail."""
    env = None
    if env_pp is not None:
        env = {**os.environ, "PYTHONPATH": str(repo / env_pp)}
    proc = subprocess.run(cmd, cwd=cwd or repo, env=env, capture_output=True, text=True)
    out = (proc.stdout + proc.stderr).strip()
    return label, proc.returncode == 0, out


def check_tooling(args, repo: Path) -> int:
    tools_py = sorted(str(p.relative_to(repo)) for p in (repo / "tools").glob("*.py"))
    checks: list[tuple[str, list[str], dict]] = []

    # 1. ruff lint
    for root in _RUFF_ROOTS:
        checks.append((f"ruff:{root}", [_RUFF, "check", root], {}))
    checks.append(("ruff:tools/*.py", [_RUFF, "check", *tools_py], {}))
    # 2. ruff format --check
    for root in _RUFF_ROOTS:
        checks.append((f"fmt:{root}", [_RUFF, "format", "--check", root], {}))
    checks.append(("fmt:tools/*.py", [_RUFF, "format", "--check", *tools_py], {}))
    # 3. pyright (point at the venv interpreter so imports resolve)
    if not args.no_types:
        for root in _PYRIGHT_ROOTS:
            checks.append((f"pyright:{root}", [_PYRIGHT, "--pythonpath", sys.executable, root], {}))
    # 3b. Generated ground truth must match the headers it was derived from.
    # `rustconv/ddk_enum_constants.py` is the value set that fact sealing protects;
    # if an SDK header adds an enum constant and this goes stale, that constant
    # silently goes back to being copied by the model — the invisible-to-rustc class
    # that produced 61 invented kernel bugcheck codes (field manual §9.113).
    checks.append(
        (
            "generated:ddk_enum_constants",
            [sys.executable, "tools/rustconv/gen_enum_constants.py", "--check"],
            {},
        )
    )
    # 4. pytest
    if not args.no_tests:
        for pkg, pp, target in _PYTEST_SUITES:
            checks.append(
                (
                    f"pytest:{pkg}",
                    [sys.executable, "-m", "pytest", target, "-q"],
                    {"cwd": repo / pkg, "env_pp": pp},
                )
            )

    results = []
    for label, cmd, kw in checks:
        results.append(_run(label, cmd, repo, **kw))

    failed = [(lbl, tail) for lbl, ok, tail in results if not ok]
    width = max(len(lbl) for lbl, _, _ in results)
    for lbl, ok, _ in results:
        print(f"  {'✓' if ok else '✗'} {lbl.ljust(width)}")
    print()
    if failed:
        print(f"FAIL — {len(failed)}/{len(results)} checks failed:\n")
        for lbl, tail in failed:
            print(f"=== {lbl} ===")
            print("\n".join(tail.splitlines()[-25:]))
            print()
        return 1
    print(f"OK — all {len(results)} tooling checks passed.")
    return 0
