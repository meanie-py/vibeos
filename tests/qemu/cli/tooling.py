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
# tests/qemu was linted only below cli/, so the harness core -- vm.py, backend.py and
# test_smoke.py itself -- carried 29 findings, including two `assert False` in the
# north-star test that `python -O` strips. The whole tree is ruff- and format-checked;
_RUFF_ROOTS = ["tools/rustconv/rustconv", "tests/qemu", "tools/fex"]
_PYRIGHT_ROOTS = ["tools/rustconv/rustconv", "tests/qemu"]
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
    # The arm64 firmware-table parsers (MADT/SPCR/MCFG in the loader, IORT in the HAL)
    # are compiled only for the target, where nothing runs their tests; a wrong answer
    # from one is a dead device on a board nobody here has.
    (".", ".", "tools/test_arm64_firmware_tables.py"),
    # The Wi-Fi supplicant (ndis-rust's wpa.rs): a wrong MIC or key looks on the bench like
    # an access point silently dropping the handshake.
    (".", ".", "tools/test_wpa_supplicant.py"),
    # An oracle needs an oracle. verify_c_signatures reports its own type-resolution gaps as
    # defects in the code when it is wrong, and reports "verified" over types it never
    # resolved when it is blind — neither shows up in review or in its own output.
    (".", ".", "tools/test_verify_c_signatures.py"),
    (".", ".", "tools/test_scan_wide_literals.py"),
    (".", ".", "tools/test_scan_unaligned_wide.py"),
    (".", ".", "tools/test_scan_pack_drift.py"),
    # The apitest sweep's parser. Its failure mode is FALSE GREEN, not a crash --
    # it shipped scoring `afd_apitest send` OK on the boot whose BUGCHECK that
    # subtest caused, because the subtest printed a clean summary before taking
    # the kernel down. A parser that cannot tell a crash from a pass turns the
    # whole harness into decoration, so it gets its own oracle.
    (".", ".", "tools/test_parse_apitest_serial.py"),
    # Every payload build.sh gates its exe on pecheck (GUI subsystem, required and
    # forbidden imports). It replaced `objdump | grep -q` pipes that passed and failed
    # at random under pipefail, so it must not become a new untested oracle.
    (".", ".", "tools/payloads/test_pecheck.py"),
    (".", ".", "tools/payloads/olewinetest/test_classify_failures.py"),
    # kmtest lists are hand-kept; a name the arch compiles out reads as a NO-END crash.
    (".", ".", "tools/payloads/kmtest/test_kmtest_lists.py"),
    # appstorm64's guest set is hand-kept against appstorm's app table; an app missing
    # from it runs native and reads as an app failure.
    (".", ".", "tools/payloads/appstorm/test_appstorm64_roots.py"),
    # x86 conformance's writable root: a missed entry silently boots the read-only CD again.
    (".", ".", "tools/conformance/test_mk_writable_livecd.py"),
    # modrun's grader is the whole verdict (the guest only reports), so it gets an oracle.
    (".", ".", "tools/payloads/modrun/test_modrun.py"),
    # The payload drain budget. `launch_drain_secs` counted READ ITERATIONS while
    # being named and documented as seconds, so a chatty payload spent a declared
    # 7200s in 12-28 minutes. Nothing could see it: the boot still "worked", it
    # just stopped early, and the driver above turned that into false quarantine
    # entries blaming whichever subtest was in flight. Only a chatty-guest test
    # distinguishes the two implementations.
    (".", ".", "tools/test_payload_drain.py"),
    (".", ".", "tools/test_wine_registry_resources.py"),
    (".", ".", "tools/test_gen_delaylib.py"),
    (".", ".", "tools/test_gen_apisets_table.py"),
    # The characterisation driver's accusation rule. Its failure mode is to
    # quarantine INNOCENT tests -- it blamed seven for a drain bug, and four more
    # for the VM watchdog on the very next run after that was fixed. Raising
    # budgets only moves the next-lowest bound into place, so the oracle is on
    # the rule itself: no accusation without guest-side fault evidence.
    (".", ".", "tools/test_characterize_attribution.py"),
    # The probe-then-raw-deref scanner. It reports leads, so its failure modes
    # are false positives (which bury the real ones under the commented-out C
    # these files carry) and false negatives on the shape that actually
    # bugchecked. Both are pinned.
    (".", ".", "tools/test_scan_probe_then_deref.py"),
    (".", ".", "tools/test_fix_probe_then_deref.py"),
    (".", ".", "tools/test_gen_suites.py"),
    # win10_surface writes generated blocks into ~226 specs. Its failure modes are all
    # quiet: a data export lined out as code, a name declared twice, its own block read
    # back as "already declared", or the hand-written part of a spec re-flowed.
    (".", ".", "tools/test_win10_surface.py"),
    # The stub-hit report's parsing/ranking, and spec2def's `-stub=VALUE` built from source.
    (".", ".", "tools/test_stub_hits.py"),
    (".", ".", "tools/test_spec2def_stub_value.py"),
    # The UP and MP kernel cells name their Rust libraries by hand; keep them equal.
    (".", ".", "tools/test_kernel_cells_parity.py"),
    # Same reason, one layer out: verify_cohab_coverage gates the merge train but reads
    # OBJECT FILES, so it cannot join test_scanner_controls (which drives source-only
    # `check` lints from fixture dirs) and would otherwise have no oracle at all.
    (".", ".", "tools/test_verify_cohab_coverage.py"),
    (".", ".", "tools/test_scan_import_cc.py"),
    (".", ".", "tools/test_scan_octal_literals.py"),
    (".", ".", "tools/test_scan_fourcc_literals.py"),
    (".", ".", "tools/test_scan_match_arm_break.py"),
    (".", ".", "tools/test_scan_calloc_elem_type.py"),
    (".", ".", "tools/test_scan_varargs_spec_arity.py"),
    # The i386 no-SSE ABI time bomb: -sse,-sse2 on the builtin i686 triple is a
    # phase-out warning today and a hard error after a toolchain bump. Fires loudly
    # here, with the migration, the moment the pinned rustc turns it into an error.
    (".", ".", "tools/test_scan_i386_sse_abi.py"),
    (".", ".", "tools/test_scan_ptr_highword_mask.py"),
    (".", ".", "tools/test_scan_dowhile_continue.py"),
    (".", ".", "tools/test_scan_case_bitor_pattern.py"),
    (".", ".", "tools/test_rank_unused_assignments.py"),
    (".", ".", "tools/test_apply_dead_assignments.py"),
    (".", ".", "tools/test_apply_rustc_suggestions.py"),
    (".", ".", "tools/test_extern_to_use.py"),
    (".", ".", "tools/test_scan_byteptr_stride.py"),
    (".", ".", "tools/test_scan_gmem_flags.py"),
    (".", ".", "tools/test_scan_raw_bitfield.py"),
    (".", ".", "tools/test_scan_winerror_consts.py"),
    (".", ".", "tools/test_scan_export_shadows.py"),
    (".", ".", "tools/test_scan_res_named_types.py"),
    (".", ".", "tools/test_scan_bindgen_padding_index.py"),
    (".", ".", "tools/test_scan_extern_static_width.py"),
    (".", ".", "tools/test_ramdisk_tree.py"),
    (".", ".", "tools/test_scan_const_vs_island.py"),
    (".", ".", "tools/test_scan_raw_uart_writes.py"),
    # The callee pass of the user-pointer lint: its quieting rules (kernel arm,
    # early return, scoped rebind, null guard) each hid or invented rows once.
    (".", ".", "tools/test_scan_raw_user_access.py"),
    # The double-fetch lint's quieting rules (sibling arms, early-exit branch,
    # advanced source, inner-cast normalisation) each over- or under-fired once.
    (".", ".", "tools/test_scan_double_fetch.py"),
    # The safe-probe-deref lint: a range probe validates a user pointer that is then
    # touched raw. Its quieting rules (fault-tolerant copy, trusted kernel arm,
    # capture-shadow, field-of-capture) share _walk with scan_raw_user_access.
    (".", ".", "tools/test_scan_safe_probe_deref.py"),
    (".", ".", "tools/test_scan_ptr_typedef_indirection.py"),
    (".", ".", "tools/test_lint_facts.py"),
    (".", ".", "tools/test_cargo_link_context.py"),
    (".", ".", "tools/test_cargo_orchestrate.py"),
    (".", ".", "tools/test_rust_storage.py"),
    (".", ".", "tools/test_rust_storage_report.py"),
    (".", ".", "tools/test_artifact_inputs.py"),
    (".", ".", "tools/test_artifact_worker.py"),
    (".", ".", "tools/test_artifact_executor.py"),
    (".", ".", "tools/test_artifact_economics.py"),
    (".", ".", "tools/test_artifact_native_inputs.py"),
    (".", ".", "tools/test_artifact_dlltool.py"),
    (".", ".", "tools/test_artifact_consumer.py"),
    (".", ".", "tools/test_artifact_store.py"),
    (".", ".", "tools/test_artifact_runtime.py"),
    (".", ".", "tools/test_artifact_runtime_real.py"),
    (".", ".", "tools/test_artifact_ninja.py"),
    (".", ".", "tools/test_artifact_repair.py"),
    (".", ".", "tools/test_artifact_config.py"),
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
    (".", ".", "tools/test_link_pulls.py"),
    (".", ".", "tools/test_ninja_closure_prune.py"),
    (".", ".", "tools/test_gen_ctype_tables.py"),
    (".", ".", "tools/test_verify_lost_exports.py"),
    (".", ".", "tools/test_verify_translation_literals.py"),
    (".", ".", "tools/test_verify_ucrtbase_cohab.py"),
    (".", ".", "tools/test_ucrt_tier2_refset.py"),
    # The sharded sweep's completeness oracle. Its failure mode is FALSE
    # COMPLETENESS -- a run cut short reading as a finished one -- which is the
    # defect the single-boot harness shipped for weeks without anything noticing.
    (".", ".", "tools/test_ucrt_tier2_sweep.py"),
    (".", ".", "tools/test_docs_only.py"),
    # The port ledger's classification is the whole measurement: one Wine module left in
    # the core bucket moves the published coverage figure and nothing in the output says
    # so. Its build-graph cases skip when this worktree has no build-* dirs.
    (".", ".", "tools/test_port_audit.py"),
    (".", ".", "tools/test_wine_gap.py"),
    (".", ".", "tools/test_pe_equiv.py"),
    (".", ".", "tools/test_cargo_cell_gen.py"),
    # Upstream triage decides which unmerged commits are BUG ORACLES. Its first draft
    # called 31 of 65 commits oracle hits by counting winetests sources the build does not
    # compile by configuration; the wrong number reads exactly like the right one.
    (".", ".", "tools/test_upstream_triage.py"),
    (".", ".", "tools/test_verify_signatures.py"),
    # The vendored-code move plan is a reference list: a missed spelling (relative
    # add_subdirectory, a .cmake fragment, a ${VAR}-built path, an include root) is a
    # build that breaks after the move. Hermetic fixtures, no build dirs needed.
    (".", ".", "tools/test_vendor_move_plan.py"),
    (".", ".", "tools/test_hostdisk.py"),
    (".", ".", "tools/test_scan_flex_array_panic.py"),
    (".", ".", "tools/test_fn_call_to_macro.py"),
    (".", ".", "tools/test_check_freeldr_size.py"),
    (".", ".", "tools/test_rust_warning_census.py"),
    (".", ".", "tools/test_kernel_probe.py"),
    (".", ".", "tools/test_link_guard.py"),
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
    (".", ".", "tools/test_scan_export_thunks.py"),
    (".", ".", "tools/test_scan_xchg_ebx_deref.py"),
    (".", ".", "tools/test_scan_unresolved_imports.py"),
    (".", ".", "tools/test_scan_assert_polarity.py"),
    (".", ".", "tools/test_scan_dropped_asserts.py"),
    (".", ".", "tools/test_scan_byte_step_one.py"),
    (".", ".", "tools/test_scan_call_literal_vs_c.py"),
    (".", ".", "tools/test_scan_formatless_printf.py"),
    (".", ".", "tools/test_scan_ptr_typedef_cast_read.py"),
    (".", ".", "tools/test_strip_msvc_cmake.py"),
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
    # The structural merge drivers for gen_driver_types.sh and the cohab census, including
    # one real `git merge` through the configured driver (tools/merge_drivers.py).
    (".", ".", "tools/test_merge_drivers.py"),
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
        checks.append(
            (
                "rust:vibeos-lint",
                [
                    "cargo",
                    "test",
                    "--offline",
                    "--locked",
                    "--manifest-path",
                    "tools/vibeos-lint/Cargo.toml",
                    "--target-dir",
                    ".cache/vibeos-lint/test-target",
                ],
                {},
            )
        )
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
                    # The rustconv suite is 152 files run serially: 433s of a 433s
                    # check-tooling wall (every other check finishes under it), so
                    # it alone decides what a cache miss costs the whole fleet.
                    + (
                        ["-n", str(min(8, lint_jobs()))]
                        if target == "tools/test_build_invalidation.py"
                        or (pkg == "tools/rustconv" and target == "tests")
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

    if getattr(args, "skip_generated", False):
        from .tooling_receipts import GENERATED_INPUT_CHECKS

        skipped = sorted(label for label, _c, _k in checks if label in GENERATED_INPUT_CHECKS)
        checks = [check for check in checks if check[0] not in GENERATED_INPUT_CHECKS]
        for label in skipped:
            print(f"  - {label}  SKIPPED (--skip-generated: reads this tree's build output)")

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
            for label in sorted(receipts.stale_generated_hits()):
                i = next(i for i, (lbl, _cmd, _kw) in enumerate(checks) if lbl == label)
                _lbl, cmd, kw = checks[i]
                print(f"Generated headers changed under the reused {label} result; rerunning it.")
                results[i] = _run(label, cmd, repo, **kw)
                receipts.mark_rerun(label)
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
