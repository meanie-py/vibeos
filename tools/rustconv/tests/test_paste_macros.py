"""wined3d's GL_EXTCALL(f) pastes `p_##f` onto the caller's gl_info: 590 Rust call sites."""

from rustconv.local_macros import CMacro
from rustconv.paste_macros import expand_member_paste_calls, member_paste_macros


def _defs(**bodies: tuple[tuple[str, ...] | None, str]) -> dict[str, CMacro]:
    return {n: CMacro(n, params, body, "x.h") for n, (params, body) in bodies.items()}


def test_only_a_member_chain_pasted_with_the_parameter_qualifies():
    got = member_paste_macros(
        _defs(
            GL_EXTCALL=(("f",), "(gl_info->gl_ops.ext.p_##f)"),
            VK_CALL=(("f",), "(vk_info->vk_ops.f)"),
            BARE=(("f",), "(f)"),
            KEY=(("a",), "KEY_##a"),  # no member chain: nothing to say about the receiver
            TWO=(("a", "b"), "x->a##b"),
            OBJ=(None, "x->y"),
        )
    )
    assert got == {"GL_EXTCALL": "(*gl_info).gl_ops.ext.p_", "VK_CALL": "(*vk_info).vk_ops."}


def test_call_sites_expand_to_the_entry_point_field():
    m = {"GL_EXTCALL": "(*gl_info).gl_ops.ext.p_"}
    text, n, odd = expand_member_paste_calls(
        "GL_EXTCALL(glActiveTexture(GL_TEXTURE0 + i));\nif GL_EXTCALL!(glFoo).is_some() {}\n"
        'GL_EXTCALL(glBar(f(1), "("));\nGL_EXTCALL(a + b);\n',
        m,
    )
    assert n == 3 and odd == ["GL_EXTCALL(a + b)"]
    assert "((*gl_info).gl_ops.ext.p_glActiveTexture.unwrap_unchecked())(GL_TEXTURE0 + i);" in text
    assert "if (*gl_info).gl_ops.ext.p_glFoo.is_some()" in text
    assert '((*gl_info).gl_ops.ext.p_glBar.unwrap_unchecked())(f(1), "(");' in text


def test_a_trailing_comma_and_a_pre_expanded_path_are_the_same_call():
    m = {"VK_CALL": "(*vk_info).vk_ops."}
    text, n, odd = expand_member_paste_calls(
        "VK_CALL!(\n    vkGetInstanceProcAddr(\n        instance,\n    ),\n);\n"
        "VK_CALL!((*vk_info).vk_ops.vkCmdPipelineBarrier(cb, 1));\n",
        m,
    )
    assert n == 2 and odd == []
    assert "((*vk_info).vk_ops.vkGetInstanceProcAddr.unwrap_unchecked())(" in text
    assert "((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(cb, 1);" in text


def test_a_hand_port_declares_the_caller_locals_its_call_sites_pass():
    from rustconv.paste_macros import caller_locals, pass_caller_locals

    prelude = "// port: caller-locals checkGLcall(gl_info)\nmacro_rules! checkGLcall { }\n"
    locals_ = caller_locals(prelude)
    assert locals_ == {"checkGLcall": ["gl_info"]}
    text, n = pass_caller_locals(
        'checkGLcall("glBindTexture");\ncheckGLcall!(b"x\\0".as_ptr());\n'
        'checkGLcall!(gl_info, "kept");\nfoo.checkGLcall(1);\n',
        locals_,
    )
    assert n == 2
    assert 'checkGLcall!(gl_info, "glBindTexture");' in text
    assert 'checkGLcall!(gl_info, b"x\\0".as_ptr());' in text
    assert 'checkGLcall!(gl_info, "kept");' in text and "foo.checkGLcall(1);" in text
