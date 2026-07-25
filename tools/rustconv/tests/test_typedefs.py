"""The pointer-typedef authority: which C typedefs a literal `NULL` may be spelled at.

`trivial_c` proves pointerness from the callee's signature, and for `_locale_t` the
signature has no star in it — the star is in a header. These tests pin what counts as
proof and, more importantly, what does NOT: a typedef this module rounds *up* to
"pointer" would put `core::ptr::null_mut()` at a position that is not one, on the path
that has no human reviewer.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.typedefs import (
    NULL_CONST,
    NULL_MUT,
    REPO_ROOT,
    pointer_typedefs,
    pointer_typedefs_for_source,
)


def scan(tmp_path: Path, text: str, name: str = "h.h") -> dict[str, str]:
    path = tmp_path / name
    path.write_text(text)
    return pointer_typedefs([path])


# --------------------------------------------------------------------------- #
# what counts as a pointer
# --------------------------------------------------------------------------- #


def test_plain_pointer_typedef_is_null_mut(tmp_path):
    assert scan(tmp_path, "typedef struct s *ps;") == {"ps": NULL_MUT}


def test_const_pointee_is_null_const(tmp_path):
    assert scan(tmp_path, "typedef const char *PCSTR;") == {"PCSTR": NULL_CONST}


def test_const_pointer_to_mutable_stays_null_mut(tmp_path):
    """`char *const` makes the POINTER const; the pointee is still mutable."""
    assert scan(tmp_path, "typedef char *const CPTR;") == {"CPTR": NULL_MUT}


def test_double_pointer_is_null_mut_even_with_const(tmp_path):
    """`const char **` points at `const char *`, which is itself mutable."""
    assert scan(tmp_path, "typedef const char **PPCSTR;") == {"PPCSTR": NULL_MUT}


def test_multi_declarator_typedef_splits_correctly(tmp_path):
    """The `_locale_t` shape itself: one struct, one value name, one pointer name."""
    found = scan(tmp_path, "typedef struct locale_s { int a; } _locale_tstruct, *_locale_t;")
    assert found == {"_locale_t": NULL_MUT}


def test_alias_of_a_pointer_typedef_resolves(tmp_path):
    found = scan(tmp_path, "typedef struct s *ps;\ntypedef ps handle_t;")
    assert found["handle_t"] == NULL_MUT


def test_alias_chain_resolves_to_a_fixpoint(tmp_path):
    found = scan(tmp_path, "typedef struct s *a_t;\ntypedef a_t b_t;\ntypedef b_t c_t;")
    assert found["c_t"] == NULL_MUT


# --------------------------------------------------------------------------- #
# what must REFUSE
# --------------------------------------------------------------------------- #


def test_function_pointer_typedef_is_refused(tmp_path):
    """Its Rust is `Option<unsafe extern "C" fn()>`, whose null is `None`, not null_mut()."""
    assert scan(tmp_path, "typedef void (*fn_t)(void);") == {}


def test_array_typedef_is_refused(tmp_path):
    assert scan(tmp_path, "typedef int arr_t[4];") == {}


def test_non_pointer_typedef_is_not_reported(tmp_path):
    assert scan(tmp_path, "typedef unsigned int uint_t;") == {}


def test_alias_of_a_non_pointer_is_refused(tmp_path):
    assert scan(tmp_path, "typedef unsigned int uint_t;\ntypedef uint_t b_t;") == {}


def test_alias_of_an_unknown_type_is_refused(tmp_path):
    """Nothing in the closure defines `mystery_t`, so `b_t` stays unproven."""
    assert scan(tmp_path, "typedef mystery_t b_t;") == {}


def test_disagreement_across_headers_drops_the_name(tmp_path):
    """Header order is preprocessor state, and we do not run the preprocessor."""
    (tmp_path / "a.h").write_text("typedef struct s *conflicted;")
    (tmp_path / "b.h").write_text("typedef unsigned int conflicted;")
    found = pointer_typedefs([tmp_path / "a.h", tmp_path / "b.h"])
    assert "conflicted" not in found


def test_disagreement_on_constness_drops_the_name(tmp_path):
    (tmp_path / "a.h").write_text("typedef const char *maybe;")
    (tmp_path / "b.h").write_text("typedef char *maybe;")
    found = pointer_typedefs([tmp_path / "a.h", tmp_path / "b.h"])
    assert "maybe" not in found


def test_identical_redefinitions_are_not_a_disagreement(tmp_path):
    """The `#ifndef X_DEFINED` idiom repeats a typedef verbatim across headers."""
    (tmp_path / "a.h").write_text("typedef struct s *ps;")
    (tmp_path / "b.h").write_text("typedef struct s *ps;")
    assert pointer_typedefs([tmp_path / "a.h", tmp_path / "b.h"]) == {"ps": NULL_MUT}


def test_typedef_in_a_dead_preproc_branch_is_not_evidence(tmp_path):
    """`#ifndef __REACTOS__` never compiles here, so it cannot prove anything."""
    found = scan(
        tmp_path,
        "#ifndef __REACTOS__\ntypedef struct s *wine_only_t;\n#endif\n",
    )
    assert "wine_only_t" not in found


def test_a_dead_branch_cannot_veto_a_live_definition(tmp_path):
    """The dead arm is skipped outright, so it never registers as a disagreement."""
    found = scan(
        tmp_path,
        "typedef struct s *ps;\n#ifndef __REACTOS__\ntypedef unsigned int ps;\n#endif\n",
    )
    assert found == {"ps": NULL_MUT}


# --------------------------------------------------------------------------- #
# against the real tree
# --------------------------------------------------------------------------- #

MBCS = REPO_ROOT / "dll/win32/msvcrt/mbcs.c"


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_locale_t_is_proven_from_msvcrts_real_include_closure():
    """The single typedef this whole module exists for — 145 lowerings ride on it."""
    found = pointer_typedefs_for_source(MBCS)
    assert found["_locale_t"] == NULL_MUT
    assert found["pthreadlocinfo"] == NULL_MUT


@pytest.mark.skipif(not MBCS.is_file(), reason="msvcrt sources not present")
def test_generated_header_types_stay_unproven():
    """Documented limitation, pinned so widening the search path is a DELIBERATE act.

    `psdk/winnt.h` is generated into `build-<arch>/`, so `HANDLE` is not on the
    source-only path. Failing closed here costs nothing measurable (every refused
    NULL position in msvcrt is `_locale_t`) and keeps the verdict independent of
    whether a build dir happens to exist.
    """
    found = pointer_typedefs_for_source(MBCS)
    assert "HANDLE" not in found
