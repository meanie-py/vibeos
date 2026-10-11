from rustconv.postprocess import fix_debug_macros
from rustconv.function_coverage import rust_definitions


def test_debug_calls_keep_side_effects_and_following_function_bodies():
    source = "fn A() {\nDPRINT1(\n next());\n}\nfn B() { work(); }"
    result = fix_debug_macros(source)
    assert "DPRINT1!(\n next());" in result
    assert rust_definitions(result) == {"A", "B"}


def test_unimplemented_is_a_diagnostic_not_a_rust_panic():
    result = fix_debug_macros("fn A() { UNIMPLEMENTED; UNIMPLEMENTED!(); }")
    assert result.count("UNIMPLEMENTED!();") == 2
    assert "unimplemented!" not in result


def test_strings_and_macro_definitions_are_opaque():
    source = 'const X: &str = "DPRINT1(x)"; macro_rules! M { () => { DPRINT1(x) }; }'
    assert fix_debug_macros(source) == source


def test_a_bare_unimplemented_once_becomes_a_macro_call():
    from rustconv.debug_macros import preserve_debug_macros

    # user32_vista's dpi.c: `UNIMPLEMENTED_ONCE;` stayed a bare path, rustc E0423.
    assert "UNIMPLEMENTED_ONCE!();" in preserve_debug_macros("fn A() { UNIMPLEMENTED_ONCE; }")
