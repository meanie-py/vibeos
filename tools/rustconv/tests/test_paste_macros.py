"""wined3d's GL_EXTCALL(f) pastes `p_##f` onto the caller's gl_info: 590 Rust call sites."""

from rustconv.local_macros import CMacro
from rustconv.paste_macros import expand_member_paste_calls, member_paste_macros


def _defs(**bodies: tuple[tuple[str, ...] | None, str]) -> dict[str, CMacro]:
    return {n: CMacro(n, params, body, "x.h") for n, (params, body) in bodies.items()}


def test_only_a_member_chain_pasted_with_the_parameter_qualifies():
    got = member_paste_macros(
        _defs(
            GL_EXTCALL=(("f",), "(gl_info->gl_ops.ext.p_##f)"),
            KEY=(("a",), "KEY_##a"),  # no member chain: nothing to say about the receiver
            TWO=(("a", "b"), "x->a##b"),
            OBJ=(None, "x->y"),
        )
    )
    assert got == {"GL_EXTCALL": "(*gl_info).gl_ops.ext.p_"}


def test_call_sites_expand_to_the_entry_point_field():
    m = {"GL_EXTCALL": "(*gl_info).gl_ops.ext.p_"}
    text, n, odd = expand_member_paste_calls(
        'GL_EXTCALL(glActiveTexture(GL_TEXTURE0 + i));\nif GL_EXTCALL!(glFoo).is_some() {}\n'
        'GL_EXTCALL(glBar(f(1), "("));\nGL_EXTCALL(a + b);\n',
        m,
    )
    assert n == 3 and odd == ["GL_EXTCALL(a + b)"]
    assert "((*gl_info).gl_ops.ext.p_glActiveTexture.unwrap_unchecked())(GL_TEXTURE0 + i);" in text
    assert "if (*gl_info).gl_ops.ext.p_glFoo.is_some()" in text
    assert '((*gl_info).gl_ops.ext.p_glBar.unwrap_unchecked())(f(1), "(");' in text
