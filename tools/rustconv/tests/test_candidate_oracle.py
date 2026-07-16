from rustconv.candidate_oracle import (
    CandidateOracleError,
    CandidateSpec,
    build_oracle_crate,
    carry_deterministic_cpp_support,
    carry_deterministic_wide_statics,
    function_span,
    load_manifest,
    replace_function,
    worker_distance,
)


def test_carry_deterministic_cpp_com_ptr_guard_for_spliced_worker():
    base = "pub unsafe fn worker() { CppComPtrGuard::new(slot); }"
    candidate = """pub struct CppComPtrGuard<T> { slot: *mut *mut T }
impl<T> CppComPtrGuard<T> { pub const fn new(slot: *mut *mut T) -> Self { Self { slot } } }
impl<T> Drop for CppComPtrGuard<T> {
    fn drop(&mut self) { unsafe { if !(*self.slot).is_null() { *self.slot = core::ptr::null_mut(); } } }
}
pub unsafe fn worker() { CppComPtrGuard::new(slot); }
"""
    carried = carry_deterministic_cpp_support(base, candidate)
    assert carried.count("pub struct CppComPtrGuard") == 1
    assert "impl<T> Drop for CppComPtrGuard<T>" in carried
    assert carried.endswith(base)


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


def test_worker_distance_ignores_signature_and_whitespace():
    base = "pub unsafe fn body(x: u32) -> u32 { let y = x + 1; y }"
    same_body = "fn body(value: usize) {\nlet y=x+1; y\n}"
    changed = "fn body() { let y = x + 2; y }"
    assert worker_distance(base, same_body, "body")["edit_tokens"] == 0
    distance = worker_distance(base, changed, "body")
    assert distance["edit_tokens"] == 1
    assert distance["edit_hunks"] == 1


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


def test_generated_crate_maps_generated_worker_to_accepted_internal_name(tmp_path):
    types = tmp_path / "types.rs"
    shellcom = tmp_path / "shellcom.rs"
    types.write_text("pub type DWORD = u32;\n")
    shellcom.write_text("")
    base = tmp_path / "base.rs"
    base.write_text("unsafe fn load(value: DWORD) -> DWORD { value }")
    candidate = tmp_path / "candidate.rs"
    candidate.write_text(
        "pub unsafe fn C_Load_body(value: DWORD) -> DWORD { value.wrapping_add(1) }"
    )
    specs = [CandidateSpec("load", "C_Load_body", candidate, base, "load")]
    paths, _ = build_oracle_crate(specs, types, shellcom, tmp_path / "oracle")
    generated = next(iter(paths)).read_text()
    assert "unsafe fn load(value: DWORD) -> DWORD { value.wrapping_add(1) }" in generated
    assert "C_Load_body" not in generated


def test_generated_crate_accepts_full_module_without_accepted_function_base(tmp_path):
    types = tmp_path / "types.rs"
    shellcom = tmp_path / "shellcom.rs"
    types.write_text("pub type DWORD = u32;\n")
    shellcom.write_text("")
    candidate = tmp_path / "candidate.rs"
    candidate.write_text("use crate::types::*; pub unsafe fn body(x: DWORD) -> DWORD { x }")
    specs = [CandidateSpec("module", "", candidate, candidate, full_module=True)]
    paths, repairs = build_oracle_crate(specs, types, shellcom, tmp_path / "oracle")
    assert next(iter(paths)).read_text() == candidate.read_text()
    assert repairs == {"module": []}


def test_manifest_full_module_defaults_base_to_candidate(tmp_path):
    candidate = tmp_path / "candidate.rs"
    candidate.write_text("fn body() {}")
    types = tmp_path / "types.rs"
    shellcom = tmp_path / "shellcom.rs"
    types.write_text("")
    shellcom.write_text("")
    manifest = tmp_path / "module.json"
    manifest.write_text(
        '{"target":"shell32","types":"types.rs","shellcom":"shellcom.rs",'
        '"candidates":[{"name":"module","candidate":"candidate.rs",'
        '"full_module":true}]}'
    )
    specs, _, _ = load_manifest(manifest)
    assert specs == [CandidateSpec("module", "", candidate.resolve(), candidate.resolve(), None, True)]


def test_corpus_manifest_defaults_candidate_to_durable_candidates_dir(tmp_path):
    (tmp_path / "candidates").mkdir()
    candidate = tmp_path / "candidates/case-1.rs"
    base = tmp_path / "base.rs"
    candidate.write_text("fn body() {}")
    base.write_text("fn body() {}")
    types = tmp_path / "types.rs"
    shellcom = tmp_path / "shellcom.rs"
    types.write_text("")
    shellcom.write_text("")
    manifest = tmp_path / "corpus.json"
    manifest.write_text(
        '{"target":"shell32","types":"types.rs","shellcom":"shellcom.rs",'
        '"cases":[{"id":"case-1","function":"body","candidate_function":"body",'
        '"base":"base.rs"}]}'
    )
    specs, _, _ = load_manifest(manifest)
    assert specs == [CandidateSpec("case-1", "body", candidate.resolve(), base.resolve())]


def test_only_generated_wide_statics_are_carried_into_base():
    candidate = """pub static CPP_COM_WSTR_0123456789AB: [WCHAR; 2] = [65, 0];
pub static MODEL_DATA: [WCHAR; 2] = [66, 0];
fn body() {}
"""
    merged = carry_deterministic_wide_statics("fn base() {}\n", candidate)
    assert "CPP_COM_WSTR_0123456789AB" in merged
    assert "MODEL_DATA" not in merged
