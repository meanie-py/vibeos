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
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from .lints import lint_jobs
from .tooling_receipts import ReceiptSession

# venv that launched us (vibeos-cli picks the main checkout's .venv); its bin/
# holds ruff + pyright, and sys.executable is the matching interpreter.
_VBIN = Path(sys.executable).parent
_RUFF = str(_VBIN / "ruff")
_PYRIGHT = str(_VBIN / "pyright")

# Surface roots. rustconv carries its own [tool.ruff]/[tool.pyright]; the cli +
# tools/*.py use the repo-root config. ruff/pyright discover the nearest config
# per file, so passing the paths is enough.
# tools/fex was outside every lint root, which is how an unused import and an
# unlinted closure walker sat there unnoticed. A tool the merge train depends on
# belongs under the same standard as the rest of our python.
_RUFF_ROOTS = ["tools/rustconv/rustconv", "tests/qemu/cli", "tools/fex"]
_PYRIGHT_ROOTS = ["tools/rustconv/rustconv", "tests/qemu/cli"]
# (package_dir, pythonpath, pytest_target) — pytest needs each package importable.
_PYTEST_SUITES = [
    ("tools/rustconv", "tools/rustconv", "tests"),
    (".", ".", "tests/qemu/cli"),
    (".", ".", "tests/qemu/cli/test_portlease.py"),
    (".", ".", "tools/wow64gen/test_wow64gen.py"),
    # The SysWOW64 closure decides which DLLs a 32-bit guest can load at all, and
    # its DYNAMIC half has no other oracle: a provider reached only through the
    # Winsock catalog is invisible to the import walk, so dropping one is silent
    # until a guest calls socket() and fails somewhere unrelated.
    (".", ".", "tools/fex/test_wow64_closure.py"),
    # tools/test_vendored.py existed but was collected by nothing, so the registry
    # that gates "is this C ours to translate" had its oracle sitting unrun.
    (".", ".", "tools/test_vendored.py"),
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
    (".", ".", "tools/test_lint_facts.py"),
    (".", ".", "tools/test_cargo_link_context.py"),
    (".", ".", "tools/test_rust_storage.py"),
    (".", ".", "tools/test_host_jobs.py"),
    (".", ".", "tools/test_binary_facts.py"),
    (".", ".", "tools/test_binary_scanner_cache.py"),
    (".", ".", "tools/test_binary_corpus.py"),
    (".", ".", "tools/test_tooling_receipts.py"),
    (".", ".", "tools/test_registry_security_persistence.py"),
    (".", ".", "tools/test_nested_cargo_deps.py"),
    (".", ".", "tools/test_scan_invented_constants.py"),
    # These six existed and were collected by NOTHING — the same defect the two comments
    # above record, twice more. test_verify_ucrtbase_cohab.py is the sharpest case: it was
    # written the night the ucrtbase math flip landed, to prove the cohab checker parses
    # `-arch=` spec filters, and it had never run in the gate once. The enumeration is now
    # guarded by test_cli_units, so a new tools/test_*.py cannot go quiet again.
    (".", ".", "tools/test_conformance_probe.py"),
    (".", ".", "tools/test_export_census.py"),
    (".", ".", "tools/test_gen_ctype_tables.py"),
    (".", ".", "tools/test_verify_lost_exports.py"),
    (".", ".", "tools/test_verify_translation_literals.py"),
    (".", ".", "tools/test_verify_ucrtbase_cohab.py"),
    (".", ".", "tools/test_docs_only.py"),
    (".", ".", "tools/test_hostdisk.py"),
    (".", ".", "tools/test_scan_flex_array_panic.py"),
    (".", ".", "tools/test_kernel_probe.py"),
    (".", ".", "tools/test_scan_orphan_modules.py"),
    (".", ".", "tools/test_cohab_census.py"),
    (".", ".", "tools/test_build_invalidation.py"),
    (".", ".", "tools/test_bindgen_generation.py"),
    (".", ".", "tools/test_gate_source_partitions.py"),
    (".", ".", "tools/test_source_walk.py"),
    (".", ".", "tools/test_cargo_depgraph.py"),
    (".", ".", "tools/test_retarget_cargo_depfile.py"),
    (".", ".", "tools/test_rust_module_tree.py"),
    (".", ".", "tools/test_scan_self_recursive.py"),
    (".", ".", "tools/test_scan_dropped_asserts.py"),
    # The substrate half of scan_extern_decl_drift has no other oracle: its
    # findings are about declarations of symbols NOTHING in the tree defines, so
    # neither rustc nor the linker nor a same-crate definition can adjudicate one.
    # Its two scoping rules are also the kind that fail SILENTLY -- judging a crate
    # by an island it does not link invents findings, and resolving pointer width
    # arch-agnostically inside a `#[cfg(target_arch)]`d module invents three more.
    (".", ".", "tools/test_scan_extern_decl_drift.py"),
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
    # Unit tests for scan_com_double_release's matcher. The CONTROL in
    # test_scanner_controls proves the scanner is not blind; these pin the four
    # CORRECT spellings it must stay quiet about (release-then-null,
    # release-then-reassign, release with no guard, and a guard whose release
    # lives in a DIFFERENT function), since each was a false positive an earlier
    # cut of the matcher produced against real shipping code.
    (".", ".", "tools/test_scan_com_double_release.py"),
    # A WRITE-capable tool needs the same treatment as a read-only one, and more:
    # migrate_debug_assert rewrote 454 sites, and its failure modes (a dropped
    # message, a mangled path-qualified invocation, a silently skipped form) all
    # leave a tree that still compiles. It shipped `core::ntkrnl_sys::ASSERT!` into
    # 15 files on its first run.
    (".", ".", "tools/test_migrate_debug_assert.py"),
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
            # Label by the TARGET, not the package: twenty root-dir suites all
            # rendered as `pytest:.`, so a failure line named nothing runnable.
            label_target = target if pkg == "." else f"{pkg}/{target}"
            checks.append(
                (
                    f"pytest:{label_target}",
                    [sys.executable, "-m", "pytest", target, "-q"]
                    # Each invalidation case builds its own tmp_path project.
                    # Keep these live, but distribute cases across the host.
                    + (
                        ["-n", str(min(8, lint_jobs()))]
                        if target == "tools/test_build_invalidation.py"
                        else []
                    )
                    + (
                        ["--ignore=tests/qemu/cli/test_portlease.py"]
                        if target == "tests/qemu/cli"
                        else []
                    ),
                    {"cwd": repo / pkg, "env_pp": pp},
                )
            )

    from contextlib import nullcontext

    context = (
        ReceiptSession(repo) if os.environ.get("VIBEOS_TOOLING_CACHE") != "0" else nullcontext()
    )
    receipt_error = None
    with context as receipts:

        def execute(label, cmd, kw):
            def compute():
                return _run(label, cmd, repo, **kw)

            return receipts.run(label, cmd, compute, **kw) if receipts else compute()

        with ThreadPoolExecutor(max_workers=min(lint_jobs(), len(checks))) as pool:
            futures = [pool.submit(execute, label, cmd, kw) for label, cmd, kw in checks]
            results = [future.result() for future in futures]
        if receipts:
            try:
                receipts.finish()
            except (OSError, RuntimeError) as error:
                receipt_error = str(error)
            print(f"Reused {len(receipts.hits)}/{len(checks)} tooling checks with matching inputs.")

    failed = [(lbl, tail) for lbl, ok, tail in results if not ok]
    if receipt_error:
        failed.append(("receipt-validation", receipt_error))
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
