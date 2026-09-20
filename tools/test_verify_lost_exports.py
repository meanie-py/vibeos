"""Controls for verify_lost_exports.

The tool is excluded from `test_scanner_controls.py` because it needs a matching .spec +
CMakeLists + crate layout rather than a lone source file — which is exactly why it went a long
time with no oracle at all while acquiring three separate blind spots, each of which reported a
SERVED export as lost:

  * a `rust_varargs_shim.c` the flip deliberately kept (the only place stable Rust can put a
    C-variadic entry point),
  * a `#[no_mangle]` separated from its `fn` by a comment,
  * a module whose sources are `.cpp`.

Widening a checker to clear false positives is the move most likely to switch it off entirely,
so every must-NOT-fire case below is paired with a must-fire one.
"""

from __future__ import annotations

import importlib.util
import pathlib
import sys

_MOD = pathlib.Path(__file__).resolve().parent / "verify_lost_exports.py"
_spec = importlib.util.spec_from_file_location("verify_lost_exports", _MOD)
assert _spec and _spec.loader
vle = importlib.util.module_from_spec(_spec)
sys.modules["verify_lost_exports"] = vle
_spec.loader.exec_module(vle)


def _module(tmp_path, *, spec: str, cmake: str, sources: dict[str, str], rust: dict[str, str]):
    ddir = tmp_path / "dll" / "win32" / "m"
    ddir.mkdir(parents=True)
    (ddir / "m.spec").write_text(spec)
    (ddir / "CMakeLists.txt").write_text(cmake)
    for name, body in sources.items():
        path = ddir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
    crate = tmp_path / "sdk" / "lib" / "rust" / "m-rust" / "src"
    crate.mkdir(parents=True)
    for name, body in rust.items():
        (crate / name).write_text(body)
    return ddir, crate


FLIPPED_CMAKE = "list(APPEND SOURCE\n#    impl.c\n    other.c)\n"


def test_reports_a_genuinely_dropped_export(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": "// nothing defines Gone\n"},
    )
    lost, _ = vle.audit("m", ddir, crate)
    assert [s for s, _ in lost] == ["Gone"]


def test_rust_definition_clears_it(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": '#[no_mangle]\npub unsafe extern "system" fn Gone() {}\n'},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_no_mangle_separated_by_a_comment_still_counts(tmp_path):
    # snmpapi's SnmpUtilDbgPrint carries a six-line note between the attribute and the fn.
    ddir, crate = _module(
        tmp_path,
        spec="@ varargs Gone(long str)\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={
            "lib.rs": "#[no_mangle]\n// why this is fixed-arity\n// second line\n"
            'pub unsafe extern "C" fn Gone() {}\n'
        },
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_a_live_c_serving_the_export_clears_it(tmp_path):
    # The varargs-shim shape: the entry point stays in C because stable Rust cannot define one.
    ddir, crate = _module(
        tmp_path,
        spec="@ varargs Gone(long str)\n",
        cmake="list(APPEND SOURCE\n#    impl.c\n    rust_varargs_shim.c)\n",
        sources={
            "impl.c": "VOID\nGone(void)\n{\n}\n",
            "rust_varargs_shim.c": "void CDECL Gone(int a, ...)\n{\n    Gone_v(a);\n}\n",
        },
        rust={"lib.rs": "// only the _v twin lives here\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_a_comment_naming_a_live_file_does_not_mark_it_flipped(tmp_path):
    # A flip's comment block explains which .c stayed; reading that as the fact declared the
    # live file flipped and re-reported the export it serves.
    ddir, crate = _module(
        tmp_path,
        spec="@ varargs Gone(long str)\n",
        cmake=(
            "# rust_varargs_shim.c is NOT a translation reverted to C.\n"
            "list(APPEND SOURCE\n#    impl.c\n    rust_varargs_shim.c)\n"
        ),
        sources={
            "impl.c": "VOID\nGone(void)\n{\n}\n",
            "rust_varargs_shim.c": "void CDECL Gone(int a, ...)\n{\n}\n",
        },
        rust={"lib.rs": "// nothing\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_cpp_sources_are_read(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake="list(APPEND SOURCE\n#    impl.cpp\n    live.cpp)\n",
        sources={
            "impl.cpp": "VOID\nGone(void)\n{\n}\n",
            "live.cpp": "HRESULT WINAPI Gone(void)\n{\n    return 0;\n}\n",
        },
        rust={"lib.rs": "// nothing\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_a_prototype_is_not_a_definition(tmp_path):
    # kernel32's Get/SetFirmwareEnvironmentVariableEx* are DECLARED and defined nowhere; the
    # `{` requirement is what keeps them off the report.
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void);\n", "other.c": "int x;\n"},
        rust={"lib.rs": "// nothing\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_dead_preprocessor_branch_is_not_a_definition(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "#if 0\nVOID\nGone(void)\n{\n}\n#endif\n", "other.c": "int x;\n"},
        rust={"lib.rs": "// nothing\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_a_call_inside_a_live_function_does_not_serve_the_export(tmp_path):
    # The loosened definition regex must not let an ordinary call clear a real finding.
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={
            "impl.c": "VOID\nGone(void)\n{\n}\n",
            "other.c": "void caller(void)\n{\n    Gone();\n}\n",
        },
        rust={"lib.rs": "// nothing\n"},
    )
    assert [s for s, _ in vle.audit("m", ddir, crate)[0]] == ["Gone"]


def test_forwarded_export_is_served(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone() Real\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": "// nothing\n"},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_no_mangle_on_the_SAME_LINE_as_the_fn_still_counts(tmp_path):
    """`#[no_mangle] pub unsafe extern "system" fn Gone()` — attribute and fn on ONE line.

    Found 2026-09-19 flipping shell32's utils.cpp: that file writes every export this way, so
    the lint called SHOpenEffectiveToken (SHELL32.235) a LOST export while utils.rs was
    defining it. A sweep of sdk/lib/rust found 209 such definitions across 46 files invisible
    to the old spelling, each one a false LOST export waiting for its .c to be commented out.
    """
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": '#[no_mangle] pub unsafe extern "system" fn Gone() {}\n'},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_same_line_no_mangle_with_a_trailing_comment_still_counts(tmp_path):
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": '#[no_mangle] // SHELL32.235\npub unsafe extern "system" fn Gone() {}\n'},
    )
    assert vle.audit("m", ddir, crate)[0] == []


def test_a_bare_attribute_with_no_fn_after_it_is_still_not_a_definition(tmp_path):
    """Widening the attribute/fn gap must not start matching an attribute on its own."""
    ddir, crate = _module(
        tmp_path,
        spec="@ stdcall Gone()\n",
        cmake=FLIPPED_CMAKE,
        sources={"impl.c": "VOID\nGone(void)\n{\n}\n", "other.c": "int x;\n"},
        rust={"lib.rs": "#[no_mangle]\npub static Gone: u32 = 0;\n"},
    )
    assert [s for s, _ in vle.audit("m", ddir, crate)[0]] == ["Gone"]
