"""A preamble reply that is only prose must never be stitched in as code."""

from rustconv.__main__ import _comment_out_prose_preamble

# imm32's candidate.c, verbatim: one paragraph and no Rust.
PROSE = (
    "The preamble contains one object-like constant-free macro (`ROUNDUP4` is function-like, "
    "so it is skipped) and the `WINE_DEFAULT_DEBUG_CHANNEL` macro invocation. Neither "
    "produces a Rust declaration, so nothing is emitted here."
)


def test_an_all_prose_reply_becomes_comments():
    out = _comment_out_prose_preamble(PROSE)
    assert out.startswith("// The preamble") and "\n" not in out


def test_real_items_pass_through_untouched():
    for rust in (
        "pub static mut ghImmHeap: HANDLE = core::ptr::null_mut();",
        "// WINE_DEFAULT_DEBUG_CHANNEL(imm): no item\n",
        "#[repr(C)]\npub struct A {\n    pub x: u32,\n}\nconst B: u32 = 1;",
        "",
    ):
        assert _comment_out_prose_preamble(rust) == rust


def test_prose_mixed_into_items_is_left_for_the_stitch_to_reject():
    mixed = "Here is the preamble:\npub const X: u32 = 1;"
    assert _comment_out_prose_preamble(mixed) == mixed
