import pytest

from rustconv.function_coverage import (
    CoverageError,
    require_c_plan,
    require_rust_definitions,
    rust_definitions,
)


def test_private_function_missing_from_plan():
    with pytest.raises(CoverageError, match="Lost"):
        require_c_plan("static void Lost(void) {}\nvoid Kept(void) {}", ["Kept"])


def test_foreign_declarations_and_comment_bodies_are_not_definitions():
    text = """extern "C" { fn Lost(); }
// fn Commented() {}
/* fn BlockCommented() {} */
const S: &str = "fn InString() {}";
pub unsafe extern "system" fn Kept() {}
"""
    assert rust_definitions(text) == {"Kept"}
    with pytest.raises(CoverageError, match="Lost"):
        require_rust_definitions(["Lost", "Kept"], text)


def test_missing_entire_output():
    with pytest.raises(CoverageError, match="Lost"):
        require_rust_definitions(["Lost"], "")


def test_macro_arguments_are_not_assumed_to_expand_to_definitions():
    assert rust_definitions("discard! { fn Lost() {} }") == set()


def test_final_assembler_refuses_a_definition_lost_during_postprocess(tmp_path, monkeypatch):
    from rustconv import assembler
    from rustconv.batch_runner import TranslationResult

    monkeypatch.setattr(assembler, "postprocess", lambda *a, **k: "// fn Lost() {}")
    result = TranslationResult(
        source_file="sample.c",
        rust_code="fn Lost() {}",
        model="test",
        input_tokens=0,
        output_tokens=0,
        success=True,
    )
    with pytest.raises(CoverageError, match="Lost"):
        assembler.assemble_file(
            result,
            "pilot",
            "void Lost(void) {}",
            output_dir=tmp_path,
            expected_definitions={"Lost"},
        )
    assert not (tmp_path / "src/pilot/sample.rs").exists()
