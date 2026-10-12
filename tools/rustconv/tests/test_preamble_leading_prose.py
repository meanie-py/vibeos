import shutil

import pytest

from rustconv.__main__ import _strip_leading_prose

pytestmark = pytest.mark.skipif(shutil.which("rustfmt") is None, reason="needs rustfmt")


def test_a_sentence_before_the_first_item_is_dropped():
    """newdev's wizard.c preamble, as Haiku returned it twice."""
    reply = (
        "The preamble contains one file-scope declaration, `HANDLE hThread;`, which is a "
        "mutable global with no initializer.\n\n"
        "pub static mut hThread: HANDLE = core::ptr::null_mut();"
    )
    assert (
        _strip_leading_prose(reply) == "pub static mut hThread: HANDLE = core::ptr::null_mut();\n"
    )


def test_commentary_between_items_is_left_for_the_parse_gate():
    """mspaint's shape: the prose is not a prefix, so nothing is stripped and it still fails."""
    reply = (
        "pub static A: i32 = 1;\n"
        "Wait -- the rules require `B` to be const.\n"
        "pub const B: i32 = 2;\n"
    )
    assert _strip_leading_prose(reply) == reply


def test_a_clean_reply_is_unchanged():
    reply = "pub static mut X: u32 = 0;\n"
    assert _strip_leading_prose(reply) == reply
