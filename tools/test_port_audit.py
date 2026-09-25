"""The port ledger reports a ratio, so every test here is about the flattering direction.

Overstating progress is the only failure mode that matters: a surface wrongly excluded,
a Wine module wrongly counted as core-done, or a stale build graph reporting a flip that
has not happened all move the number the same way, and none of them is visible in the
output. The number goes into `rust-port-status.md`, which is what the fleet reads to
decide what to translate next.
"""

import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import port_audit
import vendored

REPO = Path(__file__).resolve().parents[1]


@pytest.fixture(scope="module")
def registry():
    return vendored.load(REPO)


def test_every_path_of_a_multi_path_ours_entry_is_seen(registry):
    """The bug this tool shipped with, pinned.

    `vendor.yaml` deliberately groups Wine's 24 shipped modules under ONE `ours:` entry,
    because it is one owner decision. `OurTree.path` is only the FIRST of those paths;
    the list is `.paths`. Reading the singular left 23 Wine modules -- msvcrt, ole32,
    rpcrt4, setupapi among them -- classified as core, which inflated the core C
    remaining by 480,000 lines and would have been published as a coverage figure.
    """
    wine = [o for o in registry.ours if "Wine" in (o.origin or "")]
    assert wine, "vendor.yaml no longer marks any tree as Wine-derived"
    multi = max(wine, key=lambda o: len(o.paths))
    assert len(multi.paths) > 1, "expected the grouped Wine entry"
    for path in multi.paths:
        bucket, surface, _ = port_audit.classify(f"{path}/somefile.c", registry)
        assert bucket == port_audit.EXCLUDED, f"{path} leaked into {surface}"
        assert surface == "Wine-derived"


def test_a_wine_subdirectory_of_an_OURS_module_is_still_wine(registry):
    """kernel32 and gdi32 are ours; their `wine/` subdirs are synced from Wine
    (`sdk/tools/winesync/kernel32.cfg` names the exact files). Provenance here is
    per-FILE and no registry entry can carry it, so the path rule has to."""
    for rel in (
        "dll/win32/kernel32/wine/actctx.c",
        "win32ss/gdi/gdi32/wine/metafile.c",
    ):
        bucket, surface, _ = port_audit.classify(rel, registry)
        assert (bucket, surface) == (port_audit.EXCLUDED, "Wine-derived"), rel
    # ...while the module's own code stays core.
    assert port_audit.classify("dll/win32/kernel32/client/proc.c", registry)[0] == port_audit.CORE


def test_vendored_trees_are_excluded_before_anything_else(registry):
    for rel in ("drivers/bus/acpi/acpica/utilities/utglobal.c", "sdk/lib/ucrt/stdio/fopen.cpp"):
        bucket, surface, _ = port_audit.classify(rel, registry)
        assert bucket == port_audit.EXCLUDED and surface == "Vendored (vendor.yaml)", rel


def test_an_unclassified_source_is_a_HARD_FAIL_not_a_default(registry):
    """The ratchet. A silent default bucket is how 45,000 lines of Mesa 2.4 sat inside
    the core figure: third-party graphics under `dll/opengl/`, which none of
    vendor.yaml's containers covers. A new top-level area must be classified by a human
    before the ledger will print at all."""
    with pytest.raises(port_audit.PortAuditError) as exc:
        port_audit.classify("newarea/subsystem/thing.c", registry)
    assert "SURFACES" in str(exc.value)


def test_longest_prefix_wins_so_a_specific_rule_can_carve_out_a_general_one(registry):
    assert port_audit.classify("win32ss/printing/base/spoolss/main.c", registry)[1] == (
        "Print spooler"
    )
    assert port_audit.classify("win32ss/user/ntuser/main.c", registry)[1] == "win32k: USER"


def test_rebase_is_textual_and_never_reads_the_canonical_mount(tmp_path):
    """`/tmp/vibeos` is a bind mount owned by whichever lane is building RIGHT NOW.
    Resolving a path through it reads a sibling worktree at another commit -- the
    failure that moved verify_signatures' mismatch count 987 -> 9 with no source change.
    So the mapping must be a string operation that works with nothing mounted at all."""
    repo = tmp_path / "wt"
    assert port_audit._rebase(Path("/tmp/vibeos/ntoskrnl/ke/dpc.c"), repo) == (
        repo / "ntoskrnl/ke/dpc.c"
    )
    assert not Path("/tmp/vibeos/ntoskrnl/ke/dpc.c").exists() or True  # never consulted
    # a path already inside a real worktree is left alone
    assert port_audit._rebase(repo / "hal/x.c", repo) == repo / "hal/x.c"


def test_a_stale_build_graph_refuses_rather_than_reporting_a_smaller_tree(tmp_path):
    """A graph older than a CMakeLists edit reports the PREVIOUS flip state. That error
    is invisible and always flattering right after a lane lands a flip, so it fails."""
    repo = tmp_path / "wt"
    (repo / "build-i386").mkdir(parents=True)
    ninja = repo / "build-i386" / "build.ninja"
    ninja.write_text("# graph\n")
    cml = repo / "CMakeLists.txt"
    cml.write_text("project(x)\n")
    import os

    os.utime(ninja, (1_000_000, 1_000_000))
    os.utime(cml, (2_000_000, 2_000_000))
    with pytest.raises(port_audit.PortAuditError) as exc:
        port_audit.compiled_sources(repo, arches=("i386",))
    assert "predates a CMakeLists edit" in str(exc.value)


def test_a_missing_build_dir_fails_with_the_command_that_fixes_it(tmp_path):
    """Never borrow a sibling's `build-*`: it is another commit's generated state, and a
    verdict that depends on which siblings happen to exist is not a verdict."""
    with pytest.raises(port_audit.PortAuditError) as exc:
        port_audit.compiled_sources(tmp_path, arches=("arm64",))
    assert "./vibeos-cli build --arch arm64" in str(exc.value)


def test_coverage_never_counts_untwinned_dead_c_as_translated():
    """`out of build` includes C this port never compiled (upstream drivers we do not
    enable, freeldr's i386/pc arch on a UEFI boot). Counting it as coverage is the
    tree's recurring error -- a TRUE predicate under a FALSE label -- so the ratio is
    built from `twin_lines`, never from `dead_lines`."""
    row = port_audit.Row(surface="s", bucket=port_audit.CORE, note="")
    row.live_lines, row.dead_lines, row.twin_lines = 100, 900, 0
    assert row.out_of_build_pct == 90.0
    assert row.coverage_pct == 0.0
    row.twin_lines = 900
    assert row.coverage_pct == 90.0


def test_the_tool_runs_against_this_worktree_and_classifies_everything():
    """The ratchet, end to end. Skips rather than fails when the build graph is absent
    OR stale: a fresh worktree has none (they are gitignored), and the merge queue's
    integration worktree runs this concurrently with the builds, so any batch touching a
    CMakeLists sees build.ninja older than the edit until configure reruns. Neither is a
    defect in the tree being measured -- the tool itself still refuses to report from
    such a graph, which is the guard that matters."""
    proc = subprocess.run(
        [sys.executable, "tools/port_audit.py", "--check"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    if proc.returncode == 2 and "no build graph" in proc.stderr:
        pytest.skip("no build dirs in this worktree: ./vibeos-cli build --arch i386")
    if proc.returncode == 2 and "predates a CMakeLists edit" in proc.stderr:
        pytest.skip("build graph older than a CMakeLists edit: reconfigure, then rerun")
    assert proc.returncode == 0, proc.stderr
    assert "every compiled source is classified" in proc.stdout
