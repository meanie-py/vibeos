"""`#if _MSVCR_VER >= 80` is not conditional — the module's CMake fixes it at 0.

msvcrt carries 284 `_MSVCR_VER` conditionals and builds every target with `_MSVCR_VER=0`.
Leaving them undecided cost real money and real correctness on mbcs.c: two functions under a
FALSE arm were extracted, paid for, and emitted `#[no_mangle]` though the C never compiles
them, and one statement under a TRUE arm came back as `#[cfg(0)]` — which neither compiles
nor keeps the behaviour.

The other half of these tests is the harder half: what must stay UNDECIDED. One Rust file
serves i386, amd64 and arm64, so an arch conditional is `#[cfg(target_arch)]`'s job. Folding
`defined(__i386__)` to 0 deleted ten live float functions from mathf.c on the first attempt.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from rustconv.scanner import _eval_pp_cond, _resolve_known_preproc, cmake_target_defines, scan_file

REPO_ROOT = Path(__file__).resolve().parents[3]
MSVCRT = REPO_ROOT / "dll/win32/msvcrt"

DEFS = {"_MSVCR_VER": "0", "WINVER": "0x601"}


# --------------------------------------------------------------------------- #
# reading the build's own -D flags
# --------------------------------------------------------------------------- #


def test_cmake_target_definitions_are_read(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text(
        "target_compile_definitions(msvcrt_static PRIVATE _MSVCR_VER=0)\n"
    )
    assert cmake_target_defines(tmp_path / "x.c") == {"_MSVCR_VER": "0"}


def test_add_definitions_form_is_read(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text("add_definitions(-D_MSVCR_VER=0 -DFOO=7)\n")
    assert cmake_target_defines(tmp_path / "x.c") == {"_MSVCR_VER": "0", "FOO": "7"}


def test_scope_keywords_and_target_names_are_not_macros(tmp_path):
    """`PRIVATE` and the target name are arguments, not definitions."""
    (tmp_path / "CMakeLists.txt").write_text(
        "target_compile_definitions(msvcrt PRIVATE _MSVCR_VER=0 __UCRTSUPPORT__)\n"
    )
    # __UCRTSUPPORT__ has no value, so it is indistinguishable from a target name here.
    assert cmake_target_defines(tmp_path / "x.c") == {"_MSVCR_VER": "0"}


def test_targets_disagreeing_on_a_value_drop_it(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text(
        "target_compile_definitions(a PRIVATE _MSVCR_VER=0)\n"
        "target_compile_definitions(b PRIVATE _MSVCR_VER=140)\n"
    )
    assert "_MSVCR_VER" not in cmake_target_defines(tmp_path / "x.c")


def test_cmake_comments_are_ignored(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text(
        "# target_compile_definitions(a PRIVATE _MSVCR_VER=140)\n"
        "target_compile_definitions(b PRIVATE _MSVCR_VER=0)\n"
    )
    assert cmake_target_defines(tmp_path / "x.c") == {"_MSVCR_VER": "0"}


def test_no_cmakelists_yields_nothing(tmp_path):
    """A module with no CMake definitions must behave exactly as before this existed."""
    assert cmake_target_defines(tmp_path / "x.c") == {}


# --------------------------------------------------------------------------- #
# deciding conditions
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    ("expr", "expected"),
    [
        ("_MSVCR_VER >= 80", False),
        ("_MSVCR_VER>=140", False),
        ("_MSVCR_VER > 100", False),
        ("_MSVCR_VER == 0", True),
        ("_MSVCR_VER <= 90", True),
        ("_MSVCR_VER >= 60 && _MSVCR_VER <= 71", False),
        ("WINVER >= 0x600", True),
    ],
)
def test_known_macros_decide(expr, expected):
    assert _eval_pp_cond("if", expr, DEFS) is expected


@pytest.mark.parametrize(
    "expr",
    [
        "defined(__i386__) || (_MSVCR_VER > 0 && _MSVCR_VER < 80)",  # mathf.c, the real one
        "defined(__x86_64__) && _MSVCR_VER>=140",
        "_MSVCR_VER >= 80 && SOMETHING_ELSE",
        "defined(_WIN64)",
    ],
)
def test_expressions_mentioning_an_unknown_symbol_stay_undecided(expr):
    """Undecided keeps BOTH arms verbatim. Guessing the unknown as 0 deletes live code."""
    assert _eval_pp_cond("if", expr, DEFS) is None


def test_no_defines_means_no_decisions():
    assert _eval_pp_cond("if", "_MSVCR_VER >= 80", {}) is None


# --------------------------------------------------------------------------- #
# resolving whole blocks
# --------------------------------------------------------------------------- #


def test_dead_arm_and_its_functions_are_removed():
    text = "int live(void){return 1;}\n#if _MSVCR_VER>=80\nint dead(void){return 2;}\n#endif\n"
    out = _resolve_known_preproc(text, DEFS)
    assert "live" in out and "dead" not in out


def test_live_arm_is_unwrapped_not_cfg_gated():
    text = "int f(void){\n#if _MSVCR_VER == 0\n  return 1;\n#endif\n  return 2;\n}\n"
    out = _resolve_known_preproc(text, DEFS)
    assert "return 1;" in out and "#if" not in out


def test_else_arm_is_taken_when_the_condition_is_false():
    text = (
        "#if _MSVCR_VER>=60 && _MSVCR_VER<=71\n"
        "int a(void){return 1;}\n"
        "#else\n"
        "int b(void){return 2;}\n"
        "#endif\n"
    )
    out = _resolve_known_preproc(text, DEFS)
    assert "int b" in out and "int a" not in out


def test_undecidable_block_is_left_completely_verbatim():
    text = "#if defined(__i386__)\nint f(void){return 1;}\n#endif\n"
    assert _resolve_known_preproc(text, DEFS) == text


# --------------------------------------------------------------------------- #
# against the real tree
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(not MSVCRT.is_dir(), reason="msvcrt sources not present")
def test_msvcrt_defines_msvcr_ver_zero():
    assert cmake_target_defines(MSVCRT / "mbcs.c")["_MSVCR_VER"] == "0"


@pytest.mark.skipif(not MSVCRT.is_dir(), reason="msvcrt sources not present")
def test_mbcs_dead_functions_are_not_scanned():
    """Both sit under `#if _MSVCR_VER>=80`; the C never compiles them."""
    names = {f.name for f in scan_file(MSVCRT / "mbcs.c").functions}
    assert "___mb_cur_max_l_func" not in names
    assert "_mbsncpy_s_l" not in names
    assert "_mbstok" in names  # the `== 0` arm is LIVE and must survive


@pytest.mark.skipif(not MSVCRT.is_dir(), reason="msvcrt sources not present")
def test_mathf_arch_gated_floats_survive():
    """`defined(__i386__) || ...` is undecidable, so all ten stay. Regression guard."""
    names = {f.name for f in scan_file(MSVCRT / "mathf.c").functions}
    for fn in ("sinf", "cosf", "tanf", "atan2f", "expf", "logf", "sqrtf", "floorf", "ceilf"):
        assert fn in names, f"{fn} was dropped — an arch conditional got folded away"


def test_a_module_header_no_tu_includes_has_only_dead_macros():
    from pathlib import Path

    from rustconv.port import _compiled_out

    fc_c, fc_h, noros = Path("/m/fc.c"), Path("/m/fc.h"), Path("/m/conutils_noros.h")
    where = {"MAX_VIEW_SIZE": fc_h, "ConPuts": noros, "IS_DOTS": fc_c, "OLD": fc_h}
    live = {fc_c: {"IS_DOTS"}, fc_h: {"MAX_VIEW_SIZE"}}
    assert _compiled_out(where, live, [fc_c]) == ["ConPuts", "OLD"]
    # A TU nobody preprocessed could be the one that includes it: undecidable, kept.
    assert _compiled_out(where, live, [fc_c, Path("/m/other.c")]) == ["OLD"]
