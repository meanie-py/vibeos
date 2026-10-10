"""A preamble reply that is prose is kept as a comment, never pasted in as code."""

from rustconv.stitcher import _prose_preamble_as_comment


def test_prose_reply_becomes_a_comment():
    reply = (
        "The preamble contains only `#undef` directives, which have no Rust equivalent "
        "and define no declarations, so there is nothing to emit."
    )
    out = _prose_preamble_as_comment(reply)
    assert out.startswith("// The preamble")
    assert "`" not in out.replace("// ", "", 1).split("\n")[0][:3]


def test_real_rust_is_untouched():
    reply = "pub const X: u32 = 1;\nstatic mut Y: u32 = 0;"
    assert _prose_preamble_as_comment(reply) == reply


def test_mixed_reply_stays_code_so_it_fails_where_it_is_seen():
    reply = "Here is the code:\npub const X: u32 = 1;"
    assert _prose_preamble_as_comment(reply) == reply
