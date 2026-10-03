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
    # ucrt is why this exists (task #55; its source is gone, its headers remain); ACPICA
    # and FreeType were the two the prose named; mbedtls was the largest omission from
    # the tuple this replaced.
    for p in (
        "sdk/include/ucrt",
        "vendor/acpica",
        "vendor/freetype",
        "vendor/mbedtls",
    ):
        assert p in paths, f"{p} is not registered"


def test_ucrt_headers_are_vendored_but_our_msvcrt_is_not():
    """The boundary task #55 turned on. ucrt's source is ours now (ucrt-rust) and gone;
    what is still Microsoft's is the headers, and a path under the deleted tree must not
    count as vendored -- that would hide a file reappearing there from every scan."""
    reg = vendored.load(REPO)
    assert not reg.is_vendored("sdk/lib/ucrt/string/strupr.c")
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
    # mbedtls's headers still sit in that mixed directory; zlib's and libpng's moved into
    # their own vendor/<lib>/include with the rest of the library (2026-09-26).
    assert reg.is_vendored("vendor/mbedtls/include/mbedtls/x.h")
    for foreign in ("zlib", "libpng"):
        assert reg.is_vendored(f"vendor/{foreign}/include/x.h"), foreign


def test_prefix_match_respects_path_boundaries():
    """`sdk/include/ucrtx` must not be swallowed by the entry for `sdk/include/ucrt`."""
    reg = vendored.load(REPO)
    assert reg.is_vendored("sdk/include/ucrt")
    assert not reg.is_vendored("sdk/include/ucrtx/a.h")


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


def test_a_single_file_tree_is_registered_without_a_stamp(tmp_path):
    """Upstream vendors a few loose files into ReactOS-authored directories (setup's
    treelist.c, the libdxg d3dkmthk.h). They must be registrable, must own exactly their
    own path and not their neighbours, and must not demand a VENDORED stamp -- there is
    no directory of theirs to put one in, and one in the parent would claim the parent.
    """
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: app/tree.c
            name: Tree
            reason: test
            headers:
              - app/tree.h
    """,
    )
    (tmp_path / "app").mkdir()
    (tmp_path / "app" / "tree.c").write_text("int a;\nint b;\n")
    (tmp_path / "app" / "tree.h").write_text("int a;\n")
    (tmp_path / "app" / "main.c").write_text("int main;\n")
    reg = vendored.load(tmp_path)
    assert vendored.check(reg) == []
    assert vendored.write_stamps(reg) == []
    assert reg.is_vendored("app/tree.c") and reg.is_vendored("app/tree.h")
    assert not reg.is_vendored("app/main.c")
    assert not reg.is_vendored("app/tree.cpp")
    assert reg.loc_path("app/tree.c") == 2
    (tmp_path / "app" / "tree.h").unlink()
    assert any("header path does not exist" in p for p in vendored.check(reg))


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
    assert reg.is_vendored("vendor/wdf/shared/core/fxdevice.cpp")
    assert reg.is_vendored("vendor/wdf/include/kmdf/1.17/wdf.h")
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


# ── the structural check: a field swallowed by the value above it ───────────


def test_structural_check_catches_a_field_joined_onto_the_previous_value(tmp_path):
    """The failure this exists for is NOT a syntax error, which is what makes it
    dangerous: `license: MIT    modified: yes` parses fine and `modified` silently
    stops being a field. Broke vendor.yaml three times in one session this way."""
    f = tmp_path / "vendor.yaml"
    f.write_text("trees:\n  - path: a    modified: yes\n    name: A\n")
    problems = vendored.structural_problems(f)
    assert len(problems) == 1 and "modified:" in problems[0]


def test_structural_check_catches_a_field_swallowed_by_a_block_scalar(tmp_path):
    """The other shape: a `>` block eats the next key because the inserted prose had
    no trailing newline."""
    f = tmp_path / "vendor.yaml"
    f.write_text("    review: >\n      some prose here.    reason: X\n")
    assert len(vendored.structural_problems(f)) == 1


def test_structural_check_does_not_fire_on_ordinary_prose(tmp_path):
    """It must be able to pass: prose inside a block scalar routinely contains colons,
    and a check that flags those would be deleted within a week."""
    f = tmp_path / "vendor.yaml"
    f.write_text(
        "trees:\n  - path: a\n    modified: yes\n    reason: fine\n"
        "    review: >\n      prose mentioning a colon: like this, and review of things\n"
    )
    assert vendored.structural_problems(f) == []


def test_the_committed_registry_has_no_swallowed_fields():
    assert vendored.structural_problems(REPO / "vendor.yaml") == []


# ── generated notices ───────────────────────────────────────────────────────


def test_the_committed_notices_file_is_current():
    """The shipped THIRD-PARTY-NOTICES.md matches what the registry generates now.

    It is checked in so a distributor gets it without running our tooling, which means
    it can drift; this is the check that stops it.
    """
    reg = vendored.load(REPO)
    path = REPO / vendored.NOTICES_PATH
    # Existence is asserted HERE rather than in check(), which is pointed at synthetic
    # fixture repos that have no business shipping a legal document.
    assert path.is_file(), (
        f"{vendored.NOTICES_PATH} is missing — "
        f"generate with: tools/vendored.py --notices {vendored.NOTICES_PATH}"
    )
    committed = path.read_text(encoding="utf-8")
    assert committed == vendored.notices(reg), (
        f"{vendored.NOTICES_PATH} is stale — "
        f"regenerate with: tools/vendored.py --notices {vendored.NOTICES_PATH}"
    )


def test_every_tree_has_a_reproducible_grant_or_says_why_not():
    reg = vendored.load(REPO)
    for t in reg.trees:
        source, text = vendored.license_text(reg, t)
        assert text.strip(), f"{t.name}: empty licence text from {source}"


def test_notices_name_every_registered_tree():
    reg = vendored.load(REPO)
    doc = vendored.notices(reg)
    for t in reg.trees:
        assert f"## {t.name}" in doc, f"{t.name} has no section in the notices"


def test_notices_carry_the_extra_licences_not_just_the_headline():
    """FEX is recorded MIT and also carries LGPL/GPL; a notices file built from
    `license:` alone would be wrong about it."""
    reg = vendored.load(REPO)
    doc = vendored.notices(reg)
    fex = next(t for t in reg.trees if t.name == "FEX-Emu")
    assert fex.license_extra, "FEX should still record extra licences"
    for lic in fex.license_extra:
        assert f"Also contains: {lic}" in doc


def test_a_tree_with_no_grant_and_no_note_fails(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            reason: test
    """,
    )
    (tmp_path / "libs" / "thing").mkdir(parents=True)
    problems = vendored.check(vendored.load(tmp_path))
    assert any("--notices to quote" in p for p in problems)


def test_a_license_note_satisfies_the_grant_requirement(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            license_note: upstream is MIT but the drop shipped no LICENSE file
            reason: test
    """,
    )
    (tmp_path / "libs" / "thing").mkdir(parents=True)
    problems = vendored.check(vendored.load(tmp_path))
    assert not any("--notices to quote" in p for p in problems)
    reg = vendored.load(tmp_path)
    source, text = vendored.license_text(reg, reg.trees[0])
    assert source == "NO GRANT IN TREE"
    assert "shipped no LICENSE file" in text


def test_license_ref_extraction_stops_at_the_end_of_the_header_comment(tmp_path):
    """Regression: `#` was treated as a comment lead, so extraction ran past the
    licence and emitted the file's `#ifndef`/`#include` lines with hashes stripped."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            license_ref: libs/thing/thing.h
            reason: test
    """,
    )
    d = tmp_path / "libs" / "thing"
    d.mkdir(parents=True)
    (d / "thing.h").write_text(
        "/*\n"
        " * Copyright (c) 1999 Somebody\n"
        " * Permission is hereby granted, free of charge.\n"
        " */\n"
        "\n"
        "#ifndef THING_H\n"
        "#define THING_H\n"
        "#include <stdio.h>\n"
        "#endif\n"
    )
    reg = vendored.load(tmp_path)
    _source, text = vendored.license_text(reg, reg.trees[0])
    assert "Permission is hereby granted" in text
    assert "ifndef" not in text
    assert "include" not in text


def test_license_ref_extraction_handles_line_comments(tmp_path):
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            license_ref: libs/thing/thing.c
            reason: test
    """,
    )
    d = tmp_path / "libs" / "thing"
    d.mkdir(parents=True)
    (d / "thing.c").write_text(
        "// Copyright (c) 2001 Somebody\n"
        "// Redistribution is permitted.\n"
        "\n"
        "int main(void) { return 0; }\n"
    )
    reg = vendored.load(tmp_path)
    _source, text = vendored.license_text(reg, reg.trees[0])
    assert "Redistribution is permitted." in text
    assert "int main" not in text


# ── the prose half of the licence scan (added 2026-09-20) ───────────────────


def _tree_with_file(tmp: Path, recorded: str, body: str, extra: str = "") -> object:
    """A one-tree registry whose single source file says `body` about itself."""
    _write(
        tmp,
        f"""
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: {recorded}
            license_note: test
{extra}            reason: test
    """,
    )
    d = tmp / "libs" / "thing"
    d.mkdir(parents=True)
    (d / "thing.c").write_text(body)
    return vendored.load(tmp)


def test_a_prose_licence_name_with_no_spdx_tag_is_caught(tmp_path):
    """The gap that motivated this: glu32's 73 SGI-B-1.1 files and zlib's Info-ZIP
    minizip both sat in the tree while --licences printed all-clear, because neither
    states its licence in a machine-readable form."""
    reg = _tree_with_file(
        tmp_path,
        "MIT",
        "/* This library is free software under the terms of the\n"
        " * GNU Lesser General Public License as published by the FSF. */\n",
    )
    drift = vendored.license_drift(reg)
    assert drift, "an LGPL file inside an MIT tree must be reported"
    assert "LGPL (prose)" in drift[0][2]


def test_an_lgpl_tree_does_not_absorb_a_real_gpl_file(tmp_path):
    """Substring matching cannot do this job: "LGPL-2.1" contains "GPL". If the
    accounting test were `in`, a tree recorded LGPL would silently swallow a GPL file,
    which is the copyleft escalation this whole check exists to surface."""
    reg = _tree_with_file(
        tmp_path,
        "LGPL-2.1",
        "/* Licensed under the GNU General Public License, version 2. */\n",
    )
    drift = vendored.license_drift(reg)
    assert drift, "a GPL file inside an LGPL tree must be reported"
    assert "GPL (prose)" in drift[0][2]


def test_an_lgpl_file_in_an_lgpl_tree_is_quiet(tmp_path):
    reg = _tree_with_file(
        tmp_path,
        "LGPL-2.1",
        "/* GNU Lesser General Public License, version 2.1 or later. */\n",
    )
    assert vendored.license_drift(reg) == []


def test_lesser_is_tried_before_plain_gpl(tmp_path):
    """Order in _PROSE_LICENCES is load-bearing: the LGPL notice contains the words
    "General Public License", so a plain-GPL pattern tried first reports every LGPL
    file as GPL and the real escalation becomes indistinguishable from noise."""
    reg = _tree_with_file(
        tmp_path,
        "MIT",
        "/* GNU Lesser General Public License */\n",
    )
    found = vendored.license_drift(reg)[0][2]
    assert "LGPL (prose)" in found and "GPL (prose)" not in found


def test_a_permissive_notice_body_does_not_fire(tmp_path):
    """Why prose matching is restricted to NAMES and to copyleft/named families: every
    ACPICA file reproduces a BSD notice, and reporting those would be 226 true hits that
    are all the recorded licence. Redistribution boilerplate must stay silent."""
    reg = _tree_with_file(
        tmp_path,
        "MIT",
        "/* Redistribution and use in source and binary forms, with or without\n"
        " * modification, are permitted provided that the following conditions\n"
        " * are met: ... THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AS IS */\n",
    )
    assert vendored.license_drift(reg) == []


def test_an_spdx_tag_suppresses_the_prose_pass_for_that_file(tmp_path):
    """A file that has already answered machine-readably is not asked again: otherwise
    every dual-licensed header reports twice, once as its tag and once as the family
    named in the notice underneath it."""
    reg = _tree_with_file(
        tmp_path,
        "MIT",
        "/* SPDX-License-Identifier: MIT\n"
        " * Portions derived from code under the GNU General Public License. */\n",
    )
    assert vendored.license_drift(reg) == []


def test_license_extra_settles_a_prose_finding(tmp_path):
    reg = _tree_with_file(
        tmp_path,
        "MIT",
        "/* GNU Lesser General Public License v2.1 */\n",
        extra="            license_extra:\n"
        "              LGPL-2.1-or-later: the one vendored dependency\n",
    )
    assert vendored.license_drift(reg) == []


def test_the_three_trees_the_prose_pass_actually_found_stay_recorded():
    """Regression guard on the real registry. These were found by the prose pass on
    2026-09-20 after the SPDX-only version had passed over them for weeks; dropping any
    of them from license_extra would make the notices file wrong again."""
    reg = vendored.load(REPO)
    by_path = {t.path: t for t in reg.trees}
    assert "BSL-1.0" in by_path["vendor/fex"].license_extra
    assert "LGPL-2.0-or-later" in by_path["vendor/adns"].license_extra
    assert "LGPL-2.1-or-later" in by_path["vendor/libtirpc"].license_extra


# ── the Rust-crate half (added 2026-09-20) ─────────────────────────────────


def test_every_external_crate_in_the_lockfile_is_recorded():
    """The gap that motivated this: smoltcp — the TCP/IP stack — shipped for months
    with no entry anywhere, because the registry modelled vendored C and nothing
    modelled what cargo pulls in."""
    lock = set(vendored.cargo_lock_crates(REPO))
    snap = {(n, v) for n, v, _lic, _ships in vendored.crates_rows(REPO)}
    assert lock == snap, (
        f"in Cargo.lock but not recorded: {sorted(lock - snap)}\n"
        f"recorded but gone from Cargo.lock: {sorted(snap - lock)}\n"
        "regenerate with: tools/vendored.py --write-crates"
    )


def test_the_crates_the_os_actually_links_are_marked_shipping():
    """A tripwire on the split collapsing. These four are load-bearing and obvious --
    smoltcp is the network stack, virtio-drivers and safe-mmio are how it reaches the
    hardware, zlib-rs is compression. If a refactor of the reachability walk quietly
    marked them host-only, the notices file would stop listing them and nothing else
    would notice."""
    ships = {n for n, _v, _l, s in vendored.crates_rows(REPO) if s}
    for name in ("smoltcp", "virtio-drivers", "safe-mmio", "zlib-rs"):
        assert name in ships, f"{name} is no longer marked as shipping"


def test_proc_macro_crates_are_never_marked_shipping():
    """A proc-macro runs inside the compiler and cannot be in the product. This is the
    one half of the split that is provable rather than best-effort, so it is the half
    worth asserting."""
    rows = {n: s for n, _v, _l, s in vendored.crates_rows(REPO)}
    for name in ("zerocopy-derive", "thiserror-impl", "defmt-macros", "enumn"):
        assert name in rows, f"{name} vanished from the snapshot"
        assert not rows[name], f"{name} is a proc-macro and must not be marked shipping"


def test_a_lockfile_crate_missing_from_the_snapshot_fails_the_check(tmp_path):
    """The ratchet has to be able to fail, or it is decoration."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            license_note: test
            reason: test
    """,
    )
    (tmp_path / "libs" / "thing").mkdir(parents=True)
    (tmp_path / "sdk" / "lib" / "rust").mkdir(parents=True)
    (tmp_path / "sdk" / "lib" / "rust" / "Cargo.lock").write_text(
        '[[package]]\nname = "brand-new-crate"\nversion = "1.0.0"\n'
        'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
    )
    (tmp_path / "tools").mkdir()
    (tmp_path / "tools" / "rust-crates.yaml").write_text("crates:\n")
    problems = vendored.check(vendored.load(tmp_path))
    assert any("brand-new-crate 1.0.0" in p for p in problems), problems


def test_the_notices_file_lists_the_shipped_crates():
    doc = vendored.notices(vendored.load(REPO))
    assert "## Rust crates" in doc
    for name in ("smoltcp", "virtio-drivers", "zlib-rs"):
        assert f"| {name} |" in doc, f"{name} is missing from the notices table"
    # ...and does NOT list the host-only ones, or the document overstates what ships.
    assert "| zerocopy-derive |" not in doc


# ── the declared-licence ratchet (added 2026-09-20) ────────────────────────


def test_no_file_declares_a_non_floss_licence():
    """The ratchet that would have caught drivers/storage/ide/atapi on day one: a
    shipped driver carrying the Windows NT DDK END-USER LICENSE AGREEMENT, invisible
    to every other scan here because atapi.c and atapi.h contain no copyright line at
    all and the directory does ship a licence file — it is just a EULA."""
    reg = vendored.load(REPO)
    rows = vendored.declared_licences(reg)
    assert rows == [], "\n".join(f"{n} file(s) declare `{v}` — e.g. {ex}" for v, ex, n, _d in rows)


def test_a_pointer_is_not_a_licence(tmp_path):
    """The first cut of this scan found NOTHING, because the FLOSS allowlist carried
    `LICENSE.TXT` — and atapi's header reads "DDK - see license.txt in the root dir".
    An allowance for the pointer excused the one file the scan exists to find."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: libs/thing
            name: Thing
            license: MIT
            license_note: test
            reason: test
    """,
    )
    (tmp_path / "libs" / "thing").mkdir(parents=True)
    (tmp_path / "drivers").mkdir()
    (tmp_path / "drivers" / "d.c").write_text(
        "/*\n * LICENSE:  DDK - see license.txt in the root dir\n */\n"
    )
    rows = vendored.declared_licences(vendored.load(tmp_path))
    assert any("DDK" in v for v, _e, _n, _d in rows), rows


def test_a_registered_tree_is_not_re_reported(tmp_path):
    """Recording the terms in vendor.yaml is one of the three ways to settle a hit,
    so a registered tree must stop firing or the ratchet cannot be satisfied."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: drivers/weird
            name: Weird
            license: Some-Proprietary-EULA
            license_note: recorded on purpose
            reason: test
    """,
    )
    d = tmp_path / "drivers" / "weird"
    d.mkdir(parents=True)
    (d / "d.c").write_text("/*\n * LICENSE:  DDK - see license.txt\n */\n")
    assert vendored.declared_licences(vendored.load(tmp_path)) == []


def test_reactos_own_gpl_headers_do_not_fire():
    """`See COPYING in the top level directory` is ReactOS's own GPL and the file is
    actually at the repo root — ~20 files say it, and a ratchet that fired on all of
    them on day one would have been turned off on day two."""
    assert (REPO / "COPYING").exists()
    reg = vendored.load(REPO)
    assert not any("COPYING" in v for v, _e, _n, _d in vendored.declared_licences(reg))


# ── the grant ratchet: what counts as a grant, and what only looks like one ──


def _grant_repo(tmp: Path) -> Path:
    # One registered tree, because an empty `trees:` is itself a registry error.
    _write(
        tmp,
        """
        containers: []
        trees:
          - path: libs/x
            name: X
            license: MIT
            license_ref: upstream README
            reason: test
    """,
    )
    (tmp / "libs" / "x").mkdir(parents=True)
    (tmp / "COPYING").write_text("GNU GENERAL PUBLIC LICENSE Version 2\n")
    (tmp / "COPYING.LIB").write_text("GNU LESSER GENERAL PUBLIC LICENSE\n")
    return tmp


def _claim(d: Path, name: str, note: str) -> None:
    """Three files claiming a copyright -- `min_files` is 3."""
    d.mkdir(parents=True, exist_ok=True)
    for i in range(3):
        (d / f"{name}{i}.c").write_text(f"/*\n * Copyright (c) 2003 Someone\n * {note}\n */\n")


def test_a_pointer_to_a_todo_list_is_not_a_grant(tmp_path):
    """The laundering hole, caught before it shipped. The pointer regex accepts any
    bare `*.txt` so that libjpeg's and the NT4 DDK's references parse; uniata's
    id_ata.cpp says "see todo.txt", todo.txt is right there, and treating that as
    coverage excused 36,368 lines whose only licence content is a disclaimer."""
    repo = _grant_repo(tmp_path)
    d = repo / "drivers" / "uniata"
    _claim(d, "id_ata", "Notes: see todo.txt for what is left")
    (d / "todo.txt").write_text("- make it faster\n")
    no_grant, _ = vendored.grant_gaps(vendored.load(repo))
    assert [x[0] for x in no_grant] == ["drivers/uniata"]


def test_a_prose_pointer_to_a_real_licence_is_a_grant(tmp_path):
    """schilytools spells its CDDL notice as prose -- "See the file CDDL.Schily.txt
    in this distribution for details" -- with no ReactOS-style header to hang it on."""
    repo = _grant_repo(tmp_path)
    (repo / "schilytools").mkdir()
    (repo / "schilytools" / "CDDL.Schily.txt").write_text("COMMON DEVELOPMENT AND DISTRIBUTION\n")
    _claim(
        repo / "schilytools" / "libschily",
        "cvmod",
        "See the file CDDL.Schily.txt in this distribution for details.",
    )
    no_grant, dangling = vendored.grant_gaps(vendored.load(repo))
    assert no_grant == []
    assert dangling == {}


def test_a_sentence_ending_full_stop_is_not_part_of_the_filename(tmp_path):
    """The FSF's own boilerplate ends the sentence: "see the file COPYING.  If not,
    write to ...". Greedy name matching ate the stop and reported `copying.` as a
    dangling pointer across 40 files."""
    repo = _grant_repo(tmp_path)
    _claim(repo / "app", "gdb2", "along with this program; see the file COPYING.  If not,")
    no_grant, dangling = vendored.grant_gaps(vendored.load(repo))
    assert dangling == {}
    assert no_grant == []


def test_the_pointer_the_tree_ships_under_another_name_resolves(tmp_path):
    """ReactOS's files have said "See LGPL.txt in the top level directory" since
    before the file was called COPYING.LIB. The text is there; the name is not."""
    repo = _grant_repo(tmp_path)
    _claim(repo / "dll" / "psapi", "psapi", "LICENSE: See LGPL.txt in the top level directory")
    _, dangling = vendored.grant_gaps(vendored.load(repo))
    assert dangling == {}


def test_an_alias_cannot_invent_a_grant(tmp_path):
    """...but only because COPYING.LIB is actually there. Remove it and the
    reference is dangling again -- an alias renames a text, it does not supply one."""
    repo = _grant_repo(tmp_path)
    (repo / "COPYING.LIB").unlink()
    _claim(repo / "dll" / "psapi", "psapi", "LICENSE: See LGPL.txt in the top level directory")
    _, dangling = vendored.grant_gaps(vendored.load(repo))
    assert "lgpl.txt" in dangling


def test_the_pointer_may_sit_on_the_copyright_line(tmp_path):
    """ReactOS's house header puts it there and leaves LICENSE: off entirely, which
    is how hdaudbus read as ungranted while its grant was the GPL at the repo root,
    exactly where it pointed."""
    repo = _grant_repo(tmp_path)
    _claim(
        repo / "drivers" / "hdaudbus", "hda", "COPYRIGHT: See COPYING in the top level directory"
    )
    no_grant, _ = vendored.grant_gaps(vendored.load(repo))
    assert no_grant == []


def test_a_component_licence_covers_its_subtree_but_a_collection_one_does_not(tmp_path):
    """ncftp ships the Clarified Artistic License one level above libncftp: someone
    put it there for that component. `modules/rosapps/COPYING` is the opposite --
    the collection's GPL, sitting three levels above Ken Kato's Virtual Floppy
    Drive the way the repo-root COPYING sits above everything. Letting it count
    made 33 files of third-party code with no grant of their own vanish."""
    repo = _grant_repo(tmp_path)
    (repo / "modules" / "rosapps").mkdir(parents=True)
    (repo / "modules" / "rosapps" / "COPYING").write_text("GNU GENERAL PUBLIC LICENSE\n")
    _claim(repo / "modules" / "rosapps" / "drivers" / "vfd", "vfd", "Virtual Floppy Drive")

    ncftp = repo / "modules" / "rosapps" / "applications" / "ncftp"
    ncftp.mkdir(parents=True)
    (ncftp / "LICENSE.txt").write_text("The Clarified Artistic License\n")
    _claim(ncftp / "libncftp", "cmds", "NCEMRSoft. All rights reserved.")

    no_grant, _ = vendored.grant_gaps(vendored.load(repo))
    assert [x[0] for x in no_grant] == ["modules/rosapps/drivers/vfd"]


def test_a_registered_tree_may_keep_its_licence_text_elsewhere(tmp_path):
    """libpng's png.h says "see LICENSE" from the copy of its headers we install
    under sdk/include; the text ships with the sources in dll/3rdparty. A recipient
    can read the terms, which is the only thing this check asks."""
    repo = _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: dll/3rdparty/libpng
            name: libpng
            license: libpng-2.0
            license_file: dll/3rdparty/libpng/LICENSE
            headers:
              - sdk/include/libs/libpng
            reason: test
    """,
    )
    (repo / "dll" / "3rdparty" / "libpng").mkdir(parents=True)
    (repo / "dll" / "3rdparty" / "libpng" / "LICENSE").write_text("PNG Reference Library\n")
    inc = repo / "sdk" / "include" / "libs" / "libpng"
    inc.mkdir(parents=True)
    (inc / "png.h").write_text("/* Copyright (c) 2024 PNG authors, see LICENSE */\n")
    _, dangling = vendored.grant_gaps(vendored.load(repo))
    assert dangling == {}


def test_the_real_tree_still_reports_the_finding_that_needs_a_decision(tmp_path):
    """A live check that the ratchet has not been tuned into silence. kbdtool's
    COPYING.BSD -- a text ReactOS has never shipped, with no in-tree precedent for
    its wording -- is an open question, not a tool defect. uniata was the other
    one and it was RESOLVED by deleting the ungranted C, which is why no
    directory finding survives: if that list ever grows again, it is a real
    finding, not a regression in here."""
    no_grant, dangling = vendored.grant_gaps(vendored.load(REPO))
    assert no_grant == []
    assert "copying.bsd" in dangling


def test_a_new_grant_finding_fails_the_gate(tmp_path):
    """`--grants` was advisory for its whole life, which made every licence gap in
    the tree conditional on somebody remembering to run it."""
    repo = _grant_repo(tmp_path)
    _claim(repo / "drivers" / "mystery", "m", "Notes: THIS SOFTWARE IS PROVIDED AS IS")
    problems = vendored.check(vendored.load(repo))
    assert any("NEW licence-grant finding `dir:drivers/mystery`" in p for p in problems), problems


def test_the_real_tree_passes_its_own_grant_gate():
    """...and the two findings it does carry are accepted BY NAME, with the
    argument written down beside them."""
    assert not [p for p in vendored.check(vendored.load(REPO)) if "licence-grant" in p]
    assert set(vendored._GRANTS_ACCEPTED) == {"ptr:copying.bsd"}


def test_an_accepted_finding_that_got_fixed_fails_too(monkeypatch):
    """A tolerated finding that has quietly gone away is one nobody will re-read,
    so the gate runs in both directions."""
    monkeypatch.setitem(vendored._GRANTS_ACCEPTED, "dir:drivers/does/not/exist", "invented")
    problems = vendored.check(vendored.load(REPO))
    assert any("no longer reports it" in p for p in problems), problems


def test_every_committed_tree_carries_a_valid_translate_verdict():
    """The owner's per-tree call on whether we will own the code. A missing or
    misspelt value must not pass silently as a real one."""
    reg = vendored.load(REPO)
    assert all(t.translate in vendored.TRANSLATE_VALUES for t in reg.trees)
    by_path = {t.path: t for t in reg.trees}
    assert by_path["vendor/wdf"].translate == "planned"
    assert by_path["vendor/mbedtls"].translate == "keep"


def test_a_bare_no_is_rejected_because_yaml_reads_it_as_false(tmp_path):
    """`translate: no` parses as the boolean false, not the string "no". It shipped for
    one draft of this field and zeroed the `keep` count; check() must catch it."""
    _write(
        tmp_path,
        """
        containers: []
        trees:
          - path: lib
            name: Lib
            reason: test
            translate: no
    """,
    )
    (tmp_path / "lib").mkdir()
    (tmp_path / "lib" / "a.c").write_text("int a;\n")
    problems = vendored.check(vendored.load(tmp_path))
    assert any("translate: 'False'" in p for p in problems), problems
