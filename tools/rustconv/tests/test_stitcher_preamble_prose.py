"""stitcher: narration in a preamble reply becomes a comment, never code (hh, 2026-10-10)."""

from rustconv.stitcher import comment_preamble_prose


def test_a_narrating_line_is_commented_and_items_survive():
    body = (
        "The only file-scope declaration is a function-pointer typedef. Since it is a typedef of "
        'a function type (not a struct), it maps to a type alias for an `extern "system"` pointer.\n'
        "\n"
        'pub type DOWINMAIN = unsafe extern "system" fn(hMod: HMODULE, cmdline: LPSTR) -> i32;\n'
        "static T: [u8; 2] = [\n    1, 2,\n];\n"
        "Static tables follow below.\n"
    )
    out = comment_preamble_prose(body)
    assert out.splitlines()[0].startswith("// The only file-scope declaration")
    assert "\npub type DOWINMAIN = unsafe extern" in out
    assert "static T: [u8; 2] = [\n    1, 2,\n];" in out
    assert out.splitlines()[-1] == "// Static tables follow below."


def test_code_is_left_alone():
    body = "pub const A: u32 = 1;\nstatic mut B: [u16; 1] = [0];\nmacro_rules! X { () => {}; }\n"
    assert comment_preamble_prose(body) == body.rstrip("\n")
