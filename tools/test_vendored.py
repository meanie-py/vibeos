"""Unit tests for the vendored-C registry (tools/vendored.py + vendor.yaml).

The registry's whole job is to be the ONE answer to "is this C ours to translate", so
the tests that matter are the ones proving it can say NO — a check that cannot fail is
worse than no check, because green stops anyone looking again.
"""

from __future__ import annotations

import textwrap
from pathlib import Path

import pytest
import vendored

REPO = Path(__file__).resolve().parent.parent


def _write(tmp: Path, body: str) -> Path:
    """A minimal repo skeleton with a registry, for the negative cases."""
    (tmp / "vendor.yaml").write_text(textwrap.dedent(body))
    return tmp


# ── the real registry ───────────────────────────────────────────────────────


def test_the_committed_registry_agrees_with_the_tree():
    """`vibeos-cli check` runs exactly this."""
    reg = vendored.load(REPO)
    assert vendored.check(reg) == []


def test_the_committed_registry_registers_the_trees_that_motivated_it():
    reg = vendored.load(REPO)
    paths = {t.path for t in reg.trees}
    # ucrt is why this exists (task #55); ACPICA and FreeType were the two the prose
    # named; mbedtls was the largest omission from the tuple this replaced.
    for p in (
        "sdk/lib/ucrt",
        "drivers/bus/acpi/acpica",
        "sdk/lib/3rdparty/freetype",
        "dll/3rdparty/mbedtls",
    ):
        assert p in paths, f"{p} is not registered"


def test_ucrt_headers_are_vendored_but_our_msvcrt_is_not():
    """The boundary task #55 turns on: ucrtbase = vendored ucrt + OUR msvcrt_shared."""
    reg = vendored.load(REPO)
    assert reg.is_vendored("sdk/lib/ucrt/string/strupr.c")
    assert reg.is_vendored("sdk/include/ucrt/stdio.h")
    # msvcrt_shared is ours and must stay in the denominator.
    assert not reg.is_vendored("dll/win32/msvcrt/cpp.c")
    assert not reg.is_vendored("dll/win32/msvcrt/reactos/ucrt_tls_sup.c")


def test_reactos_own_code_under_libs_is_not_swept_up():
    """sdk/include/reactos/libs/ mixes foreign with ours; the registry must not glob."""
    reg = vendored.load(REPO)
    for ours in (
        "audiosrv",
        "samsrv",
        "umpnpmgr",
        "syssetup",
        "fslib",
        "epsapi",
        "cportlib",
        "fast486",
    ):
        assert not reg.is_vendored(f"sdk/include/reactos/libs/{ours}/x.h"), ours
    for foreign in ("zlib", "mbedtls", "libpng"):
        assert reg.is_vendored(f"sdk/include/reactos/libs/{foreign}/x.h"), foreign


def test_prefix_match_respects_path_boundaries():
    """`sdk/lib/ucrtx` must not be swallowed by the entry for `sdk/lib/ucrt`."""
    reg = vendored.load(REPO)
    assert reg.is_vendored("sdk/lib/ucrt")
    assert not reg.is_vendored("sdk/lib/ucrtx/a.c")


def test_every_version_probe_resolves():
    """A probe that stops matching is a stale pointer, not an unknown version."""
    reg = vendored.load(REPO)
    for t in reg.trees:
        version, err = reg.resolve_version(t)
        assert err is None, err
        if t.version:
            assert version and version != "unknown", f"{t.name} probe produced nothing"


# ── the check must be able to fail ──────────────────────────────────────────


def test_unregistered_tree_in_a_container_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: [libs]
        trees:
          - path: libs/known
            name: Known
            reason: test
    """,
    )
    (tmp_path / "libs" / "known").mkdir(parents=True)
    (tmp_path / "libs" / "surprise").mkdir()
    reg = vendored.load(tmp_path)
    problems = vendored.check(reg)
    assert any("UNREGISTERED" in p and "libs/surprise" in p for p in problems)


def test_missing_path_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: nope/gone
            name: Gone
            reason: test
    """,
    )
    assert any("does not exist" in p for p in vendored.check(vendored.load(tmp_path)))


def test_stale_version_probe_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/x
            name: X
            reason: test
            version:
              file: libs/x/v.h
              pattern: '#define X_VERSION "([^"]+)"'
    """,
    )
    (tmp_path / "libs" / "x").mkdir(parents=True)
    (tmp_path / "libs" / "x" / "v.h").write_text('#define X_RENAMED_VERSION "9"\n')
    assert any("no match" in p for p in vendored.check(vendored.load(tmp_path)))


def test_stale_stamp_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/x
            name: X
            upstream: http://example.invalid/
            reason: test
    """,
    )
    (tmp_path / "libs" / "x").mkdir(parents=True)
    reg = vendored.load(tmp_path)
    assert any("missing" in p and "VENDORED" in p for p in vendored.check(reg))
    vendored.write_stamps(reg)
    assert vendored.check(reg) == []
    (tmp_path / "libs" / "x" / "VENDORED").write_text("something else\n")
    assert any("STALE" in p for p in vendored.check(reg))


def test_duplicate_path_is_a_registry_error(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: a, name: A, reason: r}
          - {path: a, name: B, reason: r}
    """,
    )
    with pytest.raises(vendored.RegistryError, match="duplicate path"):
        vendored.load(tmp_path)


def test_missing_required_field_is_a_registry_error(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: a, name: A}
    """,
    )
    with pytest.raises(vendored.RegistryError, match="reason"):
        vendored.load(tmp_path)


def test_absent_registry_hard_fails_rather_than_reading_as_empty(tmp_path):
    """An empty registry would silently un-exclude 1.7M lines. Fail loud instead."""
    with pytest.raises(vendored.RegistryError, match="not found"):
        vendored.load(tmp_path)


def test_suspects_never_reports_an_already_registered_tree():
    """The discovery scan must not re-surface what the registry already answers."""
    reg = vendored.load(REPO)
    for d, _n in vendored.suspects(reg):
        assert not reg.is_vendored(d), f"{d} is registered yet reported as a suspect"


def test_every_foreign_copyright_directory_is_classified():
    """The ratchet. `--suspects` is now empty because each hit has a recorded verdict.

    It found ~400K lines of Microsoft code nobody had catalogued — WDF the largest, at
    290K. Each is now answered ONE way (task #62), which is what lets `check` treat a
    remaining hit as a finding: the list is undecided trees, not foreign ones.
    """
    reg = vendored.load(REPO)
    assert vendored.suspects(reg) == []


def test_the_two_halves_disagree_about_nothing():
    """Both verdicts are recorded, and no path carries both."""
    reg = vendored.load(REPO)
    # WDF is vendored: a stable ABI third-party binaries bind to, tracked upstream.
    assert reg.is_vendored("sdk/lib/drivers/wdf/shared/core/fxdevice.cpp")
    assert reg.is_vendored("sdk/include/wdf/kmdf/1.17/wdf.h")
    # The storage class drivers are ours: MS-PL, already diverged, no ABI contract.
    assert reg.is_declared_ours("drivers/storage/class/classpnp/class.c")
    assert not reg.is_vendored("drivers/storage/class/classpnp/class.c")
    # Neither list is a claim about the rest of the repo. The example used to be
    # dll/win32/msvcrt/cpp.c, which stopped being one on 2026-09-19: msvcrt is
    # Wine-derived, `--suspects` learned to see Wine, and the owner classified the whole
    # Wine bucket as `ours:`. So the example has to be code that is ReactOS-authored
    # rather than merely ours-by-decision.
    assert not reg.is_classified("ntoskrnl/ke/dpc.c")
    assert not reg.is_classified("base/shell/explorer/explorer.cpp")
    # And the Wine bucket really is the other thing: foreign copyright, ours by SCOPE,
    # deliberately IN the translation denominator rather than excluded from it.
    assert reg.is_declared_ours("dll/win32/msvcrt/cpp.c")
    assert not reg.is_vendored("dll/win32/msvcrt/cpp.c")
    assert reg.is_declared_ours("dll/win32/mshtml/htmldoc.c")


def test_an_undecided_foreign_tree_fails_the_check(tmp_path):
    """The whole point of `ours:` — an unanswered question must hold the gate red."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
    """,
    )
    (tmp_path / "vend").mkdir()
    (tmp_path / "newly_imported").mkdir()
    for i in range(6):
        (tmp_path / "newly_imported" / f"f{i}.c").write_text(
            "/* Copyright (c) Microsoft Corporation */\n"
        )
    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    assert any("UNCLASSIFIED" in p and "newly_imported" in p for p in vendored.check(reg))


def test_a_nested_worktree_is_not_this_tree(tmp_path):
    """Fleet isolation, read side: another lane's checkout is not our finding.

    Agents run worktrees nested inside worktrees (.claude/worktrees/<lane>), and an
    unguarded walk read one at a DIFFERENT commit and reported 50 UNCLASSIFIED trees
    that belong to another branch. A gate whose verdict changes with which worktrees
    happen to exist on disk is not a gate — and the pruning has to key on the checkout
    itself, not on the ".claude" name, or the next nesting convention walks right in.
    """
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
    """,
    )
    (tmp_path / "vend").mkdir()
    lane = tmp_path / "somewhere" / "another-lane"
    lane.mkdir(parents=True)
    # A LINKED worktree carries a .git FILE, which is the case that bit us.
    (lane / ".git").write_text("gitdir: /elsewhere/.git/worktrees/another-lane\n")
    for i in range(8):
        (lane / f"f{i}.c").write_text("/* Copyright (c) Microsoft Corporation */\n")

    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    assert vendored.check(reg) == [], "another worktree's files were reported as ours"

    # ...but the same files in OUR tree still have to be found, or the prune has
    # quietly become a way to hide foreign code from the registry.
    ours = tmp_path / "somewhere" / "our-import"
    ours.mkdir(parents=True)
    for i in range(8):
        (ours / f"f{i}.c").write_text("/* Copyright (c) Microsoft Corporation */\n")
    reg = vendored.load(tmp_path)
    assert any("UNCLASSIFIED" in p and "our-import" in p for p in vendored.check(reg))


def test_a_primary_checkout_nested_in_the_tree_is_also_skipped(tmp_path):
    """The .git DIRECTORY form — a plain clone someone left inside the tree."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
    """,
    )
    (tmp_path / "vend").mkdir()
    clone = tmp_path / "vendor-scratch"
    (clone / ".git").mkdir(parents=True)
    for i in range(8):
        (clone / f"f{i}.c").write_text("/* Copyright (c) Microsoft Corporation */\n")
    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    assert vendored.check(reg) == []


def test_declaring_it_ours_settles_it_without_pretending_it_is_vendored(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
        ours:
          - {path: newly_imported, name: NI, reason: we own this one}
    """,
    )
    (tmp_path / "vend").mkdir()
    (tmp_path / "newly_imported").mkdir()
    for i in range(6):
        (tmp_path / "newly_imported" / f"f{i}.c").write_text(
            "/* Copyright (c) Microsoft Corporation */\n"
        )
    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    assert vendored.check(reg) == []
    # An `ours:` tree is IN the denominator, so it must not be marked as excluded from it.
    assert not (tmp_path / "newly_imported" / "VENDORED").exists()
    assert not reg.is_vendored("newly_imported/f0.c")


def test_a_path_cannot_be_both_vendored_and_ours(tmp_path):
    """Otherwise the answer depends on which list the caller happens to consult."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: libs/x, name: X, reason: r}
        ours:
          - {path: libs/x, name: X again, reason: r}
    """,
    )
    with pytest.raises(vendored.RegistryError, match="cannot be both"):
        vendored.load(tmp_path)


def test_a_missing_ours_path_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
        ours:
          - {path: gone, name: Gone, reason: r}
    """,
    )
    (tmp_path / "vend").mkdir()
    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    assert any("`ours:` path does not exist" in p for p in vendored.check(reg))


def test_suspects_ignores_a_registered_tree_in_a_temp_repo(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - {path: vend, name: V, reason: r}
    """,
    )
    (tmp_path / "vend").mkdir()
    (tmp_path / "vend" / "a.c").write_text("/* Copyright (c) Microsoft Corporation */\n")
    (tmp_path / "ours").mkdir()
    (tmp_path / "ours" / "b.c").write_text("/* Copyright (c) Microsoft Corporation */\n")
    reg = vendored.load(tmp_path)
    vendored.write_stamps(reg)
    found = dict(vendored.suspects(reg, min_files=1))
    assert "ours" in found
    assert "vend" not in found
