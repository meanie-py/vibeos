"""The stitcher's `use` hoister must not reach into a block comment."""

from rustconv.stitcher import _extract_use_statements

TOOLBAR_DRAW_IMAGE = """unsafe extern "C" fn TOOLBAR_DrawImage() {
    use crate::prelude::BOOL;
    let c = '"'; /* a quote char must not open a string */
    if hot {
        /* if hot, attempt to draw with hot image list, if fails,
           use default image list */
        himl = hot_list();
    }
}"""


def test_a_comment_line_starting_with_use_stays_in_the_body():
    """toolbar.c: hoisting the comment's second line left the comment open and hid the
    other 168 functions of the file from the module."""
    uses, rest = _extract_use_statements(TOOLBAR_DRAW_IMAGE)
    assert uses == ["use crate::prelude::BOOL;"]
    assert "use default image list */" in rest
