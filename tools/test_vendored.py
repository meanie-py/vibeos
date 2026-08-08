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


def test_suspects_finds_the_wdf_tree_the_registry_does_not_cover():
    """The scan's reason for existing: ~400K lines of Microsoft code, uncatalogued.

    The container ratchet only watches sdk/lib/3rdparty, dll/3rdparty and external/, so
    a vendored tree living anywhere else is invisible to it. WDF is the largest example
    and is deliberately NOT registered — whether a driver framework this OS ships is a
    translation target is a judgement call, not a copyright grep.
    """
    reg = vendored.load(REPO)
    found = {d for d, _ in vendored.suspects(reg)}
    assert any(d.startswith("sdk/lib/drivers/wdf") for d in found)


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
