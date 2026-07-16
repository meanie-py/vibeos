from rustconv.candidate_oracle import (
    CandidateOracleError,
    CandidateSpec,
    build_oracle_crate,
    function_span,
    replace_function,
)


def test_function_span_ignores_braces_in_strings_and_comments():
    rust = """pub unsafe fn body(x: u32) -> u32 {
        let _ = "}"; // }
        if x > 0 { x } else { 1 }
    }
    fn next() {}"""
    start, end = function_span(rust, "body")
    assert rust[start:end].rstrip().endswith("}")
    assert "fn next" not in rust[start:end]


def test_replace_function_keeps_base_neighbors():
    base = "fn before() {}\npub unsafe fn body(x: u32) -> u32 { x }\nfn after() {}"
    candidate = "pub unsafe fn body(x: u32) -> u32 { x + 1 }"
    merged = replace_function(base, candidate, "body")
    assert "fn before" in merged and "fn after" in merged
    assert "{ x + 1 }" in merged


def test_missing_function_fails_loudly():
    try:
        function_span("fn present() {}", "missing")
    except CandidateOracleError as error:
        assert "function not found" in str(error)
    else:
        raise AssertionError("missing function was silently accepted")


def test_generated_crate_batches_candidate_modules(tmp_path):
    types = tmp_path / "types.rs"
    shellcom = tmp_path / "shellcom.rs"
    types.write_text("pub type DWORD = u32;\n")
    shellcom.write_text("")
    base = tmp_path / "base.rs"
    base.write_text("use crate::types::*; pub unsafe fn body(x: DWORD) -> DWORD { x }")
    first = tmp_path / "first.rs"
    second = tmp_path / "second.rs"
    first.write_text("pub unsafe fn body(x: DWORD) -> DWORD { x + 1 }")
    second.write_text("pub unsafe fn body(x: DWORD) -> DWORD { x + 2 }")
    candidates = [
        CandidateSpec("first", "body", first, base),
        CandidateSpec("second", "body", second, base),
    ]
    paths, repairs = build_oracle_crate(candidates, types, shellcom, tmp_path / "oracle")
    assert set(paths.values()) == {"first", "second"}
    assert repairs == {}
    assert len(list((tmp_path / "oracle/src").glob("candidate_*.rs"))) == 2
