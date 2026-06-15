"""Pure (no-LLM, no-network) behavior of the semdiff helpers.

semdiff's value is finding C↔Rust divergences via an LLM, but the parsing it
feeds the model is deterministic and worth pinning: which Rust items count as
'functions', and how the model's JSON reply is recovered from fenced/prose text.
"""

from rustconv.semdiff import _parse_json, extract_rust_functions


class TestExtractRustFunctions:
    def test_plain_definition_is_captured(self):
        fns = extract_rust_functions("pub fn add(a: u32, b: u32) -> u32 {\n    a + b\n}\n")
        assert set(fns) == {"add"}
        assert "a + b" in fns["add"].body_text
        assert fns["add"].line_start == 1

    def test_extern_declaration_without_body_is_skipped(self):
        # A decl (`fn Foo(...);`) hits ';' before any '{' → not a definition.
        text = 'extern "C" {\n    fn KeBugCheck(code: u32);\n}\n'
        assert extract_rust_functions(text) == {}

    def test_generic_signature_is_captured(self):
        fns = extract_rust_functions("fn wrap<T: Copy>(x: T) -> T {\n    x\n}\n")
        assert "wrap" in fns

    def test_multiple_functions_and_brace_matching(self):
        text = "fn a() {\n    if true { let _ = 1; }\n}\nfn b() {\n    loop { break; }\n}\n"
        fns = extract_rust_functions(text)
        assert set(fns) == {"a", "b"}
        # body brace-matching must not let a's body swallow b.
        assert "fn b" not in fns["a"].body_text

    def test_attributes_are_pulled_into_the_item(self):
        text = '#[no_mangle]\npub unsafe extern "C" fn Foo() {\n    ()\n}\n'
        body = extract_rust_functions(text)["Foo"].body_text
        assert "#[no_mangle]" in body  # ABI qualifiers must be visible to the diff

    def test_first_definition_wins_on_duplicate_names(self):
        text = "fn dup() {\n    1;\n}\nfn dup() {\n    2;\n}\n"
        assert extract_rust_functions(text)["dup"].body_text.count("1;") == 1


class TestParseJson:
    def test_bare_object(self):
        assert _parse_json('{"a": 1}') == {"a": 1}

    def test_fenced_json_block(self):
        assert _parse_json('```json\n{"ok": true}\n```') == {"ok": True}

    def test_leading_prose_then_object(self):
        assert _parse_json('Here is the result:\n{"x": [1, 2]}') == {"x": [1, 2]}

    def test_no_brace_returns_none(self):
        assert _parse_json("no json here") is None

    def test_invalid_json_returns_none(self):
        assert _parse_json("{not: valid, json}") is None

    def test_nested_object_brace_matching(self):
        assert _parse_json('{"a": {"b": 1}, "c": 2}') == {"a": {"b": 1}, "c": 2}
