"""The move plan is a reference list, so every test is about a reference it must NOT miss.

Its failure mode is silent: a missed reference is a build that breaks on one arch after
the move, or a bindgen island regenerated from nothing. Each test below is one spelling
of a reference that a plain full-path grep cannot see, and three of them are spellings
the first cut of the tool actually missed on the real tree.
"""

import sys
import textwrap
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import vendor_move_plan as vmp


def _repo(tmp: Path, files: dict[str, str]) -> Path:
    (tmp / "vendor.yaml").write_text(
        textwrap.dedent(
            """
            containers: []
            trees:
              - path: libs/foo
                name: Foo
                reason: test
              - path: sdk/include/GL
                name: GL headers
                reason: test
              - path: drivers/bus/acpi/acpica
                name: ACPICA
                reason: test
            """
        )
    )
    for rel, body in files.items():
        (tmp / rel).parent.mkdir(parents=True, exist_ok=True)
        (tmp / rel).write_text(body)
    return tmp


def _refs(tmp: Path, tree: str) -> dict[str, set[str]]:
    return next(p for p in vmp.plan(tmp) if p.path == tree).refs


def test_relative_add_subdirectory_is_a_build_reference(tmp_path):
    """`add_subdirectory(foo)` never spells `libs/foo`."""
    repo = _repo(tmp_path, {"libs/foo/a.c": "", "libs/CMakeLists.txt": "add_subdirectory(foo)\n"})
    assert _refs(repo, "libs/foo")["build"] == {"libs/CMakeLists.txt"}


def test_a_cmake_fragment_resolves_against_its_ANCESTOR(tmp_path):
    """sdk/lib/crt/math/math.cmake lists `math/ieee754/x.c` relative to sdk/lib/crt,
    because it is include()d from there. Missed by the first cut."""
    repo = _repo(
        tmp_path,
        {"libs/foo/a.c": "", "libs/sub/frag.cmake": "list(APPEND SOURCE foo/a.c)\n"},
    )
    assert "libs/sub/frag.cmake" in _refs(repo, "libs/foo")["build"]


def test_a_path_assembled_from_a_variable_is_flagged_for_review(tmp_path):
    """acpica-sys's bindgen script says `${ACPI_DIR}/acpica/include`. Missed by the
    first cut; a full-path search structurally cannot see it."""
    repo = _repo(
        tmp_path,
        {
            "drivers/bus/acpi/acpica/a.c": "",
            "scripts/gen.sh": 'clang -I"${ACPI_DIR}/acpica/include" x.h\n',
        },
    )
    assert _refs(repo, "drivers/bus/acpi/acpica")["review"] == {"scripts/gen.sh"}


def test_an_include_root_consumer_is_counted(tmp_path):
    """`#include <GL/gl.h>` reaches sdk/include/GL through a root. Missed by the first
    cut, which reported zero consumers for the GL headers."""
    repo = _repo(
        tmp_path,
        {"sdk/include/GL/gl.h": "", "dll/x/x.c": "#include <GL/gl.h>\n"},
    )
    assert _refs(repo, "sdk/include/GL")["include"] == {"dll/x/x.c"}


def test_outbound_relative_include_is_found_and_inbound_self_reference_is_not(tmp_path):
    """A tree reaching OUT breaks on a move even when nothing references it; a tree
    naming itself moves with itself and is not a reference."""
    repo = _repo(
        tmp_path,
        {
            "libs/foo/a.c": '#include "../../common/util.h"\n#include "../foo/b.h"\n',
            "libs/foo/b.h": "",
            "libs/foo/CMakeLists.txt": "add_library(foo a.c)  # libs/foo\n",
        },
    )
    refs = _refs(repo, "libs/foo")
    assert refs["outbound"] == {"libs/foo/a.c"}
    assert not refs["build"]


def test_build_dirs_and_subagent_checkouts_are_never_read(tmp_path):
    """Fleet isolation: another checkout's references are not this tree's."""
    repo = _repo(
        tmp_path,
        {
            "libs/foo/a.c": "",
            "build-i386/libs/CMakeLists.txt": "add_subdirectory(foo)\n",
            ".claude/worktrees/x/libs/CMakeLists.txt": "add_subdirectory(foo)\n",
        },
    )
    refs = _refs(repo, "libs/foo")
    assert not refs["build"] and not refs["review"]
