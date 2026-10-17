"""tools/strip_msvc_cmake.py: resolve MSVC conditions, touch nothing else."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import strip_msvc_cmake as smc

_CLANG_AMD64 = '((CMAKE_C_COMPILER_ID STREQUAL "Clang") AND (ARCH STREQUAL "amd64"))'


@pytest.mark.parametrize(
    ("cond", "value", "text"),
    [
        ("MSVC", False, None),
        ("NOT MSVC", True, None),
        ('CMAKE_C_COMPILER_ID STREQUAL "MSVC"', False, None),
        ('MSVC AND ARCH STREQUAL "i386"', False, None),
        ('MSVC OR ARCH STREQUAL "i386"', None, 'ARCH STREQUAL "i386"'),
        ('NOT (MSVC AND CMAKE_C_COMPILER_ID STREQUAL "Clang")', True, None),
        (
            f"MSVC OR USE_NATIVE_SEH OR {_CLANG_AMD64}",
            None,
            f"USE_NATIVE_SEH OR {_CLANG_AMD64}",
        ),
        (
            'NOT MSVC AND NOT CMAKE_C_COMPILER_ID STREQUAL "Clang"',
            None,
            'NOT CMAKE_C_COMPILER_ID STREQUAL "Clang"',
        ),
        ("NOT DEFINED MSVC_STATIC_RUNTIME AND MSVC", False, None),
        ('ARCH STREQUAL "i386"', None, None),  # no fact in it: untouched
        ('CMAKE_C_COMPILER_ID STREQUAL "Clang"', None, None),
    ],
)
def test_conditions_are_partially_evaluated(cond, value, text):
    assert smc.evaluate(cond) == (value, text)


def test_a_false_block_disappears_with_its_msvc_comment():
    src = "a()\n\n# MSVC needs this\nif(MSVC)\n    b()\nendif()\n\nc()\n"
    assert smc.rewrite(src) == "a()\n\nc()\n"


def test_an_attached_comment_goes_with_a_deleted_block_whatever_it_says():
    src = "a()\n\n# Newer VS versions link to _ftol2\nif(MSVC AND X)\n    b()\nendif()\n"
    assert smc.rewrite(src) == "a()\n"


def test_a_detached_comment_stays():
    src = "# Section header\n\nif(MSVC)\n    b()\nendif()\nc()\n"
    assert smc.rewrite(src) == "# Section header\n\nc()\n"


def test_a_kept_body_loses_only_an_msvc_comment():
    src = (
        "# clang-cl defines this one for itself\nif(NOT (MSVC AND Y))\n    a()\nendif()\n"
        "# Activate assembly\nif(MSVC)\n    m()\nelse()\n    g()\nendif()\n"
    )
    assert smc.rewrite(src) == "a()\n# Activate assembly\ng()\n"


def test_the_else_arm_wins_and_is_dedented():
    src = "if(MSVC)\n    a()\nelse()\n    b()\n    if(X)\n        c()\n    endif()\nendif()\n"
    assert smc.rewrite(src) == "b()\nif(X)\n    c()\nendif()\n"


def test_not_msvc_keeps_its_body_at_the_enclosing_indent():
    src = "if(Y)\n    if (NOT MSVC)\n        a()\n    endif()\nendif()\n"
    assert smc.rewrite(src) == "if(Y)\n    a()\nendif()\n"


def test_a_false_if_promotes_its_elseif():
    src = 'if(MSVC)\n  a()\nelseif(ARCH STREQUAL "arm")\n  b()\nelse()\n  c()\nendif()\n'
    assert smc.rewrite(src) == 'if(ARCH STREQUAL "arm")\n  b()\nelse()\n  c()\nendif()\n'


def test_a_false_elseif_is_dropped_and_its_neighbours_kept_verbatim():
    src = "if(A)\n  a()\nelseif(MSVC)\n  b()\nelse()\n  c()\nendif()\n"
    assert smc.rewrite(src) == "if(A)\n  a()\nelse()\n  c()\nendif()\n"


def test_a_compound_condition_is_rewritten_not_removed():
    src = 'if(MSVC OR ARCH STREQUAL "i386")\n    a()\nendif()\n'
    assert smc.rewrite(src) == 'if(ARCH STREQUAL "i386")\n    a()\nendif()\n'


def test_a_multiline_condition_parses():
    src = 'if (NOT MSVC AND\n    (ARCH STREQUAL "i386"))\n    a()\nendif()\n'
    assert smc.rewrite(src) == 'if(ARCH STREQUAL "i386")\n    a()\nendif()\n'


def test_conditions_without_facts_are_untouched_byte_for_byte():
    src = "IF ( X )  # keep\n  a()\nELSE ()\n  b()\nENDIF ( X )\n"
    assert smc.rewrite(src) == src
    assert smc.pending(src) == 0


def test_check_counts_what_write_would_resolve(tmp_path):
    f = tmp_path / "CMakeLists.txt"
    f.write_text("if(MSVC)\n  a()\nendif()\nif(NOT MSVC)\n  b()\nendif()\n")
    assert smc.pending(f.read_text()) == 2
    assert smc.main(["--write", str(f)]) == 0
    assert f.read_text() == "b()\n"
    assert smc.pending(f.read_text()) == 0
